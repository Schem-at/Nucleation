//! Composable, shader-like material rules, predicates, fluids, and surface
//! scatter for the unified SDF voxel pipeline.
//!
//! A [`Material`] can be a simple solid block, a color/palette gradient or
//! explicit block ramp driven by *any* [`Channel`] (world coordinates, SDF
//! distance, depth below surface, surface slope, mean curvature, or an
//! arbitrary [`Field`]), a shaded/spotlight brush, a CSG material-tag dispatch
//! with smooth seam dithering, or a layered stack of conditional rules.

use super::field::{normalize_f64, Field, FieldError};
use super::noise::hash01_3;
use crate::blockpedia::ExtendedColorData;
use crate::building::{
    BlockPalette, Brush, BrushEnum, GradientStop, InterpolationSpace,
};
use crate::BlockState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Bitmask constants for exposed neighbor faces of a solid voxel.
pub mod face_mask {
    pub const INTERIOR: u8 = 0;
    pub const TOP: u8 = 1 << 0; // +Y is air
    pub const BOTTOM: u8 = 1 << 1; // -Y is air
    pub const NORTH: u8 = 1 << 2; // -Z is air
    pub const SOUTH: u8 = 1 << 3; // +Z is air
    pub const EAST: u8 = 1 << 4; // +X is air
    pub const WEST: u8 = 1 << 5; // -X is air
    pub const SIDE: u8 = NORTH | SOUTH | EAST | WEST;
    pub const ANY: u8 = TOP | BOTTOM | SIDE;
}

/// Direction a [`ScatterRule`] places decorations relative to a solid voxel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScatterFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West,
}

impl ScatterFace {
    #[inline]
    pub fn offset(self) -> [i32; 3] {
        match self {
            ScatterFace::Top => [0, 1, 0],
            ScatterFace::Bottom => [0, -1, 0],
            ScatterFace::North => [0, 0, -1],
            ScatterFace::South => [0, 0, 1],
            ScatterFace::East => [1, 0, 0],
            ScatterFace::West => [-1, 0, 0],
        }
    }

    #[inline]
    pub fn mask(self) -> u8 {
        match self {
            ScatterFace::Top => face_mask::TOP,
            ScatterFace::Bottom => face_mask::BOTTOM,
            ScatterFace::North => face_mask::NORTH,
            ScatterFace::South => face_mask::SOUTH,
            ScatterFace::East => face_mask::EAST,
            ScatterFace::West => face_mask::WEST,
        }
    }
}

/// Scalar data channel readable by [`Condition`]s, gradients, and ramps.
#[derive(Debug, Clone)]
pub enum Channel {
    WorldX,
    WorldY,
    WorldZ,
    /// Signed distance value `d <= 0` at the voxel center.
    Distance,
    /// Vertical blocks below the top of the local solid column run (`0` = top surface).
    DepthBelowSurface,
    /// `1.0 - normal.y` in `[0, 2]`: `0` = flat top, `1` = vertical wall, `2` = overhang.
    Slope,
    /// Vertical component of the unit outward surface normal in `[-1, 1]`.
    NormalY,
    /// Mean curvature proxy (discrete Laplacian): positive on ridges/peaks, negative in crevices.
    Curvature,
    /// Any arbitrary [`Field`] evaluated at the voxel center (CSE-shared with geometry!).
    Field(Field),
}

/// Composable boolean predicate over a sampled voxel.
#[derive(Debug, Clone)]
pub enum Condition {
    Always,
    And(Vec<Condition>),
    Or(Vec<Condition>),
    Not(Box<Condition>),
    /// Matches when the voxel's primary CSG material tag equals `id`.
    MaterialTag(u16),
    /// Matches when `channel` lies within `[min, max]` (inclusive, optional bounds).
    ChannelRange {
        channel: Channel,
        min: Option<f32>,
        max: Option<f32>,
    },
    /// Matches when `dot(normal, dir)` lies within `[min, max]`.
    NormalDot {
        dir: [f32; 3],
        min: Option<f32>,
        max: Option<f32>,
    },
    /// Matches exposed faces: if `mask == 0`, requires an interior voxel (no air neighbors).
    /// Otherwise if `require_all`, all bits in `mask` must be exposed; else any bit.
    ExposedFace {
        mask: u8,
        require_all: bool,
    },
    /// Deterministic spatial hash gate matching with probability in `[0, 1]`.
    Chance {
        probability: f32,
        seed: i32,
    },
}

