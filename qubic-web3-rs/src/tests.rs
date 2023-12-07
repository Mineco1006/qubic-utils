use qubic_tcp_types::{prelude::TransactionFlags, types::{ExchangePublicPeers, WorkSolution}, events::NetworkEvent};
use qubic_types::{QubicId, Nonce};

use crate::{*, transport::Tcp, client::Client};

const COMPUTOR: &str = "135.181.246.50:21841";

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test() {
    use qubic_tcp_types::types::transactions::RawTransaction;
    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    use client::Client;
    use qubic_types::{QubicId, QubicWallet};
    use transport::Tcp;
    let client = Client::<Tcp>::new(COMPUTOR);

    let current_tick = dbg!(client.qu().get_current_tick_info().unwrap());
    let to = QubicId::from_str("BGKBSSHTGNLYOBUNOBYZNPEYDNABWKCHIWGOOUJRTGJOXTYPPWSXMGUAXHKI").unwrap();
    let wallet = QubicWallet::from_seed(seed).unwrap();
    let entity = dbg!(client.qu().request_entity(to).unwrap());
    println!("{}", entity.public_key);
    let balance = entity.incoming_amount - entity.outgoing_amount;
    println!("Balance: {}", balance);

    let tx = RawTransaction {
        from: wallet.public_key,
        to,
        amount: 10,
        tick: current_tick.tick + 30,
        ..Default::default()
    };

    dbg!(tx);
}

const TICK: u32 = 10422573;

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_tick_transactions() {
    let client = Client::<Tcp>::new(COMPUTOR);

    let tick_txns = client.qu().request_tick_transactions(TICK, TransactionFlags::all()).unwrap();

    dbg!(tick_txns);
}

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_subscription() {
    let client = Client::<Tcp>::new(COMPUTOR);

    let (tx, rx) = crossbeam_channel::unbounded::<NetworkEvent>();

    client.qu().subscribe(ExchangePublicPeers::default(), move |event| {
        let tx = tx.clone();
        tx.send(event)?;
        Ok(())
    }).unwrap();

    let mut solutions = 0;
    let mut transactions = 0;
    let mut current_tick = 0;
    let mut ep = 0;

    loop {

        std::thread::sleep(std::time::Duration::from_millis(2_000));
        while !rx.is_empty() {

            match rx.recv().unwrap() {
                NetworkEvent::BroadcastMessage(_) => {
                    solutions += 1;
                },
                NetworkEvent::BroadcastTransaction(tx) => {
                    if tx.verify() {
                        transactions += 1;
                    }
                },
                NetworkEvent::BroadcastTick(t) => {
                    current_tick = t.tick;
                    ep = t.epoch;
                },
                _ => ()
            }
        }

        println!("Tick: {} | EP: {} | Solutions: {} | Transactions: {}", current_tick, ep, solutions, transactions);
    }
}

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_asset() {
    let client = Client::<Tcp>::new("144.76.237.194:21841");

    dbg!(client.qx().get_owned_asset(QubicId::from_str("REOQXSSEZVSGBAWPIGYHPDOUSGYASJQDICASQTVRRGURUTLCFQJWSDNGBZJG").unwrap()).unwrap());
}


#[cfg(any(feature = "async", feature = "http"))]
#[tokio::test]
async fn test() {
    use qubic_tcp_types::prelude::*;
    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    use client::Client;
    use qubic_types::{QubicId, QubicWallet};
    use transport::Tcp;
    let client = Client::<Tcp>::new(COMPUTOR).await;

    let current_tick = dbg!(client.qu().get_current_tick_info().await.unwrap());
    let to = QubicId::from_str("BGKBSSHTGNLYOBUNOBYZNPEYDNABWKCHIWGOOUJRTGJOXTYPPWSXMGUAXHKI").unwrap();
    let wallet = QubicWallet::from_seed(seed).unwrap();
    let entity = dbg!(client.qu().request_entity(wallet.public_key).await.unwrap());
    println!("{}", entity.public_key);
    let balance = entity.incoming_amount - entity.outgoing_amount;
    println!("Balance: {}", balance);
    let tx = RawTransaction {
        from: wallet.public_key,
        to,
        amount: 10,
        tick: current_tick.tick + 30,
        ..Default::default()
    };

    dbg!(tx);
}

#[cfg(any(feature = "async", feature = "http"))]
#[tokio::test]
async fn test_tick_transactions() {
    let client = Client::<Tcp>::new(COMPUTOR).await;

    let tick_txns = client.qu().request_tick_transactions(TICK, TransactionFlags::all()).await.unwrap();

    dbg!(tick_txns);
}

#[cfg(any(feature = "async", feature = "http"))]
#[tokio::test]
async fn test_subscription() {
    let client = Client::<Tcp>::new(COMPUTOR).await;

    let (tx, rx) = crossbeam_channel::unbounded::<NetworkEvent>();

    client.qu().subscribe(ExchangePublicPeers::default(), move |event| {
        let tx = tx.clone();
        tx.send(event)?;
        Ok(())
    }).await.unwrap();

    let mut solutions = 0;
    let mut transactions = 0;
    let mut current_tick = 0;
    let mut ep = 0;

    loop {

        std::thread::sleep(std::time::Duration::from_millis(2_000));
        while !rx.is_empty() {

            match rx.recv().unwrap() {
                NetworkEvent::BroadcastMessage(_) => {
                    solutions += 1;
                },
                NetworkEvent::BroadcastTransaction(_) => {
                    transactions += 1;
                },
                NetworkEvent::BroadcastTick(t) => {
                    current_tick = t.tick;
                    ep = t.epoch;
                },
                _ => ()
            }
        }

        println!("Tick: {} | EP: {} | Solutions: {} | Transactions: {}", current_tick, ep, solutions, transactions);
    }
}