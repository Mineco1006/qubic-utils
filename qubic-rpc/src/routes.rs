use axum::{extract::State, response::IntoResponse, Json};
use std::sync::Arc;

use crate::RPCState;

pub async fn index() -> impl IntoResponse {
    Json("Qubic RPC API v2".to_string())
}
