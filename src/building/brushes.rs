use crate::BlockState;
use blockpedia::{all_blocks, ExtendedColorData};
use std::sync::{Arc, OnceLock};

/// A palette of blocks used for color matching
pub struct BlockPalette {
    blocks: Vec<(ExtendedColorData, String)>,
}

impl BlockPalette {
    pub fn new_all() -> Self {
        let mut blocks = Vec::new();
        for facts in all_blocks() {
            if let Some(c) = &facts.extras.color {
                blocks.push((c.to_extended(), facts.id.to_string()));
            }
        }
        Self { blocks }
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

pub enum InterpolationSpace {
    Rgb,
    Oklab,
}

/// A brush that interpolates color linearly between two points
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
                let r = (self.start_color.rgb[0] as f64 * (1.0 - t) + self.end_color.rgb[0] as f64 * t) as u8;
                let g = (self.start_color.rgb[1] as f64 * (1.0 - t) + self.end_color.rgb[1] as f64 * t) as u8;
                let b = (self.start_color.rgb[2] as f64 * (1.0 - t) + self.end_color.rgb[2] as f64 * t) as u8;
                ExtendedColorData::from_rgb(r, g, b)
            }
            InterpolationSpace::Oklab => {
                let l = self.start_color.oklab[0] * (1.0 - t) as f32 + self.end_color.oklab[0] * t as f32;
                let a = self.start_color.oklab[1] * (1.0 - t) as f32 + self.end_color.oklab[1] * t as f32;
                let b_val = self.start_color.oklab[2] * (1.0 - t) as f32 + self.end_color.oklab[2] * t as f32;
                
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

pub struct MultiPointGradientBrush {
    start_pos: (f64, f64, f64),
    end_pos: (f64, f64, f64),
    stops: Vec<GradientStop>,
    palette: Arc<BlockPalette>,
    length_sq: f64,
    space: InterpolationSpace,
}

impl MultiPointGradientBrush {
    pub fn new(
        p1: (i32, i32, i32),
        p2: (i32, i32, i32),
        stops: Vec<(f64, (u8, u8, u8))>,
    ) -> Self {
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
             return self.palette.find_closest(&start_stop.color).map(|id| BlockState::new(id));
        }
        // If t is after last stop
        if t >= end_stop.position {
             return self.palette.find_closest(&end_stop.color).map(|id| BlockState::new(id));
        }

        // Find the two stops surrounding t
        for i in 0..self.stops.len() - 1 {
            if t >= self.stops[i].position && t <= self.stops[i+1].position {
                start_stop = &self.stops[i];
                end_stop = &self.stops[i+1];
                break;
            }
        }

        // Remap t to [0, 1] between stops
        let local_t = (t - start_stop.position) / (end_stop.position - start_stop.position);

        let color = match self.space {
            InterpolationSpace::Rgb => {
                let r = (start_stop.color.rgb[0] as f64 * (1.0 - local_t) + end_stop.color.rgb[0] as f64 * local_t) as u8;
                let g = (start_stop.color.rgb[1] as f64 * (1.0 - local_t) + end_stop.color.rgb[1] as f64 * local_t) as u8;
                let b = (start_stop.color.rgb[2] as f64 * (1.0 - local_t) + end_stop.color.rgb[2] as f64 * local_t) as u8;
                ExtendedColorData::from_rgb(r, g, b)
            }
            InterpolationSpace::Oklab => {
                let l = start_stop.color.oklab[0] * (1.0 - local_t) as f32 + end_stop.color.oklab[0] * local_t as f32;
                let a = start_stop.color.oklab[1] * (1.0 - local_t) as f32 + end_stop.color.oklab[1] * local_t as f32;
                let b_val = start_stop.color.oklab[2] * (1.0 - local_t) as f32 + end_stop.color.oklab[2] * local_t as f32;
                
                let mut c = start_stop.color; 
                c.oklab = [l, a, b_val];
                c
            }
        };

        self.palette.find_closest(&color).map(|id| BlockState::new(id))
    }
}


/// A brush that shades blocks based on surface normal relative to a light source
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
