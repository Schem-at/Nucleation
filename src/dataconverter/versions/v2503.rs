//! V2503 (1.15.2 + 273) — schematic-relevant subset of `V2503.java`.
//!
//! BLOCK_STATE structure converter for the 14 `*_wall` block states (andesite,
//! brick, cobblestone, diorite, end_stone_brick, granite, mossy_cobblestone,
//! mossy_stone_brick, nether_brick, prismarine, red_nether_brick, red_sandstone,
//! sandstone, stone_brick). The boolean-string `east`/`west`/`north`/`south`
//! `Properties` are rewritten so that `"true"` -> `"low"` and anything else ->
//! `"none"` (V2503.java:44-64). A property that is absent is left untouched.
//!
//! VERSION = MCVersions.V1_15_2 (2230) + 273 = 2503.
//!
//! Skipped: the non-schematic ADVANCEMENTS rename
//! (`ConverterAbstractAdvancementsRename`) at V2503.java:65-69.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 2503;

const WALL_BLOCKS: &[&str] = &[
    "minecraft:andesite_wall",
    "minecraft:brick_wall",
    "minecraft:cobblestone_wall",
    "minecraft:diorite_wall",
    "minecraft:end_stone_brick_wall",
    "minecraft:granite_wall",
    "minecraft:mossy_cobblestone_wall",
    "minecraft:mossy_stone_brick_wall",
    "minecraft:nether_brick_wall",
    "minecraft:prismarine_wall",
    "minecraft:red_nether_brick_wall",
    "minecraft:red_sandstone_wall",
    "minecraft:sandstone_wall",
    "minecraft:stone_brick_wall",
];

fn change_wall_property(properties: &mut NbtMap, path: &str) {
    // Java: only act when the property exists (getString != null).
    if let Some(property) = properties.get_string(path) {
        let new_value = if property == "true" { "low" } else { "none" };
        properties.set_string(path, new_value);
    }
}

/// Inverse of `change_wall_property`: the new wall connection enum
/// (`none`/`low`/`tall`) -> the old boolean-string (`"true"`/`"false"`).
///
/// The pre-2503 format only distinguished connected (`"true"`) from not
/// connected (`"false"`). The forward mapped `"true"` -> `"low"` and anything
/// else (i.e. `"false"`) -> `"none"`. The reverse therefore maps:
///   `"low"`  -> `"true"`  (exact inverse of the forward)
///   `"none"` -> `"false"` (exact inverse of the forward)
///   `"tall"` -> `"true"`  ("tall" did not exist in the old format; a tall
///                          connection is still a connection, so the only old
///                          value that could represent it is `"true"`).
/// A modern `tall` connection did not exist in the old format. It can only be
/// approximated as connected (`"true"`), so report the lost low/tall distinction.
fn change_wall_property_reverse(properties: &mut NbtMap, path: &str) {
    if let Some(property) = properties.get_string(path) {
        let new_value = match property {
            "none" => "false",
            "low" => "true",
            "tall" => {
                report_loss(
                    VERSION,
                    LossKind::FingerprintCollapse,
                    Severity::Approximated,
                    format!(
                        "wall property `{path}` value `tall` approximated as legacy connected=true"
                    ),
                );
                "true"
            }
            other => {
                report_loss(
                    VERSION,
                    LossKind::UnsupportedInTarget,
                    Severity::Approximated,
                    format!("wall property `{path}` has unknown value `{other}`; approximated as connected=true"),
                );
                "true"
            }
        };
        properties.set_string(path, new_value);
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.block_state.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            match data.get_string("Name") {
                Some(name) if WALL_BLOCKS.contains(&name) => {}
                _ => return,
            }

            // No Properties -> nothing to do (Java returns null).
            if data.get_map("Properties").is_none() {
                return;
            }
            let properties = data.get_map_mut("Properties").unwrap();

            change_wall_property(properties, "east");
            change_wall_property(properties, "west");
            change_wall_property(properties, "north");
            change_wall_property(properties, "south");
        }),
    );

    // Reverse: new wall connection enum (none/low/tall) -> old boolean-string.
    reg.block_state.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            match data.get_string("Name") {
                Some(name) if WALL_BLOCKS.contains(&name) => {}
                _ => return,
            }

            if data.get_map("Properties").is_none() {
                return;
            }
            let properties = data.get_map_mut("Properties").unwrap();

            change_wall_property_reverse(properties, "east");
            change_wall_property_reverse(properties, "west");
            change_wall_property_reverse(properties, "north");
            change_wall_property_reverse(properties, "south");
        }),
    );
}

#[cfg(test)]
mod tests {
    use crate::dataconverter::convert_block_state_reverse;
    use crate::dataconverter::types::MapExt;
    use crate::nbt::NbtMap;

    #[test]
    fn reverse_reports_tall_wall_connection_approximation() {
        let mut properties = NbtMap::new();
        properties.set_string("east", "tall");
        let mut state = NbtMap::new();
        state.set_string("Name", "minecraft:cobblestone_wall");
        state.set_map("Properties", properties);

        let report = convert_block_state_reverse(&mut state, 2503, 2502);

        assert_eq!(
            state.get_map("Properties").unwrap().get_string("east"),
            Some("true")
        );
        assert_eq!(report.loss_count(), 0);
        assert_eq!(report.len(), 1);
        assert!(report.summary().contains("tall"));
    }
}
