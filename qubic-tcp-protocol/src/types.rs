use crate::{utils::{AsByteEncoded, GetMessageType}, Header, MessageType};

macro_rules! set_message_type {
    ($impl: ident, $message_type: expr) => {
        impl GetMessageType for $impl {
            fn get_message_type(&self) -> MessageType {
                $message_type
            }
        }
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Transaction {
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub amount: u64,
    pub tick: u32,
    pub input_type: u16,
    pub input_size: u16,
    pub signature: [u8; 64]
}

set_message_type!(Transaction, MessageType::BroadcastTransaction);

impl AsByteEncoded for Transaction {}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GetCurrentTickInfo;
set_message_type!(GetCurrentTickInfo, MessageType::RequestCurrentTickInfo);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct CurrentTickInfo {
    pub tick_duration: u16,
    pub epoch: u16,
    pub tick: u32,
    pub number_of_aligned_votes: u16,
    pub number_of_misaligned_votes: u16
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BroadcastMessage {
    pub source_public_key: [u8; 32],
    pub destination_public_key: [u8; 32],
    pub gamming_nonce: [u8; 32],
    pub solution_nonce: [u8; 32],
    pub signature: [u8; 64]
}

impl AsByteEncoded for BroadcastMessage {}

set_message_type!(BroadcastMessage, MessageType::BroadcastMessage);

#[derive(Debug, Clone, Copy)]
pub struct WorkSolution {
    pub public_key: [u8; 32],
    pub nonce: [u8; 32],
}

impl Into<BroadcastMessage> for WorkSolution {
    fn into(self) -> BroadcastMessage {
        BroadcastMessage {
            source_public_key: self.public_key,
            destination_public_key: [0; 32],
            gamming_nonce: [0; 32],
            solution_nonce: [0; 32],
            signature: [0; 64]
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RequestEntity {
    pub public_key: [u8; 32]
}

set_message_type!(RequestEntity, MessageType::RequestEntity);

#[derive(Debug, Clone, Copy)]
pub struct Entity {
    pub public_key: [u8; 32],
    pub incoming_amount: u64,
    pub outgoing_amount: u64,
    pub number_of_incoming_transfers: u32,
    pub number_of_outgoing_transfers: u32,
    pub latest_incoming_transfer_tick: u32,
    pub latest_outgoing_transfer_tick: u32
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RequestComputors;

impl AsByteEncoded for RequestComputors {}

set_message_type!(RequestComputors, MessageType::RequestComputors);

#[derive(Debug, Clone, Copy)]
pub struct Computors {
    pub epoch: u16,
    pub public_key: [[u8; 32]; 676],
    pub signature: [u8; 64]
}


#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RequestContractIpo {
    pub contract_index: u32
}

impl AsByteEncoded for RequestContractIpo {}

set_message_type!(RequestContractIpo, MessageType::RequestContractIPO);

#[derive(Debug, Clone, Copy)]
pub struct ContractIpo {
    pub contract_index: u32,
    pub tick: u32,
    pub public_keys: [[u8; 32]; 676],
    pub prices: [u64; 676]
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RequestTickData {
    pub tick: u32
}

impl AsByteEncoded for RequestTickData {}

set_message_type!(RequestTickData, MessageType::RequestTickData);

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
    pub transaction_digest: [[u8; 32]; 128],
    pub contract_fees: [u64; 1024],

    pub signature: [u8; 64]
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RequestQuorumTick {
    pub tick: u32,
    pub vote_flags: [u8; (676 + 7)/8]
}

impl AsByteEncoded for RequestQuorumTick {}

set_message_type!(RequestQuorumTick, MessageType::RequestQuorumTick);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Packet<T: Sized> {
    header: Header,
    data: T
}

impl<T: Sized> AsByteEncoded for Packet<T> {}

impl<T: Sized + GetMessageType> Packet<T> {
    pub fn new(data: T, randomize_dejavu: bool) -> Packet<T> {
        Self {
            header: Header::new(std::mem::size_of::<Header>() + std::mem::size_of::<T>(), crate::ProtocolVersionB(0), data.get_message_type(), randomize_dejavu),
            data
        }
    }
}