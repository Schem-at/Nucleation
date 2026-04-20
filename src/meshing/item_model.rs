//! Minecraft item model export via plane-based approach.
//!
//! Slices a schematic into 2D planes per direction, composites block face textures
//! into per-plane PNG images, and generates a JSON item model with one thin element
//! per plane. Far more efficient than per-block elements (max ~288 elements vs thousands).

use std::collections::{HashMap, HashSet};
use std::io::Write;

use schematic_mesher::resolver::{resolve_block, ModelResolver};
use schematic_mesher::resource_pack::TextureData;
use schematic_mesher::{Direction, InputBlock, ResourcePack};
use serde::{Deserialize, Serialize};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use super::{MeshError, ResourcePackSource, Result};
use crate::{Region, UniversalSchematic};

/// Maximum dimension for Minecraft item models (coordinate range: -16 to 32 = 48).
const MAX_DIM: i32 = 48;

/// Maximum atlas page dimension in pixels.
/// Plane chunk textures are packed into atlas pages of up to this size.
/// Chunks exceeding this in any dimension are split before packing.
const ATLAS_PAGE_SIZE: u32 = 4096;

/// Scale mode for item model generation.
///
/// Controls how schematic block coordinates map to Minecraft model coordinates.
/// The scale factor represents **blocks per model unit**:
/// - `1.0` = 1 block = 1 model unit (max 48 blocks per axis)
/// - `2.0` = 2 blocks per model unit (max 96 blocks, everything half-size)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemModelScale {
    /// Automatically compute uniform scale to fit within 48 model units.
    Auto,
    /// Same scale factor on all axes (clamped to >= 1.0).
    Uniform(f32),
    /// Per-axis scale factors (x, y, z) (each clamped to >= 1.0).
    NonUniform(f32, f32, f32),
}

impl Default for ItemModelScale {
    fn default() -> Self {
        Self::Auto
    }
}

/// Resolve scale enum to concrete (sx, sy, sz) tuple.
fn resolve_scale(scale: &ItemModelScale, w: i32, h: i32, d: i32) -> (f32, f32, f32) {
    match scale {
        ItemModelScale::Auto => {
            let s = (w.max(h).max(d) as f32 / 48.0).max(1.0);
            (s, s, s)
        }
        ItemModelScale::Uniform(s) => {
            let s = s.max(1.0);
            (s, s, s)
        }
        ItemModelScale::NonUniform(sx, sy, sz) => (sx.max(1.0), sy.max(1.0), sz.max(1.0)),
    }
}

/// Configuration for item model generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemModelConfig {
    /// Model name (used in file paths, e.g., "my_schematic").
    pub model_name: String,
    /// Resource pack namespace (default: "nucleation").
    pub namespace: String,
    /// Center the schematic in the model bounds (default: true).
    pub center: bool,
    /// Pixels per block face (default: 16).
    pub texture_resolution: u32,
    /// Minecraft item to bind to (default: "paper"). Used for item definition JSON.
    pub item: String,
    /// Custom model data string value (default: "1"). Used to select this model via
    /// `/give @s minecraft:paper[custom_model_data={strings:["1"]}]`.
    pub custom_model_data: String,
    /// Scale mode (default: Auto). Controls how blocks map to model coordinates.
    pub scale: ItemModelScale,
}

impl Default for ItemModelConfig {
    fn default() -> Self {
        Self {
            model_name: "schematic".to_string(),
            namespace: "nucleation".to_string(),
            center: true,
            texture_resolution: 16,
            item: "paper".to_string(),
            custom_model_data: "1".to_string(),
            scale: ItemModelScale::Auto,
        }
    }
}

impl ItemModelConfig {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
            ..Default::default()
        }
    }

    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    pub fn with_center(mut self, center: bool) -> Self {
        self.center = center;
        self
    }

    pub fn with_texture_resolution(mut self, resolution: u32) -> Self {
        self.texture_resolution = resolution;
        self
    }

    pub fn with_item(mut self, item: impl Into<String>) -> Self {
        self.item = item.into();
        self
    }

    pub fn with_custom_model_data(mut self, cmd: impl Into<String>) -> Self {
        self.custom_model_data = cmd.into();
        self
    }

    pub fn with_scale(mut self, scale: ItemModelScale) -> Self {
        self.scale = scale;
        self
    }
}

/// Statistics about the generated item model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemModelStats {
    /// Number of elements in the model.
    pub element_count: usize,
    /// Number of texture images generated.
    pub texture_count: usize,
    /// Total planes across all 6 directions.
    pub plane_count: usize,
    /// Schematic dimensions (width, height, depth).
    pub dimensions: (i32, i32, i32),
    /// Resolved scale factors (sx, sy, sz).
    pub scale: (f32, f32, f32),
}

/// Result of item model generation.
pub struct ItemModelResult {
    /// Minecraft item model JSON.
    pub model_json: String,
    /// Map of texture name (e.g., "north_3") to PNG bytes.
    pub textures: HashMap<String, Vec<u8>>,
    /// Generation statistics.
    pub stats: ItemModelStats,
    /// Config used for generation (needed for ZIP paths).
    config: ItemModelConfig,
}

impl ItemModelResult {
    /// Package as a complete Minecraft resource pack ZIP.
    ///
    /// Structure:
    /// ```text
    /// pack.mcmeta
    /// assets/minecraft/items/{item}.json
    /// assets/{namespace}/models/item/{model_name}.json
    /// assets/{namespace}/textures/item/{model_name}/{tex_name}.png
    /// ```
    pub fn to_resource_pack_zip(&self) -> Result<Vec<u8>> {
        build_resource_pack(&[self])
    }
}

/// Build a Minecraft resource pack ZIP from one or more item model results.
///
/// Merges all models and textures into a single ZIP. Item definitions are grouped
/// by item type — multiple schematics bound to the same item (e.g., paper) will
/// share one item definition file with multiple `custom_model_data` cases.
///
/// # Example
/// ```ignore
/// let result1 = schem1.to_item_model(&pack, &config1)?;
/// let result2 = schem2.to_item_model(&pack, &config2)?;
/// let zip = build_resource_pack(&[&result1, &result2])?;
/// ```
pub fn build_resource_pack(results: &[&ItemModelResult]) -> Result<Vec<u8>> {
    if results.is_empty() {
        return Err(MeshError::Export("No results to pack".to_string()));
    }

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // pack.mcmeta
        zip.start_file("pack.mcmeta", options)
            .map_err(|e| MeshError::Export(e.to_string()))?;
        zip.write_all(br#"{"pack":{"pack_format":46,"description":"Generated by Nucleation"}}"#)
            .map_err(|e| MeshError::Export(e.to_string()))?;

        // Group results by item type for combined item definitions
        let mut item_cases: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

        for result in results {
            // Model JSON
            let model_path = format!(
                "assets/{}/models/item/{}.json",
                result.config.namespace, result.config.model_name
            );
            zip.start_file(&model_path, options)
                .map_err(|e| MeshError::Export(e.to_string()))?;
            zip.write_all(result.model_json.as_bytes())
                .map_err(|e| MeshError::Export(e.to_string()))?;

            // Texture PNGs
            for (tex_name, png_data) in &result.textures {
                let tex_path = format!(
                    "assets/{}/textures/item/{}/{}.png",
                    result.config.namespace, result.config.model_name, tex_name
                );
                zip.start_file(&tex_path, options)
                    .map_err(|e| MeshError::Export(e.to_string()))?;
                zip.write_all(png_data)
                    .map_err(|e| MeshError::Export(e.to_string()))?;
            }

            // Collect item definition case
            let case = serde_json::json!({
                "when": result.config.custom_model_data,
                "model": {
                    "type": "minecraft:model",
                    "model": format!("{}:item/{}", result.config.namespace, result.config.model_name)
                }
            });
            item_cases
                .entry(result.config.item.clone())
                .or_default()
                .push(case);
        }

        // Write item definition files (one per unique item type)
        for (item, cases) in &item_cases {
            let item_def_path = format!("assets/minecraft/items/{}.json", item);
            let item_def = serde_json::json!({
                "model": {
                    "type": "minecraft:select",
                    "property": "minecraft:custom_model_data",
                    "fallback": {
                        "type": "minecraft:model",
                        "model": format!("minecraft:item/{}", item)
                    },
                    "cases": cases
                }
            });
            zip.start_file(&item_def_path, options)
                .map_err(|e| MeshError::Export(e.to_string()))?;
            zip.write_all(
                serde_json::to_string_pretty(&item_def)
                    .unwrap_or_default()
                    .as_bytes(),
            )
            .map_err(|e| MeshError::Export(e.to_string()))?;
        }

        zip.finish().map_err(|e| MeshError::Export(e.to_string()))?;
    }
    Ok(buf)
}

// ─── Block info cache ───────────────────────────────────────────────────────

/// A single face element to blit into a plane cell.
struct FaceElement {
    /// RGBA pixels of the face texture.
    pixels: Vec<u8>,
    /// Texture pixel dimensions.
    tex_w: u32,
    tex_h: u32,
    /// Sub-cell position as fraction of the cell (0.0..1.0).
    cell_x: f32,
    cell_y: f32,
    cell_w: f32,
    cell_h: f32,
}

