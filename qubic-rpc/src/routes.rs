use std::{fmt, str::FromStr, sync::Arc};

use anyhow::anyhow;
use axum::{
    extract::{OriginalUri, Path, Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use base64::Engine;
use serde::{de, Deserialize, Deserializer};

use crate::{
    archiver,
    qubic_rpc_types::{
        APIStatus, Balance, BlockHeight, BlockHeightResponse, BroadcastTransactionPayload,
        ComputorsResponse, LatestTick, Pagination, QubicRpcError, RPCStatus, RequestSCPayload,
        RichList, RichListResponse, TickInfoResponse, TickTransactions, TransactionResponse,
        TransactionsResponse, TransferResponse, WalletBalance,
    },
    RPCState,
};
use qubic_rs::{
    qubic_tcp_types::types::transactions::{Transaction, TransactionFlags, TransactionWithData},
    qubic_types::{traits::FromBytes, QubicId},
};

pub async fn index() -> impl IntoResponse {
    Redirect::permanent("/healthcheck")
}
pub async fn latest_tick(
    State(state): State<Arc<RPCState>>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let latest_tick_resp: LatestTick = state.client.qu().get_current_tick_info().await?.into();
    Ok(Json(latest_tick_resp))
}
pub async fn broadcast_transaction(
    State(state): State<Arc<RPCState>>,
    Json(payload): Json<BroadcastTransactionPayload>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let tx = Transaction::from_bytes(
        &base64::engine::general_purpose::STANDARD.decode(payload.encoded_transaction)?,
    )?;
    let _ = state.client.qu().send_signed_transaction(tx).await?;
    Ok(Json("Broadcast successful"))
}
/// Returns the balance of a specific wallet from the API.
pub async fn wallet_balance(
    State(state): State<Arc<RPCState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let public_key = QubicId::from_str(&id)?;
    let entity_response = state.client.qu().request_entity(public_key).await?;
    let balance: Balance = entity_response.into();
    Ok(Json(WalletBalance { balance }))
}
pub async fn status(Path(_id): Path<QubicId>) -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}

/// Returns information for a given transaction
pub async fn transaction(
    State(state): State<Arc<RPCState>>,
    Path(tx_id): Path<String>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let tx_tree = state.db.open_tree("transactions")?;
    if let Some(value) = tx_tree.get(tx_id)? {
        let tx: TransactionWithData = bincode::deserialize(&value)?;
        let tx_resp = TransactionResponse {
            transaction: tx.into(),
            timestamp: 0.to_string(), // TODO: get timestamp
            money_flew: false,        // TODO: get money_flew
        };
        Ok(Json(tx_resp))
    } else {
        Err(anyhow!("Transaction not found").into())
    }
}

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

    let latest_tick = state.client.qu().get_current_tick_info().await?.tick;

    let start_tick = query_params.start_tick.unwrap_or(latest_tick);
    let end_tick = query_params.end_tick.unwrap_or(latest_tick);

    if end_tick < start_tick {
        return Err(anyhow!("end_tick should be higher or equal to start_tick").into());
    }

    let max_tick_range = 100000000;
    if end_tick - start_tick > max_tick_range {
        return Err(anyhow!("tick range too big").into());
    }

    let mut transfer_resp = TransferResponse::new();

    // TODO: support sc_only, desc
    for tick in start_tick..=end_tick {
        let tick_transactions = state
            .client
            .qu()
            .request_tick_transactions(tick, flags)
            .await?
            .into_iter()
            .filter(|tx| tx.raw_transaction.to == id || tx.raw_transaction.from == id)
            .collect::<Vec<_>>();

        if tick_transactions.len() > 0 {
            let tick_transactions = TickTransactions {
                tick_number: tick,
                identity: id.to_string(),
                transactions: tick_transactions.into_iter().map(Into::into).collect(),
            };
            transfer_resp
                .transfer_transactions_per_tick
                .push(tick_transactions);
        }
    }
    Ok(Json(transfer_resp))
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
    State(state): State<Arc<RPCState>>,
    Path(_epoch): Path<u32>, // ignore epoch for now, request_computors only returns for one epoch
) -> Result<impl IntoResponse, QubicRpcError> {
    let computors = state.client.qu().request_computors().await?;
    Ok(Json(ComputorsResponse {
        computors: computors.into(),
    }))
}
pub async fn query_sc(
    State(state): State<Arc<RPCState>>,
    Json(payload): Json<RequestSCPayload>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let resp = state
        .client
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
    let tick_info = state.client.qu().get_current_tick_info().await?;
    Ok(Json(TickInfoResponse { tick_info }))
}
pub async fn block_height(
    State(state): State<Arc<RPCState>>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let tick_info = state.client.qu().get_current_tick_info().await?;

    let block_height = BlockHeight {
        tick: tick_info.tick,
        duration: tick_info.tick_duration,
        epoch: tick_info.epoch,
        initial_tick: tick_info.initial_tick,
    };
    Ok(Json(BlockHeightResponse { block_height }))
}
pub async fn latest_stats() -> Result<impl IntoResponse, QubicRpcError> {
    Ok(Json(""))
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

/// Returns list of wallets with highest recorded balances
/// Uses pagination to return sensible amounts of wallets,
/// if no `page` or `page_size` are passed in the query params
/// 1 and 50 (respectively) are used.
pub async fn rich_list(
    State(state): State<Arc<RPCState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(50);

    if page == 0 || page_size == 0 {
        return Err(anyhow!("page and page_size must both be higher than zero").into());
    }

    let max_page_size = 200;
    let total_records = archiver::rich_list_size(state.db.clone()).await;
    if page_size > max_page_size {
        return Err(anyhow!(format!("page_size must not be higher than {max_page_size}")).into());
    }

    let total_pages = total_records.div_ceil(page_size);
    if page > total_pages {
        return Err(anyhow!(format!(
            "page must not be higher than total_pages ({total_pages})"
        ))
        .into());
    }

    let selected_rich =
        archiver::rich_list((page - 1) * page_size, page_size, state.db.clone()).await?;

    let tick_info = state.client.qu().get_current_tick_info().await?;
    let rich_list_response = RichListResponse {
        pagination: Pagination {
            total_records: total_records.into(),
            total_pages: total_pages.into(),
            current_page: page,
        },
        epoch: tick_info.epoch,
        rich_list: RichList {
            entities: selected_rich,
        },
    };
    Ok(Json(rich_list_response))
}

/// Returns all transactions for a specific tick (block height)
pub async fn tick_transactions(
    State(state): State<Arc<RPCState>>,
    Path(tick): Path<u32>,
) -> Result<impl IntoResponse, QubicRpcError> {
    let flags = TransactionFlags::all();
    let tick_txs = TransactionsResponse {
        transactions: state
            .client
            .qu()
            .request_tick_transactions(tick, flags)
            .await?
            .into_iter()
            .map(Into::into)
            .collect(),
        timestamp: 0.to_string(), // TODO: get timestamp
        money_flew: false,        // TODO: get money_flew
    };

    Ok(Json(tick_txs))
}
/// Returns the approved transactions for a specific tick (block height)
pub async fn approved_tick_transactions(
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