impl Condition {
    pub fn always() -> Self {
        Self::Always
    }

    pub fn and(self, other: Condition) -> Self {
        match (self, other) {
            (Condition::Always, b) => b,
            (a, Condition::Always) => a,
            (Condition::And(mut a), Condition::And(b)) => {
                a.extend(b);
                Condition::And(a)
            }
            (Condition::And(mut a), b) => {
                a.push(b);
                Condition::And(a)
            }
            (a, Condition::And(mut b)) => {
                b.insert(0, a);
                Condition::And(b)
            }
            (a, b) => Condition::And(vec![a, b]),
        }
    }

    pub fn or(self, other: Condition) -> Self {
        match (self, other) {
            (Condition::Or(mut a), Condition::Or(b)) => {
                a.extend(b);
                Condition::Or(a)
            }
            (Condition::Or(mut a), b) => {
                a.push(b);
                Condition::Or(a)
            }
            (a, b) => Condition::Or(vec![a, b]),
        }
    }

    pub fn not(self) -> Self {
        Condition::Not(Box::new(self))
    }

    pub fn tag(material_id: u16) -> Self {
        Condition::MaterialTag(material_id)
    }

    pub fn channel_range(channel: Channel, min: Option<f32>, max: Option<f32>) -> Self {
        Condition::ChannelRange { channel, min, max }
    }

    pub fn field_range(field: Field, min: Option<f32>, max: Option<f32>) -> Self {
        Condition::ChannelRange {
            channel: Channel::Field(field),
            min,
            max,
        }
    }

    pub fn y_range(min: Option<f32>, max: Option<f32>) -> Self {
        Condition::ChannelRange {
            channel: Channel::WorldY,
            min,
            max,
        }
    }

    pub fn depth_below_surface(min: Option<i32>, max: Option<i32>) -> Self {
        Condition::ChannelRange {
            channel: Channel::DepthBelowSurface,
            min: min.map(|v| v as f32),
            max: max.map(|v| v as f32),
        }
    }

    pub fn distance_range(min: Option<f32>, max: Option<f32>) -> Self {
        Condition::ChannelRange {
            channel: Channel::Distance,
            min,
            max,
        }
    }

    pub fn slope_range(min: Option<f32>, max: Option<f32>) -> Self {
        Condition::ChannelRange {
            channel: Channel::Slope,
            min,
            max,
        }
    }

    pub fn curvature_range(min: Option<f32>, max: Option<f32>) -> Self {
        Condition::ChannelRange {
            channel: Channel::Curvature,
            min,
            max,
        }
    }

    pub fn normal_dot(
        dir: [f32; 3],
        min: Option<f32>,
        max: Option<f32>,
    ) -> Result<Self, FieldError> {
        let dir = normalize_f64(dir)?;
        Ok(Condition::NormalDot { dir, min, max })
    }

    pub fn exposed_face(mask: u8, require_all: bool) -> Self {
        Condition::ExposedFace { mask, require_all }
    }

    pub fn chance(probability: f32, seed: i32) -> Result<Self, FieldError> {
        if !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
            return Err(FieldError::InvalidArgument(
                "chance probability must be in [0, 1]".into(),
            ));
        }
        Ok(Condition::Chance { probability, seed })
    }
}

/// One conditional layer inside [`Material::Layers`].
#[derive(Debug, Clone)]
pub struct MaterialLayer {
    pub condition: Condition,
    pub material: Material,
}

