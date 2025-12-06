#[cfg(test)]
mod tests {
    use nucleation::BlockState;
    use nucleation::UniversalSchematic;

    #[test]
    fn test_fluent_api_rust() {
        let mut schematic = UniversalSchematic::new("FluentTest".to_string());

        // Setup some blocks
        schematic.set_block_str(0, 0, 0, "minecraft:stone");
        schematic.set_block_str(1, 0, 0, "minecraft:dirt");
        schematic.set_block_str(2, 0, 0, "minecraft:stone");

        // Create region with chaining
        schematic
            .create_region("test_region".to_string(), (0, 0, 0), (2, 0, 0))
            .add_bounds((0, 1, 0), (2, 1, 0))
            .set_color(0xFF0000)
            .with_metadata("type", "test");

        // Verify region properties
        let region = schematic.definition_regions.get("test_region").unwrap();
        assert_eq!(region.metadata.get("color").unwrap(), "#ff0000");
        assert_eq!(region.metadata.get("type").unwrap(), "test");
        assert_eq!(region.box_count(), 2);
    }

    #[test]
    fn test_exclude_block_rust() {
        let mut schematic = UniversalSchematic::new("ExcludeTest".to_string());
        schematic.set_block_str(0, 0, 0, "minecraft:stone");

        // Create region first
        schematic.create_region("r1".to_string(), (0, 0, 0), (0, 0, 0));

        // In Rust, we cannot borrow `schematic` mutably (to get the region)
        // and immutably (to read blocks) at the same time.
        // So we must detach the region, modify it, and re-insert it,
        // OR clone the schematic (expensive),
        // OR use a different API pattern.

        // For the purpose of this test, we verify the method logic works on a detached region.
        let mut region = schematic.definition_regions.get("r1").unwrap().clone();
        region.exclude_block(&schematic, "minecraft:stone");

        assert!(region.is_empty());
    }
}
