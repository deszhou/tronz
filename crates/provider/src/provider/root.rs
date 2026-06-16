//! The base [`RootProvider`] over a transport.

use std::{collections::HashMap, sync::Arc};

use tronz_primitives::{Address, B256, ResourceCode, Trx, TxId};

use crate::{
    error::{Error, Result},
    provider::{PendingTransaction, TronProvider},
    transport::TronTransport,
    types::{
        AccountInfo, AccountNet, AccountResource, BlockInfo, ChainProperties, DelegatedResource,
        DelegatedResourceIndex, NodeAddress, NodeInfo, RawTransaction, SignWeight,
        SignedTransaction, SmartContractInfo, TransactionInfo, TransactionRequest,
        TriggerSmartContract, WitnessInfo,
    },
};

/// The base provider: wraps a transport (and optional signer address) in an
/// `Arc` so it is cheap to clone and `Send + Sync`.
#[derive(Clone)]
pub struct RootProvider<T: TronTransport> {
    inner: Arc<RootProviderInner<T>>,
}

struct RootProviderInner<T> {
    transport: T,
    signer_address: Option<Address>,
}

impl<T: TronTransport> RootProvider<T> {
    /// Create a read-only provider.
    pub fn new(transport: T) -> Self {
        Self {
            inner: Arc::new(RootProviderInner {
                transport,
                signer_address: None,
            }),
        }
    }

    /// Create a provider that knows its signer's address.
    pub fn new_with_signer(transport: T, signer_address: Address) -> Self {
        Self {
            inner: Arc::new(RootProviderInner {
                transport,
                signer_address: Some(signer_address),
            }),
        }
    }

    /// Borrow the transport.
    pub fn transport(&self) -> &T {
        &self.inner.transport
    }

    /// The signer address, if known.
    pub fn signer_address(&self) -> Option<Address> {
        self.inner.signer_address
    }
}

impl<T: TronTransport> TronProvider for RootProvider<T> {
    type Transport = T;

    fn transport(&self) -> &T {
        RootProvider::transport(self)
    }

    fn signer_address(&self) -> Option<Address> {
        RootProvider::signer_address(self)
    }

