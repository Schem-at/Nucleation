use crate::BlockState;
use blockpedia::color::block_palettes::BlockFilter;
use blockpedia::{all_blocks, BlockFacts, ExtendedColorData};
use std::sync::{Arc, OnceLock};

pub struct PaletteBuilder {
    filter: BlockFilter,
}

impl PaletteBuilder {
    pub fn new() -> Self {
        Self {
            filter: BlockFilter::default(),
        }
    }

    pub fn exclude_falling(mut self) -> Self {
        self.filter.exclude_falling = true;
        self
    }

    pub fn exclude_tile_entities(mut self) -> Self {
        self.filter.exclude_tile_entities = true;
        self
    }

    pub fn full_blocks_only(mut self) -> Self {
        self.filter.full_blocks_only = true;
        self
    }

    pub fn exclude_needs_support(mut self) -> Self {
        self.filter.exclude_needs_support = true;
        self
    }

    pub fn exclude_transparent(mut self) -> Self {
        self.filter.exclude_transparent = true;
        self
    }

    pub fn exclude_light_sources(mut self) -> Self {
        self.filter.exclude_light_sources = true;
        self
    }

    pub fn survival_obtainable_only(mut self) -> Self {
        self.filter.survival_obtainable_only = true;
        self
    }

    pub fn exclude_keyword(mut self, keyword: &str) -> Self {
        self.filter.exclude_patterns.push(keyword.to_string());
        self
    }

    pub fn include_keyword(mut self, keyword: &str) -> Self {
        self.filter.include_patterns.push(keyword.to_string());
        self
    }

    pub fn build(self) -> BlockPalette {
        BlockPalette::new_from_filter(self.filter)
    }
}

/// A palette of blocks used for color matching
pub struct BlockPalette {
    blocks: Vec<(ExtendedColorData, String)>,
}

impl BlockPalette {
    pub fn new_all() -> Self {
        Self::new_filtered(|_| true)
    }

    pub fn builder() -> PaletteBuilder {
        PaletteBuilder::new()
    }

    /// Create a palette using a blockpedia BlockFilter
    pub fn new_from_filter(filter: BlockFilter) -> Self {
        Self::new_filtered(|f| filter.allows_block(f))
    }

    /// Create a palette containing only solid blocks (no transparent, gravity, etc.)
    pub fn new_solid() -> Self {
        Self::new_from_filter(BlockFilter::solid_blocks_only())
    }

    /// Create a palette containing only structural blocks (conservative set)
    pub fn new_structural() -> Self {
        Self::new_from_filter(BlockFilter::structural_blocks_only())
    }

    /// Create a palette containing decorative blocks (allows stairs/slabs but no tile entities)
    pub fn new_decorative() -> Self {
        Self::new_from_filter(BlockFilter::decorative_blocks())
    }

    pub fn new_filtered<F>(filter: F) -> Self
    where
        F: Fn(&BlockFacts) -> bool,
    {
        let mut blocks = Vec::new();
        for facts in all_blocks() {
            if let Some(c) = &facts.extras.color {
                if filter(facts) {
                    blocks.push((c.to_extended(), facts.id.to_string()));
                }
            }
        }
        Self { blocks }
    }

    /// Create a palette containing only concrete blocks
    pub fn new_concrete() -> Self {
        Self::new_filtered(|f| f.id.contains("concrete") && !f.id.contains("powder"))
    }

    /// Create a palette containing only wool blocks
    pub fn new_wool() -> Self {
        Self::new_filtered(|f| f.id.contains("wool"))
    }

    /// Create a palette containing only terracotta blocks
    pub fn new_terracotta() -> Self {
        Self::new_filtered(|f| f.id.contains("terracotta") && !f.id.contains("glazed"))
    }

    /// Create a palette containing only grayscale blocks
    pub fn new_grayscale() -> Self {
        Self::new_filtered(|f| {
            let id = &f.id;
            id.contains("white")
                || id.contains("gray")
                || id.contains("black")
                || id.contains("stone")
                || id.contains("basalt")
                || id.contains("andesite")
                || id.contains("diorite")
                || id.contains("tuff")
                || id.contains("deepslate")
                || id.contains("bedrock")
        })
    }

