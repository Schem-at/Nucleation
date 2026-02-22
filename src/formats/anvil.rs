use crate::block_entity::BlockEntity;
use crate::entity::Entity;
use crate::BlockState;
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use quartz_nbt::io::Flavor;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use std::error::Error;
use std::io::{Cursor, Read, Write};

// ─── Data Structures ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionType {
    Gzip = 1,
    Zlib = 2,
    Uncompressed = 3,
    Lz4 = 4,
}

impl CompressionType {
    pub fn from_byte(b: u8) -> Result<Self, Box<dyn Error>> {
        match b {
            1 => Ok(CompressionType::Gzip),
            2 => Ok(CompressionType::Zlib),
            3 => Ok(CompressionType::Uncompressed),
            4 => Err("LZ4 compression (type 4) is not supported".into()),
            _ => Err(format!("Unknown compression type: {}", b).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub y: i8,
    pub palette: Vec<BlockState>,
    /// 4096 entries (16x16x16), each is an index into `palette`.
    pub block_states: Vec<u16>,
}

#[derive(Debug, Clone)]
pub struct ChunkData {
    pub x: i32,
    pub z: i32,
    pub data_version: i32,
    pub status: String,
    pub sections: Vec<ChunkSection>,
    pub block_entities: Vec<BlockEntity>,
    pub entities: Vec<Entity>,
    /// Minimum section Y (e.g. -4 for overworld 1.18+)
    pub y_pos: i32,
}

#[derive(Debug, Clone)]
pub struct McaFile {
    pub chunks: Vec<Option<ChunkData>>,
    pub region_x: i32,
    pub region_z: i32,
}

// ─── Detection ──────────────────────────────────────────────────────────────

/// Check if data looks like an MCA region file.
/// Must be at least 8192 bytes (two 4KiB tables) and have at least one valid location entry.
pub fn is_mca(data: &[u8]) -> bool {
    if data.len() < 8192 {
        return false;
    }
    // Reject zip files (PK\x03\x04 magic) — these are handled by WorldZipFormat
    if data[0] == 0x50 && data[1] == 0x4B && data[2] == 0x03 && data[3] == 0x04 {
        return false;
    }
    // Check for at least one valid location entry with offset >= 2
    for i in 0..1024 {
        let offset = i * 4;
        let loc_offset = ((data[offset] as u32) << 16)
            | ((data[offset + 1] as u32) << 8)
            | (data[offset + 2] as u32);
        let sector_count = data[offset + 3];
        if loc_offset >= 2 && sector_count > 0 {
            return true;
        }
    }
    false
}

// ─── Read Path ──────────────────────────────────────────────────────────────

impl McaFile {
    /// Parse an MCA region file from raw bytes.
    pub fn from_bytes(data: &[u8], region_x: i32, region_z: i32) -> Result<Self, Box<dyn Error>> {
        if data.len() < 8192 {
            return Err("MCA file too small (< 8192 bytes)".into());
        }

        let mut chunks = Vec::with_capacity(1024);
        for _ in 0..1024 {
            chunks.push(None);
        }

        // Parse location table (first 4096 bytes)
        for i in 0..1024u32 {
            let offset = (i as usize) * 4;
            let loc_offset = ((data[offset] as u32) << 16)
                | ((data[offset + 1] as u32) << 8)
                | (data[offset + 2] as u32);
            let sector_count = data[offset + 3] as u32;

            if loc_offset < 2 || sector_count == 0 {
                continue;
            }

            let byte_offset = (loc_offset as usize) * 4096;
            if byte_offset + 5 > data.len() {
                continue;
            }

            // Read chunk header: 4-byte length + 1-byte compression type
            let chunk_len = ((data[byte_offset] as u32) << 24)
                | ((data[byte_offset + 1] as u32) << 16)
                | ((data[byte_offset + 2] as u32) << 8)
                | (data[byte_offset + 3] as u32);

            if chunk_len <= 1 {
                continue;
            }

            let compression_byte = data[byte_offset + 4];
            let compression = CompressionType::from_byte(compression_byte)?;

            let compressed_start = byte_offset + 5;
            let compressed_len = (chunk_len as usize) - 1;
            if compressed_start + compressed_len > data.len() {
                continue;
            }

            let compressed_data = &data[compressed_start..compressed_start + compressed_len];

            // Decompress
            let decompressed = decompress_chunk(compressed_data, compression)?;

            // Parse NBT
            let (nbt, _) =
                quartz_nbt::io::read_nbt(&mut Cursor::new(&decompressed), Flavor::Uncompressed)?;

            // Parse chunk data
            let chunk_x = (region_x * 32) + ((i % 32) as i32);
            let chunk_z = (region_z * 32) + ((i / 32) as i32);

            match parse_chunk_nbt(&nbt, chunk_x, chunk_z) {
                Ok(chunk) => {
                    chunks[i as usize] = Some(chunk);
                }
                Err(_e) => {
                    // Skip malformed chunks
                    continue;
                }
            }
        }

        Ok(McaFile {
            chunks,
            region_x,
            region_z,
        })
    }

    /// Parse an MCA file without known region coordinates (inferred from chunk data).
    pub fn from_bytes_auto(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        // First pass: find any chunk to determine region coordinates
        let mut region_x = 0i32;
        let mut region_z = 0i32;
        let mut found = false;

        if data.len() < 8192 {
            return Err("MCA file too small (< 8192 bytes)".into());
        }

        for i in 0..1024u32 {
            let offset = (i as usize) * 4;
            let loc_offset = ((data[offset] as u32) << 16)
                | ((data[offset + 1] as u32) << 8)
                | (data[offset + 2] as u32);
            let sector_count = data[offset + 3] as u32;

            if loc_offset < 2 || sector_count == 0 {
                continue;
            }

            let byte_offset = (loc_offset as usize) * 4096;
            if byte_offset + 5 > data.len() {
                continue;
            }

            let chunk_len = ((data[byte_offset] as u32) << 24)
                | ((data[byte_offset + 1] as u32) << 16)
                | ((data[byte_offset + 2] as u32) << 8)
                | (data[byte_offset + 3] as u32);

            if chunk_len <= 1 {
                continue;
            }

            let compression_byte = data[byte_offset + 4];
            let compression = match CompressionType::from_byte(compression_byte) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let compressed_start = byte_offset + 5;
            let compressed_len = (chunk_len as usize) - 1;
            if compressed_start + compressed_len > data.len() {
                continue;
            }

            let compressed_data = &data[compressed_start..compressed_start + compressed_len];
            let decompressed = match decompress_chunk(compressed_data, compression) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let (nbt, _) = match quartz_nbt::io::read_nbt(
                &mut Cursor::new(&decompressed),
                Flavor::Uncompressed,
            ) {
                Ok(r) => r,
                Err(_) => continue,
            };

            // Try to get xPos/zPos from chunk NBT
            if let (Ok(cx), Ok(cz)) = (nbt.get::<_, i32>("xPos"), nbt.get::<_, i32>("zPos")) {
                region_x = floor_div(cx, 32);
                region_z = floor_div(cz, 32);
                found = true;
                break;
            }
        }

        if !found {
            // Default to 0,0 if we can't determine
            region_x = 0;
            region_z = 0;
        }

        Self::from_bytes(data, region_x, region_z)
    }
}

fn decompress_chunk(data: &[u8], compression: CompressionType) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut decompressed = Vec::new();
    match compression {
        CompressionType::Zlib => {
            let mut decoder = ZlibDecoder::new(data);
            decoder.read_to_end(&mut decompressed)?;
        }
        CompressionType::Gzip => {
            let mut decoder = GzDecoder::new(data);
            decoder.read_to_end(&mut decompressed)?;
        }
        CompressionType::Uncompressed => {
            decompressed = data.to_vec();
        }
        CompressionType::Lz4 => {
            return Err("LZ4 compression is not supported".into());
        }
    }
    Ok(decompressed)
}

fn parse_chunk_nbt(
    nbt: &NbtCompound,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<ChunkData, Box<dyn Error>> {
    let data_version = nbt.get::<_, i32>("DataVersion").unwrap_or(3700);

    // Status can be at root level or under "Status"
    let status = nbt
        .get::<_, &str>("Status")
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "minecraft:full".to_string());

    let x_pos = nbt.get::<_, i32>("xPos").unwrap_or(chunk_x);
    let z_pos = nbt.get::<_, i32>("zPos").unwrap_or(chunk_z);
    let y_pos = nbt.get::<_, i32>("yPos").unwrap_or(-4);

    // Parse sections
    let mut sections = Vec::new();
    if let Ok(section_list) = nbt.get::<_, &NbtList>("sections") {
        for section_tag in section_list.iter() {
            if let NbtTag::Compound(section_nbt) = section_tag {
                if let Ok(section) = parse_section(section_nbt) {
                    sections.push(section);
                }
            }
        }
    }

    // Parse block entities
    let mut block_entities = Vec::new();
    if let Ok(be_list) = nbt.get::<_, &NbtList>("block_entities") {
        for be_tag in be_list.iter() {
            if let NbtTag::Compound(be_nbt) = be_tag {
                if let Ok(be) = parse_block_entity(be_nbt) {
                    block_entities.push(be);
                }
            }
        }
    }

    // Parse entities (1.17+ stores in separate files, but some chunks still have them)
    let mut entities = Vec::new();
    if let Ok(entity_list) = nbt.get::<_, &NbtList>("Entities") {
        for entity_tag in entity_list.iter() {
            if let NbtTag::Compound(entity_nbt) = entity_tag {
                if let Ok(entity) = Entity::from_nbt(entity_nbt) {
                    entities.push(entity);
                }
            }
        }
    }

    Ok(ChunkData {
        x: x_pos,
        z: z_pos,
        data_version,
        status,
        sections,
        block_entities,
        entities,
        y_pos,
    })
}

fn parse_section(section_nbt: &NbtCompound) -> Result<ChunkSection, Box<dyn Error>> {
    let y = section_nbt.get::<_, i8>("Y")?;

    // Parse block_states compound
    let block_states_compound = match section_nbt.get::<_, &NbtCompound>("block_states") {
        Ok(bs) => bs,
        Err(_) => {
            // No block_states = all air section
            return Ok(ChunkSection {
                y,
                palette: vec![BlockState::new("minecraft:air".to_string())],
                block_states: vec![0; 4096],
            });
        }
    };

    // Parse palette
    let palette = match block_states_compound.get::<_, &NbtList>("palette") {
        Ok(palette_list) => {
            let mut palette = Vec::new();
            for tag in palette_list.iter() {
                if let NbtTag::Compound(compound) = tag {
                    palette.push(BlockState::from_nbt(compound)?);
                }
            }
            palette
        }
        Err(_) => {
            vec![BlockState::new("minecraft:air".to_string())]
        }
    };

    // Parse packed block state data
    let block_states = if palette.len() <= 1 {
        // Single-entry palette, all blocks are index 0 (no data array needed)
        vec![0u16; 4096]
    } else {
        match block_states_compound.get::<_, &[i64]>("data") {
            Ok(packed_data) => unpack_block_states(packed_data, palette.len()),
            Err(_) => vec![0u16; 4096],
        }
    };

    Ok(ChunkSection {
        y,
        palette,
        block_states,
    })
}

/// Unpack block states from Minecraft's chunk format.
/// CRITICAL: Entries do NOT span across long boundaries (unlike Litematic).
/// Each i64 holds floor(64/bits_per_entry) entries, minimum 4 bits per entry.
pub fn unpack_block_states(packed: &[i64], palette_size: usize) -> Vec<u16> {
    let bits_per_entry = std::cmp::max(
        (palette_size as f64).log2().ceil() as u32,
        4, // Minecraft minimum is 4 bits per entry for chunk sections
    );

    let entries_per_long = 64 / bits_per_entry;
    let mask = (1u64 << bits_per_entry) - 1;

    let mut result = Vec::with_capacity(4096);

    for &long_val in packed {
        let long_unsigned = long_val as u64;
        for j in 0..entries_per_long {
            if result.len() >= 4096 {
                break;
            }
            let index = (long_unsigned >> (j * bits_per_entry)) & mask;
            result.push(index as u16);
        }
    }

    // Pad with 0 if we somehow have fewer than 4096
    result.resize(4096, 0);
    result
}

/// Pack block states into Minecraft's chunk format.
/// Entries do NOT span across long boundaries.
pub fn pack_block_states(indices: &[u16], palette_size: usize) -> Vec<i64> {
    if palette_size <= 1 {
        return Vec::new();
    }

    let bits_per_entry = std::cmp::max((palette_size as f64).log2().ceil() as u32, 4);

    let entries_per_long = 64 / bits_per_entry;
    let num_longs = (4096 + entries_per_long as usize - 1) / entries_per_long as usize;
    let mask = (1u64 << bits_per_entry) - 1;

    let mut packed = vec![0i64; num_longs];

    for (i, &index) in indices.iter().enumerate().take(4096) {
        let long_index = i / entries_per_long as usize;
        let bit_offset = (i % entries_per_long as usize) as u32 * bits_per_entry;
        let value = (index as u64) & mask;
        packed[long_index] |= (value << bit_offset) as i64;
    }

    packed
}

fn parse_block_entity(nbt: &NbtCompound) -> Result<BlockEntity, Box<dyn Error>> {
    let id = nbt
        .get::<_, &str>("id")
        .map(|s| s.to_string())
        .unwrap_or_default();

    let x = nbt.get::<_, i32>("x").unwrap_or(0);
    let y = nbt.get::<_, i32>("y").unwrap_or(0);
    let z = nbt.get::<_, i32>("z").unwrap_or(0);

    let mut block_entity = BlockEntity::new(id, (x, y, z));

    // Copy all NBT fields except x, y, z, id (those are handled separately)
    for (key, value) in nbt.inner() {
        match key.as_str() {
            "x" | "y" | "z" | "id" => continue,
            _ => {
                block_entity
                    .nbt
                    .insert(key.clone(), crate::utils::NbtValue::from_quartz_nbt(value));
            }
        }
    }

    Ok(block_entity)
}

// ─── Entity Region Files (1.17+) ────────────────────────────────────────────

/// Data from a single chunk in an entity region file.
/// Since 1.17+, entities are stored in separate `entities/r.x.z.mca` files.
#[derive(Debug, Clone)]
pub struct EntityChunkData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub entities: Vec<Entity>,
}

/// Parse entity chunks from an MCA file (entities/r.x.z.mca format).
/// Uses the same binary MCA format but chunk NBT structure is:
/// { DataVersion: int, Position: int[2], Entities: list<compound> }
pub fn parse_entity_mca(
    data: &[u8],
    region_x: i32,
    region_z: i32,
) -> Result<Vec<EntityChunkData>, Box<dyn Error>> {
    if data.len() < 8192 {
        return Err("Entity MCA file too small (< 8192 bytes)".into());
    }

    let mut result = Vec::new();

    for i in 0..1024u32 {
        let offset = (i as usize) * 4;
        let loc_offset = ((data[offset] as u32) << 16)
            | ((data[offset + 1] as u32) << 8)
            | (data[offset + 2] as u32);
        let sector_count = data[offset + 3] as u32;

        if loc_offset < 2 || sector_count == 0 {
            continue;
        }

        let byte_offset = (loc_offset as usize) * 4096;
        if byte_offset + 5 > data.len() {
            continue;
        }

        let chunk_len = ((data[byte_offset] as u32) << 24)
            | ((data[byte_offset + 1] as u32) << 16)
            | ((data[byte_offset + 2] as u32) << 8)
            | (data[byte_offset + 3] as u32);

        if chunk_len <= 1 {
            continue;
        }

        let compression_byte = data[byte_offset + 4];
        let compression = match CompressionType::from_byte(compression_byte) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let compressed_start = byte_offset + 5;
        let compressed_len = (chunk_len as usize) - 1;
        if compressed_start + compressed_len > data.len() {
            continue;
        }

        let compressed_data = &data[compressed_start..compressed_start + compressed_len];
        let decompressed = match decompress_chunk(compressed_data, compression) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let (nbt, _) =
            match quartz_nbt::io::read_nbt(&mut Cursor::new(&decompressed), Flavor::Uncompressed) {
                Ok(r) => r,
                Err(_) => continue,
            };

        // Entity chunk NBT: Position is int array [chunkX, chunkZ]
        let (chunk_x, chunk_z) = if let Ok(pos) = nbt.get::<_, &[i32]>("Position") {
            if pos.len() >= 2 {
                (pos[0], pos[1])
            } else {
                let cx = (region_x * 32) + ((i % 32) as i32);
                let cz = (region_z * 32) + ((i / 32) as i32);
                (cx, cz)
            }
        } else {
            let cx = (region_x * 32) + ((i % 32) as i32);
            let cz = (region_z * 32) + ((i / 32) as i32);
            (cx, cz)
        };

        let mut entities = Vec::new();
        if let Ok(entity_list) = nbt.get::<_, &NbtList>("Entities") {
            for entity_tag in entity_list.iter() {
                if let NbtTag::Compound(entity_nbt) = entity_tag {
                    if let Ok(entity) = Entity::from_nbt(entity_nbt) {
                        entities.push(entity);
                    }
                }
            }
        }

        if !entities.is_empty() {
            result.push(EntityChunkData {
                chunk_x,
                chunk_z,
                entities,
            });
        }
    }

    Ok(result)
}

/// Build entity chunk NBT for writing to entity region files.
fn build_entity_chunk_nbt(chunk: &EntityChunkData, data_version: i32) -> NbtCompound {
    let mut root = NbtCompound::new();

    root.insert("DataVersion", NbtTag::Int(data_version));
    root.insert(
        "Position",
        NbtTag::IntArray(vec![chunk.chunk_x, chunk.chunk_z]),
    );

    let entity_tags: Vec<NbtTag> = chunk.entities.iter().map(|e| e.to_nbt()).collect();
    root.insert("Entities", NbtTag::List(NbtList::from(entity_tags)));

    root
}

/// Write entity chunks to an MCA file (entities/r.x.z.mca format).
pub fn write_entity_mca(
    chunks: &[EntityChunkData],
    _region_x: i32,
    _region_z: i32,
    data_version: i32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut chunk_data_parts: Vec<(u32, Vec<u8>)> = Vec::new();

    for chunk in chunks {
        let local_x = floor_mod(chunk.chunk_x, 32) as u32;
        let local_z = floor_mod(chunk.chunk_z, 32) as u32;
        let index = local_x + local_z * 32;

        let nbt = build_entity_chunk_nbt(chunk, data_version);
        let mut nbt_bytes = Vec::new();
        quartz_nbt::io::write_nbt(&mut nbt_bytes, None, &nbt, Flavor::Uncompressed)?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&nbt_bytes)?;
        let compressed = encoder.finish()?;

        chunk_data_parts.push((index, compressed));
    }

