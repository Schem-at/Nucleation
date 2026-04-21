//! Legacy MCEdit / Classic Schematic (`.schematic`) format importer.
//!
//! This is the pre-Sponge schematic format used by MCEdit, WorldEdit (pre-1.13),
//! and countless community builds. It predates Minecraft's 1.13 "flattening" —
//! block identity is encoded as a numeric ID (0–4095) plus 4-bit metadata,
//! rather than a string blockstate. The format is deprecated; Nucleation
//! supports it for **import only** so old community files can be loaded and
//! re-exported to a modern format.
//!
//! ## Structure
//!
//! ```text
//! Schematic (compound)
//!   Width:      short    (X size)
//!   Height:     short    (Y size)
//!   Length:     short    (Z size)
//!   Materials:  string   ("Alpha" for Java Edition)
//!   Blocks:     byte[]   one numeric block ID per cell (lower 8 bits)
//!   AddBlocks:  byte[]   (optional) upper 4 bits of block ID, nibble-packed
//!   Data:       byte[]   one metadata byte per cell (only low nibble used)
//!   Entities:   list<compound>
//!   TileEntities: list<compound>
//!   WEOffsetX/Y/Z, WEOriginX/Y/Z: int (WorldEdit extensions)
//! ```
//!
//! Array order is `y * Width * Length + z * Width + x`.

use flate2::read::GzDecoder;
use quartz_nbt::io::{read_nbt, Flavor};
use quartz_nbt::{NbtCompound, NbtList, NbtTag};
use std::io::{Cursor, Read};

use crate::block_entity::BlockEntity;
use crate::block_state::BlockState;
use crate::entity::Entity;
use crate::formats::manager::SchematicImporter;
use crate::region::Region;
use crate::universal_schematic::UniversalSchematic;

pub struct ClassicSchematicFormat;

impl SchematicImporter for ClassicSchematicFormat {
    fn name(&self) -> String {
        "classic_schematic".to_string()
    }

    fn detect(&self, data: &[u8]) -> bool {
        is_classic_schematic(data)
    }

    fn read(&self, data: &[u8]) -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
        from_classic_schematic(data)
    }
}

/// Detect whether `data` is a legacy MCEdit `.schematic`. Returns `true` when
/// the file has a `Blocks` byte-array and a `Materials` string, and does NOT
/// have the Sponge-era `Version` or `DataVersion` fields.
pub fn is_classic_schematic(data: &[u8]) -> bool {
    let Ok(root) = decompress_and_parse(data) else {
        return false;
    };
    let has_blocks_bytearr = matches!(root.get::<_, &NbtTag>("Blocks"), Ok(NbtTag::ByteArray(_)));
    let has_materials = root.get::<_, &str>("Materials").is_ok();
    let no_sponge_version = root.get::<_, i32>("Version").is_err();
    let no_data_version = root.get::<_, i32>("DataVersion").is_err();
    has_blocks_bytearr && has_materials && no_sponge_version && no_data_version
}

fn decompress_and_parse(data: &[u8]) -> Result<NbtCompound, Box<dyn std::error::Error>> {
    let mut decoder = GzDecoder::new(data);
    let mut raw = Vec::new();
    decoder.read_to_end(&mut raw)?;
    let (root, _) = read_nbt(&mut Cursor::new(raw), Flavor::Uncompressed)?;
    Ok(root)
}

