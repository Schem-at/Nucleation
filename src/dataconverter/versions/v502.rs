//! V502 (16w20a+1) — schematic-relevant subset of
//! `DataConverterJava/.../versions/V502.java`.
//!
//! Ports both registrations from V502.java:
//!   * the `cooked_fished` -> `cooked_fish` item rename (ITEM_NAME), and
//!   * the `Zombie` entity converter that migrates the legacy `IsVillager`
//!     boolean to the `ZombieType` profession index (ENTITY).
//!
//! Nothing in this version targets a non-schematic type, so nothing is skipped.

use rand::Rng;

use crate::nbt::NbtMap;

use super::super::helpers::{map_renamer, register_item_rename};
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 502;

pub fn register(reg: &mut RegistryBuilder) {
    // `cooked_fished` -> `cooked_fish` (V502.java:18-20). The Java side uses a
    // function rename, but it maps exactly one id, so a table renamer is
    // equivalent and trivially reversible.
    register_item_rename(
        reg,
        VERSION,
        map_renamer(&[("minecraft:cooked_fished", "minecraft:cooked_fish")]),
    );

    // Villager-zombie migration (V502.java:21-44). Only the legacy `Zombie`
    // entity id carried the `IsVillager` byte at this version.
    reg.entity.add_converter_for_id(
        "Zombie",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_bool("IsVillager") != Some(true) {
                return;
            }

            data.take("IsVillager");

            if data.has_key("ZombieType") {
                return;
            }

            // Vanilla leaves the `VillagerProfession` tag in place, so we do too.
            let mut profession = data.get_i32("VillagerProfession").unwrap_or(-1);
            if !(0..6).contains(&profession) {
                profession = rand::thread_rng().gen_range(0..6);
            }

            data.set_i32("ZombieType", profession);
        }),
    );

    // Inverse of the villager-zombie migration. A modern `Zombie` carrying a
    // `ZombieType` is a villager-zombie, so we restore the legacy encoding:
    // `IsVillager = true`, and copy the profession index back into
    // `VillagerProfession` (the forward sourced `ZombieType` from it), then drop
    // `ZombieType`. `ZombieType` uniquely encodes the old profession, so this is
    // an exact structural inverse — no loss. (The forward's random fill for an
    // invalid profession isn't recoverable, but for genuine modern data
    // `ZombieType` is the source of truth.)
    reg.entity.add_reverse_converter_for_id(
        "Zombie",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let Some(zombie_type) = data.get_i32("ZombieType") else {
                return;
            };

            data.take("ZombieType");
            match zombie_type {
                1..=5 => {
                    data.set_bool("IsVillager", true);
                    data.set_i32("VillagerProfession", zombie_type);
                }
                0 => {
                    data.set_bool("IsVillager", false);
                    report_loss(
                        VERSION,
                        LossKind::EntityMergeAmbiguous,
                        Severity::Approximated,
                        "ZombieType=0 could be a plain zombie or a villager zombie with profession 0; restoring plain zombie",
                    );
                }
                6 => {
                    data.set_bool("IsVillager", false);
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        "Husk/ZombieType=6 has no pre-V502 representation; restoring plain zombie",
                    );
                }
                other => {
                    data.set_bool("IsVillager", false);
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        format!(
                            "ZombieType={other} has no pre-V502 villager-zombie representation; restoring plain zombie"
                        ),
                    );
                }
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_entity_reverse;

    #[test]
    fn reverse_plain_zombie_type_zero_restores_plain_zombie() {
        let mut zombie = NbtMap::new();
        zombie.set_string("id", "Zombie");
        zombie.set_i32("ZombieType", 0);

        let report = convert_entity_reverse(&mut zombie, 502, 501);

        assert_eq!(zombie.get_bool("IsVillager"), Some(false));
        assert!(!zombie.has_key("ZombieType"));
        assert_eq!(report.len(), 1);
        assert_eq!(report.loss_count(), 0);
    }
}
