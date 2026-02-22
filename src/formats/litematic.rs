use crate::block_entity::BlockEntity;
use crate::entity::Entity;
use crate::region::Region;
use crate::{BlockState, UniversalSchematic};
use flate2::read::GzDecoder;
use quartz_nbt::io::Flavor;
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use std::io::{Cursor, Read};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn is_litematic(data: &[u8]) -> bool {
    // Stream-decompress directly into NBT parser (no intermediate buffer)
    let reader = std::io::BufReader::with_capacity(1 << 20, data);
    let mut gz = GzDecoder::new(reader);
    let (root, _) = match quartz_nbt::io::read_nbt(&mut gz, Flavor::Uncompressed) {
        Ok(result) => result,
        Err(_) => return false,
    };

    // Check for required fields as per the Litematic format
    root.get::<_, i32>("Version").is_ok()
        && root.get::<_, &NbtCompound>("Metadata").is_ok()
        && root.get::<_, &NbtCompound>("Regions").is_ok()
}
/// Default compression level for litematic serialization.
/// Level 3 balances speed (~2x faster than L6) with size (~15% larger than L6).
const DEFAULT_COMPRESSION: flate2::Compression = flate2::Compression::new(3);

pub fn to_litematic(schematic: &UniversalSchematic) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    to_litematic_with_compression(schematic, DEFAULT_COMPRESSION)
}

pub fn to_litematic_with_compression(
    schematic: &UniversalSchematic,
    compression: flate2::Compression,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut root = NbtCompound::new();

    // Add Version and SubVersion
    root.insert("Version", NbtTag::Int(6));
    root.insert("SubVersion", NbtTag::Int(1));

    // Add MinecraftDataVersion
    root.insert(
        "MinecraftDataVersion",
        NbtTag::Int(schematic.metadata.mc_version.unwrap_or(3700)),
    );

    // Add Metadata
    let metadata = create_metadata(schematic);
    root.insert("Metadata", NbtTag::Compound(metadata));

    // Add Regions
    let regions = create_regions(schematic);
    root.insert("Regions", NbtTag::Compound(regions));

    // Compress and return the NBT data
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), compression);
    quartz_nbt::io::write_nbt(
        &mut encoder,
        None,
        &root,
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(encoder.finish()?)
}

pub fn from_litematic(data: &[u8]) -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
    // Stream-decompress directly into NBT parser (no intermediate buffer)
    let reader = std::io::BufReader::with_capacity(1 << 20, data);
    let mut gz = flate2::read::GzDecoder::new(reader);
    let (root, _) = quartz_nbt::io::read_nbt(&mut gz, quartz_nbt::io::Flavor::Uncompressed)?;

    let mut schematic = UniversalSchematic::new("Unnamed".to_string());

    // Parse Metadata
    parse_metadata(&root, &mut schematic)?;

    // Parse Regions
    parse_regions(&root, &mut schematic)?;

    Ok(schematic)
}

