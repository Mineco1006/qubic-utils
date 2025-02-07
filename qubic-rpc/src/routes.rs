use axum::{
    extract::{Path, State},
    response::{IntoResponse, Redirect},
    Json,
};
use base64::Engine;
use qubic_rs::{
    client::Client,
    qubic_tcp_types::types::transactions::Transaction,
    qubic_types::{traits::FromBytes, QubicId},
    transport::Tcp,
};
use std::{str::FromStr, sync::Arc};

use crate::{
    qubic_rpc_types::{
        APIStatus, Balance, BroadcastTransactionPayload, LatestTick, QubicRpcError, RPCStatus,
        WalletBalance,
    },
    RPCState,
};

pub async fn index() -> impl IntoResponse {
    Redirect::permanent("/healthcheck")
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
/// Returns the balance of a specific wallet from the API.
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
pub async fn status(Path(_id): Path<QubicId>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
/// Returns information for a given transaction
pub async fn transaction(
    State(_state): State<Arc<RPCState>>,
    Path(_id): Path<String>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
/// Returns the same information as `/transactions/{tx_id}`
pub async fn transaction_status(Path(id): Path<String>) -> impl IntoResponse {
    Redirect::permanent(&format!("/transactions/{id}"))
}
pub async fn transfer_transactions_per_tick(
    Path(_id): Path<QubicId>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
/// Returns general health information about RPC server
pub async fn health_check(
    State(state): State<Arc<RPCState>>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(RPCStatus {
        status: APIStatus::Ok,
        uptime: state.start_time.elapsed().as_secs(),
        version: "v2".to_string(),
    }))
}
pub async fn computors(
    State(_state): State<Arc<RPCState>>,
    Path(_epoch): Path<u32>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn query_sc(
    State(_state): State<Arc<RPCState>>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn tick_info(
    State(state): State<Arc<RPCState>>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let client = Client::<Tcp>::new(&state.computor_address).await?;
    Ok(Json(client.qu().get_current_tick_info().await?))
}
pub async fn block_height() -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn latest_stats() -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn rich_list() -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}

/// Returns the approved transactions for a specific tick (block height)
pub async fn approved_transactions_for_tick(
    Path(_tick): Path<u32>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn tick_data(Path(_tick): Path<u32>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
/// Returns the chain hash (hexadecimal digest) for a specific tick number
pub async fn chain_hash(Path(_tick): Path<u32>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
/// Returns quorum data for a specific tick (block height)
pub async fn quorum_tick_data(Path(_tick): Path<u32>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn store_hash(Path(_tick): Path<u32>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}

pub async fn issued_assets(
    Path(_identity): Path<QubicId>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn owned_assets(
    Path(_identity): Path<QubicId>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn possessed_assets(
    Path(_identity): Path<QubicId>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
