use nucleation::formats::manager::get_manager;
use nucleation::UniversalSchematic;
use std::fs;

#[test]
fn test_registry_listing() {
    let registry_arc = get_manager();
    let registry = registry_arc.lock().unwrap_or_else(|e| e.into_inner());
    let importers = registry.list_importers();
    let exporters = registry.list_exporters();

    assert!(importers.contains(&"litematic".to_string()));
    assert!(importers.contains(&"schematic".to_string()));
    assert!(exporters.contains(&"litematic".to_string()));
    assert!(exporters.contains(&"schematic".to_string()));
}

#[test]
fn test_detect_litematic() {
    let data = fs::read("tests/samples/sample.litematic").expect("Failed to read sample.litematic");
    let registry_arc = get_manager();
    let registry = registry_arc.lock().unwrap_or_else(|e| e.into_inner());
    let format = registry.detect_format(&data);
    assert_eq!(format, Some("litematic".to_string()));
}

#[test]
fn test_detect_schematic() {
    let data = fs::read("tests/samples/sample.schem").expect("Failed to read sample.schem");
    let registry_arc = get_manager();
    let registry = registry_arc.lock().unwrap_or_else(|e| e.into_inner());
    let format = registry.detect_format(&data);
    assert_eq!(format, Some("schematic".to_string()));
}

#[test]
fn test_read_litematic_via_registry() {
    let data = fs::read("tests/samples/sample.litematic").expect("Failed to read sample.litematic");
    let registry_arc = get_manager();
    let registry = registry_arc.lock().unwrap_or_else(|e| e.into_inner());
    let schematic = registry.read(&data).expect("Failed to read litematic");
    assert!(schematic.total_blocks() > 0);
}

#[test]
fn test_read_schematic_via_registry() {
    let data = fs::read("tests/samples/sample.schem").expect("Failed to read sample.schem");
    let registry_arc = get_manager();
    let registry = registry_arc.lock().unwrap_or_else(|e| e.into_inner());
    let schematic = registry.read(&data).expect("Failed to read schematic");
    assert!(schematic.total_blocks() > 0);
}

#[test]
fn test_round_trip_registry() {
    // Read schem
    let data =
        fs::read("tests/samples/cutecounter.schem").expect("Failed to read cutecounter.schem");
    let registry_arc = get_manager();
    let registry = registry_arc.lock().unwrap_or_else(|e| e.into_inner());
    let schematic = registry.read(&data).expect("Failed to read schematic");

    // Write as litematic
    let litematic_data = registry
        .write("litematic", &schematic, None)
        .expect("Failed to write litematic");

    // Read back as litematic
    let schematic2 = registry
        .read(&litematic_data)
        .expect("Failed to read back litematic");

    // Write as schem (V3 default)
    let schem_data = registry
        .write("schematic", &schematic2, None)
        .expect("Failed to write schematic");
    let schematic3 = registry
        .read(&schem_data)
        .expect("Failed to read back schematic");

    println!("Original blocks: {}", schematic.total_blocks());
    println!("Litematic roundtrip blocks: {}", schematic2.total_blocks());
    println!("Schem roundtrip blocks: {}", schematic3.total_blocks());

    // Try V2
    let schem_v2_data = registry
        .write("schematic", &schematic2, Some("v2"))
        .expect("Failed to write schematic v2");
    let schematic4 = registry
        .read(&schem_v2_data)
        .expect("Failed to read back schematic v2");
    println!("Schem V2 roundtrip blocks: {}", schematic4.total_blocks());

    assert_eq!(schematic.total_blocks(), schematic2.total_blocks());

    // Assert full roundtrip fidelity
    assert_eq!(
        schematic.total_blocks(),
        schematic3.total_blocks(),
        "Schematic V3 export failed (block count mismatch)"
    );

    // Check V2 as well (already worked, but good to keep)
    if schematic4.total_blocks() > 0 {
        assert_eq!(
            schematic.total_blocks(),
            schematic4.total_blocks(),
            "Schematic V2 export failed"
        );
    }
}

#[test]
fn test_version_support() {
    let registry_arc = get_manager();
    let registry = registry_arc.lock().unwrap_or_else(|e| e.into_inner());

    let schematic = UniversalSchematic::new("test".to_string());

    let v2_data = registry.write("schematic", &schematic, Some("v2"));
    assert!(v2_data.is_ok());

    let v3_data = registry.write("schematic", &schematic, Some("v3"));
    assert!(v3_data.is_ok());

    let bad_version = registry.write("schematic", &schematic, Some("v99"));
    assert!(bad_version.is_err());
}