/// Composable block shader evaluated for each solid voxel.
#[derive(Clone)]
pub enum Material {
    /// Fixed block state.
    Solid(BlockState),
    /// Emit nothing (skip writing this voxel).
    Skip,
    /// Nearest block to RGB in a palette (with optional palette dithering).
    Color {
        color: ExtendedColorData,
        palette: Arc<BlockPalette>,
    },
    /// Map any [`Channel`] over `[lo, hi]` through a multi-stop color gradient + palette.
    ChannelGradient {
        channel: Channel,
        stops: Vec<GradientStop>,
        lo: f32,
        hi: f32,
        palette: Arc<BlockPalette>,
        space: InterpolationSpace,
    },
    /// Map any [`Channel`] over `[lo, hi]` directly onto an explicit ordered list of blocks.
    ChannelRamp {
        channel: Channel,
        lo: f32,
        hi: f32,
        blocks: Vec<BlockState>,
        dither: bool,
    },
    /// Deterministic weighted random block selection per position.
    Weighted {
        entries: Vec<(BlockState, f32)>,
        total_weight: f32,
        seed: i32,
    },
    /// Lambertian shading driven by the SDF surface normal.
    Shaded {
        base_rgb: [u8; 3],
        light_dir: [f32; 3],
        ambient: f32,
        palette: Arc<BlockPalette>,
    },
    /// Cone spotlight shading driven by the SDF surface normal.
    Spotlight {
        pos: [f32; 3],
        dir: [f32; 3],
        cone_angle_rad: f32,
        base_rgb: [u8; 3],
        ambient: f32,
        palette: Arc<BlockPalette>,
    },
    /// Any existing [`BrushEnum`] wrapped as a [`Material`].
    Brush(BrushEnum),
    /// First matching conditional layer wins; falls back to `default` (or `Skip`).
    Layers {
        layers: Vec<MaterialLayer>,
        default: Option<Box<Material>>,
    },
    /// Dispatch material by CSG `material_id`, optionally dithering across smooth seams.
    MatchTag {
        cases: Vec<(u16, Material)>,
        default: Box<Material>,
        blend_seams: bool,
        seed: i32,
    },
}

impl std::fmt::Debug for Material {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Material::Solid(b) => write!(f, "Solid({})", b.get_name()),
            Material::Skip => write!(f, "Skip"),
            Material::Color { .. } => write!(f, "Color(..)"),
            Material::ChannelGradient { .. } => write!(f, "ChannelGradient(..)"),
            Material::ChannelRamp { blocks, .. } => {
                write!(f, "ChannelRamp(len={})", blocks.len())
            }
            Material::Weighted { entries, .. } => write!(f, "Weighted(len={})", entries.len()),
            Material::Shaded { .. } => write!(f, "Shaded(..)"),
            Material::Spotlight { .. } => write!(f, "Spotlight(..)"),
            Material::Brush(_) => write!(f, "Brush(..)"),
            Material::Layers { layers, .. } => write!(f, "Layers(len={})", layers.len()),
            Material::MatchTag { cases, .. } => write!(f, "MatchTag(len={})", cases.len()),
        }
    }
}

impl Material {
    pub fn solid(block: impl Into<String>) -> Self {
        Material::Solid(BlockState::new(block.into()))
    }

    pub fn solid_state(block: BlockState) -> Self {
        Material::Solid(block)
    }

    pub fn skip() -> Self {
        Material::Skip
    }

    pub fn color(rgb: [u8; 3], palette: Arc<BlockPalette>) -> Self {
        Material::Color {
            color: ExtendedColorData::from_rgb(rgb[0], rgb[1], rgb[2]),
            palette,
        }
    }

    pub fn from_brush(brush: BrushEnum) -> Self {
        Material::Brush(brush)
    }

    pub fn channel_gradient(
        channel: Channel,
        stops: Vec<GradientStop>,
        lo: f32,
        hi: f32,
        palette: Arc<BlockPalette>,
        space: InterpolationSpace,
    ) -> Result<Self, FieldError> {
        if stops.is_empty()
            || !lo.is_finite()
            || !hi.is_finite()
            || lo >= hi
            || stops
                .iter()
                .any(|s| !s.position.is_finite() || !(0.0..=1.0).contains(&s.position))
            || stops.windows(2).any(|p| p[0].position > p[1].position)
        {
            return Err(FieldError::InvalidArgument(
                "gradient requires lo < hi and nondecreasing stops in [0, 1]".into(),
            ));
        }
        Ok(Material::ChannelGradient {
            channel,
            stops,
            lo,
            hi,
            palette,
            space,
        })
    }

