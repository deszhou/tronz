//! The high-level [`TronProvider`] trait and its concrete implementations.

pub mod builder;
pub mod pending;
pub mod root;

use core::future::Future;
use std::collections::HashMap;

pub use builder::{FilledProvider, ProviderBuilder};
pub use pending::PendingTransaction;
pub use root::RootProvider;
use tronz_primitives::{Address, B256, ResourceCode, Trx, TxId};

use crate::{
    builders::{
        AccountPermissionUpdateBuilder, CancelAllUnfreezeBuilder, ClearContractAbiBuilder,
        CreateAccountBuilder, DelegateBuilder, FreezeBuilder, FreezeV1Builder, SetAccountIdBuilder,
        TransferBuilder, UndelegateBuilder, UnfreezeBuilder, UnfreezeV1Builder,
        UpdateAccountBuilder, UpdateContractEnergyLimitBuilder, UpdateContractSettingBuilder,
        VoteBuilder, WithdrawBalanceBuilder, WithdrawExpireBuilder,
    },
    error::Result,
    transport::TronTransport,
    types::{
        AccountInfo, AccountNet, AccountResource, BlockInfo, ChainProperties, DelegatedResource,
        DelegatedResourceIndex, NodeAddress, NodeInfo, RawTransaction, SignWeight,
        SignedTransaction, SmartContractInfo, TransactionInfo, TransactionRequest,
        TriggerSmartContract, WitnessInfo,
    },
};