    // Sort by index for deterministic output
    chunk_data_parts.sort_by_key(|(idx, _)| *idx);

    // Build MCA file (same format as block region files)
    let mut location_table = vec![0u8; 4096];
    let timestamp_table = vec![0u8; 4096];
    let mut data_sectors = Vec::new();

    let mut current_sector: u32 = 2;

    for (index, compressed) in &chunk_data_parts {
        let chunk_payload_len = compressed.len() as u32 + 1;
        let total_len = 4 + chunk_payload_len;
        let sector_count = ((total_len as usize) + 4095) / 4096;

        let loc_offset = *index as usize * 4;
        location_table[loc_offset] = ((current_sector >> 16) & 0xFF) as u8;
        location_table[loc_offset + 1] = ((current_sector >> 8) & 0xFF) as u8;
        location_table[loc_offset + 2] = (current_sector & 0xFF) as u8;
        location_table[loc_offset + 3] = sector_count as u8;

        let mut chunk_sector = Vec::new();
        chunk_sector.push(((chunk_payload_len >> 24) & 0xFF) as u8);
        chunk_sector.push(((chunk_payload_len >> 16) & 0xFF) as u8);
        chunk_sector.push(((chunk_payload_len >> 8) & 0xFF) as u8);
        chunk_sector.push((chunk_payload_len & 0xFF) as u8);
        chunk_sector.push(2); // zlib
        chunk_sector.extend_from_slice(compressed);

        let padded_len = sector_count * 4096;
        chunk_sector.resize(padded_len, 0);

        data_sectors.extend_from_slice(&chunk_sector);
        current_sector += sector_count as u32;
    }

