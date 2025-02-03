use qubic_rs::{
    qubic_tcp_types::types::transactions::TransactionWithData,
    qubic_types::{QubicId, QubicWallet, Signature},
    transport::Tcp,
};
use serde_json::json;
use std::str::FromStr;

mod common;

const ORACLE_RPC: &str = "https://rpc.qubic.org/v1";

async fn check_oracle(actual_url: &str, oracle_url: &str) -> bool {
    let expected = reqwest::get(oracle_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let actual = reqwest::get(actual_url)
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    dbg!(&actual);
    dbg!(&expected);

    expected == actual
}

#[tokio::test]
async fn get_index() {
    let (port, server_handle) = common::setup().await;

    let body = reqwest::get(format!("http://127.0.0.1:{port}"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    dbg!(&body);

    // Shut down the server
    server_handle.abort();
}

#[tokio::test]
async fn latest_tick() {
    let (port, server_handle) = common::setup().await;

    let oracle_url = format!("{ORACLE_RPC}/latestTick");
    let actual_url = format!("http://127.0.0.1:{port}/latestTick");

    assert!(check_oracle(&actual_url, &oracle_url).await);

    // Shut down the server
    server_handle.abort();
}
#[tokio::test]
async fn broadcast_transaction() {
    let (port, server_handle) = common::setup().await;

    // create new wallet
    // create tx of 5 from wallet to wallet
    let seed = "";
    let destination_id = "";
    let amount = 5;

    let client = qubic_rs::client::Client::<Tcp>::new(common::COMPUTOR_ADDRESS).await?;
    let latest_tick: u32 = client.qu().get_current_tick_info().await?.tick;

    let wallet =
        QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let mut tx = TransactionWithData::default();
    tx.sign(&wallet);

    // let tx_encoded = base64.b64encode(tx).decode("utf-8");
    let payload = json!({
        "encodedTransaction": tx.encode() // base64 encoded tx
    });

    let http_client = reqwest::Client::new();

    let expected = http_client
        .post(format!("{ORACLE_RPC}/latestTick"))
        .body(payload)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let actual = http_client
        .post(format!("http://127.0.0.1:{port}/latestTick"))
        .body(payload)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    dbg!(&actual);
    dbg!(&expected);

    assert_eq!(expected, actual);

    // Shut down the server
    server_handle.abort();
}
#[tokio::test]
async fn approved_transactions_for_tick() {
    assert!(false);
}

#[tokio::test]
async fn tick_data() {
    assert!(false);
}

#[tokio::test]
async fn chain_hash() {
    assert!(false);
}

#[tokio::test]
async fn quorum_tick_data() {
    assert!(false);
}

#[tokio::test]
async fn store_hash() {
    assert!(false);
}
