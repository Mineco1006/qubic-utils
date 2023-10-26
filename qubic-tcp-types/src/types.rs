use std::net::Ipv4Addr;

use qubic_types::{QubicId, Signature, H256, Nonce};

use crate::{utils::{QubicRequest, QubicReturnType}, Header, MessageType};

macro_rules! set_message_type {
    ($impl: ident, $message_type: expr) => {
        impl QubicRequest for $impl {
            fn get_message_type() -> MessageType {
                $message_type
            }
        }
    };
}

macro_rules! set_return_type {
    ($impl: ident, $return_type: ty) => {
        impl QubicReturnType for $impl {
            type ReturnType = $return_type;
        }
    };
}

#[derive(Debug, Clone, Copy, Default)]
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


#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Transaction {
    pub raw_transaction: RawTransaction,
    pub signature: Signature
}

set_message_type!(Transaction, MessageType::BroadcastTransaction);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GetCurrentTickInfo;
set_message_type!(GetCurrentTickInfo, MessageType::RequestCurrentTickInfo);
set_return_type!(GetCurrentTickInfo, CurrentTickInfo);

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct CurrentTickInfo {
    pub tick_duration: u16,
    pub epoch: u16,
    pub tick: u32,
    pub number_of_aligned_votes: u16,
    pub number_of_misaligned_votes: u16
}
set_message_type!(CurrentTickInfo, MessageType::RespondCurrentTickInfo);

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct BroadcastMessage {
    pub source_public_key: QubicId,
    pub destination_public_key: QubicId,
    pub gamming_nonce: Nonce,
    pub solution_nonce: Nonce,
    pub signature: Signature
}
set_message_type!(BroadcastMessage, MessageType::BroadcastMessage);

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkSolution {
    pub public_key: QubicId,
    pub nonce: Nonce,
}

impl Into<BroadcastMessage> for WorkSolution {
    fn into(self) -> BroadcastMessage {
        BroadcastMessage {
            source_public_key: self.public_key,
            destination_public_key: QubicId::default(),
            gamming_nonce: Nonce::default(),
            solution_nonce: Nonce::default(),
            signature: Signature::default()
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RequestEntity {
    pub public_key: QubicId
}

set_message_type!(RequestEntity, MessageType::RequestEntity);
set_return_type!(RequestEntity, Entity);

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entity {
    pub public_key: QubicId,
    pub incoming_amount: u64,
    pub outgoing_amount: u64,
    pub number_of_incoming_transfers: u32,
    pub number_of_outgoing_transfers: u32,
    pub latest_incoming_transfer_tick: u32,
    pub latest_outgoing_transfer_tick: u32
}

set_message_type!(Entity, MessageType::RespondEntity);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RequestComputors;

set_message_type!(RequestComputors, MessageType::RequestComputors);
set_return_type!(RequestComputors, Computors);

#[derive(Debug, Clone, Copy)]
pub struct Computors {
    pub epoch: u16,
    pub public_key: [QubicId; 676],
    pub signature: Signature
}

set_message_type!(Computors, MessageType::BroadcastComputors);

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RequestContractIpo {
    pub contract_index: u32
}

set_message_type!(RequestContractIpo, MessageType::RequestContractIPO);
set_return_type!(RequestContractIpo, ContractIpo);

#[derive(Debug, Clone, Copy)]
pub struct ContractIpo {
    pub contract_index: u32,
    pub tick: u32,
    pub public_keys: [QubicId; 676],
    pub prices: [u64; 676]
}

set_message_type!(ContractIpo, MessageType::RespondContractIPO);

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RequestTickData {
    pub tick: u32
}

set_message_type!(RequestTickData, MessageType::RequestTickData);
set_return_type!(RequestTickData, TickData);

#[derive(Debug, Clone, Copy)]
pub struct Proposal {
    pub uri_size: u8,
    pub uri: [u8; 255]
}

#[derive(Debug, Clone, Copy)]
pub struct Ballot {
    pub zero: u16,
    pub votes: [u8; (676 * 3 + 7)/8],
    pub quasi_random_number: u8
}

#[derive(Debug, Clone, Copy)]
pub struct VarStruct {
    pub proposal: Proposal,
    pub ballot: Ballot
}

#[derive(Debug, Clone, Copy)]
pub struct TickData {
    pub computor_index: u16,
    pub epoch: u16,
    pub tick: u32,

    pub millisecond: u16,
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,

    pub var_struct: VarStruct,

    pub time_lock: [u8; 32],
    pub transaction_digest: [H256; 128],
    pub contract_fees: [u64; 1024],

    pub signature: Signature
}

set_message_type!(TickData, MessageType::BroadcastTick);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RequestQuorumTick {
    pub tick: u32,
    pub vote_flags: [u8; (676 + 7)/8]
}

set_message_type!(RequestQuorumTick, MessageType::RequestQuorumTick);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ExchangePublicPeers {
    pub peers: [Ipv4Addr; 4]
}

set_message_type!(ExchangePublicPeers, MessageType::ExchangePublicPeers);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Packet<T: Sized> {
    header: Header,
    pub data: T
}

impl<T: Sized + QubicRequest> Packet<T> {
    pub fn new(data: T, randomize_dejavu: bool) -> Packet<T> {
        Self {
            header: Header::new(std::mem::size_of::<Header>() + std::mem::size_of::<T>(), T::get_message_type(), randomize_dejavu),
            data
        }
    }
}