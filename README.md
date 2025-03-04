# qubic-utils
Rust utilities to interact with the Qubic network. This repo contains the following:
- `qubic-rs`: a Rust SDK to interact with Qubic Computors
- `qubic-rpc`: Rust wrappers and types to interact with qubic RPC servers such as `rpc.qubic.org` (in `qubic-rpc::client`) as well as to spawn your own server (in `qubic-rpc::server`, **server is unfinished**).

## qubic-rpc
### RPC Client
Check cargo docs for examples on how to use the qubic RPC client.

### RPC Server
Starting RPC server (with archiver) using CLI
```bash
cargo run --bin server -- --computor 66.23.193.243
```
**Important:** You must provide a gateway computor which will give the server access to the qubic network. If the gateway goes offline, you'll have to find another one using the [network explorer](https://app.qubic.li/).

Running tests
```bash
cd qubic-rpc
cargo test # test both client and server
cargo test client # only test RPC client
cargo test server # only test RPC server
```
**Important:** If most of the server tests are hanging, it is very likely that the computer being used is offline, please go to the [network explorer](https://app.qubic.li/) to find another peer and copy its IP address into the `mod tests` of `qubic-rpc/src/lib.rs`. Likewise, if many of the client tests are failing, check to see if `rpc.qubic.org` is working properly.

## qubic-rs
Refer to the [example directory](examples) for common usage examples of `qubic-rs`.
