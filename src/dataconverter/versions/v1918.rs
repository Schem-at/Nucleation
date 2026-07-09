//! V1918 (18w49a+2) — schematic-relevant subset of `V1918.java`.
//!
//! ENTITY converter (shared by `minecraft:villager` and
//! `minecraft:zombie_villager`): collapses the legacy `Profession`/`Career`/
//! `CareerLevel` triple into the modern `VillagerData` compound
//! (V1918.java:39-62). Java reads `getInt(...)` (default 0) for profession/career
//! and `getInt("CareerLevel", 1)` (default 1).
//!
//! VERSION = MCVersions.V18W49A (1916) + 2 = 1918.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::version::EncodedVersion;

const VERSION: i32 = 1918;

/// `getProfessionString` (V1918.java:13-37).
fn profession_string(profession_id: i32, career_id: i32) -> &'static str {
    match profession_id {
        0 => match career_id {
            2 => "minecraft:fisherman",
            3 => "minecraft:shepherd",
            4 => "minecraft:fletcher",
            _ => "minecraft:farmer",
        },
        1 => {
            if career_id == 2 {
                "minecraft:cartographer"
            } else {
                "minecraft:librarian"
            }
        }
        2 => "minecraft:cleric",
        3 => match career_id {
            2 => "minecraft:weaponsmith",
            3 => "minecraft:toolsmith",
            _ => "minecraft:armorer",
        },
        4 => {
            if career_id == 2 {
                "minecraft:leatherworker"
            } else {
                "minecraft:butcher"
            }
        }
        5 => "minecraft:nitwit",
        _ => "minecraft:none",
    }
}

fn convert(data: &mut NbtMap, _from: EncodedVersion, _to: EncodedVersion) {
    let profession = data.get_i32("Profession").unwrap_or(0);
    let career = data.get_i32("Career").unwrap_or(0);
    let career_level = data.get_i32("CareerLevel").unwrap_or(1);
    data.take("Profession");
    data.take("Career");
    data.take("CareerLevel");

    let mut villager_data = NbtMap::new();
    villager_data.set_string("type", "minecraft:plains");
    villager_data.set_string("profession", profession_string(profession, career));
    villager_data.set_i32("level", career_level);
    data.set_map("VillagerData", villager_data);
}

/// Inverse of `profession_string` (V1918.java:13-37).
///
/// `getProfessionString` is many-to-one: each (profession, career) pair maps to
/// one modern profession string, but several pairs (those that fall through to the
/// `else` branches — e.g. career 0/1 both → farmer) collapse to the same string.
/// For a real downgrade we recover the canonical preimage: we pick the smallest
/// `career` that yields the string, which reproduces the value the old data most
/// commonly carried. The exact original `career` for `else`-branch strings cannot
/// be recovered from modern data alone, so this is lossy.
fn profession_pair(profession: &str) -> Option<(i32, i32)> {
    match profession {
        // profession 0
        "minecraft:fisherman" => Some((0, 2)),
        "minecraft:shepherd" => Some((0, 3)),
        "minecraft:fletcher" => Some((0, 4)),
        "minecraft:farmer" => Some((0, 0)), // else-branch: career 0/1 ambiguous
        // profession 1
        "minecraft:cartographer" => Some((1, 2)),
        "minecraft:librarian" => Some((1, 0)), // else-branch
        // profession 2
        "minecraft:cleric" => Some((2, 0)), // career irrelevant in forward
        // profession 3
        "minecraft:weaponsmith" => Some((3, 2)),
        "minecraft:toolsmith" => Some((3, 3)),
        "minecraft:armorer" => Some((3, 0)), // else-branch
        // profession 4
        "minecraft:leatherworker" => Some((4, 2)),
        "minecraft:butcher" => Some((4, 0)), // else-branch
        // profession 5
        "minecraft:nitwit" => Some((5, 0)),
        // else → none
        "minecraft:none" => Some((6, 0)),
        _ => None,
    }
}

