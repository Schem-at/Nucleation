use crate::block_entity::BlockEntity;
use crate::block_position::BlockPosition;
use crate::formats::anvil::{
    floor_div, floor_mod, is_mca, parse_entity_mca, write_entity_mca, ChunkData, ChunkSection,
    EntityChunkData, McaFile,
};
use crate::formats::manager::{SchematicExporter, SchematicImporter};
use crate::universal_schematic::UniversalSchematic;
use crate::BlockState;
use flate2::write::GzEncoder;
use flate2::Compression;
use quartz_nbt::io::Flavor;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

// ─── Types ──────────────────────────────────────────────────────────────────

/// A collection of files representing a Minecraft world.
/// Keys are relative paths (e.g. "level.dat", "region/r.0.0.mca").
pub type WorldFiles = HashMap<String, Vec<u8>>;

/// Options for world export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldExportOptions {
    #[serde(default = "default_world_name")]
    pub world_name: String,
    /// 0=Survival, 1=Creative, 2=Adventure, 3=Spectator
    #[serde(default = "default_game_mode")]
    pub game_mode: i32,
    /// 0=Peaceful, 1=Easy, 2=Normal, 3=Hard
    #[serde(default)]
    pub difficulty: i32,
    /// World spawn position (x, y, z)
    #[serde(default)]
    pub spawn_position: Option<(i32, i32, i32)>,
    /// Minecraft data version (4671 = 1.21.4)
    #[serde(default = "default_data_version")]
    pub data_version: i32,
    /// Minecraft version string (e.g. "1.21.4")
    #[serde(default = "default_version_name")]
    pub version_name: String,
    /// Use void/superflat world generation
    #[serde(default = "default_true")]
    pub void_world: bool,
    /// Block coordinate offset for placement
    #[serde(default)]
    pub offset: (i32, i32, i32),
    /// Enable cheats/commands
    #[serde(default = "default_true")]
    pub allow_commands: bool,
    /// Fixed time of day (None = normal day cycle)
    #[serde(default = "default_day_time")]
    pub day_time: Option<i64>,
    /// Disable mob spawning
    #[serde(default = "default_true")]
    pub disable_mob_spawning: bool,
}

fn default_world_name() -> String {
    "Nucleation Export".to_string()
}
fn default_game_mode() -> i32 {
    1
}
fn default_data_version() -> i32 {
    4671
}
fn default_version_name() -> String {
    "1.21.4".to_string()
}
fn default_true() -> bool {
    true
}
fn default_day_time() -> Option<i64> {
    Some(6000)
}

impl Default for WorldExportOptions {
    fn default() -> Self {
        WorldExportOptions {
            world_name: default_world_name(),
            game_mode: default_game_mode(),
            difficulty: 0,
            spawn_position: None,
            data_version: default_data_version(),
            version_name: default_version_name(),
            void_world: true,
            offset: (0, 0, 0),
            allow_commands: true,
            day_time: default_day_time(),
            disable_mob_spawning: true,
        }
    }
}

/// Settings for world import (bounds filtering).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldImportSettings {
    pub min_x: Option<i32>,
    pub min_y: Option<i32>,
    pub min_z: Option<i32>,
    pub max_x: Option<i32>,
    pub max_y: Option<i32>,
    pub max_z: Option<i32>,
}

impl WorldImportSettings {
    fn bounds(&self) -> Option<(i32, i32, i32, i32, i32, i32)> {
        match (
            self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z,
        ) {
            (Some(min_x), Some(min_y), Some(min_z), Some(max_x), Some(max_y), Some(max_z)) => {
                Some((min_x, min_y, min_z, max_x, max_y, max_z))
            }
            _ => None,
        }
    }
}

// ─── Format Handlers (for FormatManager auto-detection) ─────────────────────

pub struct McaFormat;

impl SchematicImporter for McaFormat {
    fn name(&self) -> String {
        "MCA".to_string()
    }

    fn detect(&self, data: &[u8]) -> bool {
        is_mca(data)
    }

    fn read(&self, data: &[u8]) -> Result<UniversalSchematic, Box<dyn Error>> {
        from_mca(data)
    }

    fn read_with_settings(
        &self,
        data: &[u8],
        settings: Option<&str>,
    ) -> Result<UniversalSchematic, Box<dyn Error>> {
        let bounds = settings
            .map(|s| serde_json::from_str::<WorldImportSettings>(s))
            .transpose()?
            .and_then(|s| s.bounds());
        match bounds {
            Some((min_x, min_y, min_z, max_x, max_y, max_z)) => {
                from_mca_bounded(data, min_x, min_y, min_z, max_x, max_y, max_z)
            }
            None => from_mca(data),
        }
    }

    fn import_settings_schema(&self) -> Option<String> {
        serde_json::to_string_pretty(&WorldImportSettings::default()).ok()
    }
}

pub struct WorldZipFormat;

impl SchematicImporter for WorldZipFormat {
    fn name(&self) -> String {
        "WorldZip".to_string()
    }

    fn detect(&self, data: &[u8]) -> bool {
        is_world_zip(data)
    }

    fn read(&self, data: &[u8]) -> Result<UniversalSchematic, Box<dyn Error>> {
        from_world_zip(data)
    }

    fn read_with_settings(
        &self,
        data: &[u8],
        settings: Option<&str>,
    ) -> Result<UniversalSchematic, Box<dyn Error>> {
        let bounds = settings
            .map(|s| serde_json::from_str::<WorldImportSettings>(s))
            .transpose()?
            .and_then(|s| s.bounds());
        match bounds {
            Some((min_x, min_y, min_z, max_x, max_y, max_z)) => {
                from_world_zip_bounded(data, min_x, min_y, min_z, max_x, max_y, max_z)
            }
            None => from_world_zip(data),
        }
    }

