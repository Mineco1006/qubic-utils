//! qubic-rpc is an RPC server for Qubic built on top of qubic-rs
//!
//! # Methods
//!
//! - This method
//! ```rust,no_run
//! ```
use std::{sync::Arc, time::Instant};

use async_channel::{Receiver, Sender};
use axum::{
    http::Method,
    routing::{get, post},
    Router,
};
use log::{debug, info};
use sled::Db;
use tokio::{
    net::TcpListener,
    task::{JoinHandle, JoinSet},
};
use tower_http::cors::{Any, CorsLayer};

use qubic_rs::{
    client::Client, qubic_tcp_types::types::transactions::TransactionFlags,
    qubic_types::QubicTxHash, transport::Tcp,
};

pub mod qubic_rpc_types;
pub mod routes;

#[derive(Clone)]
pub struct RPCState {
    db: Arc<Db>,
    client: Arc<Client<Tcp>>,
    start_time: Instant,
}

impl RPCState {
    pub fn new(db: Arc<Db>, client: Arc<Client<Tcp>>) -> Self {
        let start_time = Instant::now();
        Self {
            db,
            client,
            start_time,
        }
    }
}

/// Producer puts ticks into a channel for processing.
/// Produces 10 ticks backwards and then checks for new ticks
/// (e.g. from 1010 it will add 1009 to 999 and then check to see where the network is at,
/// if it is at 1012, it'll add 1011 and 1012)
async fn archiver_producer(tx: Sender<u32>, client: Arc<Client<Tcp>>) {
    let current_tick = client
        .qu()
        .get_current_tick_info()
        .await
        .expect("Could not get current tick")
        .tick;

    if let Err(_) = tx.send(current_tick).await {
        eprintln!("Receiver dropped, stopping producer.");
        return;
    }

    let mut latest_viewed_tick = current_tick;
    let mut earliest_viewed_tick = current_tick;
    loop {
        // add 10 ticks backwards
        if earliest_viewed_tick > 0 {
            for i in 1..=10 {
                let new_tick = earliest_viewed_tick.saturating_sub(i);
                if let Err(_) = tx.send(new_tick).await {
                    eprintln!("Receiver dropped, stopping producer.");
                    return;
                }
                earliest_viewed_tick = new_tick;
            }
        }

        // check for forward ticks
        let current_tick = client
            .qu()
            .get_current_tick_info()
            .await
            .expect("Could not get current tick")
            .tick;

        // add all ticks between current and latest_viewed_tick
        for tick in (latest_viewed_tick + 1)..=current_tick {
            if let Err(_) = tx.send(tick).await {
                eprintln!("Receiver dropped, stopping producer.");
                return;
            }
        }
        latest_viewed_tick = current_tick;

        // wait not to overload channel with mostly older ticks
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn archiver_consumer(id: usize, rx: Receiver<u32>, db: Arc<Db>, client: Arc<Client<Tcp>>) {
    while let Ok(tick) = rx.recv().await {
        let tick_transactions = client
            .qu()
            .request_tick_transactions(tick, TransactionFlags::all())
            .await
            .expect("Could not get transactions for tick {tick}");

        for tx in tick_transactions {
            let serialized = bincode::serialize(&tx).unwrap();
            let tx_hash: QubicTxHash = tx.into();
            let tx_id = tx_hash.get_identity();

            db.insert(tx_id, serialized).unwrap();
        }
        debug!("Consumer {} processed tick {}", id, tick);
    }
}

pub async fn spawn_server(
    port: u32,
    computor_address: String,
    db_file: String,
) -> (JoinSet<()>, JoinHandle<()>) {
    let db = Arc::new(sled::open(db_file).expect("Failed to open DB"));
    let client = Arc::new(
        Client::<Tcp>::new(&computor_address)
            .await
            .expect("Failed to create qubic client"),
    );
    let state = Arc::new(RPCState::new(db.clone(), client.clone()));

    let mut archiver_handles = JoinSet::new();

    // spawn producer (sends ticks) and consumers (fetch tick transactions) for archiver
    let (tx, rx) = async_channel::unbounded();
    archiver_handles.spawn(archiver_producer(tx, client.clone()));
    for client_id in 0..4 {
        let rx_clone = rx.clone();
        let db_clone = db.clone();
        let client_clone = client.clone();
        archiver_handles.spawn(async move {
            archiver_consumer(client_id, rx_clone, db_clone, client_clone).await;
        });
    }

    let routes = qubic_rpc_router_v2(state.clone());

    info!("Binding server to port {}", port);
    let tcp_listener = TcpListener::bind(&format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    let server_handle = tokio::spawn(async move {
        axum::serve(tcp_listener, routes.with_state(state))
            .await
            .unwrap();
    });

    (archiver_handles, server_handle)
}

pub fn qubic_rpc_router_v2<S>(state: Arc<RPCState>) -> Router<S> {
    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let ticks_router = Router::new()
        .route("/{tick}/transactions", get(routes::tick_transactions))
        .route(
            "/{tick}/approved-transactions",
            get(routes::approved_tick_transactions),
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
        .route("/identities/{id}/transfers", get(routes::transfers))
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use axum::body::Body;
    use base64::Engine;
    use http::{Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use serde_json::json;
    use tower::ServiceExt; // for `call`, `oneshot`, and `ready` // for `collect`

    use qubic_rs::{
        client::Client,
        qubic_tcp_types::types::{
            contracts::ResponseContractFunction, transactions::TransactionWithData,
        },
        qubic_types::{
            traits::{Sign, ToBytes},
            QubicWallet,
        },
        transport::Tcp,
    };

    use crate::{
        qubic_rpc_router_v2,
        qubic_rpc_types::{APIStatus, LatestTick, RPCStatus, TransferResponse, WalletBalance},
        RPCState,
    };

    const COMPUTOR_ADDRESS: &str = "66.23.193.243:21841";

    async fn setup() -> Arc<RPCState> {
        let db = Arc::new(
            sled::Config::new()
                .temporary(true)
                .open()
                .expect("Failed to open DB"),
        );
        let client = Arc::new(
            Client::<Tcp>::new(&COMPUTOR_ADDRESS)
                .await
                .expect("Failed to create qubic client"),
        );
        Arc::new(RPCState::new(db, client))
    }

    #[tokio::test]
    async fn get_index() {
        let state = setup().await;
        let app = qubic_rpc_router_v2(state.clone());
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        dbg!(&response);

        // should redirect to /healthcheck
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        let location_header = response.headers().get(http::header::LOCATION);
        assert!(location_header.is_some(), "Missing Location header");
        assert_eq!(location_header.unwrap(), "/healthcheck");
    }

    #[tokio::test]
    async fn latest_tick() {
        let state = setup().await;
        let app = qubic_rpc_router_v2(state.clone());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/latestTick")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&response);

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let actual: LatestTick = serde_json::from_slice(&body_bytes).unwrap();

        assert!(actual.latest_tick > 0, "Invalid latest tick");
    }

    #[tokio::test]
    async fn broadcast_transaction() {
        let state = setup().await;
        let app = qubic_rpc_router_v2(state.clone());

        let wallet =
            QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap();
        let mut tx = TransactionWithData::default();
        let _ = tx.sign(&wallet);

        let mut payload = HashMap::new();
        payload.insert(
            "encodedTransaction",
            base64::engine::general_purpose::STANDARD.encode(tx.to_bytes()),
        );
        let json_payload = serde_json::to_string(&payload).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/broadcast-transaction")
                    .header("Content-Type", "application/json")
                    .method(Method::POST)
                    .body(Body::from(json_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&response);

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn wallet_balance() {
        let state = setup().await;
        let app = qubic_rpc_router_v2(state.clone());

        let wallet = "MGPAJNYEIENVTAQXEBARMUADANKBOOWIETOVESQIDCFFVZOVHLFBYIKDWITM";

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/balances/{wallet}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&response);

        let expected: WalletBalance = serde_json::from_value(json!({"balance": {
            "id": "MGPAJNYEIENVTAQXEBARMUADANKBOOWIETOVESQIDCFFVZOVHLFBYIKDWITM",
            "balance": "0",
            "validForTick": 19511023,
            "latestIncomingTransferTick": 0,
            "latestOutgoingTransferTick": 0,
            "incomingAmount": "0",
            "outgoingAmount": "0",
            "numberOfIncomingTransfers": 0,
            "numberOfOutgoingTransfers": 0
          }}
        ))
        .unwrap();

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let mut actual: WalletBalance = serde_json::from_slice(&body_bytes).unwrap();

        // force validForTick to be equal (not checking it)
        actual.balance.valid_for_tick = expected.balance.valid_for_tick;
        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn health_check() {
        let state = setup().await;
        let app = qubic_rpc_router_v2(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/healthcheck"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&response);

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let actual: RPCStatus = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(actual.status, APIStatus::Ok);
    }

    #[tokio::test]
    async fn transfer_transactions_per_tick() {
        let state = setup().await;
        let app = qubic_rpc_router_v2(state.clone());

        let wallet_id = "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ";
        let start_tick = 19385438;
        let end_tick = 19386228;

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/identities/{wallet_id}/transfer-transactions?start_tick={start_tick}&end_tick={end_tick}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&response);

        // should redirect to /identities/{identity}/transfers
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        let location_header = response.headers().get(http::header::LOCATION);
        assert!(location_header.is_some(), "Missing Location header");
        assert_eq!(
            location_header.unwrap(),
            &format!(
                "/identities/{wallet_id}/transfers?start_tick={start_tick}&end_tick={end_tick}"
            )
        );
    }

    #[tokio::test]
    async fn transfers() {
        let state = setup().await;
        let app = qubic_rpc_router_v2(state.clone());

        let wallet_id = "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ";
        let start_tick = 19385438;
        let end_tick = 19385439;

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/identities/{wallet_id}/transfers?start_tick={start_tick}&end_tick={end_tick}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&response);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let actual: TransferResponse = serde_json::from_slice(&body_bytes).unwrap_or_else(|e| {
            eprintln!("{}\n{}", e, String::from_utf8_lossy(&body_bytes));
            panic!("Deserialization failed");
        });
        let expected: TransferResponse = serde_json::from_value(json!({
            "transferTransactionsPerTick": [
                {
                    "tickNumber": 19385438,
                    "identity": "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ",
                    "transactions": [
                        {
                            "sourceId": "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ",
                            "destId": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB",
                            "amount": "1000000",
                            "tickNumber": 19385438,
                            "inputType": 2,
                            "inputSize": 64,
                            "inputHex": "b3ca3033fcef262e35541db32ff50fdc79a2112b561856f11944dc6c07d5b404716c692b6375646194005632dfcdfe963c5b7c08ab00f446f1fd4bbb908bf1fc",
                            "signatureHex": "9ee030559a78d5f2faee3a3936397a158bdfb4836d88d8e952f17557117a4f62453c0797462bbbfc83e31377337629abdc62c72b6bc98fb6001e619513610900",
                            "txId": "yhxstxyqmofoihqxpmjkzbhpkejecdnxtmqxkguimcduomifdftpewhefvri"
                        }
                    ]
                }
            ]
        })).unwrap();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn query_sc() {
        let state = setup().await;
        let app = qubic_rpc_router_v2(state.clone());
        let mut payload = HashMap::new();
        payload.insert("contractIndex", 1.to_string());
        payload.insert("inputType", 1.to_string());
        payload.insert("inputSize", 0.to_string());
        payload.insert(
            "requestData",
            base64::engine::general_purpose::STANDARD.encode("".as_bytes()),
        );
        let json_payload = serde_json::to_string(&payload).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/querySmartContract")
                    .header("Content-Type", "application/json")
                    .method(Method::POST)
                    .body(Body::from(json_payload))
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&response);

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let actual: ResponseContractFunction = serde_json::from_slice(&body_bytes).unwrap();

        let expected = ResponseContractFunction {
            output: base64::engine::general_purpose::STANDARD
                .decode("AMqaO2QAAADAxi0A")
                .unwrap(),
        };

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn tick_info() {
        let state = setup().await;
        let app = qubic_rpc_router_v2(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/tick-info"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        dbg!(&response);

        assert_eq!(response.status(), StatusCode::OK);
    }
}