    async fn get_now_block(&self) -> Result<BlockInfo> {
        self.inner
            .transport
            .get_now_block()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_account(&self, address: Address) -> Result<AccountInfo> {
        self.inner
            .transport
            .get_account(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_account_resource(&self, address: Address) -> Result<AccountResource> {
        self.inner
            .transport
            .get_account_resource(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_transaction(&self, tx_id: TxId) -> Result<SignedTransaction> {
        self.inner
            .transport
            .get_transaction_by_id(tx_id)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_transaction_info(&self, tx_id: TxId) -> Result<TransactionInfo> {
        self.inner
            .transport
            .get_transaction_info(tx_id)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_delegated_resource_v1(
        &self,
        from: Address,
        to: Address,
    ) -> Result<Vec<DelegatedResource>> {
        self.inner
            .transport
            .get_delegated_resource_v1(from, to)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_delegated_resource_index_v1(
        &self,
        address: Address,
    ) -> Result<DelegatedResourceIndex> {
        self.inner
            .transport
            .get_delegated_resource_index_v1(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_delegated_resource(
        &self,
        from: Address,
        to: Address,
    ) -> Result<Vec<DelegatedResource>> {
        self.inner
            .transport
            .get_delegated_resource(from, to)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_delegated_resource_index(
        &self,
        address: Address,
    ) -> Result<DelegatedResourceIndex> {
        self.inner
            .transport
            .get_delegated_resource_index(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_can_delegate_max(&self, address: Address, resource: ResourceCode) -> Result<Trx> {
        self.inner
            .transport
            .get_can_delegate_max(address, resource)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_reward(&self, address: Address) -> Result<Trx> {
        self.inner
            .transport
            .get_reward(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn chain_parameters(&self) -> Result<HashMap<String, i64>> {
        self.inner
            .transport
            .get_chain_parameters()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_contract_info(&self, address: Address) -> Result<SmartContractInfo> {
        self.inner
            .transport
            .get_contract_info(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn list_witnesses(&self) -> Result<Vec<WitnessInfo>> {
        self.inner
            .transport
            .list_witnesses()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_bandwidth_prices(&self) -> Result<String> {
        self.inner
            .transport
            .get_bandwidth_prices()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_energy_prices(&self) -> Result<String> {
        self.inner
            .transport
            .get_energy_prices()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_memo_fee(&self) -> Result<u64> {
        self.inner
            .transport
            .get_memo_fee()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_next_maintenance_time(&self) -> Result<i64> {
        self.inner
            .transport
            .get_next_maintenance_time()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_burn_trx(&self) -> Result<u64> {
        self.inner
            .transport
            .get_burn_trx()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_total_transactions(&self) -> Result<u64> {
        self.inner
            .transport
            .get_total_transactions()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_node_info(&self) -> Result<NodeInfo> {
        self.inner
            .transport
            .get_node_info()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn list_nodes(&self) -> Result<Vec<NodeAddress>> {
        self.inner
            .transport
            .list_nodes()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_dynamic_properties(&self) -> Result<ChainProperties> {
        self.inner
            .transport
            .get_dynamic_properties()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_block_by_id(&self, block_id: B256) -> Result<BlockInfo> {
        self.inner
            .transport
            .get_block_by_id(block_id)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_blocks_by_latest_num(&self, count: i64) -> Result<Vec<BlockInfo>> {
        self.inner
            .transport
            .get_blocks_by_latest_num(count)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_blocks_by_limit(&self, start: i64, end: i64) -> Result<Vec<BlockInfo>> {
        self.inner
            .transport
            .get_blocks_by_limit(start, end)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_transaction_count_by_block_num(&self, block_num: i64) -> Result<u64> {
        self.inner
            .transport
            .get_transaction_count_by_block_num(block_num)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_transactions_from(
        &self,
        address: Address,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<RawTransaction>> {
        self.inner
            .transport
            .get_transactions_from(address, offset, limit)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_transactions_to(
        &self,
        address: Address,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<RawTransaction>> {
        self.inner
            .transport
            .get_transactions_to(address, offset, limit)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_transaction_info_by_block_num(
        &self,
        block_num: i64,
    ) -> Result<Vec<TransactionInfo>> {
        self.inner
            .transport
            .get_transaction_info_by_block_num(block_num)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_pending_size(&self) -> Result<u64> {
        self.inner
            .transport
            .get_pending_size()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_transaction_from_pending(&self, tx_id: TxId) -> Result<RawTransaction> {
        self.inner
            .transport
            .get_transaction_from_pending(tx_id)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_pending_transactions(&self) -> Result<Vec<RawTransaction>> {
        self.inner
            .transport
            .get_pending_transactions()
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_transaction_sign_weight(&self, tx: &RawTransaction) -> Result<SignWeight> {
        self.inner
            .transport
            .get_transaction_sign_weight(tx)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_transaction_approved_list(&self, tx: &RawTransaction) -> Result<Vec<Address>> {
        self.inner
            .transport
            .get_transaction_approved_list(tx)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_account_net(&self, address: Address) -> Result<AccountNet> {
        self.inner
            .transport
            .get_account_net(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_brokerage(&self, address: Address) -> Result<u64> {
        self.inner
            .transport
            .get_brokerage(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_reward_info(&self, address: Address) -> Result<u64> {
        self.inner
            .transport
            .get_reward_info(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_can_withdraw_unfreeze_amount(
        &self,
        address: Address,
        timestamp_ms: i64,
    ) -> Result<Trx> {
        self.inner
            .transport
            .get_can_withdraw_unfreeze_amount(address, timestamp_ms)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_available_unfreeze_count(&self, address: Address) -> Result<i64> {
        self.inner
            .transport
            .get_available_unfreeze_count(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn estimate_energy(&self, params: TriggerSmartContract) -> Result<i64> {
        self.inner
            .transport
            .estimate_energy(params)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn send_transaction(&self, _req: TransactionRequest) -> Result<PendingTransaction<Self>> {
        Err(Error::NoSigner)
    }

    async fn broadcast(&self, tx: SignedTransaction) -> Result<PendingTransaction<Self>> {
        let tx_id = tx.raw.tx_id();
        self.inner
            .transport
            .broadcast_transaction(&tx)
            .await
            .map_err(|e| Error::Transport(e.into()))?;
        Ok(PendingTransaction::new(self.clone(), tx_id))
    }
}
