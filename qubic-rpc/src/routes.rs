use axum::{
    extract::Path,
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use base64::Engine;
use http::status::StatusCode;
use qubic_rs::{
    client::Client,
    qubic_tcp_types::types::{
        transactions::{Transaction, TransactionWithData},
        RespondedEntity,
    },
    qubic_types::{traits::FromBytes, QubicId, QubicWallet},
    transport::Tcp,
};
use serde::{Deserialize, Serialize};
use std::{str::FromStr, sync::Arc};

use crate::{qubic_rpc_types::LatestTick, RPCState};

pub struct QubicRpcError(anyhow::Error);

impl IntoResponse for QubicRpcError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self.0)).into_response()
    }
}

// Enables using `?` on functions that return `Result<_, anyhow::Error>`
impl<E> From<E> for QubicRpcError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastTransactionPayload {
    encoded_transaction: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WalletBalance {
    pub balance: Balance,
}

pub async fn index() -> impl IntoResponse {
    Json("Qubic RPC API v2".to_string())
}
#[axum::debug_handler]
pub async fn latest_tick(
    State(state): State<Arc<RPCState>>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let client = Client::<Tcp>::new(&state.computor_address).await?;
    let latest_tick_resp: LatestTick = client.qu().get_current_tick_info().await?.into();
    Ok(Json(latest_tick_resp))
}
pub async fn broadcast_transaction(
    State(state): State<Arc<RPCState>>,
    Json(payload): Json<BroadcastTransactionPayload>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let client = Client::<Tcp>::new(&state.computor_address).await?;

    let tx = Transaction::from_bytes(
        &base64::engine::general_purpose::STANDARD.decode(payload.encoded_transaction)?,
    )?;
    let _ = client.qu().send_signed_transaction(tx).await?;

    Ok(Json("Broadcast successful"))
}
pub async fn wallet_balance(
    State(state): State<Arc<RPCState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let public_key = QubicId::from_str(&id)?;
    let client = Client::<Tcp>::new(&state.computor_address).await?;
    let entity_response = client.qu().request_entity(public_key).await?;
    let balance: Balance = entity_response.into();
    Ok(Json(WalletBalance { balance }))
}
pub async fn status() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn transaction(Path(_tx): Path<Transaction>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn transaction_status(Path(_tx): Path<Transaction>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn transfer_transactions_per_tick(Path(_id): Path<QubicId>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn computors(Path(_epoch): Path<u32>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn query_sc() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn tick_info() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn block_height() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn latest_stats() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn rich_list() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}

pub async fn approved_transactions_for_tick(Path(_tick): Path<u32>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn tick_data(Path(_tick): Path<u32>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn chain_hash(Path(_tick): Path<u32>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn quorum_tick_data(Path(_tick): Path<u32>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn store_hash(Path(_tick): Path<u32>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}

pub async fn issued_assets(Path(_identity): Path<QubicId>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn owned_assets(Path(_identity): Path<QubicId>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn possessed_assets(Path(_identity): Path<QubicId>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
