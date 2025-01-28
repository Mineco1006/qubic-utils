use axum::http::Method;
use axum::{extract::State, routing::post, Json, Router};
use clap::Parser;

use qubic_rpc::{
    early_return_result,
    qubic_rpc_types::{
        QubicJsonRpcRequest, QubicJsonRpcResponse, RequestError, RequestMethods, RequestResults,
        ResponseType,
    },
    request_handler, result_or_501, Args,
};

use qubic_rs::{client::Client, transport::Tcp};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Args::parse();

    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let state = Arc::new(args);

    let app = Router::new()
        .route("/", post(request_handler))
        .with_state(state.clone())
        .layer(cors);

    info!("Binding server to port {}", state.port);
    let tcp_listener = TcpListener::bind(&format!("0.0.0.0:{}", state.port))
        .await
        .unwrap();
    axum::serve(tcp_listener, app.into_make_service())
        .await
        .unwrap();
}
