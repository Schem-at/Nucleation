//! V705 (1.10.2 + 193) — legacy entity ids -> namespaced ids, plus the per-id
//! entity walkers registered under the new namespaced ids.
//!
//! Ported from
//! `DataConverterJava/.../minecraft/versions/V705.java`.
//!
//! Schematic-relevant registrations ported:
//!   * Entity id rename (`ConverterAbstractEntityRename`): legacy id (e.g. `Pig`)
//!     -> namespaced id (e.g. `minecraft:pig`), applied to the ENTITY `id` field
//!     and the ENTITY_NAME value type.
//!   * The per-id ENTITY walkers (block names, item lists/items, tile entities,
//!     particle, spawner, etc.) re-registered under the new namespaced ids.
//!   * ENTITY namespaced-id structure hook + ENTITY_NAME namespaced value hook.
//!
//! The `villager` walker recurses both its `Inventory` (ITEM_STACK) and
//! `Offers.Recipes` (VILLAGER_TRADE); the `zombie_villager` walker recurses only
//! `Offers.Recipes` (VILLAGER_TRADE). The commented-out `registerMob` lines in
//! Java are mobs that became "simple" (no nested typed data) and need no walker.

use std::sync::Arc;

use super::super::helpers::{
    enforce_namespaced_id_hook, enforce_namespaced_value_hook, map_renamer, register_entity_rename,
};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{block_names, convert, convert_list, item_lists, items, tile_entities};

const VERSION: i32 = 705;

/// Legacy entity id -> namespaced id (V705.java:22-99). Drives both the ENTITY
/// `id` rewrite and the ENTITY_NAME value rename.
const ENTITY_ID_UPDATE: &[(&str, &str)] = &[
    ("AreaEffectCloud", "minecraft:area_effect_cloud"),
    ("ArmorStand", "minecraft:armor_stand"),
    ("Arrow", "minecraft:arrow"),
    ("Bat", "minecraft:bat"),
    ("Blaze", "minecraft:blaze"),
    ("Boat", "minecraft:boat"),
    ("CaveSpider", "minecraft:cave_spider"),
    ("Chicken", "minecraft:chicken"),
    ("Cow", "minecraft:cow"),
    ("Creeper", "minecraft:creeper"),
    ("Donkey", "minecraft:donkey"),
    ("DragonFireball", "minecraft:dragon_fireball"),
    ("ElderGuardian", "minecraft:elder_guardian"),
    ("EnderCrystal", "minecraft:ender_crystal"),
    ("EnderDragon", "minecraft:ender_dragon"),
    ("Enderman", "minecraft:enderman"),
    ("Endermite", "minecraft:endermite"),
    ("EyeOfEnderSignal", "minecraft:eye_of_ender_signal"),
    ("FallingSand", "minecraft:falling_block"),
    ("Fireball", "minecraft:fireball"),
    ("FireworksRocketEntity", "minecraft:fireworks_rocket"),
    ("Ghast", "minecraft:ghast"),
    ("Giant", "minecraft:giant"),
    ("Guardian", "minecraft:guardian"),
    ("Horse", "minecraft:horse"),
    ("Husk", "minecraft:husk"),
    ("Item", "minecraft:item"),
    ("ItemFrame", "minecraft:item_frame"),
    ("LavaSlime", "minecraft:magma_cube"),
    ("LeashKnot", "minecraft:leash_knot"),
    ("MinecartChest", "minecraft:chest_minecart"),
    ("MinecartCommandBlock", "minecraft:commandblock_minecart"),
    ("MinecartFurnace", "minecraft:furnace_minecart"),
    ("MinecartHopper", "minecraft:hopper_minecart"),
    ("MinecartRideable", "minecraft:minecart"),
    ("MinecartSpawner", "minecraft:spawner_minecart"),
    ("MinecartTNT", "minecraft:tnt_minecart"),
    ("Mule", "minecraft:mule"),
    ("MushroomCow", "minecraft:mooshroom"),
    ("Ozelot", "minecraft:ocelot"),
    ("Painting", "minecraft:painting"),
    ("Pig", "minecraft:pig"),
    ("PigZombie", "minecraft:zombie_pigman"),
    ("PolarBear", "minecraft:polar_bear"),
    ("PrimedTnt", "minecraft:tnt"),
    ("Rabbit", "minecraft:rabbit"),
    ("Sheep", "minecraft:sheep"),
    ("Shulker", "minecraft:shulker"),
    ("ShulkerBullet", "minecraft:shulker_bullet"),
    ("Silverfish", "minecraft:silverfish"),
    ("Skeleton", "minecraft:skeleton"),
    ("SkeletonHorse", "minecraft:skeleton_horse"),
    ("Slime", "minecraft:slime"),
    ("SmallFireball", "minecraft:small_fireball"),
    ("SnowMan", "minecraft:snowman"),
    ("Snowball", "minecraft:snowball"),
    ("SpectralArrow", "minecraft:spectral_arrow"),
    ("Spider", "minecraft:spider"),
    ("Squid", "minecraft:squid"),
    ("Stray", "minecraft:stray"),
    ("ThrownEgg", "minecraft:egg"),
    ("ThrownEnderpearl", "minecraft:ender_pearl"),
    ("ThrownExpBottle", "minecraft:xp_bottle"),
    ("ThrownPotion", "minecraft:potion"),
    ("Villager", "minecraft:villager"),
    ("VillagerGolem", "minecraft:villager_golem"),
    ("Witch", "minecraft:witch"),
    ("WitherBoss", "minecraft:wither"),
    ("WitherSkeleton", "minecraft:wither_skeleton"),
    ("WitherSkull", "minecraft:wither_skull"),
    ("Wolf", "minecraft:wolf"),
    ("XPOrb", "minecraft:xp_orb"),
    ("Zombie", "minecraft:zombie"),
    ("ZombieHorse", "minecraft:zombie_horse"),
    ("ZombieVillager", "minecraft:zombie_villager"),
];

