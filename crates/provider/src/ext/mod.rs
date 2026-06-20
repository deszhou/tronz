//! Extended provider APIs.
//!
//! Each module defines a domain-specific trait with a blanket implementation
//! for any [`TronProvider`](crate::TronProvider). Import the trait to unlock
//! the methods on your provider:
//!
//! ```ignore
//! use tronz_provider::ext::{Trc10Api, WitnessApi};
//!
//! let info = provider.get_asset_info("1000001").await?;
//! let pending = provider
//!     .become_witness()
//!     .url("https://my-sr.example.com")
//!     .send()
//!     .await?;
//! ```

mod exchange;
mod governance;
mod market;
mod trc10;
mod witness;
pub use exchange::{
    ExchangeApi, ExchangeCreateBuilder, ExchangeInjectBuilder, ExchangeTradeBuilder,
    ExchangeWithdrawBuilder,
};
pub use governance::{
    ApproveProposalBuilder, CancelProposalBuilder, GovernanceApi, SubmitProposalBuilder,
};
pub use market::{MarketApi, MarketCancelBuilder, MarketSellBuilder};
pub use trc10::{
    IssueTrc10Builder, ParticipateTrc10Builder, TransferTrc10Builder, Trc10Api,
    UnfreezeTrc10Builder, UpdateTrc10Builder,
};
pub use witness::{BecomeWitnessBuilder, UpdateBrokerageBuilder, UpdateWitnessBuilder, WitnessApi};
