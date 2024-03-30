use crate::{types::{BroadcastMessage, ExchangePublicPeers}, prelude::{Tick, TickData, TransactionWithData}};

#[derive(Debug)]
pub enum NetworkEvent {
    ExchangePublicPeers(ExchangePublicPeers),
    BroadcastMessage(BroadcastMessage),
    BroadcastTransaction(TransactionWithData),
    BroadcastTick(Tick),
    BroadcastFutureTick(Box<TickData>)
}