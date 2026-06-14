//! Transaction fillers — composable units that populate a
//! [`TransactionRequest`] before signing.
//!
//! Modeled on alloy's `TxFiller` / `JoinFill` pattern (see `DESIGN.md` §5.3).

use core::future::Future;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tronz_primitives::{Address, RecoverableSignature, Trx, B256};
use tronz_signer::{SignerError, TronSigner};

use crate::error::Result;
use crate::provider::TronProvider;
use crate::types::TransactionRequest;

/// Whether a filler still has work to do for a given request.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FillerStatus {
    /// All of this filler's fields are already present.
    Ready,
    /// This filler has fields left to fill (sync or async).
    NeedsWork,
    /// This filler is a no-op.
    Finished,
}

/// A composable transaction filler.
pub trait TxFiller: Clone + Send + Sync {
    /// Report whether this filler needs to act on `tx`.
    fn status(&self, _tx: &TransactionRequest) -> FillerStatus {
        FillerStatus::Ready
    }

    /// Fill fields that are available synchronously (no network).
    fn fill_sync(&self, _tx: &mut TransactionRequest) {}

    /// Fill fields that require a network round-trip.
    ///
    /// The explicit `+ Send` bound is required so filler futures can run on a
    /// multi-threaded executor, hence the manual `impl Future` form.
    #[allow(clippy::manual_async_fn)]
    fn fill(
        &self,
        tx: TransactionRequest,
        _provider: &impl TronProvider,
    ) -> impl Future<Output = Result<TransactionRequest>> + Send {
        async move { Ok(tx) }
    }
}

/// The empty filler. Does nothing; the identity element for [`JoinFill`].
#[derive(Clone, Copy, Debug, Default)]
pub struct Identity;

impl TxFiller for Identity {
    fn status(&self, _tx: &TransactionRequest) -> FillerStatus {
        FillerStatus::Finished
    }
}

/// Zero-cost combinator that runs `left` then `right`.
#[derive(Clone, Copy, Debug)]
pub struct JoinFill<L, R> {
    /// The first filler to run.
    pub left: L,
    /// The second filler to run.
    pub right: R,
}

impl<L, R> JoinFill<L, R> {
    /// Combine two fillers.
    pub fn new(left: L, right: R) -> Self {
        Self { left, right }
    }
}

impl<L: TxFiller, R: TxFiller> TxFiller for JoinFill<L, R> {
    fn fill_sync(&self, tx: &mut TransactionRequest) {
        self.left.fill_sync(tx);
        self.right.fill_sync(tx);
    }

    #[allow(clippy::manual_async_fn)]
    fn fill(
        &self,
        tx: TransactionRequest,
        provider: &impl TronProvider,
    ) -> impl Future<Output = Result<TransactionRequest>> + Send {
        async move {
            let mut tx = self.left.fill(tx, provider).await?;
            self.left.fill_sync(&mut tx);
            let mut tx = self.right.fill(tx, provider).await?;
            self.right.fill_sync(&mut tx);
            Ok(tx)
        }
    }
}

/// Fills TAPOS fields (`ref_block_*`, `expiration`, `timestamp`) from the
/// latest block. Required before broadcasting client-built transactions.
#[derive(Clone, Copy, Debug)]
pub struct TaposFiller {
    expiry: Duration,
}

impl TaposFiller {
    /// Default 60-second expiry.
    pub fn new() -> Self {
        Self {
            expiry: Duration::from_secs(60),
        }
    }

    /// Override the expiry window.
    pub fn with_expiry(expiry: Duration) -> Self {
        Self { expiry }
    }
}

impl Default for TaposFiller {
    fn default() -> Self {
        Self::new()
    }
}

impl TxFiller for TaposFiller {
    fn status(&self, tx: &TransactionRequest) -> FillerStatus {
        if tx.ref_block_bytes.is_none() {
            FillerStatus::NeedsWork
        } else {
            FillerStatus::Ready
        }
    }

    fn fill(
        &self,
        tx: TransactionRequest,
        provider: &impl TronProvider,
    ) -> impl Future<Output = Result<TransactionRequest>> + Send {
        let expiry = self.expiry;
        async move {
            // Skip if TAPOS was already filled server-side (e.g. trigger calls).
            if tx.ref_block_bytes.is_some() {
                return Ok(tx);
            }
            let mut tx = tx;
            let block = provider.get_now_block().await?;
            tx.ref_block_bytes = Some(block.ref_block_bytes());
            tx.ref_block_hash = Some(block.ref_block_hash());
            let now_ms = unix_now_ms();
            tx.timestamp = Some(now_ms);
            tx.expiration = Some(now_ms + expiry.as_millis() as i64);
            Ok(tx)
        }
    }
}

