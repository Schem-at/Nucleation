//! V2523 (20w13b + 2) — schematic-relevant subset of `V2523.java`.
//!
//! Old attribute-name rename (`ConverterAbstractOldAttributesRename`): legacy
//! attribute ids (e.g. `generic.maxHealth`, `Max Health`) -> the modern
//! namespaced snake-case ids (e.g. `minecraft:generic.max_health`)
//! (V2523.java:13-37).
//!
//! The shared helper registers this on ENTITY (`Attributes[].Name`), PLAYER, and
//! ITEM_STACK (`AttributeModifiers[].AttributeName`). PLAYER never appears in a
//! schematic so it is skipped; ENTITY and ITEM_STACK are ported inline.
//!
//! VERSION = MCVersions.V20W13B (2521) + 2 = 2523.

use crate::nbt::NbtMap;

use super::super::loss::{report_loss, LossKind, Severity};
use super::super::registry::RegistryBuilder;
use super::super::types::{MapExt, ValueExt};

const VERSION: i32 = 2523;

const RENAMES: &[(&str, &str)] = &[
    ("generic.maxHealth", "minecraft:generic.max_health"),
    ("Max Health", "minecraft:generic.max_health"),
    (
        "zombie.spawnReinforcements",
        "minecraft:zombie.spawn_reinforcements",
    ),
    (
        "Spawn Reinforcements Chance",
        "minecraft:zombie.spawn_reinforcements",
    ),
    ("horse.jumpStrength", "minecraft:horse.jump_strength"),
    ("Jump Strength", "minecraft:horse.jump_strength"),
    ("generic.followRange", "minecraft:generic.follow_range"),
    ("Follow Range", "minecraft:generic.follow_range"),
    (
        "generic.knockbackResistance",
        "minecraft:generic.knockback_resistance",
    ),
    (
        "Knockback Resistance",
        "minecraft:generic.knockback_resistance",
    ),
    ("generic.movementSpeed", "minecraft:generic.movement_speed"),
    ("Movement Speed", "minecraft:generic.movement_speed"),
    ("generic.flyingSpeed", "minecraft:generic.flying_speed"),
    ("Flying Speed", "minecraft:generic.flying_speed"),
    ("generic.attackDamage", "minecraft:generic.attack_damage"),
    (
        "generic.attackKnockback",
        "minecraft:generic.attack_knockback",
    ),
    ("generic.attackSpeed", "minecraft:generic.attack_speed"),
    (
        "generic.armorToughness",
        "minecraft:generic.armor_toughness",
    ),
];

fn rename(name: &str) -> Option<&'static str> {
    RENAMES
        .iter()
        .find(|(old, _)| *old == name)
        .map(|(_, new)| *new)
}

/// Reverse rename table (modern -> old). The forward map is many-to-one: most
/// modern ids had TWO legacy preimages — the canonical `generic.*`/camelCase
/// NBT name and an even older human "Display Name" alternate (`Max Health`,
/// `Spawn Reinforcements Chance`, ...). On the way back we always restore the
/// canonical `generic.*`/camelCase form (the name that immediately preceded the
/// flattening); the `ambiguous` flag marks the 7 ids whose Display-Name
/// preimage is unrecoverable, so we emit an Approximated loss for them. The
/// remaining 4 (`attack_damage`, `attack_knockback`, `attack_speed`,
/// `armor_toughness`) had a unique preimage -> lossless.
const REVERSE_RENAMES: &[(&str, &str, bool)] = &[
    ("minecraft:generic.max_health", "generic.maxHealth", true),
    (
        "minecraft:zombie.spawn_reinforcements",
        "zombie.spawnReinforcements",
        true,
    ),
    ("minecraft:horse.jump_strength", "horse.jumpStrength", true),
    (
        "minecraft:generic.follow_range",
        "generic.followRange",
        true,
    ),
    (
        "minecraft:generic.knockback_resistance",
        "generic.knockbackResistance",
        true,
    ),
    (
        "minecraft:generic.movement_speed",
        "generic.movementSpeed",
        true,
    ),
    (
        "minecraft:generic.flying_speed",
        "generic.flyingSpeed",
        true,
    ),
    (
        "minecraft:generic.attack_damage",
        "generic.attackDamage",
        false,
    ),
    (
        "minecraft:generic.attack_knockback",
        "generic.attackKnockback",
        false,
    ),
    (
        "minecraft:generic.attack_speed",
        "generic.attackSpeed",
        false,
    ),
    (
        "minecraft:generic.armor_toughness",
        "generic.armorToughness",
        false,
    ),
];

fn rename_back(name: &str) -> Option<(&'static str, bool)> {
    REVERSE_RENAMES
        .iter()
        .find(|(new, _, _)| *new == name)
        .map(|(_, old, ambiguous)| (*old, *ambiguous))
}

/// Inverse of `rename_string`: restore the canonical legacy attribute id at
/// `key`, reporting an Approximated loss when the modern id had a collapsed
/// Display-Name alternate that can't be told apart.
fn rename_string_back(map: &mut NbtMap, key: &str) {
    if let Some(cur) = map.get_string(key) {
        if let Some((old, ambiguous)) = rename_back(cur) {
            if ambiguous {
                report_loss(
                    VERSION,
                    LossKind::RenameAmbiguous,
                    Severity::Approximated,
                    format!(
                        "attribute '{cur}' had two legacy names; restored canonical '{old}' (Display-Name alternate unrecoverable)"
                    ),
                );
            }
            map.set_string(key, old);
        }
    }
}

/// `RenameHelper.renameString(map, key, renamer)`: read the string at `key`,
/// rename it if the renamer returns a value.
fn rename_string(map: &mut NbtMap, key: &str) {
    if let Some(cur) = map.get_string(key) {
        if let Some(new) = rename(cur) {
            map.set_string(key, new);
        }
    }
}

pub fn register(reg: &mut RegistryBuilder) {
    // ENTITY: Attributes[].Name
    reg.entity.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(attributes) = data.get_list_mut("Attributes") {
                for el in attributes.iter_mut() {
                    if let Some(attr) = el.as_compound_mut() {
                        rename_string(attr, "Name");
                    }
                }
            }
        }),
    );

    // REVERSE — ENTITY: Attributes[].Name modern id -> legacy id.
    reg.entity.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(attributes) = data.get_list_mut("Attributes") {
                for el in attributes.iter_mut() {
                    if let Some(attr) = el.as_compound_mut() {
                        rename_string_back(attr, "Name");
                    }
                }
            }
        }),
    );

    // ITEM_STACK: AttributeModifiers[].AttributeName
    reg.item_stack.add_structure_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(attributes) = data.get_list_mut("AttributeModifiers") {
                for el in attributes.iter_mut() {
                    if let Some(attr) = el.as_compound_mut() {
                        rename_string(attr, "AttributeName");
                    }
                }
            }
        }),
    );

    // REVERSE — ITEM_STACK: AttributeModifiers[].AttributeName modern id -> legacy id.
    reg.item_stack.add_reverse_converter(
        VERSION,
        0,
        Box::new(|data: &mut NbtMap, _from, _to| {
            if let Some(attributes) = data.get_list_mut("AttributeModifiers") {
                for el in attributes.iter_mut() {
                    if let Some(attr) = el.as_compound_mut() {
                        rename_string_back(attr, "AttributeName");
                    }
                }
            }
        }),
    );
}
