//! The base [`RootProvider`] over a transport.

use std::collections::HashMap;
use std::sync::Arc;

use tronz_primitives::{Address, ResourceCode, Trx, TxId};

use crate::error::{Error, Result};
use crate::provider::{PendingTransaction, TronProvider};
use crate::transport::TronTransport;
use crate::types::{
    AccountInfo, AccountResource, BlockInfo, DelegatedResource, DelegatedResourceIndex,
    SignedTransaction, SmartContractInfo, TransactionInfo, TransactionRequest, TriggerSmartContract,
    WitnessInfo,
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
        self.inner.transport.get_now_block().await.map_err(|e| Error::Transport(e.into()))
    }

    async fn get_account(&self, address: Address) -> Result<AccountInfo> {
        self.inner.transport.get_account(address).await.map_err(|e| Error::Transport(e.into()))
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

    async fn get_can_delegate_max(
        &self,
        address: Address,
        resource: ResourceCode,
    ) -> Result<Trx> {
        self.inner
            .transport
            .get_can_delegate_max(address, resource)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_reward(&self, address: Address) -> Result<Trx> {
        self.inner.transport.get_reward(address).await.map_err(|e| Error::Transport(e.into()))
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

    async fn send_transaction(
        &self,
        _req: TransactionRequest,
    ) -> Result<PendingTransaction<Self>> {
        Err(Error::NoSigner)
    }

    async fn broadcast(&self, tx: SignedTransaction) -> Result<PendingTransaction<Self>> {
        let tx_id = tx.raw.tx_id();
        self.inner.transport.broadcast_transaction(&tx).await.map_err(|e| Error::Transport(e.into()))?;
        Ok(PendingTransaction::new(self.clone(), tx_id))
    }
}
