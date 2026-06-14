//! Claim block/vote rewards builder.

use tronz_primitives::Address;

use crate::error::{Error, Result};
use crate::provider::{PendingTransaction, TronProvider};
use crate::types::{ContractType, TransactionRequest, WithdrawBalanceContract};

/// Claim accumulated block/vote rewards (`WithdrawBalance`).
///
/// Note: TRON allows this at most once per 24h per account.
pub struct WithdrawBalanceBuilder<'a, P> {
    provider: &'a P,
    owner: Option<Address>,
}

impl<'a, P: TronProvider> WithdrawBalanceBuilder<'a, P> {
    /// Start a new builder.
    pub fn new(provider: &'a P) -> Self {
        Self {
            provider,
            owner: None,
        }
    }

    /// Override the account.
    pub fn owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Build, sign, and broadcast.
    pub async fn send(self) -> Result<PendingTransaction<P>> {
        let owner = self
            .owner
            .or_else(|| self.provider.signer_address())
            .ok_or(Error::NoSigner)?;
        let req = TransactionRequest {
            contract: Some(ContractType::WithdrawBalance(WithdrawBalanceContract {
                owner_address: owner,
            })),
            ..Default::default()
        };
        self.provider.send_transaction(req).await
    }
}