    pub fn channel_ramp(
        channel: Channel,
        lo: f32,
        hi: f32,
        blocks: Vec<BlockState>,
        dither: bool,
    ) -> Result<Self, FieldError> {
        if blocks.is_empty() || !lo.is_finite() || !hi.is_finite() || lo >= hi {
            return Err(FieldError::InvalidArgument(
                "channel ramp requires non-empty blocks and lo < hi".into(),
            ));
        }
        Ok(Material::ChannelRamp {
            channel,
            lo,
            hi,
            blocks,
            dither,
        })
    }

    pub fn weighted(entries: Vec<(BlockState, f32)>, seed: i32) -> Result<Self, FieldError> {
        if entries.is_empty() {
            return Err(FieldError::InvalidArgument(
                "weighted material requires at least one entry".into(),
            ));
        }
        let mut total = 0.0f32;
        for (_, w) in &entries {
            if !w.is_finite() || *w <= 0.0 {
                return Err(FieldError::InvalidArgument(
                    "weights must be finite and positive".into(),
                ));
            }
            total += *w;
        }
        Ok(Material::Weighted {
            entries,
            total_weight: total,
            seed,
        })
    }

    pub fn shaded(
        base_rgb: [u8; 3],
        light_dir: [f32; 3],
        ambient: f32,
        palette: Arc<BlockPalette>,
    ) -> Result<Self, FieldError> {
        let dir = normalize_f64(light_dir)?;
        if !ambient.is_finite() || !(0.0..=1.0).contains(&ambient) {
            return Err(FieldError::InvalidArgument(
                "ambient must be in [0, 1]".into(),
            ));
        }
        Ok(Material::Shaded {
            base_rgb,
            light_dir: dir,
            ambient,
            palette,
        })
    }

    pub fn spotlight(
        pos: [f32; 3],
        dir: [f32; 3],
        cone_angle_deg: f32,
        base_rgb: [u8; 3],
        ambient: f32,
        palette: Arc<BlockPalette>,
    ) -> Result<Self, FieldError> {
        if pos.iter().any(|v| !v.is_finite())
            || !cone_angle_deg.is_finite()
            || cone_angle_deg <= 0.0
            || cone_angle_deg > 180.0
            || !ambient.is_finite()
            || !(0.0..=1.0).contains(&ambient)
        {
            return Err(FieldError::InvalidArgument(
                "invalid spotlight parameters".into(),
            ));
        }
        let dir = normalize_f64(dir)?;
        Ok(Material::Spotlight {
            pos,
            dir,
            cone_angle_rad: cone_angle_deg.to_radians(),
            base_rgb,
            ambient,
            palette,
        })
    }

    pub fn layered(layers: Vec<MaterialLayer>, default: Option<Material>) -> Self {
        Material::Layers {
            layers,
            default: default.map(Box::new),
        }
    }

    pub fn match_tag(
        cases: Vec<(u16, Material)>,
        default: Material,
        blend_seams: bool,
        seed: i32,
    ) -> Self {
        Material::MatchTag {
            cases,
            default: Box::new(default),
            blend_seams,
            seed,
        }
    }
}

/// Optional fluid volume filling air cells where `field <= 0` and solid geometry `> 0`.
#[derive(Debug, Clone)]
pub struct FluidLayer {
    pub field: Field,
    pub block: BlockState,
}

impl FluidLayer {
    pub fn new(field: Field, block: impl Into<String>) -> Self {
        Self {
            field,
            block: BlockState::new(block.into()),
        }
    }

    /// Flat horizontal water/lava table filling air at `y <= level`.
    pub fn level(y_level: f32, block: impl Into<String>) -> Result<Self, FieldError> {
        let field = Field::plane([0.0, 1.0, 0.0], -y_level)?;
        Ok(Self::new(field, block))
    }
}

