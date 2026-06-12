//! Structural diff + edit distance between two builds. See
//! docs/superpowers/specs/2026-06-01-diff-engine-design.md

pub mod align;
#[cfg(feature = "meshing")]
mod overlay;
pub mod regions;

#[cfg(feature = "meshing")]
pub use overlay::{OverlayError, OverlayOptions};

use crate::block_state::BlockState;
use crate::fingerprint::classifier::Token;
use crate::fingerprint::symmetry::{RigidOp, Symmetry};
use crate::fingerprint::FingerprintSpec;

pub type IVec3 = (i32, i32, i32);

#[derive(Clone, Debug)]
pub struct Transform {
    pub rotate: RigidOp,
    pub translate: IVec3,
}

#[derive(Clone, Copy, Debug)]
pub struct CostModel {
    pub add: u32,
    pub delete: u32,
    pub change: u32,
    pub swap: u32,
}
impl Default for CostModel {
    fn default() -> Self {
        Self {
            add: 1,
            delete: 1,
            change: 1,
            swap: 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AlignOptions {
    pub anchor_max_count: usize,
    pub fft_fallback: bool,
    pub ambiguous_margin: f32,
}
impl Default for AlignOptions {
    fn default() -> Self {
        Self {
            anchor_max_count: 64,
            fft_fallback: true,
            ambiguous_margin: 1.5,
        }
    }
}

#[derive(Clone)]
pub struct DiffSpec {
    pub fingerprint: FingerprintSpec,
    pub costs: CostModel,
    pub align: AlignOptions,
}
/// Optional tweaks layered on top of a preset. `None` keeps the preset's value.
#[derive(Clone, Debug, Default)]
pub struct SpecOverrides {
    pub cost_add: Option<u32>,
    pub cost_delete: Option<u32>,
    pub cost_change: Option<u32>,
    pub cost_swap: Option<u32>,
    pub symmetry: Option<Symmetry>,
}

impl DiffSpec {
    pub fn from_preset(fingerprint: FingerprintSpec) -> Self {
        Self {
            fingerprint,
            costs: CostModel::default(),
            align: AlignOptions::default(),
        }
    }

    /// Resolve a preset by name (see [`FingerprintSpec::from_preset`]). None on
    /// unknown preset.
    pub fn from_preset_name(name: &str) -> Option<Self> {
        FingerprintSpec::from_preset(name).map(Self::from_preset)
    }

    /// Resolve a preset by name, then apply `overrides`. None on unknown preset.
    pub fn resolve(preset: &str, ov: &SpecOverrides) -> Option<Self> {
        let mut spec = Self::from_preset_name(preset)?;
        if let Some(v) = ov.cost_add {
            spec.costs.add = v;
        }
        if let Some(v) = ov.cost_delete {
            spec.costs.delete = v;
        }
        if let Some(v) = ov.cost_change {
            spec.costs.change = v;
        }
        if let Some(v) = ov.cost_swap {
            spec.costs.swap = v;
        }
        if let Some(sym) = ov.symmetry {
            spec.fingerprint.symmetry = sym;
        }
        Some(spec)
    }
}

pub struct Diff {
    pub transform: Transform,
    pub distance: u32,
    pub added: Vec<(IVec3, BlockState)>,
    pub removed: Vec<(IVec3, BlockState)>,
    pub changed: Vec<(IVec3, BlockState, BlockState)>,
    /// Cells covered by a palette swap (positions kept for visualization).
    pub swapped: Vec<(IVec3, BlockState, BlockState)>,
    pub palette_swaps: Vec<(Token, Token)>,
    pub support: f32,
}

/// A tokenized cell: B-frame position, its token, and the (rotated) blockstate.
pub(crate) type Cell = (IVec3, Token, BlockState);

use std::collections::HashMap;

use crate::universal_schematic::UniversalSchematic;

/// Tokenize a build under a rotation: B-frame positions + tokens + rotated blocks.
pub(crate) fn cells(schem: &UniversalSchematic, g: &RigidOp, spec: &FingerprintSpec) -> Vec<Cell> {
    schem
        .iter_blocks()
        .filter_map(|(pos, b)| {
            let rb = g.apply_block(b);
            spec.blocks
                .tokenize(&rb)
                .map(|tok| (g.apply_pos((pos.x, pos.y, pos.z)), tok, rb))
        })
        .collect()
}

/// Raw cell diff (before palette-swap collapsing).
pub(crate) struct RawDiff {
    pub added: Vec<(IVec3, BlockState)>,
    pub removed: Vec<(IVec3, BlockState)>,
    pub changed: Vec<(IVec3, BlockState, BlockState)>,
    pub matched: usize,
}

/// Compare A-cells (shifted by `t`) against B-cells.
pub(crate) fn compare(a: &[Cell], t: IVec3, b: &[Cell]) -> RawDiff {
    let bmap: HashMap<IVec3, &Cell> = b.iter().map(|c| (c.0, c)).collect();
    let mut amap: HashMap<IVec3, &Cell> = HashMap::new();
    for c in a {
        amap.insert((c.0 .0 + t.0, c.0 .1 + t.1, c.0 .2 + t.2), c);
    }
    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut changed = Vec::new();
    let mut matched = 0usize;
    for (p, ac) in &amap {
        match bmap.get(p) {
            Some(bc) => {
                if ac.1 == bc.1 {
                    matched += 1;
                } else {
                    changed.push((*p, ac.2.clone(), bc.2.clone()));
                }
            }
            None => removed.push((*p, ac.2.clone())),
        }
    }
    for (p, bc) in &bmap {
        if !amap.contains_key(p) {
            added.push((*p, bc.2.clone()));
        }
    }
    RawDiff {
        added,
        removed,
        changed,
        matched,
    }
}

/// Collapse changed cells whose A-token maps consistently (≥80%) to one B-token
/// into palette swaps. Returns (swaps, residual_changes).
/// (palette swaps, residual changes, cells covered by a swap).
type SwapSplit = (
    Vec<(Token, Token)>,
    Vec<(IVec3, BlockState, BlockState)>,
    Vec<(IVec3, BlockState, BlockState)>,
);

pub(crate) fn collapse_swaps(
    a: &[Cell],
    t: IVec3,
    b: &[Cell],
    changed: Vec<(IVec3, BlockState, BlockState)>,
) -> SwapSplit {
    let bmap: HashMap<IVec3, Token> = b.iter().map(|c| (c.0, c.1.clone())).collect();
    let mut a_tok: HashMap<IVec3, Token> = HashMap::new();
    for c in a {
        a_tok.insert((c.0 .0 + t.0, c.0 .1 + t.1, c.0 .2 + t.2), c.1.clone());
    }
    let mut confusion: HashMap<Token, HashMap<Token, usize>> = HashMap::new();
    for (p, _, _) in &changed {
        if let (Some(at), Some(bt)) = (a_tok.get(p), bmap.get(p)) {
            *confusion
                .entry(at.clone())
                .or_default()
                .entry(bt.clone())
                .or_default() += 1;
        }
    }
    let mut swaps = Vec::new();
    let mut swapped: std::collections::HashSet<(Token, Token)> = std::collections::HashSet::new();
    for (at, bts) in &confusion {
        let total: usize = bts.values().sum();
        if let Some((bt, &cnt)) = bts.iter().max_by_key(|(_, c)| **c) {
            if cnt * 100 >= total * 80 {
                swaps.push((at.clone(), bt.clone()));
                swapped.insert((at.clone(), bt.clone()));
            }
        }
    }
    let (residual, swapped_cells): (Vec<_>, Vec<_>) =
        changed
            .into_iter()
            .partition(|(p, _, _)| match (a_tok.get(p), bmap.get(p)) {
                (Some(at), Some(bt)) => !swapped.contains(&(at.clone(), bt.clone())),
                _ => true,
            });
    (swaps, residual, swapped_cells)
}

fn diff_for_rotation(
    a: &UniversalSchematic,
    b_cells: &[Cell],
    g: &RigidOp,
    spec: &DiffSpec,
) -> Diff {
    let a_cells = cells(a, g, &spec.fingerprint);
    let (mut t, margin) = crate::diff::align::hough_translate(&a_cells, b_cells, &spec.align);
    if spec.align.fft_fallback && margin < spec.align.ambiguous_margin {
        if let Some(ft) = crate::diff::align::fft_translate(&a_cells, b_cells, 96) {
            // keep whichever offset yields fewer residual changes
            let score = |tt: IVec3| {
                let r = compare(&a_cells, tt, b_cells);
                r.added.len() + r.removed.len() + r.changed.len()
            };
            if score(ft) < score(t) {
                t = ft;
            }
        }
    }
    diff_at(&a_cells, b_cells, g, t, spec)
}

/// Diff A-cells against B-cells at a FIXED translation `t` (no alignment
/// search). Used by `diff_for_rotation` after choosing `t`, and by
/// `diff_identity` with `t = (0, 0, 0)`.
fn diff_at(a_cells: &[Cell], b_cells: &[Cell], g: &RigidOp, t: IVec3, spec: &DiffSpec) -> Diff {
    let raw = compare(a_cells, t, b_cells);
    let (swaps, changed, swapped) = collapse_swaps(a_cells, t, b_cells, raw.changed);
    let max_cells = a_cells.len().max(b_cells.len()).max(1);
    let distance = spec.costs.add * raw.added.len() as u32
        + spec.costs.delete * raw.removed.len() as u32
        + spec.costs.change * changed.len() as u32
        + spec.costs.swap * swaps.len() as u32;
    Diff {
        transform: Transform {
            rotate: g.clone(),
            translate: t,
        },
        distance,
        added: raw.added,
        removed: raw.removed,
        changed,
        swapped,
        palette_swaps: swaps,
        support: raw.matched as f32 / max_cells as f32,
    }
}

pub fn diff(a: &UniversalSchematic, b: &UniversalSchematic, spec: &DiffSpec) -> Diff {
    let b_cells = cells(b, &RigidOp::identity(), &spec.fingerprint);
    let mut best: Option<Diff> = None;
    for g in spec.fingerprint.symmetry.elements() {
        let d = diff_for_rotation(a, &b_cells, &g, spec);
        if best
            .as_ref()
            .map(|bd| d.distance < bd.distance)
            .unwrap_or(true)
        {
            best = Some(d);
        }
    }
    best.unwrap_or_else(|| Diff {
        transform: Transform {
            rotate: RigidOp::identity(),
            translate: (0, 0, 0),
        },
        distance: 0,
        added: Vec::new(),
        removed: Vec::new(),
        changed: Vec::new(),
        swapped: Vec::new(),
        palette_swaps: Vec::new(),
        support: 0.0,
    })
}

/// Diff two schematics that share an absolute coordinate frame: no
/// rotation/symmetry search AND no translation alignment — the diff is
/// computed at the fixed identity transform, so added/removed/changed are
/// reported in absolute coordinates (used by world_stream's per-chunk diff,
/// where both worlds use the same world coordinates).
pub fn diff_identity(a: &UniversalSchematic, b: &UniversalSchematic, spec: &DiffSpec) -> Diff {
    let id = RigidOp::identity();
    let a_cells = cells(a, &id, &spec.fingerprint);
    let b_cells = cells(b, &id, &spec.fingerprint);
    diff_at(&a_cells, &b_cells, &id, (0, 0, 0), spec)
}

impl Diff {
    /// B's added blocks (B-frame positions).
    pub fn added(&self) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("diff-added".to_string());
        for (p, b) in &self.added {
            s.set_block(p.0, p.1, p.2, b);
        }
        s
    }

    /// A's removed blocks (B-frame positions).
    pub fn removed(&self) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("diff-removed".to_string());
        for (p, b) in &self.removed {
            s.set_block(p.0, p.1, p.2, b);
        }
        s
    }

    /// B's version at changed cells.
    pub fn changed(&self) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("diff-changed".to_string());
        for (p, _a, b) in &self.changed {
            s.set_block(p.0, p.1, p.2, b);
        }
        s
    }