/// Read a legacy MCEdit `.schematic` into a `UniversalSchematic`.
pub fn from_classic_schematic(
    data: &[u8],
) -> Result<UniversalSchematic, Box<dyn std::error::Error>> {
    let root = decompress_and_parse(data)?;

    let width = root.get::<_, i16>("Width")? as i32;
    let height = root.get::<_, i16>("Height")? as i32;
    let length = root.get::<_, i16>("Length")? as i32;
    if width <= 0 || height <= 0 || length <= 0 {
        return Err("Classic schematic has invalid dimensions".into());
    }

    let blocks = match root.get::<_, &NbtTag>("Blocks")? {
        NbtTag::ByteArray(v) => v.as_slice(),
        _ => return Err("Classic schematic 'Blocks' is not a byte array".into()),
    };
    let meta_arr = match root.get::<_, &NbtTag>("Data")? {
        NbtTag::ByteArray(v) => v.as_slice(),
        _ => return Err("Classic schematic 'Data' is not a byte array".into()),
    };
    let add_blocks = match root.get::<_, &NbtTag>("AddBlocks") {
        Ok(NbtTag::ByteArray(v)) => Some(v.as_slice()),
        _ => None,
    };

    let volume = (width as usize) * (height as usize) * (length as usize);
    if blocks.len() != volume || meta_arr.len() != volume {
        return Err("Blocks/Data length does not match Width*Height*Length".into());
    }

    let mut region = Region::new("Main".to_string(), (0, 0, 0), (width, height, length));

    let w = width as usize;
    let l = length as usize;
    for idx in 0..volume {
        let x = (idx % w) as i32;
        let z = ((idx / w) % l) as i32;
        let y = (idx / (w * l)) as i32;

        let base_id = blocks[idx] as u8 as u16;
        let high = add_blocks
            .and_then(|a| a.get(idx / 2).copied())
            .map(|byte| {
                let byte = byte as u8;
                if idx % 2 == 0 {
                    byte & 0x0F
                } else {
                    (byte >> 4) & 0x0F
                }
            })
            .unwrap_or(0);
        let id = base_id | ((high as u16) << 8);
        let meta = (meta_arr[idx] as u8) & 0x0F;

        if id == 0 {
            continue; // air — leave the cell at its default
        }

        let block = legacy_to_block_state(id, meta);
        region.set_block(x, y, z, &block);
    }

    // TileEntities: position + Id + opaque NBT payload. Positions in MCEdit are
    // absolute x/y/z relative to the schematic origin (0,0,0).
    if let Ok(list) = root.get::<_, &NbtList>("TileEntities") {
        for tag in list.iter() {
            if let NbtTag::Compound(be_nbt) = tag {
                if let Some(be) = parse_tile_entity(be_nbt) {
                    region.add_block_entity(be);
                }
            }
        }
    }

    // Entities: positions in `Pos` list of doubles.
    if let Ok(list) = root.get::<_, &NbtList>("Entities") {
        for tag in list.iter() {
            if let NbtTag::Compound(e_nbt) = tag {
                if let Ok(entity) = Entity::from_nbt(e_nbt) {
                    region.add_entity(entity);
                }
            }
        }
    }

    region.rebuild_non_air_count();
    region.rebuild_tight_bounds();

    let name = root
        .get::<_, &str>("Name")
        .ok()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Classic Schematic".to_string());
    let mut schematic = UniversalSchematic::new(name);
    schematic.set_default_region(region);

    Ok(schematic)
}

fn parse_tile_entity(nbt: &NbtCompound) -> Option<BlockEntity> {
    let id = nbt.get::<_, &str>("id").ok()?.to_string();
    let x = nbt.get::<_, i32>("x").ok()?;
    let y = nbt.get::<_, i32>("y").ok()?;
    let z = nbt.get::<_, i32>("z").ok()?;

    // Translate the legacy schema (`id`+`x`+`y`+`z`) into the modern one
    // (`Id`+`Pos`) that BlockEntity::from_nbt expects, preserving the rest
    // of the payload untouched.
    let mut modern = nbt.clone();
    modern.insert("Id", NbtTag::String(normalize_tile_entity_id(&id)));
    modern.insert("Pos", NbtTag::IntArray(vec![x, y, z]));
    Some(BlockEntity::from_nbt(&modern))
}

/// MCEdit stored tile entity ids in TitleCase ("Chest", "Sign"). 1.13+ uses
/// "minecraft:chest". Normalise to the modern form where we can.
fn normalize_tile_entity_id(id: &str) -> String {
    if id.starts_with("minecraft:") {
        return id.to_string();
    }
    let mapped = match id {
        "Chest" => "minecraft:chest",
        "Trap" => "minecraft:trapped_chest",
        "Furnace" => "minecraft:furnace",
        "Dispenser" => "minecraft:dispenser",
        "Dropper" => "minecraft:dropper",
        "Hopper" => "minecraft:hopper",
        "MobSpawner" => "minecraft:mob_spawner",
        "Sign" => "minecraft:sign",
        "Skull" => "minecraft:skull",
        "Control" => "minecraft:command_block",
        "Beacon" => "minecraft:beacon",
        "EnderChest" => "minecraft:ender_chest",
        "EnchantTable" => "minecraft:enchanting_table",
        "Music" => "minecraft:noteblock",
        "RecordPlayer" => "minecraft:jukebox",
        "Piston" => "minecraft:piston",
        "Comparator" => "minecraft:comparator",
        "Cauldron" => "minecraft:brewing_stand",
        "Banner" => "minecraft:banner",
        other => {
            // Fall through: use a lowercased form under minecraft:
            return format!("minecraft:{}", other.to_lowercase());
        }
    };
    mapped.to_string()
}