/// Surface dressing / scatter rule emitting blocks on air cells adjacent to
/// exposed solid faces (supports top grass/flowers, hanging vines on ceilings,
/// or side attachments, plus multi-block column heights).
#[derive(Debug, Clone)]
pub struct ScatterRule {
    pub face: ScatterFace,
    pub density: f32,
    pub blocks: Vec<(BlockState, f32)>,
    pub total_weight: f32,
    pub seed: i32,
    pub condition: Condition,
    pub on_block: Option<String>,
    pub height_min: u32,
    pub height_max: u32,
}

impl ScatterRule {
    pub fn new(
        face: ScatterFace,
        density: f32,
        blocks: Vec<(BlockState, f32)>,
        seed: i32,
    ) -> Result<Self, FieldError> {
        if !density.is_finite() || !(0.0..=1.0).contains(&density) || blocks.is_empty() {
            return Err(FieldError::InvalidArgument(
                "scatter rule requires density in [0, 1] and non-empty blocks".into(),
            ));
        }
        let mut total_weight = 0.0f32;
        for (_, w) in &blocks {
            if !w.is_finite() || *w <= 0.0 {
                return Err(FieldError::InvalidArgument(
                    "scatter weights must be finite and positive".into(),
                ));
            }
            total_weight += *w;
        }
        Ok(Self {
            face,
            density,
            blocks,
            total_weight,
            seed,
            condition: Condition::Always,
            on_block: None,
            height_min: 1,
            height_max: 1,
        })
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = condition;
        self
    }

    pub fn on_block(mut self, block_prefix: impl Into<String>) -> Self {
        self.on_block = Some(block_prefix.into());
        self
    }

    pub fn with_height_range(mut self, min: u32, max: u32) -> Result<Self, FieldError> {
        if min == 0 || min > max || max > 64 {
            return Err(FieldError::InvalidArgument(
                "scatter height range must satisfy 1 <= min <= max <= 64".into(),
            ));
        }
        self.height_min = min;
        self.height_max = max;
        Ok(self)
    }

    pub(crate) fn pick_block(&self, x: i32, y: i32, z: i32) -> &BlockState {
        let mut r = hash01_3(
            x.wrapping_mul(31),
            y.wrapping_mul(17),
            z.wrapping_mul(13),
            self.seed.wrapping_add(1),
        ) * self.total_weight;
        for (block, w) in &self.blocks {
            if r <= *w {
                return block;
            }
            r -= *w;
        }
        &self.blocks[self.blocks.len() - 1].0
    }

    pub(crate) fn pick_height(&self, x: i32, y: i32, z: i32) -> u32 {
        if self.height_min == self.height_max {
            return self.height_min;
        }
        let span = self.height_max - self.height_min + 1;
        let h = hash01_3(
            x.wrapping_mul(7),
            y.wrapping_mul(29),
            z.wrapping_mul(53),
            self.seed.wrapping_add(2),
        );
        self.height_min + ((h * span as f32) as u32).min(span - 1)
    }
}

// ── Per-Voxel Evaluation Context ─────────────────────────────────────────────

/// Full per-voxel sample passed to [`Material`] and [`Condition`] evaluators.
pub struct VoxelContext<'a> {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub distance: f32,
    pub normal: [f32; 3],
    pub curvature: f32,
    pub depth_below_surface: i32,
    pub exposed_faces: u8,
    pub material_id: u16,
    pub blend_material_id: u16,
    pub blend_weight: f32,
    pub aux_fields: &'a [f32],
}

/// Resolved index map from `Field` pointers to auxiliary roots in `CompiledGraph`.
#[derive(Debug, Default, Clone)]
pub(crate) struct ChannelRegistry {
    pub aux_fields: Vec<Field>,
    pub ptr_to_aux: std::collections::HashMap<usize, usize>,
    pub needs_normal: bool,
    pub needs_curvature: bool,
    pub needs_depth: bool,
    pub needs_exposed: bool,
    pub needs_tags: bool,
}

impl ChannelRegistry {
    pub fn register_field(&mut self, field: &Field) -> usize {
        let ptr = Arc::as_ptr(&field.0) as usize;
        if let Some(&idx) = self.ptr_to_aux.get(&ptr) {
            return idx;
        }
        let idx = self.aux_fields.len();
        self.aux_fields.push(field.clone());
        self.ptr_to_aux.insert(ptr, idx);
        idx
    }