    /// B's version at cells covered by a palette swap.
    pub fn swapped(&self) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("diff-swapped".to_string());
        for (p, _a, b) in &self.swapped {
            s.set_block(p.0, p.1, p.2, b);
        }
        s
    }

    /// Colored marker overlay: added=lime, removed=red, changed=yellow,
    /// palette-swapped=light blue.
    pub fn markers(&self) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("diff-markers".to_string());
        let lime = BlockState::new("minecraft:lime_stained_glass");
        let red = BlockState::new("minecraft:red_stained_glass");
        let yellow = BlockState::new("minecraft:yellow_stained_glass");
        let blue = BlockState::new("minecraft:light_blue_stained_glass");
        for (p, _) in &self.added {
            s.set_block(p.0, p.1, p.2, &lime);
        }
        for (p, _) in &self.removed {
            s.set_block(p.0, p.1, p.2, &red);
        }
        for (p, _, _) in &self.changed {
            s.set_block(p.0, p.1, p.2, &yellow);
        }
        for (p, _, _) in &self.swapped {
            s.set_block(p.0, p.1, p.2, &blue);
        }
        s
    }

    /// JSON summary: transform, distance, support, swaps, and change regions.
    /// Lossless JSON: everything needed to reconstruct the Diff (transform +
    /// every added/removed/changed/swapped cell with position and blockstate).
    pub fn to_json(&self) -> String {
        let cell2 = |(p, b): &(IVec3, BlockState)| serde_json::json!({ "pos": [p.0, p.1, p.2], "block": b.to_string() });
        let cell3 = |(p, from, to): &(IVec3, BlockState, BlockState)| {
            serde_json::json!({
                "pos": [p.0, p.1, p.2],
                "from": from.to_string(),
                "to": to.to_string(),
            })
        };
        serde_json::json!({
            "schema": "nucleation.diff/1",
            "distance": self.distance,
            "support": self.support,
            "transform": {
                "rotate": serde_json::to_value(&self.transform.rotate)
                    .unwrap_or(serde_json::Value::Null),
                "translate": [self.transform.translate.0, self.transform.translate.1, self.transform.translate.2],
            },
            "added": self.added.iter().map(cell2).collect::<Vec<_>>(),
            "removed": self.removed.iter().map(cell2).collect::<Vec<_>>(),
            "changed": self.changed.iter().map(cell3).collect::<Vec<_>>(),
            "swapped": self.swapped.iter().map(cell3).collect::<Vec<_>>(),
            "palette_swaps": self
                .palette_swaps
                .iter()
                .map(|(a, b)| [a.to_string(), b.to_string()])
                .collect::<Vec<_>>(),
        })
        .to_string()
    }

    /// Compact human summary (counts + regions + swaps), no per-cell data.
    pub fn summary_json(&self) -> String {
        let regs = crate::diff::regions::regions(self);
        let region_json: Vec<serde_json::Value> = regs
            .iter()
            .map(|r| {
                serde_json::json!({
                    "min": [r.min.0, r.min.1, r.min.2],
                    "max": [r.max.0, r.max.1, r.max.2],
                    "kind": format!("{:?}", r.kind),
                    "count": r.count,
                })
            })
            .collect();
        serde_json::json!({
            "distance": self.distance,
            "support": self.support,
            "translate": [self.transform.translate.0, self.transform.translate.1, self.transform.translate.2],
            "counts": { "added": self.added.len(), "removed": self.removed.len(),
                        "changed": self.changed.len(), "swapped": self.swapped.len() },
            "swaps": self.palette_swaps.iter().map(|(a, b)| [a.to_string(), b.to_string()]).collect::<Vec<_>>(),
            "regions": region_json,
        })
        .to_string()
    }

    /// Reconstruct a Diff from lossless [`to_json`](Self::to_json) output.
    pub fn from_json(s: &str) -> Result<Self, DiffError> {
        let v: serde_json::Value = serde_json::from_str(s).map_err(|e| DiffError(e.to_string()))?;
        let err = |m: &str| DiffError(m.to_string());

        let ivec = |a: &serde_json::Value| -> Result<IVec3, DiffError> {
            let p = a
                .get("pos")
                .and_then(|p| p.as_array())
                .ok_or_else(|| err("cell missing pos"))?;
            if p.len() != 3 {
                return Err(err("pos must be [x,y,z]"));
            }
            Ok((
                p[0].as_i64().ok_or_else(|| err("pos x"))? as i32,
                p[1].as_i64().ok_or_else(|| err("pos y"))? as i32,
                p[2].as_i64().ok_or_else(|| err("pos z"))? as i32,
            ))
        };
        let block = |a: &serde_json::Value, key: &str| -> Result<BlockState, DiffError> {
            let s = a
                .get(key)
                .and_then(|b| b.as_str())
                .ok_or_else(|| err("cell missing block"))?;
            BlockState::from_block_string(s).map_err(DiffError)
        };
        let arr = |key: &str| {
            v.get(key)
                .and_then(|x| x.as_array())
                .cloned()
                .unwrap_or_default()
        };

        let mut added = Vec::new();
        for c in arr("added") {
            added.push((ivec(&c)?, block(&c, "block")?));
        }
        let mut removed = Vec::new();
        for c in arr("removed") {
            removed.push((ivec(&c)?, block(&c, "block")?));
        }
        let mut changed = Vec::new();
        for c in arr("changed") {
            changed.push((ivec(&c)?, block(&c, "from")?, block(&c, "to")?));
        }
        let mut swapped = Vec::new();
        for c in arr("swapped") {
            swapped.push((ivec(&c)?, block(&c, "from")?, block(&c, "to")?));
        }

        let tr = v.get("transform").ok_or_else(|| err("missing transform"))?;
        let rotate: RigidOp = serde_json::from_value(
            tr.get("rotate")
                .cloned()
                .ok_or_else(|| err("missing rotate"))?,
        )
        .map_err(|e| DiffError(e.to_string()))?;
        let t = tr
            .get("translate")
            .and_then(|t| t.as_array())
            .ok_or_else(|| err("missing translate"))?;
        let translate = (
            t.first().and_then(|x| x.as_i64()).unwrap_or(0) as i32,
            t.get(1).and_then(|x| x.as_i64()).unwrap_or(0) as i32,
            t.get(2).and_then(|x| x.as_i64()).unwrap_or(0) as i32,
        );

        let palette_swaps = arr("palette_swaps")
            .iter()
            .filter_map(|p| {
                let a = p.get(0)?.as_str()?;
                let b = p.get(1)?.as_str()?;
                Some((a.into(), b.into()))
            })
            .collect();

        Ok(Diff {
            transform: Transform { rotate, translate },
            distance: v.get("distance").and_then(|d| d.as_u64()).unwrap_or(0) as u32,
            support: v.get("support").and_then(|s| s.as_f64()).unwrap_or(0.0) as f32,
            added,
            removed,
            changed,
            swapped,
            palette_swaps,
        })
    }
}

