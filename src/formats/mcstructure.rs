use crate::block_entity::BlockEntity;
use crate::entity::Entity;
use crate::formats::manager::{SchematicExporter, SchematicImporter};
use crate::nbt::io::{read_nbt, write_nbt};
use crate::nbt::{Endian, NbtMap, NbtValue};
use crate::region::Region;
use crate::universal_schematic::UniversalSchematic;
use crate::BlockState;
use blockpedia;
use std::collections::HashMap;
use std::io::Cursor;

pub struct McStructureFormat;

impl SchematicImporter for McStructureFormat {
    fn name(&self) -> String {
        "mcstructure".to_string()
    }

    fn detect(&self, data: &[u8]) -> bool {
        let mut cursor = Cursor::new(data);
        match read_nbt(&mut cursor, Endian::Little) {
            Ok(NbtValue::Compound(root)) => {
                root.get("format_version").is_some()
                    && root.get("size").is_some()
                    && root.get("structure").is_some()
            }
            _ => false,
        }
    }

    fn read(&self, data: &[u8]) -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
        from_mcstructure(data)
    }
}

impl SchematicExporter for McStructureFormat {
    fn name(&self) -> String {
        "mcstructure".to_string()
    }

    fn extensions(&self) -> Vec<String> {
        vec!["mcstructure".to_string()]
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
        to_mcstructure(schematic)
    }
}

