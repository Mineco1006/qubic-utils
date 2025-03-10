use std::str::FromStr;

use crate::{
    client::Client,
    qubic_tcp_types::{
        events::NetworkEvent,
        prelude::TransactionFlags,
        types::{ticks::TickData, ExchangePublicPeers},
    },
    qubic_types::{traits::VerifySignature, QubicId},
    transport::Tcp,
};

const COMPUTOR: &str = "178.237.58.210:21841"; // check https://app.qubic.li/network/live for current peers

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test() {
    use crate::qubic_tcp_types::types::transactions::RawTransaction;
    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    use crate::{
        client::Client,
        qubic_types::{QubicId, QubicWallet},
        transport::Tcp,
    };
    let client = Client::<Tcp>::new(COMPUTOR).unwrap();

    let current_tick = dbg!(client.qu().get_current_tick_info().unwrap());
    let to =
        QubicId::from_str("BGKBSSHTGNLYOBUNOBYZNPEYDNABWKCHIWGOOUJRTGJOXTYPPWSXMGUAXHKI").unwrap();
    let wallet = QubicWallet::from_seed(seed).unwrap();
    let entity = dbg!(client.qu().request_entity(to).unwrap().entity);
    println!("{}", entity.public_key);
    println!("Balance: {}", entity.balance());

    let tx = RawTransaction {
        from: wallet.public_key,
        to,
        amount: 10,
        tick: current_tick.tick + 30,
        ..Default::default()
    };

    dbg!(tx);
}

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_tick_transactions() {
    let client = Client::<Tcp>::new(COMPUTOR).unwrap();

    let current_tick = client.qu().get_current_tick_info().unwrap();

    dbg!(current_tick);

    let tick_txns = client
        .qu()
        .request_tick_transactions(current_tick.tick - 5, TransactionFlags::all())
        .unwrap();

    for tx in &tick_txns {
        assert!(tx.verify());
    }

    dbg!(tick_txns);
}

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_tick_data() {
    let client = Client::<Tcp>::new(COMPUTOR).unwrap();

    let current_tick = client.qu().get_current_tick_info().unwrap();

    dbg!(current_tick);
    dbg!(std::mem::size_of::<TickData>());

    let tick_data = client
        .qu()
        .request_tick_data(current_tick.tick - 10)
        .unwrap();

    dbg!(tick_data);
}

// disable test, computors giving "resource temporarily unavailable"
// #[cfg(not(any(feature = "async", feature = "http")))]
// #[test]
// fn test_mining_score() {
//     use crate::qubic_types::QubicWallet;

//     let client = Client::<Tcp>::new(COMPUTOR).unwrap();

//     let mining_score = client
//         .qu()
//         .special_command_get_mining_ranking(
//             &QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
//                 .unwrap(),
//         )
//         .unwrap();

//     println!("{:?}", mining_score);
// }

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_period_detection() {
    let client = Client::<Tcp>::new(COMPUTOR).unwrap();

    let current_tick = client.qu().get_current_tick_info().unwrap();

    println!("{:?}", current_tick.tick_period());
}

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_check() {
    let client = Client::<Tcp>::new(COMPUTOR).unwrap();

    // use a new tick to make sure computor will have data about it
    let current_tick = client.qu().get_current_tick_info().unwrap();
    let tick = current_tick.tick - 10;

    // get transactions in tick
    let tx_hash = client
        .qu()
        .request_tick_data(tick)
        .unwrap()
        .transaction_digest[0];

    client.qu().check_transaction_status(tx_hash, tick).unwrap();
}

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_subscription() {
    use crate::qubic_tcp_types::types::transactions::TransactionData;

    let client = Client::<Tcp>::new(COMPUTOR).unwrap();

    let (tx, rx) = crossbeam_channel::unbounded::<NetworkEvent>();

    client
        .qu()
        .subscribe(ExchangePublicPeers::default(), move |event| {
            let tx = tx.clone();
            tx.send(event)?;
            Ok(())
        })
        .unwrap();

    let mut solutions = 0;
    let mut transactions = 0;
    let mut current_tick = 0;
    let mut ep = 0;

    // run 5 times
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        while !rx.is_empty() {
            match rx.recv().unwrap() {
                NetworkEvent::BroadcastMessage(_) => {
                    solutions += 1;
                }
                NetworkEvent::BroadcastTransaction(tx) => {
                    if tx.verify() {
                        match tx.data {
                            TransactionData::SubmitWork { seed: _, nonce: _ } => solutions += 1,
                            _ => transactions += 1,
                        }
                    }
                }
                NetworkEvent::BroadcastTick(t) => {
                    current_tick = t.tick;
                    ep = t.epoch;
                }
                _ => (),
            }
        }

        println!(
            "Tick: {} | EP: {} | Solutions: {} | Transactions: {}",
            current_tick, ep, solutions, transactions
        );
    }
}

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_ipo() {
    use crate::qubic_types::QubicWallet;

    let client = Client::<Tcp>::new(COMPUTOR).unwrap();
    let wallet =
        QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();

    dbg!(client.qu().request_contract_ipo(3).unwrap().public_keys);

    let current_tick = client.qu().get_current_tick_info().unwrap();

    client
        .qu()
        .make_ipo_bid(&wallet, 3, 4, 2, current_tick.tick + 10)
        .unwrap();
}

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_asset() {
    let client = Client::<Tcp>::new(COMPUTOR).unwrap();

    dbg!(client
        .qu()
        .request_entity(
            QubicId::from_str("FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ")
                .unwrap()
        )
        .unwrap());
    dbg!(client
        .qx()
        .request_owned_assets(
            QubicId::from_str("FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ")
                .unwrap()
        )
        .unwrap());
    dbg!(client
        .qx()
        .request_issued_assets(
            QubicId::from_str("FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ")
                .unwrap()
        )
        .unwrap());
}

