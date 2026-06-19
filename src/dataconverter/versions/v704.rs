//! V704 (1.10.2 + 192) — legacy block-entity ids -> namespaced ids, plus the
//! version-aware ITEM_STACK structure walker.
//!
//! Ported from
//! `DataConverterJava/.../minecraft/versions/V704.java`.
//!
//! Schematic-relevant registrations ported:
//!   * TILE_ENTITY structure converter: legacy id (e.g. `Chest`) -> namespaced
//!     id (e.g. `minecraft:chest`) via `TILE_ID_UPDATE`.
//!   * `copyWalkers` for every renamed tile-entity id (so the V99 walkers
//!     registered under the legacy id also fire under the new id).
//!   * ITEM_STACK structure walker (replaces the V99 one): recurses `id`, nested
//!     `tag` items / written-book pages / display Name+Lore (version-gated) /
//!     EntityTag (with spawn-egg + version-floor id resolution) / BlockEntityTag
//!     (with suspicious_sand special case) / CanDestroy / CanPlaceOn.
//!   * TILE_ENTITY namespaced-id structure hook.
//!
//! Skipped (non-schematic / dev-only): the Vanilla item-registry integrity check
//! (commented out in Java), and all `LOGGER.error/warn` diagnostics (the
//! `IGNORE_ABSENT_WALKERS` set and "Failed to find walkers" / "Unable to resolve"
//! warnings are purely logging and do not affect output data).

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::super::helpers::enforce_namespaced_id_hook;
use super::super::registry::{Registry, RegistryBuilder};
use super::super::types::MapExt;
use super::super::version::{encode_versions, EncodedVersion};
use super::super::walker::{convert_list, convert_value, convert_value_list};

const VERSION: i32 = 704;

// Versions referenced by the version-gated branches of the ITEM_STACK walker.
const V99_VERSION: i32 = 99;
const V705_VERSION: i32 = 705;
const V1458_VERSION: i32 = 1458;
const V1803_VERSION: i32 = 1803;
const V3438_VERSION: i32 = 3438;

/// Legacy tile-entity id -> namespaced id (V704.java:292-317). Used both by the
/// structure converter (rewrite `id`) and to drive `copyWalkers`.
const TILE_ID_UPDATE: &[(&str, &str)] = &[
    ("Airportal", "minecraft:end_portal"),
    ("Banner", "minecraft:banner"),
    ("Beacon", "minecraft:beacon"),
    ("Cauldron", "minecraft:brewing_stand"),
    ("Chest", "minecraft:chest"),
    ("Comparator", "minecraft:comparator"),
    ("Control", "minecraft:command_block"),
    ("DLDetector", "minecraft:daylight_detector"),
    ("Dropper", "minecraft:dropper"),
    ("EnchantTable", "minecraft:enchanting_table"),
    ("EndGateway", "minecraft:end_gateway"),
    ("EnderChest", "minecraft:ender_chest"),
    ("FlowerPot", "minecraft:flower_pot"),
    ("Furnace", "minecraft:furnace"),
    ("Hopper", "minecraft:hopper"),
    ("MobSpawner", "minecraft:mob_spawner"),
    ("Music", "minecraft:noteblock"),
    ("Piston", "minecraft:piston"),
    ("RecordPlayer", "minecraft:jukebox"),
    ("Sign", "minecraft:sign"),
    ("Skull", "minecraft:skull"),
    ("Structure", "minecraft:structure_block"),
    ("Trap", "minecraft:dispenser"),
];

