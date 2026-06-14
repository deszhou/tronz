#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod error;
pub use error::SignerError;

mod signer;
pub use signer::{NoSigner, TronSigner};

mod local;
pub use local::LocalSigner;

pub use k256;
pub use tronz_primitives::RecoverableSignature;