/// Reverse of `convert`: rebuild `Profession`/`Career`/`CareerLevel` from the modern
/// `VillagerData` compound (inverse of V1918.java:39-62).
///
/// Lossy: the forward `getProfessionString` collapses several (profession, career)
/// pairs into one string, so the exact `Career` cannot always be recovered. We pick
/// the canonical preimage and report loss for the genuinely-ambiguous strings. The
/// discarded `VillagerData.type` (always "minecraft:plains" on the forward) has no
/// counterpart in the old format, so it is simply dropped — not a loss.
fn revert(data: &mut NbtMap, _from: EncodedVersion, _to: EncodedVersion) {
    let villager_data = match data.take("VillagerData") {
        Some(crate::nbt::NbtValue::Compound(m)) => m,
        _ => return,
    };

    let profession = villager_data
        .get_string("profession")
        .unwrap_or("minecraft:none");
    let level = villager_data.get_i32("level").unwrap_or(1);
    if let Some(villager_type) = villager_data.get_string("type") {
        if villager_type != "minecraft:plains" {
            report_loss(
                VERSION,
                LossKind::UnsupportedInTarget,
                Severity::Loss,
                format!(
                    "VillagerData.type '{villager_type}' has no legacy Profession/Career field; dropping biome type"
                ),
            );
        }
    }

    let (profession_id, career_id) = match profession_pair(profession) {
        Some(pair) => pair,
        None => {
            report_loss(
                VERSION,
                LossKind::RenameAmbiguous,
                Severity::Approximated,
                "VillagerData.profession has no legacy Profession/Career mapping; defaulting to farmer",
            );
            (0, 0)
        }
    };

    // else-branch professions: the original career was one of several values that
    // all map to this string, so the exact value is unrecoverable.
    if matches!(
        profession,
        "minecraft:farmer"
            | "minecraft:librarian"
            | "minecraft:armorer"
            | "minecraft:butcher"
            | "minecraft:cleric"
            | "minecraft:nitwit"
            | "minecraft:none"
    ) {
        report_loss(
            VERSION,
            LossKind::RenameAmbiguous,
            Severity::Approximated,
            "VillagerData.profession collapses multiple Career values; restoring canonical Career",
        );
    }

    data.set_i32("Profession", profession_id);
    data.set_i32("Career", career_id);
    data.set_i32("CareerLevel", level);
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(convert),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:villager",
        VERSION,
        0,
        Box::new(revert),
    );
    reg.entity.add_converter_for_id(
        "minecraft:zombie_villager",
        VERSION,
        0,
        Box::new(convert),
    );
    reg.entity.add_reverse_converter_for_id(
        "minecraft:zombie_villager",
        VERSION,
        0,
        Box::new(revert),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::registry::convert_entity_reverse;

    #[test]
    fn reverse_villager_none_uses_non_mapped_profession_preimage() {
        let mut villager_data = NbtMap::new();
        villager_data.set_string("type", "minecraft:plains");
        villager_data.set_string("profession", "minecraft:none");
        villager_data.set_i32("level", 2);
        let mut villager = NbtMap::new();
        villager.set_string("id", "minecraft:villager");
        villager.set_map("VillagerData", villager_data);

        let _ = convert_entity_reverse(&mut villager, 1918, 1917);

        assert_eq!(villager.get_i32("Profession"), Some(6));
        assert_eq!(villager.get_i32("Career"), Some(0));
        assert_eq!(profession_string(6, 0), "minecraft:none");
    }

    #[test]
    fn reverse_non_plains_villager_type_reports_loss() {
        let mut villager_data = NbtMap::new();
        villager_data.set_string("type", "minecraft:desert");
        villager_data.set_string("profession", "minecraft:fletcher");
        let mut villager = NbtMap::new();
        villager.set_string("id", "minecraft:villager");
        villager.set_map("VillagerData", villager_data);

        let report = convert_entity_reverse(&mut villager, 1918, 1917);

        assert_eq!(report.loss_count(), 1);
        assert_eq!(villager.get_i32("Profession"), Some(0));
        assert_eq!(villager.get_i32("Career"), Some(4));
    }
}