    pub fn find_closest(&self, target: &ExtendedColorData) -> Option<String> {
        let mut best_dist = f32::MAX;
        let mut best_id = None;
        for (color, id) in &self.blocks {
            let dist = target.distance_oklab(color);
            if dist < best_dist {
                best_dist = dist;
                best_id = Some(id);
            }
        }
        best_id.cloned()
    }
}

// Global default palette
static DEFAULT_PALETTE: OnceLock<Arc<BlockPalette>> = OnceLock::new();

fn get_default_palette() -> Arc<BlockPalette> {
    DEFAULT_PALETTE
        .get_or_init(|| Arc::new(BlockPalette::new_all()))
        .clone()
}

pub trait Brush {
    /// Get the block to place at the given coordinates, optionally using the surface normal
    fn get_block(&self, x: i32, y: i32, z: i32, normal: (f64, f64, f64)) -> Option<BlockState>;
}

/// A brush that places a single specific block
#[derive(Clone)]
pub struct SolidBrush {
    block: BlockState,
}

impl SolidBrush {
    pub fn new(block: BlockState) -> Self {
        Self { block }
    }
}

impl Brush for SolidBrush {
    fn get_block(&self, _x: i32, _y: i32, _z: i32, _normal: (f64, f64, f64)) -> Option<BlockState> {
        Some(self.block.clone())
    }
}

/// A brush that places blocks closest to a specific color
#[derive(Clone)]
pub struct ColorBrush {
    target_color: ExtendedColorData,
    palette: Arc<BlockPalette>,
}

impl ColorBrush {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            target_color: ExtendedColorData::from_rgb(r, g, b),
            palette: get_default_palette(),
        }
    }

    pub fn with_palette(r: u8, g: u8, b: u8, palette: Arc<BlockPalette>) -> Self {
        Self {
            target_color: ExtendedColorData::from_rgb(r, g, b),
            palette,
        }
    }
}

impl Brush for ColorBrush {
    fn get_block(&self, _x: i32, _y: i32, _z: i32, _normal: (f64, f64, f64)) -> Option<BlockState> {
        self.palette
            .find_closest(&self.target_color)
            .map(|id| BlockState::new(id))
    }
}

#[derive(Clone, Copy)]
pub enum InterpolationSpace {
    Rgb,
    Oklab,
}

/// A brush that interpolates color linearly between two points
#[derive(Clone)]
pub struct LinearGradientBrush {
    start_pos: (f64, f64, f64),
    end_pos: (f64, f64, f64),
    start_color: ExtendedColorData,
    end_color: ExtendedColorData,
    palette: Arc<BlockPalette>,
    length_sq: f64,
    space: InterpolationSpace,
}

impl LinearGradientBrush {
    pub fn new(
        p1: (i32, i32, i32),
        c1: (u8, u8, u8),
        p2: (i32, i32, i32),
        c2: (u8, u8, u8),
    ) -> Self {
        let start_pos = (p1.0 as f64, p1.1 as f64, p1.2 as f64);
        let end_pos = (p2.0 as f64, p2.1 as f64, p2.2 as f64);
        let dx = end_pos.0 - start_pos.0;
        let dy = end_pos.1 - start_pos.1;
        let dz = end_pos.2 - start_pos.2;

        Self {
            start_pos,
            end_pos,
            start_color: ExtendedColorData::from_rgb(c1.0, c1.1, c1.2),
            end_color: ExtendedColorData::from_rgb(c2.0, c2.1, c2.2),
            palette: get_default_palette(),
            length_sq: dx * dx + dy * dy + dz * dz,
            space: InterpolationSpace::Rgb,
        }
    }

    pub fn with_space(mut self, space: InterpolationSpace) -> Self {
        self.space = space;
        self
    }

