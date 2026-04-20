//! Generates two .litematic files for manual verification of the entity
//! bbox fix. Load each in Minecraft with Litematica; the reported entity
//! counts should match and you should see both entities in-game.
//!
//! Run with: `cargo run --example entity_bbox_test`
//! Output:   ./entity_outside_positive.litematic
//!           ./entity_outside_negative.litematic

use nucleation::{litematic, BlockState, Entity, UniversalSchematic};
use std::path::Path;

fn build_and_save(path: &str, description: &str, build: impl FnOnce(&mut UniversalSchematic)) {
    let mut schem = UniversalSchematic::new(description.to_string());
    build(&mut schem);

    let content = schem
        .get_content_bounds()
        .map(|b| format!("{:?}", b))
        .unwrap_or_else(|| "<empty>".into());
    let entity_count: usize = schem
        .get_all_regions()
        .iter()
        .map(|(_, r)| r.entities.len())
        .sum();

    println!("-- {}", description);
    println!("   content_bounds: {}", content);
    println!("   entity count:   {}", entity_count);

    let bytes = litematic::to_litematic(&schem).expect("export failed");
    std::fs::write(Path::new(path), bytes).expect("write failed");
    println!("   wrote: {}", path);

    // Round-trip sanity check
    let reloaded_bytes = std::fs::read(path).unwrap();
    let reloaded = litematic::from_litematic(&reloaded_bytes).expect("reload failed");
    let reloaded_count: usize = reloaded
        .get_all_regions()
        .iter()
        .map(|(_, r)| r.entities.len())
        .sum();
    println!("   reloaded count: {}\n", reloaded_count);
}

fn main() {
    // Case 1: entity far from block origin in positive coords.
    // Before fix: creeper was silently dropped on Litematica load.
    // After fix:  creeper appears at (50.5, 40.0, 30.5) when pasted.
    build_and_save(
        "entity_outside_positive.litematic",
        "entity_outside_positive",
        |s| {
            s.set_block(
                0,
                0,
                0,
                &BlockState::new("minecraft:diamond_block".to_string()),
            );
            s.add_entity(Entity::new(
                "minecraft:creeper".to_string(),
                (50.5, 40.0, 30.5),
            ));
        },
    );

    // Case 2: entity at negative coords + another inside block bounds.
    // Exercises min-side expansion and confirms multiple entities survive.
    build_and_save(
        "entity_outside_negative.litematic",
        "entity_outside_negative",
        |s| {
            s.set_block(
                0,
                0,
                0,
                &BlockState::new("minecraft:gold_block".to_string()),
            );
            s.add_entity(Entity::new(
                "minecraft:zombie".to_string(),
                (-20.5, -5.0, -15.5),
            ));
            s.add_entity(Entity::new(
                "minecraft:villager".to_string(),
                (0.5, 1.0, 0.5),
            ));
        },
    );

    println!("Paste each schematic in Litematica; you should see the entities.");
    println!("If any entity is missing, the bug is not fully fixed.");
}
