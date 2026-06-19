//! V4307 (25w03a + 3) — schematic-relevant subset of `V4307.java`.
//!
//! DATA_COMPONENTS structure converter that migrates the per-component
//! `show_in_tooltip` flags and the `minecraft:hide_tooltip` /
//! `minecraft:hide_additional_tooltip` markers into a single
//! `minecraft:tooltip_display` component (V4307.java:484-599):
//!   * `minecraft:can_break` / `minecraft:can_place_on`: unwrap `predicates`
//!     (the map collapses to its `predicates` value) and, if `show_in_tooltip`
//!     was false, mark the component hidden.
//!   * `minecraft:trim` / `minecraft:unbreakable`: drop `show_in_tooltip`, mark
//!     hidden if it was false.
//!   * `minecraft:dyed_color`/`attribute_modifiers`/`enchantments`/
//!     `stored_enchantments`/`jukebox_playable`: same tooltip handling, then
//!     unwrap the inner value (`rgb`/`modifiers`/`levels`/`song`).
//!   * `minecraft:hide_tooltip` / `minecraft:hide_additional_tooltip` flags are
//!     consumed; the latter hides every present `ADDITIONAL_TOOLTIP_COMPONENTS`
//!     entry. When anything is hidden a `minecraft:tooltip_display` compound is
//!     written.
//!
//! Plus the DATA_COMPONENTS structure walker (V4307.java:602-684) recursing all
//! the nested typed sub-structures.
//!
//! VERSION = V25W03A(4304) + 3. Entirely schematic-relevant (DATA_COMPONENTS).
//!
//! Note on `written_book_content` pages (walker): the engine's TEXT_COMPONENT is
//! a compound type whose `convert` only operates on map nodes, so String/List
//! page entries (legacy stringified components) cannot be recursed through it —
//! this matches the existing engine limitation (e.g. the V704 ITEM_STACK walker
//! also only descends compound pages). Compound pages, including the filterable
//! `raw`/`filtered` form, are handled exactly as Java does.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::{Registry, RegistryBuilder};
use super::super::types::{MapExt, ValueExt};
use super::super::version::EncodedVersion;
use super::super::walker::{
    convert, convert_list, convert_list_path, convert_value, convert_value_list,
};

const VERSION: i32 = 4307;

/// Components hidden wholesale by `minecraft:hide_additional_tooltip`
/// (V4307.java:485-503).
const ADDITIONAL_TOOLTIP_COMPONENTS: &[&str] = &[
    "minecraft:banner_patterns",
    "minecraft:bees",
    "minecraft:block_entity_data",
    "minecraft:block_state",
    "minecraft:bundle_contents",
    "minecraft:charged_projectiles",
    "minecraft:container",
    "minecraft:container_loot",
    "minecraft:firework_explosion",
    "minecraft:fireworks",
    "minecraft:instrument",
    "minecraft:map_id",
    "minecraft:painting/variant",
    "minecraft:pot_decorations",
    "minecraft:potion_contents",
    "minecraft:tropical_fish/pattern",
    "minecraft:written_book_content",
];

/// Insertion-ordered set push (LinkedHashSet semantics from Java).
fn push_hidden(hidden: &mut Vec<String>, path: &str) {
    if !hidden.iter().any(|p| p == path) {
        hidden.push(path.to_string());
    }
}

/// `unwrapBlockPredicates` (V4307.java:505-519).
fn unwrap_block_predicates(root: &mut NbtMap, path: &str, hidden: &mut Vec<String>) {
    let component = match root.get_map(path) {
        Some(c) => c,
        None => return,
    };
    let predicates = match component.get("predicates") {
        Some(p) => p.clone(),
        None => return,
    };
    let show = component.get_bool("show_in_tooltip").unwrap_or(true);
    root.set_generic(path, predicates);
    if !show {
        push_hidden(hidden, path);
    }
}

/// `updateComponent` (V4307.java:521-532).
fn update_component(root: &mut NbtMap, path: &str, hidden: &mut Vec<String>) {
    let component = match root.get_map_mut(path) {
        Some(c) => c,
        None => return,
    };
    let show = component.get_bool("show_in_tooltip").unwrap_or(true);
    if !show {
        push_hidden(hidden, path);
    }
    component.take("show_in_tooltip");
}

