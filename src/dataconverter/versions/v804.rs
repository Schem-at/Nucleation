//! V804 (16w35a+1) — schematic-relevant subset.
//!
//! Port of DataConverterJava .../versions/V804.java. Migrates legacy banner
//! item stacks: when `tag.BlockEntityTag.Base` is a number, it becomes the item
//! `Damage` (low 4 bits), and the now-redundant `Base`/`BlockEntityTag`/`tag`
//! containers are pruned when they become empty. Skips the conversion entirely
//! (returning early, leaving the data untouched) when `tag.display.Lore` is the
//! single placeholder entry "(+NBT)", matching the Java updater.
//!
//! V804.java only touches ITEM_STACK, so nothing is skipped here.

use crate::nbt::{NbtMap, NbtValue};

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 804;

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_converter_for_id(
        "minecraft:banner",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Java: final MapType tag = data.getMap("tag"); if null return.
            if data.get_map("tag").is_none() {
                return;
            }

            // Java: final MapType blockEntity = tag.getMap("BlockEntityTag"); if null return.
            {
                let tag = data.get_map("tag").unwrap();
                if tag.get_map("BlockEntityTag").is_none() {
                    return;
                }
                let block_entity = tag.get_map("BlockEntityTag").unwrap();

                // Java: if (!blockEntity.hasKey("Base", ObjectType.NUMBER)) return;
                let base = match block_entity.get("Base").and_then(NbtValue::as_number_i64) {
                    Some(b) => b,
                    None => return,
                };

                // Java: data.setShort("Damage", (short)(blockEntity.getShort("Base") & 15));
                data.set_short("Damage", (base & 15) as i16);
            }

            // Java: if display.Lore is exactly the single STRING "(+NBT)", return (skip pruning).
            {
                let tag = data.get_map("tag").unwrap();
                if let Some(display) = tag.get_map("display") {
                    if let Some(NbtValue::List(lore)) = display.get("Lore") {
                        if lore.len() == 1 && lore[0].as_str() == Some("(+NBT)") {
                            return;
                        }
                    }
                }
            }

            // Java: blockEntity.remove("Base"); if blockEntity.isEmpty() tag.remove("BlockEntityTag");
            {
                let tag = data.get_map_mut("tag").unwrap();
                let block_entity = tag.get_map_mut("BlockEntityTag").unwrap();
                block_entity.take("Base");
                if block_entity.is_empty() {
                    tag.take("BlockEntityTag");
                }
            }

            // Java: if tag.isEmpty() data.remove("tag");
            {
                let tag = data.get_map("tag").unwrap();
                if tag.is_empty() {
                    data.take("tag");
                }
            }
        }),
    );

    // Inverse of the banner migration. The forward moved the banner base color
    // out of `tag.BlockEntityTag.Base` into the item `Damage` (low 4 bits),
    // pruning the now-empty containers. A modern `minecraft:banner` carries its
    // base color exclusively in `Damage & 15`, which uniquely determines the
    // legacy `Base` value, so reconstructing `tag.BlockEntityTag.Base` is exact
    // (lossless, rule 11) — the pre-804 format always carried this tag for
    // colored banners. We rebuild the `tag` -> `BlockEntityTag` chain that the
    // forward may have pruned and write `Base` back from `Damage`.
    reg.item_stack.add_reverse_converter_for_id(
        "minecraft:banner",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Without a `Damage` value there is no base color to restore; the
            // forward only ran when it produced one.
            let base = match data.get("Damage").and_then(NbtValue::as_number_i64) {
                Some(d) => (d & 15) as i16,
                None => return,
            };

            // Re-create the `tag` -> `BlockEntityTag` containers the forward
            // pruned (or reuse them if a modern stack still has them), then
            // restore `Base`.
            if data.get_map("tag").is_none() {
                data.set_map("tag", NbtMap::new());
            }
            let tag = data.get_map_mut("tag").unwrap();
            if tag.get_map("BlockEntityTag").is_none() {
                tag.set_map("BlockEntityTag", NbtMap::new());
            }
            let block_entity = tag.get_map_mut("BlockEntityTag").unwrap();
            block_entity.set_short("Base", base);
        }),
    );
}