    fn import_settings_schema(&self) -> Option<String> {
        serde_json::to_string_pretty(&WorldImportSettings::default()).ok()
    }
}

pub struct WorldFormat;

impl SchematicExporter for WorldFormat {
    fn name(&self) -> String {
        "world".to_string()
    }

    fn extensions(&self) -> Vec<String> {
        vec!["zip".to_string()]
    }

    fn available_versions(&self) -> Vec<String> {
        vec!["default".to_string()]
    }

    fn default_version(&self) -> String {
        "default".to_string()
    }

    fn write(
        &self,
        schematic: &UniversalSchematic,
        _version: Option<&str>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let opts = WorldExportOptions::default();
        let files = to_world(schematic, None)?;
        let prefixed = prefix_world_files(&files, &opts.world_name);
        zip_world_files(&prefixed)
    }

    fn write_with_settings(
        &self,
        schematic: &UniversalSchematic,
        _version: Option<&str>,
        settings: Option<&str>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let options: Option<WorldExportOptions> =
            settings.map(|s| serde_json::from_str(s)).transpose()?;
        let world_name = options
            .as_ref()
            .map(|o| o.world_name.clone())
            .unwrap_or_else(default_world_name);
        let files = to_world(schematic, options)?;
        let prefixed = prefix_world_files(&files, &world_name);
        zip_world_files(&prefixed)
    }

    fn export_settings_schema(&self) -> Option<String> {
        serde_json::to_string_pretty(&WorldExportOptions::default()).ok()
    }
}

/// Add a wrapper directory prefix to all file paths in a WorldFiles map.
fn prefix_world_files(files: &WorldFiles, world_name: &str) -> WorldFiles {
    files
        .iter()
        .map(|(k, v)| (format!("{}/{}", world_name, k), v.clone()))
        .collect()
}

/// Zip a WorldFiles HashMap into a single byte buffer.
pub fn zip_world_files(files: &WorldFiles) -> Result<Vec<u8>, Box<dyn Error>> {
    use std::io::Cursor;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let buf = Vec::new();
    let cursor = Cursor::new(buf);
    let mut zip = ZipWriter::new(cursor);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let mut paths: Vec<&String> = files.keys().collect();
    paths.sort();

    for path in paths {
        let data = &files[path];
        zip.start_file(path, options)?;
        std::io::Write::write_all(&mut zip, data)?;
    }

    let cursor = zip.finish()?;
    Ok(cursor.into_inner())
}

/// Export a schematic as a zipped Minecraft world.
pub fn to_world_zip(
    schematic: &UniversalSchematic,
    options: Option<WorldExportOptions>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let world_name = options
        .as_ref()
        .map(|o| o.world_name.clone())
        .unwrap_or_else(default_world_name);
    let files = to_world(schematic, options)?;
    let prefixed = prefix_world_files(&files, &world_name);
    zip_world_files(&prefixed)
}

// ─── Import: Single MCA ────────────────────────────────────────────────────

/// Import all chunks from a single MCA file.
pub fn from_mca(data: &[u8]) -> Result<UniversalSchematic, Box<dyn Error>> {
    let mca = McaFile::from_bytes_auto(data)?;
    let mut schematic = UniversalSchematic::new("MCA Import".to_string());
    load_mca_into_schematic(&mca, &mut schematic, None);
    Ok(schematic)
}

/// Import chunks from a single MCA file, filtering by block coordinate bounds.
pub fn from_mca_bounded(
    data: &[u8],
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
) -> Result<UniversalSchematic, Box<dyn Error>> {
    let mca = McaFile::from_bytes_auto(data)?;
    let mut schematic = UniversalSchematic::new("MCA Import".to_string());
    let bounds = (min_x, min_y, min_z, max_x, max_y, max_z);
    load_mca_into_schematic(&mca, &mut schematic, Some(bounds));
    Ok(schematic)
}

// ─── Import: Zipped World ──────────────────────────────────────────────────

fn is_world_zip(data: &[u8]) -> bool {
    // Check for PK magic bytes
    if data.len() < 4 || data[0] != 0x50 || data[1] != 0x4B || data[2] != 0x03 || data[3] != 0x04 {
        return false;
    }

    // Try to open as zip and look for region/*.mca files
    let cursor = std::io::Cursor::new(data);
    if let Ok(mut archive) = zip::ZipArchive::new(cursor) {
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index_raw(i) {
                let name = file.name().to_lowercase();
                if name.contains("region/") && name.ends_with(".mca") {
                    return true;
                }
            }
        }
    }
    false
}

/// Import from a zipped Minecraft world folder.
pub fn from_world_zip(data: &[u8]) -> Result<UniversalSchematic, Box<dyn Error>> {
    from_world_zip_impl(data, None)
}

/// Import from a zipped world folder with bounds filtering.
pub fn from_world_zip_bounded(
    data: &[u8],
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
) -> Result<UniversalSchematic, Box<dyn Error>> {
    let bounds = (min_x, min_y, min_z, max_x, max_y, max_z);
    from_world_zip_impl(data, Some(bounds))
}

