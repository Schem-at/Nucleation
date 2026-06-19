//! V99 (the pre-converter / legacy registrar) — schematic-relevant subset.
//!
//! This is the template every other version file follows: register only the
//! schematic-relevant types (STRUCTURE, BLOCK_STATE, BLOCK_NAME, TILE_ENTITY,
//! ITEM_STACK, ITEM_NAME, ENTITY, ENTITY_NAME, DATA_COMPONENTS, ENTITY_EQUIPMENT,
//! TEXT_COMPONENT, UNTAGGED_SPAWNER, PARTICLE). Non-schematic registrations from
//! V99.java (LEVEL/CHUNK/PLAYER/SAVED_DATA*/TEAM/OBJECTIVE/VILLAGER_TRADE) are
//! intentionally omitted — they never appear in a schematic file.

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::super::engine::Walker;
use super::super::helpers::{enforce_namespaced_id_hook, enforce_namespaced_value_hook};
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::version::EncodedVersion;
use super::super::walker::{
    block_names, convert, convert_list, convert_list_path, convert_value, convert_value_list,
    item_lists, item_names, items, tile_entities,
};

const VERSION: i32 = 99;

/// `minecraft:<item id>` -> legacy block-entity id, for resolving the id of a
/// `BlockEntityTag` carried inside an item (V99.java:39-69). Trimmed to the
/// container-bearing items most likely to appear in a schematic; extend as
/// needed when porting the full legacy item set.
const ITEM_ID_TO_TILE_ENTITY_ID: &[(&str, &str)] = &[
    ("minecraft:furnace", "Furnace"),
    ("minecraft:lit_furnace", "Furnace"),
    ("minecraft:chest", "Chest"),
    ("minecraft:trapped_chest", "Chest"),
    ("minecraft:ender_chest", "EnderChest"),
    ("minecraft:dispenser", "Trap"),
    ("minecraft:dropper", "Dropper"),
    ("minecraft:hopper", "Hopper"),
    ("minecraft:brewing_stand", "Cauldron"),
    ("minecraft:beacon", "Beacon"),
    ("minecraft:jukebox", "RecordPlayer"),
    ("minecraft:mob_spawner", "MobSpawner"),
    ("minecraft:flower_pot", "FlowerPot"),
    ("minecraft:standing_banner", "Banner"),
    ("minecraft:wall_banner", "Banner"),
    ("minecraft:noteblock", "Music"),
    ("minecraft:command_block", "Control"),
    ("minecraft:repeating_command_block", "Control"),
    ("minecraft:chain_command_block", "Control"),
    ("minecraft:skull", "Skull"),
    ("minecraft:standing_sign", "Sign"),
    ("minecraft:wall_sign", "Sign"),
];

fn register_inventory(reg: &mut RegistryBuilder, id: &str) {
    reg.tile_entity.add_walker(VERSION, 0, id, item_lists(&["Items"]));
}

fn register_projectile(reg: &mut RegistryBuilder, id: &str) {
    reg.entity.add_walker(VERSION, 0, id, block_names(&["inTile"]));
}

/// Sign text walker (V99.java:80-95). Shared by later sign id renames.
pub fn sign_walker() -> Walker {
    Arc::new(|reg, data, from, to| {
        for p in [
            "Text1", "Text2", "Text3", "Text4", "FilteredText1", "FilteredText2", "FilteredText3",
            "FilteredText4",
        ] {
            convert(reg, &reg.text_component, data, p, from, to);
        }
    })
}

