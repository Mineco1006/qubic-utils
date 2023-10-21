#![feature(impl_trait_in_assoc_type)]

pub mod transport;
pub mod client;

pub extern crate qubic_tcp_types;

use std::{ptr::copy_nonoverlapping, io::Write, net::TcpStream};
use kangarootwelve::KangarooTwelve;
use qubic_tcp_types::{Header, MessageType};
use rand::Rng;

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
        let mut packet = Self { header: Header::new(std::mem::size_of::<Self>(), MessageType::BroadcastMessage, true), message: Message { source_public_key: source, destination_public_key: destination, gamming_nonce: [0; 32] }, data: BroadcastMessageData { solution_nonce: [0; 32], signature: [0; 64] }};
        
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

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test() {
    use qubic_tcp_types::types::RawTransaction;
    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    use client::Client;
    use qubic_types::{QubicId, QubicWallet};
    use transport::Tcp;
    let client = Client::<Tcp>::new("136.243.41.86:21841");

    let current_tick = dbg!(client.qu().get_current_tick_info().unwrap());
    let to = QubicId::from_str("BGKBSSHTGNLYOBUNOBYZNPEYDNABWKCHIWGOOUJRTGJOXTYPPWSXMGUAXHKI").unwrap();
    let wallet = QubicWallet::from_seed(seed).unwrap();
    let entity = dbg!(client.qu().request_entity(wallet.public_key).unwrap());
    println!("{}", entity.public_key);
    let balance = entity.incoming_amount - entity.outgoing_amount;
    println!("Balance: {}", balance);

    let tx = RawTransaction {
        from: wallet.public_key,
        to,
        amount: 10,
        tick: current_tick.tick + 30,
        ..Default::default()
    };

    dbg!(tx);
}

#[cfg(any(feature = "async", feature = "http"))]
#[tokio::test]
async fn test() {
    use qubic_tcp_types::types::RawTransaction;
    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    use client::Client;
    use qubic_types::{QubicId, QubicWallet};
    use transport::Tcp;
    let client = Client::<Tcp>::new("136.243.41.86:21841").await;

    let current_tick = dbg!(client.qu().get_current_tick_info().await.unwrap());
    let to = QubicId::from_str("BGKBSSHTGNLYOBUNOBYZNPEYDNABWKCHIWGOOUJRTGJOXTYPPWSXMGUAXHKI").unwrap();
    let wallet = QubicWallet::from_seed(seed).unwrap();
    let entity = dbg!(client.qu().request_entity(wallet.public_key).await.unwrap());
    println!("{}", entity.public_key);
    let balance = entity.incoming_amount - entity.outgoing_amount;
    println!("Balance: {}", balance);
    let tx = RawTransaction {
        from: wallet.public_key,
        to,
        amount: 10,
        tick: current_tick.tick + 30,
        ..Default::default()
    };

    dbg!(tx);
}