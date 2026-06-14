//! tonic-backed gRPC transport targeting the TRON full-node WalletClient API.
//!
//! Default endpoint: `https://grpc.trongrid.io:443` (TronGrid mainnet, TLS).
//! For local/private nodes use `http://127.0.0.1:50051` (no TLS).

mod codec;

use std::collections::HashMap;

use prost::Message as _;
use tonic::transport::{Channel, Endpoint};

use tronz_primitives::{Address, ResourceCode, Trx, TxId};

use crate::error::TransportError;
use crate::proto::wallet_client::WalletClient;
use crate::proto::{self, EmptyMessage};
use crate::transport::TronTransport;
use crate::types::{
    AccountInfo, AccountPermissionUpdateContract, AccountResource, AssetInfo, BlockInfo,
    CancelAllUnfreezeV2Contract, ConstantCallResult, CreateAccountContract, CreateSmartContract,
    DelegateResourceContract, DelegatedResource, DelegatedResourceIndex, FreezeBalanceV2Contract,
    RawTransaction, SignedTransaction, SmartContractInfo, TransactionInfo, TransferAssetContract,
    TransferContract, TriggerSmartContract, UnDelegateResourceContract, UnfreezeBalanceV2Contract,
    UpdateAccountContract, VoteWitnessContract, WitnessInfo, WithdrawBalanceContract,
    WithdrawExpireUnfreezeContract,
};

/// TronGrid mainnet gRPC endpoint (TLS).
pub const TRONGRID_MAINNET: &str = "https://grpc.trongrid.io:443";
/// TronGrid Nile testnet gRPC endpoint (plain HTTP/2, no TLS).
///
/// TronGrid's wildcard TLS cert (`*.trongrid.io`) does not cover the
/// three-level hostname `grpc.nile.trongrid.io`, so connect without TLS:
/// ```no_run
/// use tronz_provider::{ProviderBuilder, transport::grpc::TRONGRID_NILE};
/// # async fn run() -> tronz_provider::Result<()> {
/// let provider = ProviderBuilder::new().on_grpc(TRONGRID_NILE).await?;
/// # Ok(()) }
/// ```
pub const TRONGRID_NILE: &str = "http://grpc.nile.trongrid.io:50051";

/// gRPC transport wrapping a tonic [`Channel`].
///
/// Cheap to clone — the channel is already `Arc`-backed.
#[derive(Clone)]
pub struct GrpcTransport {
    channel: Channel,
    api_key: Option<String>,
}

impl GrpcTransport {
    /// Connect to a TRON gRPC node.
    ///
    /// `uri` may be:
    /// - `"https://grpc.trongrid.io:443"` (TronGrid mainnet, TLS)
    /// - `"http://127.0.0.1:50051"` (local node, plain HTTP/2)
    pub async fn connect(uri: impl AsRef<str>) -> Result<Self, TransportError> {
        let endpoint = Endpoint::from_shared(uri.as_ref().to_owned())
            .map_err(|e| TransportError::Malformed(e.to_string()))?;

        #[cfg(feature = "grpc-tls")]
        let endpoint = endpoint
            .tls_config(tonic::transport::ClientTlsConfig::new().with_native_roots())
            .map_err(TransportError::Connect)?;

        let channel = endpoint.connect().await?;
        Ok(Self { channel, api_key: None })
    }

    /// Attach a TronGrid API key (sent as `TRON-PRO-API-KEY` header on each call).
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    fn wallet_client(&self) -> WalletClient<Channel> {
        WalletClient::new(self.channel.clone())
    }


    /// Check a `Return` message, converting failures to [`TransportError::NodeError`].
    fn check_return(ret: Option<proto::Return>) -> Result<(), TransportError> {
        if let Some(r) = ret {
            if !r.result {
                let msg = String::from_utf8_lossy(&r.message).into_owned();
                return Err(TransportError::NodeError(msg));
            }
        }
        Ok(())
    }

    /// Extract a [`RawTransaction`] from a [`proto::TransactionExtention`].
    fn raw_from_extention(
        ext: proto::TransactionExtention,
    ) -> Result<RawTransaction, TransportError> {
        Self::check_return(ext.result)?;

        let tx = ext
            .transaction
            .ok_or_else(|| TransportError::Malformed("missing transaction in extention".into()))?;

        let (expiration, timestamp) = tx
            .raw_data
            .as_ref()
            .map(|r| (r.expiration, r.timestamp))
            .unwrap_or((0, 0));

        let raw_proto = tx.encode_to_vec();
        RawTransaction::from_proto_extention(ext.txid, raw_proto, expiration, timestamp)
    }
}

