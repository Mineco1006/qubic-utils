use base64::Engine;
use qubic_rpc::qubic_rpc_types::{
    APIStatus, LatestTick, RPCStatus, TransactionResponse, WalletBalance,
};
use std::collections::HashMap;

use qubic_rs::{
    qubic_tcp_types::types::{
        contracts::ResponseContractFunction, ticks::CurrentTickInfo,
        transactions::TransactionWithData,
    },
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
    let actual = reqwest::get(actual_url)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(&actual);

    let expected = reqwest::get(oracle_url)
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(&expected);

    (expected, actual)
}

#[tokio::test]
async fn get_index() {
    let rt = common::get_global_runtime();
    let port = common::ensure_server_started().await;

    let body = reqwest::get(format!("http://127.0.0.1:{port}"))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    dbg!(&body);
}

#[tokio::test]
async fn latest_tick() {
    let port = common::ensure_server_started().await;

    let oracle_url = format!("{ORACLE_RPC}/latestTick");
    let actual_url = format!("http://127.0.0.1:{port}/latestTick");

    let (actual, expected): (LatestTick, LatestTick) = check_oracle(&actual_url, &oracle_url).await;

    // our latestTick can be a bit behind sometimes, account for that
    assert!(actual.latest_tick >= expected.latest_tick - 10);
}

#[tokio::test]
async fn broadcast_transaction() {
    let port = common::ensure_server_started().await;

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
}

#[tokio::test]
async fn wallet_balance() {
    let port = common::ensure_server_started().await;

    let wallet = "MGPAJNYEIENVTAQXEBARMUADANKBOOWIETOVESQIDCFFVZOVHLFBYIKDWITM";
    let oracle_url = format!("{ORACLE_RPC}/balances/{wallet}");
    let actual_url = format!("http://127.0.0.1:{port}/balances/{wallet}");

    let (mut expected, actual): (WalletBalance, WalletBalance) =
        check_oracle(&actual_url, &oracle_url).await;

    // sometimes ticks will misalign (see latest_tick test)
    if actual.balance.valid_for_tick >= expected.balance.valid_for_tick - 10 {
        expected.balance.valid_for_tick = actual.balance.valid_for_tick;
    }
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn health_check() {
    let port = common::ensure_server_started().await;

    let resp = reqwest::get(format!("http://127.0.0.1:{port}/healthcheck"))
        .await
        .unwrap();

    let http_status = resp.status();
    let rpc_status: RPCStatus = resp.json().await.unwrap();

    let api_status = rpc_status.status;

    assert_eq!(http_status, StatusCode::OK);
    assert_eq!(api_status, APIStatus::Ok);
}

#[tokio::test]
async fn transaction() {
    let port = common::ensure_server_started().await;

    let tx_id = "rlinciclnsqteajcanbecoedphdftskhikawqvedkfzbmiclqqnpgoagsbpb";
    let oracle_url = format!("{ORACLE_RPC}/transactions/{tx_id}");
    let actual_url = format!("http://127.0.0.1:{port}/transactions/{tx_id}");

    let (expected, actual): (TransactionResponse, TransactionResponse) =
        check_oracle(&actual_url, &oracle_url).await;

    assert_eq!(expected, actual);
}

#[tokio::test]
async fn transfer_transactions_per_tick() {
    let port = common::ensure_server_started().await;

    let wallet_id = "rlinciclnsqteajcanbecoedphdftskhikawqvedkfzbmiclqqnpgoagsbpb";
    let oracle_url = format!("{ORACLE_RPC}/identities/{wallet_id}/transfer-transactions");
    let actual_url =
        format!("http://127.0.0.1:{port}/identities/{wallet_id}/transfer-transactions");

    let (expected, actual): (TransactionResponse, TransactionResponse) =
        check_oracle(&actual_url, &oracle_url).await;

    assert_eq!(expected, actual);
}

#[tokio::test]
async fn query_sc() {
    let port = common::ensure_server_started().await;

    let contract_index = "1".to_string();
    let input_type = "1".to_string();
    let input_size = "0".to_string();
    let request_data = "".to_string();

    let mut payload = HashMap::new();
    payload.insert("contractIndex", contract_index);
    payload.insert("inputType", input_type);
    payload.insert("inputSize", input_size);
    payload.insert(
        "requestData",
        base64::engine::general_purpose::STANDARD.encode(request_data.as_bytes()),
    );

    let http_client = reqwest::Client::new();

    let resp = http_client
        .post(format!("http://127.0.0.1:{port}/querySmartContract"))
        .json(&payload)
        .send()
        .await
        .unwrap();

    let status = resp.status();
    let actual_sc_data: ResponseContractFunction = resp.json().await.unwrap(); // must succeed

    let expected_sc_data = ResponseContractFunction {
        output: base64::engine::general_purpose::STANDARD
            .decode("AMqaO2QAAADAxi0A")
            .unwrap(),
    };

    assert_eq!(status, StatusCode::OK);
    assert_eq!(actual_sc_data, expected_sc_data);
}

#[tokio::test]
async fn tick_info() {
    let port = common::ensure_server_started().await;

    let resp = reqwest::get(format!("http://127.0.0.1:{port}/tick-info"))
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let _resp_tick_info: CurrentTickInfo = resp.json().await.unwrap(); // must succeed
}