// ─── legacy (id, meta) → BlockState ────────────────────────────────────────

/// Convert a legacy MC 1.12 `(id, meta)` pair to a modern `BlockState`.
///
/// Covers common building blocks, wool, slabs, stairs, logs, leaves, glass,
/// and redstone basics. Unknown ids fall back to `minecraft:stone` so the
/// geometry is preserved even if the exact block isn't recognised.
fn legacy_to_block_state(id: u16, meta: u8) -> BlockState {
    // Helpers ---------------------------------------------------------------
    let simple = |name: &str| BlockState::new(format!("minecraft:{}", name));
    let with_facing = |name: &str, facing: &str| {
        BlockState::new(format!("minecraft:{}", name)).with_property("facing", facing)
    };

    match id {
        0 => simple("air"),
        1 => match meta {
            0 => simple("stone"),
            1 => simple("granite"),
            2 => simple("polished_granite"),
            3 => simple("diorite"),
            4 => simple("polished_diorite"),
            5 => simple("andesite"),
            6 => simple("polished_andesite"),
            _ => simple("stone"),
        },
        2 => simple("grass_block"),
        3 => match meta {
            1 => simple("coarse_dirt"),
            2 => simple("podzol"),
            _ => simple("dirt"),
        },
        4 => simple("cobblestone"),
        5 => simple(wood_variant(meta, "planks")),
        6 => simple(&format!("{}_sapling", wood_variant(meta & 0x07, ""))),
        7 => simple("bedrock"),
        8 | 9 => simple("water"),
        10 | 11 => simple("lava"),
        12 => match meta {
            1 => simple("red_sand"),
            _ => simple("sand"),
        },
        13 => simple("gravel"),
        14 => simple("gold_ore"),
        15 => simple("iron_ore"),
        16 => simple("coal_ore"),
        17 => log_block(meta, false),
        18 => leaves_block(meta, false),
        19 => simple(if meta == 1 { "wet_sponge" } else { "sponge" }),
        20 => simple("glass"),
        21 => simple("lapis_ore"),
        22 => simple("lapis_block"),
        23 => with_facing("dispenser", facing_north_s_e_w_u_d(meta & 0x07)),
        24 => match meta {
            1 => simple("chiseled_sandstone"),
            2 => simple("smooth_sandstone"),
            _ => simple("sandstone"),
        },
        25 => simple("note_block"),
        26 => simple("red_bed"), // color lost — approximate
        27 => simple("powered_rail"),
        28 => simple("detector_rail"),
        29 => with_facing("sticky_piston", facing_north_s_e_w_u_d(meta & 0x07)),
        30 => simple("cobweb"),
        31 => match meta {
            1 => simple("grass"),
            2 => simple("fern"),
            _ => simple("dead_bush"),
        },
        32 => simple("dead_bush"),
        33 => with_facing("piston", facing_north_s_e_w_u_d(meta & 0x07)),
        35 => wool_block(meta),
        37 => simple("dandelion"),
        38 => simple("poppy"), // many flower subtypes by meta
        39 => simple("brown_mushroom"),
        40 => simple("red_mushroom"),
        41 => simple("gold_block"),
        42 => simple("iron_block"),
        43 => double_stone_slab(meta),
        44 => stone_slab(meta),
        45 => simple("bricks"),
        46 => simple("tnt"),
        47 => simple("bookshelf"),
        48 => simple("mossy_cobblestone"),
        49 => simple("obsidian"),
        50 => torch_block("torch", meta),
        51 => simple("fire"),
        52 => simple("spawner"),
        53 => stairs_block("oak_stairs", meta),
        54 => with_facing("chest", facing_2_3_4_5(meta & 0x07)),
        55 => simple("redstone_wire"),
        56 => simple("diamond_ore"),
        57 => simple("diamond_block"),
        58 => simple("crafting_table"),
        59 => simple("wheat"),
        60 => simple("farmland"),
        61 => with_facing("furnace", facing_2_3_4_5(meta & 0x07)),
        62 => with_facing("lit_furnace", facing_2_3_4_5(meta & 0x07)),
        63 => simple("sign"),
        64 => simple("oak_door"),
        65 => with_facing("ladder", facing_2_3_4_5(meta & 0x07)),
        66 => simple("rail"),
        67 => stairs_block("cobblestone_stairs", meta),
        68 => with_facing("wall_sign", facing_2_3_4_5(meta & 0x07)),
        69 => simple("lever"),
        70 => simple("stone_pressure_plate"),
        71 => simple("iron_door"),
        72 => simple("oak_pressure_plate"),
        73 => simple("redstone_ore"),
        74 => simple("redstone_ore"),
        75 => torch_block("redstone_wall_torch", meta),
        76 => torch_block("redstone_wall_torch", meta),
        77 => simple("stone_button"),
        78 => simple("snow"),
        79 => simple("ice"),
        80 => simple("snow_block"),
        81 => simple("cactus"),
        82 => simple("clay"),
        83 => simple("sugar_cane"),
        84 => simple("jukebox"),
        85 => simple("oak_fence"),
        86 => simple("pumpkin"),
        87 => simple("netherrack"),
        88 => simple("soul_sand"),
        89 => simple("glowstone"),
        90 => simple("nether_portal"),
        91 => simple("jack_o_lantern"),
        95 => stained_glass_block("stained_glass", meta),
        96 => simple("oak_trapdoor"),
        97 => simple("infested_stone"),
        98 => match meta {
            1 => simple("mossy_stone_bricks"),
            2 => simple("cracked_stone_bricks"),
            3 => simple("chiseled_stone_bricks"),
            _ => simple("stone_bricks"),
        },
        99 => simple("brown_mushroom_block"),
        100 => simple("red_mushroom_block"),
        101 => simple("iron_bars"),
        102 => simple("glass_pane"),
        103 => simple("melon"),
        108 => stairs_block("brick_stairs", meta),
        109 => stairs_block("stone_brick_stairs", meta),
        112 => simple("nether_bricks"),
        113 => simple("nether_brick_fence"),
        114 => stairs_block("nether_brick_stairs", meta),
        121 => simple("end_stone"),
        123 => simple("redstone_lamp"),
        124 => simple("redstone_lamp"),
        126 => wood_slab(meta),
        128 => stairs_block("sandstone_stairs", meta),
        129 => simple("emerald_ore"),
        133 => simple("emerald_block"),
        134 => stairs_block("spruce_stairs", meta),
        135 => stairs_block("birch_stairs", meta),
        136 => stairs_block("jungle_stairs", meta),
        139 => simple(if meta == 1 {
            "mossy_cobblestone_wall"
        } else {
            "cobblestone_wall"
        }),
        152 => simple("redstone_block"),
        155 => match meta {
            1 => simple("chiseled_quartz_block"),
            2 => simple("quartz_pillar"),
            _ => simple("quartz_block"),
        },
        156 => stairs_block("quartz_stairs", meta),
        159 => stained_glass_block("stained_terracotta", meta),
        160 => stained_glass_block("stained_glass_pane", meta),
        161 => leaves_block(meta & 0x01, true),
        162 => log_block(meta & 0x01, true),
        163 => stairs_block("acacia_stairs", meta),
        164 => stairs_block("dark_oak_stairs", meta),
        170 => simple("hay_block"),
        171 => wool_block(meta), // carpet — color mapping matches wool
        172 => simple("terracotta"),
        173 => simple("coal_block"),
        174 => simple("packed_ice"),
        179 => match meta {
            1 => simple("chiseled_red_sandstone"),
            2 => simple("smooth_red_sandstone"),
            _ => simple("red_sandstone"),
        },
        180 => stairs_block("red_sandstone_stairs", meta),
        181 => double_red_sandstone_slab(),
        182 => simple("red_sandstone_slab"),
        183 => simple("spruce_fence_gate"),
        184 => simple("birch_fence_gate"),
        185 => simple("jungle_fence_gate"),
        186 => simple("dark_oak_fence_gate"),
        187 => simple("acacia_fence_gate"),
        188 => simple("spruce_fence"),
        189 => simple("birch_fence"),
        190 => simple("jungle_fence"),
        191 => simple("dark_oak_fence"),
        192 => simple("acacia_fence"),
        199 => simple("chorus_plant"),
        200 => simple("chorus_flower"),
        201 => simple("purpur_block"),
        202 => simple("purpur_pillar"),
        203 => stairs_block("purpur_stairs", meta),
        205 => simple("purpur_slab"),
        206 => simple("end_stone_bricks"),
        208 => simple("grass_path"),
        210 => simple("repeating_command_block"),
        211 => simple("chain_command_block"),
        213 => simple("magma_block"),
        214 => simple("nether_wart_block"),
        215 => simple("red_nether_bricks"),
        216 => simple("bone_block"),
        251 => wool_block(meta), // concrete — reuse color mapping
        252 => wool_block(meta), // concrete powder — reuse color mapping

        // Unknown — preserve geometry as stone so the shape survives the import.
        _ => simple("stone"),
    }
}

