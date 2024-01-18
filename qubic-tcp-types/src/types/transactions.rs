use std::{num::NonZeroUsize, ptr::read_unaligned, fmt::Debug};

use kangarootwelve::KangarooTwelve;
use qubic_types::{QubicId, Signature, QubicTxHash, traits::{ToBytes, FromBytes, GetSigner}, Nonce};

use crate::{MessageType, consts::NUMBER_OF_TRANSACTION_PER_TICK, utils::QubicRequest};

use super::{assets::IssueAssetInput, ContractIpoBid};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RawTransaction {
    pub from: QubicId,
    pub to: QubicId,
    pub amount: u64,
    pub tick: u32,
    pub input_type: u16,
    pub input_size: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RawCall<T: Copy> {
    pub tx: RawTransaction,
    pub input: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Call<T: Copy> {
    pub raw_call: RawCall<T>,
    pub signature: Signature
}

impl<T: Copy> QubicRequest for Call<T> {
    fn get_message_type() -> MessageType {
        MessageType::BroadcastTransaction
    }
}

impl<T: Copy> GetSigner for Call<T> {
    fn get_signer(&self) -> &QubicId {
        &self.raw_call.tx.from
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct TransferAssetInput {
    pub destination: QubicId
}


#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Transaction {
    pub raw_transaction: RawTransaction,
    pub signature: Signature,
}

impl GetSigner for Transaction {
    fn get_signer(&self) -> &QubicId {
        &self.raw_transaction.from
    }
}

set_message_type!(Transaction, MessageType::BroadcastTransaction);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct TransactionFlags([u8; NUMBER_OF_TRANSACTION_PER_TICK/8]);

impl TransactionFlags {
    pub fn all() -> Self {
        Self([0; NUMBER_OF_TRANSACTION_PER_TICK/8])
    }

    pub fn first(first: NonZeroUsize) -> Self {
        let mut flags = [0u8; NUMBER_OF_TRANSACTION_PER_TICK/8];
        let full = usize::from(first)/8;

        for i in 0..full {
            flags[i] = u8::MAX;
        }

        let remaining = usize::from(first)%8;
        
        for i in 0..remaining {
            flags[full-1] = 1 << i;
        }

        Self(flags)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestedTickTransactions {
    pub tick: u32,
    pub flags: TransactionFlags
}

set_message_type!(RequestedTickTransactions, MessageType::RequestTickTransactions);

impl Into<QubicTxHash> for Transaction {
    fn into(self) -> QubicTxHash {
        let mut hash = [0; 32];
        let mut kg = KangarooTwelve::hash(&self.to_bytes(), &[]);

        kg.squeeze(&mut hash);

        QubicTxHash(hash)
    }
}

impl<T: Copy> Into<QubicTxHash> for Call<T> {
    fn into(self) -> QubicTxHash {
        let mut hash = [0; 32];
        let mut kg = KangarooTwelve::hash(&self.to_bytes(), &[]);

        kg.squeeze(&mut hash);

        QubicTxHash(hash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactionData {
    TransferAsset(TransferAssetInput),
    IssueAsset(IssueAssetInput),
    IpoBid(ContractIpoBid),
    SubmitWork(Nonce),
    Unknown(Vec<u8>),
    None,
}

impl ToBytes for TransactionData {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            TransactionData::TransferAsset(d) => d.to_bytes(),
            TransactionData::IssueAsset(d) => d.to_bytes(),
            TransactionData::IpoBid(d) => d.to_bytes(),
            TransactionData::SubmitWork(d) => d.to_bytes(),
            TransactionData::Unknown(d) => d.clone(),
            TransactionData::None => vec![]
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionWithData {
    pub raw_transaction: RawTransaction,
    pub data: TransactionData,
    pub signature: Signature
}

impl ToBytes for TransactionWithData {
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = self.raw_transaction.to_bytes();

        data.extend(self.data.to_bytes());

        data.extend(self.signature.to_bytes());

        data
    }
}

impl FromBytes for TransactionWithData {
    fn from_bytes(data: &[u8]) -> Result<Self, qubic_types::errors::ByteEncodingError> {
        if data.len() < std::mem::size_of::<RawTransaction>() + std::mem::size_of::<Signature>() {
            return Err(qubic_types::errors::ByteEncodingError::InvalidMinimumDataLength { expected_min: std::mem::size_of::<RawTransaction>() + std::mem::size_of::<Signature>(), found: data.len() })
        }

        let raw_tx = unsafe {
            read_unaligned(data.as_ptr() as *const RawTransaction)
        };

        let sig = unsafe {
            read_unaligned(&data[data.len() - std::mem::size_of::<Signature>()] as *const u8 as *const Signature)
        };

        let tx_data = data[std::mem::size_of::<RawTransaction>()..data.len()-std::mem::size_of::<Signature>()].to_vec();

        let data;

        match raw_tx.input_type {
            0 => {
                if raw_tx.input_size as usize == std::mem::size_of::<Nonce>() && raw_tx.amount == 0 {
                    data = TransactionData::SubmitWork(Nonce(tx_data.try_into().unwrap()));
                } else if raw_tx.input_size == 16 {
                    let bid = unsafe {
                        read_unaligned(tx_data.as_ptr() as *const ContractIpoBid)
                    };

                    data = TransactionData::IpoBid(bid);
                } else {
                    data = TransactionData::Unknown(tx_data);
                }
            },
            1 => {
                if raw_tx.input_size as usize == std::mem::size_of::<IssueAssetInput>() {
                    let input = unsafe {
                        read_unaligned(tx_data.as_ptr() as *const IssueAssetInput)
                    };

                    data = TransactionData::IssueAsset(input);
                } else {
                    data = TransactionData::Unknown(tx_data);
                }
            },
            2 => {
                if raw_tx.input_size as usize == std::mem::size_of::<IssueAssetInput>() {
                    let input = unsafe {
                        read_unaligned(tx_data.as_ptr() as *const TransferAssetInput)
                    };

                    data = TransactionData::TransferAsset(input);
                } else {
                    data = TransactionData::Unknown(tx_data);
                }
            }
            _ => {
                data = TransactionData::Unknown(tx_data);
            }
        }

        Ok(
            Self { raw_transaction: raw_tx, data, signature: sig }
        )
    }
}

impl GetSigner for TransactionWithData {
    fn get_signer(&self) -> &QubicId {
        &self.raw_transaction.from
    }
}

impl Into<QubicTxHash> for TransactionWithData {
    fn into(self) -> QubicTxHash {
        let mut hash = [0; 32];
        let mut kg = KangarooTwelve::hash(&self.to_bytes(), &[]);

        kg.squeeze(&mut hash);

        QubicTxHash(hash)
    }
}