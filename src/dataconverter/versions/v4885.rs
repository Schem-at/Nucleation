//! V4885 (26.2-snapshot-2 + 1) — schematic-relevant subset of `V4885.java`.
//!
//! The bed block entity was removed in 26.2 (Mojang `RemoveBlockEntityTagFix`
//! for `minecraft:bed`, schema `V4885` drops the `minecraft:bed` block-entity
//! choice). Upstream registers it through
//! `ConverterAbstractTileEntityRemove.register(VERSION, "minecraft:bed")`
//! (`converters/tileentity/ConverterAbstractTileEntityRemove.java`), which adds
//! four converters. Ported:
//!   * ITEM_STACK structure converter: drop `components["minecraft:block_entity_data"]`
//!     when its `id` is `minecraft:bed` (ConverterAbstractTileEntityRemove.java:12-25).
//!   * ENTITY `minecraft:falling_block` converter: drop `TileEntityData` when its
//!     `id` is `minecraft:bed` (ConverterAbstractTileEntityRemove.java:27-36).
//!   * STRUCTURE structure converter: drop `blocks[].nbt` when its `id` is
//!     `minecraft:bed` (ConverterAbstractTileEntityRemove.java:38-54).
//!   * The CHUNK converter (drop matching `block_entities[]` entries,
//!     ConverterAbstractTileEntityRemove.java:56-72) has no CHUNK type in this
//!     port; its schematic analogue — a region's block-entity map — is applied in
//!     `apply::convert_region` through [`REMOVED_BLOCK_ENTITIES`].
//!
//! Reverse: nothing to restore. Since the Flattening a bed block entity carries
//! no data beyond its `id` and position (the colour lives in the block id), and
//! pre-26.2 games recreate a missing bed block entity on demand, so the removal
//! is information-free and no inverse converter (or loss report) is needed
//! (cheatsheet rule 10).
//!
//! VERSION = MCVersions.V26_2_SNAPSHOT2 (4884) + 1 = 4885.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4885;

/// The block-entity id removed at this version.
const BED: &str = "minecraft:bed";

/// Block-entity ids removed by a `ConverterAbstractTileEntityRemove` step, as
/// `(version, id)`. Used by `apply::convert_region` for the CHUNK-converter
/// analogue: a schematic region's block entity of this id is dropped when a
/// forward conversion crosses `version`.
pub(in crate::dataconverter) const REMOVED_BLOCK_ENTITIES: &[(i32, &str)] = &[(VERSION, BED)];

/// `ConverterAbstractTileEntityRemove.isType` (lines 75-77).
fn is_type(data: Option<&NbtMap>, ty: &str) -> bool {
    data.is_some_and(|d| d.get_string("id") == Some(ty))
}

pub fn register(reg: &mut RegistryBuilder) {
    // ITEM_STACK: drop a bed `block_entity_data` component.
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(components) = data.get_map_mut("components") {
                if is_type(components.get_map("minecraft:block_entity_data"), BED) {
                    components.take("minecraft:block_entity_data");
                }
            }
        }),
    );

    // ENTITY falling_block: drop bed `TileEntityData`.
    reg.entity.add_converter_for_id(
        "minecraft:falling_block",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if is_type(data.get_map("TileEntityData"), BED) {
                data.take("TileEntityData");
            }
        }),
    );

    // STRUCTURE: drop bed `blocks[].nbt`.
    reg.structure.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(blocks) = data.get_list_mut("blocks") {
                for block in blocks.iter_mut() {
                    if let crate::nbt::NbtValue::Compound(block) = block {
                        if is_type(block.get_map("nbt"), BED) {
                            block.take("nbt");
                        }
                    }
                }
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_entity::BlockEntity;
    use crate::dataconverter::types::ValueExt;
    use crate::dataconverter::{
        convert_entity, convert_item_stack, convert_item_stack_reverse, convert_schematic,
        convert_schematic_reverse, convert_structure,
    };
    use crate::nbt::NbtValue;
    use crate::UniversalSchematic;

    fn bed_be() -> NbtMap {
        let mut be = NbtMap::new();
        be.set_string("id", BED);
        be
    }

    #[test]
    fn item_bed_block_entity_data_dropped() {
        let mut components = NbtMap::new();
        components.set_map("minecraft:block_entity_data", bed_be());
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:red_bed");
        item.set_i32("count", 1);
        item.set_map("components", components);

        convert_item_stack(&mut item, VERSION - 1, VERSION);
        let components = item.get_map("components").unwrap();
        assert!(!components.has_key("minecraft:block_entity_data"));

        // Reverse is a no-op (nothing to restore) and lossless.
        let report = convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
        assert!(report.is_empty());
    }

    #[test]
    fn item_other_block_entity_data_kept() {
        let mut be = NbtMap::new();
        be.set_string("id", "minecraft:chest");
        let mut components = NbtMap::new();
        components.set_map("minecraft:block_entity_data", be);
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:chest");
        item.set_i32("count", 1);
        item.set_map("components", components);

        convert_item_stack(&mut item, VERSION - 1, VERSION);
        assert!(item
            .get_map("components")
            .unwrap()
            .has_key("minecraft:block_entity_data"));
    }

    #[test]
    fn falling_block_bed_tile_entity_data_dropped() {
        let mut e = NbtMap::new();
        e.set_string("id", "minecraft:falling_block");
        e.set_map("TileEntityData", bed_be());
        convert_entity(&mut e, VERSION - 1, VERSION);
        assert!(!e.has_key("TileEntityData"));
    }

    #[test]
    fn structure_bed_block_nbt_dropped() {
        let mut bed_block = NbtMap::new();
        bed_block.set_i32("state", 0);
        bed_block.set_map("nbt", bed_be());
        let mut chest = NbtMap::new();
        chest.set_string("id", "minecraft:chest");
        let mut chest_block = NbtMap::new();
        chest_block.set_i32("state", 1);
        chest_block.set_map("nbt", chest);
        let mut root = NbtMap::new();
        root.set_list(
            "blocks",
            vec![
                NbtValue::Compound(bed_block),
                NbtValue::Compound(chest_block),
            ],
        );

        convert_structure(&mut root, VERSION - 1, VERSION);
        let blocks = root.get_list("blocks").unwrap();
        assert!(!blocks[0].as_compound_ref().unwrap().has_key("nbt"));
        assert!(blocks[1].as_compound_ref().unwrap().has_key("nbt"));
    }

    #[test]
    fn schematic_bed_block_entity_dropped_forward_only() {
        let mut schem = UniversalSchematic::new("beds".to_string());
        schem
            .default_region
            .block_entities
            .insert((0, 0, 0), BlockEntity::new(BED.to_string(), (0, 0, 0)));
        schem.default_region.block_entities.insert(
            (1, 0, 0),
            BlockEntity::new("minecraft:chest".to_string(), (1, 0, 0)),
        );

        // Not crossing V4885: kept.
        convert_schematic(&mut schem, VERSION - 2, VERSION - 1);
        assert_eq!(schem.default_region.block_entities.len(), 2);

        // Crossing V4885: the bed block entity is gone, the chest stays.
        convert_schematic(&mut schem, VERSION - 1, VERSION);
        assert_eq!(schem.default_region.block_entities.len(), 1);
        assert!(schem
            .default_region
            .block_entities
            .get(&(1, 0, 0))
            .is_some());

        // Downgrading does not recreate it, and reports nothing.
        let report = convert_schematic_reverse(&mut schem, VERSION, VERSION - 1);
        assert!(report.is_empty());
        assert_eq!(schem.default_region.block_entities.len(), 1);
    }
}