fn wood_variant(meta: u8, _suffix: &str) -> &'static str {
    match meta & 0x07 {
        0 => "oak",
        1 => "spruce",
        2 => "birch",
        3 => "jungle",
        4 => "acacia",
        5 => "dark_oak",
        _ => "oak",
    }
}

fn wool_block(meta: u8) -> BlockState {
    let color = match meta & 0x0F {
        0 => "white",
        1 => "orange",
        2 => "magenta",
        3 => "light_blue",
        4 => "yellow",
        5 => "lime",
        6 => "pink",
        7 => "gray",
        8 => "light_gray",
        9 => "cyan",
        10 => "purple",
        11 => "blue",
        12 => "brown",
        13 => "green",
        14 => "red",
        _ => "black",
    };
    BlockState::new(format!("minecraft:{}_wool", color))
}

fn stained_glass_block(suffix: &str, meta: u8) -> BlockState {
    let color = match meta & 0x0F {
        0 => "white",
        1 => "orange",
        2 => "magenta",
        3 => "light_blue",
        4 => "yellow",
        5 => "lime",
        6 => "pink",
        7 => "gray",
        8 => "light_gray",
        9 => "cyan",
        10 => "purple",
        11 => "blue",
        12 => "brown",
        13 => "green",
        14 => "red",
        _ => "black",
    };
    BlockState::new(format!("minecraft:{}_{}", color, suffix))
}