fn from_world_zip_impl(
    data: &[u8],
    bounds: Option<(i32, i32, i32, i32, i32, i32)>,
) -> Result<UniversalSchematic, Box<dyn Error>> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    let mut schematic = UniversalSchematic::new("World Import".to_string());

    // Find block region files (region/*.mca) and entity region files (entities/*.mca)
    let mut region_indices: Vec<usize> = Vec::new();
    let mut entity_indices: Vec<usize> = Vec::new();

    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index_raw(i) {
            let name = file.name().to_lowercase();
            if name.ends_with(".mca") {
                if name.contains("region/") {
                    region_indices.push(i);
                } else if name.contains("entities/") {
                    entity_indices.push(i);
                }
            }
        }
    }

    // Process block region files
    for idx in region_indices {
        let mut file = archive.by_index(idx)?;
        let name = file.name().to_string();

        let (rx, rz) = parse_region_filename(&name).unwrap_or((0, 0));

        let mut mca_data = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut mca_data)?;

        if let Ok(mca) = McaFile::from_bytes(&mca_data, rx, rz) {
            load_mca_into_schematic(&mca, &mut schematic, bounds);
        }
    }

    // Process entity region files (1.17+)
    for idx in entity_indices {
        let mut file = archive.by_index(idx)?;
        let name = file.name().to_string();

        let (rx, rz) = parse_region_filename(&name).unwrap_or((0, 0));

        let mut mca_data = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut mca_data)?;

        if let Ok(entity_chunks) = parse_entity_mca(&mca_data, rx, rz) {
            load_entities_into_schematic(&entity_chunks, &mut schematic, bounds);
        }
    }

    Ok(schematic)
}

// ─── Import: Directory (not available on WASM) ─────────────────────────────

/// Import from a Minecraft world directory.
#[cfg(not(target_arch = "wasm32"))]
pub fn from_world_directory(path: &Path) -> Result<UniversalSchematic, Box<dyn Error>> {
    from_world_directory_impl(path, None)
}

/// Import from a world directory with bounds filtering.
#[cfg(not(target_arch = "wasm32"))]
pub fn from_world_directory_bounded(
    path: &Path,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
) -> Result<UniversalSchematic, Box<dyn Error>> {
    let bounds = (min_x, min_y, min_z, max_x, max_y, max_z);
    from_world_directory_impl(path, Some(bounds))
}

#[cfg(not(target_arch = "wasm32"))]
fn from_world_directory_impl(
    path: &Path,
    bounds: Option<(i32, i32, i32, i32, i32, i32)>,
) -> Result<UniversalSchematic, Box<dyn Error>> {
    let region_dir = path.join("region");
    if !region_dir.exists() {
        return Err(format!("No 'region' directory found at {}", path.display()).into());
    }

    let mut schematic = UniversalSchematic::new("World Import".to_string());

    // Process block region files
    for entry in std::fs::read_dir(&region_dir)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.extension().map_or(false, |ext| ext == "mca") {
            let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let (rx, rz) = parse_region_filename(filename).unwrap_or((0, 0));

            let mca_data = std::fs::read(&file_path)?;
            if let Ok(mca) = McaFile::from_bytes(&mca_data, rx, rz) {
                load_mca_into_schematic(&mca, &mut schematic, bounds);
            }
        }
    }

    // Process entity region files (1.17+)
    let entities_dir = path.join("entities");
    if entities_dir.exists() {
        for entry in std::fs::read_dir(&entities_dir)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path.extension().map_or(false, |ext| ext == "mca") {
                let filename = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                let (rx, rz) = parse_region_filename(filename).unwrap_or((0, 0));

                let mca_data = std::fs::read(&file_path)?;
                if let Ok(entity_chunks) = parse_entity_mca(&mca_data, rx, rz) {
                    load_entities_into_schematic(&entity_chunks, &mut schematic, bounds);
                }
            }
        }
    }

    Ok(schematic)
}

// ─── Core Import Helper ────────────────────────────────────────────────────

/// Load chunks from an MCA file into a schematic, optionally filtering by bounds.
fn load_mca_into_schematic(
    mca: &McaFile,
    schematic: &mut UniversalSchematic,
    bounds: Option<(i32, i32, i32, i32, i32, i32)>,
) {
    for chunk_opt in &mca.chunks {
        let chunk = match chunk_opt {
            Some(c) => c,
            None => continue,
        };

        let chunk_world_x = chunk.x * 16;
        let chunk_world_z = chunk.z * 16;

        for section in &chunk.sections {
            let section_world_y = (section.y as i32) * 16;

            for local_y in 0..16 {
                for local_z in 0..16 {
                    for local_x in 0..16 {
                        let world_x = chunk_world_x + local_x;
                        let world_y = section_world_y + local_y;
                        let world_z = chunk_world_z + local_z;

                        // Check bounds if specified
                        if let Some((min_x, min_y, min_z, max_x, max_y, max_z)) = bounds {
                            if world_x < min_x
                                || world_x > max_x
                                || world_y < min_y
                                || world_y > max_y
                                || world_z < min_z
                                || world_z > max_z
                            {
                                continue;
                            }
                        }

                        let idx = (local_y * 16 * 16 + local_z * 16 + local_x) as usize;
                        let palette_idx = section.block_states[idx] as usize;

                        if palette_idx >= section.palette.len() {
                            continue;
                        }

                        let block = &section.palette[palette_idx];
                        // Skip air blocks
                        if is_air(&block.name) {
                            continue;
                        }

                        schematic.set_block(world_x, world_y, world_z, block);
                    }
                }
            }
        }

        // Add block entities
        for be in &chunk.block_entities {
            if let Some((min_x, min_y, min_z, max_x, max_y, max_z)) = bounds {
                let (bx, by, bz) = be.position;
                if bx < min_x || bx > max_x || by < min_y || by > max_y || bz < min_z || bz > max_z
                {
                    continue;
                }
            }
            schematic.default_region.set_block_entity(
                BlockPosition {
                    x: be.position.0,
                    y: be.position.1,
                    z: be.position.2,
                },
                be.clone(),
            );
        }

        // Add entities
        for entity in &chunk.entities {
            if let Some((min_x, min_y, min_z, max_x, max_y, max_z)) = bounds {
                let (ex, ey, ez) = (
                    entity.position.0 as i32,
                    entity.position.1 as i32,
                    entity.position.2 as i32,
                );
                if ex < min_x || ex > max_x || ey < min_y || ey > max_y || ez < min_z || ez > max_z
                {
                    continue;
                }
            }
            schematic.default_region.add_entity(entity.clone());
        }
    }
}