/// `minecraft:<item id>` -> legacy block-entity id (V704.java:48-191). Injected
/// into a `BlockEntityTag` that lacks its own `id` so the TILE_ENTITY converter
/// can resolve it. Note: the resolved id here is already namespaced (this is the
/// post-V704 form), matching Java's map values.
const ITEM_ID_TO_TILE_ENTITY_ID: &[(&str, &str)] = &[
    ("minecraft:furnace", "minecraft:furnace"),
    ("minecraft:lit_furnace", "minecraft:furnace"),
    ("minecraft:chest", "minecraft:chest"),
    ("minecraft:trapped_chest", "minecraft:chest"),
    ("minecraft:ender_chest", "minecraft:ender_chest"),
    ("minecraft:jukebox", "minecraft:jukebox"),
    ("minecraft:dispenser", "minecraft:dispenser"),
    ("minecraft:dropper", "minecraft:dropper"),
    ("minecraft:sign", "minecraft:sign"),
    ("minecraft:mob_spawner", "minecraft:mob_spawner"),
    ("minecraft:spawner", "minecraft:mob_spawner"),
    ("minecraft:noteblock", "minecraft:noteblock"),
    ("minecraft:brewing_stand", "minecraft:brewing_stand"),
    ("minecraft:enhanting_table", "minecraft:enchanting_table"),
    ("minecraft:command_block", "minecraft:command_block"),
    ("minecraft:beacon", "minecraft:beacon"),
    ("minecraft:skull", "minecraft:skull"),
    ("minecraft:daylight_detector", "minecraft:daylight_detector"),
    ("minecraft:hopper", "minecraft:hopper"),
    ("minecraft:banner", "minecraft:banner"),
    ("minecraft:flower_pot", "minecraft:flower_pot"),
    ("minecraft:repeating_command_block", "minecraft:command_block"),
    ("minecraft:chain_command_block", "minecraft:command_block"),
    ("minecraft:shulker_box", "minecraft:shulker_box"),
    ("minecraft:white_shulker_box", "minecraft:shulker_box"),
    ("minecraft:orange_shulker_box", "minecraft:shulker_box"),
    ("minecraft:magenta_shulker_box", "minecraft:shulker_box"),
    ("minecraft:light_blue_shulker_box", "minecraft:shulker_box"),
    ("minecraft:yellow_shulker_box", "minecraft:shulker_box"),
    ("minecraft:lime_shulker_box", "minecraft:shulker_box"),
    ("minecraft:pink_shulker_box", "minecraft:shulker_box"),
    ("minecraft:gray_shulker_box", "minecraft:shulker_box"),
    ("minecraft:silver_shulker_box", "minecraft:shulker_box"),
    ("minecraft:cyan_shulker_box", "minecraft:shulker_box"),
    ("minecraft:purple_shulker_box", "minecraft:shulker_box"),
    ("minecraft:blue_shulker_box", "minecraft:shulker_box"),
    ("minecraft:brown_shulker_box", "minecraft:shulker_box"),
    ("minecraft:green_shulker_box", "minecraft:shulker_box"),
    ("minecraft:red_shulker_box", "minecraft:shulker_box"),
    ("minecraft:black_shulker_box", "minecraft:shulker_box"),
    ("minecraft:bed", "minecraft:bed"),
    ("minecraft:light_gray_shulker_box", "minecraft:shulker_box"),
    ("minecraft:white_banner", "minecraft:banner"),
    ("minecraft:orange_banner", "minecraft:banner"),
    ("minecraft:magenta_banner", "minecraft:banner"),
    ("minecraft:light_blue_banner", "minecraft:banner"),
    ("minecraft:yellow_banner", "minecraft:banner"),
    ("minecraft:lime_banner", "minecraft:banner"),
    ("minecraft:pink_banner", "minecraft:banner"),
    ("minecraft:gray_banner", "minecraft:banner"),
    ("minecraft:silver_banner", "minecraft:banner"),
    ("minecraft:cyan_banner", "minecraft:banner"),
    ("minecraft:purple_banner", "minecraft:banner"),
    ("minecraft:blue_banner", "minecraft:banner"),
    ("minecraft:brown_banner", "minecraft:banner"),
    ("minecraft:green_banner", "minecraft:banner"),
    ("minecraft:red_banner", "minecraft:banner"),
    ("minecraft:black_banner", "minecraft:banner"),
    ("minecraft:standing_sign", "minecraft:sign"),
    ("minecraft:wall_sign", "minecraft:sign"),
    ("minecraft:piston_head", "minecraft:piston"),
    ("minecraft:daylight_detector_inverted", "minecraft:daylight_detector"),
    ("minecraft:unpowered_comparator", "minecraft:comparator"),
    ("minecraft:powered_comparator", "minecraft:comparator"),
    ("minecraft:wall_banner", "minecraft:banner"),
    ("minecraft:standing_banner", "minecraft:banner"),
    ("minecraft:structure_block", "minecraft:structure_block"),
    ("minecraft:end_portal", "minecraft:end_portal"),
    ("minecraft:end_gateway", "minecraft:end_gateway"),
    ("minecraft:shield", "minecraft:banner"),
    ("minecraft:white_bed", "minecraft:bed"),
    ("minecraft:orange_bed", "minecraft:bed"),
    ("minecraft:magenta_bed", "minecraft:bed"),
    ("minecraft:light_blue_bed", "minecraft:bed"),
    ("minecraft:yellow_bed", "minecraft:bed"),
    ("minecraft:lime_bed", "minecraft:bed"),
    ("minecraft:pink_bed", "minecraft:bed"),
    ("minecraft:gray_bed", "minecraft:bed"),
    ("minecraft:silver_bed", "minecraft:bed"),
    ("minecraft:cyan_bed", "minecraft:bed"),
    ("minecraft:purple_bed", "minecraft:bed"),
    ("minecraft:blue_bed", "minecraft:bed"),
    ("minecraft:brown_bed", "minecraft:bed"),
    ("minecraft:green_bed", "minecraft:bed"),
    ("minecraft:red_bed", "minecraft:bed"),
    ("minecraft:black_bed", "minecraft:bed"),
    ("minecraft:oak_sign", "minecraft:sign"),
    ("minecraft:spruce_sign", "minecraft:sign"),
    ("minecraft:birch_sign", "minecraft:sign"),
    ("minecraft:jungle_sign", "minecraft:sign"),
    ("minecraft:acacia_sign", "minecraft:sign"),
    ("minecraft:dark_oak_sign", "minecraft:sign"),
    ("minecraft:crimson_sign", "minecraft:sign"),
    ("minecraft:warped_sign", "minecraft:sign"),
    ("minecraft:skeleton_skull", "minecraft:skull"),
    ("minecraft:wither_skeleton_skull", "minecraft:skull"),
    ("minecraft:zombie_head", "minecraft:skull"),
    ("minecraft:player_head", "minecraft:skull"),
    ("minecraft:creeper_head", "minecraft:skull"),
    ("minecraft:dragon_head", "minecraft:skull"),
    ("minecraft:barrel", "minecraft:barrel"),
    ("minecraft:conduit", "minecraft:conduit"),
    ("minecraft:smoker", "minecraft:smoker"),
    ("minecraft:blast_furnace", "minecraft:blast_furnace"),
    ("minecraft:lectern", "minecraft:lectern"),
    ("minecraft:bell", "minecraft:bell"),
    ("minecraft:jigsaw", "minecraft:jigsaw"),
    ("minecraft:campfire", "minecraft:campfire"),
    ("minecraft:bee_nest", "minecraft:beehive"),
    ("minecraft:beehive", "minecraft:beehive"),
    ("minecraft:sculk_sensor", "minecraft:sculk_sensor"),
    ("minecraft:decorated_pot", "minecraft:decorated_pot"),
    ("minecraft:crafter", "minecraft:crafter"),
    // missing from Vanilla up to 1.20.5
    ("minecraft:enchanting_table", "minecraft:enchanting_table"),
    ("minecraft:comparator", "minecraft:comparator"),
    ("minecraft:light_gray_bed", "minecraft:bed"),
    ("minecraft:light_gray_banner", "minecraft:banner"),
    ("minecraft:soul_campfire", "minecraft:campfire"),
    ("minecraft:sculk_catalyst", "minecraft:sculk_catalyst"),
    ("minecraft:mangrove_sign", "minecraft:sign"),
    ("minecraft:sculk_shrieker", "minecraft:sculk_shrieker"),
    ("minecraft:chiseled_bookshelf", "minecraft:chiseled_bookshelf"),
    ("minecraft:bamboo_sign", "minecraft:sign"),
    ("minecraft:oak_hanging_sign", "minecraft:sign"),
    ("minecraft:spruce_hanging_sign", "minecraft:sign"),
    ("minecraft:birch_hanging_sign", "minecraft:sign"),
    ("minecraft:jungle_hanging_sign", "minecraft:sign"),
    ("minecraft:acacia_hanging_sign", "minecraft:sign"),
    ("minecraft:dark_oak_hanging_sign", "minecraft:sign"),
    ("minecraft:mangrove_hanging_sign", "minecraft:sign"),
    ("minecraft:bamboo_hanging_sign", "minecraft:sign"),
    ("minecraft:crimson_hanging_sign", "minecraft:sign"),
    ("minecraft:warped_hanging_sign", "minecraft:sign"),
    ("minecraft:piglin_head", "minecraft:skull"),
    ("minecraft:suspicious_sand", "minecraft:brushable_block"),
    ("minecraft:cherry_sign", "minecraft:sign"),
    ("minecraft:cherry_hanging_sign", "minecraft:sign"),
    ("minecraft:suspicious_gravel", "minecraft:brushable_block"),
    ("minecraft:calibrated_sculk_sensor", "minecraft:calibrated_sculk_sensor"),
    ("minecraft:trial_spawner", "minecraft:trial_spawner"),
    ("minecraft:vault", "minecraft:vault"),
];

