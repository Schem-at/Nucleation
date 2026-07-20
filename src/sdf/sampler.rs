//! Samples an [`SdfNode`] into a [`UniversalSchematic`] using declarative
//! material rules.
//!
//! Determinism: block placement depends only on the SDF tree, the rules, and
//! their embedded seeds — identical inputs produce identical schematics on
//! every platform and binding.

use super::node::SdfNode;
use super::noise::{fbm2, hash01_2, hash01_3};
use crate::building::{palette_by_name, BlockPalette};
use crate::UniversalSchematic;
use serde::{Deserialize, Serialize};

/// Inclusive numeric range; either bound may be omitted.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Range {
    #[serde(default)]
    pub min: Option<i32>,
    #[serde(default)]
    pub max: Option<i32>,
}

impl Range {
    fn contains(&self, v: i32) -> bool {
        self.min.is_none_or(|m| v >= m) && self.max.is_none_or(|m| v <= m)
    }
}

/// 2D noise condition over (x, z): matches when FBM(x, z) > threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseCondition {
    pub threshold: f32,
    pub frequency: f32,
    #[serde(default)]
    pub seed: i32,
    #[serde(default = "default_octaves")]
    pub octaves: u32,
}

fn default_octaves() -> u32 {
    3
}

/// Conditions of a fill rule; all present conditions must match (AND).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct When {
    /// Depth measured from the top of the local solid run in the column
    /// (0 = surface block of that run).
    #[serde(default)]
    pub depth_below_surface: Option<Range>,
    /// Absolute world Y range.
    #[serde(default)]
    pub y_range: Option<Range>,
    /// 2D noise gate over (x, z).
    #[serde(default)]
    pub noise: Option<NoiseCondition>,
}

/// One material rule: the first rule whose `when` matches wins.
/// A rule without `when` always matches (use it last as the default).
///
/// Exactly one of `block` (fixed block string) or `gradient` (palette-driven,
/// position-dependent block choice) must be present; rules with both or
/// neither are rejected when sampling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillRule {
    #[serde(default)]
    pub when: Option<When>,
    /// Block string, `set_block_str` syntax (properties allowed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,
    /// Palette/gradient-driven fill: the block is chosen per position from a
    /// palette instead of being fixed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gradient: Option<GradientFill>,
}

/// Palette selector for a [`GradientFill`]: either one of the preset names
/// (`"all"`, `"solid"`, `"structural"`, `"decorative"`, `"concrete"`,
/// `"wool"`, `"terracotta"`, `"grayscale"`, `"wood"`) or an explicit block
/// list `{"ids": ["minecraft:stone", ...]}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PaletteSpec {
    Named(String),
    Ids { ids: Vec<String> },
}

/// Axis a [`GradientFill`] measures position along.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GradientAxis {
    /// Absolute world Y.
    Y,
    /// Depth below the local surface (same measure as `depthBelowSurface`).
    Depth,
}

/// Ramp mode for a [`GradientFill`] that indexes the palette directly
/// instead of interpolating between two colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RampMode {
    /// Position indexes the palette sorted by perceptual lightness
    /// (dark at `range[0]` → light at `range[1]`).
    Lightness,
}

/// Palette-driven fill: the block at each position is chosen by mapping the
/// position along `axis` over `range` (clamped) to either an interpolated
/// `from`→`to` color snapped to the palette (default), or directly into the
/// lightness-sorted palette (`"ramp": "lightness"`). Fully deterministic —
/// the block depends only on the position and the rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientFill {
    pub palette: PaletteSpec,
    /// RGB at `range[0]` (required unless `ramp` is set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<[u8; 3]>,
    /// RGB at `range[1]` (required unless `ramp` is set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<[u8; 3]>,
    /// Axis the gradient runs along; defaults to `"y"`.
    #[serde(default = "default_axis")]
    pub axis: GradientAxis,
    /// `[min, max]` position span the gradient is stretched over; positions
    /// outside are clamped.
    pub range: [i32; 2],
    /// When set, index the sorted palette directly instead of color-lerping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ramp: Option<RampMode>,
    /// Ordered (Bayer 4x4) dithering between adjacent gradient steps —
    /// reads as extra intermediate shades. Default false.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dither: bool,
}

fn default_axis() -> GradientAxis {
    GradientAxis::Y
}

/// Scatter decoration placed on air directly above a surface block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceRule {
    /// Probability per surface column, 0..1.
    pub density: f32,
    /// Candidate block strings; one is picked deterministically per position.
    pub blocks: Vec<String>,
    #[serde(default)]
    pub seed: i32,
    /// Only decorate surfaces whose block matches this name (optional,
    /// compared against the fill rule's block string prefix).
    #[serde(default)]
    pub on: Option<String>,
}

/// Full material specification for sampling.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MaterialRules {
    #[serde(default)]
    pub fill: Vec<FillRule>,
    #[serde(default)]
    pub surface: Vec<SurfaceRule>,
}

