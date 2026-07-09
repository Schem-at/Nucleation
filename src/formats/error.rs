//! Error type for the schematic/world format-parsing layer
//! (`src/formats/`, `src/dataconverter/`).
//!
//! Replaces the ad-hoc `Box<dyn std::error::Error>` / bare `String` errors
//! historically used across these modules with a single `thiserror`-based
//! enum, mirroring the conventions in [`crate::store::error::StoreError`]
//! and [`crate::meshing::MeshError`].
//!
//! The overwhelming majority of call sites in this layer either propagate a
//! concrete library error via `?` (NBT, zip, JSON, bincode, io) or construct
//! an ad-hoc message via `"...".into()` / `format!(...).into()`. The
//! [`From<String>`] / [`From<&str>`] impls below mean those message-based
//! call sites need no changes beyond the enclosing function's signature.

use thiserror::Error;

/// Everything that can go wrong parsing, converting, or serializing a
/// schematic/world format.
#[derive(Debug, Error)]
pub enum FormatError {
    /// An ad-hoc, human-readable parse/validation failure constructed in
    /// this crate (e.g. `"MCA file too small (< 8192 bytes)".into()`).
    #[error("{0}")]
    Parse(String),

    /// Underlying IO failure (file read/write, seek, decompression stream, …).
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Malformed or truncated NBT binary data (gzip/zlib NBT stream framing,
    /// tag type mismatches while reading/writing raw bytes).
    #[error("NBT io error: {0}")]
    NbtIo(#[from] quartz_nbt::io::NbtIoError),

    /// A well-formed NBT compound was missing an expected key or the value
    /// under it had the wrong tag type.
    #[error("NBT structure error: {0}")]
    NbtRepr(#[from] quartz_nbt::NbtReprError),

    /// Failure opening, reading, or writing a ZIP archive (world export/import).
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    /// Malformed JSON (import/export settings, `NucleationDefinitions`, …).
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Malformed bincode payload (`.nusn` snapshot format).
    #[error("bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    /// A fixed-size byte slice (e.g. a version header) was the wrong length.
    #[error("slice conversion error: {0}")]
    TryFromSlice(#[from] std::array::TryFromSliceError),
}

impl From<String> for FormatError {
    fn from(s: String) -> Self {
        FormatError::Parse(s)
    }
}

impl From<&str> for FormatError {
    fn from(s: &str) -> Self {
        FormatError::Parse(s.to_string())
    }
}

/// Convenience alias for `Result<T, FormatError>`, used throughout
/// `src/formats/` and `src/dataconverter/` instead of spelling out the error
/// type on every function.
pub type Result<T> = std::result::Result<T, FormatError>;
