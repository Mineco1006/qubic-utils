use axum::Json;

extern crate qubic_rpc;

pub const RPC: &str = "http://127.0.0.1:2003/";

#[derive(Debug, Parser)]
struct State {
    /// Port to listen to
    port: String,

    /// String such as "95.156.230.174:21841"
    computor: String,
}

pub async fn server() {
    let state = Arc::new(State {
        port: 2003,
        computor: "188.241.26.108:21841".to_string(),
    });

    let app = Router::new()
        .route("/", post(qubic_rpc::request_handler))
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
