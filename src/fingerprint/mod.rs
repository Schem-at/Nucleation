//! Canonical build fingerprinting: exact `Fingerprint`, invariant `Signature`,
//! and FFT `Footprint`.
//!
//! See `docs/superpowers/specs/2026-06-01-fingerprint-engine-design.md`.
//!
//! Submodules are declared as they are implemented (each keeps the crate
//! compiling on its own commit).

pub mod classifier;
pub mod footprint;
pub mod rulesets;
pub mod symmetry;
pub mod voxel;

pub use footprint::{footprint, Footprint};

#[cfg(test)]
pub(crate) mod testgen;

use crate::block_state::BlockState;
use crate::fingerprint::classifier::{Classifier, Token};
use crate::fingerprint::symmetry::Symmetry;
use crate::utils::{NbtMap, NbtValue};

/// Top-level block-entity NBT keys that restate WHERE the entity sits (or its
/// id, which the block token already carries) rather than what it contains.
/// Excluded from the stable NBT hash so translated copies stay equal.
fn is_positional_key(k: &str) -> bool {
    matches!(k, "x" | "y" | "z" | "id" | "Id" | "Pos")
}

/// Top-level block-entity NBT keys that encode a facing/rotation. Under a
/// rotation-tolerant symmetry these must be excluded from the hash: the block
/// token is rotated by the group element but the NBT is not, so a genuinely
/// rotated copy would otherwise fail to match itself. Under `Symmetry::None`
/// they are kept — there is no rotation to be invariant to, and the rotation
/// is real content.
fn is_directional_key(k: &str) -> bool {
    matches!(
        k,
        "Rotation" | "rotation" | "Facing" | "facing" | "Rot" | "rot"
    )
}

/// Unambiguous, order-independent text serialization of an NBT value:
/// compound keys sorted recursively, strings length-prefixed, floats by bit
/// pattern. Used only for hashing (never parsed back).
fn write_nbt_value(out: &mut String, v: &NbtValue) {
    use std::fmt::Write;
    let write_str = |out: &mut String, s: &str| {
        let _ = write!(out, "{}:", s.len());
        out.push_str(s);
    };
    match v {
        NbtValue::Byte(x) => {
            let _ = write!(out, "B{x}");
        }
        NbtValue::Short(x) => {
            let _ = write!(out, "S{x}");
        }
        NbtValue::Int(x) => {
            let _ = write!(out, "I{x}");
        }
        NbtValue::Long(x) => {
            let _ = write!(out, "L{x}");
        }
        NbtValue::Float(x) => {
            let _ = write!(out, "F{}", x.to_bits());
        }
        NbtValue::Double(x) => {
            let _ = write!(out, "D{}", x.to_bits());
        }
        NbtValue::String(s) => {
            out.push('T');
            write_str(out, s);
        }
        NbtValue::ByteArray(a) => {
            out.push_str("BA[");
            for x in a {
                let _ = write!(out, "{x},");
            }
            out.push(']');
        }
        NbtValue::IntArray(a) => {
            out.push_str("IA[");
            for x in a {
                let _ = write!(out, "{x},");
            }
            out.push(']');
        }
        NbtValue::LongArray(a) => {
            out.push_str("LA[");
            for x in a {
                let _ = write!(out, "{x},");
            }
            out.push(']');
        }
        NbtValue::List(l) => {
            out.push('[');
            for x in l {
                write_nbt_value(out, x);
                out.push(',');
            }
            out.push(']');
        }
        NbtValue::Compound(m) => {
            let mut entries: Vec<(&String, &NbtValue)> = m.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));
            out.push('{');
            for (k, v) in entries {
                write_str(out, k);
                out.push('=');
                write_nbt_value(out, v);
                out.push(';');
            }
            out.push('}');
        }
    }
}