#[cfg(not(any(feature = "async", feature = "http")))]
#[test]
fn test_tick() {
    let client = Client::<Tcp>::new(COMPUTOR).unwrap();

    let tick = 13237172;
    let id =
        QubicId::from_str("DTWINQHFOSIBVDHKQWCYLAMNSCJDWARQRNAYCHDRIBBFYTSVUWHREYEEUBXF").unwrap();

    let txns = client
        .qu()
        .request_tick_transactions(tick, TransactionFlags::all())
        .unwrap();

    dbg!(txns.len());
    for tx in txns {
        if tx.raw_transaction.to == id {
            dbg!(tx);
        }
    }
}

#[cfg(any(feature = "async", feature = "http"))]
#[tokio::test]
async fn test() {
    use crate::qubic_tcp_types::prelude::*;
    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    use crate::client::Client;
    use crate::qubic_types::{QubicId, QubicWallet};
    use crate::transport::Tcp;
    let client = Client::<Tcp>::new(COMPUTOR).await.unwrap();

    let current_tick = dbg!(client.qu().get_current_tick_info().await.unwrap());
    let to =
        QubicId::from_str("BGKBSSHTGNLYOBUNOBYZNPEYDNABWKCHIWGOOUJRTGJOXTYPPWSXMGUAXHKI").unwrap();
    let wallet = QubicWallet::from_seed(seed).unwrap();
    let entity = dbg!(client.qu().request_entity(to).await.unwrap().entity);
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
    let client = Client::<Tcp>::new(COMPUTOR).await.unwrap();

    let tick_txns = client
        .qu()
        .request_tick_transactions(12380150, TransactionFlags::all())
        .await
        .unwrap();

    dbg!(tick_txns);
}

#[cfg(any(feature = "async", feature = "http"))]
#[tokio::test(flavor = "multi_thread")]
async fn test_subscription() {
    let client = Client::<Tcp>::new(COMPUTOR).await.unwrap();

    let (tx, rx) = crossbeam_channel::unbounded::<NetworkEvent>();

    client
        .qu()
        .subscribe(ExchangePublicPeers::default(), move |event| {
            let tx = tx.clone();
            tx.send(event)?;
            Ok(())
        })
        .await
        .unwrap();

    let mut solutions = 0;
    let mut transactions = 0;
    let mut current_tick = 0;
    let mut ep = 0;

    // run 5 times
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        while !rx.is_empty() {
            match rx.recv().unwrap() {
                NetworkEvent::BroadcastMessage(_) => {
                    solutions += 1;
                }
                NetworkEvent::BroadcastTransaction(_) => {
                    transactions += 1;
                }
                NetworkEvent::BroadcastTick(t) => {
                    current_tick = t.tick;
                    ep = t.epoch;
                }
                _ => (),
            }
        }

        println!(
            "Tick: {} | EP: {} | Solutions: {} | Transactions: {}",
            current_tick, ep, solutions, transactions
        );
    }
}

#[cfg(any(feature = "async", feature = "http"))]
#[tokio::test]
async fn test_read_only_qu() {
    let client = Client::<Tcp>::new(COMPUTOR).await.unwrap();

    dbg!(client
        .qu()
        .request_entity(
            QubicId::from_str("XOHYYIZLBNOAWDRWRMSGFTOBSEPATZLQYNTRBPHFXDAIOYQTGTNFTDABLLFA")
                .unwrap()
        )
        .await
        .unwrap());
    dbg!(client
        .qu()
        .exchange_public_peers(ExchangePublicPeers::default())
        .await
        .unwrap());
    let current_tick = dbg!(client.qu().get_current_tick_info().await.unwrap());
    dbg!(client
        .qu()
        .request_quorum_tick(current_tick.tick - 10, [0u8; (676 + 7) / 8])
        .await
        .unwrap());
    dbg!(client
        .qu()
        .request_tick_data(current_tick.tick - 10)
        .await
        .unwrap());
}

#[cfg(any(feature = "async", feature = "http"))]
#[tokio::test]
async fn test_asset() {
    let client = Client::<Tcp>::new(COMPUTOR).await.unwrap();

    dbg!(client
        .qu()
        .request_entity(
            QubicId::from_str("FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ")
                .unwrap()
        )
        .await
        .unwrap());
    dbg!(client
        .qu()
        .request_owned_assets(
            QubicId::from_str("FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ")
                .unwrap()
        )
        .await
        .unwrap());
    dbg!(client
        .qx()
        .request_issued_assets(
            QubicId::from_str("FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ")
                .unwrap()
        )
        .await
        .unwrap());
}