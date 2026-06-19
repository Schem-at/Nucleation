//! V2831 (1.17.1 + 101) — schematic-relevant subset of `V2831.java`.
//!
//! Ported (UNTAGGED_SPAWNER, which appears in schematics inside mob-spawner
//! block entities / spawner minecarts):
//!   * Structure converter: migrate the spawner to the new weighted-list format —
//!     `SpawnData` is wrapped as `{ entity: <old SpawnData> }`, and each
//!     `SpawnPotentials` entry's `Entity`/`Weight` pair becomes
//!     `{ weight: <Weight>, data: { entity: <Entity> } }`.
//!   * Structure walker (new format): recurse `SpawnPotentials[].data.entity` and
//!     `SpawnData.entity` as ENTITY.
//!
//! Nothing non-schematic exists in this version file.
//!
//! VERSION = MCVersions.V1_17_1 (2730) + 101 = 2831.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::convert;

const VERSION: i32 = 2831;

pub fn register(reg: &mut RegistryBuilder) {
    // New-format UNTAGGED_SPAWNER structure walker (V2831.java:17-23).
    reg.untagged_spawner.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, root, from, to| {
            // WalkerUtils.convertListPath(ENTITY, root, "SpawnPotentials", "data", "entity"):
            // for each entry in the SpawnPotentials list, descend data.entity and
            // convert as ENTITY. (The Rust `convert_list_path` only descends one
            // level, so the inner `data` -> `entity` hop is done inline.)
            if let Some(list) = root.get_list_mut("SpawnPotentials") {
                for el in list.iter_mut() {
                    if let Some(entry) = el.as_compound_mut() {
                        if let Some(data) = entry.get_map_mut("data") {
                            convert(reg, &reg.entity, data, "entity", from, to);
                        }
                    }
                }
            }

            // WalkerUtils.convert(ENTITY, root.getMap("SpawnData"), "entity"):
            // descend into the SpawnData compound and convert its `entity` child.
            if let Some(spawn_data) = root.get_map_mut("SpawnData") {
                convert(reg, &reg.entity, spawn_data, "entity", from, to);
            }
        }),
    );

    // UNTAGGED_SPAWNER structure converter (V2831.java:25-60): migrate to the new
    // weighted-list / wrapped-entity format.
    reg.untagged_spawner.add_structure_converter(
        VERSION,
        0,
        Box::new(|root: &mut NbtMap, _from, _to| {
            // SpawnData -> { entity: <old SpawnData> }.
            if let Some(NbtValue::Compound(spawn_data)) = root.take("SpawnData") {
                let mut wrapped = NbtMap::new();
                wrapped.set_map("entity", spawn_data);
                root.set_map("SpawnData", wrapped);
            }

            // Each SpawnPotentials entry: Entity/Weight -> weight + data.entity.
            if let Some(spawn_potentials) = root.get_list_mut("SpawnPotentials") {
                for el in spawn_potentials.iter_mut() {
                    let entry = match el.as_compound_mut() {
                        Some(m) => m,
                        None => continue,
                    };

                    // getInt("Weight", 1) — default 1 when absent.
                    let weight = entry.get_i32("Weight").unwrap_or(1);
                    let entity = entry.take("Entity");
                    entry.take("Weight");

                    entry.set_i32("weight", weight);

                    let mut data = NbtMap::new();
                    // setMap("entity", entity): Java passes the (possibly null)
                    // Entity map straight through; mirror by only inserting when
                    // it was actually a compound.
                    if let Some(NbtValue::Compound(entity_map)) = entity {
                        data.set_map("entity", entity_map);
                    }
                    entry.set_map("data", data);
                }
            }
        }),
    );

    // Reverse of the UNTAGGED_SPAWNER structure converter (V2831.java:25-60).
    // Lossless structural inverse (bucket B): both transforms are exact splits
    // that fully encode the old shape, so nothing is lost on a real downgrade.
    //
    // Order note (rule 3): by the time this runs, the walker has already
    // descended `SpawnData.entity` and `SpawnPotentials[].data.entity` as
    // ENTITY and downgraded those children. The restructured fields hold ENTITY
    // (not UNTAGGED_SPAWNER), so rule 12 (self-recursion) does NOT apply — we
    // simply move the already-converted child compounds back to their old keys.
    reg.untagged_spawner.add_reverse_converter(
        VERSION,
        0,
        Box::new(|root: &mut NbtMap, _from, _to| {
            // Undo SpawnData wrapping: { entity: <X> } -> <X>.
            if let Some(NbtValue::Compound(mut wrapped)) = root.take("SpawnData") {
                // Forward always wrapped the old SpawnData under `entity`; if the
                // child entity survives, restore it as the SpawnData compound.
                if let Some(NbtValue::Compound(entity)) = wrapped.take("entity") {
                    if !wrapped.is_empty() {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            "SpawnData wrapper had modern-only fields outside `entity`; dropping them for legacy spawner format",
                        );
                    }
                    root.set_map("SpawnData", entity);
                } else {
                    if !wrapped.is_empty() {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            "SpawnData wrapper had no compound `entity`; preserving empty legacy SpawnData and dropping wrapper fields",
                        );
                    }
                    // No inner entity (e.g. an empty wrapper): put back an empty
                    // SpawnData to mirror the forward having created the wrapper.
                    root.set_map("SpawnData", NbtMap::new());
                }
            }

            // Undo each SpawnPotentials entry: weight + data.entity -> Weight + Entity.
            if let Some(spawn_potentials) = root.get_list_mut("SpawnPotentials") {
                for el in spawn_potentials.iter_mut() {
                    let entry = match el.as_compound_mut() {
                        Some(m) => m,
                        None => continue,
                    };

                    // Forward wrote `weight` from getInt("Weight", 1); restore
                    // the old `Weight` int (default 1 mirrors that default).
                    let weight = entry.get_i32("weight").unwrap_or(1);
                    entry.take("weight");
                    entry.set_i32("Weight", weight);

                    // `data` is `{ entity: <Entity> }`; pull the entity back to
                    // the old top-level `Entity` key and drop the `data` wrapper.
                    if let Some(data_value) = entry.take("data") {
                        match data_value {
                            NbtValue::Compound(mut data) => {
                                if let Some(NbtValue::Compound(entity)) = data.take("entity") {
                                    if !data.is_empty() {
                                        report_loss(
                                            VERSION,
                                            LossKind::UnsupportedInTarget,
                                            Severity::Loss,
                                            "SpawnPotentials entry data wrapper had modern-only fields outside `entity`; dropping them for legacy spawner format",
                                        );
                                    }
                                    entry.set_map("Entity", entity);
                                } else if !data.is_empty() {
                                    report_loss(
                                        VERSION,
                                        LossKind::UnsupportedInTarget,
                                        Severity::Loss,
                                        "SpawnPotentials entry data wrapper had no compound `entity`; dropping wrapper fields",
                                    );
                                }
                            }
                            _ => report_loss(
                                VERSION,
                                LossKind::UnsupportedInTarget,
                                Severity::Loss,
                                "SpawnPotentials entry `data` was not a compound and cannot be represented before V2831",
                            ),
                        }
                    }
                }
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::loss;
    use crate::dataconverter::registry::{convert_reverse_under_session, registry};
    use crate::dataconverter::types::{MapExt, ValueExt};
    use crate::nbt::{NbtMap, NbtValue};

    #[test]
    fn reverse_reports_spawner_data_wrapper_leftovers() {
        let mut entity = NbtMap::new();
        entity.set_string("id", "minecraft:pig");
        let mut data = NbtMap::new();
        data.set_map("entity", entity);
        data.set_string("custom", "lost");
        let mut entry = NbtMap::new();
        entry.set_i32("weight", 2);
        entry.set_map("data", data);
        let mut spawner = NbtMap::new();
        spawner.set_list("SpawnPotentials", vec![NbtValue::Compound(entry)]);

        let reg = registry();
        let (_, report) = loss::run_reverse(|| {
            convert_reverse_under_session(&reg.untagged_spawner, &mut spawner, 2831, 2830);
        });

        let entry = spawner.get_list("SpawnPotentials").unwrap()[0]
            .as_compound_ref()
            .unwrap();
        assert_eq!(entry.get_i32("Weight"), Some(2));
        assert_eq!(
            entry.get_map("Entity").unwrap().get_string("id"),
            Some("minecraft:pig")
        );
        assert_eq!(report.loss_count(), 1);
        assert!(report.summary().contains("modern-only fields"));
    }
}
