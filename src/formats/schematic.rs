use std::fmt;
use std::io::{BufReader, Cursor, Read};

use crate::block_entity::BlockEntity;
use crate::entity::Entity;
use crate::region::Region;
use crate::{BlockState, UniversalSchematic};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use quartz_nbt::io::{read_nbt, Flavor};
use quartz_nbt::{NbtCompound, NbtList, NbtTag};

#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

#[cfg(feature = "wasm")]
use web_sys::console;

// enum for versions of schematics
#[derive(Debug, Clone, Copy)]
pub enum SchematicVersion {
    V2,
    V3,
}

impl SchematicVersion {
    pub fn as_str(&self) -> &str {
        match self {
            SchematicVersion::V2 => "v2",
            SchematicVersion::V3 => "v3",
        }
    }

    pub fn from_str(version: &str) -> Option<SchematicVersion> {
        match version {
            "v2" => Some(SchematicVersion::V2),
            "v3" => Some(SchematicVersion::V3),
            _ => None,
        }
    }

    pub fn get_default() -> SchematicVersion {
        SchematicVersion::V3
    }

    pub fn get_all() -> Vec<SchematicVersion> {
        vec![SchematicVersion::V2, SchematicVersion::V3]
    }
}
impl fmt::Display for SchematicVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn is_schematic(data: &[u8]) -> bool {
    // Decompress the data
    let reader = BufReader::with_capacity(1 << 20, data); // 1 MiB buf
    let mut gz = GzDecoder::new(reader);
    let (root, _) = match read_nbt(&mut gz, Flavor::Uncompressed) {
        Ok(result) => result,
        Err(_) => {
            #[cfg(feature = "wasm")]
            let _: Result<(), JsValue> = Err(JsValue::from_str("Failed to read NBT data"));
            return false;
        }
    };

    //things should be under Schematic tag if not treat root as the schematic
    let root = root.get::<_, &NbtCompound>("Schematic").unwrap_or(&root);

    // get tge version of the schematic
    let version = root.get::<_, i32>("Version");
    #[cfg(feature = "wasm")]
    console::log_1(&format!("Schematic Version: {:?}", version).into());
    if version.is_err() {
        return root.get::<_, &NbtCompound>("Blocks").is_ok();
    }

    // Check if it's a v3 schematic (which has a Blocks compound)
    if version.unwrap() == 3 {
        #[cfg(feature = "wasm")]
        console::log_1(&format!("Detected v3 schematic").into());
        return root.get::<_, &NbtCompound>("Blocks").is_ok();
    }

    // Otherwise check for v2 format
    root.get::<_, i32>("DataVersion").is_ok()
        && root.get::<_, i16>("Width").is_ok()
        && root.get::<_, i16>("Height").is_ok()
        && root.get::<_, i16>("Length").is_ok()
        && root.get::<_, &Vec<i8>>("BlockData").is_ok()
}

// Default function uses v3 format
pub fn to_schematic(schematic: &UniversalSchematic) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    to_schematic_version(schematic, SchematicVersion::get_default())
}

pub fn to_schematic_version(
    schematic: &UniversalSchematic,
    version: SchematicVersion,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match version {
        SchematicVersion::V2 => to_schematic_v2(schematic),
        SchematicVersion::V3 => to_schematic_v3(schematic),
    }
}

