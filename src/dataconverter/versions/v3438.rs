//! V3438 (1.20 trail ruins) — schematic-relevant subset: pottery-shard item
//! renames (V3438.java:35-42), plus brushable_block tile-entity handling:
//! suspicious_sand -> brushable_block id rename + copyWalkers, and capitalizing
//! the legacy lowercase loot_table/loot_table_seed fields on brushable_block
//! (V3438.java:19-33). Cites V3438.java.

use super::super::helpers::{map_renamer, register_item_rename};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use crate::nbt::NbtMap;

const VERSION: i32 = 3438;

/// `(old, new)` pottery-shard renames. Exposed so the reverse engine can invert.
pub const POTTERY_SHARD_RENAMES: &[(&str, &str)] = &[
    (
        "minecraft:pottery_shard_archer",
        "minecraft:archer_pottery_shard",
    ),
    (
        "minecraft:pottery_shard_prize",
        "minecraft:prize_pottery_shard",
    ),
    (
        "minecraft:pottery_shard_arms_up",
        "minecraft:arms_up_pottery_shard",
    ),
    (
        "minecraft:pottery_shard_skull",
        "minecraft:skull_pottery_shard",
    ),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(POTTERY_SHARD_RENAMES));

    // brushable block rename (V3438.java:19): copy the suspicious_sand walker
    // onto the new brushable_block id.
    reg.tile_entity.copy_walkers(
        VERSION,
        0,
        "minecraft:suspicious_sand",
        "minecraft:brushable_block",
    );

    // ConverterAbstractTileEntityRename (V3438.java:21-23): rename the
    // tile-entity id suspicious_sand -> brushable_block.
    reg.tile_entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("id") == Some("minecraft:suspicious_sand") {
                data.set_string("id", "minecraft:brushable_block");
            }
        }),
    );

    // REVERSE of the id rename above: brushable_block -> suspicious_sand. The
    // forward did this as a hand-written structure converter (not
    // `register_*_rename`/`map_renamer`), so it is NOT auto-inverted — we supply
    // the inverse explicitly. The rename is 1:1, so this is exact (bucket B,
    // lossless): the modern id uniquely encodes the old one (rule 11).
    reg.tile_entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("id") == Some("minecraft:brushable_block") {
                data.set_string("id", "minecraft:suspicious_sand");
            }
        }),
    );

    // V3438.java:26-33: capitalize the legacy lowercase loot fields on
    // brushable_block (RenameHelper.renameSingle clobbers the target if the
    // source key is present, otherwise no-op).
    reg.tile_entity.add_converter_for_id(
        "minecraft:brushable_block",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(v) = data.take("loot_table") {
                data.set_generic("LootTable", v);
            }
            if let Some(v) = data.take("loot_table_seed") {
                data.set_generic("LootTableSeed", v);
            }
        }),
    );

    // REVERSE of the loot-field capitalization above: LootTable -> loot_table,
    // LootTableSeed -> loot_table_seed. Matched against the NEW id
    // `minecraft:brushable_block` (rule 4): any inverse id-rename to
    // suspicious_sand runs later in the descending sweep. Pure 1:1 key rename,
    // so exact (bucket A, lossless) — no loss report.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:brushable_block",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(v) = data.take("LootTable") {
                data.set_generic("loot_table", v);
            }
            if let Some(v) = data.take("LootTableSeed") {
                data.set_generic("loot_table_seed", v);
            }
        }),
    );
}
