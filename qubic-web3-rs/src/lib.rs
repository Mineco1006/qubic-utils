#![feature(impl_trait_in_assoc_type)]

pub mod transport;
pub mod client;

pub extern crate qubic_tcp_types;
pub extern crate qubic_types;


#[test]
fn test() {
    use qubic_tcp_types::types::RawTransaction;
    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    use client::Client;
    use qubic_types::{QubicId, QubicWallet};
    use transport::Tcp;
    let client = Client::<Tcp>::new("136.243.41.86:21842");

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

#[cfg(any(feature = "async", feature = "http"))]
#[tokio::test]
async fn test() {
    use qubic_tcp_types::types::RawTransaction;
    let seed = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    use client::Client;
    use qubic_types::{QubicId, QubicWallet};
    use transport::Tcp;
    let client = Client::<Tcp>::new("136.243.41.86:21841").await;

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