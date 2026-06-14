//! TRC10 (native TRON token) API extension.
//!
//! Import [`Trc10Api`] to add TRC10 methods to any [`TronProvider`].

use tronz_primitives::Address;

use crate::{
    error::{Error, Result},
    provider::{PendingTransaction, TronProvider},
    transport::TronTransport as _,
    types::{AssetInfo, ContractType, TransactionRequest, TransferAssetContract},
};

/// TRC10 native token methods, available on any [`TronProvider`].
///
/// # Example
///
/// ```no_run
/// use tronz_provider::ext::Trc10Api;
/// # async fn run(provider: impl tronz_provider::TronProvider, recipient: tronz_primitives::Address) -> tronz_provider::Result<()> {
/// // Query token metadata
/// let info = provider.get_asset_info("1000001").await?;
/// println!("{} ({}), decimals={}", info.name, info.abbr, info.decimals);
///
/// // Check a TRC10 balance (reads from get_account)
/// let balance = provider.trc10_balance(recipient, "1000001").await?;
///
/// // Transfer tokens
/// let pending = provider
///     .transfer_trc10()
///     .to(recipient)
///     .token_id("1000001")
///     .amount(1_000_000)
///     .send()
///     .await?;
/// # Ok(()) }
/// ```
pub trait Trc10Api: TronProvider + Sized {
    /// Fetch metadata for a TRC10 token by its numeric ID (e.g. `"1000001"`).
    fn get_asset_info(
        &self,
        token_id: &str,
    ) -> impl std::future::Future<Output = Result<AssetInfo>> + Send;

    /// Return the raw TRC10 balance of `address` for `token_id`.
    ///
    /// Internally calls [`get_account`](TronProvider::get_account) and
    /// extracts the balance from `trc10_balances`. Returns `0` if the account
    /// holds none of the token.
    fn trc10_balance(
        &self,
        address: Address,
        token_id: &str,
    ) -> impl std::future::Future<Output = Result<i64>> + Send;

    /// Fetch all TRC10 tokens issued by `address`.
    fn get_asset_issue_by_account(
        &self,
        address: Address,
    ) -> impl std::future::Future<Output = Result<Vec<AssetInfo>>> + Send;

    /// Fetch a paginated list of all TRC10 tokens on-chain.
    ///
    /// `offset` is the token index to start from (0-based); `limit` is the
    /// maximum number of tokens to return.
    fn get_asset_issue_list(
        &self,
        offset: i64,
        limit: i64,
    ) -> impl std::future::Future<Output = Result<Vec<AssetInfo>>> + Send;

    /// Start building a TRC10 token transfer.
    fn transfer_trc10(&self) -> TransferTrc10Builder<'_, Self>;
}

impl<P: TronProvider> Trc10Api for P {
    async fn get_asset_info(&self, token_id: &str) -> Result<AssetInfo> {
        self.transport()
            .get_asset_issue_by_id(token_id)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_asset_issue_by_account(&self, address: Address) -> Result<Vec<AssetInfo>> {
        self.transport()
            .get_asset_issue_by_account(address)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn get_asset_issue_list(&self, offset: i64, limit: i64) -> Result<Vec<AssetInfo>> {
        self.transport()
            .get_paginated_asset_issue_list(offset, limit)
            .await
            .map_err(|e| Error::Transport(e.into()))
    }

    async fn trc10_balance(&self, address: Address, token_id: &str) -> Result<i64> {
        let account = self.get_account(address).await?;
        Ok(account.trc10_balances.get(token_id).copied().unwrap_or(0))
    }

    fn transfer_trc10(&self) -> TransferTrc10Builder<'_, Self> {
        TransferTrc10Builder::new(self)
    }
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Builds a TRC10 token transfer.
///
/// Created by [`Trc10Api::transfer_trc10`].
pub struct TransferTrc10Builder<'a, P> {
    provider: &'a P,
    owner: Option<Address>,
    to: Option<Address>,
    token_id: Option<String>,
    amount: Option<i64>,
    memo: Option<Vec<u8>>,
}

impl<'a, P: TronProvider> TransferTrc10Builder<'a, P> {
    pub(crate) fn new(provider: &'a P) -> Self {
        Self { provider, owner: None, to: None, token_id: None, amount: None, memo: None }
    }

    /// Override the sender (defaults to the provider's signer address).
    pub fn owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Set the recipient address.
    pub fn to(mut self, to: Address) -> Self {
        self.to = Some(to);
        self
    }

    /// Set the numeric token ID (e.g. `"1000001"`).
    pub fn token_id(mut self, id: impl Into<String>) -> Self {
        self.token_id = Some(id.into());
        self
    }

    /// Set the amount in the token's smallest unit.
    pub fn amount(mut self, amount: i64) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Attach a memo.
    pub fn memo(mut self, memo: impl Into<Vec<u8>>) -> Self {
        self.memo = Some(memo.into());
        self
    }

    /// Build, sign, and broadcast the transfer.
    pub async fn send(self) -> Result<PendingTransaction<P>> {
        let owner = self
            .owner
            .or_else(|| self.provider.signer_address())
            .ok_or(Error::NoSigner)?;
        let to = self.to.ok_or(Error::MissingField("to"))?;
        let token_id = self.token_id.ok_or(Error::MissingField("token_id"))?;
        let amount = self.amount.ok_or(Error::MissingField("amount"))?;

        let req = TransactionRequest {
            contract: Some(ContractType::TransferAsset(TransferAssetContract {
                owner_address: owner,
                to_address: to,
                token_id,
                amount,
            })),
            memo: self.memo,
            ..Default::default()
        };
        self.provider.send_transaction(req).await
    }
}
