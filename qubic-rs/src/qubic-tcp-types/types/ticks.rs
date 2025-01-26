use core::fmt::Debug;

use qubic_types::{Signature, H256, QubicTxHash};

use crate::{MessageType, consts::{NUMBER_OF_TRANSACTION_PER_TICK, NUMBER_OF_COMPUTORS, MAX_NUMBER_OF_CONTRACTS}};

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
    pub number_of_misaligned_votes: u16,
    pub initial_tick: u32
}
set_message_type!(CurrentTickInfo, MessageType::RespondCurrentTickInfo);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TickPeriod {
    Idle { remaining: u32 },
    Mining { remaining: u32 },
}

impl CurrentTickInfo {
    pub fn tick_period(&self) -> TickPeriod {
        let ticks_in_epoch = self.tick - self.initial_tick;

        const MINING_TICKS: u32 = 676;
        const IDLE_TICKS: u32 = 677;

        let rem = ticks_in_epoch % (MINING_TICKS + IDLE_TICKS);

        if rem < MINING_TICKS {
            TickPeriod::Mining { remaining: MINING_TICKS - rem }
        } else {
            TickPeriod::Idle { remaining: MINING_TICKS + IDLE_TICKS - rem }
        }
    }
} 

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RequestTickData {
    pub tick: u32
}

set_message_type!(RequestTickData, MessageType::RequestTickData);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TickData {
    pub computor_index: u16,
    pub epoch: u16,
    pub tick: u32,

    pub time: QubicTime,

    pub time_lock: [u8; 32],
    pub transaction_digest: [QubicTxHash; NUMBER_OF_TRANSACTION_PER_TICK],
    pub contract_fees: [u64; MAX_NUMBER_OF_CONTRACTS],

    pub signature: Signature
}

set_message_type!(TickData, MessageType::BroadcastFutureTickData);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Tick {
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

set_message_type!(Tick, MessageType::BroadcastTick);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct QuorumTickData {
    pub tick: u32,
    pub vote_flags: [u8; (NUMBER_OF_COMPUTORS + 7)/8]
}

set_message_type!(QuorumTickData, MessageType::RequestQuorumTick);
