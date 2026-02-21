/**
 * .nucm binary mesh cache parser
 *
 * Parses the Nucleation binary mesh cache format (v1 and v2).
 * v1: per-chunk atlas, no flags field.
 * v2: flags bitfield, optional shared atlas in header, per-chunk atlas_mode byte.
 * Uses fflate for raw deflate decompression.
 */

import { inflateSync } from 'fflate';

class Reader {
  constructor(buffer) {
    this.view = new DataView(buffer);
    this.offset = 0;
  }

  u8() {
    const v = this.view.getUint8(this.offset);
    this.offset += 1;
    return v;
  }

  u32() {
    const v = this.view.getUint32(this.offset, true);
    this.offset += 4;
    return v;
  }

  i32() {
    const v = this.view.getInt32(this.offset, true);
    this.offset += 4;
    return v;
  }

  f32() {
    const v = this.view.getFloat32(this.offset, true);
    this.offset += 4;
    return v;
  }

  bytes(len) {
    const arr = new Uint8Array(this.view.buffer, this.offset, len);
    this.offset += len;
    return arr;
  }
}

/**
 * Read a compressed field: raw_len u32 + compressed_len u32 + deflate bytes.
 * Returns decompressed Uint8Array.
 */
function readCompressedField(r) {
  const rawLen = r.u32();
  const compressedLen = r.u32();
  const compressed = r.bytes(compressedLen);
  if (rawLen === 0) return new Uint8Array(0);
  const decompressed = inflateSync(compressed);
  if (decompressed.length !== rawLen) {
    throw new Error(`Field size mismatch: expected ${rawLen}, got ${decompressed.length}`);
  }
  return decompressed;
}

/**
 * Parse a mesh layer (opaque, cutout, or transparent).
 * Returns { positions, normals, uvs, colors, indices, vertexCount, indexCount }.
 */
function readLayer(r) {
  const vertexCount = r.u32();
  const indexCount = r.u32();

  if (vertexCount === 0) {
    // Empty layer — still has a compressed field for indices
    readCompressedField(r);
    return { positions: null, normals: null, uvs: null, colors: null, indices: null, vertexCount: 0, indexCount: 0 };
  }

  // Position AABB
  const posMin = [r.f32(), r.f32(), r.f32()];
  const posMax = [r.f32(), r.f32(), r.f32()];

  // Positions: delta-encoded u16x3, deflate compressed
  const posRaw = readCompressedField(r);
  const positions = new Float32Array(vertexCount * 3);
  const prev = [0, 0, 0]; // u16 accumulators
  for (let v = 0; v < vertexCount; v++) {
    for (let i = 0; i < 3; i++) {
      const byteIdx = (v * 3 + i) * 2;
      const delta = posRaw[byteIdx] | (posRaw[byteIdx + 1] << 8);
      prev[i] = (prev[i] + delta) & 0xFFFF;
      const range = posMax[i] - posMin[i];
      positions[v * 3 + i] = range > 0
        ? posMin[i] + (prev[i] / 65535.0) * range
        : posMin[i];
    }
  }

  // Normals: i8x3, deflate compressed
  const normRaw = readCompressedField(r);
  const normals = new Float32Array(vertexCount * 3);
  for (let v = 0; v < vertexCount; v++) {
    let nx = ((normRaw[v * 3] << 24) >> 24) / 127.0;     // sign-extend u8 to i8
    let ny = ((normRaw[v * 3 + 1] << 24) >> 24) / 127.0;
    let nz = ((normRaw[v * 3 + 2] << 24) >> 24) / 127.0;
    const len = Math.sqrt(nx * nx + ny * ny + nz * nz);
    if (len > 0) { nx /= len; ny /= len; nz /= len; }
    normals[v * 3] = nx;
    normals[v * 3 + 1] = ny;
    normals[v * 3 + 2] = nz;
  }

  // UV AABB
  const uvMin = [r.f32(), r.f32()];
  const uvMax = [r.f32(), r.f32()];

  // UVs: u16x2, deflate compressed
  const uvRaw = readCompressedField(r);
  const uvs = new Float32Array(vertexCount * 2);
  for (let v = 0; v < vertexCount; v++) {
    for (let i = 0; i < 2; i++) {
      const byteIdx = (v * 2 + i) * 2;
      const q = uvRaw[byteIdx] | (uvRaw[byteIdx + 1] << 8);
      const range = uvMax[i] - uvMin[i];
      uvs[v * 2 + i] = range > 0
        ? uvMin[i] + (q / 65535.0) * range
        : uvMin[i];
    }
  }

  // Colors: u8x4, deflate compressed
  const colRaw = readCompressedField(r);
  const colors = new Float32Array(vertexCount * 4);
  for (let v = 0; v < vertexCount; v++) {
    colors[v * 4] = colRaw[v * 4] / 255.0;
    colors[v * 4 + 1] = colRaw[v * 4 + 1] / 255.0;
    colors[v * 4 + 2] = colRaw[v * 4 + 2] / 255.0;
    colors[v * 4 + 3] = colRaw[v * 4 + 3] / 255.0;
  }

  // Indices: delta-encoded u32, deflate compressed
  const idxRaw = readCompressedField(r);
  const indices = new Uint32Array(indexCount);
  let prevIdx = 0;
  for (let i = 0; i < indexCount; i++) {
    const byteIdx = i * 4;
    const delta = idxRaw[byteIdx] | (idxRaw[byteIdx + 1] << 8) |
                  (idxRaw[byteIdx + 2] << 16) | (idxRaw[byteIdx + 3] << 24);
    prevIdx = (prevIdx + delta) >>> 0; // unsigned 32-bit add
    indices[i] = prevIdx;
  }

  return { positions, normals, uvs, colors, indices, vertexCount, indexCount };
}

