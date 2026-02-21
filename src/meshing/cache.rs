//! Binary mesh cache format (`.nucm`) for fast serialization of [`MeshOutput`].
//!
//! Large schematics are expensive to mesh. This module provides efficient binary
//! serialization so pre-meshed schematics can be cached to disk and reloaded
//! without re-meshing.
//!
//! # Format overview
//!
//! ```text
//! Header (12 bytes):
//!   magic:       [u8; 4] = b"NUCM"
//!   version:     u32 LE  = 1
//!   chunk_count: u32 LE
//!
//! Per chunk:
//!   bounds, chunk_coord, lod_level,
//!   atlas (deflate-compressed pixels + region map),
//!   animated textures,
//!   3 mesh layers (opaque, cutout, transparent) with raw vertex data
//! ```
//!
//! Vertex data is stored raw (not compressed) — floats don't compress well and
//! this allows fast loading. Atlas pixels are deflate-compressed via `flate2`
//! since RGBA image data compresses ~3-4x.
//!
//! # Example
//!
//! ```ignore
//! use nucleation::meshing::cache;
//!
//! // After meshing:
//! cache::save_cached_mesh(&meshes, Path::new("scene.nucm"))?;
//!
//! // Later, load without re-meshing:
//! let meshes = cache::load_cached_mesh(Path::new("scene.nucm"))?;
//! ```

use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use schematic_mesher::atlas::AtlasRegion;
use schematic_mesher::mesher::AnimatedTextureExport;
use schematic_mesher::{BoundingBox, MeshLayer, MeshOutput, TextureAtlas};
use std::collections::HashMap;
use std::io::{self, Cursor, Read, Write};

const MAGIC: &[u8; 4] = b"NUCM";
const FORMAT_VERSION: u32 = 2;

/// Flags bitfield for the NUCM v2 header.
const FLAG_HAS_SHARED_ATLAS: u32 = 1 << 0;

