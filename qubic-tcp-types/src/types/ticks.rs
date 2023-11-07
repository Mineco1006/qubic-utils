use qubic_types::{Signature, H256};

use crate::{MessageType, consts::{NUMBER_OF_TRANSACTION_PER_TICK, NUMBER_OF_COMPUTORS}};

use super::time::QubicTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GetCurrentTickInfo;
set_message_type!(GetCurrentTickInfo, MessageType::RequestCurrentTickInfo);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RequestTickData {
    pub tick: u32
}

set_message_type!(RequestTickData, MessageType::RequestTickData);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Proposal {
    pub uri_size: u8,
    pub uri: [u8; 255]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Ballot {
    pub zero: u16,
    pub votes: [u8; (NUMBER_OF_COMPUTORS * 3 + 7)/8],
    pub quasi_random_number: u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct VarStruct {
    pub proposal: Proposal,
    pub ballot: Ballot
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FutureTickData {
    pub computor_index: u16,
    pub epoch: u16,
    pub tick: u32,

    pub time: QubicTime,

    pub var_struct: VarStruct,

    pub time_lock: [u8; 32],
    pub transaction_digest: [H256; NUMBER_OF_TRANSACTION_PER_TICK],
    pub contract_fees: [u64; 1024],

    pub signature: Signature
}

set_message_type!(FutureTickData, MessageType::BroadcastFutureTickData);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct TickData {
    pub computor_index: u16,
    pub epoch: u16,
    pub tick: u32,

    pub time: QubicTime,

    pub prev_resource_testing_digest: u64,
    pub salted_resource_testing_digest: u64,

    pub prev_spectrum_digest: H256,
    pub prev_universe_digest: H256,
    pub prev_computor_digest: H256,
    pub salted_spectrum_digest: H256,
    pub salted_universe_digest: H256,
    pub salted_computor_digest: H256,

    pub transaction_digest: H256,
    pub expected_next_tick_transaction_digest: H256,
    pub signature: Signature
}

set_message_type!(TickData, MessageType::BroadcastTick);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct QuorumTickData {
    pub tick: u32,
    pub vote_flags: [u8; (NUMBER_OF_COMPUTORS + 7)/8]
}

set_message_type!(QuorumTickData, MessageType::RequestQuorumTick);
