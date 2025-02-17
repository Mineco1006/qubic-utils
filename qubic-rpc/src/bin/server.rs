use std::sync::Arc;

use clap::Parser;

use qubic_rpc::spawn_server;

#[derive(Debug, Parser)]
pub struct Args {
    /// Binds server to provided port
    #[arg(short, long, default_value = "2003")]
    pub port: u32,

    /// Computor IP to which send requests
    #[arg(short, long)]
    pub computor: String,

    /// Archiver database file
    #[arg(short, long, default_value = "archiver-db")]
    pub db_file: String,
}
#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let args = Arc::new(Args::parse());

    let computor_address = format!("{}:21841", args.computor);
    let (archiver_handle, server_handle) =
        spawn_server(args.port, computor_address, args.db_file.clone()).await;

    let _ = tokio::join!(server_handle);
    let _ = archiver_handle.join_all();
}
