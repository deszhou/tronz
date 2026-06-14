#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod builders;
pub mod ext;
pub mod fillers;
pub mod transport;
pub mod types;

mod error;
pub use error::{Error, Result, TransportError};

mod provider;
pub use provider::{FilledProvider, PendingTransaction, ProviderBuilder, RootProvider, TronProvider};

pub use ext::Trc10Api;
pub use fillers::HasSigner;
pub use transport::TronTransport;

// Private: prost-generated code + codec conversions never leak publicly.
pub(crate) mod proto;