pub fn from_mcstructure(data: &[u8]) -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
    let mut cursor = Cursor::new(data);
    let root_val = read_nbt(&mut cursor, Endian::Little)?;
    let root = match root_val {
        NbtValue::Compound(c) => c,
        _ => return Err("Root is not a compound".into()),
    };

    let size_list = match root.get("size") {
        Some(NbtValue::List(l)) => l,
        _ => return Err("Missing or invalid size".into()),
    };

    let mut size_iter = size_list.iter();
    let width = match size_iter.next() {
        Some(NbtValue::Int(v)) => *v,
        _ => 0,
    };
    let height = match size_iter.next() {
        Some(NbtValue::Int(v)) => *v,
        _ => 0,
    };
    let length = match size_iter.next() {
        Some(NbtValue::Int(v)) => *v,
        _ => 0,
    };

    let structure = match root.get("structure") {
        Some(NbtValue::Compound(c)) => c,
        _ => return Err("Missing structure compound".into()),
    };

    // Parse Palette
    let palette_wrapper = match structure.get("palette") {
        Some(NbtValue::Compound(c)) => c,
        _ => return Err("Missing palette".into()),
    };
    let default_palette = match palette_wrapper.get("default") {
        Some(NbtValue::Compound(c)) => c,
        _ => return Err("Missing default palette".into()),
    };
    let block_palette_list = match default_palette.get("block_palette") {
        Some(NbtValue::List(l)) => l,
        _ => return Err("Missing block_palette".into()),
    };

    let mut palette: Vec<BlockState> = Vec::new();
    for tag in block_palette_list.iter() {
        if let NbtValue::Compound(block_compound) = tag {
            let name = block_compound
                .get("name")
                .and_then(|v| v.as_string())
                .cloned()
                .unwrap_or_else(|| "minecraft:air".to_string());
            let mut properties = HashMap::new();

            if let Some(NbtValue::Compound(states)) = block_compound.get("states") {
                for (key, val) in states.iter() {
                    let val_str = match val {
                        NbtValue::Byte(b) => {
                            if *b == 1 {
                                "true".to_string()
                            } else if *b == 0 {
                                "false".to_string()
                            } else {
                                b.to_string()
                            }
                        }
                        NbtValue::Int(i) => i.to_string(),
                        NbtValue::String(s) => s.clone(),
                        _ => format!("{:?}", val), // Fallback
                    };
                    properties.insert(key.clone(), val_str);
                }
            }

            // Translate Bedrock -> Java using blockpedia
            let translated_state = if let Ok(bp_state) =
                blockpedia::BlockState::from_bedrock(&name, properties.clone())
            {
                BlockState {
                    name: bp_state.id().to_string(),
                    properties: bp_state.properties().clone(),
                }
            } else {
                BlockState { name, properties }
            };

            palette.push(translated_state);
        }
    }

    // Construct Region
    let mut region = Region::new("Main".to_string(), (0, 0, 0), (width, height, length));

    // Parse Block Indices (Multi-layer support)
    let block_indices_list = match structure.get("block_indices") {
        Some(NbtValue::List(l)) => l,
        _ => return Err("Missing block_indices".into()),
    };

    for layer in block_indices_list {
        let indices: Vec<i32> = match layer {
            NbtValue::List(list) => list
                .iter()
                .filter_map(|t| {
                    if let NbtValue::Int(i) = t {
                        Some(*i)
                    } else {
                        None
                    }
                })
                .collect(),
            _ => continue,
        };

        // Set blocks
        // Indices are ZYX order: X outer, Y middle, Z inner
        // index = SZ*SY*X + SZ*Y + Z
        for (i, &palette_idx) in indices.iter().enumerate() {
            if palette_idx < 0 {
                continue;
            } // -1 is void/air-skip

            let i = i as i32;
            let sz = length;
            let sy = height;

            if sz == 0 || sy == 0 {
                continue;
            }

            let x = i / sz / sy;
            let y = (i / sz) % sy;
            let z = i % sz;

            if palette_idx < palette.len() as i32 {
                let block = palette[palette_idx as usize].clone();

                // Simple merge logic:
                // If the block is water/lava and we already have a block here,
                // try to set waterlogged=true instead of overwriting.
                if (block.name == "minecraft:water" || block.name == "minecraft:flowing_water")
                    && region.get_block(x, y, z).is_some()
                {
                    if let Some(existing_block) = region.get_block(x, y, z) {
                        if existing_block.name != "minecraft:air" {
                            let mut updated_block = existing_block.clone();
                            updated_block
                                .properties
                                .insert("waterlogged".to_string(), "true".to_string());
                            region.set_block(x, y, z, &updated_block);
                            continue;
                        }
                    }
                }

                region.set_block(x, y, z, &block);
            }
        }
    }

    // Block Entities
    if let Some(NbtValue::Compound(block_position_data)) =
        default_palette.get("block_position_data")
    {
        for (index_str, data) in block_position_data.iter() {
            if let Ok(index) = index_str.parse::<i32>() {
                if let NbtValue::Compound(data_compound) = data {
                    if let Some(NbtValue::Compound(be_data)) =
                        data_compound.get("block_entity_data")
                    {
                        let i = index;
                        let sz = length;
                        let sy = height;

                        if sz > 0 && sy > 0 {
                            let x = i / sz / sy;
                            let y = (i / sz) % sy;
                            let z = i % sz;

                            let mut be_nbt = be_data.clone();
                            be_nbt.insert("x".to_string(), NbtValue::Int(x));
                            be_nbt.insert("y".to_string(), NbtValue::Int(y));
                            be_nbt.insert("z".to_string(), NbtValue::Int(z));

                            // Extract ID and Pos for BlockEntity constructor
                            let id = be_nbt
                                .get("id")
                                .and_then(|v| v.as_string())
                                .cloned()
                                .unwrap_or_else(|| "unknown".to_string());

                            // Construct BlockEntity manually using our NbtMap
                            // We need to convert from crate::nbt::NbtMap to whatever BlockEntity uses
                            // BlockEntity uses crate::nbt::NbtMap! (via re-export in lib.rs -> utils -> nbt)

                            // Note: BlockEntity::from_nbt expects quartz_nbt::NbtCompound.
                            // We should construct it manually to avoid unnecessary conversions.
                            let mut be = BlockEntity::new(id, (x, y, z));
                            be.nbt = be_nbt;

                            region.add_block_entity(be);
                        }
                    }
                }
            }
        }
    }

    // Entities
    if let Some(NbtValue::List(entities_list)) = structure.get("entities") {
        for tag in entities_list.iter() {
            if let NbtValue::Compound(compound) = tag {
                // Entity::from_nbt expects quartz_nbt::NbtCompound
                // We need to convert NbtMap to NbtCompound or update Entity
                // For now, convert
                if let Ok(entity) = Entity::from_nbt(&compound.to_quartz_nbt()) {
                    region.add_entity(entity);
                }
            }
        }
    }

    region.rebuild_tight_bounds();

    let mut schematic = UniversalSchematic::new("Unnamed".to_string());
    schematic.add_region(region);

    // Post-fix redstone connectivity
    schematic.fix_redstone_connectivity();

    Ok(schematic)
}

