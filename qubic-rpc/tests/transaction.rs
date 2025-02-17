use std::fs;

use tokio::time::{sleep, Duration};

extern crate qubic_rpc;
use qubic_rpc::{
    qubic_rpc_types::{LatestTick, TransactionResponseData, TransactionsResponse},
    spawn_server,
};

const COMPUTOR_ADDRESS: &str = "66.23.193.243:21841";

const DB_FILE: &str = "test-archiver-db";

#[tokio::test]
pub async fn transaction() {
    let port = 2003;

    let (mut archiver_handle, server_handle) =
        spawn_server(port, COMPUTOR_ADDRESS.to_string(), DB_FILE.to_string()).await;

    // wait 100ms for server to start
    sleep(Duration::from_millis(100)).await;

    // choose one tx from last tick's transactions
    let latest_tick: LatestTick = reqwest::get(format!("http://127.0.0.1:{port}/latestTick"))
        .await
        .expect("Failed to fetch latest tick")
        .json()
        .await
        .expect("Failed to deserialize latest tick");

    let latest_tick = latest_tick.latest_tick;
    let transactions: TransactionsResponse = reqwest::get(format!(
        "http://127.0.0.1:{port}/ticks/{latest_tick}/transactions"
    ))
    .await
    .expect("Failed to fetch approved transactions")
    .json()
    .await
    .expect("Failed to deserialize transactions");

    let tx: TransactionResponseData = transactions.transactions[0].clone(); // Choose one transaction from the list
    let tx_id = tx.tx_id.clone();

    // wait 100ms for server to cache tx
    sleep(Duration::from_millis(100)).await;

    // Check if the server has the transaction cached
    let response: TransactionResponseData =
        reqwest::get(format!("http://127.0.0.1:{port}/transactions/{tx_id}"))
            .await
            .expect("Failed to fetch transaction")
            .json()
            .await
            .expect("Failed to deserialize transaction");

    assert_eq!(response, tx);

    server_handle.abort();
    archiver_handle.abort_all();

    // cleanup db file
    let _ = fs::remove_file(DB_FILE);
}