/// Stable hash token for a block entity's NBT payload, or `None` when there
/// is no payload beyond positional/id keys (an empty tile-entity record must
/// fingerprint the same as no record at all). Deterministic across HashMap
/// iteration orders: top-level and nested compound keys are sorted.
pub(crate) fn stable_nbt_token(nbt: &NbtMap, ignore_directional: bool) -> Option<Token> {
    let mut entries: Vec<(&String, &NbtValue)> = nbt
        .iter()
        .filter(|(k, _)| !is_positional_key(k))
        .filter(|(k, _)| !(ignore_directional && is_directional_key(k)))
        .collect();
    if entries.is_empty() {
        return None;
    }
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut out = String::new();
    out.push('{');
    for (k, v) in entries {
        use std::fmt::Write;
        let _ = write!(out, "{}:", k.len());
        out.push_str(k);
        out.push('=');
        write_nbt_value(&mut out, v);
        out.push(';');
    }
    out.push('}');
    let hash = blake3::hash(out.as_bytes());
    let hex = hash.to_hex();
    Some(Token::from(&hex.as_str()[..NBT_TOKEN_HEX_LEN]))
}

/// Separator between a cell's block token and its block-entity NBT hash.
pub(crate) const NBT_MARKER: &str = "#nbt:";
/// Hex width of the truncated blake3 NBT hash.
pub(crate) const NBT_TOKEN_HEX_LEN: usize = 16;

/// A cell token augmented with its block entity's stable NBT token, so cells
/// differing only in tile-entity data (sign text, chest contents) compare and
/// hash as different.
pub(crate) fn token_with_nbt(tok: &Token, nbt_tok: &Token) -> Token {
    let mut s = String::with_capacity(tok.len() + nbt_tok.len() + NBT_MARKER.len());
    s.push_str(tok.as_str());
    s.push_str(NBT_MARKER);
    s.push_str(nbt_tok.as_str());
    Token::from(s)
}

/// True if `tok` was augmented with block-entity NBT by [`token_with_nbt`].
/// Matches the trailing marker + fixed-width hex hash rather than a bare
/// substring, so a block token that merely contains `#nbt:` (a modded id, a
/// property value) is not misread as carrying NBT.
pub(crate) fn token_has_nbt(tok: &Token) -> bool {
    match tok.as_str().rsplit_once(NBT_MARKER) {
        Some((_, hash)) => {
            hash.len() == NBT_TOKEN_HEX_LEN && hash.bytes().all(|b| b.is_ascii_hexdigit())
        }
        None => false,
    }
}

pub(crate) fn is_air(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
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
                let mut props: Vec<(&str, &str)> = b
                    .properties
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
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
    /// Fold block-entity NBT (sign text, chest contents) into each cell's
    /// token. Only meaningful for content-exact matching: the fuzzy presets
    /// deliberately ignore block identity, so making them sensitive to loot
    /// tables or sign text would defeat their purpose.
    pub block_entities: bool,
}

impl FingerprintSpec {
    pub fn exact() -> Self {
        Self {
            symmetry: Symmetry::None,
            blocks: BlockPolicy::Exact,
            block_entities: true,
        }
    }
    pub fn shape() -> Self {
        Self {
            symmetry: Symmetry::Octahedral,
            blocks: BlockPolicy::IdOnly,
            block_entities: false,
        }
    }
    pub fn structural() -> Self {
        Self {
            symmetry: Symmetry::YawMirror,
            blocks: BlockPolicy::Classify(rulesets::structural()),
            block_entities: false,
        }
    }
    pub fn redstone_computational() -> Self {
        Self {
            symmetry: Symmetry::YawMirror,
            blocks: BlockPolicy::Classify(rulesets::redstone_computational()),
            block_entities: false,
        }
    }
    pub fn redstone_survival() -> Self {
        Self {
            symmetry: Symmetry::YawMirror,
            blocks: BlockPolicy::Classify(rulesets::redstone_survival()),
            block_entities: false,
        }
    }
    pub fn custom(symmetry: Symmetry, blocks: BlockPolicy) -> Self {
        Self {
            symmetry,
            blocks,
            block_entities: false,
        }
    }

    /// Builder: opt this spec in or out of block-entity NBT sensitivity.
    pub fn with_block_entities(mut self, on: bool) -> Self {
        self.block_entities = on;
        self
    }

