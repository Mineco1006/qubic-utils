//! qubic-rpc is an RPC server for Qubic built on top of qubic-rs
//!
//! # Methods
//!
//! - This method
//! ```rust,no_run
//! ```

use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use std::{sync::Arc, time::Instant};
use tower_http::cors::{Any, CorsLayer};

pub mod qubic_rpc_types;
pub mod routes;

#[derive(Debug, Clone)]
pub struct RPCState {
    computor_address: String,
    start_time: Instant,
}

impl RPCState {
    pub fn new(computor_address: String) -> Self {
        let start_time = Instant::now();
        Self {
            computor_address,
            start_time,
        }
    }
}

pub fn qubic_rpc_router_v2<S>(state: Arc<RPCState>) -> Router<S> {
    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let ticks_router = Router::new()
        .route(
            "/{tick}/approved-transactions",
            get(routes::approved_transactions_for_tick),
        )
        .route("/{tick}/tick-data", get(routes::tick_data))
        .route("/{tick}/chain-hash", get(routes::chain_hash))
        .route("/{tick}/quorum-tick-data", get(routes::quorum_tick_data))
        .route("/{tick}/store-hash", get(routes::store_hash));

    let assets_router = Router::new()
        .route("/{identity}/issued", get(routes::issued_assets))
        .route("/{identity}/owned", get(routes::owned_assets))
        .route("/{identity}/possessed", get(routes::possessed_assets));

    Router::new()
        .layer(cors)
        .route("/", get(routes::index))
        .route("/latestTick", get(routes::latest_tick))
        .route(
            "/broadcast-transaction",
            post(routes::broadcast_transaction),
        )
        .route("/balances/{id}", get(routes::wallet_balance))
        .route("/status", get(routes::status))
        .route("/transactions/{tx_id}", get(routes::transaction))
        .route("/tx-status/{tx_id}", get(routes::transaction_status))
        .route(
            "/identities/{id}/transfer-transactions",
            get(routes::transfer_transactions_per_tick),
        )
        .route("/identities/{id}/transfer", get(routes::transfer))
        .route("/healthcheck", get(routes::health_check))
        .route("/epochs/{epoch}/computors", get(routes::computors))
        .route("/querySmartContract", post(routes::query_sc))
        .route("/tick-info", get(routes::tick_info))
        .route("/block-height", get(routes::block_height))
        .route("/latest-stats", get(routes::latest_stats))
        .route("/rich-list", get(routes::rich_list))
        .nest("/ticks", ticks_router)
        .nest("/assets", assets_router)
        .with_state(state)
}