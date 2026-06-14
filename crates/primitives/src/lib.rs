#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod address;
mod amount;
mod error;
mod resource;
mod signature;

pub use address::{Address, ADDRESS_LEN, ADDRESS_PREFIX, EVM_ADDRESS_LEN};
pub use amount::{Trx, SUN_PER_TRX};
pub use error::{AddressError, AmountError, SignatureError};
pub use resource::ResourceCode;
pub use signature::{RecoverableSignature, SIGNATURE_LEN};

/// Types re-used directly from `alloy-primitives`.
pub use alloy_primitives::{keccak256, Bytes, B256, U256};

/// A transaction id: `sha256` of the protobuf-encoded raw transaction.
///
/// Defined here so it can appear in both signer and provider signatures
/// without a dependency cycle.
pub type TxId = B256;
