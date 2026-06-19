//! V3820 (24w09a + 1) — schematic-relevant subset of `V3820.java`.
//!
//! Ported (all schematic-relevant):
//!   * TILE_ENTITY `minecraft:skull`: migrate the legacy `SkullOwner`/`ExtraType`
//!     fields into the 1.20.5 `profile` component (V3820.java:23-45).
//!   * ITEM_STACK structure converter: rename the `minecraft:lodestone_target`
//!     component to `minecraft:lodestone_tracker`, nesting its `pos`+`dimension`
//!     under a new `target` map (V3820.java:46-82).
//!   * TILE_ENTITY `minecraft:skull` walker: `custom_name` -> TEXT_COMPONENT
//!     (V3820.java:85).
//!
//! Nothing non-schematic exists in this version file.
//!
//! `convertProfile` / `convertProperties` / `isValidPlayerName` are ported inline
//! from `ConverterItemStackToDataComponents` (the 1.20.5 component-split helper),
//! since that helper is not (yet) a shared module here.
//!
//! VERSION = MCVersions.V24W09A (3819) + 1 = 3820.

use std::sync::Arc;

use crate::nbt::{NbtMap, NbtValue};

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use super::super::walker::convert;

const VERSION: i32 = 3820;

/// `isValidPlayerName` (ConverterItemStackToDataComponents:834-846): <= 16 chars,
/// all printable ASCII (0x21..=0x7E).
fn is_valid_player_name(name: &str) -> bool {
    if name.chars().count() > 16 {
        return false;
    }
    name.chars().all(|c| {
        let n = c as u32;
        n > 0x20 && n < 0x7F
    })
}

/// `convertProperties` (…:849-877): flatten the `{key: [{Value,Signature}…]}`
/// profile-properties map into a list of `{name,value,[signature]}` maps.
fn convert_properties(properties: &NbtMap) -> Vec<NbtValue> {
    let mut ret = Vec::new();
    for (property_key, values) in properties.iter() {
        let list = match values.as_list_ref() {
            Some(l) => l,
            None => continue,
        };
        for el in list {
            // getGeneric(i) instanceof MapType -> else null.
            let property = el.as_compound_ref();
            let value = property
                .and_then(|p| p.get_string("Value"))
                .unwrap_or("")
                .to_string();
            let signature = property
                .and_then(|p| p.get_string("Signature"))
                .map(|s| s.to_string());

            let mut new_property = NbtMap::new();
            new_property.set_string("name", property_key.clone());
            new_property.set_string("value", value);
            if let Some(sig) = signature {
                new_property.set_string("signature", sig);
            }
            ret.push(NbtValue::Compound(new_property));
        }
    }
    ret
}

/// `convertProfile` (…:879-911): build a profile component from either a bare
/// player-name string or a legacy `{Name,Id,Properties}` compound.
fn convert_profile(input: Option<NbtValue>) -> NbtMap {
    let mut ret = NbtMap::new();

    match input {
        Some(NbtValue::String(name)) => {
            if is_valid_player_name(&name) {
                ret.set_string("name", name);
            }
            ret
        }
        Some(NbtValue::Compound(input)) => {
            // getString("Name", "") — default "" when absent.
            let name = input.get_string("Name").unwrap_or("");
            if is_valid_player_name(name) {
                ret.set_string("name", name);
            }
            if let Some(id) = input.get("Id") {
                ret.set_generic("id", id.clone());
            }
            if let Some(properties) = input.get_map("Properties") {
                if !properties.inner().is_empty() {
                    ret.set_list("properties", convert_properties(properties));
                }
            }
            ret
        }
        // Neither string nor map: empty profile.
        _ => ret,
    }
}

/// Inverse of [`convert_properties`]: list-of-`{name, value, signature?}` back to
/// a `{name -> [{Value, Signature?}]}` map. Mirrors `components::unconvert_properties`.
fn unconvert_properties(properties: &[NbtValue]) -> NbtMap {
    let mut ret = NbtMap::new();
    for p in properties {
        let Some(pm) = p.as_compound_ref() else {
            continue;
        };
        let name = pm.get_string("name").unwrap_or("").to_string();
        let value = pm.get_string("value").unwrap_or("").to_string();
        let signature = pm.get_string("signature").map(str::to_string);

        let mut entry = NbtMap::new();
        entry.set_string("Value", value);
        if let Some(sig) = signature {
            entry.set_string("Signature", sig);
        }

        match ret.get_list_mut(&name) {
            Some(list) => list.push(NbtValue::Compound(entry)),
            None => ret.set_list(&name, vec![NbtValue::Compound(entry)]),
        }
    }
    ret
}

