pub mod transport;
pub mod utils;
pub mod client;
pub mod types;

use std::{ptr::copy_nonoverlapping, io::Write, net::TcpStream};
use client::Client;
use kangarootwelve::KangarooTwelve;
use qubic_types::{QubicId, QubicWallet};
use rand::{rngs::ThreadRng, Rng};
use transport::Tcp;

use crate::types::Transaction;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum MessageType {
    BroadcastMessage = 1, // aka SubmitWork?
    
    // computor
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

    ProcessSpecialCommand = 255
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ProtocolVersionB(pub u8);

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Header {
    size: [u8; 3],
    protocol: ProtocolVersionB,
    dejavu: [u8; 3],
    message_type: MessageType
}

impl Header {

    pub fn new(size: usize, protocol: ProtocolVersionB, message_type: MessageType, randomize_dejavu: bool) -> Self {
        
        let mut new = Self { size: [0; 3], protocol, dejavu: [0; 3], message_type};
        new.set_size(size);
        if randomize_dejavu {
            new.randomize_dejavu();
        }

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
        self.dejavu = [0; 3];
    }

    pub fn randomize_dejavu(&mut self) {
        let mut rng = rand::thread_rng();
        self.dejavu = rng.gen();
    }

    pub fn set_type(&mut self, new_type: MessageType) {
        self.message_type = new_type;
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Message {
    source_public_key: [u8; 32],
    destination_public_key: [u8; 32],
    gamming_nonce: [u8; 32],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct BroadcastMessageData {
    solution_nonce: [u8; 32],
    signature: [u8; 64]
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SubmitWorkPacket {
    header: Header,
    message: Message,
    data: BroadcastMessageData
}

impl SubmitWorkPacket {
    pub fn generate(nonce: [u8; 32], source: [u8; 32], destination: [u8; 32]) -> Self {
        let mut rng = rand::thread_rng();
        let mut packet = Self { header: Header::new(std::mem::size_of::<Self>(), ProtocolVersionB(0), MessageType::BroadcastMessage, true), message: Message { source_public_key: source, destination_public_key: destination, gamming_nonce: [0; 32] }, data: BroadcastMessageData { solution_nonce: [0; 32], signature: [0; 64] }};
        
        let mut shared_key_and_gamming_nonce = [0u64; 8];
        let mut gamming_key = [0u64; 4];

        loop {
            unsafe {
                packet.message.gamming_nonce = rng.gen();
                copy_nonoverlapping(packet.message.gamming_nonce.as_ptr(), shared_key_and_gamming_nonce.as_mut_ptr().add(4) as *mut u8, 32);
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
            packet.data.solution_nonce[i] = nonce[i] ^ gamma[i];
        }



        for sig in packet.data.signature.iter_mut() {
            *sig = rng.gen();
        }

        packet
    }

    pub fn submit(self, stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let data = struct_to_slice(&self);
        stream.write_all(data)?;
        Ok(())
    }
}



fn struct_to_slice<T: Sized>(data: &T) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts((data as *const T) as *const u8, core::mem::size_of::<T>())
    }
}

#[test]
fn test() {
    let client = Client::<Tcp>::new("104.219.232.50:21841");

    let current_tick = dbg!(client.qu().get_current_tick_info().unwrap());

    let from = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    let to = QubicId::from_str("XOHYYIZLBNOAWDRWRMSGFTOBSEPATZLQYNTRBPHFXDAIOYQTGTNFTDABLLFA").unwrap();
    let wallet = QubicWallet::from_seed(from).unwrap();

    let entity = dbg!(client.qu().request_entity(wallet.public_key).unwrap());

    let balance = entity.incoming_amount - entity.outgoing_amount;
    println!("Balance: {}", balance);

    let transaction = Transaction {
        from: wallet.public_key,
        to: to.0,
        amount: 1,
        tick: current_tick.tick + 10,
        input_type: 0,
        input_size: 0,
        signature: [0; 64]
    };

    dbg!(wallet.get_identity());

    client.qu().send_transaction(&wallet, transaction).unwrap();
    
    //dbg!(client.qu().request_computors().unwrap());

    //dbg!(client.qu().request_tick_data(8454467).unwrap());
}