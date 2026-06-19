//! V4059 (24w33a + 1) — schematic-relevant subset of `V4059.java`.
//!
//! The 1.20.5-component DATA_COMPONENTS schema. Two registrations are ported:
//!
//!   * DATA_COMPONENTS structure walker: descends every typed sub-structure of a
//!     component map — bee entities, block-entity data, bundle/container/charged
//!     items, block-predicate `blocks` (string or list) inside
//!     `can_break` / `can_place_on`, pot decorations (ITEM_NAME), entity data,
//!     `use_remainder`, equippable `allowed_entities` (ENTITY_NAME, string or
//!     list), the text components (`custom_name` / `item_name` / `lore`), and
//!     written-book pages.
//!   * DATA_COMPONENTS structure converter: migrate the old `minecraft:food`
//!     component (`eat_seconds` / `effects` / `using_converts_to`) into the new
//!     `minecraft:consumable` component (+ `minecraft:use_remainder`).
//!
//! Port note (written_book_content pages): TEXT_COMPONENT here is a compound-only
//! type, so the page-conversion handles the Map encodings exactly as Java does
//! (filterable `raw`/`filtered`, and a bare component compound), while a String
//! or List page — which Java would round-trip through TEXT_COMPONENT's value
//! form — is left untouched (no value-level text-component conversion exists in
//! this engine; it is a no-op for those shapes).
//!
//! Nothing non-schematic is present in this version.
//!
//! VERSION = MCVersions.V24W33A (4058) + 1 = 4059.
//! `V4290.VERSION` (the NBT-page gate) = MCVersions.V1_21_4 (4189) + 101 = 4290.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::{Registry, RegistryBuilder};
use super::super::types::{MapExt, ValueExt};
use super::super::version::{encode_versions, EncodedVersion};
use super::super::walker::{
    convert, convert_list, convert_list_path, convert_value, convert_value_list,
};

const VERSION: i32 = 4059;
/// Pages stored as NBT (not JSON) starting at V4290.
const V4290_VERSION: i32 = 4290;

/// `walkBlockPredicates`: a block predicate's `blocks` is either a single
/// block-id string or a list of them.
fn walk_block_predicates(
    root: &mut NbtMap,
    from: EncodedVersion,
    to: EncodedVersion,
    reg: &Registry,
) {
    match root.get("blocks") {
        Some(NbtValue::String(_)) => convert_value(&reg.block_name, root, "blocks", from, to),
        Some(NbtValue::List(_)) => convert_value_list(&reg.block_name, root, "blocks", from, to),
        _ => {}
    }
}

/// Walk every block predicate of a `can_break` / `can_place_on` component: each
/// entry of the `predicates` list, then the component map itself (the simple
/// encoding stores `blocks` directly on the root).
fn walk_can_predicates(
    component: &mut NbtMap,
    from: EncodedVersion,
    to: EncodedVersion,
    reg: &Registry,
) {
    if let Some(predicates) = component.get_list_mut("predicates") {
        // Collect indices then walk to avoid holding the list borrow across the
        // recursive helper (which needs `reg` only).
        for el in predicates.iter_mut() {
            if let Some(p) = el.as_compound_mut() {
                walk_block_predicates(p, from, to, reg);
            }
        }
    }
    walk_block_predicates(component, from, to, reg);
}