    pub fn with_palette(mut self, palette: Arc<BlockPalette>) -> Self {
        self.palette = palette;
        self
    }
}

impl Brush for LinearGradientBrush {
    fn get_block(&self, x: i32, y: i32, z: i32, _normal: (f64, f64, f64)) -> Option<BlockState> {
        let px = x as f64;
        let py = y as f64;
        let pz = z as f64;

        // Project point onto line segment
        let dx = self.end_pos.0 - self.start_pos.0;
        let dy = self.end_pos.1 - self.start_pos.1;
        let dz = self.end_pos.2 - self.start_pos.2;

        let v_x = px - self.start_pos.0;
        let v_y = py - self.start_pos.1;
        let v_z = pz - self.start_pos.2;

        let dot = v_x * dx + v_y * dy + v_z * dz;
        let t = (dot / self.length_sq).clamp(0.0, 1.0);

        let color = match self.space {
            InterpolationSpace::Rgb => {
                let r = (self.start_color.rgb[0] as f64 * (1.0 - t)
                    + self.end_color.rgb[0] as f64 * t) as u8;
                let g = (self.start_color.rgb[1] as f64 * (1.0 - t)
                    + self.end_color.rgb[1] as f64 * t) as u8;
                let b = (self.start_color.rgb[2] as f64 * (1.0 - t)
                    + self.end_color.rgb[2] as f64 * t) as u8;
                ExtendedColorData::from_rgb(r, g, b)
            }
            InterpolationSpace::Oklab => {
                let l = self.start_color.oklab[0] * (1.0 - t) as f32
                    + self.end_color.oklab[0] * t as f32;
                let a = self.start_color.oklab[1] * (1.0 - t) as f32
                    + self.end_color.oklab[1] * t as f32;
                let b_val = self.start_color.oklab[2] * (1.0 - t) as f32
                    + self.end_color.oklab[2] * t as f32;

                // We construct a dummy ExtendedColorData that has the correct Oklab values.
                // Note: find_closest ONLY uses oklab, so the other fields can be junk or approximated.
                // But for correctness if we ever change that, let's just zero them or clone start.
                let mut c = self.start_color;
                c.oklab = [l, a, b_val];
                c
            }
        };

        self.palette
            .find_closest(&color)
            .map(|id| BlockState::new(id))
    }
}

#[derive(Clone, Copy)]
pub struct GradientStop {
    pub position: f64, // 0.0 to 1.0
    pub color: ExtendedColorData,
}

#[derive(Clone)]
pub struct MultiPointGradientBrush {
    start_pos: (f64, f64, f64),
    end_pos: (f64, f64, f64),
    stops: Vec<GradientStop>,
    palette: Arc<BlockPalette>,
    length_sq: f64,
    space: InterpolationSpace,
}

impl MultiPointGradientBrush {
    pub fn new(p1: (i32, i32, i32), p2: (i32, i32, i32), stops: Vec<(f64, (u8, u8, u8))>) -> Self {
        let start_pos = (p1.0 as f64, p1.1 as f64, p1.2 as f64);
        let end_pos = (p2.0 as f64, p2.1 as f64, p2.2 as f64);
        let dx = end_pos.0 - start_pos.0;
        let dy = end_pos.1 - start_pos.1;
        let dz = end_pos.2 - start_pos.2;

        let mut gradient_stops: Vec<GradientStop> = stops
            .into_iter()
            .map(|(pos, rgb)| GradientStop {
                position: pos.clamp(0.0, 1.0),
                color: ExtendedColorData::from_rgb(rgb.0, rgb.1, rgb.2),
            })
            .collect();

        // Sort stops by position
        gradient_stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());

        Self {
            start_pos,
            end_pos,
            stops: gradient_stops,
            palette: get_default_palette(),
            length_sq: dx * dx + dy * dy + dz * dz,
            space: InterpolationSpace::Rgb,
        }
    }

    pub fn with_space(mut self, space: InterpolationSpace) -> Self {
        self.space = space;
        self
    }

    pub fn with_palette(mut self, palette: Arc<BlockPalette>) -> Self {
        self.palette = palette;
        self
    }
}