    pub fn scan_channel(&mut self, ch: &Channel) {
        match ch {
            Channel::DepthBelowSurface => self.needs_depth = true,
            Channel::Slope | Channel::NormalY => self.needs_normal = true,
            Channel::Curvature => self.needs_curvature = true,
            Channel::Field(f) => {
                self.register_field(f);
            }
            Channel::WorldX | Channel::WorldY | Channel::WorldZ | Channel::Distance => {}
        }
    }

    pub fn scan_condition(&mut self, cond: &Condition) {
        match cond {
            Condition::Always | Condition::Chance { .. } => {}
            Condition::And(cs) | Condition::Or(cs) => {
                for c in cs {
                    self.scan_condition(c);
                }
            }
            Condition::Not(c) => self.scan_condition(c),
            Condition::MaterialTag(_) => self.needs_tags = true,
            Condition::ChannelRange { channel, .. } => self.scan_channel(channel),
            Condition::NormalDot { .. } => self.needs_normal = true,
            Condition::ExposedFace { .. } => self.needs_exposed = true,
        }
    }

    pub fn scan_material(&mut self, mat: &Material) {
        match mat {
            Material::Solid(_) | Material::Skip | Material::Color { .. } | Material::Weighted { .. } => {}
            Material::ChannelGradient { channel, .. } | Material::ChannelRamp { channel, .. } => {
                self.scan_channel(channel);
            }
            Material::Shaded { .. } | Material::Spotlight { .. } => {
                self.needs_normal = true;
            }
            Material::Brush(b) => {
                if matches!(b, BrushEnum::Shaded(_) | BrushEnum::Spotlight(_)) {
                    self.needs_normal = true;
                }
            }
            Material::Layers { layers, default } => {
                for layer in layers {
                    self.scan_condition(&layer.condition);
                    self.scan_material(&layer.material);
                }
                if let Some(d) = default {
                    self.scan_material(d);
                }
            }
            Material::MatchTag { cases, default, .. } => {
                self.needs_tags = true;
                for (_, m) in cases {
                    self.scan_material(m);
                }
                self.scan_material(default);
            }
        }
    }

    pub fn lookup_field(&self, field: &Field) -> usize {
        let ptr = Arc::as_ptr(&field.0) as usize;
        self.ptr_to_aux[&ptr]
    }
}

pub(crate) fn eval_channel(ch: &Channel, ctx: &VoxelContext<'_>, reg: &ChannelRegistry) -> f32 {
    match ch {
        Channel::WorldX => ctx.x as f32 + 0.5,
        Channel::WorldY => ctx.y as f32 + 0.5,
        Channel::WorldZ => ctx.z as f32 + 0.5,
        Channel::Distance => ctx.distance,
        Channel::DepthBelowSurface => ctx.depth_below_surface as f32,
        Channel::Slope => (1.0 - ctx.normal[1].clamp(-1.0, 1.0)).clamp(0.0, 2.0),
        Channel::NormalY => ctx.normal[1],
        Channel::Curvature => ctx.curvature,
        Channel::Field(f) => ctx.aux_fields[reg.lookup_field(f)],
    }
}

pub(crate) fn eval_condition(
    cond: &Condition,
    ctx: &VoxelContext<'_>,
    reg: &ChannelRegistry,
) -> bool {
    match cond {
        Condition::Always => true,
        Condition::And(cs) => cs.iter().all(|c| eval_condition(c, ctx, reg)),
        Condition::Or(cs) => cs.iter().any(|c| eval_condition(c, ctx, reg)),
        Condition::Not(c) => !eval_condition(c, ctx, reg),
        Condition::MaterialTag(id) => ctx.material_id == *id,
        Condition::ChannelRange { channel, min, max } => {
            let v = eval_channel(channel, ctx, reg);
            v.is_finite()
                && min.is_none_or(|lo| v >= lo)
                && max.is_none_or(|hi| v <= hi)
        }
        Condition::NormalDot { dir, min, max } => {
            let d = ctx.normal[0] * dir[0] + ctx.normal[1] * dir[1] + ctx.normal[2] * dir[2];
            min.is_none_or(|lo| d >= lo) && max.is_none_or(|hi| d <= hi)
        }
        Condition::ExposedFace { mask, require_all } => {
            if *mask == face_mask::INTERIOR {
                ctx.exposed_faces == 0
            } else if *require_all {
                (ctx.exposed_faces & *mask) == *mask
            } else {
                (ctx.exposed_faces & *mask) != 0
            }
        }
        Condition::Chance { probability, seed } => {
            hash01_3(ctx.x, ctx.y, ctx.z, *seed) < *probability
        }
    }
}