/// `updateComponentAndUnwrap` (V4307.java:534-550).
fn update_component_and_unwrap(
    root: &mut NbtMap,
    component_path: &str,
    unwrap_path: &str,
    hidden: &mut Vec<String>,
) {
    let wrapped = {
        let component = match root.get_map_mut(component_path) {
            Some(c) => c,
            None => return,
        };
        let show = component.get_bool("show_in_tooltip").unwrap_or(true);
        if !show {
            push_hidden(hidden, component_path);
        }
        component.take("show_in_tooltip");
        component.get(unwrap_path).cloned()
    };
    if let Some(wrapped) = wrapped {
        root.set_generic(component_path, wrapped);
    }
}

// ---- Reverse helpers (inverse of the forward unwrap/hide migration) ----

/// The five `updateComponentAndUnwrap` components and the inner key the forward
/// hoisted to the component root (V4307.java:540-547). The reverse re-nests the
/// current value under that key.
const UNWRAP_COMPONENTS: &[(&str, &str)] = &[
    ("minecraft:dyed_color", "rgb"),
    ("minecraft:attribute_modifiers", "modifiers"),
    ("minecraft:enchantments", "levels"),
    ("minecraft:stored_enchantments", "levels"),
    ("minecraft:jukebox_playable", "song"),
];

/// The two `unwrapBlockPredicates` components: the forward replaced the component
/// map with its `predicates` value (V4307.java:512). The reverse re-nests the
/// current value under `predicates`.
const PREDICATE_COMPONENTS: &[&str] = &["minecraft:can_break", "minecraft:can_place_on"];

/// The two `updateComponent` components, which only had `show_in_tooltip`
/// stripped (no unwrap) by the forward (V4307.java:521-532).
const PLAIN_TOOLTIP_COMPONENTS: &[&str] = &["minecraft:trim", "minecraft:unbreakable"];