/// Errors that can occur during cache serialization/deserialization.
#[derive(Debug)]
pub enum CacheError {
    /// An I/O error occurred.
    Io(io::Error),
    /// The file does not start with the expected `NUCM` magic bytes.
    InvalidMagic,
    /// The file uses a format version that this library doesn't support.
    UnsupportedVersion(u32),
    /// The data is structurally invalid (e.g. truncated, bad UTF-8).
    InvalidData(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Io(e) => write!(f, "cache I/O error: {}", e),
            CacheError::InvalidMagic => write!(f, "invalid magic bytes (expected NUCM)"),
            CacheError::UnsupportedVersion(v) => {
                write!(
                    f,
                    "unsupported cache version: {} (expected {})",
                    v, FORMAT_VERSION
                )
            }
            CacheError::InvalidData(msg) => write!(f, "invalid cache data: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CacheError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for CacheError {
    fn from(e: io::Error) -> Self {
        CacheError::Io(e)
    }
}

// ─── Public API ─────────────────────────────────────────────────────────────

/// Serialize a slice of [`MeshOutput`] to the binary `.nucm` v2 format (no shared atlas).
///
/// Each chunk stores its own atlas. For shared-atlas serialization, use
/// [`serialize_meshes_with_atlas`].
pub fn serialize_meshes(meshes: &[MeshOutput]) -> Vec<u8> {
    let mut buf = Vec::new();
    write_meshes_v2(&mut buf, meshes, None).expect("writing to Vec<u8> should not fail");
    buf
}

/// Serialize meshes with a shared global atlas (NUCM v2 with `has_shared_atlas` flag).
///
/// The shared atlas is stored once in the header. Per-chunk atlas data is omitted
/// (each chunk references the shared atlas). This dramatically reduces file size
/// for multi-chunk schematics.
pub fn serialize_meshes_with_atlas(meshes: &[MeshOutput], atlas: &TextureAtlas) -> Vec<u8> {
    let mut buf = Vec::new();
    write_meshes_v2(&mut buf, meshes, Some(atlas)).expect("writing to Vec<u8> should not fail");
    buf
}

/// Deserialize a slice of bytes (`.nucm` format) back into `Vec<MeshOutput>`.
///
/// Handles both v1 and v2 formats automatically.
pub fn deserialize_meshes(data: &[u8]) -> Result<Vec<MeshOutput>, CacheError> {
    let mut cursor = Cursor::new(data);
    read_meshes_auto(&mut cursor)
}

/// Serialize and write meshes to a file (no shared atlas).
pub fn save_cached_mesh(meshes: &[MeshOutput], path: &std::path::Path) -> Result<(), CacheError> {
    let data = serialize_meshes(meshes);
    std::fs::write(path, data)?;
    Ok(())
}

/// Serialize and write meshes to a file with a shared global atlas.
pub fn save_cached_mesh_with_atlas(
    meshes: &[MeshOutput],
    atlas: &TextureAtlas,
    path: &std::path::Path,
) -> Result<(), CacheError> {
    let data = serialize_meshes_with_atlas(meshes, atlas);
    std::fs::write(path, data)?;
    Ok(())
}

/// Load meshes from a `.nucm` cache file (handles v1 and v2 automatically).
pub fn load_cached_mesh(path: &std::path::Path) -> Result<Vec<MeshOutput>, CacheError> {
    let data = std::fs::read(path)?;
    deserialize_meshes(&data)
}

// ─── Wire helpers ───────────────────────────────────────────────────────────

fn write_u8(w: &mut impl Write, v: u8) -> io::Result<()> {
    w.write_all(&[v])
}

fn write_u32(w: &mut impl Write, v: u32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

fn write_i32(w: &mut impl Write, v: i32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

fn write_f32(w: &mut impl Write, v: f32) -> io::Result<()> {
    w.write_all(&v.to_le_bytes())
}

fn write_bytes(w: &mut impl Write, data: &[u8]) -> io::Result<()> {
    w.write_all(data)
}

fn read_u8(r: &mut impl Read) -> Result<u8, CacheError> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u32(r: &mut impl Read) -> Result<u32, CacheError> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_i32(r: &mut impl Read) -> Result<i32, CacheError> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

fn read_f32(r: &mut impl Read) -> Result<f32, CacheError> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

fn read_bytes(r: &mut impl Read, len: usize) -> Result<Vec<u8>, CacheError> {
    let mut buf = vec![0u8; len];
    r.read_exact(&mut buf)?;
    Ok(buf)
}

// ─── Top-level serialize / deserialize ──────────────────────────────────────

/// Write v2 format. If `shared_atlas` is Some, stores atlas once in header.
fn write_meshes_v2(
    w: &mut impl Write,
    meshes: &[MeshOutput],
    shared_atlas: Option<&TextureAtlas>,
) -> Result<(), CacheError> {
    // Header: magic + version + flags + chunk_count
    write_bytes(w, MAGIC)?;
    write_u32(w, FORMAT_VERSION)?;

    let flags = if shared_atlas.is_some() {
        FLAG_HAS_SHARED_ATLAS
    } else {
        0
    };
    write_u32(w, flags)?;
    write_u32(w, meshes.len() as u32)?;

    // Shared atlas (if present)
    if let Some(atlas) = shared_atlas {
        write_atlas(w, atlas)?;
    }

    for mesh in meshes {
        write_mesh_output_v2(w, mesh, shared_atlas.is_some())?;
    }
    Ok(())
}

/// Read with auto-detection of v1 and v2 formats.
fn read_meshes_auto(r: &mut impl Read) -> Result<Vec<MeshOutput>, CacheError> {
    let mut magic = [0u8; 4];
    r.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(CacheError::InvalidMagic);
    }

    let version = read_u32(r)?;

    match version {
        1 => read_meshes_v1(r),
        2 => read_meshes_v2(r),
        _ => Err(CacheError::UnsupportedVersion(version)),
    }
}

/// Read v1 format (no flags, per-chunk atlas).
fn read_meshes_v1(r: &mut impl Read) -> Result<Vec<MeshOutput>, CacheError> {
    let chunk_count = read_u32(r)? as usize;
    let mut meshes = Vec::with_capacity(chunk_count);
    for _ in 0..chunk_count {
        meshes.push(read_mesh_output_v1(r)?);
    }
    Ok(meshes)
}

/// Read v2 format (flags + optional shared atlas).
fn read_meshes_v2(r: &mut impl Read) -> Result<Vec<MeshOutput>, CacheError> {
    let flags = read_u32(r)?;
    let chunk_count = read_u32(r)? as usize;

    let has_shared_atlas = flags & FLAG_HAS_SHARED_ATLAS != 0;

    // Read shared atlas from header if present
    let shared_atlas = if has_shared_atlas {
        Some(read_atlas(r)?)
    } else {
        None
    };

    let mut meshes = Vec::with_capacity(chunk_count);
    for _ in 0..chunk_count {
        meshes.push(read_mesh_output_v2(r, shared_atlas.as_ref())?);
    }
    Ok(meshes)
}

// ─── MeshOutput ─────────────────────────────────────────────────────────────

/// Write a chunk in v2 format. If `uses_shared_atlas` is true, skip per-chunk atlas.
fn write_mesh_output_v2(
    w: &mut impl Write,
    mesh: &MeshOutput,
    uses_shared_atlas: bool,
) -> Result<(), CacheError> {
    // Bounds
    for v in &mesh.bounds.min {
        write_f32(w, *v)?;
    }
    for v in &mesh.bounds.max {
        write_f32(w, *v)?;
    }

    // Chunk coord
    match mesh.chunk_coord {
        Some((cx, cy, cz)) => {
            write_u8(w, 1)?;
            write_i32(w, cx)?;
            write_i32(w, cy)?;
            write_i32(w, cz)?;
        }
        None => {
            write_u8(w, 0)?;
        }
    }

    // LOD level
    write_u8(w, mesh.lod_level)?;

    // Atlas mode: 0 = uses shared atlas (skip), 1 = has own atlas
    if uses_shared_atlas {
        write_u8(w, 0)?; // references shared atlas
    } else {
        write_u8(w, 1)?; // has own atlas
        write_atlas(w, &mesh.atlas)?;
    }

    // Animated textures
    write_u32(w, mesh.animated_textures.len() as u32)?;
    for anim in &mesh.animated_textures {
        write_animated_texture(w, anim)?;
    }

    // 3 layers: opaque, cutout, transparent
    write_layer(w, &mesh.opaque)?;
    write_layer(w, &mesh.cutout)?;
    write_layer(w, &mesh.transparent)?;

    Ok(())
}

/// Read a chunk in v1 format (always has per-chunk atlas).
fn read_mesh_output_v1(r: &mut impl Read) -> Result<MeshOutput, CacheError> {
    // Bounds
    let min = [read_f32(r)?, read_f32(r)?, read_f32(r)?];
    let max = [read_f32(r)?, read_f32(r)?, read_f32(r)?];
    let bounds = BoundingBox::new(min, max);

    // Chunk coord
    let has_chunk_coord = read_u8(r)?;
    let chunk_coord = if has_chunk_coord == 1 {
        Some((read_i32(r)?, read_i32(r)?, read_i32(r)?))
    } else {
        None
    };

    // LOD level
    let lod_level = read_u8(r)?;

    // Atlas (always present in v1)
    let atlas = read_atlas(r)?;

    // Animated textures
    let anim_count = read_u32(r)? as usize;
    let mut animated_textures = Vec::with_capacity(anim_count);
    for _ in 0..anim_count {
        animated_textures.push(read_animated_texture(r)?);
    }

    // 3 layers
    let opaque = read_layer(r)?;
    let cutout = read_layer(r)?;
    let transparent = read_layer(r)?;

    Ok(MeshOutput {
        opaque,
        cutout,
        transparent,
        atlas,
        animated_textures,
        bounds,
        chunk_coord,
        lod_level,
    })
}

/// Read a chunk in v2 format. `shared_atlas` from the header is used when atlas_mode==0.
fn read_mesh_output_v2(
    r: &mut impl Read,
    shared_atlas: Option<&TextureAtlas>,
) -> Result<MeshOutput, CacheError> {
    // Bounds
    let min = [read_f32(r)?, read_f32(r)?, read_f32(r)?];
    let max = [read_f32(r)?, read_f32(r)?, read_f32(r)?];
    let bounds = BoundingBox::new(min, max);

    // Chunk coord
    let has_chunk_coord = read_u8(r)?;
    let chunk_coord = if has_chunk_coord == 1 {
        Some((read_i32(r)?, read_i32(r)?, read_i32(r)?))
    } else {
        None
    };

    // LOD level
    let lod_level = read_u8(r)?;

    // Atlas mode
    let atlas_mode = read_u8(r)?;
    let atlas = if atlas_mode == 0 {
        // Uses shared atlas from header
        match shared_atlas {
            Some(a) => a.clone(),
            None => {
                return Err(CacheError::InvalidData(
                    "chunk references shared atlas but none was provided".into(),
                ))
            }
        }
    } else {
        // Has own atlas
        read_atlas(r)?
    };

    // Animated textures
    let anim_count = read_u32(r)? as usize;
    let mut animated_textures = Vec::with_capacity(anim_count);
    for _ in 0..anim_count {
        animated_textures.push(read_animated_texture(r)?);
    }

    // 3 layers
    let opaque = read_layer(r)?;
    let cutout = read_layer(r)?;
    let transparent = read_layer(r)?;

    Ok(MeshOutput {
        opaque,
        cutout,
        transparent,
        atlas,
        animated_textures,
        bounds,
        chunk_coord,
        lod_level,
    })
}

// ─── MeshLayer ──────────────────────────────────────────────────────────────

/// Deflate-compress a byte slice and write length-prefixed to the output.
fn write_compressed_field(w: &mut impl Write, data: &[u8]) -> Result<(), CacheError> {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    write_u32(w, data.len() as u32)?;
    write_u32(w, compressed.len() as u32)?;
    write_bytes(w, &compressed)?;
    Ok(())
}

/// Read a length-prefixed deflate-compressed field.
fn read_compressed_field(r: &mut impl Read) -> Result<Vec<u8>, CacheError> {
    let raw_len = read_u32(r)? as usize;
    let compressed_len = read_u32(r)? as usize;
    let compressed = read_bytes(r, compressed_len)?;
    let mut decoder = DeflateDecoder::new(&compressed[..]);
    let mut raw = Vec::with_capacity(raw_len);
    decoder.read_to_end(&mut raw)?;
    if raw.len() != raw_len {
        return Err(CacheError::InvalidData(format!(
            "field size mismatch: expected {}, got {}",
            raw_len,
            raw.len()
        )));
    }
    Ok(raw)
}

fn write_layer(w: &mut impl Write, layer: &MeshLayer) -> Result<(), CacheError> {
    let vertex_count = layer.vertex_count() as u32;
    let index_count = layer.indices.len() as u32;

    write_u32(w, vertex_count)?;
    write_u32(w, index_count)?;

    if vertex_count == 0 {
        // Empty layer — write zero-length compressed fields for indices only
        write_compressed_field(w, &[])?;
        return Ok(());
    }

    // ── Positions: quantize to u16×3 relative to per-layer AABB ──
    // Compute layer bounding box
    let mut pos_min = [f32::MAX; 3];
    let mut pos_max = [f32::MIN; 3];
    for p in &layer.positions {
        for i in 0..3 {
            pos_min[i] = pos_min[i].min(p[i]);
            pos_max[i] = pos_max[i].max(p[i]);
        }
    }
    // Write the AABB so we can dequantize on load
    for i in 0..3 {
        write_f32(w, pos_min[i])?;
    }
    for i in 0..3 {
        write_f32(w, pos_max[i])?;
    }

    // Quantize positions: map [min, max] → [0, 65535], then delta-encode.
    // Delta encoding makes consecutive similar positions produce small values
    // which deflate compresses dramatically better.
    let mut pos_buf = Vec::with_capacity(vertex_count as usize * 6);
    let mut prev = [0u16; 3];
    for p in &layer.positions {
        for i in 0..3 {
            let range = pos_max[i] - pos_min[i];
            let q = if range > 0.0 {
                (((p[i] - pos_min[i]) / range) * 65535.0 + 0.5) as u16
            } else {
                0
            };
            let delta = q.wrapping_sub(prev[i]);
            pos_buf.extend_from_slice(&delta.to_le_bytes());
            prev[i] = q;
        }
    }
    write_compressed_field(w, &pos_buf)?;

    // ── Normals: quantize to i8×3 ──
    // Axis-aligned normals (the vast majority) roundtrip exactly.
    // Non-axis-aligned normals get a close approximation.
    let mut norm_buf = Vec::with_capacity(vertex_count as usize * 3);
    for n in &layer.normals {
        for i in 0..3 {
            norm_buf.push((n[i] * 127.0) as i8 as u8);
        }
    }
    write_compressed_field(w, &norm_buf)?;

    // ── UVs: quantize to u16×2 ──
    // UV range is typically [0, 1] but can exceed for tiled textures.
    // Store min/max and quantize relative to that.
    let mut uv_min = [f32::MAX; 2];
    let mut uv_max = [f32::MIN; 2];
    for uv in &layer.uvs {
        for i in 0..2 {
            uv_min[i] = uv_min[i].min(uv[i]);
            uv_max[i] = uv_max[i].max(uv[i]);
        }
    }
    for i in 0..2 {
        write_f32(w, uv_min[i])?;
    }
    for i in 0..2 {
        write_f32(w, uv_max[i])?;
    }

    let mut uv_buf = Vec::with_capacity(vertex_count as usize * 4);
    for uv in &layer.uvs {
        for i in 0..2 {
            let range = uv_max[i] - uv_min[i];
            let q = if range > 0.0 {
                (((uv[i] - uv_min[i]) / range) * 65535.0 + 0.5) as u16
            } else {
                0
            };
            uv_buf.extend_from_slice(&q.to_le_bytes());
        }
    }
    write_compressed_field(w, &uv_buf)?;

    // ── Colors: quantize f32×4 → u8×4 ──
    let mut col_buf = Vec::with_capacity(vertex_count as usize * 4);
    for c in &layer.colors {
        for i in 0..4 {
            col_buf.push((c[i].clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
        }
    }
    write_compressed_field(w, &col_buf)?;

    // ── Indices: delta-encoded u32 LE, deflate-compressed ──
    // Quad meshes produce patterns like 0,1,2,2,3,0,4,5,6,6,7,4,...
    // Delta encoding turns this into small repeated values that compress well.
    let mut idx_buf = Vec::with_capacity(index_count as usize * 4);
    let mut prev_idx = 0u32;
    for &idx in &layer.indices {
        let delta = idx.wrapping_sub(prev_idx);
        idx_buf.extend_from_slice(&delta.to_le_bytes());
        prev_idx = idx;
    }
    write_compressed_field(w, &idx_buf)?;

    Ok(())
}

fn read_layer(r: &mut impl Read) -> Result<MeshLayer, CacheError> {
    let vertex_count = read_u32(r)? as usize;
    let index_count = read_u32(r)? as usize;

    if vertex_count == 0 {
        let _indices_raw = read_compressed_field(r)?;
        return Ok(MeshLayer::default());
    }

    // ── Positions: read AABB then dequantize u16×3 → f32×3 ──
    let pos_min = [read_f32(r)?, read_f32(r)?, read_f32(r)?];
    let pos_max = [read_f32(r)?, read_f32(r)?, read_f32(r)?];
    let pos_raw = read_compressed_field(r)?;
    if pos_raw.len() != vertex_count * 6 {
        return Err(CacheError::InvalidData(
            "position field size mismatch".into(),
        ));
    }
    // Undo delta encoding, then dequantize u16 → f32
    let mut prev = [0u16; 3];
    let positions: Vec<[f32; 3]> = pos_raw
        .chunks_exact(6)
        .map(|chunk| {
            let mut out = [0.0f32; 3];
            for i in 0..3 {
                let delta = u16::from_le_bytes([chunk[i * 2], chunk[i * 2 + 1]]);
                let q = prev[i].wrapping_add(delta);
                prev[i] = q;
                let range = pos_max[i] - pos_min[i];
                out[i] = if range > 0.0 {
                    pos_min[i] + (q as f32 / 65535.0) * range
                } else {
                    pos_min[i]
                };
            }
            out
        })
        .collect();

    // ── Normals: dequantize i8×3 → f32×3, then normalize ──
    let norm_raw = read_compressed_field(r)?;
    if norm_raw.len() != vertex_count * 3 {
        return Err(CacheError::InvalidData("normal field size mismatch".into()));
    }
    let normals: Vec<[f32; 3]> = norm_raw
        .chunks_exact(3)
        .map(|chunk| {
            let mut n = [0.0f32; 3];
            for i in 0..3 {
                n[i] = (chunk[i] as i8) as f32 / 127.0;
            }
            // Renormalize to unit length
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 0.0 {
                n[0] /= len;
                n[1] /= len;
                n[2] /= len;
            }
            n
        })
        .collect();

    // ── UVs: read UV AABB then dequantize u16×2 → f32×2 ──
    let uv_min = [read_f32(r)?, read_f32(r)?];
    let uv_max = [read_f32(r)?, read_f32(r)?];
    let uv_raw = read_compressed_field(r)?;
    if uv_raw.len() != vertex_count * 4 {
        return Err(CacheError::InvalidData("UV field size mismatch".into()));
    }
    let uvs: Vec<[f32; 2]> = uv_raw
        .chunks_exact(4)
        .map(|chunk| {
            let mut out = [0.0f32; 2];
            for i in 0..2 {
                let q = u16::from_le_bytes([chunk[i * 2], chunk[i * 2 + 1]]);
                let range = uv_max[i] - uv_min[i];
                out[i] = if range > 0.0 {
                    uv_min[i] + (q as f32 / 65535.0) * range
                } else {
                    uv_min[i]
                };
            }
            out
        })
        .collect();

    // ── Colors: dequantize u8×4 → f32×4 ──
    let col_raw = read_compressed_field(r)?;
    if col_raw.len() != vertex_count * 4 {
        return Err(CacheError::InvalidData("color field size mismatch".into()));
    }
    let colors: Vec<[f32; 4]> = col_raw
        .chunks_exact(4)
        .map(|chunk| {
            [
                chunk[0] as f32 / 255.0,
                chunk[1] as f32 / 255.0,
                chunk[2] as f32 / 255.0,
                chunk[3] as f32 / 255.0,
            ]
        })
        .collect();

    // ── Indices: undo delta encoding ──
    let indices_raw = read_compressed_field(r)?;
    if indices_raw.len() != index_count * 4 {
        return Err(CacheError::InvalidData("index field size mismatch".into()));
    }
    let mut prev_idx = 0u32;
    let indices: Vec<u32> = indices_raw
        .chunks_exact(4)
        .map(|chunk| {
            let delta = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let idx = prev_idx.wrapping_add(delta);
            prev_idx = idx;
            idx
        })
        .collect();

    Ok(MeshLayer {
        positions,
        normals,
        uvs,
        colors,
        indices,
    })
}

// ─── TextureAtlas ───────────────────────────────────────────────────────────

fn write_atlas(w: &mut impl Write, atlas: &TextureAtlas) -> Result<(), CacheError> {
    write_u32(w, atlas.width)?;
    write_u32(w, atlas.height)?;

    // Deflate-compress the raw RGBA pixel data
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&atlas.pixels)?;
    let compressed = encoder.finish()?;

    write_u32(w, atlas.pixels.len() as u32)?; // uncompressed size
    write_u32(w, compressed.len() as u32)?; // compressed size
    write_bytes(w, &compressed)?;

    // Regions
    write_u32(w, atlas.regions.len() as u32)?;
    for (name, region) in &atlas.regions {
        let name_bytes = name.as_bytes();
        write_u32(w, name_bytes.len() as u32)?;
        write_bytes(w, name_bytes)?;
        write_f32(w, region.u_min)?;
        write_f32(w, region.v_min)?;
        write_f32(w, region.u_max)?;
        write_f32(w, region.v_max)?;
    }

    Ok(())
}

fn read_atlas(r: &mut impl Read) -> Result<TextureAtlas, CacheError> {
    let width = read_u32(r)?;
    let height = read_u32(r)?;

    let raw_len = read_u32(r)? as usize;
    let compressed_len = read_u32(r)? as usize;
    let compressed = read_bytes(r, compressed_len)?;

    // Decompress
    let mut decoder = DeflateDecoder::new(&compressed[..]);
    let mut pixels = Vec::with_capacity(raw_len);
    decoder.read_to_end(&mut pixels)?;

    if pixels.len() != raw_len {
        return Err(CacheError::InvalidData(format!(
            "atlas pixel size mismatch: expected {}, got {}",
            raw_len,
            pixels.len()
        )));
    }

    // Regions
    let region_count = read_u32(r)? as usize;
    let mut regions = HashMap::with_capacity(region_count);
    for _ in 0..region_count {
        let name_len = read_u32(r)? as usize;
        let name_bytes = read_bytes(r, name_len)?;
        let name = String::from_utf8(name_bytes)
            .map_err(|e| CacheError::InvalidData(format!("invalid region name: {}", e)))?;

        let u_min = read_f32(r)?;
        let v_min = read_f32(r)?;
        let u_max = read_f32(r)?;
        let v_max = read_f32(r)?;

        regions.insert(
            name,
            AtlasRegion {
                u_min,
                v_min,
                u_max,
                v_max,
            },
        );
    }

    Ok(TextureAtlas {
        width,
        height,
        pixels,
        regions,
    })
}

// ─── AnimatedTextureExport ──────────────────────────────────────────────────

fn write_animated_texture(
    w: &mut impl Write,
    anim: &AnimatedTextureExport,
) -> Result<(), CacheError> {
    write_u32(w, anim.sprite_sheet_png.len() as u32)?;
    write_bytes(w, &anim.sprite_sheet_png)?;

    write_u32(w, anim.frame_count)?;
    write_u32(w, anim.frametime)?;
    write_u8(w, anim.interpolate as u8)?;

    match &anim.frames {
        Some(frames) => {
            write_u8(w, 1)?;
            write_u32(w, frames.len() as u32)?;
            for &f in frames {
                write_u32(w, f)?;
            }
        }
        None => {
            write_u8(w, 0)?;
        }
    }

    write_u32(w, anim.frame_width)?;
    write_u32(w, anim.frame_height)?;
    write_u32(w, anim.atlas_x)?;
    write_u32(w, anim.atlas_y)?;

    Ok(())
}

fn read_animated_texture(r: &mut impl Read) -> Result<AnimatedTextureExport, CacheError> {
    let sprite_len = read_u32(r)? as usize;
    let sprite_sheet_png = read_bytes(r, sprite_len)?;

    let frame_count = read_u32(r)?;
    let frametime = read_u32(r)?;
    let interpolate = read_u8(r)? != 0;

    let has_frames = read_u8(r)?;
    let frames = if has_frames == 1 {
        let count = read_u32(r)? as usize;
        let mut v = Vec::with_capacity(count);
        for _ in 0..count {
            v.push(read_u32(r)?);
        }
        Some(v)
    } else {
        None
    };

    let frame_width = read_u32(r)?;
    let frame_height = read_u32(r)?;
    let atlas_x = read_u32(r)?;
    let atlas_y = read_u32(r)?;

    Ok(AnimatedTextureExport {
        sprite_sheet_png,
        frame_count,
        frametime,
        interpolate,
        frames,
        frame_width,
        frame_height,
        atlas_x,
        atlas_y,
    })
}

// ─── Byte ↔ typed-array conversions ─────────────────────────────────────────

fn bytes_to_f32x3(data: &[u8]) -> Vec<[f32; 3]> {
    data.chunks_exact(12)
        .map(|chunk| {
            [
                f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
                f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]),
                f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]),
            ]
        })
        .collect()
}

