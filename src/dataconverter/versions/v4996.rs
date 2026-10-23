//! V4996 (26.3 snapshot) — pot decorations become per-side item stacks.
//!
//! Upstream DataConverter has no step past V4892 yet, so this ports Mojang's
//! 26.3 datafixer directly (`net.minecraft.util.datafix.DataFixers`, schema
//! 4996 and 4996:1; Mojang 26.3 ships unobfuscated). Both parts are
//! schematic-relevant:
//!
//! * Step 0 — `PotDecorationsComponentUnflatteningFix` (schema `V4996`):
//!   the `minecraft:pot_decorations` DATA_COMPONENT changes from a list of up to
//!   four item ids (`[back, left, right, front]`) to a map
//!   `{back:{id}, left:{id}, right:{id}, front:{id}}` whose values are
//!   ITEM_STACKs. Missing / non-string list entries become `minecraft:brick`;
//!   an empty-string id leaves the component untouched
//!   (`PotDecorationsComponentUnflatteningFix.unpackList`). Schema `V4996`
//!   re-declares DATA_COMPONENTS with the four sides typed as ITEM_STACK, so the
//!   DATA_COMPONENTS walker is re-registered here.
//! * Step 1 — `PotDecorationsBlockEntityUnflatteningFix` (schema `V4996_1`):
//!   the same unpacking for the `minecraft:decorated_pot` block entity's
//!   `sherds`. A missing `sherds` is read as an empty list (`orElseEmptyList`),
//!   so every decorated pot gains an all-brick `sherds` map. Schema `V4996_1`
//!   re-declares the decorated_pot walker (`sherds` sides + `item` as
//!   ITEM_STACK).
//!
//! Reverse: pack the map back into the four-entry id list (absent side ->
//! `minecraft:brick`, which is also what 26.2 `PotDecorations.ordered()`
//! writes). A side carrying more than an item id (a count other than 1, or
//! components) cannot be represented by the old id list and is reported as
//! loss. For the block entity, an all-brick `sherds` is dropped again, since
//! 26.2 omitted `sherds` for `PotDecorations.EMPTY` (bucket D).
//!
//! VERSION = 4996 (Mojang schema id; no MCVersions constant upstream yet).

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::{Registry, RegistryBuilder};
use super::super::types::{MapExt, ValueExt};
use super::super::version::EncodedVersion;
use super::super::walker::{convert, items};

const VERSION: i32 = 4996;

const BRICK: &str = "minecraft:brick";
/// Side names in list order (`PotDecorationsComponentUnflatteningFix`, `switch (i)`).
const SIDES: [&str; 4] = ["back", "left", "right", "front"];

/// `PotDecorationsComponentUnflatteningFix.unpackList`. Returns `None` when the
/// value must be left as-is (not a list, or an empty-string id).
fn unpack_list(original: &NbtValue) -> Option<NbtValue> {
    let list = original.as_list_ref()?;
    let mut result = NbtMap::new();
    for (i, side) in SIDES.iter().enumerate() {
        let id = match list.get(i) {
            Some(el) => el.as_str().unwrap_or(BRICK),
            None => BRICK,
        };
        if id.is_empty() {
            return None;
        }
        let mut stack = NbtMap::new();
        stack.set_string("id", id);
        result.set_map(side, stack);
    }
    Some(NbtValue::Compound(result))
}

/// Inverse of [`unpack_list`]: the per-side map back to the four-entry id list.
/// Returns `None` when the value is not a map (left untouched).
fn pack_map(value: &NbtValue, what: &str) -> Option<Vec<NbtValue>> {
    let map = value.as_compound_ref()?;
    let mut out = Vec::with_capacity(4);
    for side in SIDES {
        let id = match map.get(side) {
            Some(NbtValue::String(id)) => id.clone(),
            Some(NbtValue::Compound(stack)) => {
                let count = stack.get_i32("count").unwrap_or(1);
                let has_components = stack.get_map("components").is_some_and(|c| !c.is_empty());
                if count != 1 || has_components {
                    report_loss(
                        VERSION,
                        LossKind::ComponentDropped,
                        Severity::Loss,
                        format!(
                            "{what} side '{side}' is a full item stack (count/components); the pre-26.3 format only stores an item id"
                        ),
                    );
                }
                stack.get_string("id").unwrap_or(BRICK).to_string()
            }
            _ => BRICK.to_string(),
        };
        out.push(NbtValue::String(id));
    }
    Some(out)
}

