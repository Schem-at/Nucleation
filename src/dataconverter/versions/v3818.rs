//! V3818 (V24W07A + 1 = 3818) — the schematic-relevant subset of `V3818.java`,
//! the 1.20.5 "components" rework.
//!
//! Ported steps (Java step in parens):
//!   * (0) TILE_ENTITY `beehive` Bees->bees field renames + its ENTITY walker.
//!   * (1) TILE_ENTITY `banner` pattern-code -> id + colour-name conversion.
//!   * (2) ENTITY `arrow` Potion/effects/Color fold into the nested `item` tag.
//!   * (3) DATA_COMPONENTS structure walker (recurse every nested typed field of
//!         the new components map).
//!   * (4) PARTICLE structure walker (map-form item/block_state recursion).
//!   * (5) ITEM_STACK `tag` -> `components` squash
//!         ([`super::super::components::convert_item`]) + ITEM_STACK walker.
//!   * (6) ENTITY `area_effect_cloud` Color/effects/Potion -> potion_contents.
//!
//! Skipped (non-schematic): the V0 HOTBAR air-slot cleanup. The PARTICLE *string
//! -> NBT* structure converter (V3818 step 4) is intentionally not registered:
//! our PARTICLE type is compound-based (a flat-string particle never reaches it
//! through a walker), and particles do not appear in the schematic data our
//! importers surface.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::components::{banner_colour, convert_item, unconvert_item};
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::{Registry, RegistryBuilder};
use super::super::types::{MapExt, ValueExt};
use super::super::version::EncodedVersion;
use super::super::walker::{
    convert, convert_list, convert_list_path, convert_value, convert_value_list,
};

const VERSION: i32 = 3818;

/// `V3818.PATTERN_UPDATE`: legacy short banner-pattern code -> modern id.
fn pattern_update(code: &str) -> Option<&'static str> {
    Some(match code {
        "b" => "minecraft:base",
        "bl" => "minecraft:square_bottom_left",
        "br" => "minecraft:square_bottom_right",
        "tl" => "minecraft:square_top_left",
        "tr" => "minecraft:square_top_right",
        "bs" => "minecraft:stripe_bottom",
        "ts" => "minecraft:stripe_top",
        "ls" => "minecraft:stripe_left",
        "rs" => "minecraft:stripe_right",
        "cs" => "minecraft:stripe_center",
        "ms" => "minecraft:stripe_middle",
        "drs" => "minecraft:stripe_downright",
        "dls" => "minecraft:stripe_downleft",
        "ss" => "minecraft:small_stripes",
        "cr" => "minecraft:cross",
        "sc" => "minecraft:straight_cross",
        "bt" => "minecraft:triangle_bottom",
        "tt" => "minecraft:triangle_top",
        "bts" => "minecraft:triangles_bottom",
        "tts" => "minecraft:triangles_top",
        "ld" => "minecraft:diagonal_left",
        "rd" => "minecraft:diagonal_up_right",
        "lud" => "minecraft:diagonal_up_left",
        "rud" => "minecraft:diagonal_right",
        "mc" => "minecraft:circle",
        "mr" => "minecraft:rhombus",
        "vh" => "minecraft:half_vertical",
        "hh" => "minecraft:half_horizontal",
        "vhr" => "minecraft:half_vertical_right",
        "hhb" => "minecraft:half_horizontal_bottom",
        "bo" => "minecraft:border",
        "cbo" => "minecraft:curly_border",
        "gra" => "minecraft:gradient",
        "gru" => "minecraft:gradient_up",
        "bri" => "minecraft:bricks",
        "glb" => "minecraft:globe",
        "cre" => "minecraft:creeper",
        "sku" => "minecraft:skull",
        "flo" => "minecraft:flower",
        "moj" => "minecraft:mojang",
        "pig" => "minecraft:piglin",
        _ => return None,
    })
}

