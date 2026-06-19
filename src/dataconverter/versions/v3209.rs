//! V3209 (22w45a+1) — ITEM_STACK: ConverterFlattenSpawnEgg for minecraft:pig_spawn_egg,
//! re-resolving the item id from tag.EntityTag.id via an entity-id->egg-id map
//! (default minecraft:pig_spawn_egg). Cites V3209.java + ConverterFlattenSpawnEgg.java.
//! Nothing else is in this Java file.
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use crate::nbt::NbtMap;

const VERSION: i32 = 3209;

fn entity_id_to_new_egg_id(id: &str) -> &'static str {
    match id {
        "minecraft:bat" => "minecraft:bat_spawn_egg",
        "minecraft:blaze" => "minecraft:blaze_spawn_egg",
        "minecraft:cave_spider" => "minecraft:cave_spider_spawn_egg",
        "minecraft:chicken" => "minecraft:chicken_spawn_egg",
        "minecraft:cow" => "minecraft:cow_spawn_egg",
        "minecraft:creeper" => "minecraft:creeper_spawn_egg",
        "minecraft:donkey" => "minecraft:donkey_spawn_egg",
        "minecraft:elder_guardian" => "minecraft:elder_guardian_spawn_egg",
        "minecraft:ender_dragon" => "minecraft:ender_dragon_spawn_egg",
        "minecraft:enderman" => "minecraft:enderman_spawn_egg",
        "minecraft:endermite" => "minecraft:endermite_spawn_egg",
        "minecraft:evocation_illager" => "minecraft:evocation_illager_spawn_egg",
        "minecraft:ghast" => "minecraft:ghast_spawn_egg",
        "minecraft:guardian" => "minecraft:guardian_spawn_egg",
        "minecraft:horse" => "minecraft:horse_spawn_egg",
        "minecraft:husk" => "minecraft:husk_spawn_egg",
        "minecraft:iron_golem" => "minecraft:iron_golem_spawn_egg",
        "minecraft:llama" => "minecraft:llama_spawn_egg",
        "minecraft:magma_cube" => "minecraft:magma_cube_spawn_egg",
        "minecraft:mooshroom" => "minecraft:mooshroom_spawn_egg",
        "minecraft:mule" => "minecraft:mule_spawn_egg",
        "minecraft:ocelot" => "minecraft:ocelot_spawn_egg",
        "minecraft:pufferfish" => "minecraft:pufferfish_spawn_egg",
        "minecraft:parrot" => "minecraft:parrot_spawn_egg",
        "minecraft:pig" => "minecraft:pig_spawn_egg",
        "minecraft:polar_bear" => "minecraft:polar_bear_spawn_egg",
        "minecraft:rabbit" => "minecraft:rabbit_spawn_egg",
        "minecraft:sheep" => "minecraft:sheep_spawn_egg",
        "minecraft:shulker" => "minecraft:shulker_spawn_egg",
        "minecraft:silverfish" => "minecraft:silverfish_spawn_egg",
        "minecraft:skeleton" => "minecraft:skeleton_spawn_egg",
        "minecraft:skeleton_horse" => "minecraft:skeleton_horse_spawn_egg",
        "minecraft:slime" => "minecraft:slime_spawn_egg",
        "minecraft:snow_golem" => "minecraft:snow_golem_spawn_egg",
        "minecraft:spider" => "minecraft:spider_spawn_egg",
        "minecraft:squid" => "minecraft:squid_spawn_egg",
        "minecraft:stray" => "minecraft:stray_spawn_egg",
        "minecraft:turtle" => "minecraft:turtle_spawn_egg",
        "minecraft:vex" => "minecraft:vex_spawn_egg",
        "minecraft:villager" => "minecraft:villager_spawn_egg",
        "minecraft:vindication_illager" => "minecraft:vindication_illager_spawn_egg",
        "minecraft:witch" => "minecraft:witch_spawn_egg",
        "minecraft:wither" => "minecraft:wither_spawn_egg",
        "minecraft:wither_skeleton" => "minecraft:wither_skeleton_spawn_egg",
        "minecraft:wolf" => "minecraft:wolf_spawn_egg",
        "minecraft:zombie" => "minecraft:zombie_spawn_egg",
        "minecraft:zombie_horse" => "minecraft:zombie_horse_spawn_egg",
        "minecraft:zombie_pigman" => "minecraft:zombie_pigman_spawn_egg",
        "minecraft:zombie_villager" => "minecraft:zombie_villager_spawn_egg",
        _ => "minecraft:pig_spawn_egg",
    }
}

