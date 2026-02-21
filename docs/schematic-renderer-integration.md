# Schematic-Renderer Integration Guide: Global Atlas + NUCM v2 + Progress

This document is a handoff for the schematic-renderer agent. It describes all API and format changes introduced by the global atlas implementation in Nucleation v0.1.163+.

---

## What Changed (TL;DR)

1. **Global Atlas**: A single shared texture atlas is now built once for all chunks, instead of per-chunk. This eliminates duplicate atlas data and enables GPU batching.
2. **NUCM v2**: The binary mesh cache format now supports a shared atlas header. Files are dramatically smaller for multi-chunk schematics (~99% atlas storage reduction).
3. **Progress Reporting**: Chunked meshing now reports progress via callbacks (phase, chunk count, vertex/triangle counts).

All changes are **backward compatible**. Existing v1 `.nucm` files still load. The old per-chunk atlas workflow still works.

---

## 1. New Meshing Workflow

### Before (per-chunk atlas — old way, still works)

```
schematic + resource_pack + config
    → mesh_chunks(chunk_size=16)
    → Vec<MeshOutput>, each with its OWN atlas (duplicated across chunks)
    → serialize to NUCM v1
```

### After (global atlas — recommended for multi-chunk)

```
schematic + resource_pack + config
    → build_global_atlas()           ← NEW: scan palettes, build one atlas
    → mesh_chunks_with_atlas(atlas)  ← NEW: chunks skip atlas building
    → Vec<MeshOutput>, all sharing the SAME atlas
    → serialize to NUCM v2 with shared atlas header
```

### Why This Matters for the Renderer

- **GPU texture management**: With shared atlas, all chunks use the same texture. You can bind it once and draw all chunks without texture switches.
- **NUCM file size**: IRIS_B went from ~570 MB (v1, per-chunk atlas) to a fraction of that (v2, one shared atlas).
- **Loading speed**: Only one atlas to decompress and upload to GPU, not one per chunk.

---

## 2. API Reference by Binding

### 2.1 WASM (JavaScript/TypeScript)

```javascript
import init, { Schematic, ResourcePack, MeshConfig } from 'nucleation';

// Load schematic and resource pack
const schematic = Schematic.fromBytes(schemBytes);
const pack = new ResourcePack(packBytes);
const config = new MeshConfig();

// ── NEW: Build global atlas ──
const atlas = schematic.buildGlobalAtlas(pack, config);
// atlas.width()    → number (pixels)
// atlas.height()   → number (pixels)
// atlas.toBytes()  → Uint8Array (RGBA pixel data, width * height * 4)

// ── NEW: Create chunk iterator with shared atlas ──
const iter = schematic.chunkMeshIteratorWithAtlas(pack, config, 16, atlas);

// ── NEW: Progress callback ──
iter.setProgressCallback((progress) => {
    // progress.phase: "BuildingAtlas" | "MeshingChunks" | "Complete"
    // progress.chunksDone: number
    // progress.chunksTotal: number
    // progress.verticesSoFar: number (f64, may exceed 2^32)
    // progress.trianglesSoFar: number (f64)
    updateProgressBar(progress.chunksDone / progress.chunksTotal);
    updateStats(`${progress.verticesSoFar} vertices, ${progress.trianglesSoFar} triangles`);
});

// ── NEW: Check shared atlas ──
iter.hasSharedAtlas;       // true
iter.sharedAtlas();        // TextureAtlasWrapper (same as `atlas`)

// Iterate chunks (same as before)
while (iter.advance()) {
    const chunk = iter.current();
    // chunk.opaquePositions(), chunk.opaqueIndices(), etc.
    // All chunks share the same atlas — upload atlas texture ONCE
}

// ── NEW: Export as NUCM v2 with shared atlas ──
const nucmBytes = iter.toNucm();  // Uint8Array (v2 format with shared atlas)
```

### 2.2 Python

```python
from nucleation import Schematic, ResourcePack, MeshConfig

schematic = Schematic.from_bytes(schem_bytes)
pack = ResourcePack.from_bytes(pack_bytes)
config = MeshConfig()

# ── NEW: Build global atlas ──
atlas = schematic.build_global_atlas(pack, config)
# atlas.width         → int
# atlas.height        → int
# atlas.rgba_data()   → bytes (RGBA pixels)
# atlas.region_count  → int

# ── NEW: Mesh chunks with shared atlas ──
chunks = schematic.mesh_chunks_with_atlas(pack, atlas, config, chunk_size=16)
# chunks is list[MeshResult], each chunk references the shared atlas
```

### 2.3 FFI (C/C++)

