//! Error types for contract interactions.

use alloy_primitives::{B256, Selector};
use alloy_sol_types::{SolError, SolInterface};
use thiserror::Error;
use tronz_primitives::Bytes;
use tronz_provider::Error as ProviderError;

/// The result type for contract operations.
pub type Result<T, E = ContractError> = std::result::Result<T, E>;

/// Errors returned by [`ContractInstance`](crate::instance::ContractInstance) and
/// token instance methods.
#[derive(Debug, Error)]
pub enum ContractError {
    /// A provider, transport, or signing error.
    #[error(transparent)]
    Provider(#[from] ProviderError),
    /// ABI encoding or decoding failed.
    #[error("ABI error: {0}")]
    Abi(#[from] alloy_dyn_abi::Error),
    /// The requested function name was not found in the ABI.
    #[error("unknown function: `{0}`")]
    UnknownFunction(String),
    /// The requested function selector was not found in the ABI.
    #[error("unknown function selector: {0}")]
    UnknownSelector(Selector),
    /// No signer is attached to the provider.
    #[error("no signer attached")]
    NoSigner,
    /// The contract returned no data — the address may not be a contract.
    #[error("contract call to `{0}` returned no data; the address might not be a contract")]
    ZeroData(String, #[source] alloy_dyn_abi::Error),
    /// The contract call reverted. Contains the raw ABI-encoded revert data.
    #[error("contract call reverted")]
    ContractRevert(Bytes),
    /// The requested event topic was not found in the ABI.
    #[error("unknown event topic: {0}")]
    UnknownEvent(B256),
}

impl From<alloy_sol_types::Error> for ContractError {
    fn from(e: alloy_sol_types::Error) -> Self {
        Self::Abi(e.into())
    }
}

impl ContractError {
    /// Returns the raw ABI-encoded revert data if the error is a [`ContractRevert`].
    ///
    /// [`ContractRevert`]: ContractError::ContractRevert
    pub fn as_revert_data(&self) -> Option<&Bytes> {
        if let Self::ContractRevert(data) = self {
            Some(data)
        } else {
            None
        }
    }

    /// Attempt to ABI-decode the revert data into a specific [`SolError`] type.
    ///
    /// Returns `None` if the error is not a revert, or if decoding fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use alloy_sol_types::sol;
    /// sol! { error InsufficientBalance(uint256 have, uint256 need); }
    ///
    /// # fn example(err: tronz_contract::ContractError) {
    /// if let Some(e) = err.as_decoded_error::<InsufficientBalance>() {
    ///     println!("need {} but only have {}", e.need, e.have);
    /// }
    /// # }
    /// ```
    pub fn as_decoded_error<E: SolError>(&self) -> Option<E> {
        self.as_revert_data().and_then(|data| E::abi_decode(data).ok())
    }

    /// Attempt to ABI-decode the revert data into one of the custom errors in a [`SolInterface`].
    ///
    /// Returns `None` if the error is not a revert, or if decoding fails.
    pub fn as_decoded_interface_error<I: SolInterface>(&self) -> Option<I> {
        self.as_revert_data().and_then(|data| I::abi_decode(data).ok())
    }

    /// Build a [`ContractError`] from a failed output decode.
    ///
    /// Promotes empty output to [`ZeroData`] for a more helpful error message.
    ///
    /// [`ZeroData`]: ContractError::ZeroData
    pub(crate) fn decode_err(name: &str, data: &[u8], error: alloy_dyn_abi::Error) -> Self {
        if data.is_empty() {
            let short = name.split('(').next().unwrap_or(name);
            return Self::ZeroData(short.to_string(), error);
        }
        Self::Abi(error)
    }
}
