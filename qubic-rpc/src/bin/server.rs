///! Qubic RPC Server
use std::sync::Arc;

use clap::Parser;

use qubic_rpc::server::spawn_server;

#[derive(Debug, Parser)]
pub struct Args {
    /// Binds server to provided port
    #[arg(short, long, default_value = "2003")]
    pub port: u32,

    /// Computor IP to which send requests
    #[arg(short, long)]
    pub computor: String,

    /// Archiver database directory
    #[arg(short, long, default_value = "archiver-db")]
    pub db_dir: String,
}
#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Arc::new(Args::parse());

    let computor_address = format!("{}:21841", args.computor);
    let (_db, archiver_handle, server_handle) =
        spawn_server(args.port, computor_address, args.db_dir.clone()).await;

    let _ = tokio::join!(server_handle);
    let _ = archiver_handle.join_all();
}
