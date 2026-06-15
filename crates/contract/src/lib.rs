#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(test)]
extern crate self as tronz_contract;

/// Static TRC20 ABI bindings and encode/decode helpers.
pub mod trc20;

/// Event log decoding helpers for TRON smart contracts.
pub mod event;
/// Re-exported alloy ABI types for use with generated calls.
pub use alloy_sol_types::{SolCall, SolError, SolEvent, SolInterface, SolValue};
#[cfg(feature = "provider")]
pub use event::{decode_log, decode_logs, log_matches, topic0_set};
pub use tronz_primitives::{Address, Bytes, U256};

#[cfg(feature = "provider")]
mod error;
#[cfg(feature = "provider")]
pub use error::{ContractError, Result};

#[cfg(feature = "provider")]
mod interface;
#[cfg(feature = "provider")]
pub use alloy_dyn_abi::DecodedEvent;
#[cfg(feature = "provider")]
pub use interface::Interface;

#[cfg(feature = "provider")]
mod instance;
#[cfg(feature = "provider")]
pub use instance::{ContractExt, ContractInstance};

#[cfg(feature = "provider")]
mod call;
#[cfg(feature = "provider")]
pub use call::CallBuilder;

#[cfg(feature = "provider")]
mod deploy;
#[cfg(feature = "provider")]
pub use deploy::DeployBuilder;
#[cfg(feature = "provider")]
pub use trc20::{Trc20Error, Trc20Ext, Trc20Instance};
