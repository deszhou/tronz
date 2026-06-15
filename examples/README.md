# tronz Examples

Runnable examples for the [tronz](https://github.com/throgxyz/tronz) Rust SDK.

All network examples default to the **Nile testnet** (`TRONGRID_NILE`). Switch to
mainnet by changing the constant in the source file. Get Nile TRX from the
[Nile faucet](https://nileex.io/).

## Prerequisites

```bash
# Optional but recommended — avoids TronGrid rate limits
export TRON_API_KEY=<your-key>
```

---

## Running examples

```bash
# Read-only — no key needed
cargo run -p examples --example query

# Write — requires a funded Nile private key
TRON_PRIVATE_KEY=<hex> cargo run -p examples --example transfer_trx
```

---

## Primitives

TRON-specific type system: 21-byte addresses, sun-denominated amounts, U256
for token math.

| Example | Description | Status |
|---------|-------------|--------|
| `address_formats` | Parse base58check (`T…`) ↔ hex (`41…`) ↔ EVM (20-byte), derive address from private key | ✅ |
| `amount_math` | TRX ↔ sun conversions, saturating arithmetic, U256 for TRC20 token amounts | ✅ |

---

## Providers

Connect to mainnet or Nile testnet; read-only vs signed providers; custom
endpoints (local full nodes).

| Example | Description | Status |
|---------|-------------|--------|
| `query` | Connect read-only, query latest block, account balance, resources, delegations, pending reward | ✅ |
| `connect_custom` | Connect to a local full node (plain HTTP/2, no TLS); inject TronGrid API key | ✅ |

---

## Accounts

Query and manage accounts. Account activation on TRON costs 1 TRX and must be
performed by an existing account.

| Example | Description | Status |
|---------|-------------|--------|
| `account_create` | Activate a new account by sending 1 TRX to an unused address | ✅ |
| `account_update` | Set a human-readable account name on-chain | ✅ |
| `account_permissions` | Update owner / active multi-sig permissions and key weights | ✅ |

---

## Transactions

Build, sign, and broadcast transactions; read back receipts and decode logs.

| Example | Description | Status |
|---------|-------------|--------|
| `transfer_trx` | Transfer TRX on Nile testnet, poll until confirmed, print receipt | ✅ |
| `transfer_trx_memo` | Attach an on-chain memo (`raw.data`) to a TRX transfer | ✅ |
| `decode_receipt` | Fetch a known transaction by id, print full receipt (status, energy, net, logs) | ✅ |
| `decode_log` | Decode raw event logs from a receipt into typed `SolEvent` structs | ✅ |

---

## Signers

Create and use signing keys. tronz ships `LocalSigner` (in-memory secp256k1)
today; hardware wallet support is planned.

| Example | Description | Status |
|---------|-------------|--------|
| `signer_local` | Create a `LocalSigner` from a hex key; derive the TRON address; sign a hash | ✅ |
| `signer_generate` | Generate a fresh random secp256k1 key pair and print the TRON address | ✅ |

---

## Staking & Resources (Stake 2.0)

TRON's energy and bandwidth system. Stake TRX to acquire resources; delegate
them to other accounts; manage the unfreeze queue.

| Example | Description | Status |
|---------|-------------|--------|
| `stake` | Freeze TRX for energy, optionally delegate to another address, claim rewards | ✅ |
| `stake_bandwidth` | Freeze TRX for bandwidth; compare before/after `free_bandwidth_limit` | ✅ |
| `delegate` | Delegate energy to another account; optionally lock for up to 10 days | ✅ |
| `undelegate` | Reclaim previously delegated energy or bandwidth | ✅ |
| `unfreeze` | Unfreeze staked TRX (enters 3-day pending queue) | ✅ |
| `withdraw_unfreeze` | Withdraw TRX from expired unfreeze entries once the lock period ends | ✅ |
| `cancel_unfreeze` | Cancel all in-progress unfreeze operations and re-stake immediately | ✅ |

---

## Governance

Vote for Super Representatives (SRs) who produce blocks and share rewards.

| Example | Description | Status |
|---------|-------------|--------|
| `list_witnesses` | Fetch and display the current SR and SR candidate list | ✅ |
| `vote_witness` | Cast votes for one or more SR candidates using staked TRON Power | ✅ |
| `claim_rewards` | Claim accumulated block production and voting rewards | ✅ |

---

## TRC20 Tokens

TRC20 is EVM-compatible; tronz uses `alloy-sol-types` for compile-time ABI
encoding/decoding — no JSON ABI file needed.

| Example | Description | Status |
|---------|-------------|--------|
| `trc20` | Query token metadata (name, symbol, decimals, supply) and balance; optionally transfer | ✅ |
| `trc20_approve` | Approve a spender and read back the allowance | ✅ |
| `trc20_transfer_from` | Transfer tokens on behalf of an approver using `transferFrom` | ✅ |
| `trc20_decode_transfer_event` | Decode `Transfer(address,address,uint256)` logs from a confirmed receipt | ✅ |

---

## TRC10 Tokens

TRC10 is TRON's native token standard, predating TRC20. Each asset has a
numeric ID issued on-chain.

| Example | Description | Status |
|---------|-------------|--------|
| `trc10_query` | Look up TRC10 asset metadata (name, decimals, issuer, URL) by asset ID | ✅ |
| `trc10_transfer` | Transfer a TRC10 token to another address | ✅ |
| `trc10_balance` | Read the TRC10 balance map from an `AccountInfo` | ✅ |

---

## Smart Contracts

Deploy contracts, call constant (read-only) and state-changing functions,
estimate energy, decode reverts.

| Example | Description | Status |
|---------|-------------|--------|
| `contract_call` | Constant call (`trigger_constant_contract`) — read state without spending energy | ✅ |
| `contract_send` | State-changing call (`trigger_smart_contract`) — update state, poll receipt | ✅ |
| `contract_deploy` | Deploy a compiled contract from bytecode; extract the new contract address | ✅ |
| `contract_estimate_energy` | Estimate energy consumption before executing a contract call | ✅ |
| `contract_revert` | Trigger a known-revert function and decode the ABI-encoded revert reason | ✅ |
| `contract_dynamic_abi` | Use `Interface` with a JSON ABI file for runtime encoding/decoding | ✅ |

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented and runnable |

---

## Environment variables

| Variable | Used by | Description |
|----------|---------|-------------|
| `TRON_PRIVATE_KEY` | all write examples | Hex-encoded secp256k1 private key (no `0x` prefix) |
| `TRON_API_KEY` | all examples | TronGrid API key — optional, avoids 429 rate limits |
| `TRON_ADDRESS` | query, trc20, trc10_* | Address to query (defaults vary per example) |
| `TRON_TO` | transfer_trx, trc20, delegate, … | Recipient / target address |
| `TRON_AMOUNT_SUN` | transfer_trx | TRX amount in sun (default: 1 TRX = 1 000 000 sun) |
| `TRON_CONTRACT` | trc20, contract_* | Contract address |
| `TRON_FREEZE_SUN` | stake, stake_bandwidth | Amount to freeze in sun (default: 10 TRX) |
| `TRON_DELEGATE_TO` | stake, delegate | Address to delegate resources to |

---

## Notes on TRON vs Ethereum differences

These come up repeatedly across the examples and are worth internalising:

- **Addresses** — 21 bytes (`0x41` prefix + 20-byte EVM body); displayed in
  base58check as `T…` strings. Use `.as_evm_bytes()` when feeding into ABI
  encoding.
- **No gas price** — TRON uses energy (for contracts) and bandwidth (for all
  transactions). Both can be obtained by staking TRX or consumed by paying TRX
  fees directly.
- **fee\_limit** — Smart contract calls require a `fee_limit` (max TRX to spend
  on energy). `FeeLimitFiller` sets a 20 TRX default automatically.
- **TAPOS** — Every transaction references the two most recent block bytes to
  prevent replay attacks. `TaposFiller` handles this automatically.
- **Stake 2.0** — Staking is non-custodial: frozen TRX stays in your account.
  Energy/bandwidth can be delegated to any address and reclaimed at any time
  (or locked for up to 10 days for a bonus multiplier).
- **TRC20 is EVM-compatible** — The contract ABI, selector computation,
  and event encoding are identical to ERC-20; only the address format and
  transport differ.
