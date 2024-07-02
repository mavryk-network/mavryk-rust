# Mavryk-Rust

---

Utility crates, forked from [tezedge](github.com/trilitech/tezedge), used as part of the `Kernel SDK` for mavryk smart rollups.

Namely:
- [mavryk_crypto](./crypto/README.md) 
- [mavryk_encoding](./mavryk-encoding/README.md) 
- [mavryk_encoding_derive](./mavryk-encoding-derive/README.md)

## Setup

The following prerequisites are required:

- rust 1.60, with the `wasm32-unknown-unknown` target.
- clang - tested with `v11`.

> If running on MacOS - you will need to install llvm with brew, and ensure the brew-install is available in your path, rather than the default installation.

You should then be able to run:

```shell
cargo build
cargo test
cargo build --target wasm32-unknown-unknown --no-default-features
```
