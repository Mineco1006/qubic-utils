use crate::{types::{BroadcastMessage, ExchangePublicPeers}, prelude::{Transaction, TickData, FutureTickData}};


#[derive(Debug)]
pub enum NetworkEvent {
    ExchangePublicPeers(ExchangePublicPeers),
    BroadcastMessage(BroadcastMessage),
    BroadcastTransaction(Transaction),
    BroadcastTick(TickData),
    BroadcastFutureTick(FutureTickData)
}