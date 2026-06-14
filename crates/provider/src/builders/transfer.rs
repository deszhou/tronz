//! TRX transfer builder.

use tronz_primitives::{Address, Trx};

use crate::error::{Error, Result};
use crate::provider::{PendingTransaction, TronProvider};
use crate::types::{ContractType, TransactionRequest, TransferContract};

/// Builds a TRX transfer (`send_trx`).
pub struct TransferBuilder<'a, P> {
    provider: &'a P,
    owner: Option<Address>,
    to: Option<Address>,
    amount: Option<Trx>,
    memo: Option<Vec<u8>>,
}

impl<'a, P: TronProvider> TransferBuilder<'a, P> {
    /// Start a new transfer builder.
    pub fn new(provider: &'a P) -> Self {
        Self {
            provider,
            owner: None,
            to: None,
            amount: None,
            memo: None,
        }
    }

    /// Override the sender (defaults to the provider's signer address).
    pub fn owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Set the recipient.
    pub fn to(mut self, to: Address) -> Self {
        self.to = Some(to);
        self
    }

    /// Set the amount.
    pub fn amount(mut self, amount: Trx) -> Self {
        self.amount = Some(amount);
        self
    }

    /// Attach a memo.
    pub fn memo(mut self, memo: impl Into<Vec<u8>>) -> Self {
        self.memo = Some(memo.into());
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
            contract: Some(ContractType::Transfer(TransferContract {
                owner_address: owner,
                to_address: to,
                amount,
            })),
            memo: self.memo,
            ..Default::default()
        };
        self.provider.send_transaction(req).await
    }
}
