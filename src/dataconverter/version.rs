//! Encoded version arithmetic, ported from
//! `DataConverterJava/.../converters/DataConverter.java:39-53`.
//!
//! A "version" in the engine is a single 64-bit value packing the integer data
//! version in the high 32 bits and a sub-step in the low 32 bits, so that
//! `encode(v, step) < encode(v, step+1) < encode(v+1, 0)` — a total order over
//! (version, step). The walk in [`crate::dataconverter::engine`] iterates this
//! order ascending (forward) or descending (reverse).

/// An encoded (version, step) pair. High 32 bits = data version, low 32 = step.
pub type EncodedVersion = u64;

/// `encodeVersions(version, step)` — DataConverter.java:39-41.
#[inline]
pub const fn encode_versions(version: i32, step: i32) -> EncodedVersion {
    ((version as i64 as u64) << 32) | ((step as u32) as u64)
}

/// Decode the data version (high 32 bits) — DataConverter.java:43-45.
#[inline]
pub const fn get_version(encoded: EncodedVersion) -> i32 {
    (encoded >> 32) as i32
}

/// Decode the sub-step (low 32 bits) — DataConverter.java:47-49.
#[inline]
pub const fn get_step(encoded: EncodedVersion) -> i32 {
    encoded as u32 as i32
}

/// Human-readable `version.step` — DataConverter.java:51-53.
pub fn encoded_to_string(encoded: EncodedVersion) -> String {
    format!("{}.{}", get_version(encoded), get_step(encoded))
}

/// The step value used for the endpoints of a top-level conversion request:
/// `Integer.MAX_VALUE`, so the source endpoint sits past every sub-step of its
/// version and the target endpoint includes every sub-step of the target
/// (MCDataConverter.java:43-49).
pub const MAX_STEP: i32 = i32::MAX;

/// `V99.VERSION` — the legacy/pre-converter sentinel. Data with no `DataVersion`
/// tag (1.7.10 and earlier) is clamped up to this before conversion
/// (MCDataConverter.java:46).
pub const V99: i32 = 99;
