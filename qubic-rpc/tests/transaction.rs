use rand::{seq::IteratorRandom, Rng};
use std::{fs, sync::Arc};

use sled::Db;
use tokio::time::{sleep, Duration, Instant};

extern crate qubic_rpc;
use qubic_rpc::{
    qubic_rpc_types::{TransactionResponse, TransactionResponseData},
    spawn_server,
};
use qubic_rs::qubic_tcp_types::types::transactions::TransactionWithData;

const COMPUTOR_ADDRESS: &str = "66.23.193.243:21841";

const DB_DIR: &str = "test-archiver-db";

async fn random_tx(db: Arc<Db>) -> TransactionResponseData {
    // wait until database has data
    let timeout_secs = 5; // timeout is 5s
    let start = Instant::now();
    let tx_tree = db.open_tree("transactions").unwrap();
    while tx_tree.iter().keys().next().is_none() {
        if start.elapsed().as_secs() >= timeout_secs {
            panic!("Timed out waiting for database to have data!");
        }
        sleep(Duration::from_millis(100)).await; // Small delay before checking again
    }

    let mut rng = rand::rng();
    let key = tx_tree
        .iter()
        .keys()
        .choose(&mut rng)
        .expect("Database is empty")
        .expect("Could not get data from database");
    let value = tx_tree
        .get(&key)
        .expect("Key not found in database")
        .expect("Could not get data from database");
    let transaction: TransactionWithData = bincode::deserialize(&value).unwrap();
    transaction.into()
}

#[tokio::test]
pub async fn transaction() {
    let port = rand::rng().random_range(2003..2999);

    let (db, mut archiver_handle, server_handle) =
        spawn_server(port, COMPUTOR_ADDRESS.to_string(), DB_DIR.to_string()).await;

    let expected_tx = random_tx(db).await;
    let tx_id = expected_tx.tx_id.clone();

    // wait 100ms for server to cache tx
    sleep(Duration::from_millis(100)).await;

    // Check if the server has the transaction cached
    let response: TransactionResponse =
        reqwest::get(format!("http://127.0.0.1:{port}/transactions/{tx_id}"))
            .await
            .expect("Failed to fetch transaction")
            .json()
            .await
            .expect("Failed to deserialize transaction");

    let actual_tx: TransactionResponseData = response.transaction;

    assert_eq!(actual_tx, expected_tx);

    server_handle.abort();
    archiver_handle.abort_all();

    // cleanup db file
    let _ = fs::remove_dir_all(DB_DIR).expect("Could not remove dir");
}
