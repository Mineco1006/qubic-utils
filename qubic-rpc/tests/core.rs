use base64::Engine;
use qubic_rpc::{qubic_rpc_types::LatestTick, routes::WalletBalance};
use std::collections::HashMap;

use qubic_rs::{
    qubic_tcp_types::types::transactions::TransactionWithData,
    qubic_types::{
        traits::{Sign, ToBytes},
        QubicWallet,
    },
};
use reqwest::StatusCode;

mod common;

const ORACLE_RPC: &str = "https://rpc.qubic.org/v1";

async fn check_oracle<T: std::fmt::Debug + for<'de> serde::de::Deserialize<'de>>(
    actual_url: &str,
    oracle_url: &str,
) -> (T, T) {
    let expected = reqwest::get(oracle_url)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let actual = reqwest::get(actual_url)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    dbg!(&actual);
    dbg!(&expected);

    (expected, actual)
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

    let (actual, expected): (LatestTick, LatestTick) = check_oracle(&actual_url, &oracle_url).await;

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

    // Shut down the server
    server_handle.abort();
}

#[tokio::test]
async fn wallet_balance() {
    let (port, server_handle) = common::setup().await;

    let wallet = "MGPAJNYEIENVTAQXEBARMUADANKBOOWIETOVESQIDCFFVZOVHLFBYIKDWITM";
    let oracle_url = format!("{ORACLE_RPC}/balances/{wallet}");
    let actual_url = format!("http://127.0.0.1:{port}/balances/{wallet}");

    let (mut expected, actual): (WalletBalance, WalletBalance) =
        check_oracle(&actual_url, &oracle_url).await;

    // sometimes ticks will misalign (see latest_tick test)
    if actual.balance.valid_for_tick >= expected.balance.valid_for_tick - 1 {
        expected.balance.valid_for_tick = actual.balance.valid_for_tick;
    }
    assert_eq!(expected, actual);

    server_handle.abort();
}