/// The primary user-facing interface: reads, lazy operation builders, and
/// low-level send/broadcast.
pub trait TronProvider: Clone + Send + Sync + 'static {
    /// The underlying transport type.
    type Transport: TronTransport;

    /// Borrow the transport.
    fn transport(&self) -> &Self::Transport;

    /// The attached signer's address, if any.
    fn signer_address(&self) -> Option<Address>;

    // ---------- Reads ----------

    /// Fetch the latest block.
    fn get_now_block(&self) -> impl Future<Output = Result<BlockInfo>> + Send;

    /// Fetch on-chain account state.
    fn get_account(&self, address: Address) -> impl Future<Output = Result<AccountInfo>> + Send;

    /// Fetch account resource usage.
    fn get_account_resource(
        &self,
        address: Address,
    ) -> impl Future<Output = Result<AccountResource>> + Send;

    /// Fetch a transaction by id.
    fn get_transaction(
        &self,
        tx_id: TxId,
    ) -> impl Future<Output = Result<SignedTransaction>> + Send;

    /// Fetch a transaction's receipt/info.
    fn get_transaction_info(
        &self,
        tx_id: TxId,
    ) -> impl Future<Output = Result<TransactionInfo>> + Send;

    /// Query delegations between two accounts (Stake 1.0, legacy).
    fn get_delegated_resource_v1(
        &self,
        from: Address,
        to: Address,
    ) -> impl Future<Output = Result<Vec<DelegatedResource>>> + Send;

    /// Query the delegation index for an account (Stake 1.0, legacy).
    fn get_delegated_resource_index_v1(
        &self,
        address: Address,
    ) -> impl Future<Output = Result<DelegatedResourceIndex>> + Send;

    /// Query delegations between two accounts (Stake 2.0).
    fn get_delegated_resource(
        &self,
        from: Address,
        to: Address,
    ) -> impl Future<Output = Result<Vec<DelegatedResource>>> + Send;

    /// Query the delegation index for an account (Stake 2.0).
    fn get_delegated_resource_index(
        &self,
        address: Address,
    ) -> impl Future<Output = Result<DelegatedResourceIndex>> + Send;

    /// Query the max amount still delegatable for a resource.
    fn get_can_delegate_max(
        &self,
        address: Address,
        resource: ResourceCode,
    ) -> impl Future<Output = Result<Trx>> + Send;

    /// Query the pending (unclaimed) reward.
    fn get_reward(&self, address: Address) -> impl Future<Output = Result<Trx>> + Send;

    /// Fetch chain parameters.
    fn chain_parameters(&self) -> impl Future<Output = Result<HashMap<String, i64>>> + Send;

    /// Fetch contract metadata including the deployed runtime bytecode.
    fn get_contract_info(
        &self,
        address: Address,
    ) -> impl Future<Output = Result<SmartContractInfo>> + Send;

    /// List all super representatives and candidates.
    fn list_witnesses(&self) -> impl Future<Output = Result<Vec<WitnessInfo>>> + Send;

    // ---------- New pure query methods ----------

    /// Fetch the bandwidth price schedule string.
    fn get_bandwidth_prices(&self) -> impl Future<Output = Result<String>> + Send;

    /// Fetch the energy price schedule string.
    fn get_energy_prices(&self) -> impl Future<Output = Result<String>> + Send;

    /// Fetch the memo fee schedule.
    fn get_memo_fee(&self) -> impl Future<Output = Result<u64>> + Send;

    /// Fetch the next maintenance time (unix ms).
    fn get_next_maintenance_time(&self) -> impl Future<Output = Result<i64>> + Send;

    /// Fetch the total amount of TRX burned.
    fn get_burn_trx(&self) -> impl Future<Output = Result<u64>> + Send;

    /// Fetch the total number of transactions ever processed.
    fn get_total_transactions(&self) -> impl Future<Output = Result<u64>> + Send;

    /// Fetch basic info about the connected node.
    fn get_node_info(&self) -> impl Future<Output = Result<NodeInfo>> + Send;

    /// List known gossip-network peer addresses.
    fn list_nodes(&self) -> impl Future<Output = Result<Vec<NodeAddress>>> + Send;

    /// Fetch dynamic chain properties.
    fn get_dynamic_properties(&self) -> impl Future<Output = Result<ChainProperties>> + Send;

    /// Fetch a block by its hash.
    fn get_block_by_id(&self, block_id: B256) -> impl Future<Output = Result<BlockInfo>> + Send;

    /// Fetch the `count` most recent blocks.
    fn get_blocks_by_latest_num(
        &self,
        count: i64,
    ) -> impl Future<Output = Result<Vec<BlockInfo>>> + Send;

    /// Fetch blocks in the range `[start, end)`.
    fn get_blocks_by_limit(
        &self,
        start: i64,
        end: i64,
    ) -> impl Future<Output = Result<Vec<BlockInfo>>> + Send;

    /// Count transactions in a block by block number.
    fn get_transaction_count_by_block_num(
        &self,
        block_num: i64,
    ) -> impl Future<Output = Result<u64>> + Send;

    /// Fetch paginated transactions sent *from* an address.
    fn get_transactions_from(
        &self,
        address: Address,
        offset: i64,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<RawTransaction>>> + Send;

    /// Fetch paginated transactions sent *to* an address.
    fn get_transactions_to(
        &self,
        address: Address,
        offset: i64,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<RawTransaction>>> + Send;

    /// Fetch transaction infos for all transactions in a block.
    fn get_transaction_info_by_block_num(
        &self,
        block_num: i64,
    ) -> impl Future<Output = Result<Vec<TransactionInfo>>> + Send;

    /// Fetch the number of pending transactions.
    fn get_pending_size(&self) -> impl Future<Output = Result<u64>> + Send;

    /// Fetch a single pending transaction by id.
    fn get_transaction_from_pending(
        &self,
        tx_id: TxId,
    ) -> impl Future<Output = Result<RawTransaction>> + Send;

    /// Fetch all pending transactions.
    fn get_pending_transactions(&self) -> impl Future<Output = Result<Vec<RawTransaction>>> + Send;

    /// Query sign-weight for a transaction.
    fn get_transaction_sign_weight(
        &self,
        tx: &RawTransaction,
    ) -> impl Future<Output = Result<SignWeight>> + Send;

    /// Fetch addresses that have already signed a transaction.
    fn get_transaction_approved_list(
        &self,
        tx: &RawTransaction,
    ) -> impl Future<Output = Result<Vec<Address>>> + Send;

    /// Fetch bandwidth/energy net-usage for an account.
    fn get_account_net(&self, address: Address) -> impl Future<Output = Result<AccountNet>> + Send;

    /// Fetch the brokerage ratio for a super representative.
    fn get_brokerage(&self, address: Address) -> impl Future<Output = Result<u64>> + Send;

    /// Fetch the unclaimed reward (raw sun) for an address.
    fn get_reward_info(&self, address: Address) -> impl Future<Output = Result<u64>> + Send;

    // ---------- Transaction builders (lazy — no I/O until `.send()`) ----------

    /// Build a TRX transfer.
    fn send_trx(&self) -> TransferBuilder<'_, Self>
    where
        Self: Sized,
    {
        TransferBuilder::new(self)
    }

    /// Build a stake (freeze) operation (Stake 1.0, legacy).
    fn freeze_balance_v1(&self) -> FreezeV1Builder<'_, Self>
    where
        Self: Sized,
    {
        FreezeV1Builder::new(self)
    }

    /// Build an unstake (unfreeze) operation (Stake 1.0, legacy).
    fn unfreeze_balance_v1(&self) -> UnfreezeV1Builder<'_, Self>
    where
        Self: Sized,
    {
        UnfreezeV1Builder::new(self)
    }

    /// Build a stake (freeze) operation (Stake 2.0).
    fn freeze_balance(&self) -> FreezeBuilder<'_, Self>
    where
        Self: Sized,
    {
        FreezeBuilder::new(self)
    }

    /// Build an unstake (unfreeze) operation (Stake 2.0).
    fn unfreeze_balance(&self) -> UnfreezeBuilder<'_, Self>
    where
        Self: Sized,
    {
        UnfreezeBuilder::new(self)
    }

    /// Build a delegate-resource operation.
    fn delegate_resource(&self) -> DelegateBuilder<'_, Self>
    where
        Self: Sized,
    {
        DelegateBuilder::new(self)
    }

    /// Build an undelegate-resource operation.
    fn undelegate_resource(&self) -> UndelegateBuilder<'_, Self>
    where
        Self: Sized,
    {
        UndelegateBuilder::new(self)
    }

    /// Build a withdraw-expire-unfreeze operation.
    fn withdraw_expire_unfreeze(&self) -> WithdrawExpireBuilder<'_, Self>
    where
        Self: Sized,
    {
        WithdrawExpireBuilder::new(self)
    }

    /// Build a cancel-all-unfreeze operation.
    fn cancel_all_unfreeze(&self) -> CancelAllUnfreezeBuilder<'_, Self>
    where
        Self: Sized,
    {
        CancelAllUnfreezeBuilder::new(self)
    }

    /// Build a claim-rewards operation.
    fn claim_rewards(&self) -> WithdrawBalanceBuilder<'_, Self>
    where
        Self: Sized,
    {
        WithdrawBalanceBuilder::new(self)
    }

    /// Update account permissions (multisig).
    fn update_permissions(&self) -> AccountPermissionUpdateBuilder<'_, Self>
    where
        Self: Sized,
    {
        AccountPermissionUpdateBuilder::new(self)
    }

    // ---------- Smart contracts ----------

    /// Query how much TRX can be withdrawn from expired unfreeze windows.
    ///
    /// `timestamp_ms` is the reference time (unix milliseconds).
    /// Pass the current time to check what is withdrawable right now.
    fn get_can_withdraw_unfreeze_amount(
        &self,
        address: Address,
        timestamp_ms: i64,
    ) -> impl Future<Output = Result<Trx>> + Send;

    /// Query how many more unfreeze operations the account can still initiate.
    ///
    /// TRON allows at most 32 concurrent unfreeze windows per account.
    fn get_available_unfreeze_count(
        &self,
        address: Address,
    ) -> impl Future<Output = Result<i64>> + Send;

    /// Activate a new account on-chain.
    fn create_account(&self) -> CreateAccountBuilder<'_, Self>
    where
        Self: Sized,
    {
        CreateAccountBuilder::new(self)
    }

    /// Vote for super representatives.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use tronz_provider::TronProvider as _;
    /// # async fn run(provider: impl tronz_provider::TronProvider, sr: tronz_primitives::Address) -> tronz_provider::Result<()> {
    /// let pending = provider.vote_witness().vote(sr, 100).send().await?;
    /// # Ok(()) }
    /// ```
    fn vote_witness(&self) -> VoteBuilder<'_, Self>
    where
        Self: Sized,
    {
        VoteBuilder::new(self)
    }

    /// Update the account's on-chain name.
    fn update_account_name(&self) -> UpdateAccountBuilder<'_, Self>
    where
        Self: Sized,
    {
        UpdateAccountBuilder::new(self)
    }

    /// Set a short alphanumeric on-chain account ID (alias).
    ///
    /// Can only be done once per account. The ID must be unique network-wide.
    fn set_account_id(&self) -> SetAccountIdBuilder<'_, Self>
    where
        Self: Sized,
    {
        SetAccountIdBuilder::new(self)
    }

    /// Clear the ABI of a deployed smart contract.
    ///
    /// Only the contract owner can call this.
    fn clear_contract_abi(&self) -> ClearContractAbiBuilder<'_, Self>
    where
        Self: Sized,
    {
        ClearContractAbiBuilder::new(self)
    }

    /// Update the caller-energy-percentage setting on a smart contract.
    ///
    /// Only the contract owner can call this.
    fn update_contract_setting(&self) -> UpdateContractSettingBuilder<'_, Self>
    where
        Self: Sized,
    {
        UpdateContractSettingBuilder::new(self)
    }

    /// Update the per-call origin energy limit on a smart contract.
    ///
    /// Only the contract owner can call this.
    fn update_contract_energy_limit(&self) -> UpdateContractEnergyLimitBuilder<'_, Self>
    where
        Self: Sized,
    {
        UpdateContractEnergyLimitBuilder::new(self)
    }

    /// Estimate the energy a contract call would consume.
    ///
    /// Mirrors [`estimate_gas`] in alloy: no state change, no signer required.
    /// Use this before [`send_transaction`] to set an appropriate `fee_limit`.
    ///
    /// [`estimate_gas`]: https://alloy.rs
    /// [`send_transaction`]: TronProvider::send_transaction
    fn estimate_energy(
        &self,
        params: TriggerSmartContract,
    ) -> impl Future<Output = Result<i64>> + Send;

    // ---------- Low-level ----------

    /// Fill, sign, and broadcast a pre-built request.
    fn send_transaction(
        &self,
        req: TransactionRequest,
    ) -> impl Future<Output = Result<PendingTransaction<Self>>> + Send
    where
        Self: Sized;

    /// Broadcast an already-signed transaction.
    fn broadcast(
        &self,
        tx: SignedTransaction,
    ) -> impl Future<Output = Result<PendingTransaction<Self>>> + Send
    where
        Self: Sized;
}
