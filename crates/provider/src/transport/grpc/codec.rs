//! Proto ↔ domain type conversions for the gRPC transport.
//!
//! All functions are `pub(super)` — only the gRPC transport module needs them.

use prost::Message as _;
use tronz_primitives::{Address, B256, Bytes, RecoverableSignature, Trx, TxId};

use crate::{
    error::TransportError,
    proto,
    types::{
        AccountInfo, AccountPermissionUpdateContract, AccountPermissions, AccountResource,
        AssetInfo, AssetIssueContract, BlockInfo, ConstantCallResult, ContractResult,
        CreateAccountContract, CreateSmartContract, DelegatedResource, DelegatedResourceIndex,
        FreezeV2, Log, Permission, PermissionKey, RawTransaction, SignedTransaction,
        SmartContractInfo, TransactionInfo, TransferAssetContract, TransferContract,
        TriggerSmartContract, TxStatus, UnfreezeV2, UpdateAccountContract, Vote,
        VoteWitnessContract, WitnessInfo,
    },
};

// ── helpers ──────────────────────────────────────────────────────────────────

fn addr(bytes: Vec<u8>) -> Result<Address, TransportError> {
    Address::from_slice(&bytes).map_err(|e| TransportError::Malformed(format!("bad address: {e}")))
}

fn opt_addr(bytes: Vec<u8>) -> Option<Address> {
    if bytes.is_empty() {
        None
    } else {
        Address::from_slice(&bytes).ok()
    }
}

/// Convert a byte vec to a B256. Returns `B256::ZERO` when the slice is not
/// exactly 32 bytes.  Acceptable for log topics (wrong-length data from the
/// node simply won't match any filter), but **not** for block hashes —
/// see [`block_from_extention`] which validates the length explicitly.
fn b256(bytes: Vec<u8>) -> B256 {
    if bytes.len() == 32 {
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        B256::from(arr)
    } else {
        B256::ZERO
    }
}

// ── Block ─────────────────────────────────────────────────────────────────────

pub(super) fn block_from_extention(
    ext: proto::BlockExtention,
) -> Result<BlockInfo, TransportError> {
    let header = ext
        .block_header
        .ok_or_else(|| TransportError::Malformed("missing block_header".into()))?;
    let raw = header
        .raw_data
        .ok_or_else(|| TransportError::Malformed("missing block_header.raw_data".into()))?;

    // Block id must be exactly 32 bytes — a wrong length here would silently
    // corrupt TAPOS (ref_block_hash becomes zero → node rejects the tx).
    let blockid = ext.blockid;
    if blockid.len() != 32 {
        return Err(TransportError::Malformed(format!(
            "blockid must be 32 bytes, got {}",
            blockid.len()
        )));
    }

    Ok(BlockInfo {
        number: raw.number,
        hash: b256(blockid),
        timestamp: raw.timestamp,
    })
}

// ── Account ───────────────────────────────────────────────────────────────────

/// Convert a proto `Account` into `AccountInfo`.
///
/// `queried` is the address that was requested — used as a fallback when the
/// node returns an empty address field (happens for non-existent accounts on
/// some TRON fullnode versions).
pub(super) fn account_from_proto(
    a: proto::Account,
    queried: Address,
) -> Result<AccountInfo, TransportError> {
    let is_activated = !a.address.is_empty();
    let address = if a.address.is_empty() {
        queried
    } else {
        addr(a.address.clone())?
    };

    let frozen_v2 = a
        .frozen_v2
        .into_iter()
        .filter_map(|f| {
            tronz_primitives::ResourceCode::from_i32(f.r#type).map(|r| FreezeV2 {
                resource: r,
                amount: Trx::from_sun_unchecked(f.amount),
            })
        })
        .collect();

    let unfrozen_v2 = a
        .unfrozen_v2
        .into_iter()
        .filter_map(|u| {
            tronz_primitives::ResourceCode::from_i32(u.r#type).map(|r| UnfreezeV2 {
                resource: r,
                amount: Trx::from_sun_unchecked(u.unfreeze_amount),
                expire_time_ms: u.unfreeze_expire_time,
            })
        })
        .collect();

    let votes = a
        .votes
        .into_iter()
        .filter_map(|v| {
            addr(v.vote_address).ok().map(|va| Vote {
                vote_address: va,
                vote_count: v.vote_count,
            })
        })
        .collect();

    let permissions = AccountPermissions {
        owner: a
            .owner_permission
            .and_then(|p| permission_from_proto(p).ok()),
        witness: a
            .witness_permission
            .and_then(|p| permission_from_proto(p).ok()),
        actives: a
            .active_permission
            .into_iter()
            .filter_map(|p| permission_from_proto(p).ok())
            .collect(),
    };

    Ok(AccountInfo {
        address,
        balance: Trx::from_sun_unchecked(a.balance),
        name: String::from_utf8_lossy(&a.account_name).into_owned(),
        is_activated,
        frozen_v2,
        unfrozen_v2,
        votes,
        permissions,
        trc10_balances: a.asset_v2,
    })
}

