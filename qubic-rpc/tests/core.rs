use qubic_rs::{
    qubic_tcp_types::types::transactions::{RawTransaction, Transaction, TransactionBuilder},
    qubic_types::{QubicId, QubicWallet, Signature},
};
use std::str::FromStr;

mod common;

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
    assert!(false);
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

#[tokio::test]
async fn approved_transactions_for_tick() {
    assert!(false);
}