/// `minecraft:<item id>` -> entity id, keyed by version floor (V704.java:252-290).
/// Each entry lists `(version, id)` pairs in ascending version order; resolution
/// picks the value whose version is the greatest `<= fromVersion` (`getFloor`).
const ITEM_ID_TO_ENTITY_ID: &[(&str, &[(i32, &str)])] = &[
    ("minecraft:armor_stand", &[(V99_VERSION, "ArmorStand"), (V705_VERSION, "minecraft:armor_stand")]),
    ("minecraft:painting", &[(V99_VERSION, "Painting"), (V705_VERSION, "minecraft:painting")]),
    ("minecraft:boat", &[(V99_VERSION, "Boat"), (V705_VERSION, "minecraft:boat")]),
    ("minecraft:oak_boat", &[(V705_VERSION, "minecraft:boat")]),
    ("minecraft:oak_chest_boat", &[(V705_VERSION, "minecraft:chest_boat")]),
    ("minecraft:spruce_boat", &[(V705_VERSION, "minecraft:boat")]),
    ("minecraft:spruce_chest_boat", &[(V705_VERSION, "minecraft:chest_boat")]),
    ("minecraft:birch_boat", &[(V705_VERSION, "minecraft:boat")]),
    ("minecraft:birch_chest_boat", &[(V705_VERSION, "minecraft:chest_boat")]),
    ("minecraft:jungle_boat", &[(V705_VERSION, "minecraft:boat")]),
    ("minecraft:jungle_chest_boat", &[(V705_VERSION, "minecraft:chest_boat")]),
    ("minecraft:acacia_boat", &[(V705_VERSION, "minecraft:boat")]),
    ("minecraft:acacia_chest_boat", &[(V705_VERSION, "minecraft:chest_boat")]),
    ("minecraft:cherry_boat", &[(V705_VERSION, "minecraft:boat")]),
    ("minecraft:cherry_chest_boat", &[(V705_VERSION, "minecraft:chest_boat")]),
    ("minecraft:dark_oak_boat", &[(V705_VERSION, "minecraft:boat")]),
    ("minecraft:dark_oak_chest_boat", &[(V705_VERSION, "minecraft:chest_boat")]),
    ("minecraft:mangrove_boat", &[(V705_VERSION, "minecraft:boat")]),
    ("minecraft:mangrove_chest_boat", &[(V705_VERSION, "minecraft:chest_boat")]),
    ("minecraft:bamboo_raft", &[(V705_VERSION, "minecraft:boat")]),
    ("minecraft:bamboo_chest_raft", &[(V705_VERSION, "minecraft:chest_boat")]),
    ("minecraft:minecart", &[(V99_VERSION, "MinecartRideable"), (V705_VERSION, "minecraft:minecart")]),
    ("minecraft:chest_minecart", &[(V99_VERSION, "MinecartChest"), (V705_VERSION, "minecraft:chest_minecart")]),
    ("minecraft:furnace_minecart", &[(V99_VERSION, "MinecartFurnace"), (V705_VERSION, "minecraft:furnace_minecart")]),
    ("minecraft:tnt_minecart", &[(V99_VERSION, "MinecartTNT"), (V705_VERSION, "minecraft:tnt_minecart")]),
    ("minecraft:hopper_minecart", &[(V99_VERSION, "MinecartHopper"), (V705_VERSION, "minecraft:hopper_minecart")]),
    ("minecraft:item_frame", &[(V99_VERSION, "ItemFrame"), (V705_VERSION, "minecraft:item_frame")]),
    ("minecraft:glow_item_frame", &[(V705_VERSION, "minecraft:glow_item_frame")]),
    // Mojang missed these
    ("minecraft:pufferfish_bucket", &[(V705_VERSION, "minecraft:pufferfish")]),
    ("minecraft:salmon_bucket", &[(V705_VERSION, "minecraft:salmon")]),
    ("minecraft:cod_bucket", &[(V705_VERSION, "minecraft:cod")]),
    ("minecraft:tropical_fish_bucket", &[(V705_VERSION, "minecraft:tropical_fish")]),
    ("minecraft:axolotl_bucket", &[(V705_VERSION, "minecraft:axolotl")]),
    ("minecraft:tadpole_bucket", &[(V705_VERSION, "minecraft:tadpole")]),
];