impl Brush for MultiPointGradientBrush {
    fn get_block(&self, x: i32, y: i32, z: i32, _normal: (f64, f64, f64)) -> Option<BlockState> {
        let px = x as f64;
        let py = y as f64;
        let pz = z as f64;

        let dx = self.end_pos.0 - self.start_pos.0;
        let dy = self.end_pos.1 - self.start_pos.1;
        let dz = self.end_pos.2 - self.start_pos.2;

        let v_x = px - self.start_pos.0;
        let v_y = py - self.start_pos.1;
        let v_z = pz - self.start_pos.2;

        let dot = v_x * dx + v_y * dy + v_z * dz;
        let t = (dot / self.length_sq).clamp(0.0, 1.0);

        // Find stops
        if self.stops.is_empty() {
            return None;
        }

        let mut start_stop = &self.stops[0];
        let mut end_stop = &self.stops[self.stops.len() - 1];

        // If t is before first stop
        if t <= start_stop.position {
            return self
                .palette
                .find_closest(&start_stop.color)
                .map(|id| BlockState::new(id));
        }
        // If t is after last stop
        if t >= end_stop.position {
            return self
                .palette
                .find_closest(&end_stop.color)
                .map(|id| BlockState::new(id));
        }

        // Find the two stops surrounding t
        for i in 0..self.stops.len() - 1 {
            if t >= self.stops[i].position && t <= self.stops[i + 1].position {
                start_stop = &self.stops[i];
                end_stop = &self.stops[i + 1];
                break;
            }
        }

        // Remap t to [0, 1] between stops
        let local_t = (t - start_stop.position) / (end_stop.position - start_stop.position);

        let color = match self.space {
            InterpolationSpace::Rgb => {
                let r = (start_stop.color.rgb[0] as f64 * (1.0 - local_t)
                    + end_stop.color.rgb[0] as f64 * local_t) as u8;
                let g = (start_stop.color.rgb[1] as f64 * (1.0 - local_t)
                    + end_stop.color.rgb[1] as f64 * local_t) as u8;
                let b = (start_stop.color.rgb[2] as f64 * (1.0 - local_t)
                    + end_stop.color.rgb[2] as f64 * local_t) as u8;
                ExtendedColorData::from_rgb(r, g, b)
            }
            InterpolationSpace::Oklab => {
                let l = start_stop.color.oklab[0] * (1.0 - local_t) as f32
                    + end_stop.color.oklab[0] * local_t as f32;
                let a = start_stop.color.oklab[1] * (1.0 - local_t) as f32
                    + end_stop.color.oklab[1] * local_t as f32;
                let b_val = start_stop.color.oklab[2] * (1.0 - local_t) as f32
                    + end_stop.color.oklab[2] * local_t as f32;

                let mut c = start_stop.color;
                c.oklab = [l, a, b_val];
                c
            }
        };

        self.palette
            .find_closest(&color)
            .map(|id| BlockState::new(id))
    }
}

/// A brush that interpolates color bilinearly on a quad defined by 4 corners.
///
/// The quad is defined by 3 points: Origin (P00), Top-Right (P10), Bottom-Left (P01).
/// P11 is implicitly P10 + P01 - P00 (parallelogram) or explicitly P11.
///
/// For simplicity, we define it by Origin and two vectors (u_vec, v_vec) which form the plane basis.
/// We project points onto this plane to find (u, v) coordinates.
///
/// Colors:
/// c00 = Color at Origin (u=0, v=0)
/// c10 = Color at End of U (u=1, v=0)
/// c01 = Color at End of V (u=0, v=1)
/// c11 = Color at Opposite Corner (u=1, v=1)
#[derive(Clone)]
pub struct BilinearGradientBrush {
    origin: (f64, f64, f64),
    u_vec: (f64, f64, f64),
    v_vec: (f64, f64, f64),
    u_len_sq: f64,
    v_len_sq: f64,
    c00: ExtendedColorData,
    c10: ExtendedColorData,
    c01: ExtendedColorData,
    c11: ExtendedColorData,
    palette: Arc<BlockPalette>,
    space: InterpolationSpace,
}

