# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Features

- Initial implementation of `tronz-primitives`, `tronz-signer`, `tronz-provider`, `tronz-contract`.
- gRPC transport targeting TronGrid (mainnet + Nile testnet).
- `ProviderBuilder` with composable fillers: `TaposFiller`, `FeeLimitFiller`, `SignerFiller`.
- TRC-20 contract bindings via `Trc20Instance` / `Trc20Ext`.
- Stake 2.0 support: freeze, unfreeze, delegate, undelegate, withdraw.
