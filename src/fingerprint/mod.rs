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

#[cfg(test)]
mod testgen;

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

use std::collections::BTreeMap;

use crate::universal_schematic::UniversalSchematic;

/// Cheap, translation/rotation-invariant pre-filter descriptor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Signature {
    pub dims_sorted: [u32; 3],
    pub histogram: BTreeMap<Token, u32>,
    pub count: u32,
}

pub fn signature(schem: &UniversalSchematic, spec: &FingerprintSpec) -> Signature {
    let mut histogram: BTreeMap<Token, u32> = BTreeMap::new();
    let mut count = 0u32;
    let mut mn = (i32::MAX, i32::MAX, i32::MAX);
    let mut mx = (i32::MIN, i32::MIN, i32::MIN);
    for (pos, block) in schem.iter_blocks() {
        if let Some(tok) = spec.blocks.tokenize(block) {
            *histogram.entry(tok).or_default() += 1;
            count += 1;
            mn = (mn.0.min(pos.x), mn.1.min(pos.y), mn.2.min(pos.z));
            mx = (mx.0.max(pos.x), mx.1.max(pos.y), mx.2.max(pos.z));
        }
    }
    let mut dims = if count == 0 {
        [0, 0, 0]
    } else {
        [
            (mx.0 - mn.0 + 1) as u32,
            (mx.1 - mn.1 + 1) as u32,
            (mx.2 - mn.2 + 1) as u32,
        ]
    };
    dims.sort_unstable();
    Signature { dims_sorted: dims, histogram, count }
}

/// Exact canonical content fingerprint (128-bit).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fingerprint(pub u128);

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

pub fn fingerprint(schem: &UniversalSchematic, spec: &FingerprintSpec) -> Fingerprint {
    let mut best: Option<Vec<u8>> = None;
    for g in spec.symmetry.elements() {
        let mut cells: Vec<((i32, i32, i32), Token)> = Vec::new();
        for (pos, block) in schem.iter_blocks() {
            let rotated_block = g.apply_block(block);
            if let Some(tok) = spec.blocks.tokenize(&rotated_block) {
                cells.push((g.apply_pos((pos.x, pos.y, pos.z)), tok));
            }
        }
        if cells.is_empty() {
            continue;
        }
        let mn = cells.iter().fold((i32::MAX, i32::MAX, i32::MAX), |m, (p, _)| {
            (m.0.min(p.0), m.1.min(p.1), m.2.min(p.2))
        });
        for (p, _) in cells.iter_mut() {
            *p = (p.0 - mn.0, p.1 - mn.1, p.2 - mn.2);
        }
        cells.sort();
        let ser = serialize_cells(&cells);
        best = Some(match best {
            Some(cur) if cur <= ser => cur,
            _ => ser,
        });
    }
    let bytes = best.unwrap_or_default();
    let hash = blake3::hash(&bytes);
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&hash.as_bytes()[..16]);
    Fingerprint(u128::from_le_bytes(buf))
}

fn serialize_cells(cells: &[((i32, i32, i32), Token)]) -> Vec<u8> {
    let mut out = Vec::with_capacity(cells.len() * 16);
    out.extend_from_slice(&(cells.len() as u32).to_le_bytes());
    for ((x, y, z), tok) in cells {
        out.extend_from_slice(&x.to_le_bytes());
        out.extend_from_slice(&y.to_le_bytes());
        out.extend_from_slice(&z.to_le_bytes());
        out.extend_from_slice(&(tok.len() as u32).to_le_bytes());
        out.extend_from_slice(tok.as_bytes());
    }
    out
}

#[cfg(test)]
mod signature_tests {
    use super::*;
    use crate::fingerprint::testgen::{filled_box, translated};

    #[test]
    fn signature_is_translation_invariant() {
        let a = filled_box((0, 0, 0), (3, 2, 1), "minecraft:stone");
        let b = translated(&a, (40, -5, 12));
        let spec = FingerprintSpec::structural();
        assert_eq!(signature(&a, &spec), signature(&b, &spec));
    }
}

#[cfg(test)]
mod fp_tests {
    use super::*;
    use crate::fingerprint::testgen::{filled_box, rotated_y, translated};

    #[test]
    fn translation_invariant() {
        let a = filled_box((0, 0, 0), (3, 2, 1), "minecraft:stone");
        let b = translated(&a, (17, 4, -9));
        let spec = FingerprintSpec::structural();
        assert_eq!(fingerprint(&a, &spec), fingerprint(&b, &spec));
    }

    #[test]
    fn yaw_rotation_invariant_under_yawmirror() {
        let a = filled_box((0, 0, 0), (3, 0, 1), "minecraft:stone");
        let b = rotated_y(&a, 90);
        let spec = FingerprintSpec::structural();
        assert_eq!(fingerprint(&a, &spec), fingerprint(&b, &spec));
    }

    #[test]
    fn different_builds_differ() {
        let a = filled_box((0, 0, 0), (3, 0, 1), "minecraft:stone");
        let c = filled_box((0, 0, 0), (5, 0, 1), "minecraft:stone");
        let spec = FingerprintSpec::structural();
        assert_ne!(fingerprint(&a, &spec), fingerprint(&c, &spec));
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
