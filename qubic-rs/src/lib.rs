//! qubic-rs is a Rust implementation of the Qubic core client
//!
//! # Examples
//!
//! Get the latest tick
//! ```rust,no_run
//! let client = Client::<Tcp>::new(&state.computor).await.unwrap();
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