pub(crate) fn eval_material(
    mat: &Material,
    ctx: &VoxelContext<'_>,
    reg: &ChannelRegistry,
) -> Option<BlockState> {
    match mat {
        Material::Solid(b) => Some(b.clone()),
        Material::Skip => None,
        Material::Color { color, palette } => {
            palette.snap(color, ctx.x, ctx.y, ctx.z).map(BlockState::new)
        }
        Material::ChannelGradient {
            channel,
            stops,
            lo,
            hi,
            palette,
            space,
        } => {
            let v = eval_channel(channel, ctx, reg);
            let t = ((v - lo) / (hi - lo)).clamp(0.0, 1.0) as f64;
            let color = sample_stops(stops, t, *space);
            palette
                .snap(&color, ctx.x, ctx.y, ctx.z)
                .map(BlockState::new)
        }
        Material::ChannelRamp {
            channel,
            lo,
            hi,
            blocks,
            dither,
        } => {
            if blocks.len() == 1 {
                return Some(blocks[0].clone());
            }
            let v = eval_channel(channel, ctx, reg);
            let t = ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
            let pos = t * (blocks.len() - 1) as f32;
            if !dither {
                return Some(blocks[pos.round() as usize].clone());
            }
            let lo_idx = pos.floor() as usize;
            let hi_idx = (lo_idx + 1).min(blocks.len() - 1);
            let frac = pos - pos.floor();
            let threshold = bayer4(ctx.x, ctx.y, ctx.z);
            Some(blocks[if frac > threshold { hi_idx } else { lo_idx }].clone())
        }
        Material::Weighted {
            entries,
            total_weight,
            seed,
        } => {
            let mut r = hash01_3(ctx.x, ctx.y, ctx.z, *seed) * total_weight;
            for (b, w) in entries {
                if r <= *w {
                    return Some(b.clone());
                }
                r -= *w;
            }
            Some(entries[entries.len() - 1].0.clone())
        }
        Material::Shaded {
            base_rgb,
            light_dir,
            ambient,
            palette,
        } => {
            let dot = ctx.normal[0] * light_dir[0]
                + ctx.normal[1] * light_dir[1]
                + ctx.normal[2] * light_dir[2];
            let intensity = ((dot + 1.0) * 0.5 * (1.0 - ambient) + ambient).clamp(0.0, 1.0);
            let color = ExtendedColorData::from_rgb(
                (base_rgb[0] as f32 * intensity) as u8,
                (base_rgb[1] as f32 * intensity) as u8,
                (base_rgb[2] as f32 * intensity) as u8,
            );
            palette
                .snap(&color, ctx.x, ctx.y, ctx.z)
                .map(BlockState::new)
        }
        Material::Spotlight {
            pos,
            dir,
            cone_angle_rad,
            base_rgb,
            ambient,
            palette,
        } => {
            let lx = pos[0] - (ctx.x as f32 + 0.5);
            let ly = pos[1] - (ctx.y as f32 + 0.5);
            let lz = pos[2] - (ctx.z as f32 + 0.5);
            let len = (lx * lx + ly * ly + lz * lz).sqrt();
            let (lx, ly, lz) = if len < 1e-6 {
                (0.0, 1.0, 0.0)
            } else {
                (lx / len, ly / len, lz / len)
            };
            let lambert = (ctx.normal[0] * lx + ctx.normal[1] * ly + ctx.normal[2] * lz).max(0.0);
            let cos_angle = (-lx) * dir[0] + (-ly) * dir[1] + (-lz) * dir[2];
            let angle = cos_angle.clamp(-1.0, 1.0).acos();
            let inner = 0.7 * cone_angle_rad;
            let cone = if angle <= inner {
                1.0
            } else if angle >= *cone_angle_rad {
                0.0
            } else {
                let t = ((cone_angle_rad - angle) / (cone_angle_rad - inner)).clamp(0.0, 1.0);
                t * t * (3.0 - 2.0 * t)
            };
            let intensity = (lambert * cone).max(*ambient);
            let color = ExtendedColorData::from_rgb(
                (base_rgb[0] as f32 * intensity) as u8,
                (base_rgb[1] as f32 * intensity) as u8,
                (base_rgb[2] as f32 * intensity) as u8,
            );
            palette
                .snap(&color, ctx.x, ctx.y, ctx.z)
                .map(BlockState::new)
        }
        Material::Brush(brush) => brush.get_block(
            ctx.x,
            ctx.y,
            ctx.z,
            (ctx.normal[0] as f64, ctx.normal[1] as f64, ctx.normal[2] as f64),
        ),
        Material::Layers { layers, default } => {
            for layer in layers {
                if eval_condition(&layer.condition, ctx, reg) {
                    return eval_material(&layer.material, ctx, reg);
                }
            }
            default
                .as_deref()
                .and_then(|d| eval_material(d, ctx, reg))
        }
        Material::MatchTag {
            cases,
            default,
            blend_seams,
            seed,
        } => {
            let chosen_tag = if *blend_seams
                && ctx.blend_weight > 0.0
                && ctx.blend_material_id != 0
                && ctx.blend_material_id != ctx.material_id
            {
                // Deterministic spatial seam blend using ordered hash threshold
                let threshold = hash01_3(ctx.x, ctx.y, ctx.z, *seed);
                if threshold < ctx.blend_weight {
                    ctx.blend_material_id
                } else {
                    ctx.material_id
                }
            } else {
                ctx.material_id
            };
            for (id, mat) in cases {
                if *id == chosen_tag {
                    return eval_material(mat, ctx, reg);
                }
            }
            eval_material(default, ctx, reg)
        }
    }
}

