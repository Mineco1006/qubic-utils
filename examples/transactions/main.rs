use qubic_rs::{
    client::Client,
    qubic_tcp_types::types::transactions::TransactionWithData,
    qubic_types::{traits::Sign, QubicWallet},
    transport::Tcp,
};

#[tokio::main]
async fn main() {
    // go to https://app.qubic.li/ and navigate to the Peers section
    // to find available computors to request data from
    let computor_address = "188.241.26.108";

    // async clients need "async" feature from qubic_rs
    let client = Client::<Tcp>::new(format!("{computor_address}:21841"))
        .await
        .unwrap();

    // create new wallet from seed "aaaa...aaaa"
    let wallet =
        QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    // create new transaction
    let mut tx = TransactionWithData::default();

    // sign transaction with wallet
    let _ = tx.sign(&wallet);

    // broadcast transaction to the network
    client.qu().send_signed_transaction(tx).await.unwrap();

    println!("Transaction sent.");
}