fn permission_from_proto(p: proto::Permission) -> Result<Permission, TransportError> {
    let keys = p
        .keys
        .into_iter()
        .filter_map(|k| {
            addr(k.address).ok().map(|a| PermissionKey {
                address: a,
                weight: k.weight,
            })
        })
        .collect();
    Ok(Permission {
        id: p.id,
        permission_name: p.permission_name,
        threshold: p.threshold,
        keys,
    })
}

pub(super) fn account_resource_from_proto(r: proto::AccountResourceMessage) -> AccountResource {
    AccountResource {
        free_bandwidth_used: r.free_net_used,
        free_bandwidth_limit: r.free_net_limit,
        bandwidth_used: r.net_used,
        bandwidth_limit: r.net_limit,
        energy_used: r.energy_used,
        energy_limit: r.energy_limit,
        // tronPowerUsed / tronPowerLimit are in TRX units (1 vote = 1 TRX),
        // not sun — multiply by 1_000_000 to convert to the sun-based Trx type.
        tron_power_used: Trx::from_sun_unchecked(r.tron_power_used * 1_000_000),
        tron_power_limit: Trx::from_sun_unchecked(r.tron_power_limit * 1_000_000),
        ..Default::default()
    }
}

// ── Transaction ───────────────────────────────────────────────────────────────

pub(super) fn signed_tx_from_proto(
    tx: proto::Transaction,
) -> Result<SignedTransaction, TransportError> {
    let (expiration, timestamp) = tx
        .raw_data
        .as_ref()
        .map(|r| (r.expiration, r.timestamp))
        .unwrap_or((0, 0));

    // Compute txid = sha256(raw_data encoded bytes)
    let tx_id_bytes: [u8; 32] = if let Some(ref raw) = tx.raw_data {
        use sha2::{Digest, Sha256};
        let encoded = raw.encode_to_vec();
        Sha256::digest(&encoded).into()
    } else {
        [0u8; 32]
    };
    let tx_id = TxId::from(tx_id_bytes);

    let signatures: Vec<RecoverableSignature> = tx
        .signature
        .iter()
        .filter_map(|s| RecoverableSignature::from_bytes(s).ok())
        .collect();

    let raw_proto = tx.encode_to_vec();
    let raw = RawTransaction::from_proto_extention(
        tx_id.as_slice().to_vec(),
        raw_proto,
        expiration,
        timestamp,
    )?;

    Ok(SignedTransaction { raw, signatures })
}

// ── Transaction info ───────────────────────────────────────────────────────────

pub(super) fn transaction_info_from_proto(
    info: proto::TransactionInfo,
) -> Result<TransactionInfo, TransportError> {
    // An empty id means the node hasn't indexed this transaction yet.
    if info.id.is_empty() {
        return Err(TransportError::NotFound);
    }

    let tx_id = {
        let bytes: [u8; 32] = info
            .id
            .try_into()
            .map_err(|_| TransportError::Malformed("bad txid length".into()))?;
        TxId::from(bytes)
    };

    let status = if info.result == 0 {
        TxStatus::Success
    } else {
        TxStatus::Failed
    };

    let receipt = info.receipt.unwrap_or_default();
    let contract_result = match receipt.result {
        1 => ContractResult::Success,
        2 => ContractResult::Revert,
        10 => ContractResult::OutOfEnergy,
        r if r != 0 => ContractResult::Failed,
        _ => ContractResult::Default,
    };

    let logs = info
        .log
        .into_iter()
        .map(|l| Log {
            address: opt_addr(l.address).unwrap_or_else(|| Address::from_evm_bytes([0u8; 20])),
            topics: l.topics.into_iter().map(b256).collect(),
            data: Bytes::from(l.data),
        })
        .collect();

    let revert_reason = if info.res_message.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&info.res_message).into_owned())
    };

    Ok(TransactionInfo {
        tx_id,
        block_number: info.block_number,
        block_timestamp: info.block_time_stamp,
        status,
        energy_usage: receipt.energy_usage_total,
        energy_fee: Trx::from_sun_unchecked(receipt.energy_fee),
        net_usage: receipt.net_usage,
        net_fee: Trx::from_sun_unchecked(receipt.net_fee),
        contract_result,
        contract_address: opt_addr(info.contract_address),
        logs,
        revert_reason,
    })
}

