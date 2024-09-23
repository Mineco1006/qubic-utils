use core::{fmt::Debug, num::NonZeroUsize, ptr::read_unaligned};
use tiny_keccak::{Hasher, IntoXof, KangarooTwelve, Xof};
use qubic_types::{traits::{FromBytes, GetSigner, Sign, ToBytes}, MiningSeed, Nonce, QubicId, QubicTxHash, QubicWallet, Signature};

use crate::{consts::NUMBER_OF_TRANSACTION_PER_TICK, utils::QubicRequest, MessageType};

use super::{assets::{IssueAssetInput, TransferAssetInput, ISSUE_ASSET_FEE, QXID, TRANSFER_FEE}, send_to_many::{SendToManyInput, SEND_TO_MANY_CONTRACT_INDEX}, ContractIpoBid};

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

        for flag in flags.iter_mut() {
            *flag = u8::MAX;
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

impl From<Transaction> for QubicTxHash {
    fn from(val: Transaction) -> Self {
        let mut hash = [0; 32];
        let tx_bytes = val.to_bytes();
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&tx_bytes);
        kg.into_xof().squeeze(&mut hash);

        QubicTxHash(hash)
    }
}

impl<T: Copy> From<Call<T>> for QubicTxHash {
    fn from(val: Call<T>) -> Self {
        let mut hash = [0; 32];
        let call_bytes = val.to_bytes();
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&call_bytes);
        kg.into_xof().squeeze(&mut hash);

        QubicTxHash(hash)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactionData {
    TransferAsset(TransferAssetInput),
    IssueAsset(IssueAssetInput),
    IpoBid(ContractIpoBid),
    SubmitWork { seed: MiningSeed, nonce: Nonce },
    SendToMany(SendToManyInput),
    Unknown(Vec<u8>),

    #[default]
    None,
}

impl ToBytes for TransactionData {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            TransactionData::TransferAsset(d) => d.to_bytes(),
            TransactionData::IssueAsset(d) => d.to_bytes(),
            TransactionData::IpoBid(d) => d.to_bytes(),
            TransactionData::SubmitWork { seed, nonce } => [seed.to_bytes(), nonce.to_bytes()].concat(),
            TransactionData::SendToMany(d) => d.to_bytes(),
            TransactionData::Unknown(d) => d.clone(),
            TransactionData::None => vec![]
        }
    }
}

impl TransactionData {
    pub fn sanitize_transaction(&self, tx: &mut RawTransaction) {
        match self {
            Self::IpoBid(_) => {
                tx.input_type = 0;
                tx.input_size = core::mem::size_of::<ContractIpoBid>() as u16;
            },
            Self::IssueAsset(_) => {
                tx.input_type = 1;
                tx.input_size = core::mem::size_of::<IssueAssetInput>() as u16;
                tx.to = QXID;
                tx.amount = ISSUE_ASSET_FEE;
            },
            Self::SubmitWork { .. } => {
                tx.to = QubicId::default();
                tx.amount = 1_000_000;
                tx.input_type = 2;
                tx.input_size = (core::mem::size_of::<MiningSeed>() as u16) + (core::mem::size_of::<Nonce>() as u16);
            },
            Self::TransferAsset(_) => {
                tx.input_type = 2;
                tx.amount = TRANSFER_FEE;
                tx.to = QXID;
                tx.input_size = core::mem::size_of::<TransferAssetInput>() as u16;
            },
            Self::SendToMany(SendToManyInput { ids: _, amounts }) => {
                tx.input_type = 1;
                tx.input_size = core::mem::size_of::<SendToManyInput>() as u16;
                tx.to = QubicId::from_contract_id(SEND_TO_MANY_CONTRACT_INDEX);
                tx.amount += amounts.iter().sum::<u64>();
            },
            Self::Unknown(data) => {
                tx.input_size = data.len() as u16;
            },
            Self::None => ()
        }
    }
}

/// ### Heap allocated Transaction with custom serializer/deserializer for the data field
/// ```
/// use qubic_types::QubicWallet;
/// 
/// let wallet = QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
/// let mut tx = TransactionWithData::default();
/// tx.sign(&wallet);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionWithData {
    pub raw_transaction: RawTransaction,
    pub data: TransactionData,
    pub signature: Signature
}