/// Cached per-block info resolved from the resource pack.
/// Keyed by full block state string (e.g. "minecraft:polished_basalt[axis=x]")
/// so that blocks with different properties resolve different textures.
struct BlockInfoCache {
    /// Block state keys that are full opaque cubes (safe to cull behind).
    full_cubes: HashSet<String>,
    /// Face elements per (block_state_key, direction). Multiple elements for complex models.
    faces: HashMap<(String, u8), Vec<FaceElement>>,
}

impl BlockInfoCache {
    fn is_full_cube(&self, state_key: &str) -> bool {
        self.full_cubes.contains(state_key)
    }

    fn get_faces(&self, state_key: &str, dir: Direction) -> Option<&Vec<FaceElement>> {
        self.faces.get(&(state_key.to_string(), dir as u8))
    }
}

/// Build the block info cache for all unique block states in the schematic.
/// Uses full block state strings (including properties) as keys so that
/// blocks like polished_basalt[axis=x] vs [axis=y] resolve different textures.
fn build_block_cache(
    unique_states: &HashMap<String, InputBlock>,
    pack: &ResourcePack,
    resolver: &ModelResolver,
    tex_resolution: u32,
) -> BlockInfoCache {
    let mut full_cubes = HashSet::new();
    let mut faces: HashMap<(String, u8), Vec<FaceElement>> = HashMap::new();

    for (state_key, input) in unique_states {
        let resolved_models = match resolve_block(pack, input) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let model = match resolved_models.first() {
            Some(m) => m,
            None => continue,
        };

        // Check if this is a full cube
        let is_full = model.model.elements.iter().any(|e| {
            e.from[0] <= 0.01
                && e.from[1] <= 0.01
                && e.from[2] <= 0.01
                && e.to[0] >= 15.99
                && e.to[1] >= 15.99
                && e.to[2] >= 15.99
        });
        if is_full {
            full_cubes.insert(state_key.clone());
        }

        // Resolve texture map once for this model
        let resolved_textures = resolver.resolve_textures(&model.model);

        // Build face elements for each direction
        for dir in Direction::ALL {
            let mut dir_faces = Vec::new();

            for element in &model.model.elements {
                if let Some(face) = element.faces.get(&dir) {
                    // Resolve the texture
                    let tex_ref = &face.texture;
                    let tex_key = tex_ref.strip_prefix('#').unwrap_or(tex_ref);
                    let tex_path = resolved_textures
                        .get(tex_key)
                        .cloned()
                        .unwrap_or_else(|| tex_ref.clone());

                    let full_path = if tex_path.contains(':') {
                        tex_path
                    } else {
                        format!("minecraft:{}", tex_path)
                    };

                    if let Some(tex_data) = pack.get_texture(&full_path) {
                        let frame = tex_data.first_frame();
                        let mut pixels =
                            if frame.width == tex_resolution && frame.height == tex_resolution {
                                frame.pixels.clone()
                            } else {
                                resize_nearest(
                                    &frame.pixels,
                                    frame.width,
                                    frame.height,
                                    tex_resolution,
                                    tex_resolution,
                                )
                            };

                        // Apply tint if needed
                        if face.tintindex >= 0 {
                            if let Some(tint) = get_tint_color(&input.name, face.tintindex) {
                                apply_tint(&mut pixels, tint);
                            }
                        }

                        // Compute sub-cell position from element bounds
                        let (cx, cy, cw, ch) = face_cell_rect(&element.from, &element.to, dir);

                        dir_faces.push(FaceElement {
                            pixels,
                            tex_w: tex_resolution,
                            tex_h: tex_resolution,
                            cell_x: cx,
                            cell_y: cy,
                            cell_w: cw,
                            cell_h: ch,
                        });
                    }
                }
            }

            if !dir_faces.is_empty() {
                faces.insert((state_key.clone(), dir as u8), dir_faces);
            }
        }
    }

    BlockInfoCache { full_cubes, faces }
}

/// Compute the sub-cell rectangle for a face, given element from/to and face direction.
///
/// Returns (x, y, w, h) in 0.0..1.0 range, where (0,0) is top-left of the cell.
fn face_cell_rect(from: &[f32; 3], to: &[f32; 3], dir: Direction) -> (f32, f32, f32, f32) {
    match dir {
        Direction::North => {
            // Perpendicular to Z, viewing from -Z. u = X (mirrored), v = Y (inverted).
            let u_min = (16.0 - to[0]) / 16.0;
            let u_max = (16.0 - from[0]) / 16.0;
            let v_min = (16.0 - to[1]) / 16.0;
            let v_max = (16.0 - from[1]) / 16.0;
            (u_min, v_min, u_max - u_min, v_max - v_min)
        }
        Direction::South => {
            let u_min = from[0] / 16.0;
            let u_max = to[0] / 16.0;
            let v_min = (16.0 - to[1]) / 16.0;
            let v_max = (16.0 - from[1]) / 16.0;
            (u_min, v_min, u_max - u_min, v_max - v_min)
        }
        Direction::East => {
            // Perpendicular to X, viewing from +X. u = Z (mirrored), v = Y (inverted).
            let u_min = (16.0 - to[2]) / 16.0;
            let u_max = (16.0 - from[2]) / 16.0;
            let v_min = (16.0 - to[1]) / 16.0;
            let v_max = (16.0 - from[1]) / 16.0;
            (u_min, v_min, u_max - u_min, v_max - v_min)
        }
        Direction::West => {
            let u_min = from[2] / 16.0;
            let u_max = to[2] / 16.0;
            let v_min = (16.0 - to[1]) / 16.0;
            let v_max = (16.0 - from[1]) / 16.0;
            (u_min, v_min, u_max - u_min, v_max - v_min)
        }
        Direction::Up => {
            let u_min = from[0] / 16.0;
            let u_max = to[0] / 16.0;
            let v_min = from[2] / 16.0;
            let v_max = to[2] / 16.0;
            (u_min, v_min, u_max - u_min, v_max - v_min)
        }
        Direction::Down => {
            let u_min = from[0] / 16.0;
            let u_max = to[0] / 16.0;
            let v_min = (16.0 - to[2]) / 16.0;
            let v_max = (16.0 - from[2]) / 16.0;
            (u_min, v_min, u_max - u_min, v_max - v_min)
        }
    }
}

/// Get the tint color for a block given its name and tint index.
fn get_tint_color(block_name: &str, _tint_index: i32) -> Option<[u8; 3]> {
    let name = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
    match name {
        "redstone_wire" => Some([255, 0, 0]),
        "grass_block" | "grass" | "short_grass" | "tall_grass" | "fern" | "large_fern" => {
            Some([124, 189, 107])
        }
        "oak_leaves" | "jungle_leaves" | "acacia_leaves" | "dark_oak_leaves"
        | "mangrove_leaves" => Some([106, 173, 51]),
        "birch_leaves" => Some([128, 167, 85]),
        "spruce_leaves" => Some([97, 153, 97]),
        "water" | "water_cauldron" => Some([63, 118, 228]),
        "lily_pad" => Some([32, 128, 48]),
        "vine" | "hanging_roots" => Some([106, 173, 51]),
        _ => None,
    }
}

/// Multiply pixel colors by a tint color (RGB).
fn apply_tint(pixels: &mut [u8], tint: [u8; 3]) {
    for chunk in pixels.chunks_exact_mut(4) {
        chunk[0] = ((chunk[0] as u16 * tint[0] as u16) / 255) as u8;
        chunk[1] = ((chunk[1] as u16 * tint[1] as u16) / 255) as u8;
        chunk[2] = ((chunk[2] as u16 * tint[2] as u16) / 255) as u8;
        // Alpha unchanged
    }
}

// ─── Plane building ─────────────────────────────────────────────────────────

/// A 2D grid of face textures for one plane slice.
struct PlaneGrid {
    /// Width of the grid (number of blocks along u-axis).
    width: u32,
    /// Height of the grid (number of blocks along v-axis).
    height: u32,
    /// Sparse map of (u, v) -> (block_name, direction) for texture lookup.
    cells: HashMap<(u32, u32), (String, Direction)>,
}

/// A chunk of a plane grid, small enough to fit in the texture atlas.
struct PlaneChunk {
    /// Offset of this chunk's u-origin within the full grid.
    u_offset: u32,
    /// Offset of this chunk's v-origin within the full grid.
    v_offset: u32,
    /// Width of this chunk (blocks along u-axis).
    width: u32,
    /// Height of this chunk (blocks along v-axis).
    height: u32,
    /// Cells with coordinates local to this chunk (0-based).
    cells: HashMap<(u32, u32), (String, Direction)>,
}

