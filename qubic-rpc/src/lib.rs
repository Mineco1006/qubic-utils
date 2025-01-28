//! qubic-rpc is an RPC server for Qubic built on top of qubic-rs
//!
//! # Methods
//!
//! - This method
//! ```rust,no_run
//! ```

use axum::{extract::State, Json};
use clap::Parser;
use qubic_rs::{
    client::Client, qubic_tcp_types::types::transactions::TransactionFlags, transport::Tcp,
};
use std::sync::Arc;

use crate::qubic_rpc_types::{
    QubicJsonRpcRequest, QubicJsonRpcResponse, RequestError, RequestMethods, RequestResults,
    ResponseType,
};

pub mod qubic_rpc_types;

#[macro_use]
extern crate log;

#[macro_export]
macro_rules! result_or_501 {
    ($handle: expr, $rpc_method: expr) => {
        match $handle {
            Ok(res) => res,
            Err(_) => {
                return Json(QubicJsonRpcResponse {
                    jsonrpc: "2.0".to_owned(),
                    id: $rpc_method.id,
                    response: ResponseType::Error(RequestError {
                        method: $rpc_method.request.get_method(),
                        error: "InternalServerError".to_owned(),
                    }),
                })
            }
        }
    };
}

#[macro_export]
macro_rules! early_return_result {
    ($res_type: expr, $rpc_method: expr) => {
        return Json(QubicJsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            id: $rpc_method.id,
            response: ResponseType::Result($res_type),
        })
    };
}

#[derive(Debug, Parser)]
pub struct Args {
    /// Binds server to provided port
    #[arg(short, long, default_value = "2003")]
    pub port: String,

    /// Computor to send requests
    #[arg(short, long, default_value = "95.156.230.174:21841")]
    pub computor: String,
}

pub async fn request_handler(
    State(state): State<Arc<Args>>,
    Json(rpc_method): Json<QubicJsonRpcRequest>,
) -> Json<QubicJsonRpcResponse> {
    info!("Incoming request: {rpc_method:?}");

    if rpc_method.jsonrpc.as_str() != "2.0" {
        return Json(QubicJsonRpcResponse {
            jsonrpc: "2.0".to_owned(),
            id: rpc_method.id,
            response: ResponseType::Error(RequestError {
                method: rpc_method.request.get_method(),
                error: "Invalid JSON-RPC version found".to_owned(),
            }),
        });
    }

    let client = Client::<Tcp>::new(&state.computor).await.unwrap();

    match rpc_method.request {
        RequestMethods::RequestComputors => {
            let res = result_or_501!(client.qu().request_computors().await, rpc_method);

            early_return_result!(RequestResults::RequestComputors(res.into()), rpc_method);
        }
        RequestMethods::RequestCurrentTickInfo => {
            let res = result_or_501!(client.qu().get_current_tick_info().await, rpc_method);

            early_return_result!(RequestResults::RequestCurrentTickInfo(res), rpc_method);
        }
        RequestMethods::RequestEntity(id) => {
            let res = result_or_501!(client.qu().request_entity(id).await, rpc_method);

            early_return_result!(RequestResults::RequestEntity(res.entity), rpc_method);
        }
        RequestMethods::SendTransaction(tx) => {
            result_or_501!(client.qu().send_signed_transaction(tx).await, rpc_method);

            early_return_result!(RequestResults::SendTransaction(tx.into()), rpc_method);
        }
        RequestMethods::RequestTickTransactions(tick) => {
            let res = result_or_501!(
                client
                    .qu()
                    .request_tick_transactions(tick, TransactionFlags::all())
                    .await,
                rpc_method
            );

            early_return_result!(RequestResults::RequestTickTransactions(res), rpc_method);
        }
    }
}