fn new_egg_id_to_entity_id(id: &str) -> Option<&'static str> {
    match id {
        "minecraft:bat_spawn_egg" => Some("minecraft:bat"),
        "minecraft:blaze_spawn_egg" => Some("minecraft:blaze"),
        "minecraft:cave_spider_spawn_egg" => Some("minecraft:cave_spider"),
        "minecraft:chicken_spawn_egg" => Some("minecraft:chicken"),
        "minecraft:cow_spawn_egg" => Some("minecraft:cow"),
        "minecraft:creeper_spawn_egg" => Some("minecraft:creeper"),
        "minecraft:donkey_spawn_egg" => Some("minecraft:donkey"),
        "minecraft:elder_guardian_spawn_egg" => Some("minecraft:elder_guardian"),
        "minecraft:ender_dragon_spawn_egg" => Some("minecraft:ender_dragon"),
        "minecraft:enderman_spawn_egg" => Some("minecraft:enderman"),
        "minecraft:endermite_spawn_egg" => Some("minecraft:endermite"),
        "minecraft:evocation_illager_spawn_egg" => Some("minecraft:evocation_illager"),
        "minecraft:ghast_spawn_egg" => Some("minecraft:ghast"),
        "minecraft:guardian_spawn_egg" => Some("minecraft:guardian"),
        "minecraft:horse_spawn_egg" => Some("minecraft:horse"),
        "minecraft:husk_spawn_egg" => Some("minecraft:husk"),
        "minecraft:iron_golem_spawn_egg" => Some("minecraft:iron_golem"),
        "minecraft:llama_spawn_egg" => Some("minecraft:llama"),
        "minecraft:magma_cube_spawn_egg" => Some("minecraft:magma_cube"),
        "minecraft:mooshroom_spawn_egg" => Some("minecraft:mooshroom"),
        "minecraft:mule_spawn_egg" => Some("minecraft:mule"),
        "minecraft:ocelot_spawn_egg" => Some("minecraft:ocelot"),
        "minecraft:pufferfish_spawn_egg" => Some("minecraft:pufferfish"),
        "minecraft:parrot_spawn_egg" => Some("minecraft:parrot"),
        "minecraft:pig_spawn_egg" => Some("minecraft:pig"),
        "minecraft:polar_bear_spawn_egg" => Some("minecraft:polar_bear"),
        "minecraft:rabbit_spawn_egg" => Some("minecraft:rabbit"),
        "minecraft:sheep_spawn_egg" => Some("minecraft:sheep"),
        "minecraft:shulker_spawn_egg" => Some("minecraft:shulker"),
        "minecraft:silverfish_spawn_egg" => Some("minecraft:silverfish"),
        "minecraft:skeleton_spawn_egg" => Some("minecraft:skeleton"),
        "minecraft:skeleton_horse_spawn_egg" => Some("minecraft:skeleton_horse"),
        "minecraft:slime_spawn_egg" => Some("minecraft:slime"),
        "minecraft:snow_golem_spawn_egg" => Some("minecraft:snow_golem"),
        "minecraft:spider_spawn_egg" => Some("minecraft:spider"),
        "minecraft:squid_spawn_egg" => Some("minecraft:squid"),
        "minecraft:stray_spawn_egg" => Some("minecraft:stray"),
        "minecraft:turtle_spawn_egg" => Some("minecraft:turtle"),
        "minecraft:vex_spawn_egg" => Some("minecraft:vex"),
        "minecraft:villager_spawn_egg" => Some("minecraft:villager"),
        "minecraft:vindication_illager_spawn_egg" => Some("minecraft:vindication_illager"),
        "minecraft:witch_spawn_egg" => Some("minecraft:witch"),
        "minecraft:wither_spawn_egg" => Some("minecraft:wither"),
        "minecraft:wither_skeleton_spawn_egg" => Some("minecraft:wither_skeleton"),
        "minecraft:wolf_spawn_egg" => Some("minecraft:wolf"),
        "minecraft:zombie_spawn_egg" => Some("minecraft:zombie"),
        "minecraft:zombie_horse_spawn_egg" => Some("minecraft:zombie_horse"),
        "minecraft:zombie_pigman_spawn_egg" => Some("minecraft:zombie_pigman"),
        "minecraft:zombie_villager_spawn_egg" => Some("minecraft:zombie_villager"),
        _ => None,
    }
}

fn typed_spawn_egg_entity_id(id: &str) -> Option<String> {
    if let Some(entity) = new_egg_id_to_entity_id(id) {
        return Some(entity.to_string());
    }
    id.strip_prefix("minecraft:")
        .and_then(|s| s.strip_suffix("_spawn_egg"))
        .filter(|s| !s.is_empty())
        .map(|s| format!("minecraft:{s}"))
}

fn reverse_typed_spawn_egg(data: &mut NbtMap, report_unknown: bool) {
    let Some(item_id) = data.get_string("id").map(str::to_string) else {
        return;
    };
    let Some(entity_id) = typed_spawn_egg_entity_id(&item_id) else {
        return;
    };

    data.set_string("id", "minecraft:pig_spawn_egg");
    if data.get_map("tag").is_none() {
        data.set_map("tag", NbtMap::new());
    }
    let tag = data.get_map_mut("tag").expect("just inserted");
    if tag.get_map("EntityTag").is_none() {
        tag.set_map("EntityTag", NbtMap::new());
    }
    let entity_tag = tag.get_map_mut("EntityTag").expect("just inserted");
    if let Some(existing) = entity_tag.get_string("id").map(str::to_string) {
        if existing != entity_id {
            report_loss(
                VERSION,
                LossKind::FingerprintCollapse,
                Severity::Approximated,
                format!(
                    "typed spawn egg id {item_id} conflicts with tag.EntityTag.id {existing}; using item id discriminator"
                ),
            );
        }
    }
    entity_tag.set_string("id", entity_id.clone());

    if report_unknown {
        report_loss(
            VERSION,
            LossKind::UnsupportedInTarget,
            Severity::Loss,
            format!("{item_id} did not exist in the v3209 spawn-egg table; downgraded to pig_spawn_egg with EntityTag.id={entity_id}"),
        );
    }
}

