use std::{ptr::{copy_nonoverlapping, read_unaligned}, net::Ipv4Addr, str::FromStr, time::Duration};

#[cfg(not(any(feature = "async", feature = "http")))]
use std::{thread::JoinHandle, io::{Write, Read}};

use crate::transport::Transport;
use qubic_tcp_types::{types::{Packet, BroadcastMessage, RequestComputors, Computors, RequestEntity, ContractIpo, RequestContractIpo, ExchangePublicPeers, transactions::{RawTransaction, Transaction}, ticks::{CurrentTickInfo, GetCurrentTickInfo}, assets::{RespondOwnedAsset, RequestOwnedAsset, QXID, TransferAssetOwnershipAndPossessionInput, ISSUE_ASSET_FEE, IssueAssetInput, AssetName, RespondIssuedAsset, RequestIssuedAsset, RespondPossessedAsset, RequestPossessedAsset, TRANSFER_FEE}, ContractIpoBid}, MessageType, events::NetworkEvent, Header};
use qubic_tcp_types::prelude::*;
use anyhow::{Result, Ok};
use kangarootwelve::KangarooTwelve;
use qubic_types::{QubicWallet, QubicId, traits::{ToBytes, FromBytes}, Signature, QubicTxHash};
use rand::Rng;

#[cfg(any(feature = "async", feature = "http"))]
use tokio::io::{AsyncWriteExt, AsyncReadExt};


#[derive(Debug, Clone)]
pub struct Client<T: Transport> {
    transport: T
}

#[cfg(not(any(feature = "async", feature = "http")))]
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

    pub fn qx(&self) -> Qx<T> {
        Qx {
            transport: &self.transport
        }
    }
}

