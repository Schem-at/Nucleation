//! Canonical build fingerprinting: exact `Fingerprint`, invariant `Signature`,
//! and FFT `Footprint`.
//!
//! See `docs/superpowers/specs/2026-06-01-fingerprint-engine-design.md`.
//!
//! Submodules are declared as they are implemented (each keeps the crate
//! compiling on its own commit).

pub mod classifier;
pub mod symmetry;

#[cfg(test)]
mod smoke {
    #[test]
    fn module_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
