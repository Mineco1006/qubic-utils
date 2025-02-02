use axum::{
    extract::Path,
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use http::status::StatusCode;
use qubic_rs::{
    client::Client,
    qubic_tcp_types::types::{ticks::CurrentTickInfo, transactions::Transaction},
    qubic_types::{QubicId, QubicWallet},
    transport::Tcp,
};
use std::sync::Arc;

use crate::RPCState;

struct QubicRpcError(anyhow::Error);

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

pub async fn index() -> impl IntoResponse {
    Json("Qubic RPC API v2".to_string())
}
pub async fn latest_tick(
    State(state): State<Arc<RPCState>>,
) -> Result<CurrentTickInfo, QubicRpcError> {
    let client = Client::<Tcp>::new(&state.computor_address).await?;
    Ok(client.qu().get_current_tick_info().await?)
}
pub async fn broadcast_transaction() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
}
pub async fn wallet_balance(Path(_wallet): Path<QubicWallet>) -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Not implemented yet")
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
