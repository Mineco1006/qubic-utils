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
