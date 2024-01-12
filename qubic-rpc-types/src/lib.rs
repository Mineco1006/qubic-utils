use qubic_tcp_types::prelude::*;
use qubic_types::{QubicId, QubicTxHash};
use serde::{Serialize, Deserialize};

mod serializeable_types;

pub use serializeable_types::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "camelCase")]
pub enum RequestMethods {
    RequestCurrentTickInfo,
    RequestEntity(QubicId),
    RequestComputors,
    SendTransaction(Transaction),
    RequestTickTransactions(u32)
}

impl RequestMethods {
    pub fn get_method(&self) -> Methods {
        match self {
            Self::RequestComputors => Methods::RequestComputors,
            Self::RequestCurrentTickInfo => Methods::RequestCurrentTickInfo,
            Self::RequestEntity(_) => Methods::RequestEntity,
            Self::SendTransaction(_) => Methods::SendTransaction,
            Self::RequestTickTransactions(_) => Methods::RequestTickTransaction
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QubicJsonRpcRequest {
    pub jsonrpc: String,
    pub id: u32,
    #[serde(flatten)]
    pub request: RequestMethods
}

impl QubicJsonRpcRequest {
    pub fn new(id: u32, request: RequestMethods) -> Self {
        Self {
            jsonrpc: "2.0".to_owned(),
            id,
            request
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", content = "result", rename_all = "camelCase")]
pub enum RequestResults {
    RequestCurrentTickInfo(CurrentTickInfo),
    RequestEntity(Entity),
    RequestComputors(ComputorInfos),
    SendTransaction(QubicTxHash),

    /// This response is incomplete, transaction won't have valid signatures unless input_type == 0 && input_size == 0
    RequestTickTransactions(Vec<Transaction>)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Methods {
    RequestCurrentTickInfo,
    RequestEntity,
    RequestComputors,
    SendTransaction,
    RequestTickTransaction
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestError {
    pub method: Methods,
    pub error: String
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ResponseType {
    Error(RequestError),
    Result(RequestResults)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QubicJsonRpcResponse {
    pub jsonrpc: String,
    pub id: u32,

    #[serde(flatten)]
    pub response: ResponseType
}