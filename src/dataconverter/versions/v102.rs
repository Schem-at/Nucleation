//! V102 (15w32a+2) — legacy numeric item ids -> string item names; cites V102.java.
//! Ports ITEM_NAME numeric->string converter, ITEM_STACK numeric-id->string-name
//! structure converter, and the minecraft:potion id converter. The schema's
//! ITEM_STACK string-only-id walker change needs no Rust update because ITEM_NAME
//! is already generic (int or String), per V102.java:21-22.
use std::collections::HashMap;
use std::sync::LazyLock;

use super::super::flattening::{get_name_from_id, get_potion_name_from_id};
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};
use crate::nbt::{NbtMap, NbtValue};

const VERSION: i32 = 102;

// --- reverse (new -> old) lookup tables, inverted from the forward helpers ---
//
// HelperItemNameV102.ITEM_NAMES is 1:1 (no two ids share a name — verified
// against flattening::data::ITEM_NAMES_BY_ID), so name -> id is exact. Built by
// scanning the full id range 0..=2267 (max table id is the record_wait = 2267)
// and inverting `get_name_from_id`.
static ITEM_ID_BY_NAME: LazyLock<HashMap<&'static str, i32>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    for id in 0..=2267i32 {
        if let Some(name) = get_name_from_id(id) {
            m.entry(name).or_insert(id);
        }
    }
    m
});

// HelperItemNameV102.POTION_NAMES is heavily many-to-one (e.g. minecraft:water,
// minecraft:regeneration map from many damage values), so name -> damage is
// genuinely ambiguous. We keep the *lowest* base id (0..=127) per name as the
// canonical preimage; the splash bit (16384) is restored separately from the id.
static POTION_BASE_ID_BY_NAME: LazyLock<HashMap<&'static str, i16>> = LazyLock::new(|| {
    let mut m: HashMap<&'static str, i16> = HashMap::new();
    for id in 0..=127i16 {
        if let Some(name) = get_potion_name_from_id(id) {
            m.entry(name)
                .and_modify(|e| {
                    if id < *e {
                        *e = id;
                    }
                })
                .or_insert(id);
        }
    }
    m
});

