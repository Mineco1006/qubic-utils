use crate::{types::{BroadcastMessage, ExchangePublicPeers}, prelude::{Tick, TickData, TransactionWithData}};
use alloc::boxed::Box;

#[derive(Debug)]
pub enum NetworkEvent {
    ExchangePublicPeers(ExchangePublicPeers),
    BroadcastMessage(BroadcastMessage),
    BroadcastTransaction(TransactionWithData),
    BroadcastTick(Tick),
    BroadcastFutureTick(Box<TickData>)
}