/// Inverse of [`pattern_update`]: modern banner-pattern id -> legacy short code.
/// `None` for a pattern with no pre-1.20.5 short code (kept as-is + reported).
fn pattern_code(id: &str) -> Option<&'static str> {
    Some(match id {
        "minecraft:base" => "b",
        "minecraft:square_bottom_left" => "bl",
        "minecraft:square_bottom_right" => "br",
        "minecraft:square_top_left" => "tl",
        "minecraft:square_top_right" => "tr",
        "minecraft:stripe_bottom" => "bs",
        "minecraft:stripe_top" => "ts",
        "minecraft:stripe_left" => "ls",
        "minecraft:stripe_right" => "rs",
        "minecraft:stripe_center" => "cs",
        "minecraft:stripe_middle" => "ms",
        "minecraft:stripe_downright" => "drs",
        "minecraft:stripe_downleft" => "dls",
        "minecraft:small_stripes" => "ss",
        "minecraft:cross" => "cr",
        "minecraft:straight_cross" => "sc",
        "minecraft:triangle_bottom" => "bt",
        "minecraft:triangle_top" => "tt",
        "minecraft:triangles_bottom" => "bts",
        "minecraft:triangles_top" => "tts",
        "minecraft:diagonal_left" => "ld",
        "minecraft:diagonal_up_right" => "rd",
        "minecraft:diagonal_up_left" => "lud",
        "minecraft:diagonal_right" => "rud",
        "minecraft:circle" => "mc",
        "minecraft:rhombus" => "mr",
        "minecraft:half_vertical" => "vh",
        "minecraft:half_horizontal" => "hh",
        "minecraft:half_vertical_right" => "vhr",
        "minecraft:half_horizontal_bottom" => "hhb",
        "minecraft:border" => "bo",
        "minecraft:curly_border" => "cbo",
        "minecraft:gradient" => "gra",
        "minecraft:gradient_up" => "gru",
        "minecraft:bricks" => "bri",
        "minecraft:globe" => "glb",
        "minecraft:creeper" => "cre",
        "minecraft:skull" => "sku",
        "minecraft:flower" => "flo",
        "minecraft:mojang" => "moj",
        "minecraft:piglin" => "pig",
        _ => return None,
    })
}

/// Inverse of [`banner_colour`]: colour name -> legacy 0..15 id.
fn banner_colour_id(name: &str) -> Option<i32> {
    (0..16).find(|&i| banner_colour(i) == name)
}

/// `walkBlockPredicates`: a predicate compound carries `blocks` as a string or a
/// list of block ids.
fn walk_block_predicates(
    reg: &Registry,
    root: &mut NbtMap,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    match root.get("blocks") {
        Some(NbtValue::String(_)) => convert_value(&reg.block_name, root, "blocks", from, to),
        Some(NbtValue::List(_)) => convert_value_list(&reg.block_name, root, "blocks", from, to),
        _ => {}
    }
}

fn walk_block_predicates_component(
    reg: &Registry,
    root: &mut NbtMap,
    key: &str,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    let Some(comp) = root.get_map_mut(key) else {
        return;
    };
    if let Some(preds) = comp.get_list_mut("predicates") {
        for p in preds.iter_mut() {
            if let Some(pm) = p.as_compound_mut() {
                walk_block_predicates(reg, pm, from, to);
            }
        }
    }
    // Not handled by DFU: the simple encoding does not require "predicates".
    walk_block_predicates(reg, comp, from, to);
}