// ── Smart contract ─────────────────────────────────────────────────────────────

pub(super) fn trigger_smart_contract_to_proto(
    p: TriggerSmartContract,
) -> proto::TriggerSmartContract {
    proto::TriggerSmartContract {
        owner_address: p.owner_address.as_bytes().to_vec(),
        contract_address: p.contract_address.as_bytes().to_vec(),
        call_value: p.call_value.as_sun(),
        data: p.data.to_vec(),
        call_token_value: p.call_token_value.as_sun(),
        token_id: p.token_id,
    }
}

pub(super) fn constant_result_from_extention(
    ext: proto::TransactionExtention,
) -> Result<ConstantCallResult, TransportError> {
    let output = ext.constant_result.into_iter().next().unwrap_or_default();

    let revert_reason = if let Some(ref r) = ext.result {
        if !r.result {
            let msg = String::from_utf8_lossy(&r.message).into_owned();
            if output.is_empty() {
                // Protocol-level failure with no EVM output — surface as an error.
                return Err(TransportError::NodeError(msg));
            }
            // EVM reverted and left ABI-encoded revert data in output.
            Some(msg)
        } else {
            None
        }
    } else {
        None
    };

    Ok(ConstantCallResult {
        output,
        energy_used: ext.energy_used,
        revert_reason,
    })
}

/// Convert a proto `SmartContract.ABI` to a JSON ABI byte array compatible
/// with alloy's `JsonAbi` / EIP-712 tooling.
fn abi_to_json(abi: proto::smart_contract::Abi) -> Vec<u8> {
    fn state_mutability(v: i32) -> &'static str {
        match v {
            1 => "pure",
            2 => "view",
            4 => "payable",
            _ => "nonpayable",
        }
    }

    fn param_to_json(p: &proto::smart_contract::abi::entry::Param) -> serde_json::Value {
        let mut obj = serde_json::json!({ "name": p.name, "type": p.r#type });
        if p.indexed {
            obj["indexed"] = serde_json::json!(true);
        }
        obj
    }

    let entries: Vec<serde_json::Value> = abi
        .entrys
        .into_iter()
        .filter_map(|e| {
            // EntryType: 0=Unknown, 1=Constructor, 2=Function, 3=Event,
            //            4=Fallback, 5=Receive, 6=Error
            let entry = match e.r#type {
                1 => serde_json::json!({
                    "type": "constructor",
                    "inputs": e.inputs.iter().map(param_to_json).collect::<Vec<_>>(),
                    "stateMutability": state_mutability(e.state_mutability),
                }),
                2 => serde_json::json!({
                    "type": "function",
                    "name": e.name,
                    "inputs": e.inputs.iter().map(param_to_json).collect::<Vec<_>>(),
                    "outputs": e.outputs.iter().map(param_to_json).collect::<Vec<_>>(),
                    "stateMutability": state_mutability(e.state_mutability),
                }),
                3 => serde_json::json!({
                    "type": "event",
                    "name": e.name,
                    "inputs": e.inputs.iter().map(param_to_json).collect::<Vec<_>>(),
                    "anonymous": e.anonymous,
                }),
                4 => serde_json::json!({
                    "type": "fallback",
                    "stateMutability": state_mutability(e.state_mutability),
                }),
                5 => serde_json::json!({
                    "type": "receive",
                    "stateMutability": "payable",
                }),
                6 => serde_json::json!({
                    "type": "error",
                    "name": e.name,
                    "inputs": e.inputs.iter().map(param_to_json).collect::<Vec<_>>(),
                }),
                _ => return None, // skip UnknownEntryType
            };
            Some(entry)
        })
        .collect();

    serde_json::to_vec(&entries).unwrap_or_default()
}

pub(super) fn smart_contract_from_proto(c: proto::SmartContract) -> SmartContractInfo {
    SmartContractInfo {
        address: opt_addr(c.contract_address),
        origin_address: opt_addr(c.origin_address),
        abi: c.abi.map(abi_to_json).unwrap_or_default(),
        bytecode: Bytes::from(c.bytecode),
        runtime_bytecode: None,
        name: c.name,
        consume_user_resource_percent: c.consume_user_resource_percent,
        origin_energy_limit: c.origin_energy_limit,
    }
}

