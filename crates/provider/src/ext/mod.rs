//! Extended provider APIs.
//!
//! Each module defines a domain-specific trait with a blanket implementation
//! for any [`TronProvider`](crate::TronProvider). Import the trait to unlock
//! the methods on your provider:
//!
//! ```ignore
//! use tronz_provider::ext::Trc10Api;
//!
//! let info = provider.get_asset_info("1000001").await?;
//! let pending = provider.transfer_trc10()
//!     .to(recipient)
//!     .token_id("1000001")
//!     .amount(1_000_000)
//!     .send()
//!     .await?;
//! ```

mod trc10;
pub use trc10::{TransferTrc10Builder, Trc10Api};