/// The DATA_COMPONENTS structure walker (V3818 step 3): recurse every nested
/// typed field of the new components map.
fn data_components_walk(
    reg: &Registry,
    root: &mut NbtMap,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    convert_list_path(
        reg,
        &reg.entity,
        root,
        "minecraft:bees",
        "entity_data",
        from,
        to,
    );
    convert(
        reg,
        &reg.tile_entity,
        root,
        "minecraft:block_entity_data",
        from,
        to,
    );
    convert_list(
        reg,
        &reg.item_stack,
        root,
        "minecraft:bundle_contents",
        from,
        to,
    );

    walk_block_predicates_component(reg, root, "minecraft:can_break", from, to);
    walk_block_predicates_component(reg, root, "minecraft:can_place_on", from, to);

    convert_list(
        reg,
        &reg.item_stack,
        root,
        "minecraft:charged_projectiles",
        from,
        to,
    );
    convert_list_path(
        reg,
        &reg.item_stack,
        root,
        "minecraft:container",
        "item",
        from,
        to,
    );
    convert(reg, &reg.entity, root, "minecraft:entity_data", from, to);
    convert_value_list(&reg.item_name, root, "minecraft:pot_decorations", from, to);

    if let Some(food) = root.get_map_mut("minecraft:food") {
        convert(reg, &reg.item_stack, food, "using_converts_to", from, to);
    }

    convert(
        reg,
        &reg.text_component,
        root,
        "minecraft:custom_name",
        from,
        to,
    );
    convert(
        reg,
        &reg.text_component,
        root,
        "minecraft:item_name",
        from,
        to,
    );
    convert_list(reg, &reg.text_component, root, "minecraft:lore", from, to);

    if let Some(wbc) = root.get_map_mut("minecraft:written_book_content") {
        if let Some(pages) = wbc.get_list_mut("pages") {
            for page in pages.iter_mut() {
                // A string page would convert through TEXT_COMPONENT as a raw JSON
                // string; our compound-based TEXT_COMPONENT leaves it unchanged.
                if let Some(pm) = page.as_compound_mut() {
                    convert(reg, &reg.text_component, pm, "raw", from, to);
                    convert(reg, &reg.text_component, pm, "filtered", from, to);
                }
            }
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // --- step 0: beehive --------------------------------------------------
    reg.tile_entity.add_converter_for_id(
        "minecraft:beehive",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            data.rename_key("Bees", "bees");
            if let Some(bees) = data.get_list_mut("bees") {
                for bee in bees.iter_mut() {
                    if let Some(bm) = bee.as_compound_mut() {
                        bm.rename_key("EntityData", "entity_data");
                        bm.rename_key("TicksInHive", "ticks_in_hive");
                        bm.rename_key("MinOccupationTicks", "min_ticks_in_hive");
                    }
                }
            }
        }),
    );
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:beehive",
        Arc::new(|reg, data, from, to| {
            convert_list_path(reg, &reg.entity, data, "bees", "entity_data", from, to);
        }),
    );

    // --- step 1: banner ---------------------------------------------------
    reg.tile_entity.add_converter_for_id(
        "minecraft:banner",
        VERSION,
        1,
        Box::new(|data, _from, _to| {
            if let Some(patterns) = data.get_list_mut("Patterns") {
                for p in patterns.iter_mut() {
                    if let Some(pm) = p.as_compound_mut() {
                        if let Some(code) = pm.get_string("Pattern").map(str::to_string) {
                            if let Some(renamed) = pattern_update(&code) {
                                pm.set_string("Pattern", renamed);
                            }
                        }
                        pm.rename_key("Pattern", "pattern");

                        let new_colour = banner_colour(pm.get_i32("Color").unwrap_or(0));
                        pm.set_string("Color", new_colour);
                        pm.rename_key("Color", "color");
                    }
                }
            }
            data.rename_key("Patterns", "patterns");
        }),
    );

    // --- step 2: arrow ----------------------------------------------------
    reg.entity.add_converter_for_id(
        "minecraft:arrow",
        VERSION,
        2,
        Box::new(|data, _from, _to| {
            if !data.has_key("Potion")
                && !data.has_key("custom_potion_effects")
                && !data.has_key("Color")
            {
                return;
            }
            let potion = data.take("Potion");
            let custom = data.take("custom_potion_effects");
            let color = data.take("Color");

            let Some(item) = data.get_map_mut("item") else {
                return;
            };
            if item.get_map("tag").is_none() {
                item.set_map("tag", NbtMap::new());
            }
            let tag = item.get_map_mut("tag").expect("just inserted");
            if let Some(p) = potion {
                tag.set_generic("Potion", p);
            }
            if let Some(c) = custom {
                tag.set_generic("custom_potion_effects", c);
            }
            if let Some(c) = color {
                tag.set_generic("CustomPotionColor", c);
            }
        }),
    );

    // --- step 3: DATA_COMPONENTS walker -----------------------------------
    reg.data_components
        .add_structure_walker(VERSION, 3, Arc::new(data_components_walk));

    // --- step 4: PARTICLE walker (map form) -------------------------------
    reg.particle.add_structure_walker(
        VERSION,
        4,
        Arc::new(|reg, root, from, to| {
            convert(reg, &reg.item_stack, root, "item", from, to);
            convert(reg, &reg.block_state, root, "block_state", from, to);
        }),
    );

    // --- step 5: ITEM_STACK components squash -----------------------------
    reg.item_stack.add_structure_converter(
        VERSION,
        5,
        Box::new(|data, _from, _to| {
            let out = convert_item(data);
            *data = out;
        }),
    );
    reg.item_stack.add_structure_walker(
        VERSION,
        5,
        Arc::new(|reg, root, from, to| {
            convert_value(&reg.item_name, root, "id", from, to);
            convert(reg, &reg.data_components, root, "components", from, to);
        }),
    );

    // --- step 6: area_effect_cloud ----------------------------------------
    reg.entity.add_converter_for_id(
        "minecraft:area_effect_cloud",
        VERSION,
        6,
        Box::new(|data, _from, _to| {
            if !data.has_key("Color") && !data.has_key("effects") && !data.has_key("Potion") {
                return;
            }
            let color = data.take("Color");
            let effects = data.take("effects");
            let potion = data.take("Potion");

            let mut potion_contents = NbtMap::new();
            if let Some(c) = color {
                potion_contents.set_generic("custom_color", c);
            }
            if let Some(e) = effects {
                potion_contents.set_generic("custom_effects", e);
            }
            if let Some(p) = potion {
                potion_contents.set_generic("potion", p);
            }
            data.set_map("potion_contents", potion_contents);
        }),
    );

    // ======================================================================
    // REVERSE (new -> old): inverses, registered at the same (VERSION, step).
    // Walkers are direction-neutral. The walker descends FIRST in reverse, so by
    // the time step-5's reverse runs, nested items inside `components` (container,
    // bundle, charged_projectiles) have already been un-squashed to legacy form.
    // ======================================================================

    // step 0 reverse: beehive field renames back to legacy.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:beehive",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            if let Some(bees) = data.get_list_mut("bees") {
                for bee in bees.iter_mut() {
                    if let Some(bm) = bee.as_compound_mut() {
                        bm.rename_key("entity_data", "EntityData");
                        bm.rename_key("ticks_in_hive", "TicksInHive");
                        bm.rename_key("min_ticks_in_hive", "MinOccupationTicks");
                    }
                }
            }
            data.rename_key("bees", "Bees");
        }),
    );

    // step 1 reverse: banner pattern id -> short code, colour name -> id, rename
    // patterns/pattern/color back to the legacy capitalised keys.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:banner",
        VERSION,
        1,
        Box::new(|data, _from, _to| {
            if let Some(patterns) = data.get_list_mut("patterns") {
                for p in patterns.iter_mut() {
                    if let Some(pm) = p.as_compound_mut() {
                        // pattern id -> short code
                        if let Some(id) = pm.get_string("pattern").map(str::to_string) {
                            match pattern_code(&id) {
                                Some(code) => pm.set_string("pattern", code),
                                None => report_loss(
                                    VERSION,
                                    LossKind::Other,
                                    Severity::Loss,
                                    format!("banner pattern {id} has no pre-1.20.5 short code"),
                                ),
                            }
                        }
                        pm.rename_key("pattern", "Pattern");
                        // colour name -> id
                        if let Some(name) = pm.get_string("color").map(str::to_string) {
                            match banner_colour_id(&name) {
                                Some(id) => pm.set_i32("color", id),
                                None => {
                                    pm.set_i32("color", 0);
                                    report_loss(
                                        VERSION,
                                        LossKind::Other,
                                        Severity::Approximated,
                                        format!("banner color {name} has no legacy id; using 0"),
                                    );
                                }
                            }
                        }
                        pm.rename_key("color", "Color");
                    }
                }
            }
            data.rename_key("patterns", "Patterns");
        }),
    );

    // step 2 reverse: arrow item.tag.{Potion,custom_potion_effects,
    // CustomPotionColor} -> top-level {Potion,custom_potion_effects,Color}.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:arrow",
        VERSION,
        2,
        Box::new(|data, _from, _to| {
            let Some(item) = data.get_map_mut("item") else {
                return;
            };
            let Some(tag) = item.get_map_mut("tag") else {
                return;
            };
            let potion = tag.take("Potion");
            let custom = tag.take("custom_potion_effects");
            let color = tag.take("CustomPotionColor");
            let tag_empty = tag.is_empty();
            if tag_empty {
                item.take("tag");
            }
            if let Some(p) = potion {
                data.set_generic("Potion", p);
            }
            if let Some(c) = custom {
                data.set_generic("custom_potion_effects", c);
            }
            if let Some(c) = color {
                data.set_generic("Color", c);
            }
        }),
    );

    // step 5 reverse: ITEM_STACK components -> legacy tag (the inverse of the
    // 1.20.5 squash; the user's core pain point). `unconvert_item` reports any
    // component that has no legacy representation.
    reg.item_stack.add_reverse_converter(
        VERSION,
        5,
        Box::new(|data, _from, _to| {
            let out = unconvert_item(data);
            *data = out;
        }),
    );

    // step 6 reverse: area_effect_cloud potion_contents -> Color/effects/Potion.
    reg.entity.add_reverse_converter_for_id(
        "minecraft:area_effect_cloud",
        VERSION,
        6,
        Box::new(|data, _from, _to| {
            let Some(mut potion_contents) = data.take("potion_contents") else {
                return;
            };
            let Some(pc) = potion_contents.as_compound_mut() else {
                data.set_generic("potion_contents", potion_contents);
                return;
            };
            if let Some(c) = pc.take("custom_color") {
                data.set_generic("Color", c);
            }
            if let Some(e) = pc.take("custom_effects") {
                data.set_generic("effects", e);
            }
            if let Some(p) = pc.take("potion") {
                data.set_generic("Potion", p);
            }
            if !pc.is_empty() {
                report_loss(
                    VERSION,
                    LossKind::ComponentDropped,
                    Severity::Loss,
                    "area_effect_cloud potion_contents has extra fields with no legacy representation; dropped",
                );
            }
        }),
    );
}
