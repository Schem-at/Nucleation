# Nucleation Meshing, NUCM Cache Format, and Rendering Pipeline

This document describes the complete pipeline from Minecraft schematic to rendered 3D output: the meshing API that converts schematics into triangle meshes, the `.nucm` binary cache format for efficient serialization, and the wgpu/Three.js rendering pipeline that displays them.

---

## Table of Contents

1. [Meshing Pipeline](#1-meshing-pipeline)
2. [Core Data Types](#2-core-data-types)
3. [NUCM Binary Format Specification](#3-nucm-binary-format-specification)
4. [Rendering Pipeline](#4-rendering-pipeline)
5. [Browser Rendering (Three.js)](#5-browser-rendering-threejs)
6. [Binding Layer API Reference](#6-binding-layer-api-reference)

---

## 1. Meshing Pipeline

The meshing system converts a `UniversalSchematic` + `ResourcePackSource` into triangle meshes. It is gated behind the `meshing` Cargo feature.

```toml
nucleation = { version = "0.1", features = ["meshing"] }
```

### 1.1 Resource Pack

A `ResourcePackSource` wraps a Minecraft resource pack (JAR/ZIP/directory) containing blockstate definitions, block models, and textures.

```rust
let pack = ResourcePackSource::from_file("minecraft-1.21.1-client.jar")?;
// or
let pack = ResourcePackSource::from_bytes(&zip_bytes)?;
```

The mesher uses this to resolve block names to 3D geometry and textures. All block face textures are packed into a single **texture atlas** per mesh output.

### 1.2 Mesh Config

`MeshConfig` controls meshing behavior:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `cull_hidden_faces` | bool | true | Remove faces between adjacent solid blocks |
| `ambient_occlusion` | bool | true | Darken faces in corners/crevices |
| `ao_intensity` | f32 | 0.4 | AO darkening strength (0.0-1.0) |
| `biome` | Option\<String\> | None | Biome name for grass/foliage tinting |
| `atlas_max_size` | u32 | 4096 | Maximum atlas texture dimension |
| `cull_occluded_blocks` | bool | true | Skip fully-enclosed blocks entirely |
| `greedy_meshing` | bool | false | Merge coplanar faces into larger quads |

### 1.3 Meshing Modes

All meshing methods are on `UniversalSchematic`:

#### Global Atlas (recommended for chunked meshing)

```rust
use nucleation::meshing::{build_global_atlas, MeshConfig};

// Build a single shared atlas from all unique block states
let atlas = build_global_atlas(&schematic, &pack, &config)?;

// Mesh chunks using the shared atlas (no per-chunk atlas duplication)
let meshes: Vec<MeshOutput> = schematic.mesh_chunks_with_atlas(&pack, &config, 32, atlas.clone())?;

// Save with shared atlas (NUCM v2 format — dramatically smaller files)
cache::save_cached_mesh_with_atlas(&meshes, &atlas, Path::new("output.nucm"))?;
```

The global atlas scans all unique block states from region palettes (O(unique_states), not O(volume)) and builds a single `TextureAtlas`. Chunks then skip per-chunk atlas construction and reference the shared atlas, eliminating ~99.8% of atlas storage in `.nucm` files for multi-chunk schematics.

#### Single Mesh (small/medium schematics)

```rust
let mesh: MeshOutput = schematic.to_mesh(&pack, &config)?;
```

Returns one `MeshOutput` for the entire schematic. Best for schematics under ~500K blocks.

#### Per-Region Meshes

```rust
let regions: HashMap<String, MeshOutput> = schematic.mesh_by_region(&pack, &config)?;
```

One mesh per named region (e.g., Litematic regions).

#### Eager Chunk Meshing

```rust
let result: ChunkMeshResult = schematic.mesh_by_chunk_size(&pack, &config, 32)?;
// result.meshes: HashMap<(i32, i32, i32), MeshOutput>
```

Splits the schematic into `chunk_size`-wide cubes and meshes all at once.

#### Lazy Chunk Iterator (large schematics)

```rust
for chunk_result in schematic.mesh_chunks(&pack, &config, 32) {
    let mesh: MeshOutput = chunk_result?;
    // Process one chunk at a time without loading all into memory
}
```

Never loads the full mesh into memory. Ideal for streaming or progressive rendering.

#### Parallel Chunk Meshing (fastest for large schematics)

```rust
let meshes: Vec<MeshOutput> = schematic.mesh_chunks_parallel(&pack, &config, 32, 8)?;
```

Uses `std::thread::scope` to mesh chunks in parallel. Best for CLI tools and batch processing.

### 1.4 Progress Reporting

For long-running chunked meshing, `NucleationChunkIter` supports progress callbacks:

```rust
use nucleation::meshing::{MeshProgress, MeshPhase};

let mut iter = schematic.mesh_chunks(&pack, &config, 32);
iter.set_progress_callback(Box::new(|progress: MeshProgress| {
    println!("[{:?}] {}/{} chunks — {} vertices, {} triangles",
        progress.phase, progress.chunks_done, progress.chunks_total,
        progress.vertices_so_far, progress.triangles_so_far);
}));

for chunk_result in iter {
    let mesh: MeshOutput = chunk_result?;
    // ...
}
```

**`MeshProgress` fields:**

| Field | Type | Description |
|-------|------|-------------|
| `phase` | `MeshPhase` | Current phase: `BuildingAtlas`, `MeshingChunks`, or `Complete` |
| `chunks_done` | u32 | Number of chunks completed |
| `chunks_total` | u32 | Total chunks to process |
| `vertices_so_far` | u64 | Cumulative vertex count |
| `triangles_so_far` | u64 | Cumulative triangle count |

---

## 2. Core Data Types

### 2.1 MeshOutput

The primary mesh result. Each `MeshOutput` represents one chunk (or the whole schematic if single-mesh mode).

```rust
pub struct MeshOutput {
    pub opaque: MeshLayer,                          // Solid geometry (no transparency)
    pub cutout: MeshLayer,                          // Binary alpha (leaves, glass panes)
    pub transparent: MeshLayer,                     // Blended transparency (water, stained glass)
    pub atlas: TextureAtlas,                        // Packed texture atlas shared by all layers
    pub animated_textures: Vec<AnimatedTextureExport>, // Animated texture metadata
    pub bounds: BoundingBox,                        // World-space AABB
    pub chunk_coord: Option<(i32, i32, i32)>,       // Chunk coordinate (if chunked)
    pub lod_level: u8,                              // Level of detail (0 = full)
}
```

**Key methods:**
- `total_vertices()` / `total_triangles()` — aggregate counts across all layers
- `is_empty()` / `has_transparency()` — layer queries
- `to_glb()` / `to_usdz()` — export to standard 3D formats

**Three layers** exist because they require different GPU render states:
- **Opaque**: No blending, writes depth. Rendered first.
- **Cutout**: No blending, writes depth, fragment shader discards pixels with alpha < 0.5. Rendered second.
- **Transparent**: Alpha blending enabled, depth writes disabled. Rendered last.

### 2.2 MeshLayer

Structure-of-arrays vertex data for one render layer:

```rust
pub struct MeshLayer {
    pub positions: Vec<[f32; 3]>,  // Vertex positions in world space
    pub normals: Vec<[f32; 3]>,    // Per-vertex unit normals
    pub uvs: Vec<[f32; 2]>,        // Texture coordinates into the atlas
    pub colors: Vec<[f32; 4]>,     // Per-vertex RGBA color multipliers (0.0-1.0)
    pub indices: Vec<u32>,          // Triangle indices (every 3 form a triangle)
}
```

All four arrays (positions, normals, uvs, colors) have the same length (`vertex_count()`). The indices array references into them.

**Vertex colors** are multiplied with the atlas texture sample in the fragment shader. They encode biome tinting, ambient occlusion darkening, and block-specific color variations. Most vertices have `[1.0, 1.0, 1.0, 1.0]` (no tint).

### 2.3 TextureAtlas

A single texture containing all block face textures packed together:

```rust
pub struct TextureAtlas {
    pub width: u32,                              // Texture width in pixels
    pub height: u32,                             // Texture height in pixels
    pub pixels: Vec<u8>,                         // RGBA8 pixel data (width * height * 4 bytes)
    pub regions: HashMap<String, AtlasRegion>,   // Named UV regions
}
```

Each `AtlasRegion` maps a texture name (e.g., `"minecraft:block/stone"`) to normalized UV coordinates:

```rust
pub struct AtlasRegion {
    pub u_min: f32, pub v_min: f32,  // Bottom-left corner
    pub u_max: f32, pub v_max: f32,  // Top-right corner
}
```

The mesh vertex UVs are already in atlas space — they reference the correct atlas region directly. Renderers just need to upload the atlas as a texture and sample at the vertex UVs.

### 2.4 BoundingBox

```rust
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}
```

### 2.5 AnimatedTextureExport

Metadata for animated block textures (e.g., water, lava, fire):

```rust
pub struct AnimatedTextureExport {
    pub sprite_sheet_png: Vec<u8>,     // PNG-encoded sprite sheet (all frames)
    pub frame_count: u32,              // Number of animation frames
    pub frametime: u32,                // Ticks per frame
    pub interpolate: bool,             // Blend between frames
    pub frames: Option<Vec<u32>>,      // Custom frame order (None = sequential)
    pub frame_width: u32,              // Per-frame width in pixels
    pub frame_height: u32,             // Per-frame height in pixels
    pub atlas_x: u32,                  // Position in atlas (pixel X)
    pub atlas_y: u32,                  // Position in atlas (pixel Y)
}
```

---

## 3. NUCM Binary Format Specification

`.nucm` (Nucleation Cached Mesh) is a binary format for fast serialization of `Vec<MeshOutput>`. It uses lossy quantization + delta encoding + DEFLATE compression, achieving **6-8x smaller** files than GLB for the same mesh data.

All multi-byte values are **little-endian**.

### 3.1 File Header

**v1 header (12 bytes):**

| Offset | Size | Type | Field | Value |
|--------|------|------|-------|-------|
| 0 | 4 | `[u8; 4]` | magic | `b"NUCM"` (0x4E 0x55 0x43 0x4D) |
| 4 | 4 | u32 | version | 1 |
| 8 | 4 | u32 | chunk_count | Number of MeshOutput chunks |

**v2 header (16 bytes):**

| Offset | Size | Type | Field | Value |
|--------|------|------|-------|-------|
| 0 | 4 | `[u8; 4]` | magic | `b"NUCM"` (0x4E 0x55 0x43 0x4D) |
| 4 | 4 | u32 | version | 2 |
| 8 | 4 | u32 | flags | Bitfield (see below) |
| 12 | 4 | u32 | chunk_count | Number of MeshOutput chunks |

**Flags bitfield:**

| Bit | Name | Description |
|-----|------|-------------|
| 0 | `HAS_SHARED_ATLAS` | If set, a shared atlas follows the header. Chunks reference it instead of storing their own. |
| 1-31 | (reserved) | Must be 0 |

**Shared atlas (present only when `HAS_SHARED_ATLAS` is set):**

Immediately after the v2 header, the shared atlas is written using the same atlas format as per-chunk atlases (see Atlas section below). All chunks with `atlas_mode == 0` reference this shared atlas.

### 3.2 Per-Chunk Layout

Repeated `chunk_count` times, sequentially:

#### Bounds (24 bytes)

| Size | Type | Field |
|------|------|-------|
| 4 | f32 | bounds.min[0] (X) |
| 4 | f32 | bounds.min[1] (Y) |
| 4 | f32 | bounds.min[2] (Z) |
| 4 | f32 | bounds.max[0] (X) |
| 4 | f32 | bounds.max[1] (Y) |
| 4 | f32 | bounds.max[2] (Z) |

#### Chunk Coordinate (1 or 13 bytes)

| Size | Type | Field |
|------|------|-------|
| 1 | u8 | has_coord (0 = absent, 1 = present) |
| 4 | i32 | chunk_x *(only if has_coord == 1)* |
| 4 | i32 | chunk_y *(only if has_coord == 1)* |
| 4 | i32 | chunk_z *(only if has_coord == 1)* |

#### LOD Level (1 byte)

| Size | Type | Field |
|------|------|-------|
| 1 | u8 | lod_level (0 = full detail) |

#### Atlas Mode (v2 only — 1 byte)

| Size | Type | Field |
|------|------|-------|
| 1 | u8 | atlas_mode (0 = uses shared atlas from header, 1 = has own atlas below) |

**v1 files** do not have this byte — they always have a per-chunk atlas immediately after `lod_level`.

**v2 files** always have this byte. If `atlas_mode == 0`, the chunk uses the shared atlas from the header (no atlas data follows). If `atlas_mode == 1`, a per-chunk atlas follows in the same format as v1.

#### Atlas

| Size | Type | Field |
|------|------|-------|
| 4 | u32 | width |
| 4 | u32 | height |
| 4 | u32 | raw_pixel_len (uncompressed RGBA size) |
| 4 | u32 | compressed_pixel_len |
| N | bytes | DEFLATE-compressed RGBA pixels |
| 4 | u32 | region_count |

Per region (repeated `region_count` times):

| Size | Type | Field |
|------|------|-------|
| 4 | u32 | name_len |
| M | bytes | name (UTF-8) |
| 4 | f32 | u_min |
| 4 | f32 | v_min |
| 4 | f32 | u_max |
| 4 | f32 | v_max |

#### Animated Textures

| Size | Type | Field |
|------|------|-------|
| 4 | u32 | anim_count |

Per animated texture (repeated `anim_count` times):

| Size | Type | Field |
|------|------|-------|
| 4 | u32 | sprite_png_len |
| N | bytes | sprite_sheet_png (raw PNG bytes) |
| 4 | u32 | frame_count |
| 4 | u32 | frametime |
| 1 | u8 | interpolate (0 or 1) |
| 1 | u8 | has_frames (0 or 1) |
| 4 | u32 | frames_len *(only if has_frames == 1)* |
| 4*K | u32[] | frames *(only if has_frames == 1)* |
| 4 | u32 | frame_width |
| 4 | u32 | frame_height |
| 4 | u32 | atlas_x |
| 4 | u32 | atlas_y |

#### Three Mesh Layers (opaque, cutout, transparent — in order)

Each layer has the same structure:

**Layer header:**

| Size | Type | Field |
|------|------|-------|
| 4 | u32 | vertex_count |
| 4 | u32 | index_count |

**If vertex_count == 0** (empty layer):

| Size | Type | Field |
|------|------|-------|
| 4 | u32 | raw_len (0) |
| 4 | u32 | compressed_len (0 or small) |
| N | bytes | compressed empty data |

**If vertex_count > 0:**

##### Positions (quantized u16x3, delta-encoded, DEFLATE)

| Size | Type | Field |
|------|------|-------|
| 24 | f32[6] | Position AABB: min[3] + max[3] |
| | | *compressed_field:* |
| 4 | u32 | raw_len (vertex_count * 6) |
| 4 | u32 | compressed_len |
| N | bytes | DEFLATE data |

**Encoding:**
1. For each vertex position `[x, y, z]`, quantize to u16 relative to the AABB:
   ```
   q = if range > 0: ((pos - min) / range) * 65535.0 + 0.5   else: 0
   ```
2. Delta-encode: `delta = current.wrapping_sub(previous)` (u16 wrapping arithmetic)
3. Store each delta as 2 bytes LE (6 bytes per vertex: x_delta, y_delta, z_delta)
4. DEFLATE-compress the entire buffer

**Decoding:**
1. DEFLATE-decompress
2. Undo delta: `current = previous.wrapping_add(delta)`
3. Dequantize: `pos = min + (q / 65535.0) * range`

**Precision:** ~0.0003 units for a 16-unit chunk range.

##### Normals (quantized i8x3, DEFLATE)

| Size | Type | Field |
|------|------|-------|
| | | *compressed_field:* |
| 4 | u32 | raw_len (vertex_count * 3) |
| 4 | u32 | compressed_len |
| N | bytes | DEFLATE data |

**Encoding:** `i8_val = (normal_component * 127.0) as i8`, stored as u8.

**Decoding:** `f32_val = (byte_as_i8) / 127.0`, then renormalize to unit length.

Axis-aligned normals (the vast majority in Minecraft meshes) roundtrip exactly.

##### UVs (quantized u16x2, DEFLATE)

| Size | Type | Field |
|------|------|-------|
| 16 | f32[4] | UV AABB: min[2] + max[2] |
| | | *compressed_field:* |
| 4 | u32 | raw_len (vertex_count * 4) |
| 4 | u32 | compressed_len |
| N | bytes | DEFLATE data |

Same quantization as positions but no delta encoding. 2 components (u, v) per vertex.

##### Colors (quantized u8x4, DEFLATE)

| Size | Type | Field |
|------|------|-------|
| | | *compressed_field:* |
| 4 | u32 | raw_len (vertex_count * 4) |
| 4 | u32 | compressed_len |
| N | bytes | DEFLATE data |

**Encoding:** `u8_val = (color.clamp(0,1) * 255.0 + 0.5) as u8` — 4 bytes per vertex (R, G, B, A).

##### Indices (delta-encoded u32, DEFLATE)

| Size | Type | Field |
|------|------|-------|
| | | *compressed_field:* |
| 4 | u32 | raw_len (index_count * 4) |
| 4 | u32 | compressed_len |
| N | bytes | DEFLATE data |

**Encoding:** Delta-encode with wrapping u32 arithmetic, then DEFLATE. Quad mesh index patterns (0,1,2,2,3,0,4,5,6,...) produce small repeating deltas that compress extremely well.

### 3.3 Compressed Field Format

Every compressed field uses the same envelope:

```
raw_len:        u32 LE   — size of uncompressed data
compressed_len: u32 LE   — size of DEFLATE payload
data:           [u8; compressed_len] — raw DEFLATE (RFC 1951, no zlib/gzip header)
```

The Rust implementation uses `flate2` with `Compression::fast()`. JavaScript parsers should use raw DEFLATE inflate (e.g., `fflate.inflateSync()`).

### 3.4 Size Comparison

| Schematic | Vertices | GLB | NUCM | Ratio |
|-----------|----------|-----|------|-------|
| cutecounter (3x18x4) | 4.4K | 115 KB | 19.5 KB | 0.17x |
| Evaluator (25x27x37) | 178K | 4.6 MB | 546 KB | 0.12x |
| uss-texas (175x53x31) | 590K | 15.3 MB | 2.4 MB | 0.16x |
| IRIS_B (499x379x442) | 283M | N/A | 570 MB | — |

NUCM is typically **6-8x smaller** than GLB thanks to quantization and compression.

### 3.5 Rust API

```rust
use nucleation::meshing::cache;

// Serialize (v2 without shared atlas — per-chunk atlases)
let bytes: Vec<u8> = cache::serialize_meshes(&meshes);
cache::save_cached_mesh(&meshes, Path::new("output.nucm"))?;

// Serialize (v2 with shared atlas — dramatically smaller for multi-chunk)
let bytes: Vec<u8> = cache::serialize_meshes_with_atlas(&meshes, &atlas);
cache::save_cached_mesh_with_atlas(&meshes, &atlas, Path::new("output.nucm"))?;

// Deserialize (auto-detects v1 and v2)
let meshes: Vec<MeshOutput> = cache::deserialize_meshes(&bytes)?;
let meshes = cache::load_cached_mesh(Path::new("input.nucm"))?;
```

**Backward compatibility:** `deserialize_meshes()` and `load_cached_mesh()` auto-detect v1 vs v2 format. v1 files continue to load correctly.

---

## 4. Rendering Pipeline (Desktop — wgpu)

The desktop renderer (`examples/render_schematic.rs`) uses wgpu (WebGPU API) with a WGSL shader.

### 4.1 GPU Data Upload

For each `MeshOutput` chunk, the renderer creates:

1. **Per-layer vertex/index buffers** (up to 3 layers per chunk):
   - 4 separate vertex buffers: positions (Float32x3), normals (Float32x3), UVs (Float32x2), colors (Float32x4)
   - 1 index buffer (Uint32)

2. **Per-chunk atlas texture**:
   - Format: `Rgba8UnormSrgb`
   - Sampler: nearest-neighbor filtering (pixelated Minecraft look)

### 4.2 Vertex Buffer Layout

Four separate vertex buffers (not interleaved):

| Slot | Location | Format | Stride | Data |
|------|----------|--------|--------|------|
| 0 | 0 | Float32x3 | 12 bytes | positions |
| 1 | 1 | Float32x3 | 12 bytes | normals |
| 2 | 2 | Float32x2 | 8 bytes | uvs |
| 3 | 3 | Float32x4 | 16 bytes | colors (RGBA) |

### 4.3 Bind Groups

| Group | Binding | Type | Contents |
|-------|---------|------|----------|
| 0 | 0 | Uniform buffer | view_proj (mat4x4), inv_view_proj (mat4x4), params (vec4: alpha_cutoff, hdri_enabled, hdri_intensity, _) |
| 1 | 0 | Texture 2D | Per-chunk atlas texture |
| 1 | 1 | Sampler | Nearest-neighbor sampler |
| 2 | 0 | Texture 2D | HDRI environment map (or 1x1 black dummy) |
| 2 | 1 | Sampler | Linear sampler for HDRI |

### 4.4 Render Passes (Per Frame)

Three sequential passes over all chunks:

1. **Opaque pass**: No blending, depth write ON, back-face culling. `alpha_cutoff = 0.0`.
2. **Cutout pass**: No blending, depth write ON, back-face culling OFF. `alpha_cutoff = 0.5` (fragment shader discards below threshold).
3. **Transparent pass**: Alpha blending (src_alpha, one_minus_src_alpha), depth write OFF, no culling. `alpha_cutoff = 0.0`.

Optional **skybox pass** after transparent (if HDRI provided): renders a fullscreen triangle sampling the equirectangular HDRI.

### 4.5 Shader (WGSL)

**Vertex stage** (`vs_main`):
```wgsl
@vertex fn vs_main(in: VertexInput) -> VertexOutput {
    out.clip_position = uniforms.view_proj * vec4(in.position, 1.0);
    out.world_normal = in.normal;
    out.uv = in.uv;
    out.color = in.color;
}
```

**Fragment stage** (`fs_main`):
```wgsl
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(atlas_texture, atlas_sampler, in.uv);
    var color = tex * in.color;
    if color.a < uniforms.params.x { discard; }  // alpha cutoff

    // Lighting (simplified):
    let n = normalize(in.world_normal);
    if hdri_enabled {
        // IBL + key light + Reinhard tonemap
    } else {
        // ambient (0.4) + directional (0.6 * NdotL)
    }
    return vec4(final_color, color.a);
}
```

### 4.6 Camera

The camera orbits around the mesh bounding box center:
- **Yaw/Pitch**: Spherical coordinates for camera direction
- **Auto-fit**: Projects all 8 AABB corners to compute the minimum distance that fits the mesh in view
- **Zoom**: Multiplier on the auto-fit distance

---

## 5. Browser Rendering (Three.js)

The browser PoC (`examples/nucm-viewer/`) renders `.nucm` files using Three.js (WebGL2).

### 5.1 Architecture

```
.nucm file (ArrayBuffer)
    → nucm-parser.js (DataView + fflate)
    → Array of chunk objects with Float32Array/Uint32Array per layer
    → renderer.js (Three.js)
    → THREE.BufferGeometry + THREE.DataTexture per chunk
    → WebGL2 rendering with OrbitControls
```

### 5.2 Parser (`nucm-parser.js`)

```javascript
import { parseNUCM } from './nucm-parser.js';

const { chunks, totalChunks, version, sharedAtlas } = parseNUCM(arrayBuffer, maxChunks, onProgress);
// chunks[i].layers.opaque.positions → Float32Array
// chunks[i].atlas.pixels → Uint8Array (RGBA)
// version → 1 or 2
// sharedAtlas → atlas object or null (v2 with HAS_SHARED_ATLAS)
```

Supports both v1 and v2 formats. Uses `fflate.inflateSync()` for raw DEFLATE decompression. Handles all dequantization (u16→f32 positions with AABB, i8→f32 normals with renormalization, u16→f32 UVs, u8→f32 colors) and delta decoding (positions + indices).

For v2 files with `HAS_SHARED_ATLAS`, the shared atlas is parsed from the header and assigned to each chunk that uses `atlas_mode == 0`. Chunks always have an `atlas` field regardless of version — renderers don't need to distinguish between shared and per-chunk atlases.

`maxChunks` enables early exit to limit GPU memory for large schematics.

### 5.3 Renderer (`renderer.js`)

Per-chunk rendering:
- `THREE.DataTexture` from atlas pixels (nearest-neighbor, no flip)
- Up to 3 `THREE.Mesh` objects per chunk:
  - **Opaque**: `MeshBasicMaterial` with `map` + `vertexColors`
  - **Cutout**: `MeshBasicMaterial` with `alphaTest: 0.5`, `side: DoubleSide`
  - **Transparent**: `MeshBasicMaterial` with `transparent: true`, `depthWrite: false`, `side: DoubleSide`
- `THREE.BufferGeometry` with position, normal, uv, color attributes
- Camera auto-fits to scene bounding box

### 5.4 Dependencies (CDN, no build step)

- Three.js r170+ (ES module via importmap)
- fflate 0.8+ (3KB gzipped, for raw DEFLATE)

---

## 6. Binding Layer API Reference

### 6.1 WASM (JavaScript)

```javascript
// Single mesh
const mesh = schematic.toMesh(pack, config);
const nucmBytes = mesh.toNucm();     // Uint8Array
const glbBytes = mesh.toGlb();       // Uint8Array

// Chunked mesh
const chunks = schematic.meshByChunkSize(pack, config, 32);
const nucmBytes = chunks.toNucm();   // All chunks in one .nucm

// Lazy iterator
const iter = schematic.chunkMeshIterator(pack, config, 32);
while (iter.advance()) {
    const chunk = iter.current();
    const nucm = chunk.toNucm();     // Single chunk as .nucm
}

// Global atlas (v2 — shared atlas across chunks)
const atlas = schematic.buildGlobalAtlas(pack, config);
// atlas.width(), atlas.height(), atlas.toBytes() → Uint8Array (RGBA)

const iter2 = schematic.chunkMeshIteratorWithAtlas(pack, config, 32, atlas);
iter2.setProgressCallback((progress) => {
    // progress.phase: "BuildingAtlas" | "MeshingChunks" | "Complete"
    // progress.chunksDone, progress.chunksTotal
    // progress.verticesSoFar, progress.trianglesSoFar
    updateProgressBar(progress.chunksDone / progress.chunksTotal);
});
// iter2.hasSharedAtlas() → true
// iter2.sharedAtlas() → TextureAtlasWrapper
while (iter2.advance()) { ... }
const nucmV2 = iter2.toNucm();      // NUCM v2 with shared atlas
```

### 6.2 Python

```python
# Single mesh
mesh = schematic.to_mesh(pack, config)
mesh.save_nucm("output.nucm")        # Save to file
data = mesh.nucm_data                 # bytes object

# Chunked mesh
chunks = schematic.mesh_by_chunk_size(pack, config, 32)
chunks.save_nucm("output.nucm")      # All chunks in one file
data = chunks.nucm_data               # bytes object

# Multi-region mesh
regions = schematic.mesh_by_region(pack, config)
regions.save_nucm("output.nucm")     # All regions in one file

# Global atlas (v2 — shared atlas across chunks)
atlas = schematic.build_global_atlas(pack, config)
# atlas.width, atlas.height, atlas.rgba_data(), atlas.region_count

chunks = schematic.mesh_chunks_with_atlas(pack, atlas, config, chunk_size=32)
# chunks is Vec<PyMeshResult>, each chunk uses the shared atlas
```

### 6.3 FFI (C)

```c
// Single mesh
FFIMeshResult* mesh = schematic_to_mesh(schematic, pack, config);
ByteArray nucm = meshresult_nucm_data(mesh);
ByteArray glb = meshresult_glb_data(mesh);

// Chunked mesh
FFIChunkMeshResult* chunks = schematic_mesh_by_chunk_size(schematic, pack, config, 32);
ByteArray nucm = chunkmeshresult_nucm_data(chunks);

// Global atlas (v2 — shared atlas across chunks)
FFITextureAtlas* atlas = schematic_build_global_atlas(schematic, pack, config);
// textureatlas_width(atlas), textureatlas_height(atlas)
// textureatlas_rgba_data(atlas, &out_ptr, &out_len)

FFIChunkMeshResult* chunks = schematic_mesh_chunks_with_atlas(schematic, pack, config, 32, atlas);
ByteArray nucm = chunkmeshresult_nucm_data_with_atlas(chunks, atlas);

// With progress callback
typedef void (*MeshProgressCallback)(int phase, uint32_t chunks_done, uint32_t chunks_total,
                                     uint64_t vertices_so_far, uint64_t triangles_so_far,
                                     void* user_data);
FFIChunkMeshResult* chunks = schematic_mesh_chunks_with_atlas_progress(
    schematic, pack, config, 32, atlas, my_callback, user_data);

textureatlas_free(atlas);
```

### 6.4 CLI (`render_schematic` example)

```bash
# Schematic → NUCM
cargo run --release --example render_schematic --features meshing -- \
    <resource_pack> <schematic> --nucm=output.nucm

# Schematic → GLB
cargo run --release --example render_schematic --features meshing -- \
    <resource_pack> <schematic> output.glb

# NUCM → Interactive viewer
cargo run --release --example render_schematic --features meshing -- \
    --cache=input.nucm --interactive
```
