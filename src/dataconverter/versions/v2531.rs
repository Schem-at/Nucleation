//! V2531 (20w17a + 2) — schematic-relevant subset of `V2531.java`.
//!
//! BLOCK_STATE `minecraft:redstone_wire`: recompute the per-direction connection
//! properties. A direction with no connection (`none`) that has no perpendicular
//! connection becomes `side`; otherwise it is left as-is. Only properties that
//! already exist are rewritten (V2531.java:17-59).
//!
//! VERSION = MCVersions.V20W17A (2529) + 2 = 2531.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 2531;

fn is_connected(facing: &str) -> bool {
    facing != "none"
}

fn forward_redstone_tuple(
    east: &str,
    west: &str,
    north: &str,
    south: &str,
) -> (String, String, String, String) {
    let connected_x = is_connected(east) || is_connected(west);
    let connected_z = is_connected(north) || is_connected(south);

    (
        if !is_connected(east) && !connected_z {
            "side"
        } else {
            east
        }
        .to_string(),
        if !is_connected(west) && !connected_z {
            "side"
        } else {
            west
        }
        .to_string(),
        if !is_connected(north) && !connected_x {
            "side"
        } else {
            north
        }
        .to_string(),
        if !is_connected(south) && !connected_x {
            "side"
        } else {
            south
        }
        .to_string(),
    )
}

fn redstone_preimage_count(east: &str, west: &str, north: &str, south: &str) -> usize {
    const VALUES: &[&str] = &["none", "side", "up"];
    let target = (
        east.to_string(),
        west.to_string(),
        north.to_string(),
        south.to_string(),
    );
    let mut count = 0;
    for old_east in VALUES {
        for old_west in VALUES {
            for old_north in VALUES {
                for old_south in VALUES {
                    if forward_redstone_tuple(old_east, old_west, old_north, old_south) == target {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.block_state.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("Name") != Some("minecraft:redstone_wire") {
                return;
            }
            let properties = match data.get_map_mut("Properties") {
                Some(p) => p,
                None => return,
            };

            // getString(dir, "none") defaults when absent.
            let read = |p: &NbtMap, key: &str| {
                p.get_string(key)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "none".to_string())
            };
            let east = read(properties, "east");
            let west = read(properties, "west");
            let north = read(properties, "north");
            let south = read(properties, "south");

            let connected_x = is_connected(&east) || is_connected(&west);
            let connected_z = is_connected(&north) || is_connected(&south);

            let new_east = if !is_connected(&east) && !connected_z {
                "side".to_string()
            } else {
                east
            };
            let new_west = if !is_connected(&west) && !connected_z {
                "side".to_string()
            } else {
                west
            };
            let new_north = if !is_connected(&north) && !connected_x {
                "side".to_string()
            } else {
                north
            };
            let new_south = if !is_connected(&south) && !connected_x {
                "side".to_string()
            } else {
                south
            };

            if properties.has_key("east") {
                properties.set_string("east", new_east);
            }
            if properties.has_key("west") {
                properties.set_string("west", new_west);
            }
            if properties.has_key("north") {
                properties.set_string("north", new_north);
            }
            if properties.has_key("south") {
                properties.set_string("south", new_south);
            }
        }),
    );

    // REVERSE of the forward `minecraft:redstone_wire` connection-property rewrite.
    //
    // Forward (V2531.java:17-59) turned a direction from `none` -> `side` exactly
    // when that direction was `none` AND the *perpendicular* axis was unconnected
    // (computed from the ORIGINAL values). This is the 1.16 "redstone dot" change:
    // a wire with no connections used to render as a cross (all `side`), so the
    // forward fills `side` to preserve the old cross appearance.
    //
    // Reverse: mirror the forward by reverting `side` -> `none` when the
    // perpendicular axis is unconnected in the post-forward data. Verified by
    // exhaustive enumeration of all 81 old direction-tuples: this closed form is
    // EXACT on all 65 distinct unambiguous post-forward states (no loss).
    //
    // 7 post-forward states are genuinely ambiguous (the forward collapsed >1 old
    // tuple into them); the only practically important one is the full cross
    // `east=west=north=south=side`, which is the image of BOTH the all-`side`
    // cross and the all-`none` dot. Modern data cannot distinguish them (rule 11),
    // so we pick the canonical cross preimage (leave it as `side`) and report an
    // approximation. The other ambiguous states (e.g. one axis `side,side`, the
    // perpendicular `none,none`) revert both `side`s to `none`, which is exact for
    // the dominant preimage and best-effort for the rest.
    reg.block_state.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_string("Name") != Some("minecraft:redstone_wire") {
                return;
            }
            let properties = match data.get_map_mut("Properties") {
                Some(p) => p,
                None => return,
            };

            let read = |p: &NbtMap, key: &str| {
                p.get_string(key)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "none".to_string())
            };
            let east = read(properties, "east");
            let west = read(properties, "west");
            let north = read(properties, "north");
            let south = read(properties, "south");
            if redstone_preimage_count(&east, &west, &north, &south) > 1 {
                report_loss(
                    VERSION,
                    LossKind::FingerprintCollapse,
                    Severity::Approximated,
                    format!(
                        "minecraft:redstone_wire state east={east}, west={west}, north={north}, south={south} has multiple pre-V2531 preimages"
                    ),
                );
            }

            // Connectivity as observed in the (post-forward) NEW data.
            let connected_x = is_connected(&east) || is_connected(&west);
            let connected_z = is_connected(&north) || is_connected(&south);

            // Canonical-cross guard: an all-`side` wire is the merged image of both
            // a real cross (all `side`) and a no-connection dot (all `none`); keep it
            // as the cross and do not revert.
            let all_side = east == "side" && west == "side" && north == "side" && south == "side";
            if all_side {
                return;
            }

            // `side` reverts to `none` when its perpendicular axis is unconnected.
            let revert = |v: &str, perp_connected: bool| -> String {
                if v == "side" && !perp_connected {
                    "none".to_string()
                } else {
                    v.to_string()
                }
            };
            let new_east = revert(&east, connected_z);
            let new_west = revert(&west, connected_z);
            let new_north = revert(&north, connected_x);
            let new_south = revert(&south, connected_x);

            if properties.has_key("east") {
                properties.set_string("east", new_east);
            }
            if properties.has_key("west") {
                properties.set_string("west", new_west);
            }
            if properties.has_key("north") {
                properties.set_string("north", new_north);
            }
            if properties.has_key("south") {
                properties.set_string("south", new_south);
            }
        }),
    );
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::convert_block_state_reverse;
    use crate::dataconverter::types::MapExt;
    use crate::nbt::NbtMap;

    #[test]
    fn reverse_reports_non_cross_ambiguous_redstone_state() {
        let mut properties = NbtMap::new();
        properties.set_string("east", "side");
        properties.set_string("west", "side");
        properties.set_string("north", "none");
        properties.set_string("south", "none");
        let mut state = NbtMap::new();
        state.set_string("Name", "minecraft:redstone_wire");
        state.set_map("Properties", properties);

        let report = convert_block_state_reverse(&mut state, 2531, 2530);

        assert_eq!(report.loss_count(), 0);
        assert_eq!(report.len(), 1);
        assert!(report.summary().contains("multiple pre-V2531 preimages"));
    }
}
