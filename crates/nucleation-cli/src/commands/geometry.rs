//! Scene operations use the same geometry and solid brush as the library.
use nucleation::building::{
    BuildingTool, Cuboid, Curve3D, Shape, ShapeEnum, SolidBrush, Sphere, TubePath,
};
use nucleation::formats::limits::DecodeLimits;
use nucleation::{BlockState, UniversalSchematic};
use std::process::Command;

pub(super) enum Operation {
    Set([i32; 3], String),
    Cuboid([i32; 3], [i32; 3]),
    Sphere([i32; 3]),
    Curve(String),
}

pub(super) struct BrushOptions {
    pub block: String,
    pub radius: f64,
    pub steps: usize,
}
impl Default for BrushOptions {
    fn default() -> Self {
        Self {
            block: "minecraft:stone".into(),
            radius: 1.5,
            steps: 128,
        }
    }
}

fn sample_curve(expression: &str, steps: usize) -> Result<Curve3D, String> {
    if !(1..Curve3D::MAX_POINTS).contains(&steps) {
        return Err("invalid curve interval count".into());
    }
    // jq only evaluates the user-supplied formula. TubePath connects and
    // voxelizes the resulting polyline, including between sparse samples.
    const SAMPLER:&str = "def t: . * 2 * 3.141592653589793; def sin($x): $x|sin; def cos($x): $x|cos; [range(0;$n+1) | ./ $n | [POINT] | if length==1 then .[0] else error(\"expected one point per sample\") end]";
    let result = Command::new("jq")
        .args([
            "-cne",
            "--argjson",
            "n",
            &steps.to_string(),
            "--",
            &SAMPLER.replace("POINT", expression),
        ])
        .output()
        .map_err(|e| format!("Running jq to sample --curve: {e}"))?;
    if !result.status.success() {
        return Err(format!(
            "Curve formula: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }
    let points: Vec<(f64, f64, f64)> = serde_json::from_slice(&result.stdout)
        .map_err(|e| format!("Curve must return three finite coordinates per point: {e}"))?;
    Curve3D::new(points, false)
}

impl Operation {
    pub(super) fn apply(
        self,
        scene: &mut UniversalSchematic,
        options: &BrushOptions,
    ) -> Result<(), String> {
        let radius = options.radius;
        let shape = match self {
            Self::Set([x, y, z], state) => {
                return scene.set_block_from_string(x, y, z, &state).map(|_| ())
            }
            Self::Cuboid(a, b) => ShapeEnum::Cuboid(Cuboid::new(a.into(), b.into())),
            Self::Sphere(center) => {
                let r = radius.ceil() as i32;
                if !radius.is_finite()
                    || radius < 0.0
                    || center
                        .iter()
                        .any(|x| x.checked_add(r).is_none() || x.checked_sub(r).is_none())
                {
                    return Err("sphere radius or coordinates out of range".into());
                }
                ShapeEnum::Sphere(Sphere::new(center.into(), radius))
            }
            Self::Curve(expr) => {
                ShapeEnum::TubePath(TubePath::new(sample_curve(&expr, options.steps)?, radius)?)
            }
        };
        let (x, y, z, xx, yy, zz) = shape.bounds();
        let (mut low, mut high) = ([x, y, z], [xx, yy, zz]);
        let region = &scene.default_region;
        if !region.is_empty() || !region.entities.is_empty() || !region.block_entities.is_empty() {
            let b = region.get_bounding_box();
            let min: [i32; 3] = b.min.into();
            let max: [i32; 3] = b.max.into();
            for i in 0..3 {
                low[i] = low[i].min(min[i]);
                high[i] = high[i].max(max[i]);
            }
        }
        let dims: [i64; 3] = std::array::from_fn(|i| i64::from(high[i]) - i64::from(low[i]) + 1);
        DecodeLimits::default()
            .check_dimensions(dims.into())
            .map_err(|e| format!("scene bounds: {e}"))?;
        let brush = SolidBrush::new(BlockState::from_block_string(&options.block)?);
        BuildingTool::new(scene).fill(&shape, &brush);
        Ok(())
    }
}
