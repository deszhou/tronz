# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Core infrastructure
- Initial implementation of `tronz-primitives`, `tronz-signer`, `tronz-provider`, `tronz-contract`.
- gRPC transport via `tonic` targeting TronGrid (mainnet + Nile testnet).
- `ProviderBuilder` with composable filler chain: `TaposFiller`, `FeeLimitFiller`, `SignerFiller`.
- `PendingTransaction` polling (3 s interval, 20 attempts, 60 s timeout).
- `LocalSigner` — in-memory secp256k1 signer from a hex private key.

#### Native contract builders (on `TronProvider`)
- TRX transfer (`send_trx`)
- Stake 2.0: freeze, unfreeze, delegate, undelegate, withdraw expired, cancel all unfreeze
- Claim block/vote rewards (`withdraw_balance`)
- Vote for super representatives (`vote_witness`)
- Account management: create account, update name, update permissions (multisig)
- Smart contract management: set account ID, clear contract ABI, update caller-energy percentage, update origin energy limit

#### TRC10 (`Trc10Api` extension trait)
- Issue a new TRC10 token (`issue_trc10`)
- Transfer TRC10 tokens (`transfer_trc10`)
- Query balance, asset metadata by ID, paginated asset list
- Participate in a TRC10 ICO (`participate_trc10`)
- Release frozen TRC10 supply after lock period (`unfreeze_trc10`)
- Update TRC10 token metadata — description, URL, bandwidth limits (`update_trc10`)
- Look up tokens by name: exact match and list-all-by-name

#### Super representatives (`WitnessApi` extension trait)
- List all SR candidates (`list_witnesses`)
- Query brokerage ratio and unclaimed reward for any address
- Apply to become an SR candidate (`become_witness`)
- Update SR public URL (`update_witness_url`)
- Update SR brokerage ratio (`update_brokerage`)

#### Governance (`GovernanceApi` extension trait)
- List all on-chain governance proposals (`list_proposals`)
- Paginated proposal list (`get_paginated_proposal_list`)
- Fetch a single proposal by numeric ID (`get_proposal_by_id`)
- Submit a chain-parameter proposal (`submit_proposal`)
- Approve or revoke approval for a proposal (`approve_proposal`)
- Cancel a proposal (`cancel_proposal`)
- `ProposalInfo` and `ProposalState` domain types

#### TRC20 / smart contracts
- `Trc20Instance<P>` — typed bindings: `name`, `symbol`, `decimals`, `total_supply`, `balance_of`, `transfer`, `approve`, `transfer_from`, `allowance`
- `ContractInstance` + `CallBuilder` + `DeployBuilder` for dynamic ABI interaction
- `Interface` / `alloy-json-abi` integration for call-by-name
- Event decoding helpers: `decode_logs`, `decode_log`
- Energy estimation via `trigger_constant_contract`

#### Examples (39 total, Nile testnet)
- Read-only: `query`, `address_formats`, `amount_math`, `connect_custom`, `signer_generate`, `signer_local`, `list_witnesses`, `governance_list`, `trc10_query`, `trc10_by_name`, `trc10_balance`, `trc20_decode_transfer_event`, `decode_log`, `decode_receipt`, `contract_call`, `contract_estimate_energy`, `contract_revert`
- With private key: `transfer_trx`, `transfer_trx_memo`, `stake`, `stake_bandwidth`, `delegate`, `undelegate`, `unfreeze`, `cancel_unfreeze`, `withdraw_unfreeze`, `claim_rewards`, `vote_witness`, `trc10_transfer`, `trc10_issue`, `trc10_participate` *(pending)*, `account_create`, `account_update`, `account_permissions`, `trc20`, `trc20_approve`, `trc20_transfer_from`, `contract_send`, `contract_deploy`, `contract_dynamic_abi`

#### CI / tooling
- GitHub Actions: test matrix (ubuntu + windows, stable + nightly + MSRV 1.85), clippy, fmt, docs, typos, cargo-deny, feature-powerset check, CodeQL.
- `Makefile` with local equivalents: `make ci`, `make test`, `make clippy`, `make fmt`, `make docs`, `make typos`, `make deny`, `make features`.
