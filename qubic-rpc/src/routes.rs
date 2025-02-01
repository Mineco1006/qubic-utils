use axum::{extract::Path, extract::State, response::IntoResponse, Json};
use std::sync::Arc;

use crate::RPCState;

pub async fn index() -> impl IntoResponse {
    Json("Qubic RPC API v2".to_string())
}

pub async fn latest_tick() -> impl IntoResponse {
    Json("TODO".to_string())
}

pub async fn approved_transactions_for_tick(Path(_tick): Path<u32>) -> impl IntoResponse {
    Json("TODO".to_string())
}
pub async fn tick_data(Path(_tick): Path<u32>) -> impl IntoResponse {
    Json("TODO".to_string())
}
pub async fn chain_hash(Path(_tick): Path<u32>) -> impl IntoResponse {
    Json("TODO".to_string())
}
pub async fn quorum_tick_data(Path(_tick): Path<u32>) -> impl IntoResponse {
    Json("TODO".to_string())
}
pub async fn store_hash(Path(_tick): Path<u32>) -> impl IntoResponse {
    Json("TODO".to_string())
}
