#![feature(slice_flatten)]
#![allow(clippy::let_underscore_future)]
#![allow(clippy::needless_range_loop)]
#![allow(async_fn_in_trait)]


pub mod transport;
pub mod client;

pub extern crate qubic_tcp_types;
pub extern crate qubic_types;

#[cfg(test)]
mod tests;