// Version 3 format (recommended)
pub fn to_schematic_v3(
    schematic: &UniversalSchematic,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut schematic_data = NbtCompound::new();

    // Version 3 format
    schematic_data.insert("Version", NbtTag::Int(3));
    schematic_data.insert(
        "DataVersion",
        NbtTag::Int(schematic.metadata.mc_version.unwrap_or(1343)),
    );

    // Use compact region with tight bounds for export to avoid huge empty space
    let merged_region = schematic.get_merged_region();
    let compact_region = merged_region.to_compact();

    let (width, height, length) = compact_region.get_dimensions();
    let offset_pos = compact_region.position;

    schematic_data.insert("Width", NbtTag::Short((width as i16).abs()));
    schematic_data.insert("Height", NbtTag::Short((height as i16).abs()));
    schematic_data.insert("Length", NbtTag::Short((length as i16).abs()));

    // Set offset to the minimum position of the compact region
    let offset = vec![offset_pos.0, offset_pos.1, offset_pos.2];
    schematic_data.insert("Offset", NbtTag::IntArray(offset));

    // Create the Blocks container (required in v3)
    let mut blocks_container = NbtCompound::new();

    // Create clean palette and mapping from compact region
    let (palette_nbt, palette_mapping) = convert_palette_with_mapping(&compact_region.palette);

    // Store palette size before moving palette_nbt
    let _palette_size = palette_nbt.len();
    blocks_container.insert("Palette", palette_nbt);

    // Remap block data using the new palette mapping
    let remapped_blocks: Vec<u32> = compact_region
        .blocks
        .iter()
        .map(|&original_id| {
            if original_id < palette_mapping.len() {
                palette_mapping[original_id] as u32
            } else {
                // Out of bounds - map to air and log warning if debugging
                #[cfg(feature = "wasm")]
                console::log_1(
                    &format!(
                        "Warning: Block index {} out of bounds (palette size: {}), mapping to air",
                        original_id,
                        palette_mapping.len()
                    )
                    .into(),
                );
                0 // Default to air for out-of-bounds indices
            }
        })
        .collect();

    // Encode remapped block data
    let block_data: Vec<u8> = remapped_blocks
        .iter()
        .flat_map(|&block_id| encode_varint(block_id))
        .collect();

    // Add block data to Blocks container (renamed from "BlockData" to "Data" in v3)
    blocks_container.insert(
        "Data",
        NbtTag::ByteArray(block_data.iter().map(|&x| x as i8).collect()),
    );

    // Add block entities from compact region (using v3 format)
    let block_entities = convert_block_entities_v3(&compact_region);
    blocks_container.insert("BlockEntities", NbtTag::List(block_entities));

    // Entities from compact region remain at root level in v3 - with validation
    let mut entities = NbtList::new();
    let region_entities = convert_entities(&compact_region);

    // Only add valid entities
    for entity in region_entities.iter() {
        if let NbtTag::Compound(compound) = entity {
            // Validate that entity has required fields
            if compound.contains_key("Id") && compound.contains_key("Pos") {
                entities.push(entity.clone());
            } else {
                #[cfg(feature = "wasm")]
                console::log_1(&"Warning: Skipping invalid entity missing Id or Pos".into());
            }
        }
    }

    // Add the Blocks container to schematic data
    schematic_data.insert("Blocks", NbtTag::Compound(blocks_container));

    schematic_data.insert("Entities", NbtTag::List(entities));

    // Add metadata
    schematic_data.insert("Metadata", schematic.metadata.to_nbt());

    // Create the proper root structure with "Schematic" tag
    let mut root = NbtCompound::new();
    root.insert("Schematic", NbtTag::Compound(schematic_data));

    // Write NBT with proper compression
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    quartz_nbt::io::write_nbt(
        &mut encoder,
        None,
        &root,
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(encoder.finish()?)
}

// Version 2 format (legacy compatibility)
pub fn to_schematic_v2(
    schematic: &UniversalSchematic,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut schematic_data = NbtCompound::new();

    schematic_data.insert("Version", NbtTag::Int(2)); // Schematic format version 2
    schematic_data.insert(
        "DataVersion",
        NbtTag::Int(schematic.metadata.mc_version.unwrap_or(1343)),
    );

    // Use compact region with tight bounds for export
    let merged_region = schematic.get_merged_region();
    let compact_region = merged_region.to_compact();

    let (width, height, length) = compact_region.get_dimensions();
    let offset_pos = compact_region.position;

    schematic_data.insert("Width", NbtTag::Short((width as i16).abs()));
    schematic_data.insert("Height", NbtTag::Short((height as i16).abs()));
    schematic_data.insert("Length", NbtTag::Short((length as i16).abs()));

    schematic_data.insert(
        "Size",
        NbtTag::IntArray(vec![width as i32, height as i32, length as i32]),
    );

    // Set offset to the minimum position of the compact region
    let offset = vec![offset_pos.0, offset_pos.1, offset_pos.2];
    schematic_data.insert("Offset", NbtTag::IntArray(offset));

    schematic_data.insert("Palette", convert_palette_v2(&compact_region.palette).0);
    schematic_data.insert(
        "PaletteMax",
        convert_palette_v2(&compact_region.palette).1 + 1,
    );

    let block_data: Vec<u8> = compact_region
        .blocks
        .iter()
        .flat_map(|&block_id| encode_varint(block_id as u32))
        .collect();

    schematic_data.insert(
        "BlockData",
        NbtTag::ByteArray(block_data.iter().map(|&x| x as i8).collect()),
    );

    // Use block entities and entities from compact region
    let block_entities = convert_block_entities(&compact_region);
    let entities = convert_entities(&compact_region);

    schematic_data.insert("BlockEntities", NbtTag::List(block_entities));
    schematic_data.insert("Entities", NbtTag::List(entities));

    schematic_data.insert("Metadata", schematic.metadata.to_nbt());

    // Create the proper root structure with "Schematic" tag
    let mut root = NbtCompound::new();
    root.insert("Schematic", NbtTag::Compound(schematic_data));

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    quartz_nbt::io::write_nbt(
        &mut encoder,
        None,
        &root,
        quartz_nbt::io::Flavor::Uncompressed,
    )?;
    Ok(encoder.finish()?)
}

// Palette conversion for v3 (creates clean sequential indices)
fn convert_palette(palette: &Vec<BlockState>) -> (NbtCompound, i32) {
    let (nbt_palette, _) = convert_palette_with_mapping(palette);
    let max_id = nbt_palette.len() as i32 - 1;
    (nbt_palette, max_id)
}

// Helper function that returns both palette and mapping for index conversion
fn convert_palette_with_mapping(palette: &Vec<BlockState>) -> (NbtCompound, Vec<i32>) {
    let mut nbt_palette = NbtCompound::new();
    let mut mapping = vec![0i32; palette.len()]; // Default all to air (index 0)

    // Always start with air at index 0
    nbt_palette.insert("minecraft:air", NbtTag::Int(0));
    let mut next_id = 1;

    for (original_id, block_state) in palette.iter().enumerate() {
        // Handle invalid or unknown blocks by mapping them to air
        if block_state.name.is_empty() || block_state.name == "minecraft:unknown" {
            mapping[original_id] = 0; // Map to air
            continue;
        }

        // If it's already air, map to index 0
        if block_state.name == "minecraft:air" {
            mapping[original_id] = 0;
            continue;
        }

        let key = if block_state.properties.is_empty() {
            block_state.name.clone()
        } else {
            format!(
                "{}[{}]",
                block_state.name,
                block_state
                    .properties
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };

        // Check if this block state already exists in the palette
        let mut found_id = None;
        for (existing_key, tag) in nbt_palette.inner() {
            if existing_key == &key {
                if let NbtTag::Int(id) = tag {
                    found_id = Some(*id);
                    break;
                }
            }
        }

        let assigned_id = if let Some(id) = found_id {
            id
        } else {
            nbt_palette.insert(&key, NbtTag::Int(next_id));
            let id = next_id;
            next_id += 1;
            id
        };

        mapping[original_id] = assigned_id;
    }

    (nbt_palette, mapping)
}

// Palette conversion for v2 (legacy behavior with air at index 0)
fn convert_palette_v2(palette: &Vec<BlockState>) -> (NbtCompound, i32) {
    let mut nbt_palette = NbtCompound::new();
    let mut max_id = 0;

    // Always ensure air is at index 0
    nbt_palette.insert("minecraft:air", NbtTag::Int(0));

    let mut next_id = 1; // Start at 1 since air is at 0

    for block_state in palette.iter() {
        if block_state.name == "minecraft:air" {
            continue; // Skip air blocks as we already added it at index 0
        }

        let key = if block_state.properties.is_empty() {
            block_state.name.clone()
        } else {
            format!(
                "{}[{}]",
                block_state.name,
                block_state
                    .properties
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        };

        nbt_palette.insert(&key, NbtTag::Int(next_id));
        max_id = max_id.max(next_id);
        next_id += 1;
    }

    (nbt_palette, max_id as i32)
}
pub fn from_schematic(data: &[u8]) -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
    let reader = BufReader::with_capacity(1 << 20, data); // 1 MiB buf
    let mut gz = GzDecoder::new(reader);
    let (root, _) = read_nbt(&mut gz, Flavor::Uncompressed)?;

    let schem = root.get::<_, &NbtCompound>("Schematic").unwrap_or(&root);
    let schem_version = schem.get::<_, i32>("Version")?;

    let name = if let Some(metadata) = schem.get::<_, &NbtCompound>("Metadata").ok() {
        metadata.get::<_, &str>("Name").ok().map(|s| s.to_string())
    } else {
        None
    }
    .unwrap_or_else(|| "Unnamed".to_string());

    let mc_version = schem.get::<_, i32>("DataVersion").ok();

    let mut schematic = UniversalSchematic::new(name);
    schematic.metadata.mc_version = mc_version;

    let width = schem.get::<_, i16>("Width")? as u32;
    let height = schem.get::<_, i16>("Height")? as u32;
    let length = schem.get::<_, i16>("Length")? as u32;

    let block_container = if schem_version == 2 {
        schem
    } else {
        schem.get::<_, &NbtCompound>("Blocks")?
    };

    let block_palette = parse_block_palette(&block_container)?;

    let block_data = parse_block_data(&block_container, width, height, length)?;

    let mut region = Region::new(
        "Main".to_string(),
        (0, 0, 0),
        (width as i32, height as i32, length as i32),
    );
    region.palette = block_palette;

    region.blocks = block_data.iter().map(|&x| x as usize).collect();

    let block_entities = parse_block_entities(&block_container)?;
    for block_entity in block_entities {
        region.add_block_entity(block_entity);
    }

    let entities = parse_entities(&schem)?;
    for entity in entities {
        region.add_entity(entity);
    }

    schematic.add_region(region);
    Ok(schematic)
}

fn convert_block_entities(region: &Region) -> NbtList {
    let mut block_entities = NbtList::new();

    for (_, block_entity) in &region.block_entities {
        block_entities.push(block_entity.to_nbt());
    }

    block_entities
}

// Convert block entities for Sponge Schematic v3 format
// Uses to_nbt_v3() which wraps block-specific data in a "Data" compound
fn convert_block_entities_v3(region: &Region) -> NbtList {
    let mut block_entities = NbtList::new();

    for (_, block_entity) in &region.block_entities {
        block_entities.push(block_entity.to_nbt_v3());
    }

    block_entities
}

fn convert_entities(region: &Region) -> NbtList {
    let mut entities = NbtList::new();

    for entity in &region.entities {
        entities.push(entity.to_nbt());
    }

    entities
}

fn parse_block_palette(
    region_tag: &NbtCompound,
) -> Result<Vec<BlockState>, Box<dyn std::error::Error>> {
    let palette_compound = region_tag.get::<_, &NbtCompound>("Palette")?;
    let palette_max = region_tag
        .get::<_, i32>("PaletteMax") // V2
        .unwrap_or(palette_compound.len() as i32) as usize; // V3
    let mut palette = vec![BlockState::new("minecraft:air".to_string()); palette_max + 1];

    for (block_state_str, value) in palette_compound.inner() {
        if let NbtTag::Int(id) = value {
            let block_state = parse_block_state(block_state_str);
            palette[*id as usize] = block_state;
        }
    }

    Ok(palette)
}

fn parse_block_state(input: &str) -> BlockState {
    if let Some((name, properties_str)) = input.split_once('[') {
        let name = name.to_string();
        let properties = properties_str
            .trim_end_matches(']')
            .split(',')
            .filter_map(|prop| {
                let mut parts = prop.splitn(2, '=');
                Some((
                    parts.next()?.trim().to_string(),
                    parts.next()?.trim().to_string(),
                ))
            })
            .collect();
        BlockState { name, properties }
    } else {
        BlockState::new(input.to_string())
    }
}

pub fn encode_varint(value: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut val = value;
    loop {
        let mut byte = (val & 0b0111_1111) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0b1000_0000;
        }
        bytes.push(byte);
        if val == 0 {
            break;
        }
    }
    bytes
}

fn decode_varint<R: Read>(reader: &mut R) -> Result<u32, Box<dyn std::error::Error>> {
    let mut result = 0u32;
    let mut shift = 0;
    loop {
        let mut byte = [0u8; 1];
        reader.read_exact(&mut byte)?;
        result |= ((byte[0] & 0b0111_1111) as u32) << shift;
        if byte[0] & 0b1000_0000 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift >= 32 {
            return Err("Varint is too long".into());
        }
    }
}

fn parse_block_data(
    region_tag: &NbtCompound,
    width: u32,
    height: u32,
    length: u32,
) -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    // V2 = BlockData, V3 = Data
    let block_data_i8 = region_tag
        .get::<_, &Vec<i8>>("BlockData")
        .or(region_tag.get::<_, &Vec<i8>>("Data"))?;

    let mut block_data_u8: &[u8] = unsafe {
        std::slice::from_raw_parts(block_data_i8.as_ptr() as *const u8, block_data_i8.len())
    };

    // ---------- fast var-int decode ----------
    #[inline]
    fn read_varint(slice: &mut &[u8]) -> Option<u32> {
        let mut out = 0u32;
        let mut shift = 0;
        while !slice.is_empty() {
            let byte = slice[0];
            *slice = &slice[1..];
            out |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                return Some(out);
            }
            shift += 7;
        }
        None
    }

    let expected_length = (width * height * length) as usize;
    let mut block_data: Vec<u32> = Vec::with_capacity(expected_length);

    while let Some(id) = read_varint(&mut block_data_u8) {
        block_data.push(id);
    }

    if block_data.len() != expected_length {
        return Err(format!(
            "Block data length mismatch: expected {}, got {}",
            expected_length,
            block_data.len()
        )
        .into());
    }

    Ok(block_data)
}

