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

mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use axum::body::Body;
    use base64::Engine;
    use http::{Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use serde_json::json;
    use tower::{Service, ServiceExt}; // for `call`, `oneshot`, and `ready` // for `collect`

    use qubic_rs::{
        qubic_tcp_types::types::{
            contracts::ResponseContractFunction,
            ticks::CurrentTickInfo,
            transactions::{RequestedTickTransactions, TransactionWithData},
        },
        qubic_types::{
            traits::{Sign, ToBytes},
            QubicWallet,
        },
    };

    use crate::{
        qubic_rpc_router_v2,
        qubic_rpc_types::{
            APIStatus, LatestTick, RPCStatus, TransactionResponse, TransferResponse, WalletBalance,
        },
        RPCState,
    };

    const COMPUTOR_ADDRESS: &str = "66.23.193.243:21841";

    #[tokio::test]
    async fn get_index() {
        let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
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
        let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
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
        let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
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
        let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
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
        let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
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

    // #[tokio::test]
    // async fn transaction() {
    //     let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
    //     let app = qubic_rpc_router_v2(state.clone());

    //     let tx_id = "rlinciclnsqteajcanbecoedphdftskhikawqvedkfzbmiclqqnpgoagsbpb";

    //     let response = app
    //         .oneshot(
    //             Request::builder()
    //                 .uri(format!("/transactions/{tx_id}"))
    //                 .body(Body::empty())
    //                 .unwrap(),
    //         )
    //         .await
    //         .unwrap();
    //     dbg!(&response);

    //     assert_eq!(response.status(), StatusCode::OK);

    //     let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    //     let actual: TransactionResponse = serde_json::from_slice(&body_bytes).unwrap();

    //     let expected: TransactionResponse = serde_json::from_value(json!({
    //       "transaction": {
    //         "sourceId": "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ",
    //         "destId": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB",
    //         "amount": "1000000",
    //         "tickNumber": 17767809,
    //         "inputType": 2,
    //         "inputSize": 64,
    //         "inputHex": "72c56a241b10e5c982bffa7368e7280a046785e1fb659610df3c03f4508d420f716c692b637564618b025950bc2b53a778644261ade91a22c85ef752da7ee162",
    //         "signatureHex": "8ecb184c3da2dc9ee673189590846f3dea8877ad72eb04dec0be1e36791436c5b9254fd7dbe2c44352a20bed3b01973d8974320cf4a8f99c45eb662410f81300",
    //         "txId": "rlinciclnsqteajcanbecoedphdftskhikawqvedkfzbmiclqqnpgoagsbpb"
    //       }
    //     }
    //     )).unwrap();

    //     assert_eq!(expected, actual);
    // }

    #[tokio::test]
    async fn transfer_transactions_per_tick() {
        let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
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
        let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
        let app = qubic_rpc_router_v2(state.clone());

        let wallet_id = "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ";
        let start_tick = 19385438;
        let end_tick = 19386228;

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
                },
                {
                    "tickNumber": 19386228,
                    "identity": "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ",
                    "transactions": [
                        {
                            "sourceId": "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ",
                            "destId": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB",
                            "amount": "1000000",
                            "tickNumber": 19386228,
                            "inputType": 2,
                            "inputSize": 64,
                            "inputHex": "773d255d2eed904b11d4e885662662636715c7f9c87efd8ea8d5794f3c367244716c692b6375646194005972d0afe82cf85308ad47fbde3b7c6db1af526c16aa",
                            "signatureHex": "da36f546ba5bfd200460421492aae121e78ceaa9d352c5b3efad84387a69285a7834d181f98ccfe97a292cf7fa9627a2774dc151865ab328c20636ea6a870800",
                            "txId": "frvjiyacjnsnhhdpwnrhcxgucvjhhzwjovitxracuedxnarcpwxmejqdufhi"
                        }
                    ]
                }
            ]
        })).unwrap();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn query_sc() {
        let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
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
        let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));
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