    let mut result = Vec::new();
    result.extend_from_slice(&location_table);
    result.extend_from_slice(&timestamp_table);
    result.extend_from_slice(&data_sectors);

    Ok(result)
}

// ─── Write Path ─────────────────────────────────────────────────────────────

impl McaFile {
    /// Write the MCA file to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut chunk_data_parts: Vec<(u32, Vec<u8>)> = Vec::new(); // (index, compressed_nbt)

        for (i, chunk_opt) in self.chunks.iter().enumerate() {
            if let Some(chunk) = chunk_opt {
                let nbt = build_chunk_nbt(chunk);
                let mut nbt_bytes = Vec::new();
                quartz_nbt::io::write_nbt(&mut nbt_bytes, None, &nbt, Flavor::Uncompressed)?;

                // Compress with zlib
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&nbt_bytes)?;
                let compressed = encoder.finish()?;

                chunk_data_parts.push((i as u32, compressed));
            }
        }

        // Build the file: 8KiB header + chunk sectors
        let mut location_table = vec![0u8; 4096];
        let timestamp_table = vec![0u8; 4096];
        let mut data_sectors = Vec::new();

        let mut current_sector: u32 = 2; // First two sectors are headers

        for (index, compressed) in &chunk_data_parts {
            // Chunk header: 4-byte length (including compression byte) + 1-byte compression type
            let chunk_payload_len = compressed.len() as u32 + 1; // +1 for compression byte
            let total_len = 4 + chunk_payload_len; // 4-byte length prefix + payload

            let sector_count = ((total_len as usize) + 4095) / 4096;

            // Write location table entry
            let loc_offset = *index as usize * 4;
            location_table[loc_offset] = ((current_sector >> 16) & 0xFF) as u8;
            location_table[loc_offset + 1] = ((current_sector >> 8) & 0xFF) as u8;
            location_table[loc_offset + 2] = (current_sector & 0xFF) as u8;
            location_table[loc_offset + 3] = sector_count as u8;

            // Build chunk sector data
            let mut chunk_sector = Vec::new();
            // 4-byte length (big-endian)
            chunk_sector.push(((chunk_payload_len >> 24) & 0xFF) as u8);
            chunk_sector.push(((chunk_payload_len >> 16) & 0xFF) as u8);
            chunk_sector.push(((chunk_payload_len >> 8) & 0xFF) as u8);
            chunk_sector.push((chunk_payload_len & 0xFF) as u8);
            // Compression type (2 = zlib)
            chunk_sector.push(2);
            // Compressed data
            chunk_sector.extend_from_slice(compressed);

            // Pad to 4KiB boundary
            let padded_len = sector_count * 4096;
            chunk_sector.resize(padded_len, 0);

            data_sectors.extend_from_slice(&chunk_sector);
            current_sector += sector_count as u32;
        }

        // Assemble the file
        let mut result = Vec::new();
        result.extend_from_slice(&location_table);
        result.extend_from_slice(&timestamp_table);
        result.extend_from_slice(&data_sectors);

        Ok(result)
    }
}

