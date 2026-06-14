//! Delegate / undelegate resource builders.

use tronz_primitives::{Address, ResourceCode, Trx};

use crate::error::{Error, Result};
use crate::provider::{PendingTransaction, TronProvider};
use crate::types::{
    ContractType, DelegateResourceContract, TransactionRequest, UnDelegateResourceContract,
};

/// Delegate staked energy or bandwidth to another account.
pub struct DelegateBuilder<'a, P> {
    provider: &'a P,
    owner: Option<Address>,
    to: Option<Address>,
    amount: Option<Trx>,
    resource: ResourceCode,
    lock_period: Option<i64>,
}

impl<'a, P: TronProvider> DelegateBuilder<'a, P> {
    /// Start a new delegate builder (defaults to delegating energy).
    pub fn new(provider: &'a P) -> Self {
        Self {
            provider,
            owner: None,
            to: None,
            amount: None,
            resource: ResourceCode::Energy,
            lock_period: None,
        }
    }

    /// Override the delegator account.
    pub fn owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Delegatee account.
    pub fn to(mut self, to: Address) -> Self {
        self.to = Some(to);
        self
    }

    /// Amount of staked TRX whose resource is delegated.
    pub fn amount(mut self, amount: Trx) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Resource being delegated.
    pub fn resource(mut self, resource: ResourceCode) -> Self {
        self.resource = resource;
        self
    }

    /// Lock the delegation for `secs` seconds (max 864_000 per protocol).
    pub fn lock_period(mut self, secs: i64) -> Self {
        self.lock_period = Some(secs);
        self
    }

    /// Build, sign, and broadcast.
    pub async fn send(self) -> Result<PendingTransaction<P>> {
        let owner = self
            .owner
            .or_else(|| self.provider.signer_address())
            .ok_or(Error::NoSigner)?;
        let to = self.to.ok_or(Error::MissingField("to"))?;
        let amount = self.amount.ok_or(Error::MissingField("amount"))?;

        let req = TransactionRequest {
            contract: Some(ContractType::DelegateResource(DelegateResourceContract {
                owner_address: owner,
                resource: self.resource,
                balance: amount,
                receiver_address: to,
                lock_period: self.lock_period,
            })),
            ..Default::default()
        };
        self.provider.send_transaction(req).await
    }
}

/// Reclaim resources previously delegated to another account.
pub struct UndelegateBuilder<'a, P> {
    provider: &'a P,
    owner: Option<Address>,
    from: Option<Address>,
    amount: Option<Trx>,
    resource: ResourceCode,
}

impl<'a, P: TronProvider> UndelegateBuilder<'a, P> {
    /// Start a new undelegate builder (defaults to reclaiming energy).
    pub fn new(provider: &'a P) -> Self {
        Self {
            provider,
            owner: None,
            from: None,
            amount: None,
            resource: ResourceCode::Energy,
        }
    }

    /// Override the delegator account.
    pub fn owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Account whose delegation is being reclaimed.
    pub fn from(mut self, from: Address) -> Self {
        self.from = Some(from);
        self
    }

    /// Amount of staked TRX whose resource is reclaimed.
    pub fn amount(mut self, amount: Trx) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Resource being reclaimed.
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
        let from = self.from.ok_or(Error::MissingField("from"))?;
        let amount = self.amount.ok_or(Error::MissingField("amount"))?;

        let req = TransactionRequest {
            contract: Some(ContractType::UnDelegateResource(UnDelegateResourceContract {
                owner_address: owner,
                resource: self.resource,
                balance: amount,
                receiver_address: from,
            })),
            ..Default::default()
        };
        self.provider.send_transaction(req).await
    }
}