/// Re-wrap a component the forward unwrapped: replace `root[path]` (the hoisted
/// inner value) with `{inner_key: value}`. No-op if the component is absent.
fn rewrap_under(root: &mut NbtMap, path: &str, inner_key: &str) {
    if let Some(value) = root.take(path) {
        let mut component = NbtMap::new();
        component.set_generic(inner_key, value);
        root.set_map(path, component);
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.data_components.add_structure_converter(
        VERSION,
        0,
        Box::new(|root, _from, _to| {
            let mut hidden: Vec<String> = Vec::new();

            unwrap_block_predicates(root, "minecraft:can_break", &mut hidden);
            unwrap_block_predicates(root, "minecraft:can_place_on", &mut hidden);

            update_component(root, "minecraft:trim", &mut hidden);
            update_component(root, "minecraft:unbreakable", &mut hidden);

            update_component_and_unwrap(root, "minecraft:dyed_color", "rgb", &mut hidden);
            update_component_and_unwrap(
                root,
                "minecraft:attribute_modifiers",
                "modifiers",
                &mut hidden,
            );
            update_component_and_unwrap(root, "minecraft:enchantments", "levels", &mut hidden);
            update_component_and_unwrap(
                root,
                "minecraft:stored_enchantments",
                "levels",
                &mut hidden,
            );
            update_component_and_unwrap(root, "minecraft:jukebox_playable", "song", &mut hidden);

            let hide_tooltip = root.has_key("minecraft:hide_tooltip");
            let hide_additional_tooltip = root.has_key("minecraft:hide_additional_tooltip");

            if hide_additional_tooltip {
                for component in ADDITIONAL_TOOLTIP_COMPONENTS {
                    if root.has_key(component) {
                        push_hidden(&mut hidden, component);
                    }
                }
            }

            root.take("minecraft:hide_tooltip");
            root.take("minecraft:hide_additional_tooltip");

            if hide_tooltip || !hidden.is_empty() {
                let mut tooltip_display = NbtMap::new();
                tooltip_display.set_bool("hide_tooltip", hide_tooltip);
                let hidden_list: Vec<NbtValue> = hidden.into_iter().map(NbtValue::String).collect();
                tooltip_display.set_list("hidden_components", hidden_list);
                root.set_map("minecraft:tooltip_display", tooltip_display);
            }
        }),
    );

    // REVERSE DATA_COMPONENTS: rebuild the pre-4307 tooltip schema from
    // `minecraft:tooltip_display` and re-nest the components the forward
    // unwrapped. Inverse of the structure converter above (V4307.java:484-599).
    //
    // The forward did three independent things, each separately invertible:
    //
    //   1. Unwrapped components, regardless of tooltip visibility:
    //      - can_break/can_place_on: component map -> its `predicates` value.
    //      - dyed_color/attribute_modifiers/enchantments/stored_enchantments/
    //        jukebox_playable: component map -> its inner value
    //        (rgb/modifiers/levels/song).
    //      Reverse: re-nest the current value back under the original key. This
    //      always runs when the component is present (the forward unwrapped
    //      unconditionally once the inner value existed); LOSSLESS.
    //
    //   2. Moved each component's `show_in_tooltip=false` flag into
    //      `tooltip_display.hidden_components` (the `true` default was simply
    //      dropped). Reverse: for each hidden path that is one of the nine
    //      flag-bearing components, restore `show_in_tooltip=false` on it; paths
    //      not listed keep the implicit `true` default. LOSSLESS (rule 11 — the
    //      old format always carried the default, so omitting it is exact).
    //
    //   3. Consumed the `minecraft:hide_tooltip` / `minecraft:hide_additional_tooltip`
    //      unit markers (empty-map components, cf. V3825.java:97). `hide_tooltip`
    //      maps 1:1 to `tooltip_display.hide_tooltip`. `hide_additional_tooltip`
    //      had expanded to *every present* ADDITIONAL_TOOLTIP_COMPONENTS entry in
    //      `hidden_components`; since those components never carry their own
    //      `show_in_tooltip`, seeing any of them in `hidden_components` uniquely
    //      implies the marker was set — restore it (LOSSLESS for these).
    //
    // Genuine forward loss with NO modern trace (rule 11, unreportable here):
    // if `hide_additional_tooltip` was set while *no* additional component was
    // present and `hide_tooltip` was false, the forward created no
    // `tooltip_display` at all, so the marker left no detectable residue. The
    // reverse cannot observe this case (nothing to inspect) and so emits no
    // `report_loss`; the information was irrecoverably dropped in the forward.
    reg.data_components.add_reverse_converter(
        VERSION,
        0,
        Box::new(|root: &mut NbtMap, _from, _to| {
            // (1) Re-nest the unwrapped components (independent of tooltip data).
            for path in PREDICATE_COMPONENTS {
                rewrap_under(root, path, "predicates");
            }
            for (path, inner_key) in UNWRAP_COMPONENTS {
                rewrap_under(root, path, inner_key);
            }

            // (2)+(3) Decode `minecraft:tooltip_display`.
            let tooltip_display = match root.take("minecraft:tooltip_display") {
                Some(NbtValue::Compound(m)) => m,
                // Not a compound / absent: nothing to decode. Put any non-compound
                // value back untouched (mirrors getMap()==null leniency).
                Some(other) => {
                    root.set_generic("minecraft:tooltip_display", other);
                    return;
                }
                None => return,
            };

            // (3a) hide_tooltip flag -> restore the unit marker.
            if tooltip_display.get_bool("hide_tooltip").unwrap_or(false) {
                root.set_map("minecraft:hide_tooltip", NbtMap::new());
            }

            let hidden: Vec<String> = tooltip_display
                .get_list("hidden_components")
                .map(|l| l.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();

            // A path is a per-component `show_in_tooltip` flag iff it is one of the
            // nine components the forward processed individually.
            let is_flag_component = |path: &str| -> bool {
                PREDICATE_COMPONENTS.contains(&path)
                    || PLAIN_TOOLTIP_COMPONENTS.contains(&path)
                    || UNWRAP_COMPONENTS.iter().any(|(p, _)| *p == path)
            };

            let mut saw_additional = false;
            for path in &hidden {
                if is_flag_component(path) {
                    // (2) restore show_in_tooltip=false on the (re-nested) component.
                    if let Some(component) = root.get_map_mut(path) {
                        component.set_bool("show_in_tooltip", false);
                    } else {
                        report_loss(
                            VERSION,
                            LossKind::UnsupportedInTarget,
                            Severity::Loss,
                            format!("hidden component path {path} has no component to carry legacy show_in_tooltip=false; dropping it"),
                        );
                    }
                } else if ADDITIONAL_TOOLTIP_COMPONENTS.contains(&path.as_str()) {
                    // (3b) came from hide_additional_tooltip.
                    saw_additional = true;
                } else {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        format!("hidden component path {path} has no pre-4307 representation; dropping it"),
                    );
                }
            }

            if saw_additional {
                let hidden_additional_count = hidden
                    .iter()
                    .filter(|path| ADDITIONAL_TOOLTIP_COMPONENTS.contains(&path.as_str()))
                    .count();
                let present_additional_count = ADDITIONAL_TOOLTIP_COMPONENTS
                    .iter()
                    .filter(|path| root.has_key(path))
                    .count();
                if hidden_additional_count != present_additional_count {
                    report_loss(
                        VERSION,
                        LossKind::FingerprintCollapse,
                        Severity::Approximated,
                        "hidden_components contains only a subset of present additional-tooltip components; restoring legacy hide_additional_tooltip marker hides all of them",
                    );
                }
                root.set_map("minecraft:hide_additional_tooltip", NbtMap::new());
            }
        }),
    );

    reg.data_components
        .add_structure_walker(VERSION, 0, Arc::new(data_components_walker));
}

/// Recurse a single block-predicate compound's `blocks` field (V4307.java:603-613).
/// `blocks` is either a BLOCK_NAME list or a single BLOCK_NAME value.
fn walk_block_predicate(
    reg: &Registry,
    data: &mut NbtMap,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    if matches!(data.get("blocks"), Some(NbtValue::List(_))) {
        convert_value_list(&reg.block_name, data, "blocks", from, to);
    } else {
        convert_value(&reg.block_name, data, "blocks", from, to);
    }
}

/// `walkBlockPredicates` (V4307.java:615-625): the value at `path` is either a
/// single predicate map or a list of predicate maps.
fn walk_block_predicates(
    reg: &Registry,
    root: &mut NbtMap,
    path: &str,
    from: EncodedVersion,
    to: EncodedVersion,
) {
    match root.get_mut(path) {
        Some(NbtValue::Compound(map)) => walk_block_predicate(reg, map, from, to),
        Some(NbtValue::List(list)) => {
            for el in list.iter_mut() {
                if let Some(map) = el.as_compound_mut() {
                    walk_block_predicate(reg, map, from, to);
                }
            }
        }
        _ => {}
    }
}

fn data_components_walker(
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

    walk_block_predicates(reg, root, "minecraft:can_break", from, to);
    walk_block_predicates(reg, root, "minecraft:can_place_on", from, to);

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
        // Java calls both convert (single) and convertList; only one fires per
        // backing type. `allowed_entities` may be a single ENTITY_NAME or a list.
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
        if let Some(pages) = written_book_content.get_list_mut("pages") {
            for page in pages.iter_mut() {
                if let NbtValue::Compound(map) = page {
                    // Filterable {raw, filtered} vs. a plain component compound.
                    if map.has_key("raw") || map.has_key("filtered") {
                        convert(reg, &reg.text_component, map, "raw", from, to);
                        convert(reg, &reg.text_component, map, "filtered", from, to);
                    } else {
                        reg.text_component.convert(reg, map, from, to);
                    }
                }
                // String / List page entries are stringified legacy components
                // and cannot be recursed through the compound TEXT_COMPONENT
                // type (see file-level note); left unchanged.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataconverter::loss;
    use crate::dataconverter::registry::{convert_reverse_under_session, registry};

    #[test]
    fn reverse_tooltip_display_reports_unknown_and_subset_hidden_components() {
        let mut root = NbtMap::new();
        root.set_list("minecraft:bees", Vec::new());
        root.set_list("minecraft:bundle_contents", Vec::new());
        let mut tooltip = NbtMap::new();
        tooltip.set_bool("hide_tooltip", false);
        tooltip.set_list(
            "hidden_components",
            vec![
                NbtValue::String("minecraft:bees".to_string()),
                NbtValue::String("minecraft:unknown_component".to_string()),
            ],
        );
        root.set_map("minecraft:tooltip_display", tooltip);

        let reg = registry();
        let (_, report) = loss::run_reverse(|| {
            convert_reverse_under_session(&reg.data_components, &mut root, VERSION, VERSION - 1);
        });

        assert_eq!(report.loss_count(), 1);
        assert_eq!(report.len(), 2);
        assert!(root.has_key("minecraft:hide_additional_tooltip"));
    }
}