fn build_chunk_nbt(chunk: &ChunkData) -> NbtCompound {
    let mut root = NbtCompound::new();

    root.insert("DataVersion", NbtTag::Int(chunk.data_version));
    root.insert("xPos", NbtTag::Int(chunk.x));
    root.insert("yPos", NbtTag::Int(chunk.y_pos));
    root.insert("zPos", NbtTag::Int(chunk.z));
    root.insert("Status", NbtTag::String(chunk.status.clone()));

    // Build sections
    let mut section_list = Vec::new();
    for section in &chunk.sections {
        section_list.push(NbtTag::Compound(build_section_nbt(section)));
    }
    root.insert("sections", NbtTag::List(NbtList::from(section_list)));

    // Block entities
    let mut be_list = Vec::new();
    for be in &chunk.block_entities {
        let mut be_nbt = be.to_nbt();
        be_nbt.insert("x", NbtTag::Int(be.position.0));
        be_nbt.insert("y", NbtTag::Int(be.position.1));
        be_nbt.insert("z", NbtTag::Int(be.position.2));
        be_nbt.insert("id", NbtTag::String(be.id.clone()));
        be_list.push(NbtTag::Compound(be_nbt));
    }
    root.insert("block_entities", NbtTag::List(NbtList::from(be_list)));

    // Heightmaps (required by MC 1.18+ for status "minecraft:full")
    root.insert("Heightmaps", NbtTag::Compound(compute_heightmaps(chunk)));

    // isLightOn = 0: tell Minecraft to recalculate lighting on load
    root.insert("isLightOn", NbtTag::Byte(0));

    root
}

fn compute_heightmaps(chunk: &ChunkData) -> NbtCompound {
    let world_min_y = chunk.y_pos * 16;
    let mut motion_blocking = vec![0i32; 256];
    let mut world_surface = vec![0i32; 256];

    // Sort sections by Y descending so we scan from top down
    let mut sorted_sections: Vec<&ChunkSection> = chunk.sections.iter().collect();
    sorted_sections.sort_by(|a, b| b.y.cmp(&a.y));

    for lz in 0..16usize {
        for lx in 0..16usize {
            let col_idx = lz * 16 + lx;

            'outer: for section in &sorted_sections {
                let section_base_y = (section.y as i32) * 16;
                for ly in (0..16i32).rev() {
                    let block_idx = (ly * 256 + lz as i32 * 16 + lx as i32) as usize;
                    let palette_idx = section.block_states[block_idx] as usize;
                    if palette_idx >= section.palette.len() {
                        continue;
                    }
                    let name = &section.palette[palette_idx].name;
                    if !matches!(
                        name.as_str(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                    ) {
                        let world_y = section_base_y + ly;
                        let hm_value = world_y - world_min_y + 1;
                        motion_blocking[col_idx] = hm_value;
                        world_surface[col_idx] = hm_value;
                        break 'outer;
                    }
                }
            }
        }
    }

    let mut heightmaps = NbtCompound::new();
    heightmaps.insert(
        "MOTION_BLOCKING",
        NbtTag::LongArray(pack_heightmap(&motion_blocking)),
    );
    heightmaps.insert(
        "WORLD_SURFACE",
        NbtTag::LongArray(pack_heightmap(&world_surface)),
    );
    heightmaps
}

/// Pack 256 heightmap values into a long array (9 bits per entry, entries don't span longs).
fn pack_heightmap(values: &[i32]) -> Vec<i64> {
    let bits_per_entry: usize = 9;
    let entries_per_long = 64 / bits_per_entry; // 7
    let num_longs = (256 + entries_per_long - 1) / entries_per_long; // 37
    let mask = (1u64 << bits_per_entry) - 1;

    let mut packed = vec![0i64; num_longs];

    for (i, &value) in values.iter().enumerate().take(256) {
        let long_index = i / entries_per_long;
        let bit_offset = (i % entries_per_long) * bits_per_entry;
        let v = (value as u64) & mask;
        packed[long_index] |= (v << bit_offset) as i64;
    }

    packed
}

fn build_section_nbt(section: &ChunkSection) -> NbtCompound {
    let mut section_nbt = NbtCompound::new();
    section_nbt.insert("Y", NbtTag::Byte(section.y as i8));

    // Block states
    let mut block_states_compound = NbtCompound::new();

    // Palette
    let palette_nbt: Vec<NbtTag> = section.palette.iter().map(|bs| bs.to_nbt()).collect();
    block_states_compound.insert("palette", NbtTag::List(NbtList::from(palette_nbt)));

    // Packed data (only if palette has more than 1 entry)
    if section.palette.len() > 1 {
        let packed = pack_block_states(&section.block_states, section.palette.len());
        block_states_compound.insert("data", NbtTag::LongArray(packed));
    }

    section_nbt.insert("block_states", NbtTag::Compound(block_states_compound));

    // Add empty biomes section (required for MC to accept chunks)
    let mut biomes = NbtCompound::new();
    let biome_palette = vec![NbtTag::String("minecraft:plains".to_string())];
    biomes.insert("palette", NbtTag::List(NbtList::from(biome_palette)));
    section_nbt.insert("biomes", NbtTag::Compound(biomes));

    section_nbt
}

