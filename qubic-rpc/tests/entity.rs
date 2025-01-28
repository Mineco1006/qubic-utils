use qubic_rpc::qubic_rpc_types::{
    QubicJsonRpcRequest, QubicJsonRpcResponse, RequestError, RequestMethods, RequestResults,
    ResponseType,
};
use qubic_rs::qubic_types::QubicId;
use std::str::FromStr;

mod common;

#[tokio::test]
async fn entity() {
    // start server in the background
    let server_handle = task::spawn(common::server);

    let client = reqwest::Client::new();
    let request = QubicJsonRpcRequest::new(
        0,
        RequestMethods::RequestEntity(
            QubicId::from_str("XOHYYIZLBNOAWDRWRMSGFTOBSEPATZLQYNTRBPHFXDAIOYQTGTNFTDABLLFA")
                .unwrap(),
        ),
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
    dbg!(res);
}
