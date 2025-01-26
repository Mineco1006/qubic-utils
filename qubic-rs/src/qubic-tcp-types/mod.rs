#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
pub extern crate alloc;
use rand::Rng;

pub mod types;
pub mod utils;
pub mod prelude;
pub mod consts;
pub mod events;

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

    ProcessSpecialCommand = 255
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
        
        let mut new = Self { size: [0; 3], message_type, dejavu: 0};
        new.set_size(size);
        if randomize_dejavu {
            new.randomize_dejavu();
        }

        new
    }

    pub fn new_with_dejavu(size: usize, message_type: MessageType, dejavu: u32) -> Self {
        
        let mut new = Self { size: [0; 3], message_type, dejavu: 0};
        new.set_size(size);
        new.dejavu = dejavu;

        new
    }

    pub fn get_size(&self) -> usize {
        (self.size[0] as usize) | (self.size[1] as usize) << 8 | (self.size[2] as usize) << 16
    }
    
    pub fn set_size(&mut self, size: usize) {
        self.size[0] = size as u8;
        self.size[1] = (size >> 8) as u8 ;
        self.size[2] = (size >> 16) as u8 ;
    }

    pub fn zero_dejavu(&mut self) {
        self.dejavu = 0;
    }

    pub fn randomize_dejavu(&mut self) {
        let mut rng = rand::thread_rng();
        self.dejavu = rng.gen();
    }

    pub fn set_type(&mut self, new_type: MessageType) {
        self.message_type = new_type;
    }
}