set_message_type!(TransactionWithData, MessageType::BroadcastTransaction);

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
        if data.len() < core::mem::size_of::<RawTransaction>() + core::mem::size_of::<Signature>() {
            return Err(qubic_types::errors::ByteEncodingError::InvalidMinimumDataLength { expected_min: core::mem::size_of::<RawTransaction>() + core::mem::size_of::<Signature>(), found: data.len() })
        }

        let raw_tx = unsafe {
            read_unaligned(data.as_ptr() as *const RawTransaction)
        };

        let sig = unsafe {
            read_unaligned(&data[data.len() - core::mem::size_of::<Signature>()] as *const u8 as *const Signature)
        };

        let tx_data = data[core::mem::size_of::<RawTransaction>()..data.len()-core::mem::size_of::<Signature>()].to_vec();

        let data;

        match raw_tx.input_type {
            0 => {
    	        if raw_tx.input_size == 16 {
                    let bid = ContractIpoBid::from_bytes(&tx_data)?;

                    data = TransactionData::IpoBid(bid);
                } else if raw_tx.input_size == 0 {
                    data = TransactionData::None;
                } else {
                    data = TransactionData::Unknown(tx_data);
                }
            },
            1 => {
                if raw_tx.input_size as usize == core::mem::size_of::<IssueAssetInput>() {
                    let input = unsafe {
                        read_unaligned(tx_data.as_ptr() as *const IssueAssetInput)
                    };

                    data = TransactionData::IssueAsset(input);
                } else if raw_tx.input_size as usize == core::mem::size_of::<SendToManyInput>() {
                    let input = unsafe {
                        read_unaligned(tx_data.as_ptr() as *const SendToManyInput)
                    };

                    data = TransactionData::SendToMany(input);
                } else {
                    data = TransactionData::Unknown(tx_data);
                }
            },
            2 => {
                if raw_tx.input_size as usize == (core::mem::size_of::<MiningSeed>() + core::mem::size_of::<Nonce>())
                && tx_data.len() == (core::mem::size_of::<MiningSeed>() + core::mem::size_of::<Nonce>())
                && raw_tx.amount == 1_000_000 {
                    data = TransactionData::SubmitWork {
                        seed: MiningSeed::from_bytes(&tx_data[..core::mem::size_of::<MiningSeed>()])?,
                        nonce: Nonce::from_bytes(&tx_data[core::mem::size_of::<MiningSeed>()..])?
                    };
                } else if raw_tx.input_size as usize == core::mem::size_of::<IssueAssetInput>() {
                    let input = unsafe {
                        read_unaligned(tx_data.as_ptr() as *const TransferAssetInput)
                    };

                    data = TransactionData::TransferAsset(input);
                } else {
                    data = TransactionData::Unknown(tx_data);
                }
            }
            _ => {
                if raw_tx.input_size == 0 {
                    data = TransactionData::None;
                } else {
                    data = TransactionData::Unknown(tx_data);
                }
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

impl From<TransactionWithData> for QubicTxHash {
    fn from(val: TransactionWithData) -> Self {
        let mut hash = [0; 32];
        let tx_bytes = val.to_bytes();
        let mut kg = KangarooTwelve::new(b"");
        kg.update(&tx_bytes);
        kg.into_xof().squeeze(&mut hash);

        QubicTxHash(hash)
    }
}

impl From<Transaction> for TransactionWithData {
    fn from(value: Transaction) -> Self {
        Self {
            raw_transaction: value.raw_transaction,
            data: TransactionData::None,
            signature: value.signature
        }
    }
}

impl<T> From<Call<T>> for TransactionWithData
    where T: Copy + Into<TransactionData>
{
    fn from(value: Call<T>) -> Self {
        Self {
            raw_transaction: value.raw_call.tx,
            data: value.raw_call.input.into(),
            signature: value.signature
        }
    }
}

impl From<RawTransaction> for TransactionWithData {
    fn from(value: RawTransaction) -> Self {
        Self {
            raw_transaction: value,
            data: TransactionData::None,
            signature: Signature::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TransactionBuilder<'a> {
    raw_tx: RawTransaction,
    data: TransactionData,
    signer: Option<&'a QubicWallet>
}

impl<'a> TransactionBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_from_id(mut self, from: QubicId) -> Self {
        self.raw_tx.from = from;
        self
    }

    pub fn with_to_id(mut self, to: QubicId) -> Self {
        self.raw_tx.to = to;
        self
    }

    pub fn with_amount(mut self, amount: u64) -> Self {
        self.raw_tx.amount = amount;
        self
    }

    pub fn with_tx_data(mut self, data: TransactionData) -> Self {
        self.data = data;
        self
    }

    pub fn with_tick(mut self, tick: u32) -> Self {
        self.raw_tx.tick = tick;
        self
    }

    pub fn with_input_type_and_size(mut self, input_type: u16, input_size: u16) -> Self {
        self.raw_tx.input_type = input_type;
        self.raw_tx.input_size = input_size;

        self
    }

    pub fn with_signing_wallet(mut self, wallet: &'a QubicWallet) -> Self {
        self.signer = Some(wallet);
        self
    }

    pub fn build(mut self) -> TransactionWithData {
        if let Some(signer) = self.signer {
            self.data.sanitize_transaction(&mut self.raw_tx);

            self.raw_tx.from = signer.public_key;

            let mut tx = TransactionWithData {
                raw_transaction: self.raw_tx,
                data: self.data,
                signature: Signature::default()
            };

            tx.sign(signer).unwrap();

            tx
        } else {
            self.data.sanitize_transaction(&mut self.raw_tx);

            TransactionWithData {
                raw_transaction: self.raw_tx,
                data: self.data,
                signature: Signature::default()
            }
        }
    }
}