/**
 * Parse the atlas section of a chunk.
 */
function readAtlas(r) {
  const width = r.u32();
  const height = r.u32();

  // Compressed atlas pixels
  const rawLen = r.u32();
  const compressedLen = r.u32();
  const compressed = r.bytes(compressedLen);
  let pixels;
  if (rawLen === 0) {
    pixels = new Uint8Array(0);
  } else {
    pixels = inflateSync(compressed);
    if (pixels.length !== rawLen) {
      throw new Error(`Atlas pixel size mismatch: expected ${rawLen}, got ${pixels.length}`);
    }
  }

  // Regions
  const regionCount = r.u32();
  const regions = {};
  for (let i = 0; i < regionCount; i++) {
    const nameLen = r.u32();
    const nameBytes = r.bytes(nameLen);
    const name = new TextDecoder().decode(nameBytes);
    regions[name] = {
      uMin: r.f32(),
      vMin: r.f32(),
      uMax: r.f32(),
      vMax: r.f32(),
    };
  }

  return { width, height, pixels, regions };
}

/**
 * Parse an animated texture entry (skipped for rendering, but must be read to advance the offset).
 */
function readAnimatedTexture(r) {
  const spriteLen = r.u32();
  r.bytes(spriteLen); // sprite_sheet_png — skip

  const frameCount = r.u32();
  const frametime = r.u32();
  const interpolate = r.u8() !== 0;

  const hasFrames = r.u8();
  let frames = null;
  if (hasFrames === 1) {
    const count = r.u32();
    frames = [];
    for (let i = 0; i < count; i++) {
      frames.push(r.u32());
    }
  }

  const frameWidth = r.u32();
  const frameHeight = r.u32();
  const atlasX = r.u32();
  const atlasY = r.u32();

  return { frameCount, frametime, interpolate, frames, frameWidth, frameHeight, atlasX, atlasY };
}

/**
 * Parse a v1 chunk (always has its own atlas).
 */