/// `getFloor(fromVersion)` over [`ITEM_ID_TO_ENTITY_ID`]: the value for the
/// greatest registered version `<= from_version` (the version is encoded with
/// step 0, matching Java's `encodeVersions(k, 0)`).
fn entity_id_floor(item_id: &str, from: EncodedVersion) -> Option<&'static str> {
    let pairs = ITEM_ID_TO_ENTITY_ID.iter().find(|(k, _)| *k == item_id).map(|(_, v)| *v)?;
    let mut best: Option<&'static str> = None;
    for (v, id) in pairs {
        if encode_versions(*v, 0) <= from {
            best = Some(id);
        }
    }
    best
}

pub fn register(reg: &mut RegistryBuilder) {
    // TILE_ENTITY structure converter: rewrite the legacy id -> namespaced id.
    reg.tile_entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };
            if let Some((_, new)) = TILE_ID_UPDATE.iter().find(|(old, _)| *old == id) {
                data.set_string("id", *new);
            }
        }),
    );

    // Reverse of the TILE_ENTITY id rewrite (V704.java:340-351): map the NEW
    // namespaced id back to its legacy id. `TILE_ID_UPDATE` is injective on its
    // target ids (no two legacy ids share a namespaced id), so this inverse is
    // exact and lossless. Matches the new id, per reverse-converter semantics.
    reg.tile_entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let id = match data.get_string("id") {
                Some(s) => s.to_string(),
                None => return,
            };
            if let Some((old, _)) = TILE_ID_UPDATE.iter().find(|(_, new)| *new == id) {
                data.set_string("id", *old);
            }
        }),
    );

    // copyWalkers: the V99 walkers were registered under the legacy ids; copy
    // them onto the new namespaced ids. (Java's "missing walker" diagnostics are
    // pure logging and intentionally skipped.)
    for (old_id, new_id) in TILE_ID_UPDATE {
        reg.tile_entity.copy_walkers(VERSION, 0, old_id, new_id);
    }

    // ITEM_STACK structure walker (replaces the V99 one).
    reg.item_stack.add_structure_walker(VERSION, 0, Arc::new(item_stack_walker));

    // Enforce namespace for tile-entity ids.
    reg.tile_entity.add_structure_hook(VERSION, 0, enforce_namespaced_id_hook("id"));
}

