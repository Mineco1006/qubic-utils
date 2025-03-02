//! qubic-rs is a Rust implementation of the Qubic core client
//!
//! # Examples
//!
//! Get the latest tick
//! ```rust,ignore
//! const COMPUTOR: &str = "178.237.58.210:21841"; // check https://app.qubic.li/network/live for current peers
//! let client = Client::<Tcp>::new(COMPUTOR).await.unwrap();
//! let res = client.qu().get_current_tick_info().await;
//! ```
//!
//! More examples are shown in the [examples][examples] directory
//!

#![allow(clippy::let_underscore_future)]
#![allow(clippy::needless_range_loop)]
#![allow(async_fn_in_trait)]

pub mod client;
pub mod qubic_tcp_types;
pub mod qubic_types;
pub mod transport;

#[macro_use]
pub extern crate alloc;

#[cfg(test)]
mod tests;