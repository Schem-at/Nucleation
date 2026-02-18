#![cfg(any(feature = "scripting-lua", feature = "scripting-js"))]

use nucleation::scripting::ScriptingSchematic;

#[test]
fn test_shared_new() {
    let s = ScriptingSchematic::new(Some("TestSchematic".to_string()));
    assert_eq!(s.get_name(), "TestSchematic");
}

#[test]
fn test_shared_new_default_name() {
    let s = ScriptingSchematic::new(None);
    assert_eq!(s.get_name(), "Unnamed");
}

#[test]
fn test_shared_set_get_block() {
    let mut s = ScriptingSchematic::new(None);
    s.set_block(0, 0, 0, "minecraft:stone");
    let block = s.get_block(0, 0, 0);
    assert!(block.is_some());
    assert!(block.unwrap().contains("stone"));
}

#[test]
fn test_shared_fill_cuboid() {
    let mut s = ScriptingSchematic::new(None);
    s.fill_cuboid((0, 0, 0), (2, 2, 2), "minecraft:stone");
    // Check that an interior block is set
    let block = s.get_block(1, 1, 1);
    assert!(block.is_some());
    assert!(block.unwrap().contains("stone"));
}

#[test]
fn test_shared_metadata() {
    let mut s = ScriptingSchematic::new(None);
    s.set_name("My Schematic");
    s.set_author("Tester");
    s.set_description("A test schematic");
    assert_eq!(s.get_name(), "My Schematic");
    assert_eq!(s.get_author(), "Tester");
    assert_eq!(s.get_description(), "A test schematic");
}

#[test]
fn test_shared_export() {
    let mut s = ScriptingSchematic::new(Some("ExportTest".to_string()));
    s.set_block(0, 0, 0, "minecraft:stone");
    let bytes = s.to_schematic().expect("should export to schematic");
    assert!(!bytes.is_empty());
}

#[test]
fn test_shared_dimensions() {
    let mut s = ScriptingSchematic::new(None);
    s.set_block(0, 0, 0, "minecraft:stone");
    s.set_block(3, 4, 5, "minecraft:dirt");
    let (w, h, d) = s.get_dimensions();
    assert!(w >= 4);
    assert!(h >= 5);
    assert!(d >= 6);
}

#[test]
fn test_shared_block_count() {
    let mut s = ScriptingSchematic::new(None);
    s.set_block(0, 0, 0, "minecraft:stone");
    s.set_block(1, 0, 0, "minecraft:dirt");
    assert!(s.get_block_count() >= 2);
}

#[test]
fn test_shared_transforms() {
    let mut s = ScriptingSchematic::new(None);
    s.set_block(0, 0, 0, "minecraft:stone");
    s.set_block(1, 0, 0, "minecraft:dirt");
    let count_before = s.get_block_count();
    s.flip_x();
    assert_eq!(s.get_block_count(), count_before);
}

#[test]
fn test_shared_get_all_blocks() {
    let mut s = ScriptingSchematic::new(None);
    s.set_block(0, 0, 0, "minecraft:stone");
    s.set_block(1, 0, 0, "minecraft:dirt");
    let blocks = s.get_all_blocks();
    assert!(blocks.len() >= 2);
}

#[test]
fn test_shared_region_names() {
    let s = ScriptingSchematic::new(None);
    let names = s.get_region_names();
    assert!(!names.is_empty());
}