/// All distinct flattened spawn-egg ids the forward converter can produce (the values of
/// `ENTITY_ID_TO_NEW_EGG_ID` in ConverterFlattenSpawnEgg.java, incl. the `minecraft:pig_spawn_egg`
/// default). The reverse collapses every one of these back to the single legacy id.
const NEW_EGG_IDS: &[&str] = &[
    "minecraft:bat_spawn_egg",
    "minecraft:blaze_spawn_egg",
    "minecraft:cave_spider_spawn_egg",
    "minecraft:chicken_spawn_egg",
    "minecraft:cow_spawn_egg",
    "minecraft:creeper_spawn_egg",
    "minecraft:donkey_spawn_egg",
    "minecraft:elder_guardian_spawn_egg",
    "minecraft:ender_dragon_spawn_egg",
    "minecraft:enderman_spawn_egg",
    "minecraft:endermite_spawn_egg",
    "minecraft:evocation_illager_spawn_egg",
    "minecraft:ghast_spawn_egg",
    "minecraft:guardian_spawn_egg",
    "minecraft:horse_spawn_egg",
    "minecraft:husk_spawn_egg",
    "minecraft:iron_golem_spawn_egg",
    "minecraft:llama_spawn_egg",
    "minecraft:magma_cube_spawn_egg",
    "minecraft:mooshroom_spawn_egg",
    "minecraft:mule_spawn_egg",
    "minecraft:ocelot_spawn_egg",
    "minecraft:pufferfish_spawn_egg",
    "minecraft:parrot_spawn_egg",
    "minecraft:pig_spawn_egg",
    "minecraft:polar_bear_spawn_egg",
    "minecraft:rabbit_spawn_egg",
    "minecraft:sheep_spawn_egg",
    "minecraft:shulker_spawn_egg",
    "minecraft:silverfish_spawn_egg",
    "minecraft:skeleton_spawn_egg",
    "minecraft:skeleton_horse_spawn_egg",
    "minecraft:slime_spawn_egg",
    "minecraft:snow_golem_spawn_egg",
    "minecraft:spider_spawn_egg",
    "minecraft:squid_spawn_egg",
    "minecraft:stray_spawn_egg",
    "minecraft:turtle_spawn_egg",
    "minecraft:vex_spawn_egg",
    "minecraft:villager_spawn_egg",
    "minecraft:vindication_illager_spawn_egg",
    "minecraft:witch_spawn_egg",
    "minecraft:wither_spawn_egg",
    "minecraft:wither_skeleton_spawn_egg",
    "minecraft:wolf_spawn_egg",
    "minecraft:zombie_spawn_egg",
    "minecraft:zombie_horse_spawn_egg",
    "minecraft:zombie_pigman_spawn_egg",
    "minecraft:zombie_villager_spawn_egg",
];

pub fn register(reg: &mut RegistryBuilder) {
    // Note: This converter reads entity id from its sub data, but we need no breakpoint because
    // entity ids are not remapped this version.
    reg.item_stack.add_converter_for_id(
        "minecraft:pig_spawn_egg",
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            // tag.EntityTag.id -> new egg id (default minecraft:pig_spawn_egg)
            let id = data
                .get_map("tag")
                .and_then(|tag| tag.get_map("EntityTag"))
                .and_then(|entity_tag| entity_tag.get_string("id"))
                .map(|s| s.to_string());
            if let Some(id) = id {
                data.set_string("id", entity_id_to_new_egg_id(&id));
            }
        }),
    );

    // Fallback for typed spawn eggs introduced after V3209. They can still be
    // represented by the legacy placeholder item plus EntityTag.id, but the
    // target game may not know that entity, so report the downgrade.
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            let known = data
                .get_string("id")
                .and_then(new_egg_id_to_entity_id)
                .is_some();
            if !known {
                reverse_typed_spawn_egg(data, true);
            }
        }),
    );

    // Reverse (new -> old): the typed egg id itself encodes the legacy
    // EntityTag discriminator. Restore both the single legacy placeholder item
    // id and tag.EntityTag.id, matching the NEW ids (the forward's output ids).
    for &new_id in NEW_EGG_IDS {
        reg.item_stack.add_reverse_converter_for_id(
            new_id,
            VERSION,
            0,
            Box::new(|data: &mut NbtMap, _from, _to| {
                reverse_typed_spawn_egg(data, false);
            }),
        );
    }
}