/// Error parsing a diff from JSON.
#[derive(Debug)]
pub struct DiffError(pub String);
impl std::fmt::Display for DiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for DiffError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::testgen::{edited, filled_box};
    use crate::fingerprint::FingerprintSpec;

    fn nonair(s: &UniversalSchematic) -> usize {
        s.iter_blocks()
            .filter(|(_, b)| b.get_name() != "minecraft:air")
            .count()
    }

    #[test]
    fn diff_spec_resolves_with_overrides() {
        let base = DiffSpec::from_preset_name("exact").unwrap();
        let ov = SpecOverrides {
            cost_swap: Some(7),
            ..Default::default()
        };
        let spec = DiffSpec::resolve("exact", &ov).unwrap();
        assert_eq!(spec.costs.swap, 7);
        assert_eq!(spec.costs.add, base.costs.add); // untouched
        assert!(DiffSpec::resolve("nope", &ov).is_none());
    }

    #[test]
    fn diff_json_round_trips() {
        let a = filled_box((0, 0, 0), (4, 0, 4), "minecraft:stone");
        let (b, _) = edited(&a, 3);
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);

        let json = d.to_json();
        let back = Diff::from_json(&json).expect("parse");
        assert_eq!(back.distance, d.distance);
        assert_eq!(back.transform.translate, d.transform.translate);
        assert_eq!(back.added.len(), d.added.len());
        assert_eq!(back.removed.len(), d.removed.len());
        assert_eq!(back.changed.len(), d.changed.len());
        assert_eq!(back.swapped.len(), d.swapped.len());
        // the reconstructed Diff can still project:
        assert_eq!(back.markers().total_blocks(), d.markers().total_blocks());
    }

    #[test]
    fn diff_recovers_edits_same_frame() {
        let a = filled_box((0, 0, 0), (4, 2, 2), "minecraft:stone");
        let (b, (adds, removes, changes)) = edited(&a, 6);
        // exact: every distinct blockstate is its own token, so the generator's
        // glass adds/changes are all visible (structural would ignore glass).
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);
        assert_eq!(d.added.len(), adds, "adds");
        assert_eq!(d.removed.len(), removes, "removes");
        assert_eq!(d.changed.len(), changes, "changes");
        assert_eq!(
            d.distance,
            (adds + removes + changes) as u32,
            "unit-cost distance"
        );
    }

    #[test]
    fn identical_builds_have_zero_distance() {
        let a = filled_box((0, 0, 0), (3, 1, 1), "minecraft:stone");
        let spec = DiffSpec::from_preset(FingerprintSpec::structural());
        let d = diff(&a, &a, &spec);
        assert_eq!(d.distance, 0);
        assert!(d.added.is_empty() && d.removed.is_empty() && d.changed.is_empty());
    }

    #[test]
    fn repalette_is_one_swap_under_exact() {
        use crate::fingerprint::testgen::repalette;
        let a = filled_box((0, 0, 0), (4, 0, 4), "minecraft:stone");
        let b = repalette(&a, "minecraft:stone", "minecraft:cobblestone");
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);
        assert_eq!(d.palette_swaps.len(), 1, "one swap");
        assert_eq!(d.distance, 1, "one swap op, not 25 changes");
        assert!(d.changed.is_empty(), "all changes explained by the swap");
    }

    #[test]
    fn repalette_is_free_under_structural() {
        use crate::fingerprint::testgen::repalette;
        let a = filled_box((0, 0, 0), (4, 0, 4), "minecraft:stone");
        let b = repalette(&a, "minecraft:stone", "minecraft:cobblestone");
        let spec = DiffSpec::from_preset(FingerprintSpec::structural());
        let d = diff(&a, &b, &spec);
        assert_eq!(d.distance, 0);
    }

    #[test]
    fn projections_match_change_sets() {
        let a = filled_box((0, 0, 0), (4, 2, 2), "minecraft:stone");
        let (b, _) = edited(&a, 6);
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);
        assert_eq!(nonair(&d.added()), d.added.len());
        assert_eq!(
            nonair(&d.markers()),
            d.added.len() + d.removed.len() + d.changed.len()
        );
    }

    #[test]
    fn diff_aligns_a_featureless_translated_box() {
        use crate::fingerprint::testgen::translated;
        // 120-cell stone box: "solid" token exceeds the anchor threshold (64),
        // so Hough has no anchors → must fall back to FFT.
        let a = filled_box((0, 0, 0), (5, 4, 3), "minecraft:stone");
        let b = translated(&a, (12, 2, -6));
        let spec = DiffSpec::from_preset(FingerprintSpec::structural());
        let d = diff(&a, &b, &spec);
        assert_eq!(d.distance, 0, "featureless box still aligns via FFT");
        assert_eq!(d.transform.translate, (12, 2, -6));
    }

    #[test]
    fn diff_aligns_a_rotated_build() {
        use crate::fingerprint::testgen::rotated_y;
        let a = filled_box((0, 0, 0), (5, 0, 2), "minecraft:stone");
        let b = rotated_y(&a, 90);
        // structural uses YawMirror, so a 90° rebuild should diff to zero.
        let spec = DiffSpec::from_preset(FingerprintSpec::structural());
        let d = diff(&a, &b, &spec);
        assert_eq!(d.distance, 0, "rotated rebuild = no edits");
    }

    #[test]
    fn diff_aligns_a_translated_build() {
        use crate::fingerprint::testgen::translated;
        let mut a = filled_box((0, 0, 0), (5, 0, 5), "minecraft:stone");
        a.set_block(2, 0, 2, &BlockState::new("minecraft:repeater"));
        let b = translated(&a, (9, 0, -4));
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);
        assert_eq!(d.transform.translate, (9, 0, -4), "recovers the shift");
        assert_eq!(d.distance, 0, "pure translation = no edits");
    }

    #[test]
    fn json_parses_and_has_regions() {
        let a = filled_box((0, 0, 0), (3, 1, 1), "minecraft:stone");
        let (b, _) = edited(&a, 3);
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);
        // Lossless to_json: distance + per-cell arrays (no top-level regions).
        let v: serde_json::Value = serde_json::from_str(&d.to_json()).unwrap();
        assert!(v["distance"].as_u64().is_some());
        assert!(v["added"].as_array().is_some());
        assert!(v["changed"].as_array().is_some());
        // Regions live in the compact summary now.
        let s: serde_json::Value = serde_json::from_str(&d.summary_json()).unwrap();
        assert!(s["regions"].as_array().is_some());
    }
}

#[cfg(test)]
mod smoke {
    #[test]
    fn compiles() {
        assert_eq!(2 + 2, 4);
    }
}
