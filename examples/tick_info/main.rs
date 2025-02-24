use qubic_rs::{client::Client, qubic_tcp_types::types::ticks::CurrentTickInfo, transport::Tcp};

#[tokio::main]
async fn main() {
    // go to https://app.qubic.li/ and navigate to the Peers section
    // to find available computors to request data from
    let computor_address = "188.241.26.108";

    // async clients need "async" feature from qubic_rs
    let client = Client::<Tcp>::new(format!("{computor_address}:21841"))
        .await
        .unwrap();
    let tick_info: CurrentTickInfo = client.qu().get_current_tick_info().await.unwrap();

    println!("{:#?}", tick_info);
}