#[cfg(any(feature = "async", feature = "http"))]
impl<T> Client<T> where T: Transport {
    pub async fn new(url: impl ToString) -> Self {
        Self {
            transport: T::new(url.to_string()).await
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TransactionStatus {
    Failed,
    Included,
    Executed
}

pub const NUMBER_OF_EXCHANGES_PEERS: usize = 4;

#[cfg(not(any(feature = "async", feature = "http")))]
impl<'a, T> Qu<'a, T> where T: Transport {
    pub fn send_raw_transaction(&self, wallet: &QubicWallet, raw_transaction: RawTransaction) -> Result<QubicTxHash> {
        
        let transaction = Transaction {
            raw_transaction,
            signature: wallet.sign(raw_transaction)
        };

        self.transport.send_without_response(Packet::new(transaction, false))?;
        Ok(transaction.into())
    }

    pub fn send_signed_transaction(&self, transaction: Transaction) -> Result<QubicTxHash> {
        self.transport.send_without_response(Packet::new(transaction, false))?;
        Ok(transaction.into())
    }

    pub fn submit_work(&self, solution: WorkSolution) -> Result<()> {
        let mut rng = rand::thread_rng();
        let mut message: BroadcastMessage = solution.into();
        let mut shared_key_and_gamming_nonce = [0u64; 8];
        let mut gamming_key: [u64; 4];
        
        loop {
            unsafe {
                message.gamming_nonce.0 = rng.gen();
                copy_nonoverlapping(message.gamming_nonce.0.as_ptr(), shared_key_and_gamming_nonce.as_mut_ptr().add(4) as *mut u8, 32);
                let mut kg = KangarooTwelve::hash(&shared_key_and_gamming_nonce.iter().map(|i| i.to_le_bytes()).collect::<Vec<_>>().flatten(), &[]);
                let mut gk = [0; 32];
                kg.squeeze(&mut gk);
                gamming_key = std::array::from_fn(|i| u64::from_le_bytes(gk[i*8..i*8 + 8].try_into().unwrap()));
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
            message.solution_nonce.0[i] = solution.nonce.0[i] ^ gamma[i];
        }

        for sig in message.signature.0.iter_mut() {
            *sig = rng.gen();
        }

        self.transport.send_without_response(Packet::new(message, false))?;
        Ok(())
    }

    pub fn get_current_tick_info(&self) -> Result<CurrentTickInfo> {
        let packet = Packet::new(GetCurrentTickInfo, true);

        Ok(self.transport.send_with_response(packet)?)
    }

    pub fn request_computors(&self) -> Result<Computors> {
        let packet = Packet::new(RequestComputors, true);
        
        Ok(self.transport.send_with_response(packet)?)
    }

    pub fn request_entity(&self, public_key: QubicId) -> Result<Entity> {
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

    pub fn request_quorum_tick(&self, tick: u32, vote_flags: [u8; (676 + 7) / 8]) -> Result<Tick> {
        let packet = Packet::new(QuorumTickData { tick, vote_flags }, true);
        
        Ok(self.transport.send_with_response(packet)?)
    }

    pub fn exchange_public_peers(&self, peers: [Ipv4Addr; 4]) -> Result<ExchangePublicPeers> {
        let packet = Packet::new(ExchangePublicPeers { peers }, true);

        Ok(self.transport.send_with_response(packet)?)
    }

    pub fn request_tick_transactions(&self, tick: u32, flags: TransactionFlags) -> Result<Vec<TransactionWithData>> {
        let packet = Packet::new(RequestedTickTransactions { tick, flags }, true);

        Ok(self.transport.send_with_multiple_responses(packet)?)
    }

    pub fn check_transaction_status(&self, tx_hash: QubicTxHash, tick: u32) -> Result<TransactionStatus> {
        let mut status = TransactionStatus::Failed;

        let tt_packet = Packet::new(RequestedTickTransactions { tick, flags: TransactionFlags::all() }, true);
        let tt = self.transport.send_with_multiple_raw_responses(tt_packet)?;

        for tx in tt {
            let mut digest = QubicTxHash::default();
            let mut kg = KangarooTwelve::hash(&tx, &[]);
            kg.squeeze(&mut digest.0);
            if digest == tx_hash {
                status = TransactionStatus::Included;
                break;
            }
        }

        let td = self.request_tick_data(tick)?;

        for executed_tx_hash in td.transaction_digest {
            if executed_tx_hash == tx_hash {
                status = TransactionStatus::Executed;
                break;
            }
        }
        
        Ok(status)
    }

    pub fn subscribe<F>(&self, public_peers: ExchangePublicPeers, event_handler: F) -> Result<()> 
        where F: Fn(NetworkEvent) -> Result<()> + Send + Sync + 'static
    {
        let url = self.transport.get_url();
        let _: JoinHandle<Result<()>> = std::thread::Builder::new().name("qubic-event-handler".to_string()).stack_size(10_000_000).spawn(move || {
            let event_handler = event_handler;
            let url = url;
            let transport = T::new(url.clone());
            
            let mut header_buffer = vec![0u8; std::mem::size_of::<Header>()];
            let mut data_buffer = vec![0u8; 10_000_000];
            

            'connection: loop {
                let mut stream = transport.connect()?;
                stream.set_read_timeout(Some(Duration::from_secs(5)))?;
                stream.write_all(&Packet::new(public_peers, true).to_bytes())?;
                loop {
                    match stream.read_exact(&mut header_buffer) {
                        Err(_) => continue 'connection,
                        _ => ()
                    };

                    let header = unsafe {
                        read_unaligned(header_buffer.as_ptr() as *const Header)
                    };

                    match stream.read_exact(&mut data_buffer[0..(header.get_size() - std::mem::size_of::<Header>())]) {
                        Err(_) => continue 'connection,
                        _ => ()
                    };

                    match header.message_type {
                        MessageType::ExchangePublicPeers => {
                            event_handler(NetworkEvent::ExchangePublicPeers(unsafe { read_unaligned(data_buffer.as_ptr() as *const ExchangePublicPeers) }))?;
                        },
                        MessageType::BroadcastMessage => {
                            
                            event_handler(NetworkEvent::BroadcastMessage(unsafe { read_unaligned(data_buffer.as_ptr() as *const BroadcastMessage) }))?;
                        },
                        MessageType::BroadcastTransaction => {
                            let tx = TransactionWithData::from_bytes(&data_buffer[..header.get_size() - std::mem::size_of::<Header>()])?;

                            event_handler(NetworkEvent::BroadcastTransaction(tx))?;
                        },
                        MessageType::BroadcastTick => {
                            event_handler(NetworkEvent::BroadcastTick(unsafe { read_unaligned(data_buffer.as_ptr() as *const Tick) }))?;
                        },
                        MessageType::BroadcastFutureTickData => {
                            event_handler(NetworkEvent::BroadcastFutureTick(unsafe { read_unaligned(data_buffer.as_ptr() as *const TickData) }))?;
                        }
                        _ => ()
                    }
                }
            }
            
        })?;
        
        Ok(())
    }

    pub fn make_ipo_bid(&self, wallet: &QubicWallet, contract_index: u32, price_per_share: u64, number_of_shares: u16, tick: u32) -> Result<QubicTxHash> {
        let mut dst = QubicId::default();

        dst.0[0..4].copy_from_slice(&contract_index.to_le_bytes());

        let raw_call = RawCall {
            tx: RawTransaction {
                from: wallet.public_key,
                to: dst,
                amount: 0,
                tick,
                input_type: 0,
                input_size: std::mem::size_of::<ContractIpoBid>() as u16
            },
            input: ContractIpoBid {
                price: price_per_share,
                quantity: number_of_shares
            }
        };

        let call = Call {
            raw_call,
            signature: wallet.sign(raw_call)
        };

        let packet = Packet::new(call, false);

        self.transport.send_without_response(packet)?;
        Ok(call.into())
    }
}

pub struct Qx<'a, T: Transport> {
    transport: &'a T
}

#[cfg(not(any(feature = "async", feature = "http")))]
impl<'a, T: Transport> Qx<'a, T> {
    pub fn request_owned_assets(&self, id: QubicId) -> Result<Vec<RespondOwnedAsset>> {
        let packet = Packet::new(RequestOwnedAsset { public_key: id }, true);

        Ok(self.transport.send_with_multiple_responses(packet)?)
    }

    pub fn request_issued_assets(&self, id: QubicId) -> Result<Vec<RespondIssuedAsset>> {
        let packet = Packet::new(RequestIssuedAsset { public_key: id }, true);

        Ok(self.transport.send_with_multiple_responses(packet)?)
    }

    pub fn request_possessed_assets(&self, id: QubicId) -> Result<Vec<RespondPossessedAsset>> {
        let packet = Packet::new(RequestPossessedAsset { public_key: id }, true);

        Ok(self.transport.send_with_multiple_responses(packet)?)
    }

    pub fn transfer_qx_share(&self, wallet: &QubicWallet, possessor: QubicId, to: QubicId, units: i64, tick: u32) -> Result<QubicTxHash> {
        let tx = RawTransaction {
            from: wallet.public_key,
            to: QXID,
            amount: TRANSFER_FEE,
            tick,
            input_type: 2,
            input_size: std::mem::size_of::<TransferAssetOwnershipAndPossessionInput>() as u16
        };

        let mut call = Call {
            raw_call: RawCall {
                tx,
                input: TransferAssetOwnershipAndPossessionInput {
                    possessor,
                    issuer: QubicId::default(),
                    new_owner: to,
                    asset_name: AssetName::from_str("QX")?,
                    number_of_units: units
                }
            },
            signature: Signature::default()
        };

        call.signature = wallet.sign(call.raw_call);

        let packet = Packet::new(call, false);

        self.transport.send_without_response(packet)?;

        Ok(call.into())
    }

    pub fn issue_asset(&self, wallet: &QubicWallet, name: &str, unit_of_measurement: [u8; 7], number_of_units: i64, number_of_decimal_places: i8, tick: u32) -> Result<QubicTxHash> {
        let tx = RawTransaction {
            from: wallet.public_key,
            to: QXID,
            amount: ISSUE_ASSET_FEE,
            tick,
            input_type: 1,
            input_size: std::mem::size_of::<IssueAssetInput>() as u16
        };

        let mut padded_uom = [0; 8];

        padded_uom[..7].copy_from_slice(&unit_of_measurement);

        let mut call = Call {
            raw_call: RawCall {
                tx,
                input: IssueAssetInput {
                    name: AssetName::from_str(name)?,
                    number_of_units,
                    unit_of_measurement: u64::from_le_bytes(padded_uom),
                    number_of_decimal_places
                }
            },
            signature: Signature::default()
        };

        call.signature = wallet.sign(call.raw_call);

        let packet = Packet::new(call, false);
        self.transport.send_without_response(packet)?;

        Ok(call.into())
    }

    pub fn transfer_asset(&self, wallet: &QubicWallet, possessor: QubicId, issuer: QubicId, to: QubicId, name: &str, units: i64, tick: u32) -> Result<QubicTxHash> {
        let tx = RawTransaction {
            from: wallet.public_key,
            to: QXID,
            amount: 1_000_000,
            tick,
            input_type: 2,
            input_size: std::mem::size_of::<TransferAssetOwnershipAndPossessionInput>() as u16
        };

        let mut call = Call {
            raw_call: RawCall {
                tx,
                input: TransferAssetOwnershipAndPossessionInput {
                    possessor,
                    issuer,
                    new_owner: to,
                    asset_name: AssetName::from_str(name)?,
                    number_of_units: units
                }
            },
            signature: Signature::default()
        };

        call.signature = wallet.sign(call.raw_call);

        let packet = Packet::new(call, false);

        self.transport.send_without_response(packet)?;
        Ok(call.into())
    }
}

#[cfg(any(feature = "async", feature = "http"))]
impl<'a, T> Qu<'a, T> where T: Transport {
    pub async fn send_raw_transaction(&self, wallet: &QubicWallet, raw_transaction: RawTransaction) -> Result<()> {
        
        let transaction = Transaction {
            raw_transaction,
            signature: wallet.sign(&raw_transaction)
        };

        self.transport.send_without_response(Packet::new(transaction, false)).await?;
        Ok(())
    }

    pub async fn send_signed_transaction(&self, transaction: Transaction) -> Result<()> {
        self.transport.send_without_response(Packet::new(transaction, false)).await?;
        Ok(())
    }

    pub async fn submit_work(&self, solution: WorkSolution) -> Result<()> {
        let mut rng = rand::thread_rng();
        let mut message: BroadcastMessage = solution.into();
        let mut shared_key_and_gamming_nonce = [0u64; 8];
        let mut gamming_key = [0u64; 4];

        loop {
            unsafe {
                message.gamming_nonce.0 = rng.gen();
                copy_nonoverlapping(message.gamming_nonce.0.as_ptr(), shared_key_and_gamming_nonce.as_mut_ptr().add(4) as *mut u8, 32);
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
            message.solution_nonce.0[i] = solution.nonce.0[i] ^ gamma[i];
        }

        for sig in message.signature.0.iter_mut() {
            *sig = rng.gen();
        }

        self.transport.send_without_response(Packet::new(message, true)).await?;
        Ok(())
    }

    pub async fn get_current_tick_info(&self) -> Result<CurrentTickInfo> {
        let packet = Packet::new(GetCurrentTickInfo, true);

        Ok(self.transport.send_with_response(packet).await?)
    }

    pub async fn request_computors(&self) -> Result<Computors> {
        let packet = Packet::new(RequestComputors, true);
        
        Ok(self.transport.send_with_response(packet).await?)
    }

    pub async fn request_entity(&self, public_key: QubicId) -> Result<Entity> {
        let packet = Packet::new(RequestEntity { public_key }, true);
        
        Ok(self.transport.send_with_response(packet).await?)
    }

    pub async fn request_contract_ipo(&self, contract_index: u32) -> Result<ContractIpo> {
        let packet = Packet::new(RequestContractIpo { contract_index }, true);
        
        Ok(self.transport.send_with_response(packet).await?)
    }

    pub async fn request_tick_data(&self, tick: u32) -> Result<Tick> {
        let packet = Packet::new(RequestTickData { tick }, true);
    
        Ok(self.transport.send_with_response(packet).await?)
    }

    pub async fn request_quorum_tick(&self, tick: u32, vote_flags: [u8; (676 + 7) / 8]) -> Result<Tick> {
        let packet = Packet::new(QuorumTickData { tick, vote_flags }, true);
        
        Ok(self.transport.send_with_response(packet).await?)
    }

    pub async fn exchange_public_peers(&self, peers: [Ipv4Addr; 4]) -> Result<ExchangePublicPeers> {
        let packet = Packet::new(ExchangePublicPeers { peers }, true);

        Ok(self.transport.send_with_response(packet).await?)
    }

    pub async fn request_tick_transactions(&self, tick: u32, flags: TransactionFlags) -> Result<Vec<Transaction>> {
        let packet = Packet::new(RequestedTickTransactions { tick, flags }, true);

        Ok(self.transport.send_with_multiple_responses(packet).await?)
    }

    pub async fn subscribe<F>(&self, public_peers: ExchangePublicPeers, event_handler: F) -> Result<()> 
        where F: Fn(NetworkEvent) -> Result<()> + Send + Sync + 'static
    {
        let stream = self.transport.connect().await?;
        let _: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
            let event_handler = event_handler;
            let mut stream = stream;
            let mut header_buffer = vec![0u8; std::mem::size_of::<Header>()];
            let mut data_buffer = vec![0u8; 10_000_000];
            stream.write_all(Packet::new(public_peers, true).encode_as_bytes()).await?;

            loop {
                stream.read_exact(&mut header_buffer).await?;

                let header = unsafe {
                    read_unaligned(header_buffer.as_ptr() as *const Header)
                };

                stream.read_exact(&mut data_buffer[0..(header.get_size() - std::mem::size_of::<Header>())]).await?;



                match header.message_type {
                    MessageType::ExchangePublicPeers => {
                        event_handler(NetworkEvent::ExchangePublicPeers(unsafe { read_unaligned(data_buffer.as_ptr() as *const ExchangePublicPeers) }))?;
                    },
                    MessageType::BroadcastMessage => {
                        event_handler(NetworkEvent::BroadcastMessage(unsafe { read_unaligned(data_buffer.as_ptr() as *const BroadcastMessage) }))?;
                    },
                    MessageType::BroadcastTransaction => {
                        event_handler(NetworkEvent::BroadcastTransaction(unsafe { read_unaligned(data_buffer.as_ptr() as *const Transaction) }))?;
                    },
                    MessageType::BroadcastTick => {
                        event_handler(NetworkEvent::BroadcastTick(unsafe { read_unaligned(data_buffer.as_ptr() as *const Tick) }))?;
                    },
                    MessageType::BroadcastFutureTickData => {
                        event_handler(NetworkEvent::BroadcastFutureTick(unsafe { read_unaligned(data_buffer.as_ptr() as *const TickData) }))?;
                    }
                    _ => ()
                }
            }
        });
        
        Ok(())
    }
}