pub(super) fn smart_contract_info_from_wrapper(
    w: proto::SmartContractDataWrapper,
) -> SmartContractInfo {
    let mut info = w
        .smart_contract
        .map(smart_contract_from_proto)
        .unwrap_or_default();
    if !w.runtimecode.is_empty() {
        info.runtime_bytecode = Some(Bytes::from(w.runtimecode));
    }
    info
}

pub(super) fn witness_from_proto(w: proto::Witness) -> Option<WitnessInfo> {
    let address = opt_addr(w.address)?;
    Some(WitnessInfo {
        address,
        vote_count: w.vote_count,
        url: w.url,
        total_produced: w.total_produced,
        total_missed: w.total_missed,
        is_active: w.is_jobs,
    })
}

// ── Delegated resource ─────────────────────────────────────────────────────────

pub(super) fn delegated_resource_from_proto(
    d: proto::DelegatedResource,
) -> Result<DelegatedResource, TransportError> {
    Ok(DelegatedResource {
        from: addr(d.from)?,
        to: addr(d.to)?,
        bandwidth_amount: Trx::from_sun_unchecked(d.frozen_balance_for_bandwidth),
        energy_amount: Trx::from_sun_unchecked(d.frozen_balance_for_energy),
        bandwidth_expire_time_ms: d.expire_time_for_bandwidth,
        energy_expire_time_ms: d.expire_time_for_energy,
    })
}

// ── Native contracts (to proto) ────────────────────────────────────────────────

pub(super) fn transfer_to_proto(p: TransferContract) -> proto::TransferContract {
    proto::TransferContract {
        owner_address: p.owner_address.as_bytes().to_vec(),
        to_address: p.to_address.as_bytes().to_vec(),
        amount: p.amount.as_sun(),
    }
}

fn permission_to_proto(p: Permission) -> proto::Permission {
    use proto::permission::PermissionType;
    proto::Permission {
        r#type: PermissionType::Active as i32, // overridden by caller for owner/witness
        id: p.id,
        permission_name: p.permission_name,
        threshold: p.threshold,
        parent_id: 0,
        operations: vec![],
        keys: p
            .keys
            .into_iter()
            .map(|k| proto::Key {
                address: k.address.as_bytes().to_vec(),
                weight: k.weight,
            })
            .collect(),
    }
}

pub(super) fn account_permission_update_to_proto(
    p: AccountPermissionUpdateContract,
) -> proto::AccountPermissionUpdateContract {
    use proto::permission::PermissionType;

    let owner = p.owner.map(|perm| {
        let mut proto_perm = permission_to_proto(perm);
        proto_perm.r#type = PermissionType::Owner as i32;
        proto_perm
    });

    let witness = p.witness.map(|perm| {
        let mut proto_perm = permission_to_proto(perm);
        proto_perm.r#type = PermissionType::Witness as i32;
        proto_perm
    });

    // The `operations` field is a 32-byte bitfield: bit N (byte N/8, bit N%8
    // from LSB) represents ContractType N. Only set bits for types that actually
    // exist in the proto enum; setting a bit for a non-existent type causes
    // "X isn't a validate ContractType" from the node.
    //
    // Proto ContractType values (from Tron.proto Transaction.Contract.ContractType):
    //   Byte 0: 0–6 valid, 7 missing → 0x7f
    //   Byte 1: 8–15 all valid       → 0xff
    //   Byte 2: 16–20 valid, 21–23 missing → 0x1f
    //   Byte 3: 30–31 valid, 24–29 missing → 0xc0
    //   Byte 4: 32–33 valid, 34–39 missing → 0x03
    //   Byte 5: 41–46 valid, 40 & 47 missing → 0x7e
    //   Byte 6: 48–49, 51–55 valid, 50 missing → 0xfb
    //   Byte 7: 56–59 valid, 60+ missing → 0x0f
    //   Bytes 8–31: no valid types → 0x00
    const ACTIVE_OPERATIONS: [u8; 32] = {
        let mut ops = [0u8; 32];
        ops[0] = 0x7f;
        ops[1] = 0xff;
        ops[2] = 0x1f;
        ops[3] = 0xc0;
        ops[4] = 0x03;
        ops[5] = 0x7e;
        ops[6] = 0xfb;
        ops[7] = 0x0f;
        ops
    };

    let actives = p
        .actives
        .into_iter()
        .map(|perm| {
            let mut proto_perm = permission_to_proto(perm);
            proto_perm.r#type = PermissionType::Active as i32;
            proto_perm.operations = ACTIVE_OPERATIONS.to_vec();
            proto_perm
        })
        .collect();

    proto::AccountPermissionUpdateContract {
        owner_address: p.owner_address.as_bytes().to_vec(),
        owner,
        witness,
        actives,
    }
}