impl MaterialRules {
    pub fn from_json(json: &str) -> Result<MaterialRules, String> {
        serde_json::from_str(json).map_err(|e| format!("Invalid material rules JSON: {e}"))
    }
}

/// Cap on precomputed gradient steps: indexing stays deterministic and the
/// visual resolution of a color ramp saturates long before this.
const MAX_GRADIENT_STEPS: i64 = 256;

/// A [`GradientFill`] resolved to a ready-to-index block ramp.
struct ResolvedGradient {
    ids: Vec<String>,
    axis: GradientAxis,
    min: i32,
    max: i32,
    dither: bool,
}

impl ResolvedGradient {
    fn pick(&self, x: i32, y: i32, z: i32, depth: i32) -> &str {
        let v = match self.axis {
            GradientAxis::Y => y,
            GradientAxis::Depth => depth,
        };
        if self.max == self.min || self.ids.len() == 1 {
            return &self.ids[0];
        }
        let t = ((v - self.min) as f32 / (self.max - self.min) as f32).clamp(0.0, 1.0);
        let pos = t * (self.ids.len() - 1) as f32;
        if !self.dither {
            return &self.ids[pos.round() as usize];
        }
        // Ordered dithering between the two neighboring steps: the
        // fractional position becomes a per-voxel Bayer-thresholded choice.
        const BAYER: [[f32; 4]; 4] = [
            [0.0, 8.0, 2.0, 10.0],
            [12.0, 4.0, 14.0, 6.0],
            [3.0, 11.0, 1.0, 9.0],
            [15.0, 7.0, 13.0, 5.0],
        ];
        let lo = pos.floor() as usize;
        let hi = (lo + 1).min(self.ids.len() - 1);
        let frac = pos - pos.floor();
        let bx = ((x + y) & 3) as usize;
        let bz = ((z + (y >> 2)) & 3) as usize;
        let threshold = (BAYER[bx][bz] + 0.5) / 16.0;
        if frac > threshold {
            &self.ids[hi]
        } else {
            &self.ids[lo]
        }
    }
}

fn resolve_gradient(g: &GradientFill) -> Result<ResolvedGradient, String> {
    let palette = match &g.palette {
        PaletteSpec::Named(name) => palette_by_name(name)?,
        PaletteSpec::Ids { ids } => BlockPalette::from_block_ids(ids.iter().map(|s| s.as_str())),
    };
    if palette.is_empty() {
        return Err("Gradient palette is empty (unknown ids or no color data)".to_string());
    }
    let [min, max] = g.range;
    if max < min {
        return Err(format!("Gradient range [{min}, {max}] has max < min"));
    }
    let ids = match g.ramp {
        Some(RampMode::Lightness) => palette
            .sorted_by_lightness()
            .block_ids()
            .map(str::to_string)
            .collect(),
        None => {
            let from = g
                .from
                .ok_or("Gradient without `ramp` requires `from` color")?;
            let to = g.to.ok_or("Gradient without `ramp` requires `to` color")?;
            let steps = ((max as i64 - min as i64 + 1).min(MAX_GRADIENT_STEPS)) as usize;
            palette.gradient_ids((from[0], from[1], from[2]), (to[0], to[1], to[2]), steps)
        }
    };
    Ok(ResolvedGradient {
        ids,
        axis: g.axis,
        min,
        max,
        dither: g.dither,
    })
}

/// Validate one fill rule and resolve its gradient (if any).
fn resolve_fill(rule: &FillRule) -> Result<Option<ResolvedGradient>, String> {
    match (&rule.block, &rule.gradient) {
        (Some(_), Some(_)) => Err("Fill rule cannot have both `block` and `gradient`".to_string()),
        (None, None) => Err("Fill rule needs either `block` or `gradient`".to_string()),
        (Some(_), None) => Ok(None),
        (None, Some(g)) => resolve_gradient(g).map(Some),
    }
}

