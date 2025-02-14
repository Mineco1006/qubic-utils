use rand::Rng;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc, Once, OnceLock,
};
use tokio::{net::TcpListener, runtime::Runtime};

extern crate qubic_rpc;
use qubic_rpc::{qubic_rpc_router_v2, RPCState};

pub const COMPUTOR_ADDRESS: &str = "45.152.160.28:21841";

static INIT: Once = Once::new();
static SERVER_PORT: OnceLock<AtomicU32> = OnceLock::new();
static GLOBAL_RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_global_runtime() -> &'static Runtime {
    GLOBAL_RUNTIME.get_or_init(|| Runtime::new().expect("Failed to create global runtime"))
}

fn get_server_port() -> &'static AtomicU32 {
    // Generate a random port number between 2003 and 2999 (inclusive)
    SERVER_PORT.get_or_init(|| AtomicU32::new(rand::rng().random_range(2003..=2999)))
}

pub async fn ensure_server_started() -> String {
    let rt = get_global_runtime();
    let port = get_server_port().load(Ordering::Relaxed);

    // Start server in the background (only does it once per callback)
    INIT.call_once(|| {
        rt.spawn(server(port.to_string()));
        println!("server spawned on port {port}");
    });

    port.to_string()
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