// ─── Utility ────────────────────────────────────────────────────────────────

/// Floor division that handles negative numbers correctly.
/// Rust's integer division truncates toward zero, but we need toward negative infinity.
pub fn floor_div(a: i32, b: i32) -> i32 {
    let d = a / b;
    let r = a % b;
    if (r != 0) && ((r ^ b) < 0) {
        d - 1
    } else {
        d
    }
}

/// Floor modulo that handles negative numbers correctly.
pub fn floor_mod(a: i32, b: i32) -> i32 {
    ((a % b) + b) % b
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use smol_str::SmolStr;

    // ─── Floor division / modulo ────────────────────────────────────────────

    #[test]
    fn test_floor_div() {
        assert_eq!(floor_div(7, 32), 0);
        assert_eq!(floor_div(32, 32), 1);
        assert_eq!(floor_div(-1, 32), -1);
        assert_eq!(floor_div(-32, 32), -1);
        assert_eq!(floor_div(-33, 32), -2);
        assert_eq!(floor_div(0, 32), 0);
    }

    #[test]
    fn test_floor_mod() {
        assert_eq!(floor_mod(0, 32), 0);
        assert_eq!(floor_mod(1, 32), 1);
        assert_eq!(floor_mod(31, 32), 31);
        assert_eq!(floor_mod(32, 32), 0);
        assert_eq!(floor_mod(-1, 32), 31);
        assert_eq!(floor_mod(-32, 32), 0);
    }

    // ─── Chunk index formula: spec says (x & 31) + (z & 31) * 32 ───────────

    #[test]
    fn test_chunk_index_formula() {
        // Spec: headerIndex = (x & 31) + (z & 31) * 32
        // Chunk (0, 0) in region → index 0
        assert_eq!((0u32 & 31) + (0u32 & 31) * 32, 0);
        // Chunk (1, 0) → index 1
        assert_eq!((1u32 & 31) + (0u32 & 31) * 32, 1);
        // Chunk (0, 1) → index 32
        assert_eq!((0u32 & 31) + (1u32 & 31) * 32, 32);
        // Chunk (31, 31) → index 1023
        assert_eq!((31u32 & 31) + (31u32 & 31) * 32, 1023);
        // Chunk (5, 10) → index 325
        assert_eq!((5u32 & 31) + (10u32 & 31) * 32, 325);

        // Verify our read path mapping matches: i % 32 = x, i / 32 = z
        for i in 0..1024u32 {
            let x = i % 32;
            let z = i / 32;
            assert_eq!((x & 31) + (z & 31) * 32, i);
        }
    }

    // ─── Block state packing: spec says entries don't span longs ────────────

    #[test]
    fn test_pack_unpack_roundtrip() {
        let mut indices = vec![0u16; 4096];
        indices[0] = 1;
        indices[1] = 2;
        indices[15] = 3;
        indices[256] = 4;
        indices[4095] = 5;

        let palette_size = 6;
        let packed = pack_block_states(&indices, palette_size);
        let unpacked = unpack_block_states(&packed, palette_size);

        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_pack_unpack_4bit_minimum() {
        // Spec: minimum 4 bits per entry even for small palettes (2-16 entries)
        // Palette of 3 entries: ceil(log2(3)) = 2, but minimum is 4
        let mut indices = vec![0u16; 4096];
        indices[0] = 2;
        indices[100] = 1;

        let palette_size = 3;
        let packed = pack_block_states(&indices, palette_size);

        // With 4 bits per entry, 16 entries per long, need 256 longs
        assert_eq!(packed.len(), 256);

        let unpacked = unpack_block_states(&packed, palette_size);
        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_pack_unpack_exact_4bit_palette() {
        // Exactly 16 entries: needs exactly 4 bits, 16 entries per long, 256 longs
        let mut indices = vec![0u16; 4096];
        for i in 0..4096 {
            indices[i] = (i % 16) as u16;
        }

        let palette_size = 16;
        let packed = pack_block_states(&indices, palette_size);
        assert_eq!(packed.len(), 256); // 4096 / 16 entries_per_long
        let unpacked = unpack_block_states(&packed, palette_size);
        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_pack_unpack_5bit() {
        // 17-32 entries: needs 5 bits, floor(64/5)=12 entries per long
        let mut indices = vec![0u16; 4096];
        for i in 0..4096 {
            indices[i] = (i % 32) as u16;
        }

        let palette_size = 32;
        let packed = pack_block_states(&indices, palette_size);
        // 5 bits per entry, 12 entries per long, ceil(4096/12)=342 longs
        assert_eq!(packed.len(), 342);
        let unpacked = unpack_block_states(&packed, palette_size);
        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_pack_unpack_6bit() {
        // 33-64 entries: needs 6 bits, floor(64/6)=10 entries per long
        let mut indices = vec![0u16; 4096];
        for i in 0..4096 {
            indices[i] = (i % 64) as u16;
        }

        let palette_size = 64;
        let packed = pack_block_states(&indices, palette_size);
        // 6 bits, 10 entries/long, ceil(4096/10) = 410
        assert_eq!(packed.len(), 410);
        let unpacked = unpack_block_states(&packed, palette_size);
        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_pack_unpack_8bit() {
        // 129-256 entries: needs 8 bits, floor(64/8)=8 entries per long
        let mut indices = vec![0u16; 4096];
        for i in 0..4096 {
            indices[i] = (i % 256) as u16;
        }

        let palette_size = 256;
        let packed = pack_block_states(&indices, palette_size);
        assert_eq!(packed.len(), 512); // 4096 / 8
        let unpacked = unpack_block_states(&packed, palette_size);
        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_pack_unpack_12bit_max() {
        // Max palette 4096 entries: 12 bits, floor(64/12)=5 entries per long
        let mut indices = vec![0u16; 4096];
        for i in 0..4096 {
            indices[i] = i as u16;
        }

        let palette_size = 4096;
        let packed = pack_block_states(&indices, palette_size);
        // 12 bits, 5 entries/long, ceil(4096/5) = 820
        assert_eq!(packed.len(), 820);
        let unpacked = unpack_block_states(&packed, palette_size);
        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_entries_dont_span_long_boundaries() {
        // Key spec invariant: entries do NOT span across i64 boundaries.
        // With 5 bits per entry, 12 fit in a long (60 bits used, 4 bits padding).
        // If entries DID span, 64/5 = 12.8, so 12 full + 1 partial would span.
        let palette_size = 32; // 5 bits
        let entries_per_long = 64 / 5; // 12
        assert_eq!(entries_per_long, 12);
        let wasted_bits = 64 - entries_per_long * 5; // 4 padding bits
        assert_eq!(wasted_bits, 4);

        // Set entry at boundary: index 11 is last in first long, index 12 is first in second
        let mut indices = vec![0u16; 4096];
        indices[11] = 31; // max for 5-bit
        indices[12] = 31;

        let packed = pack_block_states(&indices, palette_size);

        // Verify first long: entry 11 at bits 55..59, leaving bits 60..63 as padding
        let first_long = packed[0] as u64;
        let entry_11 = (first_long >> (11 * 5)) & 0x1F;
        assert_eq!(entry_11, 31);
        // Bits 60..63 should be zero (padding)
        let padding_bits = first_long >> 60;
        assert_eq!(padding_bits, 0);

        // Second long: entry 12 at bits 0..4
        let second_long = packed[1] as u64;
        let entry_12 = second_long & 0x1F;
        assert_eq!(entry_12, 31);

        let unpacked = unpack_block_states(&packed, palette_size);
        assert_eq!(indices, unpacked);
    }

    #[test]
    fn test_single_palette_no_data() {
        // Spec: single-block sections omit the data field
        let packed = pack_block_states(&[0u16; 4096], 1);
        assert!(packed.is_empty());
    }

    // ─── MCA header / detection ─────────────────────────────────────────────

    #[test]
    fn test_is_mca_detection() {
        // Empty data
        assert!(!is_mca(&[]));
        // Too small (must be >= 8192 bytes per spec: two 4KiB tables)
        assert!(!is_mca(&[0; 100]));
        assert!(!is_mca(&[0; 8191]));
        // Valid header but all-zero location entries = no chunks
        assert!(!is_mca(&[0; 8192]));

        // Valid: one location entry at offset 2 (first data sector after headers)
        let mut data = vec![0u8; 8192 + 4096];
        // Location entry 0: 3-byte BE offset = 2, 1-byte sector count = 1
        data[0] = 0;
        data[1] = 0;
        data[2] = 2;
        data[3] = 1;
        assert!(is_mca(&data));
    }

    #[test]
    fn test_is_mca_rejects_offset_less_than_2() {
        // Offset < 2 is invalid (sectors 0-1 are the headers)
        let mut data = vec![0u8; 8192 + 4096];
        // offset=1 means pointing into timestamp table — invalid
        data[0] = 0;
        data[1] = 0;
        data[2] = 1;
        data[3] = 1;
        assert!(!is_mca(&data));
    }

    // ─── MCA binary layout ──────────────────────────────────────────────────

    #[test]
    fn test_mca_header_layout() {
        // Spec: bytes 0x00-0x0FFF = location table, 0x1000-0x1FFF = timestamp table
        let mca = make_single_chunk_mca(0, 0, 0);
        let bytes = mca.to_bytes().unwrap();

        // File must be at least 8192 (header) + some data sectors
        assert!(bytes.len() >= 8192 + 4096);
        // File size must be a multiple of 4096 (sector-aligned)
        assert_eq!(bytes.len() % 4096, 0);

        // Location entry 0 should point to sector 2 (byte 0x2000)
        let offset = ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32);
        let sector_count = bytes[3];
        assert_eq!(offset, 2);
        assert!(sector_count >= 1);

        // All other location entries should be zero (only one chunk)
        for i in 1..1024 {
            let off = i * 4;
            let loc = ((bytes[off] as u32) << 16)
                | ((bytes[off + 1] as u32) << 8)
                | (bytes[off + 2] as u32);
            let cnt = bytes[off + 3];
            assert_eq!(loc, 0, "slot {} should have offset 0", i);
            assert_eq!(cnt, 0, "slot {} should have count 0", i);
        }
    }

    #[test]
    fn test_mca_chunk_data_layout() {
        // Spec: chunk data starts with 4-byte BE length + 1-byte compression type + compressed data
        let mca = make_single_chunk_mca(0, 0, 0);
        let bytes = mca.to_bytes().unwrap();

        // Chunk data starts at sector 2 = byte 8192
        let data_start = 8192;

        // 4-byte big-endian length (payload size = compressed_data_len + 1 for compression byte)
        let length = ((bytes[data_start] as u32) << 24)
            | ((bytes[data_start + 1] as u32) << 16)
            | ((bytes[data_start + 2] as u32) << 8)
            | (bytes[data_start + 3] as u32);

        assert!(length > 1, "length must include at least compression byte");

        // Compression type byte
        let compression = bytes[data_start + 4];
        assert_eq!(compression, 2, "should be zlib (type 2)");

        // Compressed data length = length - 1 (subtract compression byte)
        let compressed_len = (length - 1) as usize;
        assert!(compressed_len > 0);

        // Verify the compressed data can be decompressed
        let compressed = &bytes[data_start + 5..data_start + 5 + compressed_len];
        let decompressed = decompress_chunk(compressed, CompressionType::Zlib).unwrap();
        assert!(!decompressed.is_empty());

        // Verify it's valid NBT
        let (nbt, _) =
            quartz_nbt::io::read_nbt(&mut Cursor::new(&decompressed), Flavor::Uncompressed)
                .unwrap();
        assert!(nbt.get::<_, i32>("DataVersion").is_ok());
    }

    #[test]
    fn test_mca_sector_alignment() {
        // Spec: "Minecraft always pads the last chunk's data to be a multiple-of-4096B"
        let mca = make_single_chunk_mca(0, 0, 0);
        let bytes = mca.to_bytes().unwrap();

        assert_eq!(bytes.len() % 4096, 0, "file size must be 4096-byte aligned");

        // Verify sector count in location table matches actual data
        let offset = ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32);
        let sector_count = bytes[3] as usize;

        let chunk_data_start = (offset as usize) * 4096;
        let chunk_data_end = chunk_data_start + sector_count * 4096;
        assert!(chunk_data_end <= bytes.len());
    }

    // ─── Chunk indexing: multiple chunks at different locations ──────────────

    #[test]
    fn test_mca_multiple_chunks_indexing() {
        // Place chunks at specific indices matching spec formula: (x & 31) + (z & 31) * 32
        let mut chunks: Vec<Option<ChunkData>> = (0..1024).map(|_| None).collect();

        // Chunk at local (0, 0) → index 0
        chunks[0] = Some(make_chunk(0, 0));
        // Chunk at local (5, 3) → index 5 + 3*32 = 101
        chunks[101] = Some(make_chunk(5, 3));
        // Chunk at local (31, 31) → index 31 + 31*32 = 1023
        chunks[1023] = Some(make_chunk(31, 31));

        let mca = McaFile {
            chunks,
            region_x: 0,
            region_z: 0,
        };

        let bytes = mca.to_bytes().unwrap();
        let mca2 = McaFile::from_bytes(&bytes, 0, 0).unwrap();

        // Verify chunks are at correct indices
        assert!(mca2.chunks[0].is_some());
        assert!(mca2.chunks[101].is_some());
        assert!(mca2.chunks[1023].is_some());

        // Verify positions
        assert_eq!(mca2.chunks[0].as_ref().unwrap().x, 0);
        assert_eq!(mca2.chunks[0].as_ref().unwrap().z, 0);
        assert_eq!(mca2.chunks[101].as_ref().unwrap().x, 5);
        assert_eq!(mca2.chunks[101].as_ref().unwrap().z, 3);
        assert_eq!(mca2.chunks[1023].as_ref().unwrap().x, 31);
        assert_eq!(mca2.chunks[1023].as_ref().unwrap().z, 31);

        // Verify no ghost chunks
        let count = mca2.chunks.iter().filter(|c| c.is_some()).count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_mca_negative_region_coords() {
        // Region (-1, -2): chunks should have absolute coords in [-32..-1] x [-64..-33]
        let mut chunks: Vec<Option<ChunkData>> = (0..1024).map(|_| None).collect();
        // local (0, 0) → absolute chunk (-32, -64)
        chunks[0] = Some(make_chunk(-32, -64));

        let mca = McaFile {
            chunks,
            region_x: -1,
            region_z: -2,
        };

        let bytes = mca.to_bytes().unwrap();
        let mca2 = McaFile::from_bytes(&bytes, -1, -2).unwrap();

        let chunk = mca2.chunks[0].as_ref().unwrap();
        assert_eq!(chunk.x, -32);
        assert_eq!(chunk.z, -64);
    }

    // ─── Chunk NBT fields per spec ──────────────────────────────────────────

    #[test]
    fn test_chunk_nbt_has_required_fields() {
        let chunk = make_chunk(5, 10);
        let nbt = build_chunk_nbt(&chunk);

        // Spec required fields
        assert_eq!(nbt.get::<_, i32>("DataVersion").unwrap(), 3700);
        assert_eq!(nbt.get::<_, i32>("xPos").unwrap(), 5);
        assert_eq!(nbt.get::<_, i32>("zPos").unwrap(), 10);
        assert_eq!(nbt.get::<_, i32>("yPos").unwrap(), -4);
        assert_eq!(nbt.get::<_, &str>("Status").unwrap(), "minecraft:full");
        assert!(nbt.get::<_, &NbtList>("sections").is_ok());
        assert!(nbt.get::<_, &NbtList>("block_entities").is_ok());
    }

    #[test]
    fn test_section_nbt_structure() {
        let section = ChunkSection {
            y: 4,
            palette: vec![
                BlockState::new("minecraft:air".to_string()),
                BlockState::new("minecraft:stone".to_string()),
            ],
            block_states: vec![0u16; 4096],
        };
        let nbt = build_section_nbt(&section);

        // Spec: Y byte, block_states compound, biomes compound
        assert_eq!(nbt.get::<_, i8>("Y").unwrap(), 4);

        let bs = nbt.get::<_, &NbtCompound>("block_states").unwrap();
        assert!(bs.get::<_, &NbtList>("palette").is_ok());
        assert!(bs.get::<_, &[i64]>("data").is_ok());

        let biomes = nbt.get::<_, &NbtCompound>("biomes").unwrap();
        assert!(biomes.get::<_, &NbtList>("palette").is_ok());
    }

    #[test]
    fn test_section_single_palette_omits_data() {
        // Spec: "single-block sections omit the data field"
        let section = ChunkSection {
            y: 0,
            palette: vec![BlockState::new("minecraft:air".to_string())],
            block_states: vec![0u16; 4096],
        };
        let nbt = build_section_nbt(&section);
        let bs = nbt.get::<_, &NbtCompound>("block_states").unwrap();
        assert!(
            bs.get::<_, &[i64]>("data").is_err(),
            "single-palette section should omit data"
        );
    }

    // ─── Block state palette in NBT ─────────────────────────────────────────

    #[test]
    fn test_palette_nbt_format() {
        let section = ChunkSection {
            y: 0,
            palette: vec![
                BlockState::new("minecraft:air".to_string()),
                BlockState::new("minecraft:oak_stairs".to_string())
                    .with_property("facing".to_string(), "north".to_string())
                    .with_property("half".to_string(), "bottom".to_string()),
            ],
            block_states: vec![0u16; 4096],
        };
        let nbt = build_section_nbt(&section);
        let bs = nbt.get::<_, &NbtCompound>("block_states").unwrap();
        let palette = bs.get::<_, &NbtList>("palette").unwrap();

        // First entry: just Name
        if let NbtTag::Compound(entry) = &palette[0] {
            assert_eq!(entry.get::<_, &str>("Name").unwrap(), "minecraft:air");
            assert!(entry.get::<_, &NbtCompound>("Properties").is_err());
        } else {
            panic!("palette entry should be Compound");
        }

        // Second entry: Name + Properties
        if let NbtTag::Compound(entry) = &palette[1] {
            assert_eq!(
                entry.get::<_, &str>("Name").unwrap(),
                "minecraft:oak_stairs"
            );
            let props = entry.get::<_, &NbtCompound>("Properties").unwrap();
            assert_eq!(props.get::<_, &str>("facing").unwrap(), "north");
            assert_eq!(props.get::<_, &str>("half").unwrap(), "bottom");
        } else {
            panic!("palette entry should be Compound");
        }
    }

    // ─── Block entity roundtrip ─────────────────────────────────────────────

    #[test]
    fn test_block_entity_roundtrip() {
        let mut be = BlockEntity::new("minecraft:chest".to_string(), (10, 64, 20));
        be.nbt.insert(
            "CustomName".to_string(),
            crate::utils::NbtValue::String("Test Chest".to_string()),
        );

        let section = ChunkSection {
            y: 4,
            palette: vec![
                BlockState::new("minecraft:air".to_string()),
                BlockState::new("minecraft:chest".to_string()),
            ],
            block_states: {
                let mut bs = vec![0u16; 4096];
                bs[0] = 1;
                bs
            },
        };

        let chunk = ChunkData {
            x: 0,
            z: 1,
            data_version: 3700,
            status: "minecraft:full".to_string(),
            sections: vec![section],
            block_entities: vec![be],
            entities: Vec::new(),
            y_pos: -4,
        };

        let mut chunks: Vec<Option<ChunkData>> = (0..1024).map(|_| None).collect();
        // index = (0 & 31) + (1 & 31) * 32 = 32
        chunks[32] = Some(chunk);

        let mca = McaFile {
            chunks,
            region_x: 0,
            region_z: 0,
        };

        let bytes = mca.to_bytes().unwrap();
        let mca2 = McaFile::from_bytes(&bytes, 0, 0).unwrap();
        let chunk2 = mca2.chunks[32].as_ref().unwrap();

        assert_eq!(chunk2.block_entities.len(), 1);
        assert_eq!(chunk2.block_entities[0].id, "minecraft:chest");
        assert_eq!(chunk2.block_entities[0].position, (10, 64, 20));
    }

    // ─── Compression type ───────────────────────────────────────────────────

    #[test]
    fn test_compression_type_values() {
        // Spec: 1=Gzip, 2=Zlib, 3=Uncompressed, 4=LZ4
        assert_eq!(
            CompressionType::from_byte(1).unwrap(),
            CompressionType::Gzip
        );
        assert_eq!(
            CompressionType::from_byte(2).unwrap(),
            CompressionType::Zlib
        );
        assert_eq!(
            CompressionType::from_byte(3).unwrap(),
            CompressionType::Uncompressed
        );
        // LZ4 returns error (unsupported)
        assert!(CompressionType::from_byte(4).is_err());
        // Unknown type
        assert!(CompressionType::from_byte(5).is_err());
        assert!(CompressionType::from_byte(127).is_err());
    }

    // ─── Section Y ordering ─────────────────────────────────────────────────

    #[test]
    fn test_multiple_sections_y_ordering() {
        let sections = vec![
            make_section(-4, "minecraft:bedrock"),
            make_section(0, "minecraft:stone"),
            make_section(4, "minecraft:air"),
        ];

        let chunk = ChunkData {
            x: 0,
            z: 0,
            data_version: 3700,
            status: "minecraft:full".to_string(),
            sections,
            block_entities: Vec::new(),
            entities: Vec::new(),
            y_pos: -4,
        };

        let mut chunks: Vec<Option<ChunkData>> = (0..1024).map(|_| None).collect();
        chunks[0] = Some(chunk);

        let mca = McaFile {
            chunks,
            region_x: 0,
            region_z: 0,
        };

        let bytes = mca.to_bytes().unwrap();
        let mca2 = McaFile::from_bytes(&bytes, 0, 0).unwrap();
        let chunk2 = mca2.chunks[0].as_ref().unwrap();

        assert_eq!(chunk2.sections.len(), 3);
        // Verify section Y values preserved
        let ys: Vec<i8> = chunk2.sections.iter().map(|s| s.y).collect();
        assert!(ys.contains(&-4));
        assert!(ys.contains(&0));
        assert!(ys.contains(&4));
    }

    // ─── Full MCA roundtrip with all features ───────────────────────────────

    #[test]
    fn test_mca_roundtrip() {
        let mut section = ChunkSection {
            y: 0,
            palette: vec![
                BlockState::new("minecraft:air".to_string()),
                BlockState::new("minecraft:stone".to_string()),
            ],
            block_states: vec![0u16; 4096],
        };
        section.block_states[0] = 1;
        section.block_states[1] = 1;
        section.block_states[100] = 1;

        let chunk = ChunkData {
            x: 0,
            z: 0,
            data_version: 3700,
            status: "minecraft:full".to_string(),
            sections: vec![section],
            block_entities: Vec::new(),
            entities: Vec::new(),
            y_pos: -4,
        };

        let mut chunks: Vec<Option<ChunkData>> = (0..1024).map(|_| None).collect();
        chunks[0] = Some(chunk);

        let mca = McaFile {
            chunks,
            region_x: 0,
            region_z: 0,
        };

        let bytes = mca.to_bytes().unwrap();
        assert!(is_mca(&bytes));

        let mca2 = McaFile::from_bytes(&bytes, 0, 0).unwrap();
        let chunk2 = mca2.chunks[0].as_ref().unwrap();
        assert_eq!(chunk2.x, 0);
        assert_eq!(chunk2.z, 0);
        assert_eq!(chunk2.data_version, 3700);
        assert_eq!(chunk2.status, "minecraft:full");
        assert_eq!(chunk2.y_pos, -4);
        assert_eq!(chunk2.sections.len(), 1);
        assert_eq!(chunk2.sections[0].palette.len(), 2);
        assert_eq!(chunk2.sections[0].palette[0].name, "minecraft:air");
        assert_eq!(chunk2.sections[0].palette[1].name, "minecraft:stone");
        assert_eq!(chunk2.sections[0].block_states[0], 1);
        assert_eq!(chunk2.sections[0].block_states[1], 1);
        assert_eq!(chunk2.sections[0].block_states[100], 1);
        assert_eq!(chunk2.sections[0].block_states[2], 0);
    }

    #[test]
    fn test_mca_roundtrip_with_properties() {
        // Verify block properties survive roundtrip
        let section = ChunkSection {
            y: 0,
            palette: vec![
                BlockState::new("minecraft:air".to_string()),
                BlockState::new("minecraft:redstone_wire".to_string())
                    .with_property("power".to_string(), "15".to_string())
                    .with_property("east".to_string(), "side".to_string()),
                BlockState::new("minecraft:oak_stairs".to_string())
                    .with_property("facing".to_string(), "north".to_string())
                    .with_property("half".to_string(), "top".to_string())
                    .with_property("shape".to_string(), "straight".to_string()),
            ],
            block_states: {
                let mut bs = vec![0u16; 4096];
                bs[0] = 1;
                bs[1] = 2;
                bs
            },
        };

        let chunk = ChunkData {
            x: 0,
            z: 0,
            data_version: 3700,
            status: "minecraft:full".to_string(),
            sections: vec![section],
            block_entities: Vec::new(),
            entities: Vec::new(),
            y_pos: -4,
        };

        let mut chunks: Vec<Option<ChunkData>> = (0..1024).map(|_| None).collect();
        chunks[0] = Some(chunk);

        let mca = McaFile {
            chunks,
            region_x: 0,
            region_z: 0,
        };

        let bytes = mca.to_bytes().unwrap();
        let mca2 = McaFile::from_bytes(&bytes, 0, 0).unwrap();
        let chunk2 = mca2.chunks[0].as_ref().unwrap();

        let redstone = &chunk2.sections[0].palette[1];
        assert_eq!(redstone.name, "minecraft:redstone_wire");
        assert_eq!(redstone.get_property("power"), Some(&SmolStr::from("15")));
        assert_eq!(redstone.get_property("east"), Some(&SmolStr::from("side")));

        let stairs = &chunk2.sections[0].palette[2];
        assert_eq!(stairs.name, "minecraft:oak_stairs");
        assert_eq!(stairs.get_property("facing"), Some(&SmolStr::from("north")));
        assert_eq!(stairs.get_property("half"), Some(&SmolStr::from("top")));
    }

    // ─── Helpers ────────────────────────────────────────────────────────────

    fn make_chunk(x: i32, z: i32) -> ChunkData {
        ChunkData {
            x,
            z,
            data_version: 3700,
            status: "minecraft:full".to_string(),
            sections: vec![ChunkSection {
                y: 0,
                palette: vec![
                    BlockState::new("minecraft:air".to_string()),
                    BlockState::new("minecraft:stone".to_string()),
                ],
                block_states: {
                    let mut bs = vec![0u16; 4096];
                    bs[0] = 1;
                    bs
                },
            }],
            block_entities: Vec::new(),
            entities: Vec::new(),
            y_pos: -4,
        }
    }

    fn make_section(y: i8, block_name: &str) -> ChunkSection {
        ChunkSection {
            y,
            palette: vec![
                BlockState::new("minecraft:air".to_string()),
                BlockState::new(block_name.to_string()),
            ],
            block_states: {
                let mut bs = vec![0u16; 4096];
                bs[0] = 1;
                bs
            },
        }
    }

    fn make_single_chunk_mca(index: usize, region_x: i32, region_z: i32) -> McaFile {
        let chunk_x = region_x * 32 + (index % 32) as i32;
        let chunk_z = region_z * 32 + (index / 32) as i32;
        let mut chunks: Vec<Option<ChunkData>> = (0..1024).map(|_| None).collect();
        chunks[index] = Some(make_chunk(chunk_x, chunk_z));
        McaFile {
            chunks,
            region_x,
            region_z,
        }
    }
}