pub(super) fn create_smart_contract_to_proto(p: CreateSmartContract) -> proto::CreateSmartContract {
    proto::CreateSmartContract {
        owner_address: p.owner_address.as_bytes().to_vec(),
        new_contract: Some(proto::SmartContract {
            origin_address: p.owner_address.as_bytes().to_vec(),
            contract_address: vec![],
            abi: None,
            bytecode: p.bytecode.to_vec(),
            call_value: p.call_value.as_sun(),
            consume_user_resource_percent: p.consume_user_resource_percent,
            name: p.name,
            origin_energy_limit: p.origin_energy_limit,
            code_hash: vec![],
            trx_hash: vec![],
            version: 0,
        }),
        call_token_value: 0,
        token_id: 0,
    }
}

// ── TRC10 ─────────────────────────────────────────────────────────────────────

pub(super) fn asset_issue_to_proto(p: AssetIssueContract) -> proto::AssetIssueContract {
    proto::AssetIssueContract {
        owner_address: p.owner_address.as_bytes().to_vec(),
        name: p.name.into_bytes(),
        abbr: p.abbr.into_bytes(),
        description: p.description.into_bytes(),
        url: p.url.into_bytes(),
        total_supply: p.total_supply,
        precision: p.precision,
        trx_num: p.trx_num,
        num: p.num,
        start_time: p.start_time,
        end_time: p.end_time,
        free_asset_net_limit: p.free_asset_net_limit,
        public_free_asset_net_limit: p.public_free_asset_net_limit,
        frozen_supply: p
            .frozen_supply
            .into_iter()
            .map(|f| proto::asset_issue_contract::FrozenSupply {
                frozen_amount: f.frozen_amount,
                frozen_days: f.frozen_days,
            })
            .collect(),
        ..Default::default()
    }
}

pub(super) fn transfer_asset_to_proto(p: TransferAssetContract) -> proto::TransferAssetContract {
    proto::TransferAssetContract {
        // After the ALLOW_SAME_TOKEN_NAME proposal, asset_name holds the numeric ID as bytes.
        asset_name: p.token_id.into_bytes(),
        owner_address: p.owner_address.as_bytes().to_vec(),
        to_address: p.to_address.as_bytes().to_vec(),
        amount: p.amount,
    }
}

pub(super) fn create_account_to_proto(p: CreateAccountContract) -> proto::AccountCreateContract {
    proto::AccountCreateContract {
        owner_address: p.owner_address.as_bytes().to_vec(),
        account_address: p.account_address.as_bytes().to_vec(),
        r#type: 0, // Normal account
    }
}

pub(super) fn vote_witness_to_proto(p: VoteWitnessContract) -> proto::VoteWitnessContract {
    proto::VoteWitnessContract {
        owner_address: p.owner_address.as_bytes().to_vec(),
        votes: p
            .votes
            .into_iter()
            .map(|v| proto::vote_witness_contract::Vote {
                vote_address: v.vote_address.as_bytes().to_vec(),
                vote_count: v.vote_count,
            })
            .collect(),
        support: false,
    }
}

pub(super) fn update_account_to_proto(p: UpdateAccountContract) -> proto::AccountUpdateContract {
    proto::AccountUpdateContract {
        account_name: p.name.into_bytes(),
        owner_address: p.owner_address.as_bytes().to_vec(),
    }
}

pub(super) fn asset_info_from_proto(
    a: proto::AssetIssueContract,
) -> Result<AssetInfo, TransportError> {
    // An empty id means the token was not found.
    if a.id.is_empty() {
        return Err(TransportError::NotFound);
    }
    let owner = addr(a.owner_address)?;
    Ok(AssetInfo {
        id: a.id,
        name: String::from_utf8_lossy(&a.name).into_owned(),
        abbr: String::from_utf8_lossy(&a.abbr).into_owned(),
        decimals: a.precision,
        owner,
        total_supply: a.total_supply,
        url: String::from_utf8_lossy(&a.url).into_owned(),
    })
}

pub(super) fn delegated_resource_index_from_proto(
    idx: proto::DelegatedResourceAccountIndex,
) -> Result<DelegatedResourceIndex, TransportError> {
    Ok(DelegatedResourceIndex {
        account: addr(idx.account)?,
        from_accounts: idx
            .from_accounts
            .into_iter()
            .filter_map(|b| addr(b).ok())
            .collect(),
        to_accounts: idx
            .to_accounts
            .into_iter()
            .filter_map(|b| addr(b).ok())
            .collect(),
    })
}
