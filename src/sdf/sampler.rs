//! Samples an [`SdfNode`] into a [`UniversalSchematic`] using declarative
//! material rules.
//!
//! Determinism: block placement depends only on the SDF tree, the rules, and
//! their embedded seeds — identical inputs produce identical schematics on
//! every platform and binding.

use super::noise::{fbm2, hash01_2, hash01_3};
use super::node::SdfNode;
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillRule {
    #[serde(default)]
    pub when: Option<When>,
    /// Block string, `set_block_str` syntax (properties allowed).
    pub block: String,
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
                    let block = pick_fill(rules, x, y, z, depth).unwrap_or(default_fill);
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

fn pick_fill(
    rules: &MaterialRules,
    x: i32,
    y: i32,
    z: i32,
    depth: i32,
) -> Option<&str> {
    for rule in &rules.fill {
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
            return Some(&rule.block);
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
