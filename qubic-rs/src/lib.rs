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