    /// Canonical preset names. `redstone` aliases `redstone_computational`.
    pub const PRESETS: &'static [&'static str] = &[
        "exact",
        "shape",
        "structural",
        "redstone_computational",
        "redstone",
        "redstone_survival",
    ];

    /// Resolve a preset name to a spec. Returns `None` for unknown names.
    pub fn from_preset(name: &str) -> Option<Self> {
        Some(match name {
            "exact" => Self::exact(),
            "shape" => Self::shape(),
            "structural" => Self::structural(),
            "redstone" | "redstone_computational" => Self::redstone_computational(),
            "redstone_survival" => Self::redstone_survival(),
            _ => return None,
        })
    }
}

use std::collections::{BTreeMap, HashMap};

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
    Signature {
        dims_sorted: dims,
        histogram,
        count,
    }
}

/// Exact canonical content fingerprint (128-bit).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Fingerprint(pub u128);

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl Fingerprint {
    /// Lowercase 32-char hex of the 128-bit fingerprint.
    pub fn to_hex(&self) -> String {
        format!("{:032x}", self.0)
    }
}

/// True if `a` and `b` share a fingerprint under `spec`.
pub fn is_duplicate(
    a: &UniversalSchematic,
    b: &UniversalSchematic,
    spec: &FingerprintSpec,
) -> bool {
    fingerprint(a, spec) == fingerprint(b, spec)
}

/// Translation-invariant fuzzy distance between two builds' FFT footprints
/// (0.0 = identical occupancy shape). Cheaper-to-compare than exact fingerprints.
pub fn footprint_distance(
    a: &UniversalSchematic,
    b: &UniversalSchematic,
    spec: &FingerprintSpec,
) -> f32 {
    footprint(a, spec).distance(&footprint(b, spec))
}

impl Signature {
    /// JSON: sorted dims, block count, and the token histogram.
    pub fn to_json(&self) -> String {
        let hist: serde_json::Map<String, serde_json::Value> = self
            .histogram
            .iter()
            .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
            .collect();
        serde_json::json!({
            "dims_sorted": self.dims_sorted,
            "count": self.count,
            "histogram": hist,
        })
        .to_string()
    }
}