pub fn register(reg: &mut RegistryBuilder) {
    // --- entities -----------------------------------------------------------
    reg.entity.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.entity, data, "Riding", from, to);
            reg.entity_equipment.convert(reg, data, from, to);
        }),
    );
    reg.entity_equipment.add_structure_walker(VERSION, 0, item_lists(&["Equipment"]));

    reg.entity.add_walker(VERSION, 0, "Item", items(&["Item"]));
    register_projectile(reg, "ThrownEgg");
    reg.entity.add_walker(VERSION, 0, "Arrow", block_names(&["inTile"]));
    reg.entity.add_walker(VERSION, 0, "TippedArrow", block_names(&["inTile"]));
    reg.entity.add_walker(VERSION, 0, "SpectralArrow", block_names(&["inTile"]));
    register_projectile(reg, "Snowball");
    register_projectile(reg, "Fireball");
    register_projectile(reg, "SmallFireball");
    register_projectile(reg, "ThrownEnderpearl");
    reg.entity.add_walker(VERSION, 0, "ThrownPotion", block_names(&["inTile"]));
    reg.entity.add_walker(VERSION, 0, "ThrownPotion", items(&["Potion"]));
    register_projectile(reg, "ThrownExpBottle");
    reg.entity.add_walker(VERSION, 0, "ItemFrame", items(&["Item"]));
    register_projectile(reg, "WitherSkull");
    reg.entity.add_walker(VERSION, 0, "FallingSand", block_names(&["Block"]));
    reg.entity.add_walker(VERSION, 0, "FallingSand", tile_entities(&["TileEntityData"]));
    reg.entity.add_walker(VERSION, 0, "FireworksRocketEntity", items(&["FireworksItem"]));

    // Minecart family (generic + specific subtypes).
    let spawner_walker: Walker = Arc::new(|reg, data, from, to| {
        reg.untagged_spawner.convert(reg, data, from, to);
    });
    reg.entity.add_walker(VERSION, 0, "Minecart", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "Minecart", item_lists(&["Items"]));
    reg.entity.add_walker(VERSION, 0, "Minecart", spawner_walker.clone());
    reg.entity.add_walker(VERSION, 0, "MinecartRideable", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "MinecartChest", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "MinecartChest", item_lists(&["Items"]));
    reg.entity.add_walker(VERSION, 0, "MinecartFurnace", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "MinecartTNT", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "MinecartSpawner", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "MinecartSpawner", spawner_walker);
    reg.entity.add_walker(VERSION, 0, "MinecartHopper", block_names(&["DisplayTile"]));
    reg.entity.add_walker(VERSION, 0, "MinecartHopper", item_lists(&["Items"]));
    reg.entity.add_walker(VERSION, 0, "MinecartCommandBlock", block_names(&["DisplayTile"]));
    reg.entity.add_walker(
        VERSION,
        0,
        "MinecartCommandBlock",
        Arc::new(|reg, data, from, to| convert(reg, &reg.text_component, data, "LastOutput", from, to)),
    );

    reg.entity.add_walker(VERSION, 0, "Enderman", block_names(&["carried"]));
    reg.entity.add_walker(VERSION, 0, "EntityHorse", item_lists(&["Items"]));
    reg.entity.add_walker(VERSION, 0, "EntityHorse", items(&["ArmorItem", "SaddleItem"]));
    reg.entity.add_walker(VERSION, 0, "Villager", item_lists(&["Inventory"]));
    reg.entity.add_walker(
        VERSION,
        0,
        "Villager",
        Arc::new(|reg, data, from, to| {
            if let Some(offers) = data.get_map_mut("Offers") {
                convert_list(reg, &reg.villager_trade, offers, "Recipes", from, to);
            }
        }),
    );
    reg.entity.add_walker(
        VERSION,
        0,
        "AreaEffectCloud",
        Arc::new(|reg, data, from, to| convert(reg, &reg.particle, data, "Particle", from, to)),
    );

    // --- tile entities ------------------------------------------------------
    reg.tile_entity.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, data, from, to| {
            convert(reg, &reg.data_components, data, "components", from, to);
        }),
    );

    register_inventory(reg, "Furnace");
    register_inventory(reg, "Chest");
    reg.tile_entity.add_walker(VERSION, 0, "RecordPlayer", items(&["RecordItem"]));
    register_inventory(reg, "Trap");
    register_inventory(reg, "Dropper");
    reg.tile_entity.add_walker(VERSION, 0, "Sign", sign_walker());
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "MobSpawner",
        Arc::new(|reg, data, from, to| reg.untagged_spawner.convert(reg, data, from, to)),
    );
    register_inventory(reg, "Cauldron");
    reg.tile_entity.add_walker(
        VERSION,
        0,
        "Control",
        Arc::new(|reg, data, from, to| convert(reg, &reg.text_component, data, "LastOutput", from, to)),
    );
    register_inventory(reg, "Hopper");
    reg.tile_entity.add_walker(VERSION, 0, "FlowerPot", item_names(&["Item"]));

    // --- item stacks --------------------------------------------------------
    reg.item_stack.add_structure_walker(VERSION, 0, Arc::new(item_stack_walker));

    // --- structure (schematic root) ----------------------------------------
    reg.structure.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, root, from, to| {
            convert_list_path(reg, &reg.entity, root, "entities", "nbt", from, to);
            convert_list_path(reg, &reg.tile_entity, root, "blocks", "nbt", from, to);
            convert_list(reg, &reg.block_state, root, "palette", from, to);
        }),
    );

    // --- villager trades (Offers.Recipes elements) -------------------------
    reg.villager_trade.add_structure_walker(
        VERSION,
        0,
        Arc::new(|reg, root, from, to| {
            convert(reg, &reg.item_stack, root, "buy", from, to);
            convert(reg, &reg.item_stack, root, "buyB", from, to);
            convert(reg, &reg.item_stack, root, "sell", from, to);
        }),
    );

    // --- id namespace enforcement ------------------------------------------
    reg.block_name.add_structure_hook(VERSION, 0, enforce_namespaced_value_hook());
    reg.item_name.add_structure_hook(VERSION, 0, enforce_namespaced_value_hook());
    reg.item_stack.add_structure_hook(VERSION, 0, enforce_namespaced_id_hook("id"));
}

