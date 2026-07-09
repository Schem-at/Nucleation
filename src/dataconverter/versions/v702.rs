//! V702 (1.11 snapshot, MCVersions.V1_10_2 + 190 = 702) — schematic-relevant
//! subset of V702.java.
//!
//! Splits the legacy `Zombie` entity into `Zombie` / `ZombieVillager` / `Husk`
//! based on the int `ZombieType` (which is then removed). For types 1..=5 the id
//! becomes `ZombieVillager` and a `Profession` int is set; type 6 becomes `Husk`.
//! Schematic-relevant: mutates an ENTITY's `id` (+ adds `Profession`).
//!
//! Also registers the `ZombieVillager` walker, which descends `Offers.Recipes`
//! into VILLAGER_TRADE.
//!
//! SKIPPED: the commented-out `registerMob("Husk")` is a no-op.

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::convert_list;

const VERSION: i32 = 702;

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "Zombie",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Java: data.getInt("ZombieType") defaults to 0 when absent.
            let zombie_type = data.get_i32("ZombieType").unwrap_or(0);
            data.take("ZombieType");

            match zombie_type {
                1..=5 => {
                    data.set_string("id", "ZombieVillager");
                    data.set_i32("Profession", zombie_type - 1);
                }
                6 => data.set_string("id", "Husk"),
                _ => {}
            }
        }),
    );

    // Reverse of V702.java:14-41. The forward split the legacy `Zombie` entity
    // into `Zombie` / `ZombieVillager` / `Husk` by reading the int `ZombieType`
    // (then removing it). Each branch is exactly invertible (rule 11):
    //   - `ZombieVillager` carries `Profession`, which uniquely encodes the old
    //     discriminator: `ZombieType = Profession + 1` (forward set
    //     `Profession = ZombieType - 1` for ZombieType in 1..=5). Lossless.
    //   - `Husk` maps unambiguously to `ZombieType = 6`. Lossless.
    //   - a plain `Zombie` had `ZombieType` removed because it was 0 (the
    //     default the legacy format always carried); restoring `ZombieType = 0`
    //     is exact, not data loss.
    //
    // `add_reverse_converter_for_id` matches the NEW id (the forward output id);
    // any inverse id-rename runs later in the descending sweep, so we match the
    // un-namespaced ids this converter emits.
    reg.entity.add_reverse_converter_for_id(
        "ZombieVillager",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Profession defaults to 0 if absent => ZombieType 1 (canonical).
            let profession = data.get_i32("Profession").unwrap_or(0);
            data.take("Profession");
            data.set_string("id", "Zombie");
            data.set_i32("ZombieType", profession + 1);
        }),
    );
    reg.entity.add_reverse_converter_for_id(
        "Husk",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            data.set_string("id", "Zombie");
            data.set_i32("ZombieType", 6);
        }),
    );
    reg.entity.add_reverse_converter_for_id(
        "Zombie",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Plain Zombie: forward removed `ZombieType` (it was 0); restore the
            // default the legacy format always carried.
            data.set_i32("ZombieType", 0);
        }),
    );

    // Java: ENTITY.addWalker(VERSION, "ZombieVillager", ...) descends
    // Offers.Recipes into VILLAGER_TRADE via WalkerUtils.convertList.
    reg.entity.add_walker(
        VERSION,
        0,
        "ZombieVillager",
        Arc::new(|reg, data, from, to| {
            if let Some(offers) = data.get_map_mut("Offers") {
                convert_list(reg, &reg.villager_trade, offers, "Recipes", from, to);
            }
        }),
    );
}