function readChunkV1(r) {
  const boundsMin = [r.f32(), r.f32(), r.f32()];
  const boundsMax = [r.f32(), r.f32(), r.f32()];

  const hasCoord = r.u8();
  let chunkCoord = null;
  if (hasCoord === 1) {
    chunkCoord = [r.i32(), r.i32(), r.i32()];
  }

  const lodLevel = r.u8();
  const atlas = readAtlas(r);

  const animCount = r.u32();
  for (let i = 0; i < animCount; i++) {
    readAnimatedTexture(r);
  }

  const opaque = readLayer(r);
  const cutout = readLayer(r);
  const transparent = readLayer(r);

  return {
    bounds: { min: boundsMin, max: boundsMax },
    chunkCoord,
    lodLevel,
    atlas,
    layers: { opaque, cutout, transparent },
  };
}

/**
 * Parse a v2 chunk (has atlas_mode byte: 0 = shared atlas, 1 = own atlas).
 */
function readChunkV2(r, sharedAtlas) {
  const boundsMin = [r.f32(), r.f32(), r.f32()];
  const boundsMax = [r.f32(), r.f32(), r.f32()];

  const hasCoord = r.u8();
  let chunkCoord = null;
  if (hasCoord === 1) {
    chunkCoord = [r.i32(), r.i32(), r.i32()];
  }

  const lodLevel = r.u8();

  // atlas_mode: 0 = uses shared atlas, 1 = has own atlas
  const atlasMode = r.u8();
  let atlas;
  if (atlasMode === 0) {
    if (!sharedAtlas) {
      throw new Error('Chunk references shared atlas but none was provided in header');
    }
    atlas = sharedAtlas;
  } else {
    atlas = readAtlas(r);
  }

  const animCount = r.u32();
  for (let i = 0; i < animCount; i++) {
    readAnimatedTexture(r);
  }

  const opaque = readLayer(r);
  const cutout = readLayer(r);
  const transparent = readLayer(r);

  return {
    bounds: { min: boundsMin, max: boundsMax },
    chunkCoord,
    lodLevel,
    atlas,
    layers: { opaque, cutout, transparent },
  };
}

/**
 * Parse a .nucm ArrayBuffer into an array of chunk objects.
 * Supports both v1 and v2 formats.
 *
 * @param {ArrayBuffer} buffer - The raw .nucm file contents
 * @param {number} maxChunks - Maximum number of chunks to parse (0 = all)
 * @param {function} onProgress - Optional callback(chunkIndex, totalChunks)
 * @returns {{ chunks: Array, totalChunks: number, version: number, sharedAtlas: object|null }}
 */
export function parseNUCM(buffer, maxChunks = 0, onProgress = null) {
  const r = new Reader(buffer);

  // Header: magic(4)
  const magic = r.bytes(4);
  if (magic[0] !== 0x4E || magic[1] !== 0x55 || magic[2] !== 0x43 || magic[3] !== 0x4D) {
    throw new Error('Invalid magic bytes (expected NUCM)');
  }

  const version = r.u32();
  if (version !== 1 && version !== 2) {
    throw new Error(`Unsupported NUCM version: ${version} (expected 1 or 2)`);
  }

  // v2 has a flags field between version and chunk_count; v1 does not
  let flags = 0;
  if (version >= 2) {
    flags = r.u32();
  }

  const totalChunks = r.u32();
  const limit = (maxChunks > 0 && maxChunks < totalChunks) ? maxChunks : totalChunks;

  // v2 with FLAG_HAS_SHARED_ATLAS (bit 0): read shared atlas from header
  const FLAG_HAS_SHARED_ATLAS = 1;
  let sharedAtlas = null;
  if (version >= 2 && (flags & FLAG_HAS_SHARED_ATLAS)) {
    sharedAtlas = readAtlas(r);
  }

  const chunks = [];
  for (let c = 0; c < totalChunks; c++) {
    let chunk;
    if (version === 1) {
      chunk = readChunkV1(r);
    } else {
      chunk = readChunkV2(r, sharedAtlas);
    }

    if (c < limit) {
      chunk.index = c;
      chunks.push(chunk);
    }

    if (onProgress) onProgress(c + 1, totalChunks);

    if (c >= limit - 1 && maxChunks > 0) {
      break;
    }
  }

  return { chunks, totalChunks, version, sharedAtlas };
}
