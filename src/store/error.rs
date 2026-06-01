//! Error taxonomy for the storage abstraction.
//!
//! Variants only ever carry strings we construct ourselves, so credentials
//! never leak through `Display`/`Debug` — backends must not place secrets in
//! these messages.

use thiserror::Error;

/// Everything that can go wrong talking to a [`Store`](super::Store).
#[derive(Debug, Error)]
pub enum StoreError {
    /// No object exists at the requested key.
    #[error("key not found: {0}")]
    NotFound(String),

    /// The key is malformed (e.g. path traversal in a filesystem store).
    #[error("invalid key: {0}")]
    InvalidKey(String),

    /// Underlying IO failure (disk, socket, …). Never contains credentials.
    #[error("io error: {0}")]
    Io(String),

    /// Authentication / authorization failed. Carries no payload by design.
    #[error("authentication failed")]
    Auth,

    /// Could not reach the backing service.
    #[error("connection error: {0}")]
    Connection(String),

    /// The backend does not support this operation or URL scheme.
    #[error("unsupported: {0}")]
    Unsupported(String),

    /// Anything else, with a safe message.
    #[error("store error: {0}")]
    Other(String),
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => StoreError::NotFound(e.to_string()),
            _ => StoreError::Io(e.to_string()),
        }
    }
}