fn create_metadata(schematic: &UniversalSchematic) -> NbtCompound {
    let mut metadata = NbtCompound::new();

    metadata.insert(
        "Name",
        NbtTag::String(schematic.metadata.name.clone().unwrap_or_default()),
    );
    metadata.insert(
        "Description",
        NbtTag::String(schematic.metadata.description.clone().unwrap_or_default()),
    );
    metadata.insert(
        "Author",
        NbtTag::String(schematic.metadata.author.clone().unwrap_or_default()),
    );

    // Get current time as milliseconds since epoch, safely handling both WASM and non-WASM environments
    let now = if let Some(time) = schematic.metadata.created {
        // Use existing timestamp if available
        time as i64
    } else {
        // Generate current timestamp based on platform
        #[cfg(all(feature = "wasm", target_arch = "wasm32"))]
        let current_time = js_sys::Date::now() as i64;

        #[cfg(not(all(feature = "wasm", target_arch = "wasm32")))]
        let current_time = {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64
        };

        current_time
    };

    // Use existing modified timestamp or fall back to creation time
    let modified = schematic.metadata.modified.unwrap_or(now as u64) as i64;

    metadata.insert("TimeCreated", NbtTag::Long(now));
    metadata.insert("TimeModified", NbtTag::Long(modified));

    // Use tight dimensions for EnclosingSize to avoid huge empty space
    let merged_region = schematic.get_merged_region();
    let (width, height, length) = if let Some(tight_bounds) = merged_region.get_tight_bounds() {
        tight_bounds.get_dimensions()
    } else {
        merged_region.get_dimensions()
    };

    let mut enclosing_size = NbtCompound::new();
    enclosing_size.insert("x", NbtTag::Int(width as i32));
    enclosing_size.insert("y", NbtTag::Int(height as i32));
    enclosing_size.insert("z", NbtTag::Int(length as i32));
    metadata.insert("EnclosingSize", NbtTag::Compound(enclosing_size));

    metadata.insert("TotalVolume", NbtTag::Int(schematic.total_volume() as i32));
    metadata.insert("TotalBlocks", NbtTag::Int(schematic.total_blocks() as i32));
    metadata.insert(
        "RegionCount",
        NbtTag::Int(schematic.other_regions.len() as i32 + 1),
    );

    metadata.insert("Software", NbtTag::String("UniversalSchematic".to_string()));

    // Add NucleationDefinitions if present
    if !schematic.definition_regions.is_empty() {
        if let Ok(json) = serde_json::to_string(&schematic.definition_regions) {
            metadata.insert("NucleationDefinitions", NbtTag::String(json));
        }
    }

    metadata
}
fn create_regions(schematic: &UniversalSchematic) -> NbtCompound {
    let mut regions = NbtCompound::new();

    for (name, region) in &schematic.get_all_regions() {
        // Use compact region to avoid huge empty space
        let compact_region = region.to_compact();

        let mut region_nbt = NbtCompound::new();

        // Position
        let mut position = NbtCompound::new();
        position.insert("x", NbtTag::Int(compact_region.position.0));
        position.insert("y", NbtTag::Int(compact_region.position.1));
        position.insert("z", NbtTag::Int(compact_region.position.2));
        region_nbt.insert("Position", NbtTag::Compound(position));

        // Size
        let mut size = NbtCompound::new();
        size.insert("x", NbtTag::Int(compact_region.size.0));
        size.insert("y", NbtTag::Int(compact_region.size.1));
        size.insert("z", NbtTag::Int(compact_region.size.2));
        region_nbt.insert("Size", NbtTag::Compound(size));

        // BlockStatePalette
        // Build reordered palette with air at index 0, and index mapping in one pass
        let mut reordered_palette: Vec<&BlockState> =
            Vec::with_capacity(compact_region.palette.len() + 1);
        let mut index_mapping = vec![0usize; compact_region.palette.len()];

        // Air always at index 0
        let air_state = BlockState::new("minecraft:air".to_string());
        let has_air = compact_region
            .palette
            .iter()
            .any(|b| b.name == "minecraft:air");
        // We'll use a reference to air from the palette or a local
        if has_air {
            // Find air in original palette and put a reference at index 0
            let air_ref = compact_region
                .palette
                .iter()
                .find(|b| b.name == "minecraft:air")
                .unwrap();
            reordered_palette.push(air_ref);
        } else {
            reordered_palette.push(&air_state);
        }

        // Add non-air blocks and build mapping simultaneously
        for (orig_idx, block) in compact_region.palette.iter().enumerate() {
            if block.name == "minecraft:air" {
                index_mapping[orig_idx] = 0;
            } else {
                index_mapping[orig_idx] = reordered_palette.len();
                reordered_palette.push(block);
            }
        }

        // Create the NBT list for the reordered palette
        let palette = NbtList::from(
            reordered_palette
                .iter()
                .map(|block_state| block_state.to_nbt())
                .collect::<Vec<NbtTag>>(),
        );
        region_nbt.insert("BlockStatePalette", NbtTag::List(palette));

        // Remap block indices and create packed states
        // Use integer log2 instead of floating-point
        let bits_per_block = if reordered_palette.len() <= 1 {
            2 // minimum 2 bits per block per litematic spec
        } else {
            std::cmp::max(
                (usize::BITS - (reordered_palette.len() - 1).leading_zeros()) as usize,
                2,
            )
        };
        let block_count = compact_region.blocks.len();
        let expected_len = (block_count * bits_per_block + 63) / 64;

        let mut packed_states = vec![0i64; expected_len];
        let mask = (1u64 << bits_per_block) - 1;

        if 64 % bits_per_block == 0 {
            // Fast path: entries never cross i64 boundaries (bits_per_block = 2, 4, 8, 16, 32)
            // Batch-pack without per-block division or branching
            let entries_per_long = 64 / bits_per_block;
            for (chunk_idx, chunk) in compact_region.blocks.chunks(entries_per_long).enumerate() {
                let mut packed: u64 = 0;
                for (i, &block_state) in chunk.iter().enumerate() {
                    packed |= (index_mapping[block_state] as u64 & mask) << (i * bits_per_block);
                }
                packed_states[chunk_idx] = packed as i64;
            }
        } else {
            // Slow path: entries may cross i64 boundaries (bits_per_block = 3, 5, 6, 7, etc.)
            for (index, &block_state) in compact_region.blocks.iter().enumerate() {
                let mapped_state = index_mapping[block_state];
                let bit_index = index * bits_per_block;
                let start_long_index = bit_index / 64;
                let end_long_index = (bit_index + bits_per_block - 1) / 64;
                let start_offset = bit_index % 64;
                let value = (mapped_state as u64) & mask;

                packed_states[start_long_index] |= (value << start_offset) as i64;
                if start_long_index != end_long_index {
                    packed_states[end_long_index] |= (value >> (64 - start_offset)) as i64;
                }
            }
        }

        region_nbt.insert("BlockStates", NbtTag::LongArray(packed_states));

        // Entities - Litematic stores entity positions relative to region Position
        let entities = NbtList::from(
            compact_region
                .entities
                .iter()
                .map(|entity| {
                    let mut entity_nbt = if let NbtTag::Compound(c) = entity.to_nbt() {
                        c
                    } else {
                        NbtCompound::new()
                    };

                    let rel_x = entity.position.0 - compact_region.position.0 as f64;
                    let rel_y = entity.position.1 - compact_region.position.1 as f64;
                    let rel_z = entity.position.2 - compact_region.position.2 as f64;

                    let pos_list = NbtList::from(vec![
                        NbtTag::Double(rel_x),
                        NbtTag::Double(rel_y),
                        NbtTag::Double(rel_z),
                    ]);
                    entity_nbt.insert("Pos", NbtTag::List(pos_list));

                    NbtTag::Compound(entity_nbt)
                })
                .collect::<Vec<NbtTag>>(),
        );
        region_nbt.insert("Entities", NbtTag::List(entities));

        // TileEntities - Litematic stores block entity coordinates relative to region Position
        let tile_entities = NbtList::from(
            compact_region
                .block_entities
                .values()
                .map(|block_entity| {
                    let mut block_entity_nbt = block_entity.to_nbt();

                    let rel_x = block_entity.position.0 - compact_region.position.0;
                    let rel_y = block_entity.position.1 - compact_region.position.1;
                    let rel_z = block_entity.position.2 - compact_region.position.2;

                    block_entity_nbt.insert("x", NbtTag::Int(rel_x));
                    block_entity_nbt.insert("y", NbtTag::Int(rel_y));
                    block_entity_nbt.insert("z", NbtTag::Int(rel_z));
                    block_entity_nbt.insert("Pos", NbtTag::IntArray(vec![rel_x, rel_y, rel_z]));

                    NbtTag::Compound(block_entity_nbt)
                })
                .collect::<Vec<NbtTag>>(),
        );
        region_nbt.insert("TileEntities", NbtTag::List(tile_entities));

        // PendingBlockTicks and PendingFluidTicks (not fully supported, using empty lists)
        region_nbt.insert("PendingBlockTicks", NbtTag::List(NbtList::new()));
        region_nbt.insert("PendingFluidTicks", NbtTag::List(NbtList::new()));

        regions.insert(name, NbtTag::Compound(region_nbt));
    }

    regions
}

