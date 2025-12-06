use nucleation::definition_region::DefinitionRegion;
use nucleation::UniversalSchematic;

fn main() {
    let mut schematic = UniversalSchematic::new("FluentRust".to_string());

    // Setup some blocks
    schematic.set_block_str(0, 0, 0, "minecraft:stone");
    schematic.set_block_str(1, 0, 0, "minecraft:dirt");
    schematic.set_block_str(2, 0, 0, "minecraft:stone");

    // 1. Basic Fluent API (No dependency on schematic content)
    // This works perfectly because we only borrow the region mutably (via the schematic),
    // and we don't need to borrow the schematic again for these methods.
    schematic
        .create_region("layout".to_string(), (0, 0, 0), (2, 0, 0))
        .add_bounds((0, 1, 0), (2, 1, 0))
        .set_color(0xFF0000)
        .with_metadata("description", "Main Layout");

    println!("Created region 'layout'");

    // 2. Advanced Fluent API (Filtering based on schematic content)
    // In Rust, we cannot do:
    // schematic.get_definition_region_mut("layout").filter_by_block(&schematic, "stone");
    // Because `get_definition_region_mut` borrows `schematic` mutably,
    // and `filter_by_block` needs to borrow `schematic` immutably.

    // Pattern A: Clone the region, modify, and update (Safe, slightly slower)
    if let Some(region) = schematic.definition_regions.get("layout") {
        let mut region_clone = region.clone();

        // Now we can pass &schematic because region_clone is separate
        region_clone.filter_by_block(&schematic, "minecraft:stone");

        // Update the schematic
        schematic
            .definition_regions
            .insert("stone_only".to_string(), region_clone);
    }

    // Pattern B: Build region separately before inserting
    let mut new_region = DefinitionRegion::from_bounds((0, 0, 0), (5, 5, 5));
    new_region.exclude_block(&schematic, "minecraft:air");
    schematic
        .definition_regions
        .insert("non_air".to_string(), new_region);

    println!("Created filtered regions 'stone_only' and 'non_air'");
}