impl BilinearGradientBrush {
    pub fn new(
        origin: (i32, i32, i32),
        u_point: (i32, i32, i32),
        v_point: (i32, i32, i32),
        c00: (u8, u8, u8), // Origin
        c10: (u8, u8, u8), // U-end
        c01: (u8, u8, u8), // V-end
        c11: (u8, u8, u8), // Opposite corner
    ) -> Self {
        let origin_f = (origin.0 as f64, origin.1 as f64, origin.2 as f64);
        let u_vec = (
            u_point.0 as f64 - origin_f.0,
            u_point.1 as f64 - origin_f.1,
            u_point.2 as f64 - origin_f.2,
        );
        let v_vec = (
            v_point.0 as f64 - origin_f.0,
            v_point.1 as f64 - origin_f.1,
            v_point.2 as f64 - origin_f.2,
        );

        let u_len_sq = u_vec.0 * u_vec.0 + u_vec.1 * u_vec.1 + u_vec.2 * u_vec.2;
        let v_len_sq = v_vec.0 * v_vec.0 + v_vec.1 * v_vec.1 + v_vec.2 * v_vec.2;

        Self {
            origin: origin_f,
            u_vec,
            v_vec,
            u_len_sq,
            v_len_sq,
            c00: ExtendedColorData::from_rgb(c00.0, c00.1, c00.2),
            c10: ExtendedColorData::from_rgb(c10.0, c10.1, c10.2),
            c01: ExtendedColorData::from_rgb(c01.0, c01.1, c01.2),
            c11: ExtendedColorData::from_rgb(c11.0, c11.1, c11.2),
            palette: get_default_palette(),
            space: InterpolationSpace::Rgb,
        }
    }

    pub fn with_space(mut self, space: InterpolationSpace) -> Self {
        self.space = space;
        self
    }

    pub fn with_palette(mut self, palette: Arc<BlockPalette>) -> Self {
        self.palette = palette;
        self
    }
}

#[derive(Clone, Copy)]
pub struct GradientPoint {
    pub position: (f64, f64, f64),
    pub color: ExtendedColorData,
}

/// A brush that interpolates color based on arbitrary points in 3D space using Inverse Distance Weighting (IDW).
#[derive(Clone)]
pub struct PointGradientBrush {
    points: Vec<GradientPoint>,
    palette: Arc<BlockPalette>,
    space: InterpolationSpace,
    falloff: f64, // Power parameter for IDW (typically 2.0)
}

impl PointGradientBrush {
    pub fn new(points: Vec<((i32, i32, i32), (u8, u8, u8))>) -> Self {
        let gradient_points = points
            .into_iter()
            .map(|(pos, rgb)| GradientPoint {
                position: (pos.0 as f64, pos.1 as f64, pos.2 as f64),
                color: ExtendedColorData::from_rgb(rgb.0, rgb.1, rgb.2),
            })
            .collect();

        Self {
            points: gradient_points,
            palette: get_default_palette(),
            space: InterpolationSpace::Rgb,
            falloff: 2.0,
        }
    }

    pub fn with_space(mut self, space: InterpolationSpace) -> Self {
        self.space = space;
        self
    }

    pub fn with_palette(mut self, palette: Arc<BlockPalette>) -> Self {
        self.palette = palette;
        self
    }

    pub fn with_decay(mut self, decay: f64) -> Self {
        self.falloff = decay;
        self
    }

    pub fn with_falloff(mut self, falloff: f64) -> Self {
        self.falloff = falloff;
        self
    }
}

