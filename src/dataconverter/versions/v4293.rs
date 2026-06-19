//! V4293 (1.21.4 + 104) — schematic-relevant subset of `V4293.java`.
//!
//! VERSION = MCVersions.V1_21_4 (4189) + 104 = 4293.
//!
//! Ported (ENTITY structure converter, V4293.java:824-877): collapse the legacy
//! per-slot drop-chance lists into a single `drop_chances` compound. The old
//! `ArmorDropChances` (feet/legs/chest/head) and `HandDropChances`
//! (mainhand/offhand) float lists plus the scalar `body_armor_drop_chance` are
//! removed; any slot whose chance differs from the default `0.085f` is written
//! under its named key in `drop_chances`, which is only added when non-empty.
//!
//! Nothing non-schematic is present in this version.
//!
//! Reverse (new -> old): rebuild the legacy per-slot float lists/scalar from the
//! `drop_chances` compound. Each forward-omitted slot was DEFAULT (`0.085f`), and
//! the old format reads these lists with `getFloat(i, DEFAULT)`, so a DEFAULT-fill
//! is read-identical to the original — a lossless inverse for real downgrades.

use crate::nbt::{NbtMap, NbtValue};

use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 4293;

const DEFAULT: f32 = 0.085;

const ARMOR_SLOTS: &[&str] = &["feet", "legs", "chest", "head"];
const HAND_SLOTS: &[&str] = &["mainhand", "offhand"];

/// `V4293.convertDropChances`: copy the (up to `names.len()`) non-default float
/// chances from `data[src_path]` into `dst[name]`.
fn convert_drop_chances(data: &NbtMap, src_path: &str, names: &[&str], dst: &mut NbtMap) {
    let old_chances = match data.get_list(src_path) {
        Some(l) => l,
        None => return,
    };

    let len = old_chances.len().min(names.len());
    for (i, name) in names.iter().enumerate().take(len) {
        // getFloat(i, DEFAULT): a non-float element falls back to the default.
        let chance = old_chances[i].as_number_f64().map(|v| v as f32).unwrap_or(DEFAULT);
        if chance != DEFAULT {
            dst.set_f32(name, chance);
        }
    }
}

/// Reverse of `convert_drop_chances`: rebuild a float list `data[dst_path]` of
/// length `names.len()`, reading each slot from `src[name]` (the value the
/// forward wrote when it was non-default) or falling back to DEFAULT (the value
/// the forward omitted). The list is written only if any slot was present in
/// `src`, matching the canonical old shape for mobs that carried drop chances.
fn reverse_drop_chances(src: &NbtMap, names: &[&str], dst: &mut NbtMap, dst_path: &str) {
    let any_present = names.iter().any(|name| src.get(name).is_some());
    if !any_present {
        return;
    }

    let list: Vec<NbtValue> = names
        .iter()
        .map(|name| {
            let chance = src
                .get(name)
                .and_then(|v| v.as_number_f64())
                .map(|v| v as f32)
                .unwrap_or(DEFAULT);
            NbtValue::Float(chance)
        })
        .collect();
    dst.set_list(dst_path, list);
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let mut drop_chances = NbtMap::new();

            convert_drop_chances(data, "ArmorDropChances", ARMOR_SLOTS, &mut drop_chances);
            convert_drop_chances(data, "HandDropChances", HAND_SLOTS, &mut drop_chances);

            data.take("ArmorDropChances");
            data.take("HandDropChances");

            // getFloat("body_armor_drop_chance", DEFAULT).
            let body = data
                .get("body_armor_drop_chance")
                .and_then(|v| v.as_number_f64())
                .map(|v| v as f32)
                .unwrap_or(DEFAULT);
            data.take("body_armor_drop_chance");

            if body != DEFAULT {
                drop_chances.set_f32("body", body);
            }

            if drop_chances.iter().next().is_some() {
                data.set_map("drop_chances", drop_chances);
            }
        }),
    );

    // Reverse: rebuild the legacy `ArmorDropChances` / `HandDropChances` lists and
    // the `body_armor_drop_chance` scalar from the `drop_chances` compound, then
    // remove `drop_chances`. Slots the forward omitted (they equalled DEFAULT) are
    // restored as DEFAULT, which the old `getFloat(i, DEFAULT)` reads identically;
    // lossless for real downgrades. Newer slots (e.g. `saddle` from V4300) are
    // already undone by the time this runs, so only the armor/hand/body keys
    // remain to consume.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Pull the compound out; if absent there is nothing to restore.
            let drop_chances = match data.take("drop_chances") {
                Some(NbtValue::Compound(m)) => m,
                Some(other) => {
                    // Not a compound: leave it removed-and-reattached untouched.
                    data.set_generic("drop_chances", other);
                    return;
                }
                None => return,
            };

            reverse_drop_chances(&drop_chances, ARMOR_SLOTS, data, "ArmorDropChances");
            reverse_drop_chances(&drop_chances, HAND_SLOTS, data, "HandDropChances");

            if let Some(body) = drop_chances.get("body").and_then(|v| v.as_number_f64()) {
                data.set_f32("body_armor_drop_chance", body as f32);
            }
        }),
    );
}
