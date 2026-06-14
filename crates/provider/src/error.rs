//! Error types for `tronz-provider`.

/// Errors originating in the transport layer (gRPC + codec).
#[derive(thiserror::Error, Debug)]
pub enum TransportError {
    /// The gRPC channel returned an error status.
    #[error("gRPC status {}: {}", .0.code(), .0.message())]
    Grpc(#[from] tonic::Status),

    /// Failed to connect or configure the gRPC channel.
    #[error("gRPC transport error: {0}")]
    Connect(#[from] tonic::transport::Error),

    /// A protobuf payload failed to decode.
    #[error("protobuf decode error: {0}")]
    Proto(#[from] prost::DecodeError),

    /// A response field was missing or malformed.
    #[error("malformed response: {0}")]
    Malformed(String),

    /// The node returned a failure result.
    #[error("node error: {0}")]
    NodeError(String),

    /// The requested resource was not found.
    #[error("not found")]
    NotFound,
}

/// Errors surfaced by the provider layer.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// A transport-level failure.
    #[error(transparent)]
    Transport(#[from] TransportError),

    /// A signing failure.
    #[error(transparent)]
    Signer(#[from] tronz_signer::SignerError),

    /// An address parsing/validation failure.
    #[error(transparent)]
    Address(#[from] tronz_primitives::AddressError),

    /// A `.send()` was attempted on a provider with no signer attached.
    #[error("no signer attached to this provider")]
    NoSigner,

    /// The transaction reverted on-chain.
    #[error("transaction reverted: {0}")]
    Revert(String),

    /// Confirmation polling exceeded its deadline.
    #[error("timed out waiting for confirmation")]
    ConfirmationTimeout,

    /// A required field was missing when building a transaction.
    #[error("missing required field: {0}")]
    MissingField(&'static str),
}

/// Convenient `Result` alias defaulting to the provider [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
