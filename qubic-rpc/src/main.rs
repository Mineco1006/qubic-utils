use axum::{
    routing::post,
    Router, Json,
};
use qubic_web3_rs::{client::Client, transport::Tcp};
use qubic_rpc_types::{JsonRpcRequest, JsonRpcResponse, ComputorInfos};
use axum::http::Method;
use tower_http::cors::{CorsLayer, Any};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::Builder::new().filter_level(log::LevelFilter::Info).init();

    let cors = CorsLayer::new()
                        .allow_methods([Method::POST])
                        .allow_origin(Any)
                        .allow_headers(Any);

    let app = Router::new().route("/", post(request_handler)).layer(cors);

    info!("Binding server to port 2003");
    axum::Server::bind(&"0.0.0.0:2003".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

const COMPUTOR_IP: &str = "84.74.68.177:21841";

async fn request_handler(Json(rpc_method): Json<JsonRpcRequest>) -> Json<JsonRpcResponse> {
    info!("Incoming request: {rpc_method:?}");
    if !rpc_method.is_version_2() {
        let res = rpc_method.init_error(Some("Invalid JSON-RPC version!".to_string()));
        return Json(res);
    }

    let client = Client::<Tcp>::new(COMPUTOR_IP);

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

    dbg!(res);

    let id = QubicId::from_str("XOHYYIZLBNOAWDRWRMSGFTOBSEPATZLQYNTRBPHFXDAIOYQTGTNFTDABLLFA").unwrap();
    let req = JsonRpcRequest::RequestEntity { jsonrpc: "2.0".to_string(), id: 0, params: id };

    let res: JsonRpcResponse = client.post("http://127.0.0.1:2003").json(&req).send().await.unwrap().json().await.unwrap();

    dbg!(res);

    /*let req = JsonRpcRequest::RequestComputors { jsonrpc: "2.0".to_string(), id: 0 };

    let res: JsonRpcResponse = client.post("http://127.0.0.1:2003").json(&req).send().await.unwrap().json().await.unwrap();

    dbg!(res);*/
}