pub fn to_mcstructure(
    schematic: &UniversalSchematic,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let merged_region = schematic.get_merged_region();
    let compact_region = merged_region.to_compact();
    let (width, height, length) = compact_region.get_dimensions();

    let mut root = NbtMap::new();
    root.insert("format_version".to_string(), NbtValue::Int(1));

    let mut size_list = Vec::new();
    size_list.push(NbtValue::Int(width));
    size_list.push(NbtValue::Int(height));
    size_list.push(NbtValue::Int(length));
    root.insert("size".to_string(), NbtValue::List(size_list));

    let mut origin_list = Vec::new();
    origin_list.push(NbtValue::Int(0));
    origin_list.push(NbtValue::Int(0));
    origin_list.push(NbtValue::Int(0));
    root.insert(
        "structure_world_origin".to_string(),
        NbtValue::List(origin_list),
    );

    let mut structure = NbtMap::new();

    // Palette
    let mut palette_compound = NbtMap::new();
    let mut default_palette = NbtMap::new();
    let mut block_palette_list = Vec::new();
    let mut block_position_data = NbtMap::new();

    for block in &compact_region.palette {
        let mut block_entry = NbtMap::new();

        // Translate Java -> Bedrock using blockpedia
        let (name, properties) =
            if let Ok(java_bp_state) = blockpedia::BlockState::parse(&block.to_string()) {
                if let Ok(bedrock_bp_state) = java_bp_state.to_bedrock() {
                    (
                        bedrock_bp_state.id().to_string(),
                        bedrock_bp_state.properties().clone(),
                    )
                } else {
                    (block.name.clone(), block.properties.clone())
                }
            } else {
                (block.name.clone(), block.properties.clone())
            };

        block_entry.insert("name".to_string(), NbtValue::String(name));

        let mut states = NbtMap::new();
        for (k, v) in &properties {
            let tag = if v == "true" {
                NbtValue::Byte(1)
            } else if v == "false" {
                NbtValue::Byte(0)
            } else if let Ok(i) = v.parse::<i32>() {
                NbtValue::Int(i)
            } else {
                NbtValue::String(v.clone())
            };
            states.insert(k.clone(), tag);
        }
        block_entry.insert("states".to_string(), NbtValue::Compound(states));
        block_entry.insert("version".to_string(), NbtValue::Int(17959425));

        block_palette_list.push(NbtValue::Compound(block_entry));
    }

    default_palette.insert(
        "block_palette".to_string(),
        NbtValue::List(block_palette_list),
    );

    // Block Entities Data
    for ((x, y, z), be) in &compact_region.block_entities {
        let rel_x = x - compact_region.position.0;
        let rel_y = y - compact_region.position.1;
        let rel_z = z - compact_region.position.2;

        if rel_x >= 0
            && rel_y >= 0
            && rel_z >= 0
            && rel_x < width
            && rel_y < height
            && rel_z < length
        {
            let index = rel_x * (length * height) + rel_y * length + rel_z;

            let mut be_compound = NbtMap::new();
            // be.to_nbt() returns NbtCompound (quartz). We need NbtMap.
            // But we can construct NbtMap from BlockEntity data directly
            let be_data_map = NbtMap::from_quartz_nbt(&be.to_nbt());

            be_compound.insert(
                "block_entity_data".to_string(),
                NbtValue::Compound(be_data_map),
            );

            block_position_data.insert(index.to_string(), NbtValue::Compound(be_compound));
        }
    }

    default_palette.insert(
        "block_position_data".to_string(),
        NbtValue::Compound(block_position_data),
    );
    palette_compound.insert("default".to_string(), NbtValue::Compound(default_palette));
    structure.insert("palette".to_string(), NbtValue::Compound(palette_compound));

    // Block Indices
    let mut indices_list = Vec::new();
    let volume = (width * height * length) as usize;

    for x in 0..width {
        for y in 0..height {
            for z in 0..length {
                let abs_x = x + compact_region.position.0;
                let abs_y = y + compact_region.position.1;
                let abs_z = z + compact_region.position.2;

                let palette_idx =
                    if let Some(idx) = compact_region.get_block_index(abs_x, abs_y, abs_z) {
                        idx as i32
                    } else {
                        -1
                    };
                indices_list.push(NbtValue::Int(palette_idx));
            }
        }
    }

    let mut block_indices = Vec::new();
    block_indices.push(NbtValue::List(indices_list));

    let mut secondary_list = Vec::new();
    for _ in 0..volume {
        secondary_list.push(NbtValue::Int(-1));
    }
    block_indices.push(NbtValue::List(secondary_list));

    structure.insert("block_indices".to_string(), NbtValue::List(block_indices));

    // Entities
    let mut entities_list = Vec::new();
    for entity in &compact_region.entities {
        if let quartz_nbt::NbtTag::Compound(c) = entity.to_nbt() {
            entities_list.push(NbtValue::Compound(NbtMap::from_quartz_nbt(&c)));
        }
    }
    structure.insert("entities".to_string(), NbtValue::List(entities_list));

    root.insert("structure".to_string(), NbtValue::Compound(structure));

    let mut cursor = Cursor::new(Vec::new());
    write_nbt(&mut cursor, &root, "", Endian::Little)?;

    Ok(cursor.into_inner())
}
