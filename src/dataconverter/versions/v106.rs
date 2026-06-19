//! V106 (15w32c+2) — old-format spawner migration + first spawner walker; cites V106.java.
//!
//! Ports both UNTAGGED_SPAWNER registrations from V106.java:
//!  - structure converter: legacy `EntityId` -> `SpawnData.id`, and each
//!    `SpawnPotentials[]` entry's `Type`/`Properties` -> standard `Entity` format.
//!  - structure walker: recurse `SpawnData` and each `SpawnPotentials[].Entity`
//!    as ENTITY (the first spawner walker registered in Java).
//! No other (LEVEL/CHUNK/etc.) registrations exist in this file.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::convert_list_path;

const VERSION: i32 = 106;

pub fn register(reg: &mut RegistryBuilder) {
    // V106.java:19-65 — old-format spawner migration. Not guarded on id: a
    // spawner can be a minecart spawner, not only a tile entity (this is only
    // invoked from data walkers).
    reg.untagged_spawner.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(entity_id) = data.get_string("EntityId").map(str::to_string) {
                data.take("EntityId");
                if data.get_map("SpawnData").is_none() {
                    data.set_map("SpawnData", NbtMap::new());
                }
                let spawn_data = data.get_map_mut("SpawnData").unwrap();
                spawn_data.set_string(
                    "id",
                    if entity_id.is_empty() {
                        "Pig"
                    } else {
                        &entity_id
                    },
                );
            }

            if let Some(spawn_potentials) = data.get_list_mut("SpawnPotentials") {
                for el in spawn_potentials.iter_mut() {
                    let Some(spawn) = el.as_compound_mut() else {
                        continue;
                    };
                    let Some(spawn_type) = spawn.get_string("Type").map(str::to_string) else {
                        continue;
                    };
                    spawn.take("Type");

                    // Java: getMap("Properties") is null for a missing OR
                    // non-map value; only remove when it was actually a map.
                    let mut properties = match spawn.get_map("Properties") {
                        Some(_) => match spawn.take("Properties") {
                            Some(NbtValue::Compound(m)) => m,
                            _ => NbtMap::new(),
                        },
                        None => NbtMap::new(),
                    };
                    properties.set_string("id", &spawn_type);

                    spawn.set_map("Entity", properties);
                }
            }
        }),
    );

    // Reverse of V106.java:19-65 — restore the legacy spawner shape (new -> old).
    // The walker (registered below) descends FIRST in reverse, so by the time
    // this runs the ENTITY children inside `SpawnData` and each
    // `SpawnPotentials[].Entity` are already downgraded; we only restructure the
    // spawner's own container fields here. Lossless (bucket B): the forward moves
    // are 1:1 field relocations whose new shape uniquely encodes the old shape.
    reg.untagged_spawner.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Inverse of EntityId -> SpawnData.id: pull the id back out to the
            // top-level `EntityId` string and drop it from `SpawnData`.
            // (Forward maps an empty EntityId to "Pig"; an empty source is a
            // degenerate spawner, so restoring EntityId = id — i.e. "Pig" — is the
            // canonical preimage and not treated as loss.)
            if let Some(spawn_data) = data.get_map_mut("SpawnData") {
                if let Some(id) = spawn_data.get_string("id").map(str::to_string) {
                    spawn_data.take("id");
                    data.set_string("EntityId", &id);
                }
            }

            // Inverse of each SpawnPotentials[] entry's Type/Properties -> Entity:
            // split `Entity` back into the `Type` string (its id) and the
            // remaining map as `Properties`. The forward only created `Properties`
            // from a pre-existing map, so we only re-add it when the leftover map
            // is non-empty, matching the old shape.
            if let Some(spawn_potentials) = data.get_list_mut("SpawnPotentials") {
                for el in spawn_potentials.iter_mut() {
                    let Some(spawn) = el.as_compound_mut() else {
                        continue;
                    };
                    let Some(entity) = spawn.get_map("Entity") else {
                        continue;
                    };
                    let Some(spawn_type) = entity.get_string("id").map(str::to_string) else {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            "SpawnPotentials[].Entity is missing id; cannot restore legacy Type",
                        );
                        continue;
                    };
                    let Some(NbtValue::Compound(mut entity)) = spawn.take("Entity") else {
                        continue;
                    };
                    entity.take("id");

                    spawn.set_string("Type", &spawn_type);
                    if !entity.is_empty() {
                        spawn.set_map("Properties", entity);
                    }
                }
            }
        }),
    );

    // V106.java:67-79 — first spawner walker: recurse SpawnData and each
    // SpawnPotentials[].Entity as ENTITY.
    reg.untagged_spawner.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, data, from, to| {
            convert_list_path(
                reg,
                &reg.entity,
                data,
                "SpawnPotentials",
                "Entity",
                from,
                to,
            );
            super::super::walker::convert(reg, &reg.entity, data, "SpawnData", from, to);
        }),
    );
}