fn log_block(meta: u8, id2: bool) -> BlockState {
    let species = if id2 {
        // id 162: acacia / dark_oak
        match meta & 0x03 {
            0 => "acacia",
            _ => "dark_oak",
        }
    } else {
        // id 17: oak / spruce / birch / jungle
        match meta & 0x03 {
            0 => "oak",
            1 => "spruce",
            2 => "birch",
            _ => "jungle",
        }
    };
    let axis = match (meta >> 2) & 0x03 {
        0 => "y",
        1 => "x",
        2 => "z",
        _ => "none",
    };
    BlockState::new(format!("minecraft:{}_log", species)).with_property("axis", axis)
}

fn leaves_block(meta: u8, id2: bool) -> BlockState {
    let species = if id2 {
        match meta & 0x01 {
            0 => "acacia",
            _ => "dark_oak",
        }
    } else {
        match meta & 0x03 {
            0 => "oak",
            1 => "spruce",
            2 => "birch",
            _ => "jungle",
        }
    };
    BlockState::new(format!("minecraft:{}_leaves", species))
}

fn stone_slab(meta: u8) -> BlockState {
    let variant = match meta & 0x07 {
        0 => "smooth_stone",
        1 => "sandstone",
        2 => "oak", // "wooden" slab — represented as oak in modern ids
        3 => "cobblestone",
        4 => "brick",
        5 => "stone_brick",
        6 => "nether_brick",
        7 => "quartz",
        _ => "smooth_stone",
    };
    let half = if meta & 0x08 != 0 { "top" } else { "bottom" };
    BlockState::new(format!("minecraft:{}_slab", variant)).with_property("type", half)
}

