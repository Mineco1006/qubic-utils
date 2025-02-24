#![cfg_attr(not(feature = "std"), no_std)]

use rand::Rng;

pub mod consts;
pub mod events;
pub mod prelude;
pub mod types;
pub mod utils;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum MessageType {
    BroadcastMessage = 1,

    ExchangePublicPeers = 0,
    BroadcastComputors = 2,
    BroadcastTick = 3,
    BroadcastFutureTickData = 8,
    RequestComputors = 11,
    RequestQuorumTick = 14,
    RequestTickData = 16,
    BroadcastTransaction = 24,

    RequestCurrentTickInfo = 27,
    RespondCurrentTickInfo = 28,

    RequestTickTransactions = 29,

    RequestEntity = 31,
    RespondEntity = 32,

    RequestContractIPO = 33,
    RespondContractIPO = 34,

    EndResponse = 35,

    RequestIssuedAsset = 36,
    RespondIssuedAsset = 37,
    RequestOwnedAsset = 38,
    RespondOwnedAsset = 39,
    RequestPossessedAsset = 40,
    RespondPossessedAsset = 41,

    RequestContractFunction = 42,
    RespondContractFunction = 43,

    RequestLog = 44,
    RespondLog = 45,

    RequestSystemInfo = 46,
    RespondSystemInfo = 47,

    ProcessSpecialCommand = 255,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Header {
    pub size: [u8; 3],
    pub message_type: MessageType,
    pub dejavu: u32,
}

impl Header {
    #[cfg(not(feature = "wasm"))]
    pub fn new(size: usize, message_type: MessageType, randomize_dejavu: bool) -> Self {
        let mut new = Self {
            size: [0; 3],
            message_type,
            dejavu: 0,
        };
        new.set_size(size);
        if randomize_dejavu {
            new.randomize_dejavu();
        }

        new
    }

    pub fn new_with_dejavu(size: usize, message_type: MessageType, dejavu: u32) -> Self {
        let mut new = Self {
            size: [0; 3],
            message_type,
            dejavu: 0,
        };
        new.set_size(size);
        new.dejavu = dejavu;

        new
    }

    pub fn get_size(&self) -> usize {
        (self.size[0] as usize) | (self.size[1] as usize) << 8 | (self.size[2] as usize) << 16
    }

    pub fn set_size(&mut self, size: usize) {
        self.size[0] = size as u8;
        self.size[1] = (size >> 8) as u8;
        self.size[2] = (size >> 16) as u8;
    }

    pub fn zero_dejavu(&mut self) {
        self.dejavu = 0;
    }

    pub fn randomize_dejavu(&mut self) {
        let mut rng = rand::rng();
        self.dejavu = rng.random();
    }

    pub fn set_type(&mut self, new_type: MessageType) {
        self.message_type = new_type;
    }
}

#[cfg(test)]
mod tests {
    use crate::qubic_tcp_types::{
        types::{
            assets::{RequestIssuedAsset, RequestOwnedAsset, RequestPossessedAsset},
            contracts::{RequestContractFunction, ResponseContractFunction},
            qlogging::RequestLog,
            // special_commands::SpecialCommand,
            ticks::{
                CurrentTickInfo, GetCurrentTickInfo, QuorumTickData, RequestTickData, Tick,
                TickData,
            },
            transactions::{RequestedTickTransactions, Transaction, TransactionWithData},
            BroadcastMessage,
            Computors,
            ContractIpo,
            ExchangePublicPeers,
            RequestComputors,
            RequestContractIpo,
            RequestEntity,
            RequestSystemInfo,
            RespondedEntity,
            SystemInfo,
        },
        utils::QubicRequest,
        MessageType,
    };

    #[test]
    fn message_types() {
        assert_eq!(
            GetCurrentTickInfo::get_message_type(),
            MessageType::RequestCurrentTickInfo
        );
        assert_eq!(
            CurrentTickInfo::get_message_type(),
            MessageType::RespondCurrentTickInfo
        );
        assert_eq!(
            RequestTickData::get_message_type(),
            MessageType::RequestTickData
        );
        assert_eq!(
            TickData::get_message_type(),
            MessageType::BroadcastFutureTickData
        );
        assert_eq!(Tick::get_message_type(), MessageType::BroadcastTick);
        assert_eq!(
            QuorumTickData::get_message_type(),
            MessageType::RequestQuorumTick
        );
        assert_eq!(
            Transaction::get_message_type(),
            MessageType::BroadcastTransaction
        );
        assert_eq!(
            TransactionWithData::get_message_type(),
            MessageType::BroadcastTransaction
        );
        assert_eq!(
            RequestedTickTransactions::get_message_type(),
            MessageType::RequestTickTransactions
        );
        assert_eq!(
            RequestIssuedAsset::get_message_type(),
            MessageType::RequestIssuedAsset
        );
        assert_eq!(
            RequestOwnedAsset::get_message_type(),
            MessageType::RequestOwnedAsset
        );
        assert_eq!(
            RequestPossessedAsset::get_message_type(),
            MessageType::RequestPossessedAsset
        );
        assert_eq!(RequestLog::get_message_type(), MessageType::RequestLog);
        assert_eq!(
            RequestContractFunction::get_message_type(),
            MessageType::RequestContractFunction
        );
        assert_eq!(
            ResponseContractFunction::get_message_type(),
            MessageType::RespondContractFunction
        );
        // SpecialCommand needs associated type
        // assert_eq!(
        //     SpecialCommand::get_message_type(),
        //     MessageType::ProcessSpecialCommand
        // );
        assert_eq!(
            BroadcastMessage::get_message_type(),
            MessageType::BroadcastMessage
        );
        assert_eq!(
            RequestEntity::get_message_type(),
            MessageType::RequestEntity
        );
        assert_eq!(
            RespondedEntity::get_message_type(),
            MessageType::RespondEntity
        );
        assert_eq!(
            RequestComputors::get_message_type(),
            MessageType::RequestComputors
        );
        assert_eq!(
            Computors::get_message_type(),
            MessageType::BroadcastComputors
        );
        assert_eq!(
            RequestContractIpo::get_message_type(),
            MessageType::RequestContractIPO
        );
        assert_eq!(
            ContractIpo::get_message_type(),
            MessageType::RespondContractIPO
        );
        assert_eq!(
            ExchangePublicPeers::get_message_type(),
            MessageType::ExchangePublicPeers
        );
        assert_eq!(
            RequestSystemInfo::get_message_type(),
            MessageType::RequestSystemInfo
        );
        assert_eq!(
            SystemInfo::get_message_type(),
            MessageType::RespondSystemInfo
        );
    }
}
