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
    /// Minimum percentage (0–100) of a source token's changes that must map to a
    /// single dominant target before they collapse into a palette swap.
    pub swap_dominance_pct: u32,
}
impl Default for CostModel {
    fn default() -> Self {
        Self {
            add: 1,
            delete: 1,
            change: 1,
            swap: 1,
            swap_dominance_pct: 80,
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
    /// Override the palette-swap dominance threshold (percent, 0–100).
    pub swap_dominance_pct: Option<u32>,
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
        if let Some(v) = ov.swap_dominance_pct {
            spec.costs.swap_dominance_pct = v;
        }
        if let Some(sym) = ov.symmetry {
            spec.fingerprint.symmetry = sym;
        }
        Some(spec)
    }
}

pub struct Diff {
    pub transform: Transform,
    pub distance: u64,
    pub added: Vec<(IVec3, BlockState)>,
    pub removed: Vec<(IVec3, BlockState)>,
    pub changed: Vec<(IVec3, BlockState, BlockState)>,
    /// Cells covered by a palette swap (positions kept for visualization).
    pub swapped: Vec<(IVec3, BlockState, BlockState)>,
    pub palette_swaps: Vec<(Token, Token)>,
    /// Alignment confidence in `[0, 1]`: the fraction of cells that found a
    /// counterpart at the chosen transform, i.e.
    /// `(matched + changed + swapped) / max(|A|, |B|)`. Re-paletted or otherwise
    /// edited cells still count as aligned; only `added`/`removed` (cells with no
    /// counterpart) are unaligned. A pure re-palette therefore scores `1.0`.
    pub support: f32,
}

/// A tokenized cell: B-frame position, its token, and the (rotated) blockstate.
pub(crate) type Cell = (IVec3, Token, BlockState);

use std::collections::HashMap;

use crate::universal_schematic::UniversalSchematic;

/// Tokenize a build under a rotation: B-frame positions + tokens + rotated blocks.
pub(crate) fn cells(schem: &UniversalSchematic, g: &RigidOp, spec: &FingerprintSpec) -> Vec<Cell> {
    // Memoize block-entity NBT serializations by NBT identity so
    // template-shared entities are hashed once, not once per cell.
    let mut nbt_memo: HashMap<*const crate::utils::NbtMap, Option<Token>> = HashMap::new();
    // Only content-exact specs fold block-entity NBT into the token; the fuzzy
    // presets deliberately ignore block identity. Rotation-tolerant specs also
    // drop facing/rotation NBT keys, which the group element does not rotate.
    let fold_nbt = spec.block_entities;
    let ignore_directional = spec.symmetry != crate::fingerprint::symmetry::Symmetry::None;
    schem
        .iter_blocks()
        // Air is absence, not a block: never tokenize it as a present cell in the
        // diff path. Done here (rather than per-policy) so it is preset-independent
        // and consistent across all bindings.
        .filter(|(_, b)| !crate::fingerprint::is_air(b.get_name()))
        .filter_map(|(pos, b)| {
            let rb = g.apply_block(b);
            spec.blocks.tokenize(&rb).map(|tok| {
                // Cells differing only in tile-entity NBT (sign text, chest
                // contents) must not compare equal: fold the entity's stable
                // NBT token into the cell token.
                let tok = match schem.get_block_entity(pos).filter(|_| fold_nbt) {
                    Some(be) => {
                        let key = std::sync::Arc::as_ptr(&be.nbt);
                        match nbt_memo.entry(key).or_insert_with(|| {
                            crate::fingerprint::stable_nbt_token(&be.nbt, ignore_directional)
                        }) {
                            Some(nbt) => crate::fingerprint::token_with_nbt(&tok, nbt),
                            None => tok,
                        }
                    }
                    None => tok,
                };
                (g.apply_pos((pos.x, pos.y, pos.z)), tok, rb)
            })
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
    threshold_pct: u32,
) -> SwapSplit {
    let bmap: HashMap<IVec3, Token> = b.iter().map(|c| (c.0, c.1.clone())).collect();
    let mut a_tok: HashMap<IVec3, Token> = HashMap::new();
    for c in a {
        a_tok.insert((c.0 .0 + t.0, c.0 .1 + t.1, c.0 .2 + t.2), c.1.clone());
    }
    let mut confusion: HashMap<Token, HashMap<Token, usize>> = HashMap::new();
    for (p, _, _) in &changed {
        if let (Some(at), Some(bt)) = (a_tok.get(p), bmap.get(p)) {
            // Tokens carrying block-entity NBT are per-cell content edits, not
            // re-skins: never collapse them into a palette swap (their hashes
            // would otherwise each form a spurious 100%-dominant "swap").
            if crate::fingerprint::token_has_nbt(at) || crate::fingerprint::token_has_nbt(bt) {
                continue;
            }
            *confusion
                .entry(at.clone())
                .or_default()
                .entry(bt.clone())
                .or_default() += 1;
        }
    }
    let mut swaps = Vec::new();
    let mut swapped: std::collections::HashSet<(Token, Token)> = std::collections::HashSet::new();
    // Iterate sources in a deterministic order (HashMap iteration order is
    // randomized per instance, which would otherwise leak into `swaps`).
    let mut sources: Vec<(&Token, &HashMap<Token, usize>)> = confusion.iter().collect();
    sources.sort_by(|x, y| x.0.cmp(y.0));
    for (at, bts) in sources {
        let total: usize = bts.values().sum();
        // Dominant target: highest count; ties broken by lexicographically
        // smallest target token (deterministic).
        let mut targets: Vec<(&Token, usize)> = bts.iter().map(|(t, &c)| (t, c)).collect();
        targets.sort_by(|x, y| y.1.cmp(&x.1).then_with(|| x.0.cmp(y.0)));
        if let Some(&(bt, cnt)) = targets.first() {
            if cnt * 100 >= total * threshold_pct as usize {
                swaps.push((at.clone(), bt.clone()));
                swapped.insert((at.clone(), bt.clone()));
            }
        }
    }
    swaps.sort();
    let (residual, swapped_cells): (Vec<_>, Vec<_>) =
        changed
            .into_iter()
            .partition(|(p, _, _)| match (a_tok.get(p), bmap.get(p)) {
                (Some(at), Some(bt)) => !swapped.contains(&(at.clone(), bt.clone())),
                _ => true,
            });
    (swaps, residual, swapped_cells)
}

/// Number of unaligned/edited cells in a raw diff (lower = better fit).
fn raw_score(raw: &RawDiff) -> usize {
    raw.added.len() + raw.removed.len() + raw.changed.len()
}

/// Hard cap on the candidates in a single refinement grid (7³ — the
/// exhaustive grid for a ±3 window).
pub(crate) const REFINE_MAX_PASSES: usize = 343;

/// Hard cap on total `compare` passes for a whole `refine_offset` call: the
/// coarse grid plus, when that grid was strided, one exhaustive sweep of the
/// gap around its winner.
pub(crate) const REFINE_MAX_TOTAL_PASSES: usize = 2 * REFINE_MAX_PASSES;

/// Candidate offsets for [`refine_offset`]: `base` plus a symmetric grid of
/// ±`window` on each axis. Windows up to 3 are exhaustive; beyond that the
/// per-axis step grows (coarser grid, extremes always sampled) so the total
/// stays ≤ [`REFINE_MAX_PASSES`] instead of blowing up as `(2·window+1)³`.
/// Per-axis grid step for a given window: `ceil(window/3)`, so at most 3
/// interior samples per side plus the extreme → ≤7 values per axis.
pub(crate) fn refinement_step(window: usize) -> usize {
    window.div_ceil(3).max(1)
}

pub(crate) fn refinement_offsets(base: IVec3, window: usize) -> Vec<IVec3> {
    let w = window as i32;
    let step = refinement_step(window) as i32;
    let mut axis = vec![0i32];
    let mut v = step;
    while v < w {
        axis.push(v);
        axis.push(-v);
        v += step;
    }
    if w > 0 {
        axis.push(w);
        axis.push(-w);
    }
    let mut out = Vec::with_capacity(axis.len().pow(3));
    for dz in &axis {
        for dy in &axis {
            for dx in &axis {
                out.push((base.0 + dx, base.1 + dy, base.2 + dz));
            }
        }
    }
    out
}

/// Snap a coarse (downsampled-FFT) offset to the exact translation by a small
/// local search within ±`window` on each axis, minimizing the residual. The
/// search is bounded to [`REFINE_MAX_PASSES`] `compare` passes (see
/// [`refinement_offsets`]). Returns the chosen offset together with its
/// `RawDiff`, so the caller reuses that comparison instead of recomputing it.
fn refine_offset(
    a_cells: &[Cell],
    b_cells: &[Cell],
    base: IVec3,
    window: usize,
) -> (IVec3, RawDiff) {
    let mut best = base;
    let mut best_raw = compare(a_cells, base, b_cells);
    let mut best_r = raw_score(&best_raw);

    // Coarse pass over the (possibly strided) grid spanning ±window, then —
    // when the grid was strided — an exhaustive sweep of the gap around the
    // coarse winner, so the true offset can't hide between grid samples.
    let step = refinement_step(window);
    let sweep = |candidates: Vec<IVec3>,
                 anchor: IVec3,
                 best: &mut IVec3,
                 best_raw: &mut RawDiff,
                 best_r: &mut usize| {
        for t in candidates {
            if t == anchor {
                continue;
            }
            let raw = compare(a_cells, t, b_cells);
            let r = raw_score(&raw);
            if r < *best_r {
                *best_r = r;
                *best = t;
                *best_raw = raw;
            }
        }
    };

    sweep(
        refinement_offsets(base, window),
        base,
        &mut best,
        &mut best_raw,
        &mut best_r,
    );
    if step > 1 {
        let coarse = best;
        sweep(
            refinement_offsets(coarse, step - 1),
            coarse,
            &mut best,
            &mut best_raw,
            &mut best_r,
        );
    }
    (best, best_raw)
}

fn diff_for_rotation(
    a: &UniversalSchematic,
    b_cells: &[Cell],
    g: &RigidOp,
    spec: &DiffSpec,
) -> Diff {
    let a_cells = cells(a, g, &spec.fingerprint);
    let (mut t, margin) = crate::diff::align::hough_translate(&a_cells, b_cells, &spec.align);
    let mut raw = compare(&a_cells, t, b_cells);
    if spec.align.fft_fallback && margin < spec.align.ambiguous_margin {
        if let Some(ft) = crate::diff::align::fft_translate(&a_cells, b_cells, 96) {
            // Exact FFT fit: keep whichever offset yields fewer residual changes.
            let raw_ft = compare(&a_cells, ft, b_cells);
            if raw_score(&raw_ft) < raw_score(&raw) {
                t = ft;
                raw = raw_ft;
            }
        } else if let Some((coarse, stride)) =
            crate::diff::align::fft_translate_downsampled(&a_cells, b_cells, 96)
        {
            // Build too large for the exact grid: align coarsely on a pooled grid,
            // then refine within ±stride to recover the exact translation.
            let (refined, raw_r) = refine_offset(&a_cells, b_cells, coarse, stride);
            if raw_score(&raw_r) < raw_score(&raw) {
                t = refined;
                raw = raw_r;
            }
        }
    }
    // Reuse the winning offset's comparison — no extra `compare` pass in diff_at.
    diff_at_raw(raw, &a_cells, b_cells, g, t, spec)
}

/// Diff A-cells against B-cells at a FIXED translation `t` (no alignment
/// search). Used by `diff_for_rotation` after choosing `t`, and by
/// `diff_identity` with `t = (0, 0, 0)`.
fn diff_at(a_cells: &[Cell], b_cells: &[Cell], g: &RigidOp, t: IVec3, spec: &DiffSpec) -> Diff {
    let raw = compare(a_cells, t, b_cells);
    diff_at_raw(raw, a_cells, b_cells, g, t, spec)
}

/// Like [`diff_at`] but takes a precomputed [`RawDiff`] for `t`, avoiding a
/// redundant `compare` pass when the caller already has it.
fn diff_at_raw(
    raw: RawDiff,
    a_cells: &[Cell],
    b_cells: &[Cell],
    g: &RigidOp,
    t: IVec3,
    spec: &DiffSpec,
) -> Diff {
    let matched = raw.matched;
    let mut added = raw.added;
    let mut removed = raw.removed;
    let (swaps, mut changed, mut swapped) = collapse_swaps(
        a_cells,
        t,
        b_cells,
        raw.changed,
        spec.costs.swap_dominance_pct,
    );
    // Canonicalize the per-cell vectors so the whole Diff is deterministic
    // (positions are unique per vector → this is a total order).
    added.sort_by(|x, y| x.0.cmp(&y.0));
    removed.sort_by(|x, y| x.0.cmp(&y.0));
    changed.sort_by(|x, y| x.0.cmp(&y.0));
    swapped.sort_by(|x, y| x.0.cmp(&y.0));
    let max_cells = a_cells.len().max(b_cells.len()).max(1);
    let distance = spec.costs.add as u64 * added.len() as u64
        + spec.costs.delete as u64 * removed.len() as u64
        + spec.costs.change as u64 * changed.len() as u64
        + spec.costs.swap as u64 * swaps.len() as u64;
    // Alignment confidence: every cell that found a counterpart at the chosen
    // transform counts as aligned (matched + changed + swapped), regardless of
    // block equivalence. Only added/removed (no counterpart) are unaligned.
    let support = (matched + changed.len() + swapped.len()) as f32 / max_cells as f32;
    Diff {
        transform: Transform {
            rotate: g.clone(),
            translate: t,
        },
        distance,
        added,
        removed,
        changed,
        swapped,
        palette_swaps: swaps,
        support,
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

    /// Cap on the `regions` array in [`summary_json`](Self::summary_json):
    /// keeps the JSON bounded for pathological scattered-change diffs.
    const SUMMARY_REGION_CAP: usize = 100;

    /// Compact human summary (counts + regions + swaps), no per-cell data.
    /// `regions` is capped at [`SUMMARY_REGION_CAP`](Self::SUMMARY_REGION_CAP)
    /// entries (the largest by changed-cell count); `region_total` always
    /// reports the uncapped count and `regions_truncated` flags the cut.
    pub fn summary_json(&self) -> String {
        let mut regs = crate::diff::regions::regions(self);
        let region_total = regs.len();
        // Largest first; deterministic tie-break by bounding box corner.
        regs.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.min.cmp(&b.min)));
        let regions_truncated = region_total > Self::SUMMARY_REGION_CAP;
        regs.truncate(Self::SUMMARY_REGION_CAP);
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
            "regions_truncated": regions_truncated,
            "region_total": region_total,
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
            distance: v.get("distance").and_then(|d| d.as_u64()).unwrap_or(0),
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
            (adds + removes + changes) as u64,
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
    fn diff_aligns_a_large_featureless_translated_box() {
        use crate::fingerprint::testgen::translated;
        // 120 long on X — exceeds both the anchor threshold (64) and the exact
        // FFT size budget (96). The old path fell back to (0,0,0) and reported a
        // huge spurious diff; the downsample-then-refine path must still align it.
        let a = filled_box((0, 0, 0), (119, 0, 1), "minecraft:stone");
        let b = translated(&a, (40, 0, 5));
        let spec = DiffSpec::from_preset(FingerprintSpec::structural());
        let d = diff(&a, &b, &spec);
        assert_eq!(d.distance, 0, "large featureless box still aligns");
        assert_eq!(d.transform.translate, (40, 0, 5));
    }

    #[test]
    fn fft_fallback_output_is_stable() {
        use crate::fingerprint::testgen::translated;
        // Featureless translated box (forces the FFT-fallback path), plus one
        // added block. Locks the fallback path's output so the compare-reuse
        // refactor stays behavior-preserving.
        let a = filled_box((0, 0, 0), (40, 0, 2), "minecraft:stone");
        let mut b = translated(&a, (7, 0, 3));
        b.set_block(100, 0, 0, &BlockState::new("minecraft:glass")); // added
                                                                     // exact: the box's "stone" token still exceeds the anchor threshold (no
                                                                     // Hough anchors → FFT fallback), but the added glass is a visible token.
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);
        assert_eq!(d.transform.translate, (7, 0, 3), "recovers the shift");
        assert_eq!(d.added.len(), 1);
        assert_eq!(d.removed.len(), 0);
        assert_eq!(d.distance, 1);
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
    fn palette_swaps_and_json_are_deterministic() {
        use crate::fingerprint::testgen::repalette;
        // A: stone region (10) + dirt region (10) + oak_planks (4, a 50/50 tie token).
        let stone = BlockState::new("minecraft:stone");
        let dirt = BlockState::new("minecraft:dirt");
        let oak = BlockState::new("minecraft:oak_planks");
        let mut a = UniversalSchematic::new("a".to_string());
        for x in 0..5 {
            for z in 0..2 {
                a.set_block(x, 0, z, &stone);
            }
            for z in 2..4 {
                a.set_block(x, 0, z, &dirt);
            }
        }
        for x in 0..4 {
            a.set_block(x, 0, 4, &oak);
        }
        // B: two distinct palette swaps (stone→cobble, dirt→gravel) over equal
        // regions, plus oak split 50/50 spruce/birch (a dominant-target tie that
        // never reaches the swap threshold → stays as `changed`).
        let mut b = repalette(&a, "minecraft:stone", "minecraft:cobblestone");
        b = repalette(&b, "minecraft:dirt", "minecraft:gravel");
        b.set_block(0, 0, 4, &BlockState::new("minecraft:spruce_planks"));
        b.set_block(1, 0, 4, &BlockState::new("minecraft:spruce_planks"));
        b.set_block(2, 0, 4, &BlockState::new("minecraft:birch_planks"));
        b.set_block(3, 0, 4, &BlockState::new("minecraft:birch_planks"));

        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let first = diff(&a, &b, &spec);
        let first_swaps = first.palette_swaps.clone();
        let first_json = first.to_json();
        assert_eq!(first_swaps.len(), 2, "two distinct swaps");
        // Run many times in-process: HashMap iteration order varies per instance,
        // so any ordering leak surfaces within this loop.
        for _ in 0..50 {
            let d = diff(&a, &b, &spec);
            assert_eq!(
                d.palette_swaps, first_swaps,
                "palette_swaps ordering must be deterministic"
            );
            assert_eq!(d.to_json(), first_json, "to_json must be deterministic");
        }
    }

    #[test]
    fn to_json_is_stable_and_position_sorted() {
        let a = filled_box((0, 0, 0), (4, 2, 2), "minecraft:stone");
        let (b, _) = edited(&a, 6);
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d1 = diff(&a, &b, &spec);
        let d2 = diff(&a, &b, &spec);
        assert_eq!(d1.to_json(), d2.to_json(), "diff JSON must be reproducible");

        let v: serde_json::Value = serde_json::from_str(&d1.to_json()).unwrap();
        for key in ["added", "removed", "changed", "swapped"] {
            let arr = v[key].as_array().unwrap();
            let positions: Vec<(i64, i64, i64)> = arr
                .iter()
                .map(|c| {
                    let p = c["pos"].as_array().unwrap();
                    (
                        p[0].as_i64().unwrap(),
                        p[1].as_i64().unwrap(),
                        p[2].as_i64().unwrap(),
                    )
                })
                .collect();
            let mut sorted = positions.clone();
            sorted.sort();
            assert_eq!(positions, sorted, "{key} cells must be position-sorted");
        }
    }

    #[test]
    fn support_is_one_for_pure_repalette_under_exact() {
        use crate::fingerprint::testgen::repalette;
        let a = filled_box((0, 0, 0), (4, 0, 4), "minecraft:stone");
        let b = repalette(&a, "minecraft:stone", "minecraft:cobblestone");
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);
        // Every cell is aligned (it's a pure swap), even though the palette changed.
        assert_eq!(d.support, 1.0, "pure re-palette is perfectly aligned");
        assert_eq!(d.distance, 1, "still a single swap op");
    }

    #[test]
    fn support_excludes_only_unaligned_cells() {
        use crate::fingerprint::testgen::translated;
        let a = filled_box((0, 0, 0), (4, 0, 4), "minecraft:stone"); // 25 cells
        let mut b = translated(&a, (0, 0, 0)); // copy
        let air = BlockState::new("minecraft:air");
        // 2 removals (no counterpart) ...
        b.set_block(0, 0, 0, &air);
        b.set_block(1, 0, 0, &air);
        // ... and 3 changes to distinct targets (counterpart found, not a swap).
        b.set_block(0, 0, 1, &BlockState::new("minecraft:glass"));
        b.set_block(1, 0, 1, &BlockState::new("minecraft:sand"));
        b.set_block(2, 0, 1, &BlockState::new("minecraft:dirt"));
        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);
        assert_eq!(d.removed.len(), 2);
        assert_eq!(d.changed.len(), 3);
        assert!(d.palette_swaps.is_empty());
        // Only added+removed are unaligned; changed cells still count as aligned.
        let max = 25.0_f32;
        let expected = 1.0 - (d.added.len() + d.removed.len()) as f32 / max;
        assert!((d.support - expected).abs() < 1e-6, "support={}", d.support);
        assert!(d.support < 1.0);
        assert!((d.support - 0.92).abs() < 1e-6);
    }

    #[test]
    fn distance_is_u64_and_does_not_overflow() {
        // A large per-op cost times a handful of cells overflows u32 (>4.29e9)
        // but fits comfortably in u64. Keeps the fixture tiny.
        let a = UniversalSchematic::new("a".to_string());
        let b = filled_box((0, 0, 0), (4, 0, 0), "minecraft:stone"); // 5 added cells
        let ov = SpecOverrides {
            cost_add: Some(1_000_000_000),
            ..Default::default()
        };
        let spec = DiffSpec::resolve("exact", &ov).unwrap();
        let d = diff(&a, &b, &spec);
        assert_eq!(d.added.len(), 5);
        let expected: u64 = 1_000_000_000u64 * 5;
        assert_eq!(d.distance, expected, "no u32 wrap");
        // round-trips through JSON at full width
        let back = Diff::from_json(&d.to_json()).unwrap();
        assert_eq!(back.distance, expected);
    }

    #[test]
    fn swap_dominance_threshold_is_configurable() {
        use crate::fingerprint::testgen::translated;
        // 10 stone cells; B turns 6→cobble, 4→andesite. Dominant target (cobble)
        // is exactly 60% of the changes.
        let stone = BlockState::new("minecraft:stone");
        let mut a = UniversalSchematic::new("a".to_string());
        for x in 0..10 {
            a.set_block(x, 0, 0, &stone);
        }
        let mut b = translated(&a, (0, 0, 0));
        for x in 0..6 {
            b.set_block(x, 0, 0, &BlockState::new("minecraft:cobblestone"));
        }
        for x in 6..10 {
            b.set_block(x, 0, 0, &BlockState::new("minecraft:andesite"));
        }

        // Default 80% threshold: 60% < 80% → not collapsed to a swap.
        let high = DiffSpec::resolve("exact", &SpecOverrides::default()).unwrap();
        let d_high = diff(&a, &b, &high);
        assert!(d_high.palette_swaps.is_empty(), "60% < 80% → no swap");

        // Lowered to 50%: 60% >= 50% → collapses into one swap.
        let ov = SpecOverrides {
            swap_dominance_pct: Some(50),
            ..Default::default()
        };
        let low = DiffSpec::resolve("exact", &ov).unwrap();
        let d_low = diff(&a, &b, &low);
        assert_eq!(d_low.palette_swaps.len(), 1, "60% >= 50% → swap");
        assert_eq!(d_low.swapped.len(), 6, "the 6 cobble cells are the swap");
    }

    #[test]
    fn explicit_air_diffs_to_zero() {
        // Air is absence, not a block: explicit air in A vs nothing in B must
        // not surface as removed/changed. Holds across presets (filtered in
        // `cells()`), checked here under exact + structural.
        for spec in [
            DiffSpec::from_preset(FingerprintSpec::exact()),
            DiffSpec::from_preset(FingerprintSpec::structural()),
        ] {
            let mut a = filled_box((0, 0, 0), (2, 0, 0), "minecraft:stone");
            a.set_block(0, 5, 0, &BlockState::new("minecraft:air"));
            a.set_block(1, 5, 0, &BlockState::new("minecraft:cave_air"));
            a.set_block(2, 5, 0, &BlockState::new("minecraft:void_air"));
            let b = filled_box((0, 0, 0), (2, 0, 0), "minecraft:stone");
            let d = diff(&a, &b, &spec);
            assert_eq!(d.distance, 0, "explicit air is absence");
            assert!(
                d.added.is_empty()
                    && d.removed.is_empty()
                    && d.changed.is_empty()
                    && d.swapped.is_empty()
            );
        }
    }

    #[test]
    fn block_entity_nbt_change_is_a_nonzero_diff() {
        use crate::block_entity::BlockEntity;
        use crate::block_position::BlockPosition;
        use crate::utils::NbtValue;

        // Identical blocks (one chest at the origin) differing ONLY in NBT.
        let mut a = filled_box((0, 0, 0), (0, 0, 0), "minecraft:chest");
        let mut b = filled_box((0, 0, 0), (0, 0, 0), "minecraft:chest");
        a.set_block_entity(
            BlockPosition { x: 0, y: 0, z: 0 },
            BlockEntity::new("minecraft:chest".to_string(), (0, 0, 0)).with_nbt_data(
                "CustomName".to_string(),
                NbtValue::String("Alpha".to_string()),
            ),
        );
        b.set_block_entity(
            BlockPosition { x: 0, y: 0, z: 0 },
            BlockEntity::new("minecraft:chest".to_string(), (0, 0, 0)).with_nbt_data(
                "CustomName".to_string(),
                NbtValue::String("Beta".to_string()),
            ),
        );

        let spec = DiffSpec::from_preset(FingerprintSpec::exact());
        let d = diff(&a, &b, &spec);
        // DESIRED behavior (not yet implemented): the NBT difference is detected
        // and the chest cell is reported as changed.
        assert!(
            d.distance > 0,
            "differing chest NBT should be a non-zero diff"
        );
        assert_eq!(d.changed.len(), 1, "the chest cell is the changed cell");
    }

    #[test]
    fn refinement_search_is_bounded_for_large_strides() {
        // A downsampled alignment of a huge build can hand refine_offset a
        // stride-12 window; the exhaustive grid would be 25^3 = 15625 compare
        // passes. The search must stay within REFINE_MAX_PASSES.
        let offs = refinement_offsets((5, -3, 7), 12);
        assert!(
            offs.len() <= REFINE_MAX_PASSES,
            "at most {} candidate offsets, got {}",
            REFINE_MAX_PASSES,
            offs.len()
        );
        // The coarser grid still spans the whole ±window cube.
        assert!(offs.contains(&(5, -3, 7)), "base offset included");
        assert!(offs.contains(&(5 + 12, -3 + 12, 7 + 12)), "+w corner");
        assert!(offs.contains(&(5 - 12, -3 - 12, 7 - 12)), "-w corner");
        // Small windows stay exhaustive (unchanged behavior).
        assert_eq!(refinement_offsets((0, 0, 0), 1).len(), 27);
        assert_eq!(refinement_offsets((0, 0, 0), 3).len(), 343);
        for w in [0usize, 1, 2, 3, 5, 7, 12, 40, 1000] {
            assert!(
                refinement_offsets((0, 0, 0), w).len() <= REFINE_MAX_PASSES,
                "window {w} exceeds the pass cap"
            );
        }
    }

    #[test]
    fn refinement_finds_an_offset_between_coarse_grid_samples() {
        // Regression: with a stride-12 window the coarse grid samples only
        // {0,±4,±8,±12} per axis, so a true offset of +1 sits BETWEEN samples.
        // The exhaustive second sweep around the coarse winner must still
        // land on it exactly.
        let spec = FingerprintSpec::exact();
        let a = filled_box((0, 0, 0), (6, 4, 5), "minecraft:stone");
        let b = filled_box((1, 0, 0), (7, 4, 5), "minecraft:stone");
        let g = RigidOp::identity();
        let a_cells = cells(&a, &g, &spec);
        let b_cells = cells(&b, &g, &spec);

        let (off, raw) = refine_offset(&a_cells, &b_cells, (0, 0, 0), 12);
        assert_eq!(off, (1, 0, 0), "exact offset recovered from a strided grid");
        assert_eq!(
            raw_score(&raw),
            0,
            "the recovered offset aligns the builds perfectly"
        );
    }

    #[test]
    fn refinement_total_passes_stay_bounded() {
        // Coarse grid + one exhaustive gap sweep, both capped.
        for w in [0usize, 1, 2, 3, 5, 7, 12, 40, 1000] {
            let coarse = refinement_offsets((0, 0, 0), w).len();
            let step = refinement_step(w);
            let gap = if step > 1 {
                refinement_offsets((0, 0, 0), step - 1).len()
            } else {
                0
            };
            assert!(
                coarse + gap <= REFINE_MAX_TOTAL_PASSES,
                "window {w}: {coarse}+{gap} exceeds the total pass cap"
            );
        }
    }

    #[test]
    fn summary_json_caps_regions_and_reports_totals() {
        // 200 isolated added cells (spaced out → 200 one-cell regions), with
        // the first region enlarged so "keep the largest" is observable.
        let stone = BlockState::new("minecraft:stone");
        let mut added = Vec::new();
        for i in 0..200 {
            added.push(((i * 5, 0, 0), stone.clone()));
        }
        // A second row attached to region 0 makes it a 4-cell region.
        for z in 1..4 {
            added.push(((0, 0, z), stone.clone()));
        }
        let d = Diff {
            transform: Transform {
                rotate: RigidOp::identity(),
                translate: (0, 0, 0),
            },
            distance: added.len() as u64,
            added,
            removed: Vec::new(),
            changed: Vec::new(),
            swapped: Vec::new(),
            palette_swaps: Vec::new(),
            support: 0.0,
        };
        let s: serde_json::Value = serde_json::from_str(&d.summary_json()).unwrap();
        let regs = s["regions"].as_array().unwrap();
        assert_eq!(regs.len(), 100, "regions capped at 100");
        assert_eq!(s["regions_truncated"], serde_json::json!(true));
        assert_eq!(s["region_total"], serde_json::json!(200));
        // Largest regions kept: the 4-cell region must survive the cut.
        assert!(
            regs.iter().any(|r| r["count"].as_u64() == Some(4)),
            "largest region kept"
        );

        // A small diff is not truncated and reports its true total.
        let small = Diff {
            transform: Transform {
                rotate: RigidOp::identity(),
                translate: (0, 0, 0),
            },
            distance: 1,
            added: vec![((0, 0, 0), stone.clone())],
            removed: Vec::new(),
            changed: Vec::new(),
            swapped: Vec::new(),
            palette_swaps: Vec::new(),
            support: 1.0,
        };
        let s: serde_json::Value = serde_json::from_str(&small.summary_json()).unwrap();
        assert_eq!(s["regions"].as_array().unwrap().len(), 1);
        assert_eq!(s["regions_truncated"], serde_json::json!(false));
        assert_eq!(s["region_total"], serde_json::json!(1));
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