/// Sets a default `fee_limit` for contract operations that require one.
#[derive(Clone, Copy, Debug)]
pub struct FeeLimitFiller {
    default: Trx,
}

impl FeeLimitFiller {
    /// Use `default` as the fee limit when none is set on a contract operation.
    pub fn new(default: Trx) -> Self {
        Self { default }
    }
}

impl TxFiller for FeeLimitFiller {
    fn fill_sync(&self, tx: &mut TransactionRequest) {
        if tx.fee_limit.is_none() && tx.contract_needs_fee_limit() {
            tx.fee_limit = Some(self.default);
        }
    }
}

/// Carries the signer for a provider. Signing itself happens in the provider's
/// `send_transaction`, after filling completes; this filler is a no-op marker.
#[derive(Clone, Copy, Debug)]
pub struct SignerFiller<S> {
    signer: S,
}

impl<S: TronSigner> SignerFiller<S> {
    /// Wrap a signer.
    pub fn new(signer: S) -> Self {
        Self { signer }
    }

    /// Borrow the wrapped signer.
    pub fn signer(&self) -> &S {
        &self.signer
    }
}

impl<S: TronSigner> TxFiller for SignerFiller<S> {
    // Intentionally a no-op: signing is performed by the provider once the
    // request is fully filled.
}

// ── HasSigner ─────────────────────────────────────────────────────────────────

/// Implemented by filler chains that (may) carry a signer.
///
/// All filler types implement this trait. Non-signer fillers return `None` from
/// both methods; [`SignerFiller`] returns the wrapped signer's address and signs.
/// [`JoinFill`] prefers the right branch, then falls back to the left.
///
/// This allows [`FilledProvider`](crate::provider::FilledProvider) to locate the
/// signer at runtime without knowing the exact filler chain type.
pub trait HasSigner {
    /// The TRON address of the attached signer, if any.
    fn signer_address(&self) -> Option<Address>;

    /// Sign `hash` with the attached signer.  Returns `None` when no signer is
    /// present in this filler chain.
    fn sign(
        &self,
        hash: B256,
    ) -> impl Future<Output = Option<Result<RecoverableSignature, SignerError>>> + Send;
}

impl HasSigner for Identity {
    fn signer_address(&self) -> Option<Address> {
        None
    }

    fn sign(
        &self,
        _hash: B256,
    ) -> impl Future<Output = Option<Result<RecoverableSignature, SignerError>>> + Send {
        async { None }
    }
}

impl HasSigner for TaposFiller {
    fn signer_address(&self) -> Option<Address> {
        None
    }

    fn sign(
        &self,
        _hash: B256,
    ) -> impl Future<Output = Option<Result<RecoverableSignature, SignerError>>> + Send {
        async { None }
    }
}

impl HasSigner for FeeLimitFiller {
    fn signer_address(&self) -> Option<Address> {
        None
    }

    fn sign(
        &self,
        _hash: B256,
    ) -> impl Future<Output = Option<Result<RecoverableSignature, SignerError>>> + Send {
        async { None }
    }
}

impl<S: TronSigner> HasSigner for SignerFiller<S> {
    fn signer_address(&self) -> Option<Address> {
        Some(self.signer.address())
    }

    fn sign(
        &self,
        hash: B256,
    ) -> impl Future<Output = Option<Result<RecoverableSignature, SignerError>>> + Send {
        let signer = self.signer.clone();
        async move { Some(signer.sign_hash(hash).await) }
    }
}

impl<L: HasSigner + Clone + Send, R: HasSigner + Clone + Send> HasSigner for JoinFill<L, R> {
    fn signer_address(&self) -> Option<Address> {
        self.right.signer_address().or_else(|| self.left.signer_address())
    }

    fn sign(
        &self,
        hash: B256,
    ) -> impl Future<Output = Option<Result<RecoverableSignature, SignerError>>> + Send {
        let left = self.left.clone();
        let right = self.right.clone();
        async move {
            if let Some(result) = right.sign(hash).await {
                Some(result)
            } else {
                left.sign(hash).await
            }
        }
    }
}

fn unix_now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}