/// Inverse of [`convert_profile`] -> a legacy `SkullOwner` value (string or
/// compound). Mirrors `components::unconvert_profile`.
///
/// The forward emitted a bare `{name}` profile when the source was a string, and
/// a `{name, id?, properties?}` profile when the source was a `{Name, Id?,
/// Properties?}` compound. The presence of `id`/`properties` in the modern profile
/// uniquely identifies the compound preimage, so this inverse is exact (lossless):
/// only-name -> the string form, otherwise the `{Name, Id?, Properties?}` compound.
fn unconvert_profile(profile: &NbtMap) -> NbtValue {
    let has_id = profile.has_key("id");
    let has_props = profile.get_map("properties").is_some()
        || matches!(profile.get("properties"), Some(NbtValue::List(_)));

    // Bare {name} came from a string; restore the string form.
    if !has_id && !has_props {
        if let Some(name) = profile.get_string("name") {
            return NbtValue::String(name.to_string());
        }
        // empty profile -> empty compound
        return NbtValue::Compound(NbtMap::new());
    }

    let mut ret = NbtMap::new();
    if let Some(name) = profile.get_string("name") {
        ret.set_string("Name", name);
    }
    if let Some(id) = profile.get("id") {
        ret.set_generic("Id", id.clone());
    }
    if let Some(props) = profile.get_list("properties") {
        ret.set_map("Properties", unconvert_properties(props));
    }
    NbtValue::Compound(ret)
}

pub fn register(reg: &mut RegistryBuilder) {
    // TILE_ENTITY skull: SkullOwner/ExtraType -> profile component.
    reg.tile_entity.add_converter_for_id(
        "minecraft:skull",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let skull_owner = data.take("SkullOwner");
            let extra_type = data.take("ExtraType");

            if skull_owner.is_none() && extra_type.is_none() {
                return;
            }

            // skullOwner == null ? extraType : skullOwner.
            let input = if skull_owner.is_some() {
                skull_owner
            } else {
                extra_type
            };
            data.set_map("profile", convert_profile(input));
        }),
    );

    // REVERSE skull: profile component -> legacy SkullOwner. Inverse of
    // convert_profile. The forward collapsed both legacy keys (SkullOwner and the
    // even-older ExtraType) into `profile` via `skullOwner == null ? extraType :
    // skullOwner`; the canonical pre-3820 key is SkullOwner, so we restore that
    // and report the key-choice approximation.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:skull",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let Some(NbtValue::Compound(profile)) = data.take("profile") else { return };
            data.set_generic("SkullOwner", unconvert_profile(&profile));
            report_loss(
                VERSION,
                LossKind::FingerprintCollapse,
                Severity::Approximated,
                "skull profile may have come from legacy SkullOwner or ExtraType; restored canonical SkullOwner",
            );
        }),
    );

    // ITEM_STACK: lodestone_target -> lodestone_tracker, nesting pos+dimension.
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let components = match data.get_map_mut("components") {
                Some(c) => c,
                None => return,
            };

            let mut old_target = match components.take("minecraft:lodestone_target") {
                Some(NbtValue::Compound(m)) => m,
                // No lodestone_target (or wrong type) -> nothing to do. Put a
                // non-compound value back untouched to mirror getMap()==null early
                // return (a non-map would have read as null and not been removed).
                Some(other) => {
                    components.set_generic("minecraft:lodestone_target", other);
                    return;
                }
                None => return,
            };

            // Move pos+dimension under a `target` sub-map when both are present.
            let pos = old_target.get("pos").cloned();
            let dim = old_target.get("dimension").cloned();
            if let (Some(pos), Some(dim)) = (pos, dim) {
                old_target.take("pos");
                old_target.take("dimension");
                let mut target = NbtMap::new();
                target.set_generic("pos", pos);
                target.set_generic("dimension", dim);
                old_target.set_map("target", target);
            }

            components.set_map("minecraft:lodestone_tracker", old_target);
        }),
    );

    // REVERSE ITEM_STACK: lodestone_tracker -> lodestone_target, un-nesting the
    // `target.{pos,dimension}` sub-map back to top-level pos+dimension. Exact
    // inverse of the forward structural rename/restructure (lossless).
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let Some(components) = data.get_map_mut("components") else {
                return;
            };

            let mut tracker = match components.take("minecraft:lodestone_tracker") {
                Some(NbtValue::Compound(m)) => m,
                // Wrong type / absent -> nothing to do; restore non-compound untouched.
                Some(other) => {
                    components.set_generic("minecraft:lodestone_tracker", other);
                    return;
                }
                None => return,
            };

            // Un-nest target.{pos,dimension} back onto the parent. The forward only
            // created `target` when both pos+dimension were present, and moved both;
            // mirror that by hoisting them out and dropping the (now-empty) target.
            if let Some(NbtValue::Compound(mut target)) = tracker.take("target") {
                if let Some(pos) = target.take("pos") {
                    tracker.set_generic("pos", pos);
                }
                if let Some(dim) = target.take("dimension") {
                    tracker.set_generic("dimension", dim);
                }
                // Restore any other keys the forward never touched.
                for key in target.keys() {
                    if let Some(v) = target.take(&key) {
                        tracker.set_generic(&key, v);
                    }
                }
            }

            components.set_map("minecraft:lodestone_target", tracker);
        }),
    );

    // TILE_ENTITY skull walker: custom_name -> TEXT_COMPONENT.
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "minecraft:skull",
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.text_component, data, "custom_name", from, to);
        }),
    );
}