/// Load entities from entity region chunks into a schematic.
fn load_entities_into_schematic(
    entity_chunks: &[EntityChunkData],
    schematic: &mut UniversalSchematic,
    bounds: Option<(i32, i32, i32, i32, i32, i32)>,
) {
    for chunk in entity_chunks {
        for entity in &chunk.entities {
            if let Some((min_x, min_y, min_z, max_x, max_y, max_z)) = bounds {
                let (ex, ey, ez) = (
                    entity.position.0 as i32,
                    entity.position.1 as i32,
                    entity.position.2 as i32,
                );
                if ex < min_x || ex > max_x || ey < min_y || ey > max_y || ez < min_z || ez > max_z
                {
                    continue;
                }
            }
            schematic.default_region.add_entity(entity.clone());
        }
    }
}

fn is_air(name: &str) -> bool {
    matches!(
        name,
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
    )
}

fn parse_region_filename(name: &str) -> Option<(i32, i32)> {
    // Handle both "r.0.-1.mca" and "path/to/region/r.0.-1.mca"
    let basename = name.rsplit('/').next().unwrap_or(name);
    let basename = basename.rsplit('\\').next().unwrap_or(basename);
    let parts: Vec<&str> = basename.split('.').collect();
    if parts.len() >= 4 && parts[0] == "r" {
        let x = parts[1].parse::<i32>().ok()?;
        let z = parts[2].parse::<i32>().ok()?;
        Some((x, z))
    } else {
        None
    }
}

// ─── Export ─────────────────────────────────────────────────────────────────

/// Export a schematic as a Minecraft world.
pub fn to_world(
    schematic: &UniversalSchematic,
    options: Option<WorldExportOptions>,
) -> Result<WorldFiles, Box<dyn Error>> {
    let opts = options.unwrap_or_default();
    let mut files = WorldFiles::new();

    // Generate level.dat
    let level_dat = generate_level_dat(&opts)?;
    files.insert("level.dat".to_string(), level_dat);

    // Generate session.lock (8-byte timestamp, big-endian)
    let timestamp: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    files.insert("session.lock".to_string(), timestamp.to_be_bytes().to_vec());

    // Collect all blocks from all regions
    let all_regions = schematic.get_all_regions();

    // Group blocks into chunk columns, then into sections
    // Key: (chunk_x, chunk_z) -> Key: section_y -> blocks
    let mut chunk_columns: HashMap<(i32, i32), HashMap<i8, SectionBuilder>> = HashMap::new();
    let mut chunk_block_entities: HashMap<(i32, i32), Vec<BlockEntity>> = HashMap::new();

    for (_name, region) in &all_regions {
        let bb = region.get_bounding_box();
        let (min_x, min_y, min_z) = bb.min;
        let (max_x, max_y, max_z) = bb.max;

        for y in min_y..=max_y {
            for z in min_z..=max_z {
                for x in min_x..=max_x {
                    if let Some(block) = region.get_block(x, y, z) {
                        if is_air(&block.name) {
                            continue;
                        }

                        let world_x = x + opts.offset.0;
                        let world_y = y + opts.offset.1;
                        let world_z = z + opts.offset.2;

                        let chunk_x = floor_div(world_x, 16);
                        let chunk_z = floor_div(world_z, 16);
                        let section_y = floor_div(world_y, 16) as i8;

                        let local_x = floor_mod(world_x, 16);
                        let local_y = floor_mod(world_y, 16);
                        let local_z = floor_mod(world_z, 16);

                        let section = chunk_columns
                            .entry((chunk_x, chunk_z))
                            .or_default()
                            .entry(section_y)
                            .or_insert_with(|| SectionBuilder::new(section_y));

                        let idx = (local_y * 16 * 16 + local_z * 16 + local_x) as usize;
                        section.set_block(idx, block);
                    }
                }
            }
        }

        // Collect block entities
        for (pos, be) in &region.block_entities {
            let world_x = pos.0 + opts.offset.0;
            let world_y = pos.1 + opts.offset.1;
            let world_z = pos.2 + opts.offset.2;

            let chunk_x = floor_div(world_x, 16);
            let chunk_z = floor_div(world_z, 16);

            let mut be_copy = be.clone();
            be_copy.position = (world_x, world_y, world_z);

            chunk_block_entities
                .entry((chunk_x, chunk_z))
                .or_default()
                .push(be_copy);
        }
    }

    // Collect entities and group by chunk
    let mut chunk_entities: HashMap<(i32, i32), Vec<crate::entity::Entity>> = HashMap::new();

    for (_name, region) in &all_regions {
        for entity in &region.entities {
            let world_x = entity.position.0 + opts.offset.0 as f64;
            let world_y = entity.position.1 + opts.offset.1 as f64;
            let world_z = entity.position.2 + opts.offset.2 as f64;

            let chunk_x = floor_div(world_x as i32, 16);
            let chunk_z = floor_div(world_z as i32, 16);

            let mut entity_copy = entity.clone();
            entity_copy.position = (world_x, world_y, world_z);

            chunk_entities
                .entry((chunk_x, chunk_z))
                .or_default()
                .push(entity_copy);
        }
    }

    // Build ChunkData for each chunk column and group by region file
    let mut region_chunks: HashMap<(i32, i32), Vec<ChunkData>> = HashMap::new();

    for ((chunk_x, chunk_z), sections_map) in &chunk_columns {
        let mut sections: Vec<ChunkSection> = sections_map
            .iter()
            .map(|(_, builder)| builder.build())
            .collect();
        sections.sort_by_key(|s| s.y);

        let block_entities = chunk_block_entities
            .remove(&(*chunk_x, *chunk_z))
            .unwrap_or_default();

        let chunk = ChunkData {
            x: *chunk_x,
            z: *chunk_z,
            data_version: opts.data_version,
            status: "minecraft:full".to_string(),
            sections,
            block_entities,
            entities: Vec::new(),
            y_pos: -4,
        };

        let region_x = floor_div(*chunk_x, 32);
        let region_z = floor_div(*chunk_z, 32);

        region_chunks
            .entry((region_x, region_z))
            .or_default()
            .push(chunk);
    }

    // Write each block region as an MCA file
    for ((rx, rz), chunks) in &region_chunks {
        let mut mca_chunks: Vec<Option<ChunkData>> = (0..1024).map(|_| None).collect();

        for chunk in chunks {
            let local_x = floor_mod(chunk.x, 32) as u32;
            let local_z = floor_mod(chunk.z, 32) as u32;
            let index = (local_x + local_z * 32) as usize;
            mca_chunks[index] = Some(chunk.clone());
        }

        let mca = McaFile {
            chunks: mca_chunks,
            region_x: *rx,
            region_z: *rz,
        };

        let mca_bytes = mca.to_bytes()?;
        let path = format!("region/r.{}.{}.mca", rx, rz);
        files.insert(path, mca_bytes);
    }

    // Write entity region files (1.17+ format)
    if !chunk_entities.is_empty() {
        let mut entity_region_chunks: HashMap<(i32, i32), Vec<EntityChunkData>> = HashMap::new();

        for ((chunk_x, chunk_z), entities) in chunk_entities {
            let region_x = floor_div(chunk_x, 32);
            let region_z = floor_div(chunk_z, 32);

            entity_region_chunks
                .entry((region_x, region_z))
                .or_default()
                .push(EntityChunkData {
                    chunk_x,
                    chunk_z,
                    entities,
                });
        }

        for ((rx, rz), chunks) in &entity_region_chunks {
            let mca_bytes = write_entity_mca(chunks, *rx, *rz, opts.data_version)?;
            let path = format!("entities/r.{}.{}.mca", rx, rz);
            files.insert(path, mca_bytes);
        }
    }

    Ok(files)
}

