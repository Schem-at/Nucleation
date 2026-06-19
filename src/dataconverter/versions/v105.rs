//! V105 (V15W32C + 1) — legacy `minecraft:spawn_egg` ITEM_STACK converter
//! (V105.java:15-46).
//!
//! For a legacy spawn egg: read the short `Damage`, zero it if non-zero, and
//! when `tag.EntityTag` has no String `id`, resolve the legacy entity name from
//! the damage value (`HelperSpawnEggNameV105.getSpawnNameFromId`, mirrored by
//! `flattening::get_spawn_name_from_id`) and write it back through tag/EntityTag.
//! Nothing else from V105.java is in scope.

use crate::nbt::NbtMap;

use super::super::flattening::get_spawn_name_from_id;
use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;

const VERSION: i32 = 105;

pub fn register(reg: &mut RegistryBuilder) {
    reg.item_stack.add_converter_for_id(
        "minecraft:spawn_egg",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            // Java getShort("Damage") defaults to 0 when absent.
            let damage = data.get_i32("Damage").unwrap_or(0) as i16;
            if damage != 0 {
                data.set_short("Damage", 0);
            }

            // Only act when EntityTag lacks a String `id`. Java builds empty
            // tag/EntityTag maps but writes them back only if a name resolves,
            // so a missing/unresolvable damage leaves the item untouched.
            let has_id = data
                .get_map("tag")
                .and_then(|tag| tag.get_map("EntityTag"))
                .and_then(|et| et.get_string("id"))
                .is_some();
            if has_id {
                return;
            }

            let Some(converted) = get_spawn_name_from_id(damage) else {
                return;
            };

            // tag.getOrCreate -> EntityTag.getOrCreate -> set id; then write back.
            if data.get_map("tag").is_none() {
                data.set_map("tag", NbtMap::new());
            }
            let tag = data.get_map_mut("tag").unwrap();
            if tag.get_map("EntityTag").is_none() {
                tag.set_map("EntityTag", NbtMap::new());
            }
            let entity_tag = tag.get_map_mut("EntityTag").unwrap();
            entity_tag.set_string("id", converted);
        }),
    );

    // Reverse of the spawn-egg converter (new -> old). The forward zeroed the
    // legacy short `Damage` (which encoded the egg's entity type) and, when
    // `tag.EntityTag` lacked a String `id`, stamped the entity name resolved
    // from that damage. Going backwards we read `tag.EntityTag.id` and restore
    // the legacy `Damage` discriminator via the inverse of
    // `HelperSpawnEggNameV105.ID_TO_STRING` (each name appears exactly once in
    // that table, so name -> damage is injective => lossless for the round
    // trip). We do NOT strip `EntityTag.id`: the legacy format could legitimately
    // carry it (the forward only *added* it when absent), and removing it could
    // drop user data; restoring `Damage` is the meaningful legacy discriminator.
    //
    // Lossy edge (no report needed, nothing to act on): if the original legacy
    // `Damage` was a value outside the table, the forward zeroed it and wrote no
    // `id`, so there is no surviving signal here to reconstruct it; we leave the
    // item untouched.
    reg.item_stack.add_reverse_converter_for_id(
        "minecraft:spawn_egg",
        VERSION,
        0,
        Box::new(|data, _from, _to| {
            let Some(name) = data
                .get_map("tag")
                .and_then(|tag| tag.get_map("EntityTag"))
                .and_then(|et| et.get_string("id"))
            else {
                return;
            };

            // Inverse of SPAWN_EGG_NAME_BY_ID (flattening/data.rs:2835) /
            // HelperSpawnEggNameV105.ID_TO_STRING: entity name -> legacy damage.
            let damage: i16 = match name {
                "Item" => 1,
                "XPOrb" => 2,
                "ThrownEgg" => 7,
                "LeashKnot" => 8,
                "Painting" => 9,
                "Arrow" => 10,
                "Snowball" => 11,
                "Fireball" => 12,
                "SmallFireball" => 13,
                "ThrownEnderpearl" => 14,
                "EyeOfEnderSignal" => 15,
                "ThrownPotion" => 16,
                "ThrownExpBottle" => 17,
                "ItemFrame" => 18,
                "WitherSkull" => 19,
                "PrimedTnt" => 20,
                "FallingSand" => 21,
                "FireworksRocketEntity" => 22,
                "TippedArrow" => 23,
                "SpectralArrow" => 24,
                "ShulkerBullet" => 25,
                "DragonFireball" => 26,
                "ArmorStand" => 30,
                "Boat" => 41,
                "MinecartRideable" => 42,
                "MinecartChest" => 43,
                "MinecartFurnace" => 44,
                "MinecartTNT" => 45,
                "MinecartHopper" => 46,
                "MinecartSpawner" => 47,
                "MinecartCommandBlock" => 40,
                "Creeper" => 50,
                "Skeleton" => 51,
                "Spider" => 52,
                "Giant" => 53,
                "Zombie" => 54,
                "Slime" => 55,
                "Ghast" => 56,
                "PigZombie" => 57,
                "Enderman" => 58,
                "CaveSpider" => 59,
                "Silverfish" => 60,
                "Blaze" => 61,
                "LavaSlime" => 62,
                "EnderDragon" => 63,
                "WitherBoss" => 64,
                "Bat" => 65,
                "Witch" => 66,
                "Endermite" => 67,
                "Guardian" => 68,
                "Shulker" => 69,
                "Pig" => 90,
                "Sheep" => 91,
                "Cow" => 92,
                "Chicken" => 93,
                "Squid" => 94,
                "Wolf" => 95,
                "MushroomCow" => 96,
                "SnowMan" => 97,
                "Ozelot" => 98,
                "VillagerGolem" => 99,
                "EntityHorse" => 100,
                "Rabbit" => 101,
                "Villager" => 120,
                "EnderCrystal" => 200,
                // Name not in the legacy table (e.g. a modern entity id):
                // there is no legacy damage encoding for it, so leave Damage
                // alone rather than fabricate one, but do not silently ignore
                // the unsupported downgrade.
                _ => {
                    report_loss(
                        VERSION,
                        LossKind::UnsupportedInTarget,
                        Severity::Loss,
                        format!(
                            "spawn_egg EntityTag.id '{name}' has no pre-V105 legacy Damage encoding"
                        ),
                    );
                    return;
                }
            };

            data.set_short("Damage", damage);
        }),
    );
}