pub fn register(reg: &mut RegistryBuilder) {
    // ITEM_NAME: numeric legacy id -> string item name (default id 0 when unknown).
    reg.item_name.add_converter(
        VERSION,
        0,
        Box::new(|val: &mut NbtValue, _from, _to| {
            // Java returns null (no change) when data isn't a Number.
            if let Some(id) = val.as_number_i64() {
                let id = id as i32;
                let remap = get_name_from_id(id)
                    .or_else(|| get_name_from_id(0))
                    .unwrap_or("minecraft:air");
                *val = NbtValue::String(remap.to_string());
            }
        }),
    );

    // REVERSE of ITEM_NAME: string name -> numeric legacy id. ITEM_NAMES is 1:1,
    // so any name the forward could have produced reverses exactly (bucket B,
    // lossless). A name with no legacy integer id (a modern item that never had
    // one) cannot be represented numerically: leave the string and report loss.
    reg.item_name.add_reverse_converter(
        VERSION,
        0,
        Box::new(|val: &mut NbtValue, _from, _to| {
            if let NbtValue::String(name) = val {
                if let Some(&id) = ITEM_ID_BY_NAME.get(name.as_str()) {
                    *val = NbtValue::Int(id);
                } else {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        "item name has no legacy numeric id (V102 ITEM_NAME reverse)",
                    );
                }
            }
        }),
    );

    // ITEM_STACK: when id is numeric, replace it with the string name (default id 0).
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // Java guard: hasKey("id", NUMBER) — only act when id is present and numeric.
            let numeric_id = match data.get("id") {
                Some(v) => v.as_number_i64(),
                None => None,
            };
            if let Some(id) = numeric_id {
                let id = id as i32;
                let remap = get_name_from_id(id)
                    .or_else(|| get_name_from_id(0))
                    .unwrap_or("minecraft:air");
                data.set_string("id", remap);
            }
        }),
    );

    // REVERSE of ITEM_STACK structure: string id -> numeric legacy id. Registered
    // after the potion for-id reverse below, so under the descending sweep
    // (reverse_converters run in .rev() order) the potion reverse runs FIRST and
    // restores id to the "minecraft:potion" string; THEN this turns the string id
    // back into the numeric id (e.g. 373). Lossless for any name the forward could
    // have emitted (bucket B); a modern-only name with no legacy id is left as a
    // string and reported (a pre-V102 reader expects a number).
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let name = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return, // already numeric or absent — nothing to undo.
            };
            if let Some(&id) = ITEM_ID_BY_NAME.get(name.as_str()) {
                data.set_i32("id", id);
            } else {
                report_loss(
                    VERSION,
                    LossKind::UnsupportedInTarget,
                    Severity::Loss,
                    "item stack id has no legacy numeric id (V102 ITEM_STACK reverse)",
                );
            }
        }),
    );

    // ITEM_STACK minecraft:potion: zero Damage, set tag.Potion, splash on bit 16384.
    reg.item_stack.add_converter_for_id(
        "minecraft:potion",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let damage = data.get_i32("Damage").unwrap_or(0) as i16;
            if damage != 0 {
                data.set_short("Damage", 0);
            }

            if data.get_map("tag").is_none() {
                data.set_map("tag", NbtMap::new());
            }

            // Only set Potion if not already a string.
            let has_potion_string = data
                .get_map("tag")
                .and_then(|t| t.get_string("Potion"))
                .is_some();
            if !has_potion_string {
                let converted = get_potion_name_from_id(damage).unwrap_or("minecraft:water");
                let tag = data.get_map_mut("tag").unwrap();
                tag.set_string("Potion", converted);
                if (damage & 16384) == 16384 {
                    data.set_string("id", "minecraft:splash_potion");
                }
            }
        }),
    );

    // REVERSE of the minecraft:potion converter. The forward read the legacy
    // numeric `Damage` (a potion discriminator + splash bit 16384), zeroed it,
    // stored the resolved name in tag.Potion, and split the splash variant into
    // the id "minecraft:splash_potion". The reverse reconstructs `Damage` from
    // tag.Potion and the id:
    //   * id "minecraft:splash_potion" -> base id | 16384, id back to "minecraft:potion".
    //   * id "minecraft:potion"        -> base id.
    // POTION_NAMES is many-to-one, so the exact original Damage cannot be
    // recovered from the name — we use the lowest matching base id as the
    // canonical preimage (Approximated). Runs BEFORE the ITEM_STACK structure
    // reverse (it is registered later), so `id` is still the string here and we
    // hand back "minecraft:potion" for that converter to renumber.
    //
    // Matches the NEW ids the forward produced (cheatsheet rule 4).
    for new_id in ["minecraft:potion", "minecraft:splash_potion"] {
        reg.item_stack.add_reverse_converter_for_id(
            new_id,
            VERSION,
            0,
            Box::new(move |data: &mut NbtMap, _from, _to| {
                // The forward only wrote tag.Potion when there was no Potion string
                // already; if it is absent we cannot reconstruct Damage — leave as is.
                let potion_name = data
                    .get_map("tag")
                    .and_then(|t| t.get_string("Potion"))
                    .map(|s| s.to_string());
                let potion_name = match potion_name {
                    Some(p) => p,
                    None => return,
                };

                let base = POTION_BASE_ID_BY_NAME
                    .get(potion_name.as_str())
                    .copied()
                    .unwrap_or(0);

                let is_splash = new_id == "minecraft:splash_potion";
                let damage = if is_splash { base | 16384 } else { base };

                // The legacy Damage is the canonical preimage, not the true original
                // (the name lost the exact id within its equivalence class).
                report_loss(
                    VERSION,
                    LossKind::FingerprintCollapse,
                    Severity::Approximated,
                    "legacy potion Damage approximated from tag.Potion (many ids share one potion name)",
                );

                data.set_short("Damage", damage);
                // Undo the id split so the ITEM_STACK structure reverse renumbers it.
                data.set_string("id", "minecraft:potion");
                // Remove the modern tag.Potion the forward introduced; drop tag if
                // it is now empty.
                if let Some(tag) = data.get_map_mut("tag") {
                    tag.take("Potion");
                    if tag.is_empty() {
                        data.take("tag");
                    }
                }
            }),
        );
    }
}
