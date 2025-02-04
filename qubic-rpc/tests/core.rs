use base64::Engine;
use qubic_rpc::qubic_rpc_types::LatestTick;
use std::collections::HashMap;

use qubic_rs::{
    qubic_tcp_types::types::transactions::TransactionWithData,
    qubic_types::{
        traits::{Sign, ToBytes},
        QubicTxHash, QubicWallet,
    },
};
use reqwest::StatusCode;

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
    let expected: LatestTick = reqwest::get(oracle_url)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let actual_url = format!("http://127.0.0.1:{port}/latestTick");
    let actual: LatestTick = reqwest::get(actual_url)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    // our latestTick can be expected - 1 sometimes, account for that
    assert!(actual.latest_tick >= expected.latest_tick - 1);

    // Shut down the server
    server_handle.abort();
}
#[tokio::test]
async fn broadcast_transaction() {
    let (port, server_handle) = common::setup().await;

    let wallet =
        QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    let mut tx = TransactionWithData::default();
    let _ = tx.sign(&wallet);

    let mut payload = HashMap::new();
    payload.insert(
        "encodedTransaction",
        base64::engine::general_purpose::STANDARD.encode(tx.to_bytes()),
    );

    let http_client = reqwest::Client::new();

    let resp = http_client
        .post(format!("http://127.0.0.1:{port}/broadcast-transaction"))
        .json(&payload)
        .send()
        .await
        .unwrap();

    let status = resp.status();
    dbg!(&resp.text().await.unwrap());
    assert_eq!(status, StatusCode::OK);
    //TODO: test for ok response

    // Shut down the server
    server_handle.abort();
}
#[tokio::test]
async fn wallet_balance() {
    assert!(false);
}
