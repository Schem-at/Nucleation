//! V3809 (24w05a) — schematic-relevant subset of `V3809.java`.
//!
//! Re-bases the inventory `Slot` index of pack animals (llama / trader_llama /
//! mule / donkey) down by two now that the saddle/decor slots are stored
//! separately: for each entry in the `Items` list, `Slot` (default 2) becomes the
//! byte `Slot - 2` (V3809.java:18-33).
//!
//! Nothing non-schematic exists in this version file.
//!
//! VERSION = MCVersions.V24W05A (3809).

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 3809;

const PACK_ANIMALS: &[&str] = &[
    "minecraft:llama",
    "minecraft:trader_llama",
    "minecraft:mule",
    "minecraft:donkey",
];

fn convert_slots(data: &mut NbtMap) {
    if let Some(items) = data.get_list_mut("Items") {
        for el in items.iter_mut() {
            if let Some(item) = el.as_compound_mut() {
                // getInt("Slot", 2) — default 2 when absent.
                let slot = item.get_i32("Slot").unwrap_or(2);
                item.set_byte("Slot", (slot - 2) as i8);
            }
        }
    }
}

/// Inverse of [`convert_slots`]: re-bases each `Items[*].Slot` back UP by two,
/// undoing the forward `Slot - 2` (V3809.java:18-33). Lossless arithmetic: the
/// forward writes a `Slot` byte on every entry, so on reverse it is present;
/// reading the byte and adding 2 exactly restores the pre-V3809 index. The
/// pre-V3809 format stored `Slot` (default 2), so we re-emit a byte matching the
/// forward's input shape.
fn revert_slots(data: &mut NbtMap) {
    if let Some(items) = data.get_list_mut("Items") {
        for el in items.iter_mut() {
            if let Some(item) = el.as_compound_mut() {
                // Forward output: byte Slot = original - 2; default original was 2,
                // so an absent value here corresponds to original Slot 0 -> +2.
                let slot = item.get_i32("Slot").unwrap_or(0);
                item.set_byte("Slot", (slot + 2) as i8);
            }
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    for id in PACK_ANIMALS {
        reg.entity.add_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| convert_slots(data)),
        );
        reg.entity.add_reverse_converter_for_id(
            id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| revert_slots(data)),
        );
    }
}
