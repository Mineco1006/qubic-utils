#![feature(slice_flatten)]

pub mod transport;
pub mod client;

pub extern crate qubic_tcp_types;
pub extern crate qubic_types;

#[cfg(test)]
mod tests;