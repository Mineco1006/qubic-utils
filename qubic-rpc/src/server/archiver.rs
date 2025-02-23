use std::{str::FromStr, sync::Arc};

use anyhow::Result;
use async_channel::{Receiver, Sender};
use log::debug;
use serde::{Deserialize, Serialize};
use sled::Db;

use crate::qubic_rpc_types::RichEntity;
use qubic_rs::{
    client::Client,
    qubic_tcp_types::types::transactions::TransactionFlags,
    qubic_types::{QubicId, QubicTxHash},
    transport::Tcp,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletEntry {
    pub identity: String,
    pub balance: String,
    pub valid_for_tick: u32,
}

/// Producer puts ticks into a channel for processing.
/// Produces 10 ticks backwards and then checks for new ticks
/// (e.g. from 1010 it will add 1009 to 999 and then check to see where the network is at,
/// if it is at 1012, it'll add 1011 and 1012)
pub async fn producer(tx: Sender<u32>, client: Arc<Client<Tcp>>) {
    let current_tick = client
        .qu()
        .get_current_tick_info()
        .await
        .expect("Could not get current tick")
        .tick;

    if let Err(_) = tx.send(current_tick).await {
        eprintln!("Receiver dropped, stopping producer.");
        return;
    }

    let mut latest_viewed_tick = current_tick;
    let mut earliest_viewed_tick = current_tick;
    loop {
        // add 10 ticks backwards
        if earliest_viewed_tick > 0 {
            for i in 1..=10 {
                let new_tick = earliest_viewed_tick.saturating_sub(i);
                if let Err(_) = tx.send(new_tick).await {
                    eprintln!("Receiver dropped, stopping producer.");
                    return;
                }
                earliest_viewed_tick = new_tick;
            }
        }

        // check for forward ticks
        let current_tick = client
            .qu()
            .get_current_tick_info()
            .await
            .expect("Could not get current tick")
            .tick;

        // add all ticks between current and latest_viewed_tick
        for tick in (latest_viewed_tick + 1)..=current_tick {
            if let Err(_) = tx.send(tick).await {
                eprintln!("Receiver dropped, stopping producer.");
                return;
            }
        }
        latest_viewed_tick = current_tick;

        // wait not to overload channel with mostly older ticks
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

/// Updates wallet if already in database, creates new one if not already present
async fn update_wallet(
    tick: u32,
    wallet_id: String,
    wallet_tree: Arc<sled::Tree>,
    client: Arc<Client<Tcp>>,
) -> Result<()> {
    let pk = QubicId::from_str(&wallet_id)?;
    let entity_response = client.qu().request_entity(pk).await?;

    let wallet: WalletEntry = match wallet_tree.get(wallet_id.as_bytes())? {
        Some(bytes) => {
            let mut wallet: WalletEntry = bincode::deserialize(&bytes)?;

            // Only update if the tick is newer
            if wallet.valid_for_tick < tick {
                wallet.valid_for_tick = entity_response.tick;
                wallet.balance = entity_response.entity.balance().to_string();
            }
            wallet
        }
        None => {
            // If wallet doesn't exist, create a new one
            WalletEntry {
                identity: wallet_id.clone(),
                valid_for_tick: entity_response.tick,
                balance: entity_response.entity.balance().to_string(),
            }
        }
    };
    let balance: u64 = wallet.balance.parse()?;
    let mut key = balance.to_be_bytes().to_vec();
    key.extend_from_slice(wallet_id.as_bytes());
    wallet_tree.insert(&key, bincode::serialize(&wallet)?)?;

    Ok(())
}

/// Consumer gets ticks from channel and queries all transactions from that tick.
/// For every transaction, update wallets of src and dst on database (if transaction is newer than
/// last wallet update).
/// If wallets are not already in database, query and add them.
pub async fn consumer(
    id: usize,
    rx: Receiver<u32>,
    db: Arc<Db>,
    client: Arc<Client<Tcp>>,
) -> Result<()> {
    let wallet_tree = Arc::new(db.open_tree("wallets").unwrap());
    let tx_tree = Arc::new(db.open_tree("transactions").unwrap());
    while let Ok(tick) = rx.recv().await {
        let tick_transactions = client
            .qu()
            .request_tick_transactions(tick, TransactionFlags::all())
            .await
            .expect("Could not get transactions for tick {tick}");

        for tx in tick_transactions {
            // update transactions
            let serialized = bincode::serialize(&tx).unwrap();
            let tx_hash: QubicTxHash = tx.clone().into();
            tx_tree.insert(tx_hash.get_identity(), serialized).unwrap();

            // update wallets
            update_wallet(
                tick,
                tx.raw_transaction.from.get_identity(),
                wallet_tree.clone(),
                client.clone(),
            )
            .await?;
            update_wallet(
                tick,
                tx.raw_transaction.to.get_identity(),
                wallet_tree.clone(),
                client.clone(),
            )
            .await?;
        }
        debug!("Consumer {} processed tick {}", id, tick);
    }
    Ok(())
}

/// Returns the size of the collected rich list
pub async fn rich_list_size(db: Arc<Db>) -> usize {
    db.open_tree("wallets").unwrap().len()
}

/// Returns up to `size` of the richest wallets in descending order starting from index `start_idx`
pub async fn rich_list(start_idx: usize, size: usize, db: Arc<Db>) -> Result<Vec<RichEntity>> {
    let wallet_tree = Arc::new(db.open_tree("wallets")?);

    // make sure we don't get more than db size
    let size = size.min(wallet_tree.len().try_into()?);

    // Get the top wallet IDs by iterating the balance index in descending order
    let mut top_wallets: Vec<RichEntity> = Vec::new();

    for result in wallet_tree.iter().rev().skip(start_idx).take(size) {
        let (_key, value) = result?;
        let wallet: WalletEntry = bincode::deserialize(&value)?;

        top_wallets.push(RichEntity {
            identity: wallet.identity,
            balance: wallet.balance.parse()?,
        });
    }

    Ok(top_wallets)
}