/// The ITEM_STACK structure walker (V99.java:224-317): recurse `id`→ITEM_NAME and
/// everything nested in `tag` (contained items, written-book pages, EntityTag,
/// BlockEntityTag, CanDestroy/CanPlaceOn).
fn item_stack_walker(reg: &super::super::registry::Registry, data: &mut NbtMap, from: EncodedVersion, to: EncodedVersion) {
    convert_value(&reg.item_name, data, "id", from, to);

    let item_id = data.get_string("id").map(|s| s.to_string());
    let tag = match data.get_map_mut("tag") {
        Some(t) => t,
        None => return,
    };

    convert_list(reg, &reg.item_stack, tag, "Items", from, to);
    convert_list(reg, &reg.item_stack, tag, "ChargedProjectiles", from, to);
    if item_id.as_deref() == Some("minecraft:written_book") {
        // pages are a LIST of TEXT_COMPONENTs (compounds in modern data).
        convert_list(reg, &reg.text_component, tag, "pages", from, to);
        convert_list(reg, &reg.text_component, tag, "filtered_pages", from, to);
    }

    // EntityTag -> ENTITY, with best-effort legacy id resolution.
    if let Some(entity_tag) = tag.get_map_mut("EntityTag") {
        let resolved = match item_id.as_deref() {
            Some("minecraft:armor_stand") => Some("ArmorStand"),
            Some("minecraft:item_frame") => Some("ItemFrame"),
            Some("minecraft:painting") => Some("Painting"),
            _ => None,
        };
        let mut remove_id = false;
        if let Some(id) = resolved {
            if !entity_tag.has_key("id") {
                remove_id = true;
                entity_tag.set_string("id", id);
            }
        }
        reg.entity.convert(reg, entity_tag, from, to);
        if remove_id {
            entity_tag.take("id");
        }
    }

    // BlockEntityTag -> TILE_ENTITY, injecting the resolved legacy id.
    if let Some(block_entity_tag) = tag.get_map_mut("BlockEntityTag") {
        let resolved = item_id
            .as_deref()
            .and_then(|id| ITEM_ID_TO_TILE_ENTITY_ID.iter().find(|(k, _)| *k == id).map(|(_, v)| *v));
        let mut remove_id = false;
        if let Some(id) = resolved {
            remove_id = !block_entity_tag.has_key("id");
            block_entity_tag.set_string("id", id);
        }
        reg.tile_entity.convert(reg, block_entity_tag, from, to);
        if remove_id {
            block_entity_tag.take("id");
        }
    }

    convert_value_list(&reg.block_name, tag, "CanDestroy", from, to);
    convert_value_list(&reg.block_name, tag, "CanPlaceOn", from, to);
}
