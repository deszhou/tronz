//! Error types for `tronz-signer`.

/// Errors produced while creating a signer or signing a payload.
#[derive(thiserror::Error, Debug)]
pub enum SignerError {
    /// The underlying ECDSA operation failed.
    #[error("signing failed: {0}")]
    Ecdsa(#[from] k256::ecdsa::Error),

    /// A private key could not be decoded from the supplied bytes/hex.
    #[error("invalid private key: {0}")]
    InvalidKey(String),

    /// hex decoding of a private key failed.
    #[error("hex decode failed: {0}")]
    Hex(#[from] hex::FromHexError),

    /// The signer has no associated address (e.g. [`NoSigner`](crate::NoSigner)).
    #[error("signer has no address")]
    NoAddress,
}