/// The V704 ITEM_STACK structure walker (V704.java:365-460).
fn item_stack_walker(reg: &Registry, data: &mut NbtMap, from: EncodedVersion, to: EncodedVersion) {
    convert_value(&reg.item_name, data, "id", from, to);

    if !data.has_key("tag") {
        return;
    }

    let item_id = data.get_string("id").map(|s| s.to_string());
    let item_id_ref = item_id.as_deref();

    let tag = match data.get_map_mut("tag") {
        Some(t) => t,
        None => return,
    };

    convert_list(reg, &reg.item_stack, tag, "Items", from, to);
    convert_list(reg, &reg.item_stack, tag, "ChargedProjectiles", from, to);

    if item_id_ref == Some("minecraft:written_book") {
        // pages/filtered_pages are TEXT_COMPONENT *only* for written books.
        convert_list(reg, &reg.text_component, tag, "pages", from, to);
        convert_list(reg, &reg.text_component, tag, "filtered_pages", from, to);
    }

    // Vanilla blindly marks display Name/Lore as TEXT_COMPONENT even though they
    // only become components after the versions below.
    if to >= encode_versions(V1458_VERSION, 0) {
        if let Some(display) = tag.get_map_mut("display") {
            // Name is TEXT_COMPONENT in V1458.
            super::super::walker::convert(reg, &reg.text_component, display, "Name", from, to);
            if to >= encode_versions(V1803_VERSION, 0) {
                // Lore is TEXT_COMPONENT in V1803.
                convert_list(reg, &reg.text_component, display, "Lore", from, to);
            }
        }
    }

    // EntityTag -> ENTITY, resolving the legacy/missing sub-entity id.
    if tag.has_key("EntityTag") {
        // Resolve the entity id first (immutable read), then inject + recurse.
        let resolved: Option<String> = if let Some(item_id) = item_id_ref {
            if item_id.contains("_spawn_egg") {
                // V1451 moved the sub-entity id into the item id (`<ns>:<id>_spawn_egg`),
                // but never wrote logic to set the sub-entity id, so we do it here.
                let idx = item_id.find("_spawn_egg").unwrap();
                Some(item_id[..idx].to_string())
            } else if let Some(mapped) = entity_id_floor(item_id, from) {
                Some(mapped.to_string())
            } else {
                // Fall back to the EntityTag's own id.
                tag.get_map("EntityTag")
                    .and_then(|et| et.get_string("id"))
                    .map(|s| s.to_string())
            }
        } else {
            tag.get_map("EntityTag")
                .and_then(|et| et.get_string("id"))
                .map(|s| s.to_string())
        };

        if let Some(entity_tag) = tag.get_map_mut("EntityTag") {
            if let Some(entity_id) = resolved {
                if entity_tag.get_string("id").is_none() {
                    entity_tag.set_string("id", entity_id);
                }
            }
            reg.entity.convert(reg, entity_tag, from, to);
        }
    }

    // BlockEntityTag -> TILE_ENTITY, with the suspicious_sand special case.
    if tag.has_key("BlockEntityTag") {
        let resolved: Option<&'static str> =
            if from < encode_versions(V3438_VERSION, 0) && item_id_ref == Some("minecraft:suspicious_sand") {
                // Renamed after this version; the map value is just a string so
                // we special-case it.
                Some("minecraft:suspicious_sand")
            } else {
                item_id_ref.and_then(|id| {
                    ITEM_ID_TO_TILE_ENTITY_ID.iter().find(|(k, _)| *k == id).map(|(_, v)| *v)
                })
            };

        if let Some(block_entity_tag) = tag.get_map_mut("BlockEntityTag") {
            if let Some(entity_id) = resolved {
                if block_entity_tag.get_string("id").is_none() {
                    block_entity_tag.set_string("id", entity_id);
                }
            }
            reg.tile_entity.convert(reg, block_entity_tag, from, to);
        }
    }

    convert_value_list(&reg.block_name, tag, "CanDestroy", from, to);
    convert_value_list(&reg.block_name, tag, "CanPlaceOn", from, to);
}