impl TronTransport for GrpcTransport {
    type Error = TransportError;

    // --- Block ---

    async fn get_now_block(&self) -> Result<BlockInfo, Self::Error> {
        let ext = self
            .wallet_client()
            .get_now_block2(EmptyMessage::default())
            .await?
            .into_inner();
        codec::block_from_extention(ext)
    }

    async fn get_block_by_number(&self, num: i64) -> Result<BlockInfo, Self::Error> {
        let ext = self
            .wallet_client()
            .get_block_by_num2(proto::NumberMessage { num })
            .await?
            .into_inner();
        codec::block_from_extention(ext)
    }

    // --- Account ---

    async fn get_account(&self, address: Address) -> Result<AccountInfo, Self::Error> {
        let req = proto::Account {
            address: address.as_bytes().to_vec(),
            ..Default::default()
        };
        let account = self.wallet_client().get_account(req).await?.into_inner();
        codec::account_from_proto(account, address)
    }

    async fn get_account_resource(&self, address: Address) -> Result<AccountResource, Self::Error> {
        let req = proto::Account {
            address: address.as_bytes().to_vec(),
            ..Default::default()
        };
        let res = self.wallet_client().get_account_resource(req).await?.into_inner();
        Ok(codec::account_resource_from_proto(res))
    }

    // --- Transaction ---

    async fn broadcast_transaction(&self, tx: &SignedTransaction) -> Result<(), Self::Error> {
        use proto::Transaction;

        let mut proto_tx = Transaction::decode(tx.raw.raw_proto.as_ref())?;
        for sig in &tx.signatures {
            proto_tx.signature.push(sig.to_bytes().to_vec());
        }

        let ret = self
            .wallet_client()
            .broadcast_transaction(proto_tx)
            .await?
            .into_inner();
        Self::check_return(Some(ret))
    }

    async fn get_transaction_by_id(&self, tx_id: TxId) -> Result<SignedTransaction, Self::Error> {
        let req = proto::BytesMessage { value: tx_id.as_slice().to_vec() };
        let tx = self.wallet_client().get_transaction_by_id(req).await?.into_inner();
        codec::signed_tx_from_proto(tx)
    }

    async fn get_transaction_info(&self, tx_id: TxId) -> Result<TransactionInfo, Self::Error> {
        let req = proto::BytesMessage { value: tx_id.as_slice().to_vec() };
        let info = self
            .wallet_client()
            .get_transaction_info_by_id(req)
            .await?
            .into_inner();
        codec::transaction_info_from_proto(info)
    }

    // --- Native contracts ---

