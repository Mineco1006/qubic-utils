#[macro_use]
mod macros;
pub mod transactions;
pub mod ticks;
pub mod time;
pub mod token;
pub mod assets;
pub mod special_commands;

use core::net::Ipv4Addr;

use alloc::vec::Vec;
use qubic_types::{QubicId, Signature, Nonce, traits::ToBytes};

use crate::{consts::SPECTRUM_DEPTH, utils::QubicRequest, Header, MessageType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkSolution {
    pub public_key: QubicId,
    pub nonce: Nonce,
}

impl From<WorkSolution> for BroadcastMessage {
    fn from(value: WorkSolution) -> Self {
        Self {
            source_public_key: QubicId::default(),
            destination_public_key: value.public_key,
            gamming_nonce: Nonce::default(),
            solution_nonce: Nonce::default(),
            signature: Signature::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RequestEntity {
    pub public_key: QubicId
}

set_message_type!(RequestEntity, MessageType::RequestEntity);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RespondedEntity {
    pub entity: Entity,
    pub tick: u32,
    pub spectrum_index: u32,
    pub siblings: [QubicId; SPECTRUM_DEPTH]
}

set_message_type!(RespondedEntity, MessageType::RespondEntity);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct Entity {
    pub public_key: QubicId,
    pub incoming_amount: u64,
    pub outgoing_amount: u64,
    pub number_of_incoming_transfers: u32,
    pub number_of_outgoing_transfers: u32,
    pub latest_incoming_transfer_tick: u32,
    pub latest_outgoing_transfer_tick: u32
}

impl Entity {
    pub fn balance(&self) -> u64 {
        self.incoming_amount - self.outgoing_amount
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestComputors;

set_message_type!(RequestComputors, MessageType::RequestComputors);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Computors {
    pub epoch: u16,
    pub public_key: [QubicId; 676],
    pub signature: Signature
}

set_message_type!(Computors, MessageType::BroadcastComputors);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct RequestContractIpo {
    pub contract_index: u32
}

set_message_type!(RequestContractIpo, MessageType::RequestContractIPO);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ContractIpo {
    pub contract_index: u32,
    pub tick: u32,
    pub public_keys: [QubicId; 676],
    pub prices: [u64; 676]
}

set_message_type!(ContractIpo, MessageType::RespondContractIPO);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct ContractIpoBid {
    pub price: u64,
    pub quantity: u16
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ExchangePublicPeers {
    pub peers: [Ipv4Addr; 4]
}

impl Default for ExchangePublicPeers {
    fn default() -> Self {
        Self { 
            peers: [Ipv4Addr::new(0, 0, 0, 0); 4]
        }
    }
}

set_message_type!(ExchangePublicPeers, MessageType::ExchangePublicPeers);


#[derive(Debug, Clone)]
#[repr(C)]
pub struct Packet<T> {
    header: Header,
    pub data: T
}

#[cfg(feature = "std")]
impl<T: Sized + QubicRequest> Packet<T> {
    pub fn new(data: T, randomize_dejavu: bool) -> Packet<T> {
        Self {
            header: Header::new(core::mem::size_of::<Header>() + core::mem::size_of_val(&data), T::get_message_type(), randomize_dejavu),
            data
        }
    }
}

impl<T: ToBytes> ToBytes for Packet<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = self.header.to_bytes();

        buffer.extend(self.data.to_bytes());
        
        buffer
    }
}