use std::num::NonZeroUsize;

use qubic_types::{QubicId, Signature};

use crate::{MessageType, consts::NUMBER_OF_TRANSACTION_PER_TICK};

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

impl Transaction {
    pub fn verify(&self) -> bool {
        self.raw_transaction.from.verify(self.raw_transaction, self.signature)
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