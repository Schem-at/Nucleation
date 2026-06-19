//! V2690 (21w05a) — schematic-relevant subset of `V2690.java`.
//!
//! The 1.17 copper-oxidation rename pass: the early-snapshot
//! `weathered`/`semi_weathered`/`lightly_weathered` copper names are remapped to
//! the final `oxidized`/`weathered`/`exposed` naming. Applied to both ITEM_NAME
//! and the block types (BLOCK_NAME / BLOCK_STATE Name / FLAT_BLOCK_STATE)
//! (V2690.java:14-41).
//!
//! VERSION = MCVersions.V21W05A = 2690.
//!
//! Note: this rename table is NOT injective in the obvious sense — e.g.
//! `weathered_copper_block` -> `oxidized_copper_block` while
//! `semi_weathered_copper_block` -> `weathered_copper_block`. The forward map is
//! still a bijection over its key/value sets (every value is distinct), so
//! `map_renamer` applies each entry independently in one pass, matching Java's
//! single `HashMap::get` lookup (no chained re-application).

use super::super::helpers::{map_renamer, register_block_rename, register_item_rename};
use super::super::registry::RegistryBuilder;

const VERSION: i32 = 2690;

/// `(old, new)` copper-oxidation renames (V2690.java:15-36).
pub const COPPER_RENAMES: &[(&str, &str)] = &[
    ("minecraft:weathered_copper_block", "minecraft:oxidized_copper_block"),
    ("minecraft:semi_weathered_copper_block", "minecraft:weathered_copper_block"),
    ("minecraft:lightly_weathered_copper_block", "minecraft:exposed_copper_block"),
    ("minecraft:weathered_cut_copper", "minecraft:oxidized_cut_copper"),
    ("minecraft:semi_weathered_cut_copper", "minecraft:weathered_cut_copper"),
    ("minecraft:lightly_weathered_cut_copper", "minecraft:exposed_cut_copper"),
    ("minecraft:weathered_cut_copper_stairs", "minecraft:oxidized_cut_copper_stairs"),
    ("minecraft:semi_weathered_cut_copper_stairs", "minecraft:weathered_cut_copper_stairs"),
    ("minecraft:lightly_weathered_cut_copper_stairs", "minecraft:exposed_cut_copper_stairs"),
    ("minecraft:weathered_cut_copper_slab", "minecraft:oxidized_cut_copper_slab"),
    ("minecraft:semi_weathered_cut_copper_slab", "minecraft:weathered_cut_copper_slab"),
    ("minecraft:lightly_weathered_cut_copper_slab", "minecraft:exposed_cut_copper_slab"),
    ("minecraft:waxed_semi_weathered_copper", "minecraft:waxed_weathered_copper"),
    ("minecraft:waxed_lightly_weathered_copper", "minecraft:waxed_exposed_copper"),
    ("minecraft:waxed_semi_weathered_cut_copper", "minecraft:waxed_weathered_cut_copper"),
    ("minecraft:waxed_lightly_weathered_cut_copper", "minecraft:waxed_exposed_cut_copper"),
    ("minecraft:waxed_semi_weathered_cut_copper_stairs", "minecraft:waxed_weathered_cut_copper_stairs"),
    ("minecraft:waxed_lightly_weathered_cut_copper_stairs", "minecraft:waxed_exposed_cut_copper_stairs"),
    ("minecraft:waxed_semi_weathered_cut_copper_slab", "minecraft:waxed_weathered_cut_copper_slab"),
    ("minecraft:waxed_lightly_weathered_cut_copper_slab", "minecraft:waxed_exposed_cut_copper_slab"),
];

pub fn register(reg: &mut RegistryBuilder) {
    register_item_rename(reg, VERSION, map_renamer(COPPER_RENAMES));
    register_block_rename(reg, VERSION, map_renamer(COPPER_RENAMES));
}