    async fn transfer_trx(&self, params: TransferContract) -> Result<RawTransaction, Self::Error> {
        let req = codec::transfer_to_proto(params);
        let ext = self.wallet_client().create_transaction2(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn account_permission_update(
        &self,
        params: AccountPermissionUpdateContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = codec::account_permission_update_to_proto(params);
        let ext = self.wallet_client().account_permission_update(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn create_smart_contract(
        &self,
        params: CreateSmartContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = codec::create_smart_contract_to_proto(params);
        let ext = self.wallet_client().deploy_contract(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    // --- Smart contracts ---

    async fn trigger_smart_contract(
        &self,
        params: TriggerSmartContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = codec::trigger_smart_contract_to_proto(params);
        let ext = self.wallet_client().trigger_contract(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn trigger_constant_contract(
        &self,
        params: TriggerSmartContract,
    ) -> Result<ConstantCallResult, Self::Error> {
        let req = codec::trigger_smart_contract_to_proto(params);
        let ext = self.wallet_client().trigger_constant_contract(req).await?.into_inner();
        codec::constant_result_from_extention(ext)
    }

    async fn estimate_energy(&self, params: TriggerSmartContract) -> Result<i64, Self::Error> {
        let req = codec::trigger_smart_contract_to_proto(params);
        let msg = self.wallet_client().estimate_energy(req).await?.into_inner();
        Self::check_return(msg.result)?;
        Ok(msg.energy_required)
    }

    // --- Staking ---

    async fn freeze_balance_v2(
        &self,
        params: FreezeBalanceV2Contract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = proto::FreezeBalanceV2Contract {
            owner_address: params.owner_address.as_bytes().to_vec(),
            frozen_balance: params.frozen_balance.as_sun(),
            resource: params.resource.as_i32(),
        };
        let ext = self.wallet_client().freeze_balance_v2(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn unfreeze_balance_v2(
        &self,
        params: UnfreezeBalanceV2Contract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = proto::UnfreezeBalanceV2Contract {
            owner_address: params.owner_address.as_bytes().to_vec(),
            unfreeze_balance: params.unfreeze_balance.as_sun(),
            resource: params.resource.as_i32(),
        };
        let ext = self.wallet_client().unfreeze_balance_v2(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn delegate_resource(
        &self,
        params: DelegateResourceContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = proto::DelegateResourceContract {
            owner_address: params.owner_address.as_bytes().to_vec(),
            resource: params.resource.as_i32(),
            balance: params.balance.as_sun(),
            receiver_address: params.receiver_address.as_bytes().to_vec(),
            lock: params.lock_period.is_some(),
            lock_period: params.lock_period.unwrap_or(0),
        };
        let ext = self.wallet_client().delegate_resource(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn undelegate_resource(
        &self,
        params: UnDelegateResourceContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = proto::UnDelegateResourceContract {
            owner_address: params.owner_address.as_bytes().to_vec(),
            resource: params.resource.as_i32(),
            balance: params.balance.as_sun(),
            receiver_address: params.receiver_address.as_bytes().to_vec(),
        };
        let ext = self.wallet_client().un_delegate_resource(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn withdraw_expire_unfreeze(
        &self,
        params: WithdrawExpireUnfreezeContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = proto::WithdrawExpireUnfreezeContract {
            owner_address: params.owner_address.as_bytes().to_vec(),
        };
        let ext = self.wallet_client().withdraw_expire_unfreeze(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn cancel_all_unfreeze_v2(
        &self,
        params: CancelAllUnfreezeV2Contract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = proto::CancelAllUnfreezeV2Contract {
            owner_address: params.owner_address.as_bytes().to_vec(),
        };
        let ext = self.wallet_client().cancel_all_unfreeze_v2(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn withdraw_balance(
        &self,
        params: WithdrawBalanceContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = proto::WithdrawBalanceContract {
            owner_address: params.owner_address.as_bytes().to_vec(),
        };
        let ext = self.wallet_client().withdraw_balance2(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    // --- Resource queries ---

    async fn get_delegated_resource(
        &self,
        from: Address,
        to: Address,
    ) -> Result<Vec<DelegatedResource>, Self::Error> {
        let req = proto::DelegatedResourceMessage {
            from_address: from.as_bytes().to_vec(),
            to_address: to.as_bytes().to_vec(),
        };
        let list = self.wallet_client().get_delegated_resource_v2(req).await?.into_inner();
        list.delegated_resource
            .into_iter()
            .map(codec::delegated_resource_from_proto)
            .collect()
    }

    async fn get_delegated_resource_index(
        &self,
        address: Address,
    ) -> Result<DelegatedResourceIndex, Self::Error> {
        let req = proto::BytesMessage { value: address.as_bytes().to_vec() };
        let idx = self
            .wallet_client()
            .get_delegated_resource_account_index_v2(req)
            .await?
            .into_inner();
        codec::delegated_resource_index_from_proto(idx)
    }

    async fn get_can_delegate_max(
        &self,
        address: Address,
        resource: ResourceCode,
    ) -> Result<Trx, Self::Error> {
        let req = proto::CanDelegatedMaxSizeRequestMessage {
            owner_address: address.as_bytes().to_vec(),
            r#type: resource.as_i32(),
        };
        let res = self.wallet_client().get_can_delegated_max_size(req).await?.into_inner();
        Ok(Trx::from_sun_unchecked(res.max_size))
    }

    async fn get_reward(&self, address: Address) -> Result<Trx, Self::Error> {
        let req = proto::BytesMessage { value: address.as_bytes().to_vec() };
        let res = self.wallet_client().get_reward_info(req).await?.into_inner();
        Ok(Trx::from_sun_unchecked(res.num))
    }

    // --- Network ---

    async fn get_chain_parameters(&self) -> Result<HashMap<String, i64>, Self::Error> {
        let params = self
            .wallet_client()
            .get_chain_parameters(EmptyMessage::default())
            .await?
            .into_inner();
        Ok(params
            .chain_parameter
            .into_iter()
            .map(|p| (p.key, p.value))
            .collect())
    }

    async fn get_contract(&self, address: Address) -> Result<SmartContractInfo, Self::Error> {
        let req = proto::BytesMessage { value: address.as_bytes().to_vec() };
        let contract = self.wallet_client().get_contract(req).await?.into_inner();
        Ok(codec::smart_contract_from_proto(contract))
    }

    async fn get_contract_info(
        &self,
        address: Address,
    ) -> Result<SmartContractInfo, Self::Error> {
        let req = proto::BytesMessage { value: address.as_bytes().to_vec() };
        let wrapper = self.wallet_client().get_contract_info(req).await?.into_inner();
        Ok(codec::smart_contract_info_from_wrapper(wrapper))
    }

    async fn list_witnesses(&self) -> Result<Vec<WitnessInfo>, Self::Error> {
        let list = self
            .wallet_client()
            .list_witnesses(proto::EmptyMessage::default())
            .await?
            .into_inner();
        Ok(list.witnesses.into_iter().filter_map(codec::witness_from_proto).collect())
    }


    // --- TRC10 ---

    async fn transfer_asset(
        &self,
        params: TransferAssetContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = codec::transfer_asset_to_proto(params);
        let ext = self.wallet_client().transfer_asset2(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn get_asset_issue_by_id(&self, token_id: &str) -> Result<AssetInfo, Self::Error> {
        let req = proto::BytesMessage { value: token_id.as_bytes().to_vec() };
        let asset = self.wallet_client().get_asset_issue_by_id(req).await?.into_inner();
        codec::asset_info_from_proto(asset)
    }

    async fn get_asset_issue_by_account(
        &self,
        address: Address,
    ) -> Result<Vec<AssetInfo>, Self::Error> {
        let req = proto::Account {
            address: address.as_bytes().to_vec(),
            ..Default::default()
        };
        let list = self.wallet_client().get_asset_issue_by_account(req).await?.into_inner();
        list.asset_issue.into_iter().map(codec::asset_info_from_proto).collect()
    }

    async fn get_paginated_asset_issue_list(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<AssetInfo>, Self::Error> {
        let req = proto::PaginatedMessage { offset, limit };
        let list =
            self.wallet_client().get_paginated_asset_issue_list(req).await?.into_inner();
        list.asset_issue.into_iter().map(codec::asset_info_from_proto).collect()
    }

    async fn create_account(
        &self,
        params: CreateAccountContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = codec::create_account_to_proto(params);
        let ext = self.wallet_client().create_account2(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn vote_witness_account(
        &self,
        params: VoteWitnessContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = codec::vote_witness_to_proto(params);
        let ext = self.wallet_client().vote_witness_account2(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn update_account(
        &self,
        params: UpdateAccountContract,
    ) -> Result<RawTransaction, Self::Error> {
        let req = codec::update_account_to_proto(params);
        let ext = self.wallet_client().update_account2(req).await?.into_inner();
        Self::raw_from_extention(ext)
    }

    async fn get_can_withdraw_unfreeze_amount(
        &self,
        address: Address,
        timestamp_ms: i64,
    ) -> Result<Trx, Self::Error> {
        let req = proto::CanWithdrawUnfreezeAmountRequestMessage {
            owner_address: address.as_bytes().to_vec(),
            timestamp: timestamp_ms,
        };
        let res = self
            .wallet_client()
            .get_can_withdraw_unfreeze_amount(req)
            .await?
            .into_inner();
        Ok(Trx::from_sun_unchecked(res.amount))
    }

    async fn get_available_unfreeze_count(
        &self,
        address: Address,
    ) -> Result<i64, Self::Error> {
        let req = proto::GetAvailableUnfreezeCountRequestMessage {
            owner_address: address.as_bytes().to_vec(),
        };
        let res = self
            .wallet_client()
            .get_available_unfreeze_count(req)
            .await?
            .into_inner();
        Ok(res.count)
    }
}