fn walk(reg: &Registry, root: &mut NbtMap, from: EncodedVersion, to: EncodedVersion) {
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

    if let Some(can_break) = root.get_map_mut("minecraft:can_break") {
        walk_can_predicates(can_break, from, to, reg);
    }
    if let Some(can_place_on) = root.get_map_mut("minecraft:can_place_on") {
        walk_can_predicates(can_place_on, from, to, reg);
    }

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
    convert(
        reg,
        &reg.item_stack,
        root,
        "minecraft:use_remainder",
        from,
        to,
    );

    if let Some(equippable) = root.get_map_mut("minecraft:equippable") {
        // allowed_entities is ENTITY_NAME, as either a single value or a list.
        convert_value(&reg.entity_name, equippable, "allowed_entities", from, to);
        convert_value_list(&reg.entity_name, equippable, "allowed_entities", from, to);
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

    if let Some(written_book_content) = root.get_map_mut("minecraft:written_book_content") {
        let is_nbt_format = from >= encode_versions(V4290_VERSION, 0);
        if let Some(pages) = written_book_content.get_list_mut("pages") {
            for page in pages.iter_mut() {
                if let Some(map_type) = page.as_compound_mut() {
                    if map_type.has_key("raw") || map_type.has_key("filtered") {
                        // Filterable format.
                        convert(reg, &reg.text_component, map_type, "raw", from, to);
                        convert(reg, &reg.text_component, map_type, "filtered", from, to);
                    } else if is_nbt_format {
                        // Bare component compound (NBT only).
                        reg.text_component.convert(reg, map_type, from, to);
                    }
                }
                // String / List page shapes: no value-level TEXT_COMPONENT
                // conversion exists in this engine — left untouched.
            }
        }
    }
}

/// `getFloat(key, default)` over the lenient numeric read.
fn get_f32(map: &NbtMap, key: &str, default: f32) -> f32 {
    map.get_f64(key).map(|v| v as f32).unwrap_or(default)
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.data_components
        .add_structure_walker(VERSION, 0, Arc::new(walk));

    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if data.get_map("minecraft:food").is_none() {
                return;
            }

            // Read everything we need off `food` before mutating `data`.
            let eat_seconds = data
                .get_map("minecraft:food")
                .map(|f| get_f32(f, "eat_seconds", 1.6))
                .unwrap_or(1.6);

            let old_effects: Vec<NbtValue> = data
                .get_map("minecraft:food")
                .and_then(|f| f.get_list("effects"))
                .map(|l| l.to_vec())
                .unwrap_or_default();

            let converts_to: Option<NbtValue> = data
                .get_map("minecraft:food")
                .and_then(|f| f.get("using_converts_to").cloned());

            // Build the new on_consume_effects list.
            let mut new_effects: Vec<NbtValue> = Vec::with_capacity(old_effects.len());
            for old in &old_effects {
                let old_effect = match old.as_compound_ref() {
                    Some(m) => m,
                    None => continue,
                };

                let mut new_effect = NbtMap::new();
                new_effect.set_string("type", "minecraft:apply_effects");

                let mut effects_inner: Vec<NbtValue> = Vec::new();
                if let Some(effect) = old_effect.get("effect").cloned() {
                    effects_inner.push(effect);
                }
                new_effect.set_list("effects", effects_inner);

                new_effect.set_f32("probability", get_f32(old_effect, "probability", 1.0));

                new_effects.push(NbtValue::Compound(new_effect));
            }

            // Strip the migrated fields off `food`.
            if let Some(food) = data.get_map_mut("minecraft:food") {
                food.take("eat_seconds");
                food.take("effects");
                food.take("using_converts_to");
            }

            if let Some(converts_to) = converts_to {
                data.set_generic("minecraft:use_remainder", converts_to);
            }

            let mut consumable = NbtMap::new();
            consumable.set_f32("consume_seconds", eat_seconds);
            consumable.set_list("on_consume_effects", new_effects);
            data.set_map("minecraft:consumable", consumable);
        }),
    );

    // REVERSE DATA_COMPONENTS: `minecraft:consumable` (+ moved
    // `minecraft:use_remainder`) -> the legacy `minecraft:food` fields
    // (`eat_seconds` / `effects` / `using_converts_to`). Inverse of the forward
    // food->consumable migration above.
    //
    // The forward (V4059.java:111-160) only ran when `minecraft:food` was present,
    // and it always created `minecraft:consumable` (consume_seconds +
    // on_consume_effects), folding food.effects into wrapper effects of the form
    // `{type:"minecraft:apply_effects", effects:[<old effect>], probability}` and
    // moving food.using_converts_to up to the top-level `minecraft:use_remainder`.
    // So the natural inverse is gated on `minecraft:consumable`.
    //
    // Lossless parts: consume_seconds -> eat_seconds; an apply_effects wrapper with
    // exactly one inner effect -> `{effect, probability}` (the old default 1.0F that
    // the legacy format always carried is restored exactly); top-level
    // use_remainder -> food.using_converts_to (pre-4059 there was no standalone
    // `use_remainder` component, so it belonged inside food).
    //
    // Lossy parts (rule 11 — only when MODERN data can't express the old shape):
    // a `minecraft:consumable` carries fields with no pre-4059 analogue
    // (`animation`, `sound`, `has_consume_particles`, ...) that are dropped, and an
    // on_consume_effect that is not a single-effect `apply_effects` (a different
    // consume-effect type, or an apply_effects wrapping multiple/zero effects)
    // cannot be encoded by the legacy `food.effects` `{effect, probability}` shape.
    reg.data_components.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Nothing the forward produced unless `minecraft:consumable` exists.
            let mut consumable = match data.take("minecraft:consumable") {
                Some(NbtValue::Compound(m)) => m,
                // Non-compound / absent: restore untouched (mirrors getMap()==null).
                Some(other) => {
                    data.set_generic("minecraft:consumable", other);
                    return;
                }
                None => return,
            };

            // consume_seconds -> food.eat_seconds (forward read the 1.6F default).
            let eat_seconds = get_f32(&consumable, "consume_seconds", 1.6);

            // on_consume_effects -> food.effects, unwrapping the apply_effects shape.
            let on_consume_effects: Vec<NbtValue> = consumable
                .get_list("on_consume_effects")
                .map(|l| l.to_vec())
                .unwrap_or_default();

            let mut old_effects: Vec<NbtValue> = Vec::with_capacity(on_consume_effects.len());
            for new in &on_consume_effects {
                let new_effect = match new.as_compound_ref() {
                    Some(m) => m,
                    None => {
                        report_loss(
                            VERSION,
                            LossKind::ComponentDropped,
                            Severity::Loss,
                            "non-compound minecraft:consumable on_consume_effect cannot be encoded in legacy minecraft:food.effects",
                        );
                        continue;
                    }
                };

                // The legacy `food.effects` entry only stored a single mob-effect
                // instance plus a probability, encoded by the forward as a
                // single-element `minecraft:apply_effects` wrapper. Anything else
                // has no legacy representation.
                if new_effect.get_string("type") != Some("minecraft:apply_effects") {
                    report_loss(
                        VERSION,
                        LossKind::ComponentDropped,
                        Severity::Loss,
                        "minecraft:consumable on_consume_effect is not minecraft:apply_effects; no legacy minecraft:food.effects encoding",
                    );
                    continue;
                }

                let inner: Vec<NbtValue> = new_effect
                    .get_list("effects")
                    .map(|l| l.to_vec())
                    .unwrap_or_default();
                if inner.len() != 1 {
                    report_loss(
                        VERSION,
                        LossKind::ComponentDropped,
                        Severity::Loss,
                        "minecraft:apply_effects wraps multiple/zero effects; legacy minecraft:food.effects holds exactly one",
                    );
                }
                let effect = match inner.into_iter().next() {
                    Some(e) => e,
                    None => continue,
                };

                let mut old_effect = NbtMap::new();
                old_effect.set_generic("effect", effect);
                old_effect.set_f32("probability", get_f32(new_effect, "probability", 1.0));
                old_effects.push(NbtValue::Compound(old_effect));
            }

            // Any consumable field beyond consume_seconds/on_consume_effects has no
            // pre-4059 equivalent.
            consumable.take("consume_seconds");
            consumable.take("on_consume_effects");
            if !consumable.is_empty() {
                report_loss(
                    VERSION,
                    LossKind::ComponentDropped,
                    Severity::Loss,
                    "minecraft:consumable has fields with no legacy minecraft:food equivalent (e.g. animation/sound/has_consume_particles)",
                );
            }

            // The pre-4059 format had no standalone `use_remainder`; the forward
            // hoisted food.using_converts_to to it, so move it back into food.
            let converts_to = data.take("minecraft:use_remainder");

            // Restore the legacy food fields (the forward kept a stripped `food`;
            // recreate it if absent since old consumables were always `food`).
            if data.get_map("minecraft:food").is_none() {
                data.set_map("minecraft:food", NbtMap::new());
            }
            if let Some(food) = data.get_map_mut("minecraft:food") {
                food.set_f32("eat_seconds", eat_seconds);
                // Forward only removed `effects`; restore only when non-empty so an
                // item that never had effects round-trips to no `effects` key.
                if !old_effects.is_empty() {
                    food.set_list("effects", old_effects);
                }
                if let Some(converts_to) = converts_to {
                    food.set_generic("using_converts_to", converts_to);
                }
            }
        }),
    );
}
