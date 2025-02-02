use axum::{http::Method, routing::post, Router};
use rand::Rng;
use std::sync::Arc;
use tokio::{
    net::TcpListener,
    task::{self, JoinHandle},
};
use tower_http::cors::{Any, CorsLayer};

extern crate qubic_rpc;
use qubic_rpc::{qubic_rpc_router_v2, RPCState};

const COMPUTOR_ADDRESS: &str = "45.152.160.28";

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
    let state = Arc::new(RPCState::new(COMPUTOR_ADDRESS.to_string()));

    let routes = qubic_rpc_router_v2(state.clone());

    let tcp_listener = TcpListener::bind(&format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    axum::serve(tcp_listener, routes.with_state(state))
        .await
        .unwrap();
}
