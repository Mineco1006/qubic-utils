use anyhow::anyhow;
use axum::{
    extract::{OriginalUri, Path, Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use base64::Engine;
use qubic_rs::{
    client::Client,
    qubic_tcp_types::types::transactions::{
        Transaction, TransactionData::TransferAsset, TransactionFlags, TransactionWithData,
    },
    qubic_types::{traits::FromBytes, QubicId},
    transport::Tcp,
};
use serde::{de, Deserialize, Deserializer};
use std::{fmt, str::FromStr, sync::Arc};

use crate::{
    qubic_rpc_types::{
        APIStatus, Balance, BroadcastTransactionPayload, LatestTick, QubicRpcError, RPCStatus,
        RequestSCPayload, TickTransactions, TransferResponse, WalletBalance,
    },
    RPCState,
};

pub async fn index() -> impl IntoResponse {
    Redirect::permanent("/healthcheck")
}
#[axum::debug_handler]
pub async fn latest_tick(
    State(state): State<Arc<RPCState>>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let client = Client::<Tcp>::new(&state.computor_address).await?;
    let latest_tick_resp: LatestTick = client.qu().get_current_tick_info().await?.into();
    Ok(Json(latest_tick_resp))
}
pub async fn broadcast_transaction(
    State(state): State<Arc<RPCState>>,
    Json(payload): Json<BroadcastTransactionPayload>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let client = Client::<Tcp>::new(&state.computor_address).await?;
    let tx = Transaction::from_bytes(
        &base64::engine::general_purpose::STANDARD.decode(payload.encoded_transaction)?,
    )?;
    let _ = client.qu().send_signed_transaction(tx).await?;
    Ok(Json("Broadcast successful"))
}
/// Returns the balance of a specific wallet from the API.
pub async fn wallet_balance(
    State(state): State<Arc<RPCState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let public_key = QubicId::from_str(&id)?;
    let client = Client::<Tcp>::new(&state.computor_address).await?;
    let entity_response = client.qu().request_entity(public_key).await?;
    let balance: Balance = entity_response.into();
    Ok(Json(WalletBalance { balance }))
}
pub async fn status(Path(_id): Path<QubicId>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}

/// Returns information for a given transaction
pub async fn transaction(State(_state): State<Arc<RPCState>>, Path(_id): Path<QubicId>) {}

/// Returns the same information as `/transactions/{tx_id}`
pub async fn transaction_status(Path(id): Path<String>) -> impl IntoResponse {
    Redirect::permanent(&format!("/transactions/{id}"))
}
pub async fn transfer_transactions_per_tick(
    OriginalUri(uri): OriginalUri,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let query_params = uri
        .path_and_query()
        .and_then(|pq| pq.query())
        .map(|q| format!("?{}", q))
        .unwrap_or_default();

    let redirect_url = format!("/identities/{id}/transfers{}", query_params);

    Redirect::permanent(&redirect_url)
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TransferQueryParams {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    start_tick: Option<u32>,
    end_tick: Option<u32>,
    sc_only: Option<bool>,
    desc: Option<bool>,
}
/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}
/// Returns information for a given transfer
pub async fn transfers(
    State(state): State<Arc<RPCState>>,
    Path(id): Path<QubicId>,
    Query(query_params): Query<TransferQueryParams>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let flags = TransactionFlags::all();
    let client = Client::<Tcp>::new(&state.computor_address).await?;

    let latest_tick = client.qu().get_current_tick_info().await?.tick;

    let start_tick = query_params.start_tick.unwrap_or(latest_tick);
    let end_tick = query_params.end_tick.unwrap_or(latest_tick);

    let mut transfer_transactions = Vec::<TransactionWithData>::new();
    if end_tick < start_tick {
        return Err(anyhow!("end_tick should be higher or equal to start_tick").into());
    }

    let max_tick_range = 100000000;
    if end_tick - start_tick > max_tick_range {
        return Err(anyhow!("tick range too big").into());
    }

    let mut resp = TransferResponse::new();

    for tick in start_tick..end_tick + 1 {
        let tick_transactions = client
            .qu()
            .request_tick_transactions(tick, flags)
            .await?
            .into_iter()
            .filter(|tx| {
                if let TransferAsset(asset_input) = tx.data {
                    asset_input.destination == id
                } else {
                    false
                }
            })
            .collect::<Vec<_>>();

        let tick_transactions = TickTransactions {
            tick_number: tick,
            identity: id.to_string(),
            transactions: tick_transactions.into_iter().map(Into::into).collect(),
        };
        resp.transfer_transactions_per_tick.push(tick_transactions);
    }
    Ok(Json(resp))
}
/// Returns general health information about RPC server
pub async fn health_check(
    State(state): State<Arc<RPCState>>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(RPCStatus {
        status: APIStatus::Ok,
        uptime: state.start_time.elapsed().as_secs(),
        version: "v2".to_string(),
    }))
}
pub async fn computors(
    State(_state): State<Arc<RPCState>>,
    Path(_epoch): Path<u32>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn query_sc(
    State(state): State<Arc<RPCState>>,
    Json(payload): Json<RequestSCPayload>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let client = Client::<Tcp>::new(&state.computor_address).await?;
    let resp = client
        .qu()
        .request_contract_function(
            payload.contract_index,
            payload.input_type,
            payload.input_size,
            base64::engine::general_purpose::STANDARD.decode(payload.request_data)?,
        )
        .await?;
    Ok(Json(resp))
}
pub async fn tick_info(
    State(state): State<Arc<RPCState>>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let client = Client::<Tcp>::new(&state.computor_address).await?;
    Ok(Json(client.qu().get_current_tick_info().await?))
}
pub async fn block_height() -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn latest_stats() -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn rich_list() -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}

/// Returns the approved transactions for a specific tick (block height)
pub async fn approved_transactions_for_tick(
    Path(_tick): Path<u32>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn tick_data(Path(_tick): Path<u32>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
/// Returns the chain hash (hexadecimal digest) for a specific tick number
pub async fn chain_hash(Path(_tick): Path<u32>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
/// Returns quorum data for a specific tick (block height)
pub async fn quorum_tick_data(Path(_tick): Path<u32>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn store_hash(Path(_tick): Path<u32>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}

pub async fn issued_assets(
    Path(_identity): Path<QubicId>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn owned_assets(
    Path(_identity): Path<QubicId>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
pub async fn possessed_assets(
    Path(_identity): Path<QubicId>,
) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}
