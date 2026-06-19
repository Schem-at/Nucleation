//! V4299 (25w02a + 1) — schematic-relevant subset of `V4299.java`.
//!
//! ITEM_STACK structure converter that lifts bucketed-mob data out of the
//! legacy `minecraft:bucket_entity_data` component (and the painting variant out
//! of `minecraft:entity_data`) into dedicated mob-variant components
//! (V4299.java:14-138):
//!   * `minecraft:axolotl_bucket`  -> `minecraft:axolotl/variant` (index lookup).
//!   * `minecraft:salmon_bucket`   -> `minecraft:salmon/size` (moves `type`).
//!   * `minecraft:tropical_fish_bucket` -> pattern/base_color/pattern_color from
//!     the packed `BucketVariantTag` int (low 16 = pattern, bits 16..24 = base
//!     colour, bits 24..32 = pattern colour).
//!   * `minecraft:painting` -> `minecraft:painting/variant` (moves the painting
//!     entity's `variant`, dropping `minecraft:entity_data` once emptied).
//!
//! Nothing non-schematic is present in this version. VERSION = V25W02A(4298) + 1.
//!
//! The fish-pattern and banner-colour lookup tables are inlined here because the
//! V3818 file they live in (which exposes `getBannerColour`) is not yet ported;
//! the tables match V3818.java:22-43 / V4299.java:24-38 exactly.

use crate::nbt::NbtMap;

use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 4299;

/// `AXOLOTL_VARIANT_LOOKUP` (V4299.java:16-22); out-of-range -> index 0 ("lucy").
const AXOLOTL_VARIANT_LOOKUP: &[&str] = &["lucy", "wild", "gold", "cyan", "blue"];

fn lookup_axolotl(index: i32) -> &'static str {
    if index >= 0 && (index as usize) < AXOLOTL_VARIANT_LOOKUP.len() {
        AXOLOTL_VARIANT_LOOKUP[index as usize]
    } else {
        AXOLOTL_VARIANT_LOOKUP[0]
    }
}

/// `FISH_PATTERN_LOOKUP` (V4299.java:24-38); default ("kob") for unknown keys.
fn lookup_fish_pattern(variant: i32) -> &'static str {
    match variant {
        1 => "flopper",
        256 => "sunstreak",
        257 => "stripey",
        512 => "snooper",
        513 => "glitter",
        768 => "dasher",
        769 => "blockfish",
        1024 => "brinely",
        1025 => "betty",
        1280 => "spotty",
        1281 => "clayfish",
        _ => "kob",
    }
}

/// `V3818.getBannerColour` (V3818.java:22-43); out-of-range -> "white".
const BANNER_COLOURS: &[&str] = &[
    "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray", "light_gray",
    "cyan", "purple", "blue", "brown", "green", "red", "black",
];

fn banner_colour(id: i32) -> &'static str {
    if id >= 0 && (id as usize) < BANNER_COLOURS.len() {
        BANNER_COLOURS[id as usize]
    } else {
        BANNER_COLOURS[0]
    }
}

// ---------------------------------------------------------------------------
// REVERSE-direction lookup inverses.
// ---------------------------------------------------------------------------

/// Inverse of [`lookup_axolotl`]: variant name -> index. The forward clamps
/// out-of-range indices to 0 ("lucy"), but for a real downgrade every modern
/// `minecraft:axolotl/variant` name is one of these five, so the inverse is exact
/// (index 0 is the canonical preimage of "lucy"). Unknown names -> 0.
fn axolotl_index(name: &str) -> i32 {
    AXOLOTL_VARIANT_LOOKUP
        .iter()
        .position(|&n| n == name)
        .map(|i| i as i32)
        .unwrap_or(0)
}

/// Inverse of [`lookup_fish_pattern`]: pattern name -> low-16-bit variant code.
/// The forward table is a bijection on its listed entries (each int maps to a
/// distinct pattern), with "kob" the default for the unlisted key 0. So the
/// inverse is exact; unknown names fold back to the "kob" code (0).
fn fish_pattern_code(pattern: &str) -> i32 {
    match pattern {
        "flopper" => 1,
        "sunstreak" => 256,
        "stripey" => 257,
        "snooper" => 512,
        "glitter" => 513,
        "dasher" => 768,
        "blockfish" => 769,
        "brinely" => 1024,
        "betty" => 1025,
        "spotty" => 1280,
        "clayfish" => 1281,
        // "kob" and anything unrecognised pack to 0.
        _ => 0,
    }
}

