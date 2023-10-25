use qubic_tcp_types::types::{CurrentTickInfo, Entity, Transaction};
use qubic_types::{QubicId, Signature};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "camelCase")]
pub enum JsonRpcRequest {
    RequestCurrentTickInfo { jsonrpc: String, id: usize },
    /// params: qubic ID
    RequestEntity { jsonrpc: String, id: usize, params: QubicId },

    RequestComputors { jsonrpc: String, id: usize },

    SendTransaction { jsonrpc: String, id: usize, params: Transaction },

    /// params: index of IPO contract
    RequestContractIpo { jsonrpc: String, id: usize, params: u32 },

    /// params: requested tick
    RequestTickData { jsonrpc: String, id: usize, params: u32 }
}



impl JsonRpcRequest {
    pub fn is_version_2(&self) -> bool {
        match self {
            Self::RequestCurrentTickInfo { jsonrpc, id: _ } => {
                jsonrpc.as_str() == "2.0"
            },
            Self::RequestEntity { jsonrpc, id: _, params: _ } => {
                jsonrpc.as_str() == "2.0"
            },
            Self::SendTransaction { jsonrpc, id: _, params: _ } => {
                jsonrpc.as_str() == "2.0"
            }
            Self::RequestComputors { jsonrpc, id: _ } => {
                jsonrpc.as_str() == "2.0"
            },
            Self::RequestContractIpo { jsonrpc, id: _, params: _ } => {
                jsonrpc.as_str() == "2.0"
            },
            Self::RequestTickData { jsonrpc, id: _, params: _ } => {
                jsonrpc.as_str() == "2.0"
            }
        }
    }

    pub fn init_error(&self, error: Option<String>) -> JsonRpcResponse {
        match self {
            Self::RequestCurrentTickInfo { jsonrpc: _, id } => {
                JsonRpcResponse::RequestCurrentTickInfo { jsonrpc: "2.0".to_string(), id: *id, result: None, error}
            },
            Self::RequestEntity { jsonrpc: _, id, params: _ } => {
                JsonRpcResponse::RequestEntity { jsonrpc: "2.0".to_string(), id: *id, result: None, error }
            },
            Self::SendTransaction { jsonrpc: _, id, params: _ } => {
                JsonRpcResponse::SendTransaction { jsonrpc: "2.0".to_string(), id: *id, error }
            }
            Self::RequestComputors { jsonrpc: _, id } => {
                JsonRpcResponse::RequestComputors { jsonrpc: "2.0".to_string(), id: *id, result: None, error }
            },
            Self::RequestContractIpo { jsonrpc: _, id, params: _ } => {
                JsonRpcResponse::RequestContractIpo { jsonrpc: "2.0".to_string(), id: *id, error }
            },
            Self::RequestTickData { jsonrpc: _, id, params: _ } => {
                JsonRpcResponse::RequestTickData { jsonrpc: "2.0".to_string(), id: *id, error }
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "camelCase")]
pub enum JsonRpcResponse {
    RequestCurrentTickInfo { jsonrpc: String, id: usize, result: Option<CurrentTickInfo>, error: Option<String> },

    RequestEntity { jsonrpc: String, id: usize, result: Option<Entity>, error: Option<String> },

    SendTransaction { jsonrpc: String, id: usize, error: Option<String> },

    RequestComputors { jsonrpc: String, id: usize, result: Option<ComputorInfos>, error: Option<String> },

    RequestContractIpo { jsonrpc: String, id: usize, error: Option<String> },

    RequestTickData { jsonrpc: String, id: usize, error: Option<String> }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputorInfos {
    pub epoch: u16,
    pub ids: Vec<QubicId>,
    pub signature: Signature
}

impl Into<ComputorInfos> for qubic_tcp_types::types::Computors {
    fn into(self) -> ComputorInfos {
        ComputorInfos {
            epoch: self.epoch,
            ids: self.public_key.to_vec(),
            signature: self.signature
        }
    }
}