fn parse_metadata(
    root: &NbtCompound,
    schematic: &mut UniversalSchematic,
) -> Result<(), Box<dyn std::error::Error>> {
    let metadata = root.get::<_, &NbtCompound>("Metadata")?;

    schematic.metadata.name = metadata.get::<_, &str>("Name").ok().map(String::from);
    schematic.metadata.description = metadata
        .get::<_, &str>("Description")
        .ok()
        .map(String::from);
    schematic.metadata.author = metadata.get::<_, &str>("Author").ok().map(String::from);
    schematic.metadata.created = metadata.get::<_, i64>("TimeCreated").ok().map(|t| t as u64);
    schematic.metadata.modified = metadata
        .get::<_, i64>("TimeModified")
        .ok()
        .map(|t| t as u64);

    // We don't need to parse EnclosingSize, TotalVolume, TotalBlocks as they will be recalculated

    // Parse NucleationDefinitions
    if let Ok(json) = metadata.get::<_, &str>("NucleationDefinitions") {
        if let Ok(regions) = serde_json::from_str(json) {
            schematic.definition_regions = regions;
        }
    }

    Ok(())
}

fn parse_regions(
    root: &NbtCompound,
    schematic: &mut UniversalSchematic,
) -> Result<(), Box<dyn std::error::Error>> {
    let regions = root.get::<_, &NbtCompound>("Regions")?;
    let mut loop_count = 0;
    for (name, region_tag) in regions.inner() {
        //if it's the first region we want to override the default region name
        if loop_count == 0 {
            schematic.default_region_name = name.clone();
        }
        loop_count += 1;

        if let NbtTag::Compound(region_nbt) = region_tag {
            let position = region_nbt.get::<_, &NbtCompound>("Position")?;
            let size = region_nbt.get::<_, &NbtCompound>("Size")?;

            let position = (
                position.get::<_, i32>("x")?,
                position.get::<_, i32>("y")?,
                position.get::<_, i32>("z")?,
            );
            let size = (
                size.get::<_, i32>("x")?,
                size.get::<_, i32>("y")?,
                size.get::<_, i32>("z")?,
            );

            let mut region = Region::new(name.to_string(), position, size);

            // Parse BlockStatePalette
            let palette = region_nbt.get::<_, &NbtList>("BlockStatePalette")?;
            region.palette = palette
                .iter()
                .filter_map(|tag| {
                    if let NbtTag::Compound(compound) = tag {
                        BlockState::from_nbt(compound).ok()
                    } else {
                        None
                    }
                })
                .collect();

            // Parse BlockStates
            let block_states = region_nbt.get::<_, &[i64]>("BlockStates")?;
            // region.unpack_block_states(block_states);
            region.blocks = region.unpack_block_states(block_states);

            // Rebuild caches after directly setting palette and blocks
            region.rebuild_palette_index();
            region.rebuild_air_index();
            region.rebuild_non_air_count();
            region.rebuild_tight_bounds();

            // Parse Entities - Litematic stores positions relative to region Position
            if let Ok(entities_list) = region_nbt.get::<_, &NbtList>("Entities") {
                region.entities = entities_list
                    .iter()
                    .filter_map(|tag| {
                        if let NbtTag::Compound(compound) = tag {
                            let mut entity = Entity::from_nbt(compound).ok()?;
                            // Convert from relative to absolute position
                            entity.position.0 += position.0 as f64;
                            entity.position.1 += position.1 as f64;
                            entity.position.2 += position.2 as f64;
                            Some(entity)
                        } else {
                            None
                        }
                    })
                    .collect();
            }

            // Parse TileEntities - Litematic stores positions relative to region Position
            if let Ok(tile_entities_list) = region_nbt.get::<_, &NbtList>("TileEntities") {
                for tag in tile_entities_list.iter() {
                    if let NbtTag::Compound(compound) = tag {
                        let mut block_entity = BlockEntity::from_nbt(compound);
                        // Convert from relative to absolute position
                        block_entity.position.0 += position.0;
                        block_entity.position.1 += position.1;
                        block_entity.position.2 += position.2;
                        region
                            .block_entities
                            .insert(block_entity.position, block_entity);
                    }
                }
            }

            schematic.add_region(region);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlockState, UniversalSchematic};
    use num_complex::Complex;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_create_metadata() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());
        schematic.metadata.author = Some("Test Author".to_string());
        schematic.metadata.description = Some("Test Description".to_string());
        schematic.metadata.created = Some(1000);
        schematic.metadata.modified = Some(2000);

        let metadata = create_metadata(&schematic);

        assert_eq!(metadata.get::<_, &str>("Name").unwrap(), "Test Schematic");
        assert_eq!(metadata.get::<_, &str>("Author").unwrap(), "Test Author");
        assert_eq!(
            metadata.get::<_, &str>("Description").unwrap(),
            "Test Description"
        );
        assert_eq!(metadata.get::<_, i64>("TimeCreated").unwrap(), 1000);
        assert_eq!(metadata.get::<_, i64>("TimeModified").unwrap(), 2000);
        assert!(metadata.contains_key("EnclosingSize"));
        assert!(metadata.contains_key("TotalVolume"));
        assert!(metadata.contains_key("TotalBlocks"));
        assert!(metadata.contains_key("RegionCount"));
        assert_eq!(
            metadata.get::<_, &str>("Software").unwrap(),
            "UniversalSchematic"
        );
    }

    #[test]
    fn test_create_regions() {
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());
        let mut region = Region::new("TestRegion".to_string(), (0, 0, 0), (2, 2, 2));

        let stone = BlockState::new("minecraft:stone".to_string());
        let _air = BlockState::new("minecraft:air".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 1, 1, &stone);

        let entity = Entity::new("minecraft:creeper".to_string(), (0.5, 0.0, 0.5));
        region.add_entity(entity);

        let block_entity = BlockEntity::new("minecraft:chest".to_string(), (0, 1, 0));
        region.add_block_entity(block_entity);

        schematic.add_region(region);

        let regions = create_regions(&schematic);

        assert!(regions.contains_key("TestRegion"));
        let region_nbt = regions.get::<_, &NbtCompound>("TestRegion").unwrap();

        assert!(region_nbt.contains_key("Position"));
        assert!(region_nbt.contains_key("Size"));
        assert!(region_nbt.contains_key("BlockStatePalette"));
        assert!(region_nbt.contains_key("BlockStates"));
        assert!(region_nbt.contains_key("Entities"));
        assert!(region_nbt.contains_key("TileEntities"));
        assert!(region_nbt.contains_key("PendingBlockTicks"));
        assert!(region_nbt.contains_key("PendingFluidTicks"));
    }

    #[test]
    fn test_parse_metadata() {
        let mut root = NbtCompound::new();
        let mut metadata = NbtCompound::new();
        metadata.insert("Name", NbtTag::String("Test Schematic".to_string()));
        metadata.insert("Author", NbtTag::String("Test Author".to_string()));
        metadata.insert(
            "Description",
            NbtTag::String("Test Description".to_string()),
        );
        metadata.insert("TimeCreated", NbtTag::Long(1000));
        metadata.insert("TimeModified", NbtTag::Long(2000));
        root.insert("Metadata", NbtTag::Compound(metadata));

        let mut schematic = UniversalSchematic::new("".to_string());
        parse_metadata(&root, &mut schematic).unwrap();

        assert_eq!(schematic.metadata.name, Some("Test Schematic".to_string()));
        assert_eq!(schematic.metadata.author, Some("Test Author".to_string()));
        assert_eq!(
            schematic.metadata.description,
            Some("Test Description".to_string())
        );
        assert_eq!(schematic.metadata.created, Some(1000));
        assert_eq!(schematic.metadata.modified, Some(2000));
    }

    #[test]
    fn test_parse_regions() {
        let mut root = NbtCompound::new();
        let mut regions = NbtCompound::new();
        let mut region = NbtCompound::new();

        let mut position = NbtCompound::new();
        position.insert("x", NbtTag::Int(0));
        position.insert("y", NbtTag::Int(0));
        position.insert("z", NbtTag::Int(0));
        region.insert("Position", NbtTag::Compound(position));

        let mut size = NbtCompound::new();
        size.insert("x", NbtTag::Int(2));
        size.insert("y", NbtTag::Int(2));
        size.insert("z", NbtTag::Int(2));
        region.insert("Size", NbtTag::Compound(size));

        let palette = NbtList::from(vec![
            BlockState::new("minecraft:air".to_string()).to_nbt(),
            BlockState::new("minecraft:stone".to_string()).to_nbt(),
        ]);
        region.insert("BlockStatePalette", NbtTag::List(palette));

        // 2x2x2 region with 2 stone blocks and 6 air blocks
        region.insert("BlockStates", NbtTag::LongArray(vec![0b10000001]));

        regions.insert("TestRegion", NbtTag::Compound(region));
        root.insert("Regions", NbtTag::Compound(regions));

        println!("{:?}", root);

        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());
        parse_regions(&root, &mut schematic).unwrap();

        assert_eq!(schematic.default_region_name, "TestRegion");

        let parsed_region = schematic.default_region;
        assert_eq!(parsed_region.position, (0, 0, 0));
        assert_eq!(parsed_region.size, (2, 2, 2));
        assert_eq!(parsed_region.palette.len(), 2);
        assert_eq!(parsed_region.count_blocks(), 2); // 2 stone blocks
    }
    #[test]
    fn test_simple_litematic() {
        let mut schematic = UniversalSchematic::new("Simple Cube".to_string());
        schematic.metadata.created = Some(1000);
        schematic.metadata.modified = Some(2000);
        // Create a 3x3x3 cube
        for x in 0..3 {
            for y in 0..3 {
                for z in 0..3 {
                    let block = match (x + y + z) % 3 {
                        0 => BlockState::new("minecraft:stone".to_string()),
                        1 => BlockState::new("minecraft:dirt".to_string()),
                        _ => BlockState::new("minecraft:oak_planks".to_string()),
                    };
                    schematic.set_block(x, y, z, &block);
                }
            }
        }

        // Set metadata
        schematic.metadata.author = Some("Test Author".to_string());
        schematic.metadata.description = Some("A simple 3x3x3 cube for testing".to_string());

        // Convert the schematic to .litematic format
        let litematic_data =
            to_litematic(&schematic).expect("Failed to convert schematic to litematic");

        // Save the .litematic file
        let mut file = File::create("simple_cube.litematic").expect("Failed to create file");
        file.write_all(&litematic_data)
            .expect("Failed to write to file");

        // Read the .litematic file back
        let _loaded_litematic_data =
            std::fs::read("simple_cube.litematic").expect("Failed to read file");

        // Clean up the generated file
        //std::fs::remove_file("simple_cube.litematic").expect("Failed to remove file");
    }

    #[test]
    fn test_litematic_roundtrip() {
        let mut original_schematic = UniversalSchematic::new("Test Schematic".to_string());
        original_schematic.metadata.created = Some(1000);
        original_schematic.metadata.modified = Some(2000);
        let mut region = Region::new("TestRegion".to_string(), (0, 0, 0), (2, 2, 2));

        let stone = BlockState::new("minecraft:stone".to_string());
        let _air = BlockState::new("minecraft:air".to_string());

        region.set_block(0, 0, 0, &stone);
        region.set_block(1, 1, 1, &stone);

        original_schematic.add_region(region);

        // Convert to Litematic
        let litematic_data = to_litematic(&original_schematic).unwrap();

        // Convert back from Litematic
        let roundtrip_schematic = from_litematic(&litematic_data).unwrap();

        // Compare original and roundtrip schematics
        assert_eq!(
            original_schematic.metadata.name,
            roundtrip_schematic.metadata.name
        );
        assert_eq!(
            original_schematic.other_regions.len(),
            roundtrip_schematic.other_regions.len()
        );

        // Compare the "TestRegion" instead of the default region
        let original_region = original_schematic.get_region("TestRegion").unwrap();
        let roundtrip_region = roundtrip_schematic.get_region("TestRegion").unwrap();

        assert_eq!(original_region.position, roundtrip_region.position);
        assert_eq!(original_region.size, roundtrip_region.size);
        assert_eq!(
            original_region.count_blocks(),
            roundtrip_region.count_blocks()
        );

        // Check if blocks are in the same positions
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    assert_eq!(
                        original_region.get_block(x, y, z),
                        roundtrip_region.get_block(x, y, z)
                    );
                }
            }
        }
    }

    /// Test that litematic export stores block entity and entity positions
    /// relative to the region Position, not as absolute coordinates.
    #[test]
    fn test_litematic_relative_positions_in_nbt() {
        let mut schematic = UniversalSchematic::new("RelativePositionTest".to_string());
        schematic.metadata.created = Some(1000);
        schematic.metadata.modified = Some(2000);

        // Place blocks and block entities at non-zero positions
        // so the compact region will have a non-zero position offset
        let stone = BlockState::new("minecraft:stone".to_string());
        schematic.set_block(10, 20, 30, &stone);
        schematic.set_block(11, 21, 31, &stone);

        // Add a block entity at absolute position (10, 20, 30)
        let block_entity = BlockEntity::new("minecraft:chest".to_string(), (10, 20, 30));
        schematic.default_region.add_block_entity(block_entity);

        // Add an entity at absolute position (10.5, 20.0, 30.5)
        let entity = Entity::new("minecraft:creeper".to_string(), (10.5, 20.0, 30.5));
        schematic.default_region.add_entity(entity);

        // Export to litematic
        let litematic_data = to_litematic(&schematic).unwrap();

        // Parse back the raw NBT to inspect stored positions
        let mut decoder = flate2::read::GzDecoder::new(litematic_data.as_slice());
        let mut decompressed = Vec::new();
        std::io::Read::read_to_end(&mut decoder, &mut decompressed).unwrap();
        let (root, _) = quartz_nbt::io::read_nbt(
            &mut std::io::Cursor::new(decompressed),
            quartz_nbt::io::Flavor::Uncompressed,
        )
        .unwrap();

        let regions = root.get::<_, &NbtCompound>("Regions").unwrap();
        // Get the first (and only) region
        let (_, region_tag) = regions.inner().iter().next().unwrap();
        let region_nbt = if let NbtTag::Compound(c) = region_tag {
            c
        } else {
            panic!("Expected compound")
        };

        // Get the region position (should be the compact region min)
        let pos_nbt = region_nbt.get::<_, &NbtCompound>("Position").unwrap();
        let region_x = pos_nbt.get::<_, i32>("x").unwrap();
        let region_y = pos_nbt.get::<_, i32>("y").unwrap();
        let region_z = pos_nbt.get::<_, i32>("z").unwrap();

        // Region position should be at the tight bounds min (10, 20, 30)
        assert_eq!(region_x, 10);
        assert_eq!(region_y, 20);
        assert_eq!(region_z, 30);

        // Check block entity positions are RELATIVE to region position
        let tile_entities = region_nbt.get::<_, &NbtList>("TileEntities").unwrap();
        assert_eq!(tile_entities.len(), 1);
        if let NbtTag::Compound(be_nbt) = &tile_entities[0] {
            // Block entity at absolute (10,20,30) with region at (10,20,30)
            // should be stored as relative (0,0,0)
            let pos = be_nbt.get::<_, &[i32]>("Pos").unwrap();
            assert_eq!(
                pos,
                &[0, 0, 0],
                "Block entity position should be relative to region, got {:?}",
                pos
            );

            assert_eq!(be_nbt.get::<_, i32>("x").unwrap(), 0);
            assert_eq!(be_nbt.get::<_, i32>("y").unwrap(), 0);
            assert_eq!(be_nbt.get::<_, i32>("z").unwrap(), 0);
        } else {
            panic!("Expected compound tag for block entity");
        }

        // Check entity positions are RELATIVE to region position
        let entities = region_nbt.get::<_, &NbtList>("Entities").unwrap();
        assert_eq!(entities.len(), 1);
        if let NbtTag::Compound(ent_nbt) = &entities[0] {
            let pos = ent_nbt.get::<_, &NbtList>("Pos").unwrap();
            // Entity at absolute (10.5, 20.0, 30.5) with region at (10, 20, 30)
            // should be stored as relative (0.5, 0.0, 0.5)
            let x = pos.get::<f64>(0).unwrap();
            let y = pos.get::<f64>(1).unwrap();
            let z = pos.get::<f64>(2).unwrap();
            assert!(
                (x - 0.5).abs() < 0.001,
                "Entity X should be 0.5 relative, got {}",
                x
            );
            assert!(
                (y - 0.0).abs() < 0.001,
                "Entity Y should be 0.0 relative, got {}",
                y
            );
            assert!(
                (z - 0.5).abs() < 0.001,
                "Entity Z should be 0.5 relative, got {}",
                z
            );
        } else {
            panic!("Expected compound tag for entity");
        }
    }

    /// Test that litematic roundtrip preserves absolute positions of block entities
    /// and entities even when the region has a non-zero position offset.
    #[test]
    fn test_litematic_roundtrip_with_offset_positions() {
        let mut schematic = UniversalSchematic::new("OffsetRoundtrip".to_string());
        schematic.metadata.created = Some(1000);
        schematic.metadata.modified = Some(2000);

        let stone = BlockState::new("minecraft:stone".to_string());
        // Place blocks at offset positions
        for x in 10..13 {
            for y in 20..23 {
                for z in 30..33 {
                    schematic.set_block(x, y, z, &stone);
                }
            }
        }

        // Add block entities at specific positions
        let chest1 = BlockEntity::new("minecraft:chest".to_string(), (10, 20, 30));
        let chest2 = BlockEntity::new("minecraft:chest".to_string(), (12, 22, 32));
        schematic.default_region.add_block_entity(chest1);
        schematic.default_region.add_block_entity(chest2);

        // Add entities
        let creeper = Entity::new("minecraft:creeper".to_string(), (11.5, 21.0, 31.5));
        schematic.default_region.add_entity(creeper);

        // Roundtrip
        let litematic_data = to_litematic(&schematic).unwrap();
        let roundtrip = from_litematic(&litematic_data).unwrap();

        let rt_region = roundtrip
            .get_region(&roundtrip.default_region_name)
            .unwrap();

        // Block entities should preserve absolute positions
        assert_eq!(rt_region.block_entities.len(), 2);
        assert!(
            rt_region.block_entities.contains_key(&(10, 20, 30)),
            "Block entity at (10,20,30) should exist after roundtrip"
        );
        assert!(
            rt_region.block_entities.contains_key(&(12, 22, 32)),
            "Block entity at (12,22,32) should exist after roundtrip"
        );

        // Entities should preserve absolute positions
        assert_eq!(rt_region.entities.len(), 1);
        let rt_entity = &rt_region.entities[0];
        assert!(
            (rt_entity.position.0 - 11.5).abs() < 0.001,
            "Entity X should be 11.5, got {}",
            rt_entity.position.0
        );
        assert!(
            (rt_entity.position.1 - 21.0).abs() < 0.001,
            "Entity Y should be 21.0, got {}",
            rt_entity.position.1
        );
        assert!(
            (rt_entity.position.2 - 31.5).abs() < 0.001,
            "Entity Z should be 31.5, got {}",
            rt_entity.position.2
        );

        // Blocks should also be preserved
        for x in 10..13 {
            for y in 20..23 {
                for z in 30..33 {
                    assert_eq!(
                        rt_region.get_block(x, y, z).map(|b| b.name.as_str()),
                        Some("minecraft:stone"),
                        "Block at ({},{},{}) should be stone",
                        x,
                        y,
                        z
                    );
                }
            }
        }
    }
}

use crate::formats::manager::{SchematicExporter, SchematicImporter};

pub struct LitematicFormat;

impl SchematicImporter for LitematicFormat {
    fn name(&self) -> String {
        "litematic".to_string()
    }

    fn detect(&self, data: &[u8]) -> bool {
        is_litematic(data)
    }

    fn read(&self, data: &[u8]) -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
        from_litematic(data)
    }
}

impl SchematicExporter for LitematicFormat {
    fn name(&self) -> String {
        "litematic".to_string()
    }

    fn extensions(&self) -> Vec<String> {
        vec!["litematic".to_string()]
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
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        to_litematic(schematic)
    }
}
