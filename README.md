# Tronz

An idiomatic, async-first Rust SDK for the [TRON](https://tron.network) network — inspired by [alloy](https://github.com/alloy-rs/alloy).

> **⚠️ Work in progress.** Tronz is under active development. APIs may change without notice and it is not yet production-ready.

[![Crates.io](https://img.shields.io/crates/v/tronz.svg)](https://crates.io/crates/tronz)
[![License: MIT / Apache-2.0](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](#license)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)

## Features

- **gRPC transport** — connects to TronGrid or any full node via tonic
- **Typed provider** — fluent builder API for every native contract operation
- **Filler chain** — automatic TAPOS, fee-limit, and signing (mirrors alloy's `JoinFill`)
- **TRX / TRC10 / TRC20** — transfers, balance queries, and token metadata
- **Staking v2** — freeze, unfreeze, delegate, claim rewards
- **Contract deploy & call** — `DeployBuilder`, `CallBuilder`, energy estimation
- **Event decoding** — decode and filter logs with `SolEvent`
- **Votes & account management** — SR voting, account activation, name update

## Installation

```toml
[dependencies]
tronz = { version = "0.1", features = ["full"] }
```

## Quick start

### Read the latest block

```rust,no_run
use tronz::{ProviderBuilder, TronProvider, TRONGRID_MAINNET};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = ProviderBuilder::new()
        .on_grpc(TRONGRID_MAINNET)
        .await?;

    let block = provider.get_now_block().await?;
    println!("block #{} at {}ms", block.number, block.timestamp);
    Ok(())
}
```

### Send TRX

```rust,no_run
use tronz::{LocalSigner, ProviderBuilder, TronProvider, TronSigner, Trx, TRONGRID_NILE};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let signer = LocalSigner::from_hex("YOUR_PRIVATE_KEY")?;
    let to = "TRecipientAddress".parse()?;

    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .with_signer(signer)
        .on_grpc(TRONGRID_NILE)
        .await?;

    let pending = provider
        .send_trx()
        .to(to)
        .amount(Trx::from_sun(1_000_000)?) // 1 TRX
        .send()
        .await?;

    let receipt = pending.get_receipt().await?;
    println!("confirmed in block #{}", receipt.block_number);
    Ok(())
}
```

### Call a TRC20 contract

```rust,no_run
use tronz::{ProviderBuilder, TronProvider, TRONGRID_MAINNET};
use tronz::contract::Trc20Ext as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let provider = ProviderBuilder::new()
        .on_grpc(TRONGRID_MAINNET)
        .await?;

    // USDT on mainnet
    let usdt = "TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t".parse()?;
    let holder = "THoldersAddress".parse()?;

    let token = provider.trc20(usdt);
    let balance = token.balance_of(holder).await?;
    let decimals = token.decimals().await?;

    println!("balance: {} (decimals: {})", balance, decimals);
    Ok(())
}
```

## Crates

| Crate | Description |
|---|---|
| [`tronz`](crates/tronz) | Meta-crate — re-exports everything |
| [`tronz-primitives`](crates/primitives) | `Address`, `Trx`, `ResourceCode`, `RecoverableSignature` |
| [`tronz-signer`](crates/signer) | `TronSigner` trait and `LocalSigner` (in-memory secp256k1) |
| [`tronz-provider`](crates/provider) | gRPC transport, provider, fillers, domain types, TRC10 ext |
| [`tronz-contract`](crates/contract) | `ContractInstance`, `DeployBuilder`, TRC20 bindings, event decoding |

## Examples

Runnable examples are in [`examples/`](examples/examples/):

```bash
# Read-only queries (no key needed)
cargo run -p examples --example query

# Send TRX on Nile testnet
TRON_PRIVATE_KEY=<hex> cargo run -p examples --example transfer_trx

# TRC20 transfer on Nile
TRON_PRIVATE_KEY=<hex> cargo run -p examples --example trc20

# Stake TRX for energy
TRON_PRIVATE_KEY=<hex> cargo run -p examples --example stake
```

## Endpoints

| Network | Endpoint |
|---|---|
| Mainnet (TLS) | `https://grpc.trongrid.io:443` |
| Nile testnet | `http://grpc.nile.trongrid.io:50051` |

```rust,no_run
use tronz::{TRONGRID_MAINNET, TRONGRID_NILE};
```

## Minimum Supported Rust Version

**1.85** (Rust 2024 edition).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
