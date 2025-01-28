use qubic_rpc::qubic_rpc_types::{
    QubicJsonRpcRequest, QubicJsonRpcResponse, RequestError, RequestMethods, RequestResults,
    ResponseType,
};

mod common;

#[tokio::test]
async fn tick_info() {
    // start server in the background
    let server_handle = task::spawn(common::server);

    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestCurrentTickInfo);
    let res: QubicJsonRpcResponse = client
        .post(RPC)
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    dbg!(&res);
}

#[tokio::test]
async fn tick_transactions() {
    // start server in the background
    let server_handle = task::spawn(common::server);

    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(0, RequestMethods::RequestCurrentTickInfo);
    let res: QubicJsonRpcResponse = client
        .post(common::RPC)
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
                .post(common::RPC)
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
}
