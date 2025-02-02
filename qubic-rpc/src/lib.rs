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
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

pub mod qubic_rpc_types;
pub mod routes;

#[macro_export]
macro_rules! result_or_501 {
    ($handle: expr, $rpc_method: expr) => {
        match $handle {
            Ok(res) => res,
            Err(_) => {
                return Json(QubicJsonRpcResponse {
                    jsonrpc: "2.0".to_owned(),
                    id: $rpc_method.id,
                    response: ResponseType::Error(RequestError {
                        method: $rpc_method.request.get_method(),
                        error: "InternalServerError".to_owned(),
                    }),
                })
            }
        }
    };
}

#[macro_export]
macro_rules! early_return_result {
    ($res_type: expr, $rpc_method: expr) => {
        return Json(QubicJsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            id: $rpc_method.id,
            response: ResponseType::Result($res_type),
        })
    };
}

#[derive(Debug, Clone)]
pub struct RPCState {
    computor_address: String,
}

impl RPCState {
    pub fn new(computor_address: String) -> Self {
        Self { computor_address }
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

// TODO: remove altogether
// pub async fn request_handler(
//     State(state): State<Arc<RPCState>>,
//     Json(rpc_method): Json<QubicJsonRpcRequest>,
// ) -> Json<QubicJsonRpcResponse> {
//     info!("Incoming request: {rpc_method:?}");

//     if rpc_method.jsonrpc.as_str() != "2.0" {
//         return Json(QubicJsonRpcResponse {
//             jsonrpc: "2.0".to_owned(),
//             id: rpc_method.id,
//             response: ResponseType::Error(RequestError {
//                 method: rpc_method.request.get_method(),
//                 error: "Invalid JSON-RPC version found".to_owned(),
//             }),
//         });
//     }

//     let client = Client::<Tcp>::new(&state.computor).await.unwrap();

//     match rpc_method.request {
//         RequestMethods::RequestComputors => {
//             let res = result_or_501!(client.qu().request_computors().await, rpc_method);

//             early_return_result!(RequestResults::RequestComputors(res.into()), rpc_method);
//         }
//         RequestMethods::RequestCurrentTickInfo => {
//             let res = result_or_501!(client.qu().get_current_tick_info().await, rpc_method);

//             early_return_result!(RequestResults::RequestCurrentTickInfo(res), rpc_method);
//         }
//         RequestMethods::RequestEntity(id) => {
//             let res = result_or_501!(client.qu().request_entity(id).await, rpc_method);

//             early_return_result!(RequestResults::RequestEntity(res.entity), rpc_method);
//         }
//         RequestMethods::SendTransaction(tx) => {
//             result_or_501!(client.qu().send_signed_transaction(tx).await, rpc_method);

//             early_return_result!(RequestResults::SendTransaction(tx.into()), rpc_method);
//         }
//         RequestMethods::RequestTickTransactions(tick) => {
//             let res = result_or_501!(
//                 client
//                     .qu()
//                     .request_tick_transactions(tick, TransactionFlags::all())
//                     .await,
//                 rpc_method
//             );

//             early_return_result!(RequestResults::RequestTickTransactions(res), rpc_method);
//         }
//     }
// }