use base64::Engine;
use qubic_rs::{
    qubic_tcp_types::types::{
        ticks::CurrentTickInfo, transactions::TransactionWithData, Computors, RespondedEntity,
    },
    qubic_types::{traits::ToBytes, QubicId, Signature},
};
use serde::de::{self, Deserializer};
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
pub struct RequestSCPayload {
    #[serde(deserialize_with = "deserialize_u32")]
    pub contract_index: u32,

    #[serde(deserialize_with = "deserialize_u16")]
    pub input_type: u16,

    #[serde(deserialize_with = "deserialize_u16")]
    pub input_size: u16,

    #[serde(deserialize_with = "deserialize_base64")]
    pub request_data: Vec<u8>,
}

// Deserialize u32 from a string
fn deserialize_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u32>().map_err(de::Error::custom)
}

// Deserialize u16 from a string
fn deserialize_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<u16>().map_err(de::Error::custom)
}

// Deserialize base64-encoded input string to Vec<u8>
fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(de::Error::custom)
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
    pub input_type: u16,
    pub input_size: u16,
    pub input_hex: String,
    pub signature_hex: String,
    pub tx_id: String,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub transactions: Vec<TransactionResponseData>,
    pub timestamp: String,
    pub money_flew: bool,
}

impl From<TransactionWithData> for TransactionResponseData {
    fn from(tx: TransactionWithData) -> Self {
        Self {
            source_id: tx.raw_transaction.from.to_string(),
            dest_id: tx.raw_transaction.to.to_string(),
            amount: tx.raw_transaction.amount.to_string(),
            tick_number: tx.raw_transaction.tick,
            input_type: tx.raw_transaction.input_type,
            input_size: tx.raw_transaction.input_size,
            input_hex: hex::encode(tx.data.to_bytes()),
            signature_hex: tx.signature.to_string(),
            tx_id: "".to_string(), // TODO: find tx id
        }
    }
}

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
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferRequest {
    pub start_tick: Option<u32>,
    pub end_tick: Option<u32>,
}
pub type TransferResponse = TransactionResponse;