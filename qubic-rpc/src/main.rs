use std::sync::Arc;
use std::env;
use axum::{
    routing::post,
    extract::State,
    Router, Json,
};
use qubic_web3_rs::{client::Client, transport::Tcp, qubic_tcp_types::types::transactions::TransactionFlags};
use qubic_rpc_types::{QubicJsonRpcRequest, QubicJsonRpcResponse, ResponseType, RequestError, RequestMethods, RequestResults};
use axum::http::Method;
use tokio::net::TcpListener;
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
    #[arg(short, long, default_value = "95.156.230.174:21841")]
    computor: String
}

#[tokio::main]
async fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::Info).init();

    let args = Args::parse();

    let port = env::var("PORT").unwrap_or(args.port);
    let computor = env::var("COMPUTOR").unwrap_or(args.computor);

    let cors = CorsLayer::new()
                        .allow_methods([Method::POST])
                        .allow_origin(Any)
                        .allow_headers(Any);

    let state = Arc::new(Args {
        port: port.clone(),
        computor: computor.clone(),
    });

    let app = Router::new().route("/", post(request_handler)).with_state(state.clone()).layer(cors);

    info!("Binding server to port {}", state.port);
    let tcp_listener = TcpListener::bind(&format!("0.0.0.0:{}", state.port)).await.unwrap();
    axum::serve(tcp_listener, app.into_make_service()).await.unwrap();
}

macro_rules! result_or_501 {
    ($handle: expr, $rpc_method: expr) => {
        match $handle {
            Ok(res) => res,
            Err(_) => {
                return Json(QubicJsonRpcResponse {
                    jsonrpc: "2.0".to_owned(),
                    id: $rpc_method.id,
                    response: ResponseType::Error(RequestError { method: $rpc_method.request.get_method(), error: "InternalServerError".to_owned() })
                })
            }
        }
    };
}

macro_rules! early_return_result {
    ($res_type: expr, $rpc_method: expr) => {
        return Json(QubicJsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            id: $rpc_method.id,
            response: ResponseType::Result($res_type)
        })
    };
}

async fn request_handler(State(state): State<Arc<Args>>, Json(rpc_method): Json<QubicJsonRpcRequest>) -> Json<QubicJsonRpcResponse> {
    info!("Incoming request: {rpc_method:?}");

    if rpc_method.jsonrpc.as_str() != "2.0" {
        return Json(QubicJsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            id: rpc_method.id,
            response: ResponseType::Error(RequestError { method: rpc_method.request.get_method(), error: "Invalid JSON-RPC version found".to_owned() })
        })
    }

    let client = Client::<Tcp>::new(&state.computor).await.unwrap();

    match rpc_method.request {
        RequestMethods::RequestComputors => {
            let res = result_or_501!(client.qu().request_computors().await, rpc_method);

            early_return_result!(RequestResults::RequestComputors(res.into()), rpc_method);
        },
        RequestMethods::RequestCurrentTickInfo => {
            let res = result_or_501!(client.qu().get_current_tick_info().await, rpc_method);

            early_return_result!(RequestResults::RequestCurrentTickInfo(res), rpc_method);
        },
        RequestMethods::RequestEntity(id) => {
            let res = result_or_501!(client.qu().request_entity(id).await, rpc_method);

            early_return_result!(RequestResults::RequestEntity(res.entity), rpc_method);
        },
        RequestMethods::SendTransaction(tx) => {
            result_or_501!(client.qu().send_signed_transaction(tx).await, rpc_method);

            early_return_result!(RequestResults::SendTransaction(tx.into()), rpc_method);
        },
        RequestMethods::RequestTickTransactions(tick) => {
            let res = result_or_501!(client.qu().request_tick_transactions(tick, TransactionFlags::all()).await, rpc_method);

            early_return_result!(RequestResults::RequestTickTransactions(res), rpc_method);
        }
    }
}

#[tokio::test]
async fn test() {
    use std::str::FromStr;
    use qubic_types::QubicId;
    const RPC: &str = "http://127.0.0.1:2003/";

    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestEntity(QubicId::from_str("XOHYYIZLBNOAWDRWRMSGFTOBSEPATZLQYNTRBPHFXDAIOYQTGTNFTDABLLFA").unwrap()));
    let res: QubicJsonRpcResponse = client.post(RPC).json(&request).send().await.unwrap().json().await.unwrap();
    dbg!(res);

    let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestCurrentTickInfo);
    let res: QubicJsonRpcResponse = client.post(RPC).json(&request).send().await.unwrap().json().await.unwrap();
    dbg!(&res);

    if let ResponseType::Result(r) = res.response {
        if let RequestResults::RequestCurrentTickInfo(current_tick) = r {
            let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestTickTransactions(current_tick.tick - 10));
            let res: QubicJsonRpcResponse = client.post(RPC).json(&request).send().await.unwrap().json().await.unwrap();
            dbg!(&res);
        }
    }
}