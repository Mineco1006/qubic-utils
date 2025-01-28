use axum::{http::Method, routing::post, Router};
use rand::Rng;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    task::{self, JoinHandle},
};
use tower_http::cors::{Any, CorsLayer};

extern crate qubic_rpc;

pub async fn setup() -> (String, JoinHandle<()>) {
    // Generate a random port number between 2003 and 2999 (inclusive)
    // to avoid tests spawning on the same port
    let port = rand::rng().random_range(2003..=2999).to_string();

    // start server in the background
    let server_handle = task::spawn(server(port.clone()));

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    (port, server_handle)
}

pub async fn server(port: String) {
    let state = Arc::new(qubic_rpc::RPCState {
        port,
        computor: qubic_rpc::COMPUTOR_ADDRESS.to_string(),
    });

    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", post(qubic_rpc::request_handler))
        .with_state(state.clone())
        .layer(cors);

    let tcp_listener = TcpListener::bind(&format!("0.0.0.0:{}", state.port))
        .await
        .unwrap();
    axum::serve(tcp_listener, app.into_make_service())
        .await
        .unwrap();
}
