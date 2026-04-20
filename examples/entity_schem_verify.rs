//! Generates a .schem with our fixed export path so you can load it in
//! WorldEdit (//schem load) and confirm entities survive.
//!
//! Run with: `cargo run --example entity_schem_verify`

use nucleation::{schematic, BlockState, Entity, UniversalSchematic};

fn main() {
    let mut s = UniversalSchematic::new("entity_schem_verify".to_string());
    s.set_block(
        0,
        0,
        0,
        &BlockState::new("minecraft:diamond_block".to_string()),
    );
    s.set_block(2, 0, 0, &BlockState::new("minecraft:stone".to_string()));

    // Creeper standing on the diamond block.
    s.add_entity(Entity::new(
        "minecraft:creeper".to_string(),
        (0.5, 1.0, 0.5),
    ));
    // Villager standing on the stone block.
    s.add_entity(Entity::new(
        "minecraft:villager".to_string(),
        (2.5, 1.0, 0.5),
    ));

    let bytes = schematic::to_schematic(&s).expect("export");
    std::fs::write("entity_schem_verify.schem", &bytes).expect("write");
    println!("wrote entity_schem_verify.schem ({} bytes)", bytes.len());

    let reloaded = schematic::from_schematic(&bytes).expect("reload");
    let count: usize = reloaded
        .get_all_regions()
        .iter()
        .map(|(_, r)| r.entities.len())
        .sum();
    println!("reloaded entity count: {}", count);

    for (_, r) in reloaded.get_all_regions() {
        for e in &r.entities {
            println!("  {} at {:?}", e.id, e.position);
        }
    }
}