/// Write world files to a directory on disk.
#[cfg(not(target_arch = "wasm32"))]
pub fn save_world(
    schematic: &UniversalSchematic,
    directory: &Path,
    options: Option<WorldExportOptions>,
) -> Result<(), Box<dyn Error>> {
    let files = to_world(schematic, options)?;

    for (path, data) in &files {
        let full_path = directory.join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full_path, data)?;
    }

    Ok(())
}

// ─── Section Builder ────────────────────────────────────────────────────────

struct SectionBuilder {
    y: i8,
    palette: Vec<BlockState>,
    palette_map: HashMap<String, u16>,
    block_states: Vec<u16>,
}

impl SectionBuilder {
    fn new(y: i8) -> Self {
        let air = BlockState::new("minecraft:air".to_string());
        let mut palette_map = HashMap::new();
        palette_map.insert(Self::block_key(&air), 0);
        SectionBuilder {
            y,
            palette: vec![air],
            palette_map,
            block_states: vec![0; 4096],
        }
    }

    fn block_key(block: &BlockState) -> String {
        if block.properties.is_empty() {
            block.name.clone()
        } else {
            let mut props: Vec<_> = block.properties.iter().collect();
            props.sort_by_key(|(k, _)| (*k).clone());
            let prop_str: Vec<String> = props.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
            format!("{}[{}]", block.name, prop_str.join(","))
        }
    }

    fn set_block(&mut self, idx: usize, block: &BlockState) {
        let key = Self::block_key(block);
        let palette_idx = if let Some(&idx) = self.palette_map.get(&key) {
            idx
        } else {
            let idx = self.palette.len() as u16;
            self.palette.push(block.clone());
            self.palette_map.insert(key, idx);
            idx
        };
        self.block_states[idx] = palette_idx;
    }

    fn build(&self) -> ChunkSection {
        ChunkSection {
            y: self.y,
            palette: self.palette.clone(),
            block_states: self.block_states.clone(),
        }
    }
}

// ─── level.dat Generation ───────────────────────────────────────────────────

