//! Canonical build fingerprinting: exact `Fingerprint`, invariant `Signature`,
//! and FFT `Footprint`.
//!
//! See `docs/superpowers/specs/2026-06-01-fingerprint-engine-design.md`.
//!
//! Submodules are declared as they are implemented (each keeps the crate
//! compiling on its own commit).

pub mod classifier;
pub mod rulesets;
pub mod symmetry;

use crate::block_state::BlockState;
use crate::fingerprint::classifier::{Classifier, Token};
use crate::fingerprint::symmetry::Symmetry;

fn is_air(name: &str) -> bool {
    matches!(name, "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air")
}

/// How a blockstate is reduced to a token before canonicalization.
#[derive(Clone)]
pub enum BlockPolicy {
    /// Full block id + sorted properties (air ignored).
    Exact,
    /// Block id only — shape (air ignored).
    IdOnly,
    /// Rule-based functional classification.
    Classify(Classifier),
}

impl BlockPolicy {
    pub fn tokenize(&self, b: &BlockState) -> Option<Token> {
        match self {
            BlockPolicy::Classify(c) => c.tokenize(b),
            BlockPolicy::IdOnly => {
                if is_air(b.get_name()) {
                    None
                } else {
                    Some(Token::from(b.get_name()))
                }
            }
            BlockPolicy::Exact => {
                if is_air(b.get_name()) {
                    return None;
                }
                let mut props: Vec<(&str, &str)> =
                    b.properties.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                props.sort_unstable();
                let mut s = b.get_name().to_string();
                for (k, v) in props {
                    s.push('|');
                    s.push_str(k);
                    s.push('=');
                    s.push_str(v);
                }
                Some(Token::from(s))
            }
        }
    }
}

/// A fingerprint preset: a symmetry group paired with a block-token policy.
#[derive(Clone)]
pub struct FingerprintSpec {
    pub symmetry: Symmetry,
    pub blocks: BlockPolicy,
}

impl FingerprintSpec {
    pub fn exact() -> Self {
        Self { symmetry: Symmetry::None, blocks: BlockPolicy::Exact }
    }
    pub fn shape() -> Self {
        Self { symmetry: Symmetry::Octahedral, blocks: BlockPolicy::IdOnly }
    }
    pub fn structural() -> Self {
        Self { symmetry: Symmetry::YawMirror, blocks: BlockPolicy::Classify(rulesets::structural()) }
    }
    pub fn redstone_computational() -> Self {
        Self {
            symmetry: Symmetry::YawMirror,
            blocks: BlockPolicy::Classify(rulesets::redstone_computational()),
        }
    }
    pub fn redstone_survival() -> Self {
        Self {
            symmetry: Symmetry::YawMirror,
            blocks: BlockPolicy::Classify(rulesets::redstone_survival()),
        }
    }
    pub fn custom(symmetry: Symmetry, blocks: BlockPolicy) -> Self {
        Self { symmetry, blocks }
    }
}

#[cfg(test)]
mod spec_tests {
    use super::*;

    #[test]
    fn policies_tokenize() {
        let repeater = BlockState::new("minecraft:repeater")
            .with_properties(vec![("facing".into(), "east".into())]);
        assert_eq!(
            BlockPolicy::Exact.tokenize(&repeater).as_deref(),
            Some("minecraft:repeater|facing=east")
        );
        assert_eq!(
            BlockPolicy::IdOnly.tokenize(&repeater).as_deref(),
            Some("minecraft:repeater")
        );
        assert_eq!(BlockPolicy::Exact.tokenize(&BlockState::new("minecraft:air")), None);
    }

    #[test]
    fn presets_build() {
        let _ = FingerprintSpec::redstone_computational();
        let _ = FingerprintSpec::structural();
        let _ = FingerprintSpec::exact();
        let _ = FingerprintSpec::shape();
        let _ = FingerprintSpec::redstone_survival();
    }
}

#[cfg(test)]
mod smoke {
    #[test]
    fn module_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
