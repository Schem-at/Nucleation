//! V3447 (23w14a + 2) — schematic-relevant: pottery `_shard` -> `_sherd` item
//! renames (V3447.java:12-46).
//!
//! VERSION = MCVersions.V23W14A (3445) + 2 = 3447.
//!
//! `ConverterAbstractItemRename.register` over the 20 pottery-shard ids, each
//! renamed by replacing `_pottery_shard` with `_pottery_sherd`. Bijective and
//! trivially reversible. Nothing in V3447 is non-schematic.

use super::super::helpers::{map_renamer, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 3447;

/// `(old, new)` pottery shard->sherd renames (V3447.java:13-43). Exposed so the
/// reverse engine can invert.
pub const POTTERY_SHERD_RENAMES: &[(&str, &str)] = &[
    ("minecraft:angler_pottery_shard", "minecraft:angler_pottery_sherd"),
    ("minecraft:archer_pottery_shard", "minecraft:archer_pottery_sherd"),
    ("minecraft:arms_up_pottery_shard", "minecraft:arms_up_pottery_sherd"),
    ("minecraft:blade_pottery_shard", "minecraft:blade_pottery_sherd"),
    ("minecraft:brewer_pottery_shard", "minecraft:brewer_pottery_sherd"),
    ("minecraft:burn_pottery_shard", "minecraft:burn_pottery_sherd"),
    ("minecraft:danger_pottery_shard", "minecraft:danger_pottery_sherd"),
    ("minecraft:explorer_pottery_shard", "minecraft:explorer_pottery_sherd"),
    ("minecraft:friend_pottery_shard", "minecraft:friend_pottery_sherd"),
    ("minecraft:heart_pottery_shard", "minecraft:heart_pottery_sherd"),
    ("minecraft:heartbreak_pottery_shard", "minecraft:heartbreak_pottery_sherd"),
    ("minecraft:howl_pottery_shard", "minecraft:howl_pottery_sherd"),
    ("minecraft:miner_pottery_shard", "minecraft:miner_pottery_sherd"),
    ("minecraft:mourner_pottery_shard", "minecraft:mourner_pottery_sherd"),
    ("minecraft:plenty_pottery_shard", "minecraft:plenty_pottery_sherd"),
    ("minecraft:prize_pottery_shard", "minecraft:prize_pottery_sherd"),
    ("minecraft:sheaf_pottery_shard", "minecraft:sheaf_pottery_sherd"),
    ("minecraft:shelter_pottery_shard", "minecraft:shelter_pottery_sherd"),
    ("minecraft:skull_pottery_shard", "minecraft:skull_pottery_sherd"),
    ("minecraft:snort_pottery_shard", "minecraft:snort_pottery_sherd"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(POTTERY_SHERD_RENAMES));
}
