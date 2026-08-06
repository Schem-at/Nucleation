//! The 3D preview: an orthographic voxel raycast over the block grid.
//!
//! No GPU, no resource pack, no mesh — a ray per pixel walks the grid
//! (Amanatides–Woo, the same traversal the tick engine's explosion
//! line-of-sight uses) to the first occupied cell and shades it by entry
//! face with a touch of depth fog. A few hundred thousand cells render a
//! 640×480 frame in well under a frame's budget in release builds.
//!
//! Colours are a small material table with a stable hash fallback, so an
//! unknown block is *some* consistent colour rather than noise.

use crate::model::VoxelGrid;

/// `0xRRGGBB` for a block name.
///
/// Blockpedia's texture-derived average is the primary source — the same
/// 1177-block table the library ships for palette work — with a small
/// hand table and a stable hash behind it for anything uncached, so an
/// unknown block is *some* consistent colour rather than noise.
pub(crate) fn block_color(name: &str) -> u32 {
    if let Some(facts) = nucleation::blockpedia::BLOCKS.get(name) {
        if let Some(color) = &facts.extras.color {
            let [r, g, b] = color.rgb;
            return (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b);
        }
    }
    let short = name.strip_prefix("minecraft:").unwrap_or(name);
    let table: u32 = match short {
        "stone" | "smooth_stone" | "cobblestone" | "stone_bricks" => 0x8f8f8f,
        "andesite" | "polished_andesite" => 0x888a85,
        "granite" => 0x9f6b58,
        "diorite" => 0xc9c9c6,
        "obsidian" | "crying_obsidian" => 0x1b1226,
        "bedrock" => 0x3a3a3a,
        "dirt" | "rooted_dirt" => 0x8a5a3b,
        "grass_block" => 0x5f9e3a,
        "sand" => 0xdbcf9c,
        "gravel" => 0x7f7c78,
        "water" => 0x3b6ecf,
        "lava" => 0xe06a10,
        "ice" | "packed_ice" | "blue_ice" | "frosted_ice" => 0xa5c8f0,
        "glass" | "tinted_glass" => 0xd8f0f2,
        "slime_block" => 0x6fd66a,
        "honey_block" => 0xe8a933,
        "tnt" => 0xc23b2a,
        "redstone_block" => 0xb01e0e,
        "redstone_wire" => 0x8f1010,
        "redstone_torch" | "redstone_wall_torch" => 0xd94a2a,
        "repeater" | "comparator" => 0xb9b0a8,
        "observer" => 0x5f5c58,
        "piston" | "sticky_piston" => 0x9a8054,
        "piston_head" | "moving_piston" => 0xb08d5a,
        "hopper" | "cauldron" => 0x4a4a4a,
        "chest" | "trapped_chest" | "barrel" => 0x9a6b2f,
        "dropper" | "dispenser" | "furnace" => 0x6f6f6f,
        "target" => 0xd8b8a0,
        "lever" | "tripwire_hook" => 0x8a7a5a,
        "tripwire" | "cobweb" => 0xcfcfcf,
        "rail" | "powered_rail" | "activator_rail" | "detector_rail" => 0x99856a,
        "iron_block" => 0xd8d8d8,
        "gold_block" => 0xf0d24a,
        "diamond_block" => 0x62e6d8,
        "emerald_block" => 0x35c65a,
        "test_block" => 0xc8c0e8,
        "command_block" => 0xb08252,
        "note_block" | "jukebox" => 0x6a4a30,
        "iron_trapdoor" => 0xc0c0c0,
        "soul_sand" => 0x5a4636,
        "netherrack" => 0x7a3230,
        "glowstone" | "shroomlight" => 0xf0c86a,
        "snow" | "snow_block" | "powder_snow" => 0xf4f8fa,
        _ => 0,
    };
    if table != 0 {
        return table;
    }
    if let Some(rest) = short.strip_suffix("_concrete") {
        return dye_color(rest);
    }
    if let Some(rest) = short.strip_suffix("_wool").or_else(|| {
        short
            .strip_suffix("_terracotta")
            .or_else(|| short.strip_suffix("_stained_glass"))
    }) {
        return dye_color(rest);
    }
    if short.ends_with("_planks") || short.ends_with("_log") || short.ends_with("_wood") {
        return 0xa07a48;
    }
    if short.ends_with("_leaves") {
        return 0x3f7a2a;
    }
    // Stable FNV-ish hash → a muted but distinct colour. The same block is
    // the same colour in every build on every run.
    let mut h: u32 = 0x811c_9dc5;
    for b in short.bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    let r = 96 + (h & 0x7f) as u8;
    let g = 96 + ((h >> 8) & 0x7f) as u8;
    let b = 96 + ((h >> 16) & 0x7f) as u8;
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

fn dye_color(dye: &str) -> u32 {
    match dye {
        "white" => 0xe9ecec,
        "orange" => 0xf07613,
        "magenta" => 0xbd44b3,
        "light_blue" => 0x3aafd9,
        "yellow" => 0xf8c527,
        "lime" => 0x70b919,
        "pink" => 0xed8dac,
        "gray" => 0x3e4447,
        "light_gray" => 0x8e8e86,
        "cyan" => 0x158991,
        "purple" => 0x792aac,
        "blue" => 0x35399d,
        "brown" => 0x724728,
        "green" => 0x546d1b,
        "red" => 0xa12722,
        "black" => 0x141519,
        _ => 0x9a9a9a,
    }
}

/// Render one orthographic frame of the grid, orbiting its centre.
///
/// `yaw`/`pitch` are radians; `zoom` 1.0 fits the whole build. Returns RGBA
/// bytes `width * height * 4`, background left transparent so the terminal
/// theme shows through.
pub(crate) fn render(
    grid: &VoxelGrid,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    pan: [f32; 3],
    width: u32,
    height: u32,
) -> image::RgbaImage {
    let (dx, dy, dz) = grid.dims;
    let dims = [dx as f32, dy as f32, dz as f32];
    let center = [dims[0] / 2.0, dims[1] / 2.0, dims[2] / 2.0];
    let radius = (dims[0] * dims[0] + dims[1] * dims[1] + dims[2] * dims[2]).sqrt() / 2.0;

    let (sy, cy) = yaw.sin_cos();
    let (sp, cp) = pitch.sin_cos();
    // Forward points *into* the scene; right/up span the image plane.
    let forward = [cy * cp, -sp, sy * cp];
    let right = [-sy, 0.0, cy];
    let up = [
        right[1] * forward[2] - right[2] * forward[1],
        right[2] * forward[0] - right[0] * forward[2],
        right[0] * forward[1] - right[1] * forward[0],
    ];

    // Perspective, matching the GPU path: eye orbits at a zoom-scaled
    // distance, ~50° vertical field of view. Orthographic frames read as
    // skewed diagrams next to it. The pan is a world-space target offset,
    // frozen at drag time — the same addition the GPU engine's `target`
    // gets, so the engines agree mid-drag and orbits revolve around the
    // panned point instead of swinging it.
    let distance = (2.2 * radius / zoom.max(0.05)).max(2.0);
    let center = [center[0] + pan[0], center[1] + pan[1], center[2] + pan[2]];
    let eye = [
        center[0] - forward[0] * distance,
        center[1] - forward[1] * distance,
        center[2] - forward[2] * distance,
    ];
    let tan_half = (50.0f32.to_radians() / 2.0).tan();
    let aspect = width as f32 / height as f32;

    let mut out = image::RgbaImage::new(width, height);
    for py in 0..height {
        for px in 0..width {
            let u = ((px as f32 + 0.5) / width as f32 - 0.5) * 2.0 * tan_half * aspect;
            let v = (0.5 - (py as f32 + 0.5) / height as f32) * 2.0 * tan_half;
            let mut dir = [
                forward[0] + right[0] * u + up[0] * v,
                forward[1] + right[1] * u + up[1] * v,
                forward[2] + right[2] * u + up[2] * v,
            ];
            let norm = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
            dir = [dir[0] / norm, dir[1] / norm, dir[2] / norm];
            let start = eye;
            if let Some((color, face, depth)) = cast(grid, start, dir, distance + radius * 4.0) {
                let shade = match face {
                    1 => 1.0,  // +y: lit from above
                    4 => 0.55, // -y: undersides darkest
                    0 | 3 => 0.82,
                    _ => 0.68,
                } * (1.0
                    - 0.25 * ((depth - distance + radius) / (radius * 2.0)).clamp(0.0, 1.0));
                let r = ((color >> 16) & 0xff) as f32 * shade;
                let g = ((color >> 8) & 0xff) as f32 * shade;
                let b = (color & 0xff) as f32 * shade;
                out.put_pixel(px, py, image::Rgba([r as u8, g as u8, b as u8, 255]));
            }
        }
    }
    out
}

/// March from `start` along `dir` to the first occupied cell. Returns its
/// colour, which axis-face the ray entered through (0..5 = +x,+y,+z,-x,-y,-z
/// respectively of travel), and the distance travelled.
fn cast(grid: &VoxelGrid, start: [f32; 3], dir: [f32; 3], max_t: f32) -> Option<(u32, u8, f32)> {
    let (dx, dy, dz) = grid.dims;
    let dims = [dx as i64, dy as i64, dz as i64];

    // Clip to the grid's box first so the walk starts on the surface.
    let (mut t0, mut t1) = (0.0f32, max_t);
    for axis in 0..3 {
        if dir[axis].abs() < 1e-6 {
            if start[axis] < 0.0 || start[axis] > dims[axis] as f32 {
                return None;
            }
            continue;
        }
        let a = (0.0 - start[axis]) / dir[axis];
        let b = (dims[axis] as f32 - start[axis]) / dir[axis];
        t0 = t0.max(a.min(b));
        t1 = t1.min(a.max(b));
    }
    if t0 > t1 {
        return None;
    }
    let enter = t0 + 1e-4;
    let pos = [
        start[0] + dir[0] * enter,
        start[1] + dir[1] * enter,
        start[2] + dir[2] * enter,
    ];
    let mut cell = [
        (pos[0].floor() as i64).clamp(0, dims[0] - 1),
        (pos[1].floor() as i64).clamp(0, dims[1] - 1),
        (pos[2].floor() as i64).clamp(0, dims[2] - 1),
    ];
    let mut t_max = [f32::INFINITY; 3];
    let mut t_delta = [f32::INFINITY; 3];
    let mut step = [0i64; 3];
    for axis in 0..3 {
        if dir[axis] > 1e-6 {
            step[axis] = 1;
            t_delta[axis] = 1.0 / dir[axis];
            t_max[axis] = enter + (cell[axis] as f32 + 1.0 - pos[axis]) / dir[axis];
        } else if dir[axis] < -1e-6 {
            step[axis] = -1;
            t_delta[axis] = -1.0 / dir[axis];
            t_max[axis] = enter + (pos[axis] - cell[axis] as f32) / -dir[axis];
        }
    }
    // The face the ray entered the box through seeds the shading axis.
    let mut last_axis = {
        let mut best = 0;
        for axis in 1..3 {
            if dir[axis].abs() > dir[best].abs() {
                best = axis;
            }
        }
        best
    };
    let mut t = enter;
    loop {
        if cell[0] < 0
            || cell[1] < 0
            || cell[2] < 0
            || cell[0] >= dims[0]
            || cell[1] >= dims[1]
            || cell[2] >= dims[2]
        {
            return None;
        }
        let index = ((cell[1] as usize) * dz + cell[2] as usize) * dx + cell[0] as usize;
        let color = grid.cells[index];
        if color != 0 {
            let face = match (last_axis, step[last_axis] > 0) {
                (0, true) => 3,
                (0, false) => 0,
                (1, true) => 4,
                (1, false) => 1,
                (2, true) => 5,
                _ => 2,
            };
            return Some((color, face, t));
        }
        let axis = (0..3).fold(0, |best, a| if t_max[a] < t_max[best] { a } else { best });
        t = t_max[axis];
        if t > t1 {
            return None;
        }
        cell[axis] += step[axis];
        t_max[axis] += t_delta[axis];
        last_axis = axis;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_block() -> VoxelGrid {
        let mut cells = vec![0u32; 27];
        cells[(1 * 3 + 1) * 3 + 1] = 0xff0000;
        VoxelGrid {
            dims: (3, 3, 3),
            cells,
        }
    }

    /// A lone red cube must appear from every yaw — the camera orbits, the
    /// build stays.
    #[test]
    fn a_single_block_is_visible_from_every_angle() {
        let grid = one_block();
        for i in 0..8 {
            let yaw = i as f32 * std::f32::consts::FRAC_PI_4;
            let frame = render(&grid, yaw, 0.6, 1.0, [0.0, 0.0, 0.0], 64, 48);
            let hit = frame.pixels().any(|p| p.0[3] != 0 && p.0[0] > p.0[2]);
            assert!(hit, "yaw step {i}: the block vanished");
        }
    }

    /// Faces shade differently: a top-down look is brighter than a
    /// bottom-up one of the same cube.
    #[test]
    fn top_faces_read_brighter_than_bottom_faces() {
        let grid = one_block();
        let brightest = |frame: &image::RgbaImage| {
            frame
                .pixels()
                .filter(|p| p.0[3] != 0)
                .map(|p| u32::from(p.0[0]))
                .max()
                .unwrap_or(0)
        };
        let above = brightest(&render(&grid, 0.3, 1.2, 1.0, [0.0, 0.0, 0.0], 64, 48));
        let below = brightest(&render(&grid, 0.3, -1.2, 1.0, [0.0, 0.0, 0.0], 64, 48));
        assert!(
            above > below,
            "top lighting must beat bottom lighting, got {above} vs {below}"
        );
    }
}
