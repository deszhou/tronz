//! Stake (freeze) and unstake (unfreeze) builders — Stake 2.0.

use tronz_primitives::{Address, ResourceCode, Trx};

use crate::error::{Error, Result};
use crate::provider::{PendingTransaction, TronProvider};
use crate::types::{
    ContractType, FreezeBalanceV2Contract, TransactionRequest, UnfreezeBalanceV2Contract,
};

/// Stake TRX to obtain energy or bandwidth (`FreezeBalanceV2`).
pub struct FreezeBuilder<'a, P> {
    provider: &'a P,
    owner: Option<Address>,
    amount: Option<Trx>,
    resource: ResourceCode,
}

impl<'a, P: TronProvider> FreezeBuilder<'a, P> {
    /// Start a new freeze builder (defaults to staking for energy).
    pub fn new(provider: &'a P) -> Self {
        Self {
            provider,
            owner: None,
            amount: None,
            resource: ResourceCode::Energy,
        }
    }

    /// Override the staking account.
    pub fn owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Amount of TRX to stake.
    pub fn amount(mut self, amount: Trx) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Resource to obtain.
    pub fn resource(mut self, resource: ResourceCode) -> Self {
        self.resource = resource;
        self
    }

    /// Build, sign, and broadcast.
    pub async fn send(self) -> Result<PendingTransaction<P>> {
        let owner = self
            .owner
            .or_else(|| self.provider.signer_address())
            .ok_or(Error::NoSigner)?;
        let amount = self.amount.ok_or(Error::MissingField("amount"))?;

        let req = TransactionRequest {
            contract: Some(ContractType::FreezeBalanceV2(FreezeBalanceV2Contract {
                owner_address: owner,
                frozen_balance: amount,
                resource: self.resource,
            })),
            ..Default::default()
        };
        self.provider.send_transaction(req).await
    }
}

/// Unstake TRX (`UnfreezeBalanceV2`); subject to the network unbonding delay.
pub struct UnfreezeBuilder<'a, P> {
    provider: &'a P,
    owner: Option<Address>,
    amount: Option<Trx>,
    resource: ResourceCode,
}

impl<'a, P: TronProvider> UnfreezeBuilder<'a, P> {
    /// Start a new unfreeze builder (defaults to releasing energy stake).
    pub fn new(provider: &'a P) -> Self {
        Self {
            provider,
            owner: None,
            amount: None,
            resource: ResourceCode::Energy,
        }
    }

    /// Override the account.
    pub fn owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Amount of TRX to unstake.
    pub fn amount(mut self, amount: Trx) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Resource being released.
    pub fn resource(mut self, resource: ResourceCode) -> Self {
        self.resource = resource;
        self
    }

    /// Build, sign, and broadcast.
    pub async fn send(self) -> Result<PendingTransaction<P>> {
        let owner = self
            .owner
            .or_else(|| self.provider.signer_address())
            .ok_or(Error::NoSigner)?;
        let amount = self.amount.ok_or(Error::MissingField("amount"))?;

        let req = TransactionRequest {
            contract: Some(ContractType::UnfreezeBalanceV2(UnfreezeBalanceV2Contract {
                owner_address: owner,
                unfreeze_balance: amount,
                resource: self.resource,
            })),
            ..Default::default()
        };
        self.provider.send_transaction(req).await
    }
}