pub fn fingerprint(schem: &UniversalSchematic, spec: &FingerprintSpec) -> Fingerprint {
    // Resolve the palette once — builds repeat blockstates heavily, so each
    // orbit element only tokenizes the distinct entries, not every cell.
    let mut palette: Vec<&BlockState> = Vec::new();
    let mut index: HashMap<&BlockState, usize> = HashMap::new();
    let mut cell_list: Vec<((i32, i32, i32), usize)> = Vec::new();
    for (pos, block) in schem.iter_blocks() {
        let id = *index.entry(block).or_insert_with(|| {
            palette.push(block);
            palette.len() - 1
        });
        cell_list.push(((pos.x, pos.y, pos.z), id));
    }

    // Block-entity NBT tokens per occupied cell, computed once (they are
    // rotation-invariant). Serializations are memoized by NBT identity so
    // template-shared entities hash once. Positions come from the store keys
    // (via `get_block_entity`), never from `BlockEntity::position`, which is
    // stale for template-shared entities.
    let mut nbt_memo: HashMap<*const NbtMap, Option<Token>> = HashMap::new();
    let mut nbt_by_pos: HashMap<(i32, i32, i32), Token> = HashMap::new();
    // Rotation-tolerant specs must not hash facing/rotation fields (see
    // `is_directional_key`).
    let ignore_directional = spec.symmetry != Symmetry::None;
    if spec.block_entities {
        for (pos, _) in &cell_list {
            if let Some(be) = schem.get_block_entity(crate::block_position::BlockPosition {
                x: pos.0,
                y: pos.1,
                z: pos.2,
            }) {
                let key = std::sync::Arc::as_ptr(&be.nbt);
                let tok = nbt_memo
                    .entry(key)
                    .or_insert_with(|| stable_nbt_token(&be.nbt, ignore_directional))
                    .clone();
                if let Some(t) = tok {
                    nbt_by_pos.insert(*pos, t);
                }
            }
        }
    }

    let mut best: Option<Vec<u8>> = None;
    for g in spec.symmetry.elements() {
        let toks: Vec<Option<Token>> = palette
            .iter()
            .map(|b| spec.blocks.tokenize(&g.apply_block(b)))
            .collect();
        let mut cells: Vec<((i32, i32, i32), Token)> = Vec::with_capacity(cell_list.len());
        for (pos, id) in &cell_list {
            if let Some(tok) = &toks[*id] {
                let tok = match nbt_by_pos.get(pos) {
                    Some(nbt) => token_with_nbt(tok, nbt),
                    None => tok.clone(),
                };
                cells.push((g.apply_pos(*pos), tok));
            }
        }
        if cells.is_empty() {
            continue;
        }
        let mn = cells
            .iter()
            .fold((i32::MAX, i32::MAX, i32::MAX), |m, (p, _)| {
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
mod block_entity_tests {
    use super::*;
    use crate::block_entity::BlockEntity;
    use crate::block_position::BlockPosition;
    use crate::fingerprint::testgen::filled_box;
    use crate::utils::NbtValue;

    fn signed_at(text: &str, at: (i32, i32, i32)) -> UniversalSchematic {
        let mut s = filled_box(at, (at.0 + 1, at.1, at.2), "minecraft:oak_sign");
        s.set_block_entity(
            BlockPosition {
                x: at.0,
                y: at.1,
                z: at.2,
            },
            BlockEntity::new("minecraft:oak_sign".to_string(), at)
                .with_nbt_data("Text1".to_string(), NbtValue::String(text.to_string())),
        );
        s
    }

    fn signed(text: &str) -> UniversalSchematic {
        signed_at(text, (0, 0, 0))
    }

    #[test]
    fn differing_block_entity_nbt_changes_the_fingerprint() {
        let spec = FingerprintSpec::exact();
        let a = signed("Alpha");
        let b = signed("Beta");
        assert_ne!(
            fingerprint(&a, &spec),
            fingerprint(&b, &spec),
            "sign text must be part of the fingerprint"
        );
    }

    #[test]
    fn identical_block_entities_share_a_fingerprint() {
        let spec = FingerprintSpec::exact();
        let a = signed("Alpha");
        let b = signed("Alpha");
        assert_eq!(fingerprint(&a, &spec), fingerprint(&b, &spec));
    }

    #[test]
    fn block_entity_fingerprint_is_translation_invariant() {
        let spec = FingerprintSpec::exact();
        let a = signed("Alpha");
        let b = signed_at("Alpha", (13, 2, -7));
        assert_eq!(
            fingerprint(&a, &spec),
            fingerprint(&b, &spec),
            "translated copy (block entities included) must fingerprint equal"
        );
    }

    /// Build a one-block sign carrying an arbitrary single NBT key.
    fn sign_with_key(key: &str, value: &str) -> UniversalSchematic {
        let at = (0, 0, 0);
        let mut s = filled_box(at, (1, 0, 0), "minecraft:oak_sign");
        s.set_block_entity(
            BlockPosition { x: 0, y: 0, z: 0 },
            BlockEntity::new("minecraft:oak_sign".to_string(), at)
                .with_nbt_data(key.to_string(), NbtValue::String(value.to_string())),
        );
        s
    }

    #[test]
    fn fuzzy_presets_ignore_block_entity_nbt() {
        // The fuzzy presets deliberately ignore block identity; making them
        // sensitive to sign text or chest loot would defeat their purpose.
        let a = signed("Alpha");
        let b = signed("Beta");
        for (name, spec) in [
            ("shape", FingerprintSpec::shape()),
            ("structural", FingerprintSpec::structural()),
            ("redstone_survival", FingerprintSpec::redstone_survival()),
        ] {
            assert!(!spec.block_entities, "{name} must not fold NBT");
            assert_eq!(
                fingerprint(&a, &spec),
                fingerprint(&b, &spec),
                "{name}: differing sign text must NOT change a fuzzy fingerprint"
            );
        }
        // ...while the exact preset still distinguishes them.
        let exact = FingerprintSpec::exact();
        assert!(exact.block_entities);
        assert_ne!(fingerprint(&a, &exact), fingerprint(&b, &exact));
    }

    #[test]
    fn rotation_tolerant_specs_ignore_directional_nbt() {
        // Under a rotation-tolerant symmetry the block token is rotated by the
        // group element but the NBT is not, so facing/rotation keys must be
        // excluded or a genuinely rotated copy fails to match itself.
        let rot = FingerprintSpec::custom(Symmetry::YawMirror, BlockPolicy::Exact)
            .with_block_entities(true);
        let a = sign_with_key("Rotation", "4");
        let b = sign_with_key("Rotation", "12");
        assert_eq!(
            fingerprint(&a, &rot),
            fingerprint(&b, &rot),
            "rotation-tolerant spec must ignore directional NBT keys"
        );
        // Non-directional content is still distinguished under that spec.
        let c = sign_with_key("Text1", "Alpha");
        let d = sign_with_key("Text1", "Beta");
        assert_ne!(
            fingerprint(&c, &rot),
            fingerprint(&d, &rot),
            "non-directional NBT is still content"
        );
        // With no symmetry there is nothing to be invariant to, so the
        // rotation is real content and must be kept.
        let none =
            FingerprintSpec::custom(Symmetry::None, BlockPolicy::Exact).with_block_entities(true);
        assert_ne!(
            fingerprint(&a, &none),
            fingerprint(&b, &none),
            "Symmetry::None must keep directional NBT"
        );
    }

    #[test]
    fn token_has_nbt_requires_a_well_formed_marker() {
        let base = Token::from("minecraft:stone");
        let nbt = Token::from("0123456789abcdef");
        assert!(token_has_nbt(&token_with_nbt(&base, &nbt)));
        assert!(!token_has_nbt(&base));
        // A block token that merely CONTAINS the marker must not be misread:
        // the suffix has to be exactly the fixed-width hex hash.
        assert!(!token_has_nbt(&Token::from("modid:weird#nbt:block")));
        assert!(!token_has_nbt(&Token::from("foo#nbt:")));
        assert!(!token_has_nbt(&Token::from("foo#nbt:0123456789abcde")));
        assert!(!token_has_nbt(&Token::from("foo#nbt:0123456789abcdefg")));
        assert!(!token_has_nbt(&Token::from("foo#nbt:zzzzzzzzzzzzzzzz")));
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
        assert_eq!(
            BlockPolicy::Exact.tokenize(&BlockState::new("minecraft:air")),
            None
        );
    }

    #[test]
    fn presets_build() {
        let _ = FingerprintSpec::redstone_computational();
        let _ = FingerprintSpec::structural();
        let _ = FingerprintSpec::exact();
        let _ = FingerprintSpec::shape();
        let _ = FingerprintSpec::redstone_survival();
    }

    #[test]
    fn preset_names_resolve() {
        for name in FingerprintSpec::PRESETS {
            assert!(FingerprintSpec::from_preset(name).is_some(), "{name}");
        }
        assert!(FingerprintSpec::from_preset("nope").is_none());
    }

    #[test]
    fn fingerprint_hex_and_dup() {
        use crate::fingerprint::testgen::filled_box;
        let spec = FingerprintSpec::structural();
        let a = filled_box((0, 0, 0), (2, 2, 2), "minecraft:stone");
        let b = filled_box((10, 0, 0), (12, 2, 2), "minecraft:stone"); // translated copy
        let hx = fingerprint(&a, &spec).to_hex();
        assert_eq!(hx.len(), 32);
        assert!(hx.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(
            is_duplicate(&a, &b, &spec),
            "translation-invariant structural dup"
        );
    }

    #[test]
    fn signature_json_and_footprint_distance() {
        use crate::fingerprint::testgen::filled_box;
        let spec = FingerprintSpec::structural();
        let a = filled_box((0, 0, 0), (2, 2, 2), "minecraft:stone");
        let b = filled_box((10, 0, 0), (12, 2, 2), "minecraft:stone");
        let sig: serde_json::Value = serde_json::from_str(&signature(&a, &spec).to_json()).unwrap();
        assert!(sig["count"].as_u64().is_some());
        assert!(sig["histogram"].is_object());
        // identical occupancy shape (translated) → ~zero footprint distance
        assert!(footprint_distance(&a, &b, &spec) < 1e-3);
    }
}

#[cfg(test)]
mod smoke {
    #[test]
    fn module_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
