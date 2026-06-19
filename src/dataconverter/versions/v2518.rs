//! V2518 (20w12a + 3) — schematic-relevant subset of `V2518.java`.
//!
//! Jigsaw block migration:
//!   * TILE_ENTITY `minecraft:jigsaw`: `attachement_type`/`target_pool` are
//!     replaced by `name`/`target`/`pool`. Java sets `name` and `target` to the
//!     (mis-spelled) `attachement_type` value and `pool` to `target_pool`
//!     (V2518.java:27-41) — the `target` = attachement_type assignment is a known
//!     vanilla quirk and is reproduced exactly.
//!   * BLOCK_STATE: for `minecraft:jigsaw`, rename the `facing` property to
//!     `orientation`, mapping the 6 old facing values to the new orientation enum
//!     (V2518.java:43-61).
//!
//! VERSION = MCVersions.V20W12A (2515) + 3 = 2518.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 2518;

/// `FACING_RENAMES` (V2518.java:15-24): old single-axis facing -> new orientation.
fn facing_rename(facing: &str) -> &str {
    match facing {
        "down" => "down_south",
        "up" => "up_north",
        "north" => "north_up",
        "south" => "south_up",
        "west" => "west_up",
        "east" => "east_up",
        other => other,
    }
}

/// Inverse of `FACING_RENAMES`. The forward map is injective over its 6 inputs, so
/// those 6 orientations invert exactly (lossless). Modern jigsaws however support 12
/// orientations; the other 6 have no legacy single-axis `facing` preimage. For those
/// we approximate by the orientation's facing axis (the part that names the actual
/// face direction) and report a loss.
///
/// Returns `(facing, exact)` where `exact == false` means the orientation was not
/// reachable from the forward map and the result is a best-effort approximation.
fn orientation_to_facing(orientation: &str) -> (&'static str, bool) {
    match orientation {
        // Exact inverses of FACING_RENAMES.
        "down_south" => ("down", true),
        "up_north" => ("up", true),
        "north_up" => ("north", true),
        "south_up" => ("south", true),
        "west_up" => ("west", true),
        "east_up" => ("east", true),
        // Orientations with no legacy preimage: approximate by the facing axis.
        // `<axis>_*` names where axis is the actual face direction.
        "down_east" | "down_north" | "down_west" => ("down", false),
        "up_east" | "up_south" | "up_west" => ("up", false),
        "north_east" | "north_west" | "north_down" | "north_south" => ("north", false),
        "south_east" | "south_west" | "south_down" | "south_north" => ("south", false),
        "west_down" | "west_north" | "west_south" | "west_east" => ("west", false),
        "east_down" | "east_north" | "east_south" | "east_west" => ("east", false),
        _ => ("north", false),
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.tile_entity.add_converter_for_id(
        "minecraft:jigsaw",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // getString(..., "minecraft:empty") defaults when absent.
            let attach = data
                .get_string("attachement_type")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "minecraft:empty".to_string());
            let pool = data
                .get_string("target_pool")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "minecraft:empty".to_string());

            data.take("attachement_type");
            data.take("target_pool");

            data.set_string("name", attach.clone());
            data.set_string("target", attach); // vanilla quirk: target := attachement_type
            data.set_string("pool", pool);
        }),
    );

    // Reverse: id is unchanged (`minecraft:jigsaw`). The forward replaced the old
    // `attachement_type`/`target_pool` pair with `name`/`target`/`pool`, where it set
    // BOTH `name` and `target` from `attachement_type`. So `name` is the canonical
    // source of the old `attachement_type` and `pool` is the old `target_pool`.
    // Restore those and drop the modern-only fields. For forward-produced data this is
    // an exact round-trip (`name == target`). A user-edited `target` differing from
    // `name` has no legacy field, so report it before dropping it.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:jigsaw",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let name = data
                .get_string("name")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "minecraft:empty".to_string());
            let pool = data
                .get_string("pool")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "minecraft:empty".to_string());
            let target = data.get_string("target").map(|s| s.to_string());
            if let Some(target) = &target {
                if target != &name {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        format!(
                            "minecraft:jigsaw target `{target}` differs from name `{name}` and has no legacy attachement_type field"
                        ),
                    );
                }
            }

            data.take("name");
            data.take("target");
            data.take("pool");

            data.set_string("attachement_type", name);
            data.set_string("target_pool", pool);
        }),
    );

    reg.block_state.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("Name") != Some("minecraft:jigsaw") {
                return;
            }
            let properties = match data.get_map_mut("Properties") {
                Some(p) => p,
                None => return,
            };

            // getString("facing", "north") defaults when absent.
            let facing = properties
                .get_string("facing")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "north".to_string());
            properties.take("facing");
            properties.set_string("orientation", facing_rename(&facing));
        }),
    );

    // Reverse: for `minecraft:jigsaw`, rename `orientation` back to `facing`. The
    // forward `FACING_RENAMES` is injective over its 6 inputs, so those 6 orientations
    // invert exactly. Modern jigsaws support 12 orientations; the other 6 have no
    // legacy single-axis `facing` preimage and are approximated by their facing axis
    // with a reported loss.
    reg.block_state.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("Name") != Some("minecraft:jigsaw") {
                return;
            }
            let properties = match data.get_map_mut("Properties") {
                Some(p) => p,
                None => return,
            };

            // Forward defaulted absent facing to "north" -> "north_up". Mirror that by
            // defaulting an absent orientation to "north_up" before inverting.
            let orientation = properties
                .get_string("orientation")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "north_up".to_string());
            let (facing, exact) = orientation_to_facing(&orientation);
            properties.take("orientation");
            properties.set_string("facing", facing);

            if !exact {
                report_loss(
                    VERSION,
                    LossKind::RenameAmbiguous,
                    Severity::Approximated,
                    "minecraft:jigsaw orientation has no legacy single-axis facing; approximated by facing axis",
                );
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::convert_block_entity_reverse;
    use crate::dataconverter::types::MapExt;
    use crate::nbt::NbtMap;

    #[test]
    fn reverse_reports_jigsaw_target_that_differs_from_name() {
        let mut jigsaw = NbtMap::new();
        jigsaw.set_string("id", "minecraft:jigsaw");
        jigsaw.set_string("name", "minecraft:a");
        jigsaw.set_string("target", "minecraft:b");
        jigsaw.set_string("pool", "minecraft:pool");

        let report = convert_block_entity_reverse(&mut jigsaw, 2518, 2517);

        assert_eq!(jigsaw.get_string("attachement_type"), Some("minecraft:a"));
        assert_eq!(jigsaw.get_string("target_pool"), Some("minecraft:pool"));
        assert_eq!(report.loss_count(), 1);
        assert!(report.summary().contains("minecraft:b"));
    }
}