fn bytes_to_f32x2(data: &[u8]) -> Vec<[f32; 2]> {
    data.chunks_exact(8)
        .map(|chunk| {
            [
                f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
                f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]),
            ]
        })
        .collect()
}

fn bytes_to_f32x4(data: &[u8]) -> Vec<[f32; 4]> {
    data.chunks_exact(16)
        .map(|chunk| {
            [
                f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]),
                f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]),
                f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]),
                f32::from_le_bytes([chunk[12], chunk[13], chunk[14], chunk[15]]),
            ]
        })
        .collect()
}

fn bytes_to_u32(data: &[u8]) -> Vec<u32> {
    data.chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_layer(vertex_count: usize) -> MeshLayer {
        MeshLayer {
            positions: (0..vertex_count)
                .map(|i| [i as f32, (i as f32) * 2.0, (i as f32) * 3.0])
                .collect(),
            normals: vec![[0.0, 1.0, 0.0]; vertex_count],
            uvs: (0..vertex_count)
                .map(|i| [i as f32 / vertex_count as f32, 0.5])
                .collect(),
            colors: vec![[1.0, 1.0, 1.0, 1.0]; vertex_count],
            indices: (0..vertex_count as u32).collect(),
        }
    }

    fn make_test_mesh_output() -> MeshOutput {
        MeshOutput {
            opaque: make_test_layer(6),
            cutout: make_test_layer(3),
            transparent: MeshLayer::default(),
            atlas: TextureAtlas {
                width: 32,
                height: 32,
                pixels: vec![128u8; 32 * 32 * 4],
                regions: {
                    let mut m = HashMap::new();
                    m.insert(
                        "minecraft:stone".to_string(),
                        AtlasRegion {
                            u_min: 0.0,
                            v_min: 0.0,
                            u_max: 0.5,
                            v_max: 0.5,
                        },
                    );
                    m.insert(
                        "minecraft:dirt".to_string(),
                        AtlasRegion {
                            u_min: 0.5,
                            v_min: 0.0,
                            u_max: 1.0,
                            v_max: 0.5,
                        },
                    );
                    m
                },
            },
            animated_textures: vec![AnimatedTextureExport {
                sprite_sheet_png: vec![0xDE, 0xAD, 0xBE, 0xEF],
                frame_count: 4,
                frametime: 2,
                interpolate: true,
                frames: Some(vec![0, 1, 2, 3, 2, 1]),
                frame_width: 16,
                frame_height: 16,
                atlas_x: 0,
                atlas_y: 0,
            }],
            bounds: BoundingBox::new([0.0, 0.0, 0.0], [10.0, 5.0, 10.0]),
            chunk_coord: Some((1, 0, -2)),
            lod_level: 0,
        }
    }

    /// Assert two f32 slices are approximately equal (for quantized roundtrip).
    fn assert_f32_approx(a: &[[f32; 3]], b: &[[f32; 3]], tolerance: f32, label: &str) {
        assert_eq!(a.len(), b.len(), "{} length mismatch", label);
        for (i, (va, vb)) in a.iter().zip(b.iter()).enumerate() {
            for j in 0..3 {
                assert!(
                    (va[j] - vb[j]).abs() <= tolerance,
                    "{} mismatch at [{i}][{j}]: {} vs {} (diff {})",
                    label,
                    va[j],
                    vb[j],
                    (va[j] - vb[j]).abs()
                );
            }
        }
    }

    #[test]
    fn roundtrip_single_mesh() {
        let original = make_test_mesh_output();
        let bytes = serialize_meshes(&[original.clone()]);
        let restored = deserialize_meshes(&bytes).unwrap();

        assert_eq!(restored.len(), 1);
        let r = &restored[0];

        // Bounds
        assert_eq!(r.bounds.min, [0.0, 0.0, 0.0]);
        assert_eq!(r.bounds.max, [10.0, 5.0, 10.0]);

        // Chunk coord
        assert_eq!(r.chunk_coord, Some((1, 0, -2)));
        assert_eq!(r.lod_level, 0);

        // Layers
        assert_eq!(r.opaque.vertex_count(), 6);
        assert_eq!(r.cutout.vertex_count(), 3);
        assert!(r.transparent.is_empty());

        // Verify vertex data roundtrips within quantization tolerance.
        // Positions: u16 quantized over a range of ~15 units → error ≤ 15/65535 ≈ 0.0003
        assert_f32_approx(
            &r.opaque.positions,
            &original.opaque.positions,
            0.001,
            "positions",
        );
        // Normals: i8 quantized → axis-aligned normals roundtrip within ±0.008
        assert_f32_approx(&r.opaque.normals, &original.opaque.normals, 0.01, "normals");
        // Colors: u8 quantized → error ≤ 1/255 ≈ 0.004
        for (a, b) in r.opaque.colors.iter().zip(original.opaque.colors.iter()) {
            for i in 0..4 {
                assert!((a[i] - b[i]).abs() < 0.005, "color mismatch");
            }
        }
        // Indices are lossless
        assert_eq!(r.opaque.indices, original.opaque.indices);

        // Atlas
        assert_eq!(r.atlas.width, 32);
        assert_eq!(r.atlas.height, 32);
        assert_eq!(r.atlas.pixels, original.atlas.pixels);
        assert_eq!(r.atlas.regions.len(), 2);

        let stone = r.atlas.regions.get("minecraft:stone").unwrap();
        assert!((stone.u_min - 0.0).abs() < f32::EPSILON);
        assert!((stone.u_max - 0.5).abs() < f32::EPSILON);

        // Animated textures
        assert_eq!(r.animated_textures.len(), 1);
        let anim = &r.animated_textures[0];
        assert_eq!(anim.sprite_sheet_png, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(anim.frame_count, 4);
        assert_eq!(anim.frametime, 2);
        assert!(anim.interpolate);
        assert_eq!(anim.frames, Some(vec![0, 1, 2, 3, 2, 1]));
        assert_eq!(anim.frame_width, 16);
        assert_eq!(anim.frame_height, 16);
    }

    #[test]
    fn roundtrip_multiple_meshes() {
        let mut m1 = make_test_mesh_output();
        m1.chunk_coord = Some((0, 0, 0));

        let mut m2 = make_test_mesh_output();
        m2.chunk_coord = Some((1, 0, 0));
        m2.animated_textures.clear();

        let bytes = serialize_meshes(&[m1, m2]);
        let restored = deserialize_meshes(&bytes).unwrap();

        assert_eq!(restored.len(), 2);
        assert_eq!(restored[0].chunk_coord, Some((0, 0, 0)));
        assert_eq!(restored[1].chunk_coord, Some((1, 0, 0)));
        assert_eq!(restored[1].animated_textures.len(), 0);
    }

    #[test]
    fn roundtrip_no_chunk_coord() {
        let mut mesh = make_test_mesh_output();
        mesh.chunk_coord = None;

        let bytes = serialize_meshes(&[mesh]);
        let restored = deserialize_meshes(&bytes).unwrap();
        assert_eq!(restored[0].chunk_coord, None);
    }

    #[test]
    fn roundtrip_empty_mesh() {
        let mesh = MeshOutput {
            opaque: MeshLayer::default(),
            cutout: MeshLayer::default(),
            transparent: MeshLayer::default(),
            atlas: TextureAtlas::empty(),
            animated_textures: Vec::new(),
            bounds: BoundingBox::new([0.0; 3], [0.0; 3]),
            chunk_coord: None,
            lod_level: 0,
        };

        let bytes = serialize_meshes(&[mesh]);
        let restored = deserialize_meshes(&bytes).unwrap();

        assert_eq!(restored.len(), 1);
        assert!(restored[0].is_empty());
    }

    #[test]
    fn roundtrip_empty_vec() {
        let bytes = serialize_meshes(&[]);
        let restored = deserialize_meshes(&bytes).unwrap();
        assert!(restored.is_empty());
    }

    #[test]
    fn roundtrip_animated_no_frames() {
        let mut mesh = make_test_mesh_output();
        mesh.animated_textures[0].frames = None;

        let bytes = serialize_meshes(&[mesh]);
        let restored = deserialize_meshes(&bytes).unwrap();
        assert_eq!(restored[0].animated_textures[0].frames, None);
    }

    #[test]
    fn invalid_magic_is_rejected() {
        let mut data = serialize_meshes(&[make_test_mesh_output()]);
        data[0] = b'X';
        let err = deserialize_meshes(&data).unwrap_err();
        assert!(matches!(err, CacheError::InvalidMagic));
    }

    #[test]
    fn unsupported_version_is_rejected() {
        let mut data = serialize_meshes(&[make_test_mesh_output()]);
        // Version is at bytes 4..8
        data[4..8].copy_from_slice(&99u32.to_le_bytes());
        let err = deserialize_meshes(&data).unwrap_err();
        assert!(matches!(err, CacheError::UnsupportedVersion(99)));
    }

    #[test]
    fn v1_files_still_load() {
        // Manually construct a v1 file and verify it loads
        let original = make_test_mesh_output();
        let mut buf = Vec::new();
        // Write v1 header
        buf.extend_from_slice(MAGIC);
        buf.extend_from_slice(&1u32.to_le_bytes()); // version 1
        buf.extend_from_slice(&1u32.to_le_bytes()); // chunk_count = 1
                                                    // Write the chunk in v1 format (which has no atlas_mode byte)
        let mut chunk_buf = Vec::new();
        write_mesh_output_v2(&mut chunk_buf, &original, false).unwrap();
        // v1 has no atlas_mode byte — it goes straight to atlas after lod_level.
        // We need to strip the atlas_mode byte (which is `1` for "has own atlas").
        // Actually, for the v1 reader, the format is: bounds, chunk_coord, lod, atlas, anim, layers.
        // Our v2 adds atlas_mode between lod and atlas. So a v1 file won't have it.
        // Let's just use the original v1 write path directly:
        let mut v1_buf = Vec::new();
        v1_buf.extend_from_slice(MAGIC);
        v1_buf.extend_from_slice(&1u32.to_le_bytes());
        v1_buf.extend_from_slice(&1u32.to_le_bytes());
        // Write bounds
        for v in &original.bounds.min {
            v1_buf.extend_from_slice(&v.to_le_bytes());
        }
        for v in &original.bounds.max {
            v1_buf.extend_from_slice(&v.to_le_bytes());
        }
        // chunk_coord
        v1_buf.push(1); // has chunk coord
        for v in [1i32, 0, -2] {
            v1_buf.extend_from_slice(&v.to_le_bytes());
        }
        // lod
        v1_buf.push(0);
        // atlas (write directly)
        let mut atlas_buf = Vec::new();
        write_atlas(&mut atlas_buf, &original.atlas).unwrap();
        v1_buf.extend_from_slice(&atlas_buf);
        // animated textures
        let mut anim_buf = Vec::new();
        write_u32(&mut anim_buf, original.animated_textures.len() as u32).unwrap();
        for a in &original.animated_textures {
            write_animated_texture(&mut anim_buf, a).unwrap();
        }
        v1_buf.extend_from_slice(&anim_buf);
        // layers
        let mut layer_buf = Vec::new();
        write_layer(&mut layer_buf, &original.opaque).unwrap();
        write_layer(&mut layer_buf, &original.cutout).unwrap();
        write_layer(&mut layer_buf, &original.transparent).unwrap();
        v1_buf.extend_from_slice(&layer_buf);

        let restored = deserialize_meshes(&v1_buf).unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].chunk_coord, Some((1, 0, -2)));
    }

    #[test]
    fn roundtrip_with_shared_atlas() {
        let m1 = make_test_mesh_output();
        let m2 = {
            let mut m = make_test_mesh_output();
            m.chunk_coord = Some((2, 0, 0));
            m
        };

        let shared_atlas = m1.atlas.clone();
        let data = serialize_meshes_with_atlas(&[m1, m2], &shared_atlas);
        let restored = deserialize_meshes(&data).unwrap();

        assert_eq!(restored.len(), 2);
        // Both chunks should have the shared atlas
        assert_eq!(restored[0].atlas.width, shared_atlas.width);
        assert_eq!(restored[0].atlas.height, shared_atlas.height);
        assert_eq!(restored[1].atlas.width, shared_atlas.width);
        assert_eq!(restored[1].atlas.pixels, shared_atlas.pixels);
    }

    #[test]
    fn truncated_data_is_rejected() {
        let data = serialize_meshes(&[make_test_mesh_output()]);
        let err = deserialize_meshes(&data[..20]).unwrap_err();
        assert!(matches!(err, CacheError::Io(_)));
    }

    #[test]
    fn header_is_correct() {
        let data = serialize_meshes(&[make_test_mesh_output()]);
        assert_eq!(&data[0..4], b"NUCM");
        assert_eq!(u32::from_le_bytes(data[4..8].try_into().unwrap()), 2); // version
        assert_eq!(u32::from_le_bytes(data[8..12].try_into().unwrap()), 0); // flags (no shared atlas)
        assert_eq!(u32::from_le_bytes(data[12..16].try_into().unwrap()), 1); // chunk_count
    }

    #[test]
    fn byte_conversion_roundtrips() {
        let f3 = vec![[1.0f32, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let raw: Vec<u8> = f3
            .iter()
            .flat_map(|v| v.iter().flat_map(|f| f.to_le_bytes()))
            .collect();
        assert_eq!(bytes_to_f32x3(&raw), f3);

        let f2 = vec![[0.5f32, 0.75], [0.25, 1.0]];
        let raw: Vec<u8> = f2
            .iter()
            .flat_map(|v| v.iter().flat_map(|f| f.to_le_bytes()))
            .collect();
        assert_eq!(bytes_to_f32x2(&raw), f2);

        let f4 = vec![[1.0f32, 0.0, 0.0, 1.0]];
        let raw: Vec<u8> = f4
            .iter()
            .flat_map(|v| v.iter().flat_map(|f| f.to_le_bytes()))
            .collect();
        assert_eq!(bytes_to_f32x4(&raw), f4);

        let u = vec![42u32, 100, 0, u32::MAX];
        let raw: Vec<u8> = u.iter().flat_map(|v| v.to_le_bytes()).collect();
        assert_eq!(bytes_to_u32(&raw), u);
    }
}
