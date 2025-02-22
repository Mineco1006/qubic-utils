use rand::Rng;
use std::fs;

use tokio::time::{sleep, Duration, Instant};

extern crate qubic_rpc;
use qubic_rpc::{
    archiver::WalletEntry,
    qubic_rpc_types::{RichEntity, RichListResponse},
    spawn_server,
};

const COMPUTOR_ADDRESS: &str = "66.23.193.243:21841";

const DB_DIR: &str = "test-archiver-db";

#[tokio::test]
pub async fn rich_list() {
    let port = rand::rng().random_range(2003..2999);

    let (db, mut archiver_handle, server_handle) =
        spawn_server(port, COMPUTOR_ADDRESS.to_string(), DB_DIR.to_string()).await;

    let wallet_tree = db.open_tree("wallets").unwrap();

    // wait until database has data
    let timeout_secs = 5; // timeout is 5s
    let start = Instant::now();
    while wallet_tree.iter().next().is_none() {
        if start.elapsed().as_secs() >= timeout_secs {
            panic!("Timed out waiting for database to have data!");
        }
        sleep(Duration::from_millis(100)).await; // Small delay before checking again
    }

    // wait 100ms for server to cache wallets
    sleep(Duration::from_millis(100)).await;

    // abort archiver workers to get a deterministic response later
    archiver_handle.abort_all();

    // Check if the server has the transaction cached
    let response: RichListResponse =
        reqwest::get(format!("http://127.0.0.1:{port}/rich-list?page_size=10"))
            .await
            .expect("Failed to fetch rich list")
            .json()
            .await
            .expect("Failed to deserialize rich list");

    assert!(response.epoch > 0);
    assert!(response.rich_list.entities.len() > 0);

    let wallets: Vec<WalletEntry> = wallet_tree
        .iter()
        .rev()
        .filter_map(|res| res.ok().map(|(_, v)| v)) // Extract values
        .take(10)
        .filter_map(|bytes| bincode::deserialize(&bytes).ok()) // Deserialize
        .collect();

    let mut top_10_richest = Vec::<RichEntity>::new();
    for wallet in wallets {
        top_10_richest.push(RichEntity {
            identity: wallet.identity,
            balance: wallet.balance,
        });
    }

    assert_eq!(response.rich_list.entities, top_10_richest);

    server_handle.abort();

    // cleanup db file
    let _ = fs::remove_dir_all(DB_DIR).expect("Could not remove dir");
}
