use qubic_rpc::qubic_rpc_types::{
    QubicJsonRpcRequest, QubicJsonRpcResponse, RequestMethods, RequestResults, ResponseType,
};
use qubic_rs::{
    qubic_tcp_types::types::transactions::{RawTransaction, Transaction, TransactionBuilder},
    qubic_types::{QubicId, QubicWallet, Signature},
};
use std::str::FromStr;

mod common;

#[tokio::test]
async fn get_tick_info() {
    let (port, server_handle) = common::setup().await;

    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestCurrentTickInfo);
    let res: QubicJsonRpcResponse = client
        .post(format!("http://127.0.0.1:{port}"))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(&res);
    // Shut down the server
    server_handle.abort();
}

#[tokio::test]
async fn get_tick_transactions() {
    let (port, server_handle) = common::setup().await;

    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestCurrentTickInfo);
    let res: QubicJsonRpcResponse = client
        .post(format!("http://127.0.0.1:{port}"))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(&res);

    if let ResponseType::Result(r) = res.response {
        if let RequestResults::RequestCurrentTickInfo(current_tick) = r {
            let request = QubicJsonRpcRequest::new(
                0,
                RequestMethods::RequestTickTransactions(current_tick.tick - 10),
            );
            let res: QubicJsonRpcResponse = client
                .post(format!("http://127.0.0.1:{port}"))
                .json(&request)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            dbg!(&res);
        }
    }
    // Shut down the server
    server_handle.abort();
}

#[tokio::test]
async fn get_entity_info() {
    let (port, server_handle) = common::setup().await;

    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(
        0,
        RequestMethods::RequestEntity(
            QubicId::from_str("XOHYYIZLBNOAWDRWRMSGFTOBSEPATZLQYNTRBPHFXDAIOYQTGTNFTDABLLFA")
                .unwrap(),
        ),
    );
    let res: QubicJsonRpcResponse = client
        .post(format!("http://127.0.0.1:{port}"))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(res);

    // Shut down the server
    server_handle.abort();
}

#[tokio::test]
async fn get_computors() {
    let (port, server_handle) = common::setup().await;

    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestComputors);
    let res: QubicJsonRpcResponse = client
        .post(format!("http://127.0.0.1:{port}"))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(res);

    // Shut down the server
    server_handle.abort();
}

#[tokio::test]
async fn send_transaction() {
    let (port, server_handle) = common::setup().await;
    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestCurrentTickInfo);
    let res: QubicJsonRpcResponse = client
        .post(format!("http://127.0.0.1:{port}"))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let wallet = QubicWallet::from_seed(seed).unwrap();
    let tx = match res.response {
        ResponseType::Result(RequestResults::RequestCurrentTickInfo(current_tick)) => Transaction {
            raw_transaction: RawTransaction {
                from: wallet.public_key,
                to: wallet.public_key,
                amount: 0,
                tick: current_tick.tick + 30,
                ..Default::default()
            },
            signature: Signature::default(),
        },
        _ => panic!("could not setup transaction"),
    };

    let send_tx_request = QubicJsonRpcRequest::new(0, RequestMethods::SendTransaction(tx));
    let res: QubicJsonRpcResponse = client
        .post(format!("http://127.0.0.1:{port}"))
        .json(&send_tx_request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(res);

    // Shut down the server
    server_handle.abort();
}

#[tokio::test]
async fn get_tick_data() {
    let (port, server_handle) = common::setup().await;

    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestCurrentTickInfo);
    let res: QubicJsonRpcResponse = client
        .post(format!("http://127.0.0.1:{port}"))
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(&res);

    if let ResponseType::Result(r) = res.response {
        if let RequestResults::RequestCurrentTickInfo(current_tick) = r {
            let request = QubicJsonRpcRequest::new(
                0,
                RequestMethods::RequestTickTransactions(current_tick.tick - 10),
            );
            let res: QubicJsonRpcResponse = client
                .post(format!("http://127.0.0.1:{port}"))
                .json(&request)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            dbg!(&res);
        }
    }
    // Shut down the server
    server_handle.abort();
}
