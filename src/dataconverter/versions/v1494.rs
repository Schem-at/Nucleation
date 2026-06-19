//! V1494 (18w20c+1) — schematic-relevant subset of `V1494.java`.
//!
//! VERSION = MCVersions.V18W20C + 1 = 1493 + 1 = 1494.
//!
//! Ported: the ITEM_STACK structure converter that migrates numeric enchantment
//! ids to namespaced enchantment names. Inside `tag`:
//!   * the `ench` list is renamed to `Enchantments`, and each entry's integer
//!     `id` is rewritten to its `minecraft:<name>` string (default `"null"`);
//!   * each entry of `StoredEnchantments` gets the same int->name `id` rewrite.
//!
//! This is the whole of V1494; nothing is skipped.

use crate::nbt::NbtValue;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 1494;

/// `ENCH_ID_TO_NAME` (V1494.java:15-51): numeric enchantment id -> namespaced
/// name. Lookup falls back to `"null"` for any unmapped id.
const ENCH_ID_TO_NAME: &[(i32, &str)] = &[
    (0, "minecraft:protection"),
    (1, "minecraft:fire_protection"),
    (2, "minecraft:feather_falling"),
    (3, "minecraft:blast_protection"),
    (4, "minecraft:projectile_protection"),
    (5, "minecraft:respiration"),
    (6, "minecraft:aqua_affinity"),
    (7, "minecraft:thorns"),
    (8, "minecraft:depth_strider"),
    (9, "minecraft:frost_walker"),
    (10, "minecraft:binding_curse"),
    (16, "minecraft:sharpness"),
    (17, "minecraft:smite"),
    (18, "minecraft:bane_of_arthropods"),
    (19, "minecraft:knockback"),
    (20, "minecraft:fire_aspect"),
    (21, "minecraft:looting"),
    (22, "minecraft:sweeping"),
    (32, "minecraft:efficiency"),
    (33, "minecraft:silk_touch"),
    (34, "minecraft:unbreaking"),
    (35, "minecraft:fortune"),
    (48, "minecraft:power"),
    (49, "minecraft:punch"),
    (50, "minecraft:flame"),
    (51, "minecraft:infinity"),
    (61, "minecraft:luck_of_the_sea"),
    (62, "minecraft:lure"),
    (65, "minecraft:loyalty"),
    (66, "minecraft:impaling"),
    (67, "minecraft:riptide"),
    (68, "minecraft:channeling"),
    (70, "minecraft:mending"),
    (71, "minecraft:vanishing_curse"),
];

/// `getOrDefault(id, "null")` over [`ENCH_ID_TO_NAME`].
fn ench_name(id: i32) -> &'static str {
    ENCH_ID_TO_NAME
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| *v)
        .unwrap_or("null")
}

/// Rewrite each map element's integer `id` to its enchantment name.
fn rewrite_enchant_ids(list: &mut [NbtValue]) {
    for el in list.iter_mut() {
        if let Some(enchant) = el.as_compound_mut() {
            // getInt defaults to 0 when absent / non-numeric.
            let id = enchant.get_i64("id").unwrap_or(0) as i32;
            enchant.set_string("id", ench_name(id));
        }
    }
}

/// Inverse of [`ench_name`]: namespaced enchantment name -> numeric id.
/// `ENCH_ID_TO_NAME` is injective over its mapped ids, so this is exact for any
/// name the forward could have produced from a mapped id. Returns `None` for a
/// name absent from the table (e.g. `"null"`, which the forward emitted for any
/// unmapped int id — a genuinely lossy collapse with no surviving discriminator).
fn ench_id(name: &str) -> Option<i32> {
    ENCH_ID_TO_NAME
        .iter()
        .find(|(_, v)| *v == name)
        .map(|(k, _)| *k)
}

/// Inverse of [`rewrite_enchant_ids`]: rewrite each map element's string `id`
/// back to its numeric id. Mapped names round-trip exactly; an unmapped name
/// (the forward's `"null"` sentinel) cannot be restored, so the int is left as
/// the lookup default `0` and the loss is reported once per occurrence.
fn restore_enchant_ids(list: &mut [NbtValue]) {
    for el in list.iter_mut() {
        if let Some(enchant) = el.as_compound_mut() {
            let name = enchant.get_string("id").unwrap_or("").to_string();
            match ench_id(&name) {
                Some(id) => enchant.set_i32("id", id),
                None => {
                    // Forward mapped unknown ints to "null" (getOrDefault), so the
                    // original numeric id is unrecoverable. Best-effort: 0.
                    report_loss(
                        VERSION,
                        LossKind::RenameAmbiguous,
                        Severity::Loss,
                        "enchantment id has no numeric preimage; restored to 0",
                    );
                    enchant.set_i32("id", 0);
                }
            }
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let tag = match data.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };

            // `ench` -> `Enchantments`, then rewrite ids.
            if let Some(NbtValue::List(mut enchants)) = tag.take("ench") {
                rewrite_enchant_ids(&mut enchants);
                tag.set_list("Enchantments", enchants);
            }

            // `StoredEnchantments`: rewrite ids in place.
            if let Some(stored) = tag.get_list_mut("StoredEnchantments") {
                rewrite_enchant_ids(stored);
            }
        }),
    );

    // Reverse: undo the int->name enchantment migration. `Enchantments` ->
    // `ench`, restoring numeric `id`s, and the same name->int rewrite for
    // `StoredEnchantments`. The forward name table is injective over mapped ids
    // so mapped names round-trip exactly (lossless); only the `"null"` sentinel
    // emitted for unmapped ints is unrecoverable (reported in
    // `restore_enchant_ids`).
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let tag = match data.get_map_mut("tag") {
                Some(t) => t,
                None => return,
            };

            // `Enchantments` -> `ench`, then restore numeric ids.
            if let Some(NbtValue::List(mut enchants)) = tag.take("Enchantments") {
                restore_enchant_ids(&mut enchants);
                tag.set_list("ench", enchants);
            }

            // `StoredEnchantments`: restore numeric ids in place.
            if let Some(stored) = tag.get_list_mut("StoredEnchantments") {
                restore_enchant_ids(stored);
            }
        }),
    );
}