```c
#include "nucleation.h"

// Build global atlas
FFITextureAtlas* atlas = schematic_build_global_atlas(schematic, pack, config);
uint32_t w = textureatlas_width(atlas);
uint32_t h = textureatlas_height(atlas);
ByteArray pixels = textureatlas_rgba_data(atlas);  // RGBA, w * h * 4 bytes

// Mesh with atlas
FFIChunkMeshResult* result = schematic_mesh_chunks_with_atlas(
    schematic, pack, config, 16, atlas);

// Or with progress callback:
void my_progress(int phase, uint32_t done, uint32_t total,
                 uint64_t verts, uint64_t tris, void* user_data) {
    // phase: 0=BuildingAtlas, 1=MeshingChunks, 2=Complete
    printf("[%d] %u/%u chunks, %llu verts\n", phase, done, total, verts);
}

FFIChunkMeshResult* result = schematic_mesh_chunks_with_atlas_progress(
    schematic, pack, config, 16, atlas, my_progress, NULL);

// Export NUCM v2
ByteArray nucm = chunkmeshresult_nucm_data_with_atlas(result, atlas);

// Cleanup
textureatlas_free(atlas);
```

---

## 3. NUCM v2 Format Changes

### Header

| Version | Layout |
|---------|--------|
| v1 | `magic(4) + version=1(4) + chunk_count(4)` = 12 bytes |
| v2 | `magic(4) + version=2(4) + flags(4) + chunk_count(4)` = 16 bytes |

**Flags** (u32 bitfield):
- Bit 0 (`0x01`): `HAS_SHARED_ATLAS` — shared atlas follows the header
- Bits 1-31: reserved (must be 0)

### Shared Atlas (when `HAS_SHARED_ATLAS` is set)

Immediately after the 16-byte header, before any chunk data:

```
width: u32
height: u32
raw_pixel_len: u32        (uncompressed RGBA size = width * height * 4)
compressed_pixel_len: u32
pixels: [u8; compressed_pixel_len]  (DEFLATE-compressed RGBA)
region_count: u32
per region:
    name_len: u32
    name: [u8; name_len]   (UTF-8)
    u_min: f32, v_min: f32, u_max: f32, v_max: f32
```

### Per-Chunk Changes

| v1 chunk | v2 chunk |
|----------|----------|
| bounds, coord, lod, **atlas**, anim, 3 layers | bounds, coord, lod, **atlas_mode(u8)**, [atlas if mode=1], anim, 3 layers |

**`atlas_mode`** (u8, v2 only):
- `0` = uses shared atlas from header (no atlas data follows)
- `1` = has own per-chunk atlas (same format as v1)

v1 chunks do NOT have `atlas_mode` — they always have a per-chunk atlas after `lod_level`.

### JavaScript Parser Update

The `nucm-parser.js` in `examples/nucm-viewer/` already handles v1 and v2:

```javascript
import { parseNUCM } from './nucm-parser.js';

const { chunks, totalChunks, version, sharedAtlas } = parseNUCM(buffer);

// version: 1 or 2
// sharedAtlas: { width, height, pixels, regions } or null
// chunks[i].atlas: always populated (shared or per-chunk, transparent to consumer)
```

**Key point for the renderer**: `chunk.atlas` is always populated regardless of version. When `atlas_mode == 0`, the parser assigns the shared atlas to the chunk. The renderer does NOT need to distinguish between shared and per-chunk atlases at render time.

---

## 4. Rendering Recommendations

### Texture Management with Shared Atlas

```
if (sharedAtlas) {
    // Upload ONE texture for the entire scene
    const atlasTexture = createTexture(sharedAtlas.width, sharedAtlas.height, sharedAtlas.pixels);

    for (const chunk of chunks) {
        // All chunks use the same texture — no texture switches
        drawChunk(chunk, atlasTexture);
    }
} else {
    // Fallback: v1 file or v2 without shared atlas
    for (const chunk of chunks) {
        const atlasTexture = createTexture(chunk.atlas.width, chunk.atlas.height, chunk.atlas.pixels);
        drawChunk(chunk, atlasTexture);
    }
}
```

### Progress Bar Integration

```javascript
let lastUpdate = 0;

iter.setProgressCallback((progress) => {
    const now = performance.now();
    if (now - lastUpdate < 100) return;  // throttle to 10 fps
    lastUpdate = now;

    const pct = progress.chunksTotal > 0
        ? (progress.chunksDone / progress.chunksTotal * 100).toFixed(1)
        : 0;

    statusEl.textContent = `${progress.phase}: ${pct}% (${progress.chunksDone}/${progress.chunksTotal})`;
    statsEl.textContent = `${(progress.verticesSoFar / 1e6).toFixed(1)}M vertices`;
    progressBar.style.width = `${pct}%`;
});
```

### Draw Call Batching

With a shared atlas, all chunks share the same texture binding. This enables:

1. **Single texture bind** per frame instead of per-chunk
2. **Instanced rendering** if chunks share the same mesh layout
3. **Render state sorting** only by transparency layer (opaque → cutout → transparent), not by atlas

---

## 5. Migration Checklist

For the schematic-renderer to adopt the new API:

- [ ] Replace `mesh_chunks()` / `mesh_by_chunk_size()` with `build_global_atlas()` + `mesh_chunks_with_atlas()`
- [ ] Upload shared atlas texture once, reuse for all chunks
- [ ] Wire up progress callback to GUI progress bar
- [ ] Handle NUCM v2 files on the read side (the JS parser already does this)
- [ ] Keep v1 fallback for old cached files (automatic — `parseNUCM` handles both)
- [ ] Test with large schematics (IRIS_B) to verify size/speed improvements
