use nucleation::formats::mcstructure::from_mcstructure;
use nucleation::formats::schematic::to_schematic;
use nucleation::nbt::NbtValue;
use std::io::Write;
use std::path::PathBuf;

#[test]
fn test_auto_register_conversion() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/samples/auto_register.mcstructure");

    if !path.exists() {
        // Skip if file not present (e.g. CI)
        return;
    }

    let data = std::fs::read(&path).expect("Failed to read file");
    let schematic = from_mcstructure(&data).expect("Failed to import mcstructure");
    let region = schematic.get_merged_region();
    let palette = region.get_palette();

    // 1. Check Comparators
    let comparators: Vec<_> = palette
        .iter()
        .filter(|b| b.name == "minecraft:comparator")
        .collect();

    assert!(
        !comparators.is_empty(),
        "Should have translated comparators"
    );

    for comp in comparators {
        assert!(
            comp.properties.contains_key("facing"),
            "Comparator missing facing: {:?}",
            comp
        );
        assert!(
            comp.properties.contains_key("mode"),
            "Comparator missing mode: {:?}",
            comp
        );
        assert!(
            comp.properties.contains_key("powered"),
            "Comparator missing powered: {:?}",
            comp
        );

        let mode = comp.properties.get("mode").unwrap();
        assert!(
            mode == "compare" || mode == "subtract",
            "Invalid mode: {}",
            mode
        );
    }

    // 2. Check Barrels (Items translation)
    let barrels_count = region
        .block_entities
        .iter()
        .filter(|(_, be)| be.id == "Barrel" || be.id == "minecraft:barrel")
        .count();
    assert!(barrels_count > 0, "Should have barrels");

    for (_, be) in &region.block_entities {
        if be.id.to_lowercase().contains("barrel") {
            if let Some(NbtValue::List(items)) = be.nbt.get("Items") {
                for item in items {
                    if let NbtValue::Compound(c) = item {
                        assert!(c.get("id").is_some(), "Item missing id: {:?}", c);
                        assert!(
                            c.get("Name").is_none(),
                            "Item should not have Name: {:?}",
                            c
                        );
                    }
                }
            }
        }
    }

    // 3. Check Redstone connections
    let redstone_wires: Vec<_> = palette
        .iter()
        .filter(|b| b.name == "minecraft:redstone_wire")
        .collect();

    for wire in redstone_wires {
        let directions = ["north", "south", "east", "west"];
        let mut connections = 0;
        for dir in directions {
            if let Some(val) = wire.properties.get(dir) {
                if val != "none" {
                    connections += 1;
                }
            }
        }

        // This assertion might fail for wires that are truly isolated (0 connections),
        // but the fix ensures that if there is 1, it becomes 2.
        // So connections can be 0 or >= 2. It cannot be 1.
        assert!(
            connections != 1,
            "Redstone wire has exactly 1 connection: {:?}",
            wire
        );
    }

    // 4. Export to .schem
    let schem_data = to_schematic(&schematic).expect("Failed to export to .schem");
    let output_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/samples/auto_register.schem");
    let mut file = std::fs::File::create(&output_path).expect("Failed to create output file");
    file.write_all(&schem_data)
        .expect("Failed to write to file");
    println!("Exported converted schematic to: {:?}", output_path);
}