#[inline]
fn bayer4(x: i32, y: i32, z: i32) -> f32 {
    const BAYER: [[f32; 4]; 4] = [
        [0.0, 8.0, 2.0, 10.0],
        [12.0, 4.0, 14.0, 6.0],
        [3.0, 11.0, 1.0, 9.0],
        [15.0, 7.0, 13.0, 5.0],
    ];
    let bx = (x.wrapping_add(y) & 3) as usize;
    let bz = (z.wrapping_add(y >> 2) & 3) as usize;
    (BAYER[bx][bz] + 0.5) / 16.0
}

fn sample_stops(stops: &[GradientStop], t: f64, space: InterpolationSpace) -> ExtendedColorData {
    if stops.is_empty() {
        return ExtendedColorData::from_rgb(0, 0, 0);
    }
    let t = t.clamp(0.0, 1.0);
    if t <= stops[0].position {
        return stops[0].color;
    }
    let last = &stops[stops.len() - 1];
    if t >= last.position {
        return last.color;
    }
    let (mut a, mut b) = (&stops[0], last);
    for i in 0..stops.len() - 1 {
        if t >= stops[i].position && t <= stops[i + 1].position {
            a = &stops[i];
            b = &stops[i + 1];
            break;
        }
    }
    let lt = if b.position > a.position {
        ((t - a.position) / (b.position - a.position)) as f32
    } else {
        0.0
    };
    match space {
        InterpolationSpace::Rgb => {
            let mix = |i: usize| {
                (a.color.rgb[i] as f32 * (1.0 - lt) + b.color.rgb[i] as f32 * lt) as u8
            };
            ExtendedColorData::from_rgb(mix(0), mix(1), mix(2))
        }
        InterpolationSpace::Oklab => {
            let mut c = a.color;
            for i in 0..3 {
                c.oklab[i] = a.color.oklab[i] * (1.0 - lt) + b.color.oklab[i] * lt;
            }
            c
        }
    }
}
