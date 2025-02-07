use qubic_rs::{
    qubic_tcp_types::types::{ticks::CurrentTickInfo, Computors, RespondedEntity},
    qubic_types::{QubicId, Signature},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputorInfos {
    pub epoch: u16,
    pub ids: Vec<QubicId>,
    pub signature: Signature,
}

impl From<Computors> for ComputorInfos {
    fn from(value: Computors) -> Self {
        ComputorInfos {
            epoch: value.epoch,
            ids: value.public_key.to_vec(),
            signature: value.signature,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestTick {
    pub latest_tick: u32,
}

impl From<CurrentTickInfo> for LatestTick {
    fn from(tick_info: CurrentTickInfo) -> Self {
        Self {
            latest_tick: tick_info.tick,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastTransactionPayload {
    pub encoded_transaction: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    id: QubicId,
    // string this is required for compatibility with main api
    balance: String,
    pub valid_for_tick: u32,
    latest_incoming_transfer_tick: u32,
    latest_outgoing_transfer_tick: u32,
    // string this is required for compatibility with main api
    incoming_amount: String,
    // string this is required for compatibility with main api
    outgoing_amount: String,
    number_of_incoming_transfers: u32,
    number_of_outgoing_transfers: u32,
}

impl From<RespondedEntity> for Balance {
    fn from(entity: RespondedEntity) -> Self {
        Self {
            id: entity.entity.public_key,
            balance: entity.entity.balance().to_string(),
            valid_for_tick: entity.tick,
            latest_incoming_transfer_tick: entity.entity.latest_incoming_transfer_tick,
            latest_outgoing_transfer_tick: entity.entity.latest_outgoing_transfer_tick,
            incoming_amount: entity.entity.incoming_amount.to_string(),
            outgoing_amount: entity.entity.outgoing_amount.to_string(),
            number_of_incoming_transfers: entity.entity.number_of_incoming_transfers,
            number_of_outgoing_transfers: entity.entity.number_of_outgoing_transfers,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalance {
    pub balance: Balance,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponseData {
    pub source_id: String,
    pub dest_id: String,
    pub amount: String,
    pub tick_number: u32,
    pub input_type: u32,
    pub input_size: u32,
    pub input_hex: String,
    pub signature_hex: String,
    pub tx_id: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub transaction: TransactionResponseData,
    pub timestamp: String,
    pub money_flew: bool,
}

// TODO: implement
// impl From<RespondedEntity> for TransactionResponse {
//     fn from(entity: RespondedEntity) -> Self {
//         Self {
//             transaction: TransactionResponseData {
//                 source_id: (),
//                 dest_id: (),
//                 amount: entity.entity.balance().to_string(),
//                 tick_number: entity.tick,
//                 input_type: (),
//                 input_size: (),
//                 input_hex: (),
//                 signature_hex: (),
//                 tx_id: entity.public_key.to_string(),
//             },
//             timestamp: 0,
//             money_flew: false,
//         }
//     }
// }

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum APIStatus {
    Ok,
    Error,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RPCStatus {
    /// Server status: `"ok"` if healthy, `"error"` otherwise
    pub status: APIStatus,
    /// Uptime in seconds
    pub uptime: u64,
    /// Qubic RPC version (v2)
    pub version: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastProcessedTick {
    pub tick_number: u64,
    pub epoch: u64,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedTick {
    pub start_tick: u64,
    pub end_tick: u64,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedTickIntervalPerEpoch {
    pub epoch: u64,
    pub intervals: Vec<TickInterval>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInterval {
    pub initial_processed_tick: u64,
    pub last_processed_tick: u64,
}