/// Inverse of [`banner_colour`]: colour name -> 0..15 id. The forward clamps
/// out-of-range ids to "white" (0), but every modern colour name round-trips to
/// its real id; unknown names fold back to white (0, the canonical preimage).
fn banner_colour_id(name: &str) -> i32 {
    (0..BANNER_COLOURS.len() as i32).find(|&i| banner_colour(i) == name).unwrap_or(0)
}

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // No components -> nothing to migrate (Java returns null).
            if data.get_map("components").is_none() {
                return;
            }

            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };

            match id.as_str() {
                "minecraft:axolotl_bucket" => {
                    // Read Variant out of bucket_entity_data, then write the new
                    // component on the parent components map.
                    let variant = {
                        let components = data.get_map_mut("components").unwrap();
                        let bucket = match components.get_map_mut("minecraft:bucket_entity_data") {
                            Some(b) => b,
                            None => return,
                        };
                        match bucket.get_i32("Variant") {
                            Some(v) => {
                                bucket.take("Variant");
                                v
                            }
                            None => return,
                        }
                    };
                    let components = data.get_map_mut("components").unwrap();
                    components.set_string("minecraft:axolotl/variant", lookup_axolotl(variant));
                }
                "minecraft:salmon_bucket" => {
                    let type_value = {
                        let components = data.get_map_mut("components").unwrap();
                        let bucket = match components.get_map_mut("minecraft:bucket_entity_data") {
                            Some(b) => b,
                            None => return,
                        };
                        match bucket.take("type") {
                            Some(v) => v,
                            None => return,
                        }
                    };
                    let components = data.get_map_mut("components").unwrap();
                    components.set_generic("minecraft:salmon/size", type_value);
                }
                "minecraft:tropical_fish_bucket" => {
                    let variant = {
                        let components = data.get_map_mut("components").unwrap();
                        let bucket = match components.get_map_mut("minecraft:bucket_entity_data") {
                            Some(b) => b,
                            None => return,
                        };
                        match bucket.get_i32("BucketVariantTag") {
                            Some(v) => {
                                bucket.take("BucketVariantTag");
                                v
                            }
                            None => return,
                        }
                    };

                    let fish_pattern = lookup_fish_pattern(variant & 0xFFFF);
                    // `>>> 16` etc. — unsigned shift on the 32-bit int.
                    let v = variant as u32;
                    let base_colour = banner_colour(((v >> 16) & 0xFF) as i32);
                    let pattern_colour = banner_colour(((v >> 24) & 0xFF) as i32);

                    let components = data.get_map_mut("components").unwrap();
                    components.set_string("minecraft:tropical_fish/pattern", fish_pattern);
                    components.set_string("minecraft:tropical_fish/base_color", base_colour);
                    components.set_string("minecraft:tropical_fish/pattern_color", pattern_colour);
                }
                "minecraft:painting" => {
                    let (variant, should_remove_entity_data) = {
                        let components = data.get_map_mut("components").unwrap();
                        let entity_data = match components.get_map_mut("minecraft:entity_data") {
                            Some(e) => e,
                            None => return,
                        };
                        if entity_data.get_string("id") != Some("minecraft:painting") {
                            return;
                        }
                        let variant = entity_data.take("variant");
                        // After removing `variant`, if only `id` remains the whole
                        // entity_data component is dropped (Java: size() == 1).
                        let remove = entity_data.inner().len() == 1;
                        (variant, remove)
                    };

                    let components = data.get_map_mut("components").unwrap();
                    if let Some(variant) = variant {
                        components.set_generic("minecraft:painting/variant", variant);
                    }
                    if should_remove_entity_data {
                        components.take("minecraft:entity_data");
                    }
                }
                _ => {}
            }
        }),
    );

    // ======================================================================
    // REVERSE (new -> old): the inverse of the bucketed-mob / painting lift.
    // Each new mob-variant component is moved back into the legacy
    // `minecraft:bucket_entity_data` (or `minecraft:entity_data`) compound, in
    // exactly the encoding the forward read it from. All four cases are
    // lossless for real downgrades: the new component uniquely encodes the old
    // value (the forward's out-of-range clamps only fold genuinely invalid
    // inputs, which never arise from modern data — see the *_index/_id helpers).
    // ======================================================================
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Mirror the forward guards: need a `components` map and an `id`.
            if data.get_map("components").is_none() {
                return;
            }
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };

            match id.as_str() {
                "minecraft:axolotl_bucket" => {
                    // axolotl/variant name -> Variant int back inside
                    // bucket_entity_data (created if the forward had emptied it).
                    let components = data.get_map_mut("components").unwrap();
                    let variant = match components.get_string("minecraft:axolotl/variant") {
                        Some(name) => axolotl_index(name),
                        None => return,
                    };
                    components.take("minecraft:axolotl/variant");
                    if components.get_map("minecraft:bucket_entity_data").is_none() {
                        components.set_map("minecraft:bucket_entity_data", NbtMap::new());
                    }
                    let bucket = components.get_map_mut("minecraft:bucket_entity_data").unwrap();
                    bucket.set_i32("Variant", variant);
                }
                "minecraft:salmon_bucket" => {
                    // salmon/size generic -> `type` back inside bucket_entity_data.
                    let components = data.get_map_mut("components").unwrap();
                    let size = match components.take("minecraft:salmon/size") {
                        Some(v) => v,
                        None => return,
                    };
                    if components.get_map("minecraft:bucket_entity_data").is_none() {
                        components.set_map("minecraft:bucket_entity_data", NbtMap::new());
                    }
                    let bucket = components.get_map_mut("minecraft:bucket_entity_data").unwrap();
                    bucket.set_generic("type", size);
                }
                "minecraft:tropical_fish_bucket" => {
                    // Repack pattern/base_color/pattern_color into the
                    // BucketVariantTag int: low 16 = pattern, bits 16..24 = base
                    // colour, bits 24..32 = pattern colour (inverse of the
                    // `>>> 16` / `>>> 24` unpack).
                    let components = data.get_map_mut("components").unwrap();
                    // Only act when at least one of the three was present
                    // (the forward always wrote all three together).
                    let pattern = components.take("minecraft:tropical_fish/pattern");
                    let base = components.take("minecraft:tropical_fish/base_color");
                    let pattern_col = components.take("minecraft:tropical_fish/pattern_color");
                    if pattern.is_none() && base.is_none() && pattern_col.is_none() {
                        return;
                    }

                    let pattern_code = pattern
                        .as_ref()
                        .and_then(|v| match v {
                            crate::nbt::NbtValue::String(s) => Some(fish_pattern_code(s)),
                            _ => None,
                        })
                        .unwrap_or(0);
                    let base_id = base
                        .as_ref()
                        .and_then(|v| match v {
                            crate::nbt::NbtValue::String(s) => Some(banner_colour_id(s)),
                            _ => None,
                        })
                        .unwrap_or(0);
                    let pattern_col_id = pattern_col
                        .as_ref()
                        .and_then(|v| match v {
                            crate::nbt::NbtValue::String(s) => Some(banner_colour_id(s)),
                            _ => None,
                        })
                        .unwrap_or(0);

                    let variant = ((pattern_code as u32) & 0xFFFF)
                        | (((base_id as u32) & 0xFF) << 16)
                        | (((pattern_col_id as u32) & 0xFF) << 24);

                    if components.get_map("minecraft:bucket_entity_data").is_none() {
                        components.set_map("minecraft:bucket_entity_data", NbtMap::new());
                    }
                    let bucket = components.get_map_mut("minecraft:bucket_entity_data").unwrap();
                    bucket.set_i32("BucketVariantTag", variant as i32);
                }
                "minecraft:painting" => {
                    // painting/variant generic -> `variant` back inside
                    // entity_data, recreating the painting entity_data compound
                    // (with id=minecraft:painting) when the forward dropped it.
                    let components = data.get_map_mut("components").unwrap();
                    let variant = match components.take("minecraft:painting/variant") {
                        Some(v) => v,
                        None => return,
                    };
                    if components.get_map("minecraft:entity_data").is_none() {
                        let mut ed = NbtMap::new();
                        ed.set_string("id", "minecraft:painting");
                        components.set_map("minecraft:entity_data", ed);
                    }
                    let entity_data = components.get_map_mut("minecraft:entity_data").unwrap();
                    // Ensure the id is present (forward only acted on painting
                    // entity_data, so this matches the legacy shape).
                    if entity_data.get_string("id").is_none() {
                        entity_data.set_string("id", "minecraft:painting");
                    }
                    entity_data.set_generic("variant", variant);
                }
                _ => {}
            }
        }),
    );
}
