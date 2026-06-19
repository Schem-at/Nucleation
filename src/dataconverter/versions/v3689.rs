//! V3689 (23w44a+1, `V23W44A + 1` = 3689) — schematic-relevant subset of
//! `V3689.java`.
//!
//! Ported: the TILE_ENTITY `minecraft:trial_spawner` walker. It descends the
//! spawner's nested entity data:
//!   * for each entry of the `spawn_potentials` list, recurse `entry.data.entity`
//!     through ENTITY (Java's three-arg
//!     `WalkerUtils.convertListPath(ENTITY, data, "spawn_potentials", "data",
//!     "entity")`);
//!   * recurse `spawn_data.entity` through ENTITY.
//!
//! The shared `convert_list_path` helper only supports the two-level
//! `list_path -> element_path` form, so the three-level descent here is
//! implemented inline with NbtMap + the single-compound `convert` primitive.
//! Trial spawners are block entities, so this is schematic content. (The
//! commented-out breeze/wind_charge mob registration in Java is a no-op.)

use std::sync::Arc;

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::convert;

const VERSION: i32 = 3689;

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:trial_spawner",
        Arc::new(|reg, data, from, to| {
            // convertListPath(ENTITY, data, "spawn_potentials", "data", "entity").
            if let Some(list) = data.get_list_mut("spawn_potentials") {
                for el in list.iter_mut() {
                    if let Some(entry) = el.as_compound_mut() {
                        if let Some(inner) = entry.get_map_mut("data") {
                            convert(reg, &reg.entity, inner, "entity", from, to);
                        }
                    }
                }
            }

            // convert(ENTITY, data.getMap("spawn_data"), "entity").
            if let Some(spawn_data) = data.get_map_mut("spawn_data") {
                convert(reg, &reg.entity, spawn_data, "entity", from, to);
            }
        }),
    );
}
