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

    /// I/O error (e.g. writing a mnemonic phrase to disk or a keystore file).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Keystore-specific error (wrong password, unsupported algorithm, etc.).
    #[cfg(feature = "keystore")]
    #[error(transparent)]
    Keystore(#[from] crate::keystore::KeystoreError),

    /// JSON serialization/deserialization error.
    #[cfg(feature = "keystore")]
    #[error("JSON error: {0}")]
    Json(serde_json::Error),

    /// BIP-32 HD key derivation error.
    #[cfg(feature = "mnemonic")]
    #[error("BIP-32 error: {0}")]
    Bip32(#[from] coins_bip32::Bip32Error),

    /// BIP-39 mnemonic error.
    #[cfg(feature = "mnemonic")]
    #[error("BIP-39 error: {0}")]
    Bip39(#[from] coins_bip39::MnemonicError),

    /// [`MnemonicBuilder`](crate::mnemonic::MnemonicBuilder) misuse.
    #[cfg(feature = "mnemonic")]
    #[error("{0}")]
    MnemonicBuilder(#[from] crate::mnemonic::MnemonicBuilderError),
}
