use std::sync::Arc;

use axum::{
    routing::post,
    extract::State,
    Router, Json,
};
use qubic_types::{QubicWallet, QubicId};
use qubic_web3_rs::{client::Client, transport::Tcp, qubic_tcp_types::types::transactions::{RawTransaction, Transaction}};
use qubic_rpc_types::{JsonRpcRequest, JsonRpcResponse, ComputorInfos};
use axum::http::Method;
use tower_http::cors::{CorsLayer, Any};
use clap::Parser;

#[macro_use]
extern crate log;

#[derive(Debug, Parser)]
struct Args {
    /// Binds server to provided port
    #[arg(short, long, default_value = "2003")]
    port: String,

    /// Computor to send requests
    #[arg(short, long, default_value = "167.235.118.235:21841")]
    computor: String
}

#[tokio::main]
async fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::Info).init();

    let args = Args::parse();

    let cors = CorsLayer::new()
                        .allow_methods([Method::POST])
                        .allow_origin(Any)
                        .allow_headers(Any);

    let state = Arc::new(args);

    let app = Router::new().route("/", post(request_handler)).with_state(state.clone()).layer(cors);

    info!("Binding server to port {}", state.port);
    axum::Server::bind(&format!("0.0.0.0:{}", state.port).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn request_handler(State(state): State<Arc<Args>>, Json(rpc_method): Json<JsonRpcRequest>) -> Json<JsonRpcResponse> {
    info!("Incoming request: {rpc_method:?}");
    if !rpc_method.is_version_2() {
        let res = rpc_method.init_error(Some("Invalid JSON-RPC version!".to_string()));
        return Json(res);
    }

    let client = Client::<Tcp>::new(&state.computor);

    match &rpc_method {
        JsonRpcRequest::RequestCurrentTickInfo { jsonrpc: _, id } => {
            let res = match client.qu().get_current_tick_info() {
                Ok(r) => r,
                Err(_) => return Json(rpc_method.init_error(Some("Internal Server Error".to_string())))
            };

            Json(JsonRpcResponse::RequestCurrentTickInfo { jsonrpc: "2.0".to_string(), id: *id, result: Some(res), error: None })
        },
        JsonRpcRequest::RequestEntity { jsonrpc: _, id, params } => {
            let res = match client.qu().request_entity(*params) {
                Ok(r) => r,
                Err(_) => return Json(rpc_method.init_error(Some("Internal Server Error".to_string())))
            };

            Json(JsonRpcResponse::RequestEntity { jsonrpc: "2.0".to_string(), id: *id, result: Some(res), error: None })
        },
        JsonRpcRequest::SendTransaction { jsonrpc: _, id: _, params } => {
            match client.qu().send_signed_transaction(*params) {
                Ok(_) => {
                    Json(rpc_method.init_error(None))
                },
                Err(e) => {
                    error!("{e:?}");
                    Json(rpc_method.init_error(Some("Internal Server Error".to_string())))
                }
            }
        }
        JsonRpcRequest::RequestComputors { jsonrpc: _, id } => {
            match client.qu().request_computors() {
                Ok(r) => {
                    let r: ComputorInfos = r.into();
                    Json(JsonRpcResponse::RequestComputors { jsonrpc: "2.0".to_string(), id: *id, result: Some(r), error: None })
                },
                Err(_) => Json(rpc_method.init_error(Some("Internal Server Error".to_string())))
            }
        },
        JsonRpcRequest::RequestContractIpo { jsonrpc: _, id: _, params: _ } => {
            Json(rpc_method.init_error(Some("Not yet implemented!".to_string())))
        },
        JsonRpcRequest::RequestTickData { jsonrpc: _, id: _, params: _ } => {
            Json(rpc_method.init_error(Some("Not yet implemented!".to_string())))
        }
    }
}

#[tokio::test]
async fn test() {
    use qubic_types::QubicId;

    let client = reqwest::Client::new();

    let req = JsonRpcRequest::RequestCurrentTickInfo { jsonrpc: "2.0".to_string(), id: 0 };

    let res: JsonRpcResponse = client.post("http://127.0.0.1:2003").json(&req).send().await.unwrap().json().await.unwrap();

    if let JsonRpcResponse::RequestCurrentTickInfo { jsonrpc: _, id: _, result, error: _ } = res {
        let result = result.unwrap();
        let wallet = QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();

        let raw_tx = RawTransaction {
            from: wallet.public_key,
            to: QubicId::default(),
            amount: 0,
            tick: result.tick + 20,
            input_size: 0,
            input_type: 0
        };

        let sig = wallet.sign(raw_tx);

        let req = JsonRpcRequest::SendTransaction { jsonrpc: "2.0".to_string(), id: 0, params: Transaction { raw_transaction: raw_tx, signature: sig } };

        let res: JsonRpcResponse = client.post("http://127.0.0.1:2003").json(&req).send().await.unwrap().json().await.unwrap();

        dbg!(res);
    }

    let id = QubicId::from_str("XOHYYIZLBNOAWDRWRMSGFTOBSEPATZLQYNTRBPHFXDAIOYQTGTNFTDABLLFA").unwrap();
    let req = JsonRpcRequest::RequestEntity { jsonrpc: "2.0".to_string(), id: 0, params: id };

    let res: JsonRpcResponse = client.post("http://127.0.0.1:2003").json(&req).send().await.unwrap().json().await.unwrap();

    dbg!(res);

    

    /*let req = JsonRpcRequest::RequestComputors { jsonrpc: "2.0".to_string(), id: 0 };

    let res: JsonRpcResponse = client.post("http://127.0.0.1:2003").json(&req).send().await.unwrap().json().await.unwrap();

    dbg!(res);*/
}