/// Walk the four ITEM_STACK sides of a pot-decorations map.
fn walk_sides(
    reg: &Registry,
    data: &mut NbtMap,
    path: &str,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    if let Some(sides) = data.get_map_mut(path) {
        for side in SIDES {
            convert(reg, &reg.item_stack, sides, side, from, to);
        }
    }
}

fn is_all_brick(value: Option<&NbtValue>) -> bool {
    let Some(map) = value.and_then(|v| v.as_compound_ref()) else {
        return false;
    };
    map.len() == SIDES.len()
        && SIDES.iter().all(|side| match map.get(side) {
            Some(NbtValue::String(id)) => id == BRICK,
            Some(NbtValue::Compound(stack)) => {
                stack.get_string("id") == Some(BRICK)
                    && stack.keys().iter().all(|k| k == "id" || k == "count")
                    && stack.get_i32("count").unwrap_or(1) == 1
            }
            _ => false,
        })
}

pub fn register(reg: &mut RegistryBuilder) {
    // --- step 0: DATA_COMPONENTS -------------------------------------------
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(unpacked) = data.get("minecraft:pot_decorations").and_then(unpack_list) {
                data.set_generic("minecraft:pot_decorations", unpacked);
            }
        }),
    );
    // Reverse: per-side map -> id list.
    reg.data_components.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(list) = data
                .get("minecraft:pot_decorations")
                .and_then(|v| pack_map(v, "minecraft:pot_decorations"))
            {
                data.set_list("minecraft:pot_decorations", list);
            }
        }),
    );
    // Schema V4996: the post-V4307 DATA_COMPONENTS walker, with
    // `minecraft:pot_decorations` now four ITEM_STACK sides instead of an
    // ITEM_NAME list. (The V4307 walker's ITEM_NAME list walk is a no-op on the
    // map shape.)
    reg.data_components.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, data, from, to| {
            super::v4307::data_components_walker(reg, data, from, to);
            walk_sides(reg, data, "minecraft:pot_decorations", from, to);
        }),
    );

    // --- step 1: decorated_pot block entity --------------------------------
    reg.tile_entity.add_converter_for_id(
        "minecraft:decorated_pot",
        VERSION,
        1,
        Box::new(|data, _from, _to| {
            let original = data
                .take("sherds")
                .unwrap_or_else(|| NbtValue::List(Vec::new()));
            let unpacked = unpack_list(&original).unwrap_or(original);
            data.set_generic("sherds", unpacked);
        }),
    );
    // Reverse: drop the all-brick default the forward adds, else pack the map.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:decorated_pot",
        VERSION,
        1,
        Box::new(|data, _from, _to| {
            if is_all_brick(data.get("sherds")) {
                data.take("sherds");
                return;
            }
            if let Some(list) = data
                .get("sherds")
                .and_then(|v| pack_map(v, "decorated_pot sherds"))
            {
                data.set_list("sherds", list);
            }
        }),
    );
    // Schema V4996_1: decorated_pot walks `sherds` sides and `item` as ITEM_STACK.
    reg.tile_entity.add_walker(
        VERSION,
        1,
        "minecraft:decorated_pot",
        Arc::new(|reg, data, from, to| walk_sides(reg, data, "sherds", from, to)),
    );
    reg.tile_entity
        .add_walker(VERSION, 1, "minecraft:decorated_pot", items(&["item"]));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::{
        convert_block_entity, convert_block_entity_reverse, convert_item_stack,
        convert_item_stack_reverse,
    };

    fn ids(list: &[&str]) -> Vec<NbtValue> {
        list.iter()
            .map(|s| NbtValue::String(s.to_string()))
            .collect()
    }

    fn side_id<'a>(map: &'a NbtMap, side: &str) -> Option<&'a str> {
        map.get_map(side).and_then(|s| s.get_string("id"))
    }

    fn pot_item(decorations: Vec<NbtValue>) -> NbtMap {
        let mut components = NbtMap::new();
        components.set_list("minecraft:pot_decorations", decorations);
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:decorated_pot");
        item.set_i32("count", 1);
        item.set_map("components", components);
        item
    }

    #[test]
    fn component_list_unpacks_to_sides_and_back() {
        let mut item = pot_item(ids(&[
            "minecraft:angler_pottery_sherd",
            "minecraft:arms_up_pottery_sherd",
        ]));
        convert_item_stack(&mut item, VERSION - 1, VERSION);
        let deco = item
            .get_map("components")
            .unwrap()
            .get_map("minecraft:pot_decorations")
            .unwrap();
        assert_eq!(
            side_id(deco, "back"),
            Some("minecraft:angler_pottery_sherd")
        );
        assert_eq!(
            side_id(deco, "left"),
            Some("minecraft:arms_up_pottery_sherd")
        );
        assert_eq!(side_id(deco, "right"), Some(BRICK));
        assert_eq!(side_id(deco, "front"), Some(BRICK));

        let report = convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
        assert!(report.is_empty(), "{report:?}");
        let list = item
            .get_map("components")
            .unwrap()
            .get_list("minecraft:pot_decorations")
            .unwrap();
        let back: Vec<&str> = list.iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(
            back,
            vec![
                "minecraft:angler_pottery_sherd",
                "minecraft:arms_up_pottery_sherd",
                BRICK,
                BRICK
            ]
        );
    }

    #[test]
    fn component_with_empty_id_is_left_alone() {
        let mut item = pot_item(ids(&["minecraft:angler_pottery_sherd", ""]));
        convert_item_stack(&mut item, VERSION - 1, VERSION);
        assert!(item
            .get_map("components")
            .unwrap()
            .get_list("minecraft:pot_decorations")
            .is_some());
    }

    #[test]
    fn reverse_reports_side_with_components() {
        let mut stack = NbtMap::new();
        stack.set_string("id", "minecraft:angler_pottery_sherd");
        let mut c = NbtMap::new();
        c.set_i32("minecraft:max_stack_size", 1);
        stack.set_map("components", c);
        let mut deco = NbtMap::new();
        deco.set_map("back", stack);
        let mut components = NbtMap::new();
        components.set_map("minecraft:pot_decorations", deco);
        let mut item = NbtMap::new();
        item.set_string("id", "minecraft:decorated_pot");
        item.set_i32("count", 1);
        item.set_map("components", components);

        let report = convert_item_stack_reverse(&mut item, VERSION, VERSION - 1);
        assert_eq!(report.len(), 1);
        assert_eq!(report.entries[0].kind, LossKind::ComponentDropped);
    }

    #[test]
    fn block_entity_sherds_unpack_and_default_is_added_then_dropped() {
        let mut pot = NbtMap::new();
        pot.set_string("id", "minecraft:decorated_pot");
        pot.set_list("sherds", ids(&["minecraft:skull_pottery_sherd"]));
        convert_block_entity(&mut pot, VERSION - 1, VERSION);
        let sherds = pot.get_map("sherds").unwrap();
        assert_eq!(
            side_id(sherds, "back"),
            Some("minecraft:skull_pottery_sherd")
        );
        assert_eq!(side_id(sherds, "front"), Some(BRICK));

        let report = convert_block_entity_reverse(&mut pot, VERSION, VERSION - 1);
        assert!(report.is_empty());
        assert_eq!(pot.get_list("sherds").unwrap().len(), 4);

        // A pot with no sherds gains the all-brick map, and loses it again.
        let mut plain = NbtMap::new();
        plain.set_string("id", "minecraft:decorated_pot");
        convert_block_entity(&mut plain, VERSION - 1, VERSION);
        assert!(is_all_brick(plain.get("sherds")));
        let report = convert_block_entity_reverse(&mut plain, VERSION, VERSION - 1);
        assert!(report.is_empty());
        assert!(!plain.has_key("sherds"));
    }

    #[test]
    fn block_entity_sherd_sides_are_walked_as_item_stacks() {
        // A side id without a namespace is corrected by the ITEM_STACK
        // namespace hook, which only fires if the side is walked as an item.
        let mut pot = NbtMap::new();
        pot.set_string("id", "minecraft:decorated_pot");
        pot.set_list("sherds", ids(&["skull_pottery_sherd"]));
        convert_block_entity(&mut pot, VERSION - 1, VERSION);
        assert_eq!(
            side_id(pot.get_map("sherds").unwrap(), "back"),
            Some("minecraft:skull_pottery_sherd")
        );
    }
}