impl Brush for PointGradientBrush {
    fn get_block(&self, x: i32, y: i32, z: i32, _normal: (f64, f64, f64)) -> Option<BlockState> {
        if self.points.is_empty() {
            return None;
        }

        let px = x as f64;
        let py = y as f64;
        let pz = z as f64;

        let mut sum_r = 0.0;
        let mut sum_g = 0.0;
        let mut sum_b = 0.0;

        let mut sum_l = 0.0;
        let mut sum_a = 0.0;
        let mut sum_ok_b = 0.0;

        let mut total_weight = 0.0;

        for point in &self.points {
            let dx = px - point.position.0;
            let dy = py - point.position.1;
            let dz = pz - point.position.2;
            let dist_sq = dx * dx + dy * dy + dz * dz;
            let dist = dist_sq.sqrt();

            if dist < 1e-6 {
                return self
                    .palette
                    .find_closest(&point.color)
                    .map(|id| BlockState::new(id));
            }

            let weight = 1.0 / dist.powf(self.falloff);
            total_weight += weight;

            match self.space {
                InterpolationSpace::Rgb => {
                    sum_r += point.color.rgb[0] as f64 * weight;
                    sum_g += point.color.rgb[1] as f64 * weight;
                    sum_b += point.color.rgb[2] as f64 * weight;
                }
                InterpolationSpace::Oklab => {
                    sum_l += point.color.oklab[0] as f64 * weight;
                    sum_a += point.color.oklab[1] as f64 * weight;
                    sum_ok_b += point.color.oklab[2] as f64 * weight;
                }
            }
        }

        let color = if total_weight > 0.0 {
            match self.space {
                InterpolationSpace::Rgb => {
                    let r = (sum_r / total_weight) as u8;
                    let g = (sum_g / total_weight) as u8;
                    let b = (sum_b / total_weight) as u8;
                    ExtendedColorData::from_rgb(r, g, b)
                }
                InterpolationSpace::Oklab => {
                    let l = (sum_l / total_weight) as f32;
                    let a = (sum_a / total_weight) as f32;
                    let b = (sum_ok_b / total_weight) as f32;

                    let mut c = self.points[0].color; // Dummy clone for layout
                    c.oklab = [l, a, b];
                    c
                }
            }
        } else {
            // Should be unreachable if points is not empty, but fallback to first point
            self.points[0].color
        };

        self.palette
            .find_closest(&color)
            .map(|id| BlockState::new(id))
    }
}

