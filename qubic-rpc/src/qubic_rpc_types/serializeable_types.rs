use std::collections::HashMap;

use base64::Engine;
use chrono::Utc;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

use qubic_rs::{
    qubic_tcp_types::types::{
        ticks::CurrentTickInfo, transactions::TransactionWithData, Computors, RespondedEntity,
    },
    qubic_types::{traits::ToBytes, QubicId, QubicTxHash},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastTransactionPayload {
    pub encoded_transaction: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub id: QubicId,
    // string this is required for compatibility with main api
    pub balance: String,
    pub valid_for_tick: u32,
    pub latest_incoming_transfer_tick: u32,
    pub latest_outgoing_transfer_tick: u32,
    // string this is required for compatibility with main api
    pub incoming_amount: String,
    // string this is required for compatibility with main api
    pub outgoing_amount: String,
    pub number_of_incoming_transfers: u32,
    pub number_of_outgoing_transfers: u32,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalance {
    pub balance: Balance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionsResponse {
    pub transactions: Vec<TransactionResponse>,
}
impl From<TransactionsResponse> for Vec<TransactionResponseData> {
    fn from(response: TransactionsResponse) -> Self {
        response.transactions.into_iter().map(Into::into).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    pub transaction: TransactionResponseData,
    pub timestamp: String,
    pub money_flew: bool,
}
impl From<TransactionResponse> for TransactionResponseData {
    fn from(response: TransactionResponse) -> Self {
        response.transaction
    }
}
impl From<TransactionWithData> for TransactionResponse {
    fn from(tx: TransactionWithData) -> Self {
        TransactionResponse {
            transaction: tx.into(),
            timestamp: Utc::now().timestamp().to_string(),
            money_flew: false, // TODO: implement
        }
    }
}

impl From<TransactionWithData> for TransactionResponseData {
    fn from(tx: TransactionWithData) -> Self {
        let tx_hash: QubicTxHash = tx.clone().into();
        Self {
            source_id: tx.raw_transaction.from.to_string(),
            dest_id: tx.raw_transaction.to.to_string(),
            amount: tx.raw_transaction.amount.to_string(),
            tick_number: tx.raw_transaction.tick,
            input_type: tx.raw_transaction.input_type,
            input_size: tx.raw_transaction.input_size,
            input_hex: hex::encode(tx.data.to_bytes()),
            signature_hex: tx.signature.to_string(),
            tx_id: tx_hash.get_identity(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum APIStatus {
    Ok,
    Error,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickTransactionsWrapper {
    pub tick_number: u32,
    pub identity: String,
    pub transactions: Vec<TransactionResponse>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickTransactions {
    pub transactions: Vec<TickTransactionsWrapper>,
}
impl From<TickTransactions> for Vec<TransactionResponseData> {
    fn from(response: TickTransactions) -> Self {
        response
            .transactions
            .into_iter()
            .flat_map(|tick_txs| tick_txs.transactions)
            .map(Into::into)
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeight {
    pub tick: u32,
    pub duration: u16,
    pub epoch: u16,
    pub initial_tick: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeightResponse {
    pub block_height: BlockHeight,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInfoWrapper {
    pub tick_info: TickInfo,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInfo {
    pub duration: u16,
    pub epoch: u16,
    pub tick: u32,
    pub initial_tick: u32,
}
impl From<CurrentTickInfo> for TickInfo {
    fn from(tick_info: CurrentTickInfo) -> Self {
        TickInfo {
            duration: tick_info.tick_duration,
            epoch: tick_info.epoch,
            tick: tick_info.tick,
            initial_tick: tick_info.initial_tick,
        }
    }
}

// cannot implement Eq because of f64
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestStats {
    pub timestamp: String,
    pub circulating_supply: String,
    pub active_addresses: u32,
    pub price: f64,
    pub market_cap: String,
    pub epoch: u16,
    pub current_tick: u32,
    pub ticks_in_current_epoch: u32,
    pub empty_ticks_in_current_epoch: u32,
    pub epoch_tick_quality: f32,
    pub burned_qus: String,
}

// cannot implement Eq because of f64
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestStatsWrapper {
    pub data: LatestStats,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    pub total_records: usize,
    pub total_pages: usize,
    pub current_page: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichEntity {
    pub identity: String,
    pub balance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichList {
    pub entities: Vec<RichEntity>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RichListWrapper {
    pub pagination: Pagination,
    pub epoch: u16,
    pub rich_list: RichList,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickDataWrapper {
    pub tick_data: TickData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickData {
    pub computor_index: u64,
    pub epoch: u16,
    pub tick_number: u32,
    pub timestamp: String,
    pub var_struct: String,
    pub time_lock: String,
    pub transaction_ids: Vec<String>,
    pub contract_fees: Vec<String>,
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hash {
    pub hex_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumTickDataWrapper {
    pub quorum_tick_data: QuorumTickData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumTickData {
    pub quorum_tick_structure: QuorumTickStructure,
    pub quorum_diff_per_computor: HashMap<String, QuorumDiff>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumTickStructure {
    pub epoch: u16,
    pub tick_number: u32,
    pub timestamp: String,
    pub prev_resource_testing_digest_hex: String,
    pub prev_spectrum_digest_hex: String,
    pub prev_universe_digest_hex: String,
    pub prev_computer_digest_hex: String,
    pub tx_digest_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuorumDiff {
    pub salted_resource_testing_digest_hex: String,
    pub salted_spectrum_digest_hex: String,
    pub salted_universe_digest_hex: String,
    pub salted_computer_digest_hex: String,
    pub expected_next_tick_tx_digest_hex: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    data: AssetData,
    info: AssetInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetData {
    issuer_identity: String,
    #[serde(rename = "type")]
    asset_type: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetInfo {
    tick: u32,
    universe_index: u32,
}

pub enum AssetType {
    Issued,
    Owned,
    Possessed,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                AssetType::Issued => "issued",
                AssetType::Owned => "owned",
                AssetType::Possessed => "possessed",
            }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcStatus {
    pub last_processed_tick: LastProcessedTick,
    pub last_processed_ticks_per_epoch: HashMap<String, u32>,
    pub skipped_ticks: Vec<SkippedTick>,
    pub processed_tick_intervals_per_epoch: Vec<ProcessedTickIntervalPerEpoch>,
    pub empty_ticks_per_epoch: HashMap<String, u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastProcessedTick {
    pub tick_number: u64,
    pub epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedTick {
    pub start_tick: u64,
    pub end_tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedTickIntervalPerEpoch {
    pub epoch: u64,
    pub intervals: Vec<TickInterval>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TickInterval {
    pub initial_processed_tick: u64,
    pub last_processed_tick: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcHealth {
    pub status: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputorsWrapper {
    pub computors: ComputorInfos,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputorInfos {
    pub epoch: u16,
    pub identities: Vec<QubicId>,
    pub signature_hex: String,
}

impl From<Computors> for ComputorInfos {
    fn from(value: Computors) -> Self {
        ComputorInfos {
            epoch: value.epoch,
            identities: value.public_key.to_vec(),
            signature_hex: value.signature.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartContract {
    response_data: String,
}