fn generate_level_dat(opts: &WorldExportOptions) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut data = NbtCompound::new();

    // ── Core identity ────────────────────────────────────────────────────
    data.insert("DataVersion", NbtTag::Int(opts.data_version));
    data.insert("LevelName", NbtTag::String(opts.world_name.clone()));
    data.insert("version", NbtTag::Int(19133)); // NBT format version

    let mut version = NbtCompound::new();
    version.insert("Id", NbtTag::Int(opts.data_version));
    version.insert("Name", NbtTag::String(opts.version_name.clone()));
    version.insert("Series", NbtTag::String("main".to_string()));
    version.insert("Snapshot", NbtTag::Byte(0));
    data.insert("Version", NbtTag::Compound(version));

    // ── Gameplay settings ────────────────────────────────────────────────
    data.insert("GameType", NbtTag::Int(opts.game_mode));
    data.insert("Difficulty", NbtTag::Byte(opts.difficulty as i8));
    data.insert("DifficultyLocked", NbtTag::Byte(0));
    data.insert(
        "allowCommands",
        NbtTag::Byte(if opts.allow_commands { 1 } else { 0 }),
    );
    data.insert("hardcore", NbtTag::Byte(0));
    data.insert("initialized", NbtTag::Byte(1));

    // ── Spawn position (1.21+ uses spawn compound) ───────────────────────
    let (sx, sy, sz) = opts.spawn_position.unwrap_or((0, 64, 0));
    let mut spawn = NbtCompound::new();
    spawn.insert(
        "dimension",
        NbtTag::String("minecraft:overworld".to_string()),
    );
    spawn.insert("pos", NbtTag::IntArray(vec![sx, sy, sz]));
    spawn.insert("yaw", NbtTag::Float(0.0));
    spawn.insert("pitch", NbtTag::Float(0.0));
    data.insert("spawn", NbtTag::Compound(spawn));

    // ── Time settings ────────────────────────────────────────────────────
    data.insert("DayTime", NbtTag::Long(opts.day_time.unwrap_or(6000)));
    data.insert("Time", NbtTag::Long(0));

    // ── Timestamp ────────────────────────────────────────────────────────
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    data.insert("LastPlayed", NbtTag::Long(now_ms));

    // ── Weather ──────────────────────────────────────────────────────────
    data.insert("raining", NbtTag::Byte(0));
    data.insert("rainTime", NbtTag::Int(0));
    data.insert("thundering", NbtTag::Byte(0));
    data.insert("thunderTime", NbtTag::Int(0));
    data.insert("clearWeatherTime", NbtTag::Int(0));

    // ── Game rules (1.21+ format: minecraft: prefix, Byte/Int values) ───
    let mut game_rules = NbtCompound::new();
    game_rules.insert(
        "minecraft:advance_time",
        NbtTag::Byte(if opts.day_time.is_some() { 0 } else { 1 }),
    );
    game_rules.insert("minecraft:advance_weather", NbtTag::Byte(0));
    game_rules.insert("minecraft:block_drops", NbtTag::Byte(1));
    game_rules.insert("minecraft:command_blocks_work", NbtTag::Byte(1));
    game_rules.insert("minecraft:command_block_output", NbtTag::Byte(0));
    game_rules.insert("minecraft:drowning_damage", NbtTag::Byte(1));
    game_rules.insert("minecraft:fall_damage", NbtTag::Byte(1));
    game_rules.insert("minecraft:fire_damage", NbtTag::Byte(1));
    game_rules.insert("minecraft:fire_spread_radius_around_player", NbtTag::Int(0));
    game_rules.insert("minecraft:keep_inventory", NbtTag::Byte(1));
    game_rules.insert("minecraft:mob_griefing", NbtTag::Byte(0));
    game_rules.insert("minecraft:mob_drops", NbtTag::Byte(0));
    game_rules.insert("minecraft:natural_health_regeneration", NbtTag::Byte(1));
    game_rules.insert("minecraft:pvp", NbtTag::Byte(1));
    game_rules.insert("minecraft:random_tick_speed", NbtTag::Int(3));
    game_rules.insert("minecraft:send_command_feedback", NbtTag::Byte(1));
    game_rules.insert("minecraft:show_death_messages", NbtTag::Byte(1));
    game_rules.insert(
        "minecraft:spawn_mobs",
        NbtTag::Byte(if opts.disable_mob_spawning { 0 } else { 1 }),
    );
    game_rules.insert("minecraft:spawn_monsters", NbtTag::Byte(1));
    game_rules.insert("minecraft:spawn_patrols", NbtTag::Byte(0));
    game_rules.insert("minecraft:spawn_phantoms", NbtTag::Byte(0));
    game_rules.insert("minecraft:spawn_wandering_traders", NbtTag::Byte(0));
    game_rules.insert("minecraft:tnt_explodes", NbtTag::Byte(1));
    data.insert("game_rules", NbtTag::Compound(game_rules));

    // ── World generation settings (void/superflat) ───────────────────────
    if opts.void_world {
        let mut world_gen = NbtCompound::new();
        world_gen.insert("bonus_chest", NbtTag::Byte(0));
        world_gen.insert("seed", NbtTag::Long(0));
        world_gen.insert("generate_features", NbtTag::Byte(0));

        let mut dimensions = NbtCompound::new();

        // Overworld (flat/void)
        let mut overworld = NbtCompound::new();
        overworld.insert("type", NbtTag::String("minecraft:overworld".to_string()));
        let mut generator = NbtCompound::new();
        generator.insert("type", NbtTag::String("minecraft:flat".to_string()));
        let mut flat_settings = NbtCompound::new();
        flat_settings.insert("biome", NbtTag::String("minecraft:the_void".to_string()));
        flat_settings.insert("features", NbtTag::Byte(0));
        flat_settings.insert("lakes", NbtTag::Byte(0));
        flat_settings.insert("layers", NbtTag::List(NbtList::new()));
        flat_settings.insert("structure_overrides", NbtTag::List(NbtList::new()));
        generator.insert("settings", NbtTag::Compound(flat_settings));
        overworld.insert("generator", NbtTag::Compound(generator));
        dimensions.insert("minecraft:overworld", NbtTag::Compound(overworld));

        // The Nether
        let mut nether = NbtCompound::new();
        nether.insert("type", NbtTag::String("minecraft:the_nether".to_string()));
        let mut nether_gen = NbtCompound::new();
        nether_gen.insert("type", NbtTag::String("minecraft:noise".to_string()));
        nether_gen.insert("settings", NbtTag::String("minecraft:nether".to_string()));
        nether_gen.insert(
            "biome_source",
            NbtTag::Compound({
                let mut bs = NbtCompound::new();
                bs.insert("type", NbtTag::String("minecraft:multi_noise".to_string()));
                bs.insert("preset", NbtTag::String("minecraft:nether".to_string()));
                bs
            }),
        );
        nether.insert("generator", NbtTag::Compound(nether_gen));
        dimensions.insert("minecraft:the_nether", NbtTag::Compound(nether));

        // The End
        let mut end = NbtCompound::new();
        end.insert("type", NbtTag::String("minecraft:the_end".to_string()));
        let mut end_gen = NbtCompound::new();
        end_gen.insert("type", NbtTag::String("minecraft:noise".to_string()));
        end_gen.insert("settings", NbtTag::String("minecraft:end".to_string()));
        end_gen.insert(
            "biome_source",
            NbtTag::Compound({
                let mut bs = NbtCompound::new();
                bs.insert("type", NbtTag::String("minecraft:the_end".to_string()));
                bs
            }),
        );
        end.insert("generator", NbtTag::Compound(end_gen));
        dimensions.insert("minecraft:the_end", NbtTag::Compound(end));

        world_gen.insert("dimensions", NbtTag::Compound(dimensions));
        data.insert("WorldGenSettings", NbtTag::Compound(world_gen));
    }

    // ── Data packs ───────────────────────────────────────────────────────
    let mut data_packs = NbtCompound::new();
    data_packs.insert(
        "Enabled",
        NbtTag::List(NbtList::from(vec![NbtTag::String("vanilla".to_string())])),
    );
    data_packs.insert("Disabled", NbtTag::List(NbtList::new()));
    data.insert("DataPacks", NbtTag::Compound(data_packs));

    // ── Misc ─────────────────────────────────────────────────────────────
    data.insert(
        "ServerBrands",
        NbtTag::List(NbtList::from(vec![NbtTag::String("vanilla".to_string())])),
    );
    data.insert("WasModded", NbtTag::Byte(0));
    data.insert("WanderingTraderSpawnChance", NbtTag::Int(25));
    data.insert("WanderingTraderSpawnDelay", NbtTag::Int(24000));
    data.insert("ScheduledEvents", NbtTag::List(NbtList::new()));

    // Dragon fight defaults
    let mut dragon_fight = NbtCompound::new();
    dragon_fight.insert("DragonKilled", NbtTag::Byte(0));
    dragon_fight.insert("PreviouslyKilled", NbtTag::Byte(0));
    dragon_fight.insert("NeedsStateScanning", NbtTag::Byte(1));
    data.insert("DragonFight", NbtTag::Compound(dragon_fight));

    // ── Wrap in root "Data" compound ─────────────────────────────────────
    let mut root = NbtCompound::new();
    root.insert("Data", NbtTag::Compound(data));

    // ── Serialize and gzip compress ──────────────────────────────────────
    let mut nbt_bytes = Vec::new();
    quartz_nbt::io::write_nbt(&mut nbt_bytes, Some(""), &root, Flavor::Uncompressed)?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&nbt_bytes)?;
    Ok(encoder.finish()?)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_region_filename() {
        assert_eq!(parse_region_filename("r.0.0.mca"), Some((0, 0)));
        assert_eq!(parse_region_filename("r.1.-2.mca"), Some((1, -2)));
        assert_eq!(parse_region_filename("region/r.3.4.mca"), Some((3, 4)));
        assert_eq!(
            parse_region_filename("world/region/r.-1.0.mca"),
            Some((-1, 0))
        );
        assert_eq!(parse_region_filename("invalid.mca"), None);
    }

    #[test]
    fn test_world_export_options_default() {
        let opts = WorldExportOptions::default();
        assert_eq!(opts.world_name, "Nucleation Export");
        assert_eq!(opts.game_mode, 1);
        assert_eq!(opts.difficulty, 0);
        assert_eq!(opts.data_version, 4671);
        assert_eq!(opts.version_name, "1.21.4");
        assert!(opts.void_world);
        assert!(opts.allow_commands);
        assert_eq!(opts.day_time, Some(6000));
        assert!(opts.disable_mob_spawning);
    }

    #[test]
    fn test_world_export_options_from_json() {
        let json = r#"{"world_name": "Test World", "game_mode": 0}"#;
        let opts: WorldExportOptions = serde_json::from_str(json).unwrap();
        assert_eq!(opts.world_name, "Test World");
        assert_eq!(opts.game_mode, 0);
        // Defaults for unspecified fields
        assert!(opts.void_world);
    }

    #[test]
    fn test_roundtrip_schematic_to_world_to_schematic() {
        let mut schematic = UniversalSchematic::new("Test".to_string());
        let stone = BlockState::new("minecraft:stone".to_string());
        let redstone = BlockState::new("minecraft:redstone_wire".to_string())
            .with_property("power".to_string(), "15".to_string());

        schematic.set_block(0, 64, 0, &stone);
        schematic.set_block(1, 64, 0, &redstone);
        schematic.set_block(0, 64, 1, &stone);

        // Export to world
        let files = to_world(&schematic, None).unwrap();
        assert!(files.contains_key("level.dat"));
        assert!(files.contains_key("session.lock"));

        // Find the MCA file
        let mca_key = files.keys().find(|k| k.ends_with(".mca")).unwrap();
        let mca_data = files.get(mca_key).unwrap();

        // Import back
        let imported = from_mca(mca_data).unwrap();

        // Verify blocks
        let imported_stone = imported.default_region.get_block(0, 64, 0);
        assert!(imported_stone.is_some());
        assert_eq!(imported_stone.unwrap().name, "minecraft:stone");

        let imported_redstone = imported.default_region.get_block(1, 64, 0);
        assert!(imported_redstone.is_some());
        assert_eq!(imported_redstone.unwrap().name, "minecraft:redstone_wire");
        assert_eq!(
            imported_redstone.unwrap().properties.get("power"),
            Some(&"15".to_string())
        );
    }

    #[test]
    fn test_is_air() {
        assert!(is_air("minecraft:air"));
        assert!(is_air("minecraft:cave_air"));
        assert!(is_air("minecraft:void_air"));
        assert!(!is_air("minecraft:stone"));
    }

    #[test]
    fn test_level_dat_generation() {
        let opts = WorldExportOptions::default();
        let level_dat = generate_level_dat(&opts).unwrap();
        // Should be gzip compressed, starts with gzip magic bytes
        assert!(level_dat.len() > 10);
        assert_eq!(level_dat[0], 0x1f);
        assert_eq!(level_dat[1], 0x8b);
    }

    #[test]
    fn test_entity_roundtrip_through_world_export() {
        use crate::entity::{Entity, NbtValue};

        let mut schematic = UniversalSchematic::new("Entity Test".to_string());
        let stone = BlockState::new("minecraft:stone".to_string());
        schematic.set_block(0, 64, 0, &stone);

        // Add entities with various NBT data
        let mut creeper = Entity::new("minecraft:creeper".to_string(), (0.5, 64.0, 0.5));
        creeper
            .nbt
            .insert("Health".to_string(), NbtValue::Float(20.0));
        creeper.nbt.insert("Fuse".to_string(), NbtValue::Short(30));
        creeper
            .nbt
            .insert("ExplosionRadius".to_string(), NbtValue::Byte(3));
        schematic.default_region.add_entity(creeper);

        // Entity with passengers (riding)
        let mut pig = Entity::new("minecraft:pig".to_string(), (2.5, 64.0, 2.5));
        pig.nbt.insert("Health".to_string(), NbtValue::Float(10.0));

        let mut passenger_nbt = HashMap::new();
        passenger_nbt.insert(
            "id".to_string(),
            NbtValue::String("minecraft:zombie".to_string()),
        );
        passenger_nbt.insert("Health".to_string(), NbtValue::Float(20.0));
        let passenger_pos = NbtValue::List(vec![
            NbtValue::Double(2.5),
            NbtValue::Double(65.0),
            NbtValue::Double(2.5),
        ]);
        passenger_nbt.insert("Pos".to_string(), passenger_pos);

        pig.nbt.insert(
            "Passengers".to_string(),
            NbtValue::List(vec![NbtValue::Compound(passenger_nbt)]),
        );
        schematic.default_region.add_entity(pig);

        // Export to world
        let files = to_world(&schematic, None).unwrap();

        // Should have entity region files
        let entity_files: Vec<&String> = files
            .keys()
            .filter(|k| k.starts_with("entities/") && k.ends_with(".mca"))
            .collect();
        assert!(!entity_files.is_empty(), "Should have entity region files");

        // Import entity region files
        let mut imported = UniversalSchematic::new("Import".to_string());

        // Import blocks from region/*.mca
        for (path, data) in &files {
            if path.starts_with("region/") && path.ends_with(".mca") {
                let (rx, rz) = parse_region_filename(path).unwrap_or((0, 0));
                let mca = McaFile::from_bytes(data, rx, rz).unwrap();
                load_mca_into_schematic(&mca, &mut imported, None);
            }
        }

        // Import entities from entities/*.mca
        for (path, data) in &files {
            if path.starts_with("entities/") && path.ends_with(".mca") {
                let (rx, rz) = parse_region_filename(path).unwrap_or((0, 0));
                let entity_chunks = parse_entity_mca(data, rx, rz).unwrap();
                load_entities_into_schematic(&entity_chunks, &mut imported, None);
            }
        }

        // Verify entities were preserved
        assert_eq!(
            imported.default_region.entities.len(),
            2,
            "Should have 2 entities"
        );

        // Find the creeper
        let creeper = imported
            .default_region
            .entities
            .iter()
            .find(|e| e.id == "minecraft:creeper")
            .expect("Should have creeper");
        assert_eq!(creeper.nbt.get("Health"), Some(&NbtValue::Float(20.0)));
        assert_eq!(creeper.nbt.get("Fuse"), Some(&NbtValue::Short(30)));
        assert_eq!(creeper.nbt.get("ExplosionRadius"), Some(&NbtValue::Byte(3)));

        // Find the pig with passenger
        let pig = imported
            .default_region
            .entities
            .iter()
            .find(|e| e.id == "minecraft:pig")
            .expect("Should have pig");
        assert_eq!(pig.nbt.get("Health"), Some(&NbtValue::Float(10.0)));
        assert!(
            pig.nbt.contains_key("Passengers"),
            "Pig should have Passengers"
        );

        if let Some(NbtValue::List(passengers)) = pig.nbt.get("Passengers") {
            assert_eq!(passengers.len(), 1);
            if let NbtValue::Compound(p) = &passengers[0] {
                assert_eq!(
                    p.get("id"),
                    Some(&NbtValue::String("minecraft:zombie".to_string()))
                );
                assert_eq!(p.get("Health"), Some(&NbtValue::Float(20.0)));
            } else {
                panic!("Passenger should be a compound");
            }
        } else {
            panic!("Passengers should be a list");
        }
    }
}