fn parse_block_entities(
    region_tag: &NbtCompound,
) -> Result<Vec<BlockEntity>, Box<dyn std::error::Error>> {
    let block_entities_list = region_tag.get::<_, &NbtList>("BlockEntities")?;
    let mut block_entities = Vec::new();

    for tag in block_entities_list.iter() {
        if let NbtTag::Compound(compound) = tag {
            let block_entity = BlockEntity::from_nbt(compound);
            block_entities.push(block_entity);
        }
    }

    Ok(block_entities)
}

fn parse_entities(region_tag: &NbtCompound) -> Result<Vec<Entity>, Box<dyn std::error::Error>> {
    if !region_tag.contains_key("Entities") {
        return Ok(Vec::new());
    }
    let entities_list = region_tag.get::<_, &NbtList>("Entities")?;
    let mut entities = Vec::new();

    for tag in entities_list.iter() {
        if let NbtTag::Compound(compound) = tag {
            entities.push(Entity::from_nbt(compound)?);
        }
    }

    Ok(entities)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    use crate::litematic::{from_litematic, to_litematic};
    use crate::{BlockState, UniversalSchematic};

    use super::*;

    #[test]
    fn test_schematic_file_generation() {
        // Create a test schematic
        let mut schematic = UniversalSchematic::new("Test Schematic".to_string());
        let stone = BlockState::new("minecraft:stone".to_string());
        let dirt = BlockState::new("minecraft:dirt".to_string());

        for x in 0..5 {
            for y in 0..5 {
                for z in 0..5 {
                    if (x + y + z) % 2 == 0 {
                        schematic.set_block(x, y, z, stone.clone());
                    } else {
                        schematic.set_block(x, y, z, dirt.clone());
                    }
                }
            }
        }

        // Convert the schematic to .schem format
        let schem_data = to_schematic(&schematic).expect("Failed to convert schematic");

        // Save the .schem file
        let mut file = File::create("test_schematic.schem").expect("Failed to create file");
        file.write_all(&schem_data)
            .expect("Failed to write to file");

        // Read the .schem file back
        let loaded_schem_data = std::fs::read("test_schematic.schem").expect("Failed to read file");

        // Parse the loaded .schem data
        let loaded_schematic =
            from_schematic(&loaded_schem_data).expect("Failed to parse schematic");

        // Compare the original and loaded schematics
        assert_eq!(schematic.metadata.name, loaded_schematic.metadata.name);
        assert_eq!(
            schematic.other_regions.len(),
            loaded_schematic.other_regions.len()
        );
        // Compare tight dimensions (actual content) instead of allocated bounds
        // The export uses compact regions, so loaded schematic will have tight bounds
        assert_eq!(
            schematic.get_tight_dimensions(),
            loaded_schematic.get_dimensions() // Loaded will have tight bounds as its actual size
        );

        let original_region = schematic.default_region;
        let loaded_region = loaded_schematic.default_region;

        assert_eq!(original_region.entities.len(), loaded_region.entities.len());
        assert_eq!(
            original_region.block_entities.len(),
            loaded_region.block_entities.len()
        );

        // Clean up the generated file
        //std::fs::remove_file("test_schematic.schem").expect("Failed to remove file");
    }

    #[test]
    fn test_varint_encoding_decoding() {
        let test_cases = vec![
            0u32,
            1u32,
            127u32,
            128u32,
            255u32,
            256u32,
            65535u32,
            65536u32,
            4294967295u32,
        ];

        for &value in &test_cases {
            let encoded = encode_varint(value);

            let mut cursor = Cursor::new(encoded);
            let decoded = decode_varint(&mut cursor).unwrap();

            assert_eq!(
                value, decoded,
                "Encoding and decoding failed for value: {}",
                value
            );
        }
    }

    #[test]
    fn test_parse_block_data() {
        let mut nbt = NbtCompound::new();
        let block_data = vec![0, 1, 2, 1, 0, 2, 1, 0]; // 8 blocks
        let encoded_block_data: Vec<u8> =
            block_data.iter().flat_map(|&v| encode_varint(v)).collect();

        nbt.insert(
            "BlockData",
            NbtTag::ByteArray(encoded_block_data.iter().map(|&x| x as i8).collect()),
        );

        let parsed_data = parse_block_data(&nbt, 2, 2, 2).expect("Failed to parse block data");
        assert_eq!(parsed_data, vec![0, 1, 2, 1, 0, 2, 1, 0]);
    }

    #[test]
    fn test_convert_palette_v3() {
        let palette = vec![
            BlockState::new("minecraft:stone".to_string()),
            BlockState::new("minecraft:dirt".to_string()),
            BlockState {
                name: "minecraft:wool".to_string(),
                properties: [("color".to_string(), "red".to_string())]
                    .into_iter()
                    .collect(),
            },
        ];

        let (nbt_palette, max_id) = convert_palette(&palette);

        // V3 now ensures air is always at index 0 for WorldEdit compatibility
        assert_eq!(max_id, 3); // air=0, stone=1, dirt=2, wool=3
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:air").unwrap(), 0);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:stone").unwrap(), 1);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:dirt").unwrap(), 2);
        assert_eq!(
            nbt_palette
                .get::<_, i32>("minecraft:wool[color=red]")
                .unwrap(),
            3
        );
    }

    #[test]
    fn test_convert_palette_v2() {
        let palette = vec![
            BlockState::new("minecraft:stone".to_string()),
            BlockState::new("minecraft:dirt".to_string()),
            BlockState {
                name: "minecraft:wool".to_string(),
                properties: [("color".to_string(), "red".to_string())]
                    .into_iter()
                    .collect(),
            },
        ];

        let (nbt_palette, max_id) = convert_palette_v2(&palette);

        // V2 behavior: Air is always at index 0, other blocks follow
        assert_eq!(max_id, 3); // Air=0, stone=1, dirt=2, wool=3
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:air").unwrap(), 0);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:stone").unwrap(), 1);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:dirt").unwrap(), 2);
        assert_eq!(
            nbt_palette
                .get::<_, i32>("minecraft:wool[color=red]")
                .unwrap(),
            3
        );
    }

    #[test]
    fn test_convert_palette_v3_with_air() {
        let palette = vec![
            BlockState::new("minecraft:air".to_string()),
            BlockState::new("minecraft:stone".to_string()),
            BlockState::new("minecraft:dirt".to_string()),
        ];

        let (nbt_palette, max_id) = convert_palette(&palette);

        // V3 with air explicitly in palette - air should still be at index 0
        assert_eq!(max_id, 2);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:air").unwrap(), 0);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:stone").unwrap(), 1);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:dirt").unwrap(), 2);
    }

    #[test]
    fn test_convert_palette_with_mapping() {
        let palette = vec![
            BlockState::new("minecraft:stone".to_string()),
            BlockState::new("minecraft:unknown".to_string()), // Should be mapped to air
            BlockState::new("minecraft:dirt".to_string()),
        ];

        let (nbt_palette, mapping) = convert_palette_with_mapping(&palette);

        // Check palette structure
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:air").unwrap(), 0);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:stone").unwrap(), 1);
        assert_eq!(nbt_palette.get::<_, i32>("minecraft:dirt").unwrap(), 2);

        // Check mapping array
        assert_eq!(mapping[0], 1); // stone -> 1
        assert_eq!(mapping[1], 0); // unknown -> 0 (air)
        assert_eq!(mapping[2], 2); // dirt -> 2
    }
    #[test]
    fn test_import_new_chest_test_schem() {
        let name = "new_chest_test";
        let input_path_str = format!("tests/samples/{}.schem", name);
        let schem_path = Path::new(&input_path_str);
        assert!(schem_path.exists(), "Sample .schem file not found");
        let schem_data =
            fs::read(schem_path).expect(format!("Failed to read {}", input_path_str).as_str());

        let mut schematic = from_schematic(&schem_data).expect("Failed to parse schematic");
        assert_eq!(schematic.metadata.name, Some("Unnamed".to_string()));
    }

    #[test]
    fn test_conversion() {
        let output_dir_path = Path::new("tests/output");
        if !output_dir_path.exists() {
            fs::create_dir_all(output_dir_path)
                .expect("Failed to create output directory 'tests/output'");
        }
        let schem_name = "tests/samples/cutecounter.schem";
        let output_litematic_name = "tests/output/cutecounter.litematic";
        let output_schematic_name = "tests/output/cutecounter.schem";

        //load the schem as a UniversalSchematic
        let schem_data = fs::read(schem_name).expect("Failed to read schem file");
        let schematic = from_schematic(&schem_data).expect("Failed to parse schematic");

        //convert the UniversalSchematic to a Litematic
        let litematic_output_data =
            to_litematic(&schematic).expect("Failed to convert to litematic");
        let mut litematic_output_file =
            File::create(output_litematic_name).expect("Failed to create litematic file");
        litematic_output_file
            .write_all(&litematic_output_data)
            .expect("Failed to write litematic file");

        //load back from the litematic file
        let litematic_data =
            fs::read(output_litematic_name).expect("Failed to read litematic file");
        let schematic_from_litematic =
            from_litematic(&litematic_data).expect("Failed to parse litematic");

        //convert the Litematic back to a UniversalSchematic
        let schematic_output_data =
            to_schematic(&schematic_from_litematic).expect("Failed to convert to schematic");
        let mut schematic_output_file =
            File::create(output_schematic_name).expect("Failed to create schematic file");
        schematic_output_file
            .write_all(&schematic_output_data)
            .expect("Failed to write schematic file");
    }
}
