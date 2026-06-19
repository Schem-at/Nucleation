//! V2501 (1.16 snapshot) — furnace `RecipesUsed` migration; cites `V2501.java`.
//!
//! VERSION = MCVersions.V1_15_2 + 271 = 2230 + 271 = 2501.
//!
//! Ported:
//!   * TILE_ENTITY `addConverterForId` for `minecraft:furnace`,
//!     `minecraft:blast_furnace`, `minecraft:smoker` (V2501.java:27-59): read and
//!     remove the legacy `RecipesUsedSize` count; if it is `<= 0`, stop. Otherwise
//!     create a fresh `RecipesUsed` compound and, for each index `i`, read+remove
//!     `RecipeLocation<i>` and `RecipeAmount<i>`, writing `RecipesUsed[<location>]
//!     = <amount>` (as an Int) — skipping `i <= 0` and null locations, exactly
//!     matching Java's `continue` guard. (Java sets the empty `RecipesUsed` map
//!     before the loop; this is observable, so we replicate it.)
//!   * TEXT_COMPONENT walker part of `registerFurnace` (V2501.java:14-24,61-63) for
//!     `minecraft:furnace`, `minecraft:smoker`, `minecraft:blast_furnace`: walk
//!     `Items` (ITEM_STACK) and `CustomName` (TEXT_COMPONENT).
//!
//! Skipped (non-schematic): the `convertKeys(RECIPE, "RecipesUsed", …)` call inside
//! `registerFurnace` (V2501.java:20) — RECIPE is not a schematic data type.

use std::sync::Arc;

use crate::nbt::NbtMap;

use super::super::engine::Converter;
use super::super::registry::RegistryBuilder;
use super::super::types::MapExt;
use super::super::walker::{convert, convert_list};

const VERSION: i32 = 2501;

/// The shared `addConverterForId` body (V2501.java:27-55), rebuilt fresh per id
/// (Java reuses one instance, but `Converter` is not `Clone`).
fn recipes_used_converter() -> Converter {
    Box::new(move |data, _from, _to| {
        let recipes_used_size = data.get_i32("RecipesUsedSize").unwrap_or(0);
        data.take("RecipesUsedSize");

        if recipes_used_size <= 0 {
            return;
        }

        let mut new_recipes = NbtMap::new();

        for i in 0..recipes_used_size {
            let location_key = format!("RecipeLocation{}", i);
            let amount_key = format!("RecipeAmount{}", i);

            let recipe_key = data.get_string(&location_key).map(str::to_string);
            data.take(&location_key);
            let recipe_amount = data.get_i32(&amount_key).unwrap_or(0);
            data.take(&amount_key);

            if i <= 0 {
                continue;
            }
            let Some(recipe_key) = recipe_key else {
                continue;
            };

            new_recipes.set_i32(&recipe_key, recipe_amount);
        }

        data.set_map("RecipesUsed", new_recipes);
    })
}

/// Inverse of `recipes_used_converter` (V2501.java:27-55): expand a `RecipesUsed`
/// compound `{location: amount}` back into the legacy flat keys
/// `RecipeLocation<i>`/`RecipeAmount<i>` plus the `RecipesUsedSize` count.
///
/// The forward always skips index 0 (`if (i <= 0) continue;`) and null
/// locations, so a real downgrade only ever sees the surviving `{location:
/// amount}` pairs. We re-emit them starting at index 1 and set
/// `RecipesUsedSize = count + 1` (reserving the always-empty slot 0). Running
/// the forward over this output reproduces the exact same `RecipesUsed` map
/// (slot 0 has no `RecipeLocation0`, so it is skipped) — so this round-trips.
/// Original index numbering / the dropped slot-0 entry are not present in the
/// modern schema, so there is no reportable loss (rule 11).
fn recipes_used_reverse_converter() -> Converter {
    Box::new(move |data, _from, _to| {
        let Some(recipes_used) = data.take("RecipesUsed") else {
            return;
        };
        let crate::nbt::NbtValue::Compound(recipes_used) = recipes_used else {
            return;
        };

        let mut i: i32 = 1;
        for (location, amount) in recipes_used.iter() {
            let recipe_amount = match amount {
                crate::nbt::NbtValue::Int(v) => *v,
                crate::nbt::NbtValue::Byte(v) => *v as i32,
                crate::nbt::NbtValue::Short(v) => *v as i32,
                crate::nbt::NbtValue::Long(v) => *v as i32,
                _ => 0,
            };
            data.set_string(&format!("RecipeLocation{}", i), location);
            data.set_i32(&format!("RecipeAmount{}", i), recipe_amount);
            i += 1;
        }

        // Slot 0 is always skipped by the forward converter, so the count must
        // reserve it: highest written index is `i - 1`, count = `i`.
        data.set_i32("RecipesUsedSize", i);
    })
}

/// `registerFurnace(id)` (V2501.java:14-24): walk `Items` (ITEM_STACK) and
/// `CustomName` (TEXT_COMPONENT). The `convertKeys(RECIPE, "RecipesUsed")` call is
/// skipped (RECIPE is non-schematic).
fn register_furnace(reg: &mut RegistryBuilder, id: &'static str) {
    reg.tile_entity.add_walker(
        VERSION,
        0,
        id,
        Arc::new(|reg, data, from, to| {
            convert_list(reg, &reg.item_stack, data, "Items", from, to);
            convert(reg, &reg.text_component, data, "CustomName", from, to);
        }),
    );
}

pub fn register(reg: &mut RegistryBuilder) {
    // TILE_ENTITY: migrate the legacy RecipeLocation<i>/RecipeAmount<i> flat keys
    // into a single RecipesUsed compound (V2501.java:27-59).
    reg.tile_entity
        .add_converter_for_id("minecraft:furnace", VERSION, 0, recipes_used_converter());
    reg.tile_entity
        .add_converter_for_id("minecraft:blast_furnace", VERSION, 0, recipes_used_converter());
    reg.tile_entity
        .add_converter_for_id("minecraft:smoker", VERSION, 0, recipes_used_converter());

    // Reverse: expand `RecipesUsed` back into the legacy flat
    // `RecipeLocation<i>`/`RecipeAmount<i>` keys + `RecipesUsedSize`. The
    // forward output id is the same as the input id (no rename), so we match on
    // the same ids. Lossless for surviving data — see
    // `recipes_used_reverse_converter`.
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:furnace",
        VERSION,
        0,
        recipes_used_reverse_converter(),
    );
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:blast_furnace",
        VERSION,
        0,
        recipes_used_reverse_converter(),
    );
    reg.tile_entity.add_reverse_converter_for_id(
        "minecraft:smoker",
        VERSION,
        0,
        recipes_used_reverse_converter(),
    );

    // TILE_ENTITY walkers (V2501.java:61-63).
    register_furnace(reg, "minecraft:furnace");
    register_furnace(reg, "minecraft:smoker");
    register_furnace(reg, "minecraft:blast_furnace");
}