/// Placement of a chunk texture within an atlas page.
#[derive(Clone)]
struct AtlasPlacement {
    page_index: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

/// An atlas page containing packed chunk textures as raw RGBA pixels.
struct AtlasPage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

/// Raw RGBA pixel data from compositing a plane chunk.
struct RawChunkTexture {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

/// A model element for a non-full-cube block, positioned in model coordinate space.
struct IndividualElement {
    from: [f32; 3],
    to: [f32; 3],
    /// Optional element-level rotation (already in output coordinate space).
    rotation: Option<OutputRotation>,
    /// Faces: (direction, texture_key in the textures map).
    faces: Vec<(Direction, String)>,
}

/// Element-level rotation ready for JSON output.
struct OutputRotation {
    origin: [f32; 3],
    axis: String,
    angle: f32,
    rescale: bool,
}

type CoordFn = fn(i32, i32, i32) -> i32;

/// Map a Direction to its axis-perpendicular (u, v) coordinate extractors
/// and the axis coordinate extractor.
///
/// Returns (extract_u, extract_v, extract_axis, flip_u, flip_v).
fn direction_axes(dir: Direction) -> (CoordFn, CoordFn, CoordFn, bool, bool) {
    match dir {
        Direction::North => (|x, _, _| x, |_, y, _| y, |_, _, z| z, true, true),
        Direction::South => (|x, _, _| x, |_, y, _| y, |_, _, z| z, false, true),
        Direction::West => (|_, _, z| z, |_, y, _| y, |x, _, _| x, false, true),
        Direction::East => (|_, _, z| z, |_, y, _| y, |x, _, _| x, true, true),
        Direction::Up => (|x, _, _| x, |_, _, z| z, |_, y, _| y, false, false),
        Direction::Down => (|x, _, _| x, |_, _, z| z, |_, y, _| y, false, true),
    }
}

/// Get the plane dimensions for a given direction.
fn plane_dimensions(dir: Direction, width: i32, height: i32, depth: i32) -> (i32, i32, i32) {
    match dir {
        Direction::North | Direction::South => (width, height, depth),
        Direction::West | Direction::East => (depth, height, width),
        Direction::Up | Direction::Down => (width, depth, height),
    }
}

/// Direction name for file naming.
fn direction_name(dir: Direction) -> &'static str {
    match dir {
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
        Direction::Up => "up",
        Direction::Down => "down",
    }
}

/// Offset to check for the neighbor in a given direction.
fn direction_offset(dir: Direction) -> (i32, i32, i32) {
    match dir {
        Direction::North => (0, 0, -1),
        Direction::South => (0, 0, 1),
        Direction::West => (-1, 0, 0),
        Direction::East => (1, 0, 0),
        Direction::Up => (0, 1, 0),
        Direction::Down => (0, -1, 0),
    }
}

/// Build face planes for all 6 directions from a region.
///
/// Only culls faces when the neighbor is a full opaque cube. Non-full blocks
/// (torches, dust, slabs, etc.) never hide faces behind them.
fn build_face_planes(
    region: &Region,
    min: (i32, i32, i32),
    max: (i32, i32, i32),
    block_cache: &BlockInfoCache,
) -> HashMap<(Direction, i32), PlaneGrid> {
    let mut planes: HashMap<(Direction, i32), PlaneGrid> = HashMap::new();
    let bbox = region.get_bounding_box();

    let width = max.0 - min.0;
    let height = max.1 - min.1;
    let depth = max.2 - min.2;

    for dir in Direction::ALL {
        let (extract_u, extract_v, extract_axis, flip_u, flip_v) = direction_axes(dir);
        let (plane_w, plane_h, _plane_depth) = plane_dimensions(dir, width, height, depth);
        let offset = direction_offset(dir);

        for index in 0..region.volume() {
            let (x, y, z) = region.index_to_coords(index);
            if x < min.0 || x >= max.0 || y < min.1 || y >= max.1 || z < min.2 || z >= max.2 {
                continue;
            }

            let block = match region.get_block(x, y, z) {
                Some(b) if b.name != "minecraft:air" => b,
                _ => continue,
            };

            // Use full state string as cache key (preserves properties like axis)
            let state_key = block.to_string();

            // Non-full-cube blocks are rendered as individual elements with proper bounds
            if !block_cache.is_full_cube(&state_key) {
                continue;
            }

            // Check neighbor — only cull if neighbor is a FULL CUBE
            let nx = x + offset.0;
            let ny = y + offset.1;
            let nz = z + offset.2;
            let face_is_hidden = if !bbox.contains((nx, ny, nz)) {
                false // Out of bounds = exposed
            } else {
                match region.get_block(nx, ny, nz) {
                    Some(b) if b.name != "minecraft:air" => {
                        block_cache.is_full_cube(&b.to_string())
                    }
                    _ => false,
                }
            };

            if face_is_hidden {
                continue;
            }

            // Skip blocks that have no face data for this direction (they'll be transparent)
            if block_cache.get_faces(&state_key, dir).is_none() {
                continue;
            }

            let axis_val = extract_axis(x, y, z);
            let raw_u = extract_u(x, y, z) - extract_u(min.0, min.1, min.2);
            let raw_v = extract_v(x, y, z) - extract_v(min.0, min.1, min.2);

            let u = if flip_u {
                (plane_w - 1 - raw_u) as u32
            } else {
                raw_u as u32
            };
            let v = if flip_v {
                (plane_h - 1 - raw_v) as u32
            } else {
                raw_v as u32
            };

            let grid = planes.entry((dir, axis_val)).or_insert_with(|| PlaneGrid {
                width: plane_w as u32,
                height: plane_h as u32,
                cells: HashMap::new(),
            });

            grid.cells.insert((u, v), (state_key, dir));
        }
    }

    planes
}

/// Split a plane grid into chunks that each produce textures within MAX_TEXTURE_PX.
fn split_plane_grid(grid: &PlaneGrid, tex_resolution: u32) -> Vec<PlaneChunk> {
    let max_blocks = ATLAS_PAGE_SIZE / tex_resolution;

    if grid.width <= max_blocks && grid.height <= max_blocks {
        return vec![PlaneChunk {
            u_offset: 0,
            v_offset: 0,
            width: grid.width,
            height: grid.height,
            cells: grid.cells.clone(),
        }];
    }

    let chunks_u = (grid.width + max_blocks - 1) / max_blocks;
    let chunks_v = (grid.height + max_blocks - 1) / max_blocks;
    let mut chunks = Vec::new();

    for cu in 0..chunks_u {
        for cv in 0..chunks_v {
            let u_start = cu * max_blocks;
            let v_start = cv * max_blocks;
            let chunk_w = max_blocks.min(grid.width - u_start);
            let chunk_h = max_blocks.min(grid.height - v_start);

            let mut chunk_cells = HashMap::new();
            for (&(u, v), val) in &grid.cells {
                if u >= u_start && u < u_start + chunk_w && v >= v_start && v < v_start + chunk_h {
                    chunk_cells.insert((u - u_start, v - v_start), val.clone());
                }
            }

            if !chunk_cells.is_empty() {
                chunks.push(PlaneChunk {
                    u_offset: u_start,
                    v_offset: v_start,
                    width: chunk_w,
                    height: chunk_h,
                    cells: chunk_cells,
                });
            }
        }
    }

    chunks
}

// ─── Texture compositing ────────────────────────────────────────────────────

/// Simple nearest-neighbor resize for RGBA pixel data.
fn resize_nearest(pixels: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let mut out = vec![0u8; (dst_w * dst_h * 4) as usize];
    for dy in 0..dst_h {
        for dx in 0..dst_w {
            let sx = (dx * src_w / dst_w).min(src_w - 1);
            let sy = (dy * src_h / dst_h).min(src_h - 1);
            let src_idx = ((sy * src_w + sx) * 4) as usize;
            let dst_idx = ((dy * dst_w + dx) * 4) as usize;
            out[dst_idx..dst_idx + 4].copy_from_slice(&pixels[src_idx..src_idx + 4]);
        }
    }
    out
}

/// Composite a plane grid into raw RGBA pixels.
///
/// Uses the block info cache for sub-cell face positioning and tinting.
fn composite_plane_pixels(
    grid: &PlaneGrid,
    block_cache: &BlockInfoCache,
    tex_resolution: u32,
) -> Result<RawChunkTexture> {
    let img_w = grid.width * tex_resolution;
    let img_h = grid.height * tex_resolution;
    let mut pixels = vec![0u8; (img_w * img_h * 4) as usize];

    for (&(u, v), (block_name, dir)) in &grid.cells {
        let face_elements = match block_cache.get_faces(block_name, *dir) {
            Some(f) => f,
            None => continue,
        };

        let base_x = u * tex_resolution;
        let base_y = v * tex_resolution;
        let res = tex_resolution as f32;

        for face in face_elements {
            // Compute pixel bounds within the cell for this element
            let px_x = (face.cell_x * res) as u32;
            let px_y = (face.cell_y * res) as u32;
            let px_w = ((face.cell_w * res) as u32).max(1);
            let px_h = ((face.cell_h * res) as u32).max(1);

            // Blit face texture into the sub-cell region, scaling as needed
            for dy in 0..px_h {
                for dx in 0..px_w {
                    // Sample from the face texture
                    let sx = (dx * face.tex_w / px_w).min(face.tex_w - 1);
                    let sy = (dy * face.tex_h / px_h).min(face.tex_h - 1);
                    let src_idx = ((sy * face.tex_w + sx) * 4) as usize;

                    let dst_x = base_x + px_x + dx;
                    let dst_y = base_y + px_y + dy;
                    if dst_x >= img_w || dst_y >= img_h {
                        continue;
                    }
                    let dst_idx = ((dst_y * img_w + dst_x) * 4) as usize;

                    if src_idx + 3 < face.pixels.len() && dst_idx + 3 < pixels.len() {
                        let src_a = face.pixels[src_idx + 3];
                        if src_a == 255 {
                            // Opaque: overwrite
                            pixels[dst_idx..dst_idx + 4]
                                .copy_from_slice(&face.pixels[src_idx..src_idx + 4]);
                        } else if src_a > 0 {
                            // Alpha blend (src over dst)
                            let sa = src_a as u16;
                            let da = pixels[dst_idx + 3] as u16;
                            let inv_sa = 255 - sa;
                            pixels[dst_idx] = ((face.pixels[src_idx] as u16 * sa
                                + pixels[dst_idx] as u16 * inv_sa)
                                / 255) as u8;
                            pixels[dst_idx + 1] = ((face.pixels[src_idx + 1] as u16 * sa
                                + pixels[dst_idx + 1] as u16 * inv_sa)
                                / 255) as u8;
                            pixels[dst_idx + 2] = ((face.pixels[src_idx + 2] as u16 * sa
                                + pixels[dst_idx + 2] as u16 * inv_sa)
                                / 255) as u8;
                            pixels[dst_idx + 3] = (sa + da * inv_sa / 255).min(255) as u8;
                        }
                    }
                }
            }
        }
    }

    Ok(RawChunkTexture {
        width: img_w,
        height: img_h,
        pixels,
    })
}

/// Pack multiple raw chunk textures into atlas pages using shelf-packing.
///
/// Returns atlas pages (with blitted pixels) and a placement for each input chunk.
/// Pages are trimmed to the actual bounding box of their content.
fn pack_into_atlas(chunks: &[RawChunkTexture]) -> (Vec<AtlasPage>, Vec<AtlasPlacement>) {
    if chunks.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut placements: Vec<AtlasPlacement> = (0..chunks.len())
        .map(|_| AtlasPlacement {
            page_index: 0,
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        })
        .collect();

    // Sort indices by height descending for better shelf packing
    let mut indices: Vec<usize> = (0..chunks.len()).collect();
    indices.sort_by(|&a, &b| chunks[b].height.cmp(&chunks[a].height));

    struct Shelf {
        page_index: usize,
        y: u32,
        x_cursor: u32,
        height: u32,
    }

    let mut shelves: Vec<Shelf> = Vec::new();
    let mut page_next_y: Vec<u32> = Vec::new();

    for &idx in &indices {
        let w = chunks[idx].width;
        let h = chunks[idx].height;

        // Try to fit on an existing shelf
        let mut placed = false;
        for shelf in shelves.iter_mut() {
            if shelf.x_cursor + w <= ATLAS_PAGE_SIZE && h <= shelf.height {
                placements[idx] = AtlasPlacement {
                    page_index: shelf.page_index,
                    x: shelf.x_cursor,
                    y: shelf.y,
                    width: w,
                    height: h,
                };
                shelf.x_cursor += w;
                placed = true;
                break;
            }
        }

        if !placed {
            // Find a page with room for a new shelf
            let mut page_idx = None;
            for (pi, ny) in page_next_y.iter().enumerate() {
                if *ny + h <= ATLAS_PAGE_SIZE {
                    page_idx = Some(pi);
                    break;
                }
            }

            let pi = page_idx.unwrap_or_else(|| {
                page_next_y.push(0);
                page_next_y.len() - 1
            });

            let shelf_y = page_next_y[pi];
            page_next_y[pi] = shelf_y + h;

            placements[idx] = AtlasPlacement {
                page_index: pi,
                x: 0,
                y: shelf_y,
                width: w,
                height: h,
            };

            shelves.push(Shelf {
                page_index: pi,
                y: shelf_y,
                x_cursor: w,
                height: h,
            });
        }
    }

    // Compute actual page dimensions and blit pixels
    let page_count = page_next_y.len();
    let mut pages = Vec::with_capacity(page_count);

    for pi in 0..page_count {
        // Find actual extent used on this page
        let mut max_x = 0u32;
        let mut max_y = 0u32;
        for p in &placements {
            if p.page_index == pi {
                max_x = max_x.max(p.x + p.width);
                max_y = max_y.max(p.y + p.height);
            }
        }

        let page_w = max_x;
        let page_h = max_y;
        let mut pixels = vec![0u8; page_w as usize * page_h as usize * 4];

        for (i, placement) in placements.iter().enumerate() {
            if placement.page_index == pi {
                let chunk = &chunks[i];
                for row in 0..chunk.height {
                    let src_start = (row * chunk.width * 4) as usize;
                    let src_end = src_start + (chunk.width * 4) as usize;
                    let dst_start = ((placement.y + row) * page_w + placement.x) as usize * 4;
                    let dst_end = dst_start + (chunk.width * 4) as usize;
                    if src_end <= chunk.pixels.len() && dst_end <= pixels.len() {
                        pixels[dst_start..dst_end]
                            .copy_from_slice(&chunk.pixels[src_start..src_end]);
                    }
                }
            }
        }

        pages.push(AtlasPage {
            width: page_w,
            height: page_h,
            pixels,
        });
    }

    (pages, placements)
}

// ─── Block transform rotation ────────────────────────────────────────────────

/// Rotate a point around Y axis (center 8,8,8) by the given degrees (0/90/180/270).
fn rotate_point_y(p: [f32; 3], degrees: i32) -> [f32; 3] {
    match degrees.rem_euclid(360) {
        0 => p,
        90 => [16.0 - p[2], p[1], p[0]],
        180 => [16.0 - p[0], p[1], 16.0 - p[2]],
        270 => [p[2], p[1], 16.0 - p[0]],
        _ => p,
    }
}

/// Rotate a point around X axis (center 8,8,8) by the given degrees (0/90/180/270).
fn rotate_point_x(p: [f32; 3], degrees: i32) -> [f32; 3] {
    match degrees.rem_euclid(360) {
        0 => p,
        90 => [p[0], p[2], 16.0 - p[1]],
        180 => [p[0], 16.0 - p[1], 16.0 - p[2]],
        270 => [p[0], 16.0 - p[2], p[1]],
        _ => p,
    }
}

/// Rotate a face direction by a Y rotation (0/90/180/270 degrees clockwise from above).
fn rotate_direction_y(dir: Direction, degrees: i32) -> Direction {
    match degrees.rem_euclid(360) {
        0 => dir,
        90 => match dir {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
            other => other,
        },
        180 => match dir {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            other => other,
        },
        270 => match dir {
            Direction::North => Direction::West,
            Direction::West => Direction::South,
            Direction::South => Direction::East,
            Direction::East => Direction::North,
            other => other,
        },
        _ => dir,
    }
}

/// Rotate a face direction by an X rotation (0/90/180/270 degrees).
fn rotate_direction_x(dir: Direction, degrees: i32) -> Direction {
    match degrees.rem_euclid(360) {
        0 => dir,
        90 => match dir {
            Direction::Up => Direction::North,
            Direction::North => Direction::Down,
            Direction::Down => Direction::South,
            Direction::South => Direction::Up,
            other => other,
        },
        180 => match dir {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            other => other,
        },
        270 => match dir {
            Direction::Up => Direction::South,
            Direction::South => Direction::Down,
            Direction::Down => Direction::North,
            Direction::North => Direction::Up,
            other => other,
        },
        _ => dir,
    }
}

/// Apply block-level transform (Y then X rotation) to an element AABB.
/// Returns new (from, to) with proper min/max after rotation.
fn transform_element(
    from: &[f32; 3],
    to: &[f32; 3],
    y_rot: i32,
    x_rot: i32,
) -> ([f32; 3], [f32; 3]) {
    let p1 = rotate_point_x(rotate_point_y(*from, y_rot), x_rot);
    let p2 = rotate_point_x(rotate_point_y(*to, y_rot), x_rot);
    (
        [p1[0].min(p2[0]), p1[1].min(p2[1]), p1[2].min(p2[2])],
        [p1[0].max(p2[0]), p1[1].max(p2[1]), p1[2].max(p2[2])],
    )
}

/// Apply block-level transform to a face direction.
fn transform_direction(dir: Direction, y_rot: i32, x_rot: i32) -> Direction {
    rotate_direction_x(rotate_direction_y(dir, y_rot), x_rot)
}

// ─── Individual block elements ──────────────────────────────────────────────

/// UV-crop a face texture region and resize to target resolution.
///
/// UV coordinates are in [0, 16] range. If None, computes auto-UV from element bounds.
fn uv_crop(pixels: &[u8], src_w: u32, src_h: u32, uv: &[f32; 4], target_size: u32) -> Vec<u8> {
    let u0 = ((uv[0] / 16.0) * src_w as f32) as u32;
    let v0 = ((uv[1] / 16.0) * src_h as f32) as u32;
    let u1 = ((uv[2] / 16.0) * src_w as f32) as u32;
    let v1 = ((uv[3] / 16.0) * src_h as f32) as u32;

    let crop_w = u1.saturating_sub(u0).max(1);
    let crop_h = v1.saturating_sub(v0).max(1);

    let mut out = vec![0u8; (target_size * target_size * 4) as usize];
    for dy in 0..target_size {
        for dx in 0..target_size {
            let sx = (u0 + dx * crop_w / target_size).min(src_w - 1);
            let sy = (v0 + dy * crop_h / target_size).min(src_h - 1);
            let src_idx = ((sy * src_w + sx) * 4) as usize;
            let dst_idx = ((dy * target_size + dx) * 4) as usize;
            if src_idx + 3 < pixels.len() {
                out[dst_idx..dst_idx + 4].copy_from_slice(&pixels[src_idx..src_idx + 4]);
            }
        }
    }
    out
}

/// Compute auto-UV from element bounds, matching Minecraft's default behavior.
fn auto_uv(from: &[f32; 3], to: &[f32; 3], dir: Direction) -> [f32; 4] {
    match dir {
        Direction::Down => [from[0], 16.0 - to[2], to[0], 16.0 - from[2]],
        Direction::Up => [from[0], from[2], to[0], to[2]],
        Direction::North => [16.0 - to[0], 16.0 - to[1], 16.0 - from[0], 16.0 - from[1]],
        Direction::South => [from[0], 16.0 - to[1], to[0], 16.0 - from[1]],
        Direction::West => [from[2], 16.0 - to[1], to[2], 16.0 - from[1]],
        Direction::East => [16.0 - to[2], 16.0 - to[1], 16.0 - from[2], 16.0 - from[1]],
    }
}

/// Convert a Nucleation BlockState to a schematic-mesher InputBlock.
fn block_state_to_input(block: &crate::BlockState) -> InputBlock {
    let mut input = InputBlock::new(block.name.as_str());
    for (k, v) in &block.properties {
        input.properties.insert(k.to_string(), v.to_string());
    }
    input
}

/// Sanitize a block name for use in texture file names.
fn sanitize_name(name: &str) -> String {
    name.strip_prefix("minecraft:")
        .unwrap_or(name)
        .replace(['[', ']', '=', ',', ' ', ':'], "_")
}

/// Rotate a direction vector by Y then X rotation (no translation).
fn rotate_vec(v: [f32; 3], y_rot: i32, x_rot: i32) -> [f32; 3] {
    let v = match y_rot.rem_euclid(360) {
        0 => v,
        90 => [-v[2], v[1], v[0]],
        180 => [-v[0], v[1], -v[2]],
        270 => [v[2], v[1], -v[0]],
        _ => v,
    };
    match x_rot.rem_euclid(360) {
        0 => v,
        90 => [v[0], v[2], -v[1]],
        180 => [v[0], -v[1], -v[2]],
        270 => [v[0], -v[2], v[1]],
        _ => v,
    }
}

/// Transform an element-level rotation by the block-level transform.
///
/// Applies block transform to the rotation origin and axis, adjusting angle sign
/// if the axis direction is negated.
fn transform_rotation(
    origin: [f32; 3],
    axis: &str,
    angle: f32,
    rescale: bool,
    y_rot: i32,
    x_rot: i32,
    bx: f32,
    by: f32,
    bz: f32,
    scale: (f32, f32, f32),
) -> OutputRotation {
    let (sx, sy, sz) = scale;
    // Transform origin (same as element from/to)
    let rot_origin = rotate_point_x(rotate_point_y(origin, y_rot), x_rot);
    let out_origin = [
        bx + rot_origin[0] / (16.0 * sx),
        by + rot_origin[1] / (16.0 * sy),
        bz + rot_origin[2] / (16.0 * sz),
    ];

    // Transform axis direction vector
    let axis_vec = match axis {
        "x" => [1.0f32, 0.0, 0.0],
        "y" => [0.0, 1.0, 0.0],
        "z" => [0.0, 0.0, 1.0],
        _ => [0.0, 1.0, 0.0],
    };
    let rotated = rotate_vec(axis_vec, y_rot, x_rot);

    // Determine new axis name and sign
    let (new_axis, sign) =
        if rotated[0].abs() > rotated[1].abs() && rotated[0].abs() > rotated[2].abs() {
            ("x", rotated[0].signum())
        } else if rotated[1].abs() > rotated[2].abs() {
            ("y", rotated[1].signum())
        } else {
            ("z", rotated[2].signum())
        };

    OutputRotation {
        origin: out_origin,
        axis: new_axis.to_string(),
        angle: angle * sign,
        rescale,
    }
}

/// Rotate square RGBA pixel data by 90° clockwise.
fn rotate_pixels_cw90(pixels: &[u8], size: u32) -> Vec<u8> {
    let mut out = vec![0u8; pixels.len()];
    for y in 0..size {
        for x in 0..size {
            let src = ((y * size + x) * 4) as usize;
            // 90° CW: (x, y) → (size-1-y, x)
            let dst = ((x * size + (size - 1 - y)) * 4) as usize;
            out[dst..dst + 4].copy_from_slice(&pixels[src..src + 4]);
        }
    }
    out
}

/// Rotate square RGBA pixel data by N*90° clockwise.
fn rotate_pixels(pixels: &[u8], size: u32, quarter_turns: i32) -> Vec<u8> {
    match quarter_turns.rem_euclid(4) {
        0 => pixels.to_vec(),
        1 => rotate_pixels_cw90(pixels, size),
        2 => {
            // 180°: reverse pixel order
            let mut out = vec![0u8; pixels.len()];
            let total = (size * size) as usize;
            for i in 0..total {
                let src = i * 4;
                let dst = (total - 1 - i) * 4;
                out[dst..dst + 4].copy_from_slice(&pixels[src..src + 4]);
            }
            out
        }
        3 => {
            // 270° CW = 90° CCW: (x, y) → (y, size-1-x)
            let mut out = vec![0u8; pixels.len()];
            for y in 0..size {
                for x in 0..size {
                    let src = ((y * size + x) * 4) as usize;
                    let dst = (((size - 1 - x) * size + y) * 4) as usize;
                    out[dst..dst + 4].copy_from_slice(&pixels[src..src + 4]);
                }
            }
            out
        }
        _ => pixels.to_vec(),
    }
}

/// Compute the number of 90° CW quarter-turns needed for UV rotation when a face
/// is transformed by a block-level rotation.
///
/// Uses UV axis vector math: projects the rotated original UV axes onto the new
/// face's UV axes to determine the rotation angle.
fn compute_face_uv_rotation(orig_dir: Direction, y_rot: i32, x_rot: i32) -> i32 {
    if y_rot == 0 && x_rot == 0 {
        return 0;
    }

    // UV axis vectors for each face direction in Minecraft's coordinate system.
    // u_axis: direction of increasing U in 3D space
    // v_axis: direction of increasing V in 3D space
    fn uv_axes(d: Direction) -> ([f32; 3], [f32; 3]) {
        match d {
            // North (facing -Z): u = 16-X (right-to-left), v = 16-Y (top-to-bottom)
            Direction::North => ([-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
            // South (facing +Z): u = X, v = 16-Y
            Direction::South => ([1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
            // East (facing +X): u = 16-Z, v = 16-Y
            Direction::East => ([0.0, 0.0, -1.0], [0.0, -1.0, 0.0]),
            // West (facing -X): u = Z, v = 16-Y
            Direction::West => ([0.0, 0.0, 1.0], [0.0, -1.0, 0.0]),
            // Up (facing +Y): u = X, v = Z
            Direction::Up => ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            // Down (facing -Y): u = X, v = 16-Z
            Direction::Down => ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
        }
    }

    let (u_orig, _v_orig) = uv_axes(orig_dir);

    // Rotate original U axis by block transform
    let u_rotated = rotate_vec(u_orig, y_rot, x_rot);

    // Get new face direction and its UV axes
    let new_dir = transform_direction(orig_dir, y_rot, x_rot);
    let (u_new, v_new) = uv_axes(new_dir);

    // Determine rotation by projecting rotated U onto new face's axes:
    // u_rotated = cos(θ) * u_new + sin(θ) * v_new
    let dot_uu = u_rotated[0] * u_new[0] + u_rotated[1] * u_new[1] + u_rotated[2] * u_new[2];
    let dot_uv = u_rotated[0] * v_new[0] + u_rotated[1] * v_new[1] + u_rotated[2] * v_new[2];

    if dot_uu > 0.5 {
        0 // 0° rotation
    } else if dot_uv > 0.5 {
        1 // 90° CW
    } else if dot_uu < -0.5 {
        2 // 180°
    } else {
        3 // 270° CW
    }
}

/// Apply block-level UV rotation to face texture pixels.
///
/// When `uvlock=false` (default), face UVs rotate with the block transform.
/// Uses UV axis vector projection to compute the correct rotation for ALL faces,
/// including faces that change direction under the transform.
fn rotate_face_texture(
    pixels: Vec<u8>,
    size: u32,
    y_rot: i32,
    x_rot: i32,
    dir: Direction,
    uvlock: bool,
) -> Vec<u8> {
    if uvlock || (y_rot == 0 && x_rot == 0) {
        return pixels;
    }
    let turns = compute_face_uv_rotation(dir, y_rot, x_rot);
    if turns == 0 {
        pixels
    } else {
        rotate_pixels(&pixels, size, turns)
    }
}

/// Pre-resolved info for one element of a block model.
struct ElementInfo {
    from: [f32; 3],
    to: [f32; 3],
    /// Optional element-level rotation.
    rotation: Option<(
        /* origin */ [f32; 3],
        /* axis */ String,
        /* angle */ f32,
        /* rescale */ bool,
    )>,
    /// Block-level transform for this element's model.
    y_rotation: i32,
    x_rotation: i32,
    /// Whether UVs are locked (don't rotate with the block transform).
    uvlock: bool,
    /// Faces: (direction, texture_key_suffix, PNG data).
    faces: Vec<(Direction, String, Vec<u8>)>,
}

/// All resolved elements for a blockstate.
struct ResolvedBlockInfo {
    elements: Vec<ElementInfo>,
}

/// Resolve a block's model info from the resource pack.
///
/// Handles ALL resolved models (including multipart blocks with multiple models).
fn resolve_block_info(
    block: &crate::BlockState,
    pack: &ResourcePack,
    resolver: &ModelResolver,
) -> Option<ResolvedBlockInfo> {
    let input = block_state_to_input(block);
    let resolved_models = resolve_block(pack, &input).ok()?;

    if resolved_models.is_empty() {
        return None;
    }

    let mut elements = Vec::new();
    let mut global_elem_idx = 0usize;

    for resolved in &resolved_models {
        let resolved_textures = resolver.resolve_textures(&resolved.model);
        let y_rotation = resolved.transform.y;
        let x_rotation = resolved.transform.x;
        let uvlock = resolved.transform.uvlock;

        for element in &resolved.model.elements {
            let mut faces = Vec::new();

            // Capture element-level rotation
            let rotation = element.rotation.as_ref().map(|r| {
                let axis = match r.axis {
                    schematic_mesher::types::Axis::X => "x",
                    schematic_mesher::types::Axis::Y => "y",
                    schematic_mesher::types::Axis::Z => "z",
                };
                (r.origin, axis.to_string(), r.angle, r.rescale)
            });

            for &dir in &Direction::ALL {
                if let Some(face) = element.faces.get(&dir) {
                    let tex_ref = &face.texture;
                    let tex_key = tex_ref.strip_prefix('#').unwrap_or(tex_ref);
                    let tex_path = resolved_textures
                        .get(tex_key)
                        .cloned()
                        .unwrap_or_else(|| tex_ref.clone());
                    let full_path = if tex_path.contains(':') {
                        tex_path
                    } else {
                        format!("minecraft:{}", tex_path)
                    };

                    if let Some(tex_data) = pack.get_texture(&full_path) {
                        let frame = tex_data.first_frame();
                        // Use face UV if present, otherwise compute auto-UV from element bounds
                        let uv = face
                            .uv
                            .unwrap_or_else(|| auto_uv(&element.from, &element.to, dir));
                        let mut pixels = uv_crop(&frame.pixels, frame.width, frame.height, &uv, 16);

                        if face.tintindex >= 0 {
                            if let Some(tint) = get_tint_color(&block.name, face.tintindex) {
                                apply_tint(&mut pixels, tint);
                            }
                        }

                        // Rotate face texture pixels to match block-level rotation
                        pixels =
                            rotate_face_texture(pixels, 16, y_rotation, x_rotation, dir, uvlock);

                        let tex = TextureData::new(16, 16, pixels);
                        if let Ok(png) = tex.to_png() {
                            let suffix = format!("{}_{}", global_elem_idx, direction_name(dir));
                            faces.push((dir, suffix, png));
                        }
                    }
                }
            }

            elements.push(ElementInfo {
                from: element.from,
                to: element.to,
                rotation,
                y_rotation,
                x_rotation,
                uvlock,
                faces,
            });
            global_elem_idx += 1;
        }
    }

    Some(ResolvedBlockInfo { elements })
}

/// Build individual model elements for all non-full-cube blocks in a region.
///
/// Returns positioned elements and inserts their textures into the textures map.
fn build_individual_elements(
    region: &Region,
    min: (i32, i32, i32),
    max: (i32, i32, i32),
    offset: (f32, f32, f32),
    scale: (f32, f32, f32),
    pack: &ResourcePack,
    resolver: &ModelResolver,
    block_cache: &BlockInfoCache,
    _tex_resolution: u32,
    textures: &mut HashMap<String, Vec<u8>>,
) -> Vec<IndividualElement> {
    let mut elements = Vec::new();
    let mut model_cache: HashMap<String, Option<ResolvedBlockInfo>> = HashMap::new();
    let mut generated_textures: HashSet<String> = HashSet::new();

    for index in 0..region.volume() {
        let (x, y, z) = region.index_to_coords(index);
        if x < min.0 || x >= max.0 || y < min.1 || y >= max.1 || z < min.2 || z >= max.2 {
            continue;
        }

        let block = match region.get_block(x, y, z) {
            Some(b) if b.name != "minecraft:air" => b,
            _ => continue,
        };

        if block_cache.is_full_cube(&block.name) {
            continue;
        }

        let state_str = block.to_string();

        let info = model_cache
            .entry(state_str.clone())
            .or_insert_with(|| resolve_block_info(block, pack, resolver));

        let info = match info {
            Some(i) => i,
            None => continue,
        };

        let (sx, sy, sz) = scale;
        let bx = x as f32 / sx + offset.0;
        let by = y as f32 / sy + offset.1;
        let bz = z as f32 / sz + offset.2;

        let clean_state = sanitize_name(&state_str);

        for elem_info in &info.elements {
            let y_rot = elem_info.y_rotation;
            let x_rot = elem_info.x_rotation;

            // Apply block-level transform to element bounds
            let (rot_from, rot_to) =
                transform_element(&elem_info.from, &elem_info.to, y_rot, x_rot);

            // Scale element from [0,16] to [0,1/scale] and position at block coords
            let from = [
                bx + rot_from[0] / (16.0 * sx),
                by + rot_from[1] / (16.0 * sy),
                bz + rot_from[2] / (16.0 * sz),
            ];
            let to = [
                bx + rot_to[0] / (16.0 * sx),
                by + rot_to[1] / (16.0 * sy),
                bz + rot_to[2] / (16.0 * sz),
            ];

            // Transform element-level rotation if present
            let rotation = elem_info
                .rotation
                .as_ref()
                .map(|(origin, axis, angle, rescale)| {
                    transform_rotation(
                        *origin, axis, *angle, *rescale, y_rot, x_rot, bx, by, bz, scale,
                    )
                });

            let mut faces = Vec::new();

            for (orig_dir, tex_suffix, png_data) in &elem_info.faces {
                let rotated_dir = transform_direction(*orig_dir, y_rot, x_rot);
                let tex_key = format!("blk_{}_{}", clean_state, tex_suffix);

                if !generated_textures.contains(&tex_key) {
                    textures.insert(tex_key.clone(), png_data.clone());
                    generated_textures.insert(tex_key.clone());
                }

                faces.push((rotated_dir, tex_key));
            }

            if !faces.is_empty() {
                elements.push(IndividualElement {
                    from,
                    to,
                    rotation,
                    faces,
                });
            }
        }
    }

    elements
}

// ─── Model JSON generation ──────────────────────────────────────────────────

fn face_key(dir: Direction) -> &'static str {
    match dir {
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
        Direction::Up => "up",
        Direction::Down => "down",
    }
}

/// Info for a plane chunk element in the model.
struct PlaneChunkInfo {
    dir: Direction,
    axis_val: i32,
    /// Atlas page texture name (e.g., "atlas_0"). Multiple chunks share a page.
    tex_name: String,
    u_offset: u32,
    v_offset: u32,
    chunk_w: u32,
    chunk_h: u32,
    grid_width: u32,
    grid_height: u32,
    /// UV coordinates within the atlas page, in Minecraft [0,16] range.
    atlas_uv: [f32; 4],
}

/// Generate element coordinates for a chunk of a plane.
///
/// Accounts for the direction-specific UV axis flipping to position the chunk
/// element correctly in model space.
fn chunk_element_coords(
    chunk: &PlaneChunkInfo,
    min: (i32, i32, i32),
    max: (i32, i32, i32),
    offset: (f32, f32, f32),
    scale: (f32, f32, f32),
) -> ([f32; 3], [f32; 3]) {
    let (sx, sy, sz) = scale;
    let full_min_x = min.0 as f32 / sx + offset.0;
    let full_min_y = min.1 as f32 / sy + offset.1;
    let full_min_z = min.2 as f32 / sz + offset.2;
    let full_max_x = max.0 as f32 / sx + offset.0;
    let full_max_y = max.1 as f32 / sy + offset.1;
    let full_max_z = max.2 as f32 / sz + offset.2;

    // Whether grid u/v axes are flipped relative to model coordinates
    let (flip_u, flip_v) = match chunk.dir {
        Direction::North => (true, true),
        Direction::South => (false, true),
        Direction::West => (false, true),
        Direction::East => (true, true),
        Direction::Up => (false, false),
        Direction::Down => (false, true),
    };

    // Full range for the u and v axes in model coordinates
    let (u_range, v_range) = match chunk.dir {
        Direction::North | Direction::South => ((full_min_x, full_max_x), (full_min_y, full_max_y)),
        Direction::West | Direction::East => ((full_min_z, full_max_z), (full_min_y, full_max_y)),
        Direction::Up | Direction::Down => ((full_min_x, full_max_x), (full_min_z, full_max_z)),
    };

    let extent_u = u_range.1 - u_range.0;
    let extent_v = v_range.1 - v_range.0;
    let gw = chunk.grid_width as f32;
    let gh = chunk.grid_height as f32;

    let (u_start, u_end) = if flip_u {
        (
            u_range.1 - extent_u * (chunk.u_offset + chunk.chunk_w) as f32 / gw,
            u_range.1 - extent_u * chunk.u_offset as f32 / gw,
        )
    } else {
        (
            u_range.0 + extent_u * chunk.u_offset as f32 / gw,
            u_range.0 + extent_u * (chunk.u_offset + chunk.chunk_w) as f32 / gw,
        )
    };

    let (v_start, v_end) = if flip_v {
        (
            v_range.1 - extent_v * (chunk.v_offset + chunk.chunk_h) as f32 / gh,
            v_range.1 - extent_v * chunk.v_offset as f32 / gh,
        )
    } else {
        (
            v_range.0 + extent_v * chunk.v_offset as f32 / gh,
            v_range.0 + extent_v * (chunk.v_offset + chunk.chunk_h) as f32 / gh,
        )
    };

    let a = chunk.axis_val as f32;
    match chunk.dir {
        Direction::North | Direction::South => {
            let z = a / sz + offset.2;
            ([u_start, v_start, z], [u_end, v_end, z + 1.0 / sz])
        }
        Direction::West | Direction::East => {
            let x = a / sx + offset.0;
            ([x, v_start, u_start], [x + 1.0 / sx, v_end, u_end])
        }
        Direction::Up | Direction::Down => {
            let y = a / sy + offset.1;
            ([u_start, y, v_start], [u_end, y + 1.0 / sy, v_end])
        }
    }
}

/// Clamp a coordinate to Minecraft's valid element range [-16, 32].
fn clamp_coord(v: f32) -> f32 {
    v.max(-16.0).min(32.0)
}

/// Clamp a from/to coordinate pair to Minecraft's valid range.
fn clamp_from_to(from: [f32; 3], to: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    (
        [
            clamp_coord(from[0]),
            clamp_coord(from[1]),
            clamp_coord(from[2]),
        ],
        [clamp_coord(to[0]), clamp_coord(to[1]), clamp_coord(to[2])],
    )
}

/// Build the Minecraft item model JSON string.
///
/// Combines plane chunk elements (for full cubes) with individual elements (for non-full-cube blocks).
fn build_model_json(
    chunks: &[PlaneChunkInfo],
    individual: &[IndividualElement],
    min: (i32, i32, i32),
    max: (i32, i32, i32),
    offset: (f32, f32, f32),
    scale: (f32, f32, f32),
    namespace: &str,
    model_name: &str,
) -> String {
    let mut tex_map = serde_json::Map::new();
    let mut tex_counter = 0usize;

    // --- Plane chunk elements (atlas-packed, multiple chunks share a texture) ---
    let mut atlas_tex_indices: HashMap<String, usize> = HashMap::new();
    for chunk in chunks {
        atlas_tex_indices
            .entry(chunk.tex_name.clone())
            .or_insert_with(|| {
                let idx = tex_counter;
                let value = format!("{}:item/{}/{}", namespace, model_name, chunk.tex_name);
                tex_map.insert(format!("tex{}", idx), serde_json::Value::String(value));
                tex_counter += 1;
                idx
            });
    }

    let mut elements = Vec::new();
    for chunk in chunks {
        let (from, to) = chunk_element_coords(chunk, min, max, offset, scale);
        let (from, to) = clamp_from_to(from, to);
        let tex_idx = atlas_tex_indices[&chunk.tex_name];
        let tex_ref = format!("#tex{}", tex_idx);

        let mut faces_map = serde_json::Map::new();
        let mut face_obj = serde_json::Map::new();
        face_obj.insert(
            "uv".to_string(),
            serde_json::json!([
                chunk.atlas_uv[0],
                chunk.atlas_uv[1],
                chunk.atlas_uv[2],
                chunk.atlas_uv[3]
            ]),
        );
        face_obj.insert("texture".to_string(), serde_json::Value::String(tex_ref));
        faces_map.insert(
            face_key(chunk.dir).to_string(),
            serde_json::Value::Object(face_obj),
        );

        elements.push(serde_json::json!({
            "from": [from[0], from[1], from[2]],
            "to": [to[0], to[1], to[2]],
            "faces": faces_map
        }));
    }

    // --- Individual block elements ---
    // Deduplicate texture entries: map tex_key -> assigned tex index
    let mut indiv_tex_indices: HashMap<String, usize> = HashMap::new();

    for elem in individual {
        let mut faces_map = serde_json::Map::new();

        for (dir, tex_key) in &elem.faces {
            let tex_idx = *indiv_tex_indices.entry(tex_key.clone()).or_insert_with(|| {
                let idx = tex_counter;
                let map_key = format!("tex{}", idx);
                let value = format!("{}:item/{}/{}", namespace, model_name, tex_key);
                tex_map.insert(map_key, serde_json::Value::String(value));
                tex_counter += 1;
                idx
            });

            let tex_ref = format!("#tex{}", tex_idx);
            let mut face_obj = serde_json::Map::new();
            face_obj.insert("uv".to_string(), serde_json::json!([0, 0, 16, 16]));
            face_obj.insert("texture".to_string(), serde_json::Value::String(tex_ref));
            faces_map.insert(
                face_key(*dir).to_string(),
                serde_json::Value::Object(face_obj),
            );
        }

        let (clamped_from, clamped_to) = clamp_from_to(elem.from, elem.to);
        let mut elem_json = serde_json::json!({
            "from": [clamped_from[0], clamped_from[1], clamped_from[2]],
            "to": [clamped_to[0], clamped_to[1], clamped_to[2]],
            "faces": faces_map
        });

        // Include element-level rotation if present
        if let Some(rot) = &elem.rotation {
            elem_json["rotation"] = serde_json::json!({
                "origin": [rot.origin[0], rot.origin[1], rot.origin[2]],
                "axis": rot.axis,
                "angle": rot.angle,
                "rescale": rot.rescale
            });
        }

        elements.push(elem_json);
    }

    let model = serde_json::json!({
        "textures": tex_map,
        "elements": elements
    });

    serde_json::to_string_pretty(&model).unwrap_or_default()
}

// ─── Main entry point ───────────────────────────────────────────────────────

impl UniversalSchematic {
    /// Generate a Minecraft item model from this schematic using the plane-based approach.
    ///
    /// Slices the schematic into 2D planes per direction, composites block face textures
    /// into per-plane PNG images, and generates a JSON item model with one thin element
    /// per plane.
    ///
    /// The schematic must fit within 48x48x48 blocks (Minecraft model coordinate
    /// range: -16 to 32).
    pub fn to_item_model(
        &self,
        pack: &ResourcePackSource,
        config: &ItemModelConfig,
    ) -> Result<ItemModelResult> {
        let (min, max) = self.compute_tight_bounds();
        let width = max.0 - min.0;
        let height = max.1 - min.1;
        let depth = max.2 - min.2;

        if width == 0 || height == 0 || depth == 0 {
            return Err(MeshError::Meshing("Schematic has no blocks".to_string()));
        }

        // Resolve scale factors
        let (sx, sy, sz) = resolve_scale(&config.scale, width, height, depth);

        // Validate that scaled dimensions fit within model bounds
        let scaled_w = width as f32 / sx;
        let scaled_h = height as f32 / sy;
        let scaled_d = depth as f32 / sz;
        if scaled_w > MAX_DIM as f32 || scaled_h > MAX_DIM as f32 || scaled_d > MAX_DIM as f32 {
            return Err(MeshError::Meshing(format!(
                "Scaled schematic dimensions {:.1}x{:.1}x{:.1} exceed maximum {}x{}x{} for item models",
                scaled_w, scaled_h, scaled_d, MAX_DIM, MAX_DIM, MAX_DIM
            )));
        }

        // Coordinate offset to center/align within -16..32 (using scaled dims)
        let offset = if config.center {
            (
                -16.0 + (48.0 - scaled_w) / 2.0 - min.0 as f32 / sx,
                -16.0 + (48.0 - scaled_h) / 2.0 - min.1 as f32 / sy,
                -16.0 + (48.0 - scaled_d) / 2.0 - min.2 as f32 / sz,
            )
        } else {
            (
                -min.0 as f32 / sx - 16.0,
                -min.1 as f32 / sy - 16.0,
                -min.2 as f32 / sz - 16.0,
            )
        };
        let scale = (sx, sy, sz);

        // Collect all unique block states across all regions (keyed by full state string)
        let mut unique_states: HashMap<String, InputBlock> = HashMap::new();
        for region in std::iter::once(&self.default_region).chain(self.other_regions.values()) {
            for state in &region.palette {
                if state.name != "minecraft:air" {
                    let key = state.to_string();
                    unique_states
                        .entry(key)
                        .or_insert_with(|| block_state_to_input(state));
                }
            }
        }

        // Pre-resolve all block models and cache face info
        let resolver = ModelResolver::new(&pack.pack);
        let block_cache = build_block_cache(
            &unique_states,
            &pack.pack,
            &resolver,
            config.texture_resolution,
        );

        // Build face planes from all regions
        let mut all_planes: HashMap<(Direction, i32), PlaneGrid> = HashMap::new();

        let region_planes = build_face_planes(&self.default_region, min, max, &block_cache);
        merge_planes(&mut all_planes, region_planes);

        for region in self.other_regions.values() {
            let region_planes = build_face_planes(region, min, max, &block_cache);
            merge_planes(&mut all_planes, region_planes);
        }

        // Auto-reduce texture resolution for large models to fit in the texture atlas.
        // With atlas packing, sprite count = number of atlas pages (not individual chunks).
        // Each page is up to ATLAS_PAGE_SIZE×ATLAS_PAGE_SIZE pixels.
        let effective_tex_res = {
            let res = config.texture_resolution;
            let non_empty: Vec<&PlaneGrid> = all_planes
                .values()
                .filter(|g| !g.cells.is_empty())
                .collect();

            if non_empty.is_empty() {
                res
            } else {
                // Two constraints for MC's items atlas (GL_MAX_TEXTURE_SIZE = 16384):
                // 1. Total pixel area of all plane textures must fit in MC's atlas
                //    (16384² = 268M px minus vanilla items, with ~65% packing efficiency)
                // 2. Atlas page count must stay reasonable (each page = 1 sprite)
                const MAX_ATLAS_PX: u64 = 150_000_000; // ~65% of 16384² minus headroom
                const MAX_PAGES: u64 = 50;
                const PAGE_PX: u64 = (ATLAS_PAGE_SIZE as u64) * (ATLAS_PAGE_SIZE as u64);
                // Shelf packing ~75% efficient
                const EFFECTIVE_PAGE_PX: u64 = PAGE_PX * 3 / 4;

                let mut best = 1u32;
                for candidate in 1..=res {
                    let mut total_px = 0u64;
                    for grid in &non_empty {
                        total_px += (grid.width as u64 * candidate as u64)
                            * (grid.height as u64 * candidate as u64);
                    }

                    let pages = (total_px + EFFECTIVE_PAGE_PX - 1) / EFFECTIVE_PAGE_PX;
                    if total_px <= MAX_ATLAS_PX && pages <= MAX_PAGES {
                        best = candidate;
                    }
                }
                best
            }
        };

        // Composite plane chunks to raw RGBA pixels
        struct PendingChunk {
            dir: Direction,
            axis_val: i32,
            u_offset: u32,
            v_offset: u32,
            chunk_w: u32,
            chunk_h: u32,
            grid_width: u32,
            grid_height: u32,
        }
        let mut raw_chunks: Vec<RawChunkTexture> = Vec::new();
        let mut pending: Vec<PendingChunk> = Vec::new();

        for (&(dir, axis_val), grid) in &all_planes {
            if grid.cells.is_empty() {
                continue;
            }
            let chunks = split_plane_grid(grid, effective_tex_res);
            for chunk in &chunks {
                let chunk_grid = PlaneGrid {
                    width: chunk.width,
                    height: chunk.height,
                    cells: chunk.cells.clone(),
                };
                let raw = composite_plane_pixels(&chunk_grid, &block_cache, effective_tex_res)?;
                raw_chunks.push(raw);
                pending.push(PendingChunk {
                    dir,
                    axis_val,
                    u_offset: chunk.u_offset,
                    v_offset: chunk.v_offset,
                    chunk_w: chunk.width,
                    chunk_h: chunk.height,
                    grid_width: grid.width,
                    grid_height: grid.height,
                });
            }
        }

        // Pack all chunk textures into atlas pages
        let (atlas_pages, placements) = pack_into_atlas(&raw_chunks);

        // Encode atlas pages to PNG and build chunk infos
        let mut textures: HashMap<String, Vec<u8>> = HashMap::new();
        let mut chunk_infos: Vec<PlaneChunkInfo> = Vec::new();

        for (page_idx, page) in atlas_pages.iter().enumerate() {
            let tex_name = format!("atlas_{}", page_idx);
            let tex = TextureData::new(page.width, page.height, page.pixels.clone());
            let png = tex
                .to_png()
                .map_err(|e| MeshError::Export(format!("PNG encode error: {}", e)))?;
            textures.insert(tex_name, png);
        }

        for (i, placement) in placements.iter().enumerate() {
            let p = &pending[i];
            let page = &atlas_pages[placement.page_index];
            let tex_name = format!("atlas_{}", placement.page_index);

            let u0 = (placement.x as f32 / page.width as f32) * 16.0;
            let v0 = (placement.y as f32 / page.height as f32) * 16.0;
            let u1 = ((placement.x + placement.width) as f32 / page.width as f32) * 16.0;
            let v1 = ((placement.y + placement.height) as f32 / page.height as f32) * 16.0;

            chunk_infos.push(PlaneChunkInfo {
                dir: p.dir,
                axis_val: p.axis_val,
                tex_name,
                u_offset: p.u_offset,
                v_offset: p.v_offset,
                chunk_w: p.chunk_w,
                chunk_h: p.chunk_h,
                grid_width: p.grid_width,
                grid_height: p.grid_height,
                atlas_uv: [u0, v0, u1, v1],
            });
        }

        chunk_infos.sort_by(|a, b| a.tex_name.cmp(&b.tex_name));

        // Build individual elements for non-full-cube blocks
        let mut all_individual: Vec<IndividualElement> = Vec::new();

        let indiv = build_individual_elements(
            &self.default_region,
            min,
            max,
            offset,
            scale,
            &pack.pack,
            &resolver,
            &block_cache,
            config.texture_resolution,
            &mut textures,
        );
        all_individual.extend(indiv);

        for region in self.other_regions.values() {
            let indiv = build_individual_elements(
                region,
                min,
                max,
                offset,
                scale,
                &pack.pack,
                &resolver,
                &block_cache,
                config.texture_resolution,
                &mut textures,
            );
            all_individual.extend(indiv);
        }

        let model_json = build_model_json(
            &chunk_infos,
            &all_individual,
            min,
            max,
            offset,
            scale,
            &config.namespace,
            &config.model_name,
        );

        let stats = ItemModelStats {
            element_count: chunk_infos.len() + all_individual.len(),
            texture_count: textures.len(),
            plane_count: all_planes.len(),
            dimensions: (width, height, depth),
            scale,
        };

        Ok(ItemModelResult {
            model_json,
            textures,
            stats,
            config: config.clone(),
        })
    }

    /// Compute tight bounding box across all regions, returning (min, max).
    fn compute_tight_bounds(&self) -> ((i32, i32, i32), (i32, i32, i32)) {
        let mut min = (i32::MAX, i32::MAX, i32::MAX);
        let mut max = (i32::MIN, i32::MIN, i32::MIN);
        let mut found_block = false;

        for region in std::iter::once(&self.default_region).chain(self.other_regions.values()) {
            for index in 0..region.volume() {
                let (x, y, z) = region.index_to_coords(index);
                if let Some(block) = region.get_block(x, y, z) {
                    if block.name != "minecraft:air" {
                        min.0 = min.0.min(x);
                        min.1 = min.1.min(y);
                        min.2 = min.2.min(z);
                        max.0 = max.0.max(x + 1);
                        max.1 = max.1.max(y + 1);
                        max.2 = max.2.max(z + 1);
                        found_block = true;
                    }
                }
            }
        }

        if !found_block {
            return ((0, 0, 0), (0, 0, 0));
        }
        (min, max)
    }
}

/// Merge source planes into destination, preferring existing entries on conflict.
fn merge_planes(
    dst: &mut HashMap<(Direction, i32), PlaneGrid>,
    src: HashMap<(Direction, i32), PlaneGrid>,
) {
    for (key, src_grid) in src {
        let dst_grid = dst.entry(key).or_insert_with(|| PlaneGrid {
            width: src_grid.width,
            height: src_grid.height,
            cells: HashMap::new(),
        });
        for (pos, val) in src_grid.cells {
            dst_grid.cells.entry(pos).or_insert(val);
        }
    }
}
