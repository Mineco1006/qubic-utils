use qubic_tcp_types::types::{Packet, GetCurrentTickInfo, RequestEntity, Transaction, RawTransaction, RequestComputors, RequestContractIpo, RequestTickData};
use crate::{JsonRpcRequest, TransactionParams};

impl Into<Transaction> for TransactionParams {
    fn into(self) -> Transaction {
        Transaction { raw_transaction: RawTransaction { from: self.from, to: self.to, amount: self.amount, tick: self.tick, input_type: 0, input_size: 0 }, signature: self.signature }
    }
}

impl From<Transaction> for TransactionParams {
    fn from(value: Transaction) -> Self {
        TransactionParams { from: value.raw_transaction.from, to: value.raw_transaction.to, amount: value.raw_transaction.amount, tick: value.raw_transaction.tick, signature: value.signature }
    }
}

impl From<Packet<GetCurrentTickInfo>> for JsonRpcRequest {
    fn from(_: Packet<GetCurrentTickInfo>) -> Self {
        Self::RequestCurrentTickInfo { jsonrpc: "2.0".to_string(), id: 0 }
    }
}

impl From<Packet<RequestEntity>> for JsonRpcRequest {
    fn from(packet: Packet<RequestEntity>) -> Self {
        Self::RequestEntity { jsonrpc: "2.0".to_string(), id: 0 , params: packet.data.public_key }
    }
}

impl From<Packet<Transaction>> for JsonRpcRequest {
    fn from(packet: Packet<Transaction>) -> Self {
        Self::SendTransaction { jsonrpc: "2.0".to_string(), id: 0 , params: packet.data.into() }
    }
}

impl From<Packet<RequestComputors>> for JsonRpcRequest {
    fn from(_: Packet<RequestComputors>) -> Self {
        Self::RequestComputors { jsonrpc: "2.0".to_string(), id: 0 }
    }
}

impl From<Packet<RequestContractIpo>> for JsonRpcRequest {
    fn from(packet: Packet<RequestContractIpo>) -> Self {
        Self::RequestContractIpo { jsonrpc: "2.0".to_string(), id: 0, params: packet.data.contract_index }
    }
}

impl From<Packet<RequestTickData>> for JsonRpcRequest {
    fn from(packet: Packet<RequestTickData>) -> Self {
        Self::RequestTickData { jsonrpc: "2.0".to_string(), id: 0, params: packet.data.tick }
    }
}