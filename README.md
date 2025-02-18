# qubic-utils
Rust utilities to interact with the Qubic network.

## qubic-rpc
Starting RPC server (with archiver) using CLI
```bash
cargo run --bin server -- --computor 66.23.193.243
```
**Important:** You must provide a gateway computor which will give the server access to the qubic network. If the gateway goes offline, you'll have to find another one using the [network explorer](https://app.qubic.li/).

Running tests
```bash
cd qubic-rpc
cargo test
```

## qubic-rs