/// Integer sampling bounds (inclusive).
#[derive(Debug, Clone, Copy)]
pub struct SampleBounds {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

/// Derive integer sampling bounds from the node's own AABB.
pub fn auto_bounds(node: &SdfNode) -> Result<SampleBounds, String> {
    let b = node.bounds().ok_or_else(|| {
        "SDF tree is unbounded (plane or uncounted repeat) — explicit sampling bounds required"
            .to_string()
    })?;
    Ok(SampleBounds {
        min: [
            (b.min[0] - 1.0).floor() as i32,
            (b.min[1] - 1.0).floor() as i32,
            (b.min[2] - 1.0).floor() as i32,
        ],
        max: [
            (b.max[0] + 1.0).ceil() as i32,
            (b.max[1] + 1.0).ceil() as i32,
            (b.max[2] + 1.0).ceil() as i32,
        ],
    })
}

const MAX_SAMPLE_VOLUME: i64 = 512 * 512 * 512;

/// Sample the SDF into a new schematic. Blocks are placed at integer
/// coordinates whose cell center (x+0.5, y+0.5, z+0.5) lies inside the
/// surface (distance ≤ 0).
pub fn sample_to_schematic(
    node: &SdfNode,
    rules: &MaterialRules,
    bounds: Option<SampleBounds>,
    name: &str,
) -> Result<UniversalSchematic, String> {
    let bounds = match bounds {
        Some(b) => b,
        None => auto_bounds(node)?,
    };
    for a in 0..3 {
        if bounds.min[a] > bounds.max[a] {
            return Err(format!("Degenerate sampling bounds on axis {a}"));
        }
    }
    let volume = (bounds.max[0] - bounds.min[0] + 1) as i64
        * (bounds.max[1] - bounds.min[1] + 1) as i64
        * (bounds.max[2] - bounds.min[2] + 1) as i64;
    if volume > MAX_SAMPLE_VOLUME {
        return Err(format!(
            "Sampling volume {volume} exceeds limit {MAX_SAMPLE_VOLUME}; pass tighter bounds"
        ));
    }

    // Validate every fill rule up front and pre-resolve gradients into
    // ready-to-index block ramps (one find_closest per ramp step, not per
    // sampled position).
    let resolved: Vec<Option<ResolvedGradient>> = rules
        .fill
        .iter()
        .map(resolve_fill)
        .collect::<Result<_, String>>()?;

    let mut schematic = UniversalSchematic::new(name.to_string());
    let default_fill = "minecraft:stone";
    let height = (bounds.max[1] - bounds.min[1] + 1) as usize;
    let mut solid = vec![false; height];

    for x in bounds.min[0]..=bounds.max[0] {
        for z in bounds.min[2]..=bounds.max[2] {
            let fx = x as f32 + 0.5;
            let fz = z as f32 + 0.5;

            for (i, s) in solid.iter_mut().enumerate() {
                let y = bounds.min[1] + i as i32;
                *s = node.eval(fx, y as f32 + 0.5, fz) <= 0.0;
            }

            // Walk runs top→bottom; depth is measured from each run's top.
            let mut i = height as i32 - 1;
            while i >= 0 {
                if !solid[i as usize] {
                    i -= 1;
                    continue;
                }
                let run_top = i;
                while i >= 0 && solid[i as usize] {
                    i -= 1;
                }
                let run_bottom = i + 1;

                let surface_y = bounds.min[1] + run_top;
                let mut surface_block: Option<&str> = None;
                for yi in (run_bottom..=run_top).rev() {
                    let y = bounds.min[1] + yi;
                    let depth = surface_y - y;
                    let block = pick_fill(rules, &resolved, x, y, z, depth).unwrap_or(default_fill);
                    if yi == run_top {
                        surface_block = Some(block);
                    }
                    schematic.set_block_str(x, y, z, block);
                }

                // Decorations sit on the air block above the run top.
                if (run_top as usize) < height - 1 || surface_y == bounds.max[1] {
                    apply_surface(rules, &mut schematic, x, surface_y, z, surface_block);
                }
            }
        }
    }

    Ok(schematic)
}

fn pick_fill<'a>(
    rules: &'a MaterialRules,
    resolved: &'a [Option<ResolvedGradient>],
    x: i32,
    y: i32,
    z: i32,
    depth: i32,
) -> Option<&'a str> {
    for (rule, gradient) in rules.fill.iter().zip(resolved) {
        let matches = match &rule.when {
            None => true,
            Some(w) => {
                w.depth_below_surface
                    .as_ref()
                    .is_none_or(|r| r.contains(depth))
                    && w.y_range.as_ref().is_none_or(|r| r.contains(y))
                    && w.noise.as_ref().is_none_or(|n| {
                        fbm2(x as f32, z as f32, n.seed, n.frequency, n.octaves) > n.threshold
                    })
            }
        };
        if matches {
            return Some(match gradient {
                Some(g) => g.pick(x, y, z, depth),
                // resolve_fill guarantees a rule without gradient has a block.
                None => rule.block.as_deref().unwrap_or(""),
            });
        }
    }
    None
}

fn apply_surface(
    rules: &MaterialRules,
    schematic: &mut UniversalSchematic,
    x: i32,
    surface_y: i32,
    z: i32,
    surface_block: Option<&str>,
) {
    for rule in &rules.surface {
        if rule.blocks.is_empty() || rule.density <= 0.0 {
            continue;
        }
        if let (Some(on), Some(sb)) = (&rule.on, surface_block) {
            if !sb.starts_with(on.as_str()) {
                continue;
            }
        }
        // Salt with surface_y so stacked overhang surfaces decorate independently.
        if hash01_3(x, surface_y, z, rule.seed) < rule.density {
            let pick = hash01_2(
                x.wrapping_mul(31),
                z.wrapping_mul(17),
                rule.seed.wrapping_add(1),
            );
            let idx = ((pick * rule.blocks.len() as f32) as usize).min(rule.blocks.len() - 1);
            schematic.set_block_str(x, surface_y + 1, z, &rule.blocks[idx]);
            return; // first matching scatter rule wins per column
        }
    }
}