impl Brush for BilinearGradientBrush {
    fn get_block(&self, x: i32, y: i32, z: i32, _normal: (f64, f64, f64)) -> Option<BlockState> {
        // Project point onto the two axes
        let px = x as f64 - self.origin.0;
        let py = y as f64 - self.origin.1;
        let pz = z as f64 - self.origin.2;

        // u = P . U / |U|^2
        let u = if self.u_len_sq > 0.0 {
            (px * self.u_vec.0 + py * self.u_vec.1 + pz * self.u_vec.2) / self.u_len_sq
        } else {
            0.0
        }
        .clamp(0.0, 1.0);

        // v = P . V / |V|^2
        let v = if self.v_len_sq > 0.0 {
            (px * self.v_vec.0 + py * self.v_vec.1 + pz * self.v_vec.2) / self.v_len_sq
        } else {
            0.0
        }
        .clamp(0.0, 1.0);

        // Bilinear interpolation
        // C(u, v) = lerp(lerp(c00, c10, u), lerp(c01, c11, u), v)

        let color = match self.space {
            InterpolationSpace::Rgb => {
                // Top edge
                let r_top = self.c00.rgb[0] as f64 * (1.0 - u) + self.c10.rgb[0] as f64 * u;
                let g_top = self.c00.rgb[1] as f64 * (1.0 - u) + self.c10.rgb[1] as f64 * u;
                let b_top = self.c00.rgb[2] as f64 * (1.0 - u) + self.c10.rgb[2] as f64 * u;

                // Bottom edge
                let r_bot = self.c01.rgb[0] as f64 * (1.0 - u) + self.c11.rgb[0] as f64 * u;
                let g_bot = self.c01.rgb[1] as f64 * (1.0 - u) + self.c11.rgb[1] as f64 * u;
                let b_bot = self.c01.rgb[2] as f64 * (1.0 - u) + self.c11.rgb[2] as f64 * u;

                // Final
                let r = (r_top * (1.0 - v) + r_bot * v) as u8;
                let g = (g_top * (1.0 - v) + g_bot * v) as u8;
                let b = (b_top * (1.0 - v) + b_bot * v) as u8;

                ExtendedColorData::from_rgb(r, g, b)
            }
            InterpolationSpace::Oklab => {
                // Similar logic but in Oklab space
                let l_top = self.c00.oklab[0] * (1.0 - u) as f32 + self.c10.oklab[0] * u as f32;
                let a_top = self.c00.oklab[1] * (1.0 - u) as f32 + self.c10.oklab[1] * u as f32;
                let b_top = self.c00.oklab[2] * (1.0 - u) as f32 + self.c10.oklab[2] * u as f32;

                let l_bot = self.c01.oklab[0] * (1.0 - u) as f32 + self.c11.oklab[0] * u as f32;
                let a_bot = self.c01.oklab[1] * (1.0 - u) as f32 + self.c11.oklab[1] * u as f32;
                let b_bot = self.c01.oklab[2] * (1.0 - u) as f32 + self.c11.oklab[2] * u as f32;

                let l = l_top * (1.0 - v) as f32 + l_bot * v as f32;
                let a = a_top * (1.0 - v) as f32 + a_bot * v as f32;
                let b = b_top * (1.0 - v) as f32 + b_bot * v as f32;

                let mut c = self.c00;
                c.oklab = [l, a, b];
                c
            }
        };

        self.palette
            .find_closest(&color)
            .map(|id| BlockState::new(id))
    }
}

/// A brush that shades blocks based on surface normal relative to a light source
#[derive(Clone)]
pub struct ShadedBrush {
    base_color: ExtendedColorData,
    light_dir: (f64, f64, f64),
    palette: Arc<BlockPalette>,
}

impl ShadedBrush {
    pub fn new(base_rgb: (u8, u8, u8), light_dir: (f64, f64, f64)) -> Self {
        // Normalize light dir
        let len =
            (light_dir.0 * light_dir.0 + light_dir.1 * light_dir.1 + light_dir.2 * light_dir.2)
                .sqrt();
        let normalized_dir = if len == 0.0 {
            (0.0, 1.0, 0.0)
        } else {
            (light_dir.0 / len, light_dir.1 / len, light_dir.2 / len)
        };

        Self {
            base_color: ExtendedColorData::from_rgb(base_rgb.0, base_rgb.1, base_rgb.2),
            light_dir: normalized_dir,
            palette: get_default_palette(),
        }
    }

    pub fn with_palette(mut self, palette: Arc<BlockPalette>) -> Self {
        self.palette = palette;
        self
    }
}

impl Brush for ShadedBrush {
    fn get_block(&self, _x: i32, _y: i32, _z: i32, normal: (f64, f64, f64)) -> Option<BlockState> {
        // Simple Lambertian shading: dot(N, L)
        // Range [-1, 1], map to brightness [0.2, 1.0] for example
        let dot =
            normal.0 * self.light_dir.0 + normal.1 * self.light_dir.1 + normal.2 * self.light_dir.2;

        // Map [-1, 1] to [0.3, 1.0] (ambient light)
        let intensity = ((dot + 1.0) / 2.0 * 0.7 + 0.3).clamp(0.0, 1.0);

        let r = (self.base_color.rgb[0] as f64 * intensity) as u8;
        let g = (self.base_color.rgb[1] as f64 * intensity) as u8;
        let b = (self.base_color.rgb[2] as f64 * intensity) as u8;

        let color = ExtendedColorData::from_rgb(r, g, b);

        self.palette
            .find_closest(&color)
            .map(|id| BlockState::new(id))
    }
}