fn double_stone_slab(meta: u8) -> BlockState {
    let variant = match meta & 0x07 {
        0 => "smooth_stone",
        1 => "sandstone",
        2 => "oak",
        3 => "cobblestone",
        4 => "brick",
        5 => "stone_brick",
        6 => "nether_brick",
        7 => "quartz",
        _ => "smooth_stone",
    };
    BlockState::new(format!("minecraft:{}_slab", variant)).with_property("type", "double")
}

fn double_red_sandstone_slab() -> BlockState {
    BlockState::new("minecraft:red_sandstone_slab".to_string()).with_property("type", "double")
}

fn wood_slab(meta: u8) -> BlockState {
    let species = match meta & 0x07 {
        0 => "oak",
        1 => "spruce",
        2 => "birch",
        3 => "jungle",
        4 => "acacia",
        5 => "dark_oak",
        _ => "oak",
    };
    let half = if meta & 0x08 != 0 { "top" } else { "bottom" };
    BlockState::new(format!("minecraft:{}_slab", species)).with_property("type", half)
}

fn stairs_block(name: &str, meta: u8) -> BlockState {
    // facing bits 0-1: 0=east, 1=west, 2=south, 3=north
    let facing = match meta & 0x03 {
        0 => "east",
        1 => "west",
        2 => "south",
        _ => "north",
    };
    // half bit 2: 0=bottom, 1=top
    let half = if meta & 0x04 != 0 { "top" } else { "bottom" };
    BlockState::new(format!("minecraft:{}", name))
        .with_property("facing", facing)
        .with_property("half", half)
}

fn facing_north_s_e_w_u_d(meta: u8) -> &'static str {
    match meta & 0x07 {
        0 => "down",
        1 => "up",
        2 => "north",
        3 => "south",
        4 => "west",
        5 => "east",
        _ => "north",
    }
}

fn facing_2_3_4_5(meta: u8) -> &'static str {
    match meta & 0x07 {
        2 => "north",
        3 => "south",
        4 => "west",
        5 => "east",
        _ => "north",
    }
}

fn torch_block(name: &str, meta: u8) -> BlockState {
    // For wall torches the legacy meta encodes which wall it's on.
    // meta=5 means standing torch on floor; 1..4 means wall-mounted.
    if name.contains("wall") {
        let facing = match meta {
            1 => "east",
            2 => "west",
            3 => "south",
            4 => "north",
            _ => "north",
        };
        BlockState::new(format!("minecraft:{}", name)).with_property("facing", facing)
    } else {
        BlockState::new("minecraft:torch".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_sponge_as_not_classic() {
        // A gzipped, empty Sponge-style compound with Version=3 should NOT be
        // detected as classic. We can build one quickly:
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use quartz_nbt::io::{write_nbt, Flavor};
        use std::io::Write;

        let mut root = NbtCompound::new();
        root.insert("Version", NbtTag::Int(3));
        root.insert("Materials", NbtTag::String("Alpha".to_string()));
        let mut raw = Vec::new();
        write_nbt(&mut raw, None, &root, Flavor::Uncompressed).unwrap();
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&raw).unwrap();
        let gz = enc.finish().unwrap();
        assert!(!is_classic_schematic(&gz));
    }

    #[test]
    fn legacy_mapping_stone() {
        let bs = legacy_to_block_state(1, 0);
        assert_eq!(bs.name, "minecraft:stone");
    }

    #[test]
    fn legacy_mapping_cobblestone_stairs() {
        let bs = legacy_to_block_state(67, 4);
        assert_eq!(bs.name, "minecraft:cobblestone_stairs");
        assert_eq!(bs.get_property("facing").map(|s| s.as_str()), Some("east"));
        assert_eq!(bs.get_property("half").map(|s| s.as_str()), Some("top"));
    }

    #[test]
    fn legacy_mapping_stone_slab_top() {
        let bs = legacy_to_block_state(44, 11);
        assert_eq!(bs.name, "minecraft:cobblestone_slab");
        assert_eq!(bs.get_property("type").map(|s| s.as_str()), Some("top"));
    }

    #[test]
    fn legacy_mapping_wool_red() {
        let bs = legacy_to_block_state(35, 14);
        assert_eq!(bs.name, "minecraft:red_wool");
    }
}