fn register_throwable_projectile(reg: &mut RegistryBuilder, id: &str) {
    reg.entity.add_walker(VERSION, 0, id, block_names(&["inTile"]));
}

pub fn register(reg: &mut RegistryBuilder) {
    register_entity_rename(reg, VERSION, map_renamer(ENTITY_ID_UPDATE));

    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:area_effect_cloud",
        Arc::new(|reg, data, from, to| convert(reg, &reg.particle, data, "Particle", from, to)),
    );
    reg.entity.add_walker(VERSION, 0, "minecraft:arrow", block_names(&["inTile"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:chest_minecart", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:chest_minecart", item_lists(&["Items"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:commandblock_minecart", block_names(&["DisplayTile"]));
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:commandblock_minecart",
        Arc::new(|reg, data, from, to| convert(reg, &reg.text_component, data, "LastOutput", from, to)),
    );
    reg.entity.add_walker(VERSION, 0, "minecraft:donkey", item_lists(&["Items"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:donkey", items(&["SaddleItem"]));
    register_throwable_projectile(reg, "minecraft:egg");
    reg.entity.add_walker(VERSION, 0, "minecraft:enderman", block_names(&["carried"]));
    register_throwable_projectile(reg, "minecraft:ender_pearl");
    reg.entity.add_walker(VERSION, 0, "minecraft:falling_block", block_names(&["Block"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:falling_block", tile_entities(&["TileEntityData"]));
    register_throwable_projectile(reg, "minecraft:fireball");
    reg.entity.add_walker(VERSION, 0, "minecraft:fireworks_rocket", items(&["FireworksItem"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:furnace_minecart", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:hopper_minecart", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:hopper_minecart", item_lists(&["Items"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:horse", items(&["ArmorItem", "SaddleItem"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:item", items(&["Item"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:item_frame", items(&["Item"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:minecart", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:mule", item_lists(&["Items"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:mule", items(&["SaddleItem"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:potion", items(&["Potion"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:potion", block_names(&["inTile"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:skeleton_horse", items(&["SaddleItem"]));
    register_throwable_projectile(reg, "minecraft:small_fireball");
    register_throwable_projectile(reg, "minecraft:snowball");
    reg.entity.add_walker(VERSION, 0, "minecraft:spawner_minecart", block_names(&["DisplayTile"]));
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:spawner_minecart",
        Arc::new(|reg, data, from, to| reg.untagged_spawner.convert(reg, data, from, to)),
    );
    reg.entity.add_walker(VERSION, 0, "minecraft:spectral_arrow", block_names(&["inTile"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:tnt_minecart", block_names(&["DisplayTile"]));
    // Villager: recurses Inventory (ITEM_STACK) and Offers.Recipes (VILLAGER_TRADE).
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:villager",
        Arc::new(|reg, data, from, to| {
            convert_list(reg, &reg.item_stack, data, "Inventory", from, to);
            if let Some(offers) = data.get_map_mut("Offers") {
                convert_list(reg, &reg.villager_trade, offers, "Recipes", from, to);
            }
        }),
    );
    register_throwable_projectile(reg, "minecraft:wither_skull");
    register_throwable_projectile(reg, "minecraft:xp_bottle");
    reg.entity.add_walker(VERSION, 0, "minecraft:zombie_horse", items(&["SaddleItem"]));
    // zombie_villager: recurses only Offers.Recipes (VILLAGER_TRADE).
    reg.entity.add_walker(
        VERSION,
        0,
        "minecraft:zombie_villager",
        Arc::new(|reg, data, from, to| {
            if let Some(offers) = data.get_map_mut("Offers") {
                convert_list(reg, &reg.villager_trade, offers, "Recipes", from, to);
            }
        }),
    );
    reg.entity.add_walker(VERSION, 0, "minecraft:llama", item_lists(&["Items"]));
    reg.entity.add_walker(VERSION, 0, "minecraft:llama", items(&["SaddleItem", "DecorItem"]));

    // Enforce namespace for ids.
    reg.entity.add_structure_hook(VERSION, 0, enforce_namespaced_id_hook("id"));
    reg.entity_name.add_structure_hook(VERSION, 0, enforce_namespaced_value_hook());
}
