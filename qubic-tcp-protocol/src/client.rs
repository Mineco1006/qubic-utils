use std::ptr::copy_nonoverlapping;

use crate::{transport::Transport, types::{Transaction, Packet, GetCurrentTickInfo, CurrentTickInfo, WorkSolution, BroadcastMessage, RequestComputors, Computors, Entity, RequestEntity, ContractIpo, RequestContractIpo, TickData, RequestTickData, RequestQuorumTick}, utils::AsByteEncoded};
use anyhow::{Result, Ok};
use kangarootwelve::KangarooTwelve;
use qubic_types::QubicWallet;
use rand::Rng;


#[derive(Debug, Clone)]
pub struct Client<T: Transport> {
    transport: T
}

impl<T> Client<T> where T: Transport {
    pub fn new(url: impl ToString) -> Self {
        Self {
            transport: T::new(url.to_string())
        }
    }

    pub fn qu(&self) -> Qu<T> {
        Qu {
            transport: &self.transport
        }
    }
}

pub struct Qu<'a, T: Transport> {
    transport: &'a T
}

impl<'a, T> Qu<'a, T> where T: Transport {
    pub fn send_transaction(&self, wallet: &QubicWallet, mut transaction: Transaction) -> Result<()> {
        let mut digest = [0; 32];
        let mut kg = KangarooTwelve::hash(&transaction.encode_as_bytes()[..std::mem::size_of::<Transaction>() - 64], &[]);
        kg.squeeze(&mut digest);
        
        transaction.signature = wallet.sign(&digest);

        self.transport.send_without_response(Packet::new(transaction, false))?;
        Ok(())
    }

    pub fn submit_work(&self, solution: WorkSolution) -> Result<()> {
        let mut rng = rand::thread_rng();
        let mut message: BroadcastMessage = solution.into();
        let mut shared_key_and_gamming_nonce = [0u64; 8];
        let mut gamming_key = [0u64; 4];

        loop {
            unsafe {
                message.gamming_nonce = rng.gen();
                copy_nonoverlapping(message.gamming_nonce.as_ptr(), shared_key_and_gamming_nonce.as_mut_ptr().add(4) as *mut u8, 32);
                kangarootwelve64to32::kangarootwelve64to32(&shared_key_and_gamming_nonce, &mut gamming_key);
            }

            if gamming_key[0] & 0xFF == 0 {
                break;
            }
        }
        let gamming_key: [u8; 32] = gamming_key.into_iter().flat_map(u64::to_le_bytes).collect::<Vec<_>>().try_into().unwrap();
        let mut gamma = [0; 32];
        let mut kg = KangarooTwelve::hash(&gamming_key, &[]);
        kg.squeeze(&mut gamma);

        for i in 0..32 {
            message.solution_nonce[i] = solution.nonce[i] ^ gamma[i];
        }

        for sig in message.signature.iter_mut() {
            *sig = rng.gen();
        }

        self.transport.send_without_response(Packet::new(message, true))?;
        Ok(())
    }

    pub fn get_current_tick_info(&self) -> Result<CurrentTickInfo> {
        let packet = Packet::new(GetCurrentTickInfo, true);

        Ok(self.transport.send_with_response::<CurrentTickInfo>(packet)?)
    }

    pub fn request_computors(&self) -> Result<Computors> {
        let packet = Packet::new(RequestComputors, true);
        
        Ok(self.transport.send_with_response(packet)?)
    }

    pub fn request_entity(&self, public_key: [u8; 32]) -> Result<Entity> {
        let packet = Packet::new(RequestEntity { public_key }, true);
        
        Ok(self.transport.send_with_response(packet)?)
    }

    pub fn request_contract_ipo(&self, contract_index: u32) -> Result<ContractIpo> {
        let packet = Packet::new(RequestContractIpo { contract_index }, true);
        
        Ok(self.transport.send_with_response(packet)?)
    }

    pub fn request_tick_data(&self, tick: u32) -> Result<TickData> {
        let packet = Packet::new(RequestTickData { tick }, true);
    
        Ok(self.transport.send_with_response(packet)?)
    }

    pub fn request_quorum_tick(&self, tick: u32, vote_flags: [u8; (676 + 7) / 8]) -> Result<TickData> {
        let packet = Packet::new(RequestQuorumTick { tick, vote_flags }, true);
        
        Ok(self.transport.send_with_response(packet)?)
    }
}