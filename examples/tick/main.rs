use qubic_rs::{client::Client, transport::Tcp};

#[tokio::main]
async fn main() {
    // async clients need "async" feature from qubic_rs
    let client = Client::<Tcp>::new("172.67.128.217:21841").await.unwrap();
    println!("{:#?}", client.qu().get_current_tick_info().await.unwrap());
}
