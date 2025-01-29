use clap::Parser;
use std::sync::Arc;
use tokio::net::TcpListener;

use qubic_rpc::qubic_rpc_router_v2;

#[macro_use]
extern crate log;

#[derive(Debug, Parser)]
pub struct Args {
    /// Binds server to provided port
    #[arg(short, long, default_value = "2003")]
    pub port: String,

    /// Computor IP to which send requests
    #[arg(short, long)]
    pub computor: String,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Arc::new(Args::parse());

    let app = qubic_rpc_router_v2(format!("{}:21841", args.computor));

    info!("Binding server to port {}", args.port);
    let tcp_listener = TcpListener::bind(&format!("0.0.0.0:{}", args.port))
        .await
        .unwrap();

    axum::serve(tcp_listener, app).await.unwrap();
}
