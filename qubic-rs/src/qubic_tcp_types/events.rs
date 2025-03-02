use crate::qubic_tcp_types::{
    prelude::{Tick, TickData, TransactionWithData},
    types::{BroadcastMessage, ExchangePublicPeers},
};

#[derive(Debug)]
pub enum NetworkEvent {
    ExchangePublicPeers(ExchangePublicPeers),
    BroadcastMessage(BroadcastMessage),
    BroadcastTransaction(TransactionWithData),
    BroadcastTick(Tick),
    BroadcastFutureTick(Box<TickData>),
}