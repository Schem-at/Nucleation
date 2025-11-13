use flate2::read::GzDecoder;
use nucleation::UniversalSchematic;
use quartz_nbt::io::{read_nbt, Flavor};
use quartz_nbt::{NbtCompound, NbtTag};
use std::io::BufReader;

/// Test various block entities to ensure proper NBT structure according to Sponge Schematic v3 spec
#[test]
fn test_block_entity_nbt_structure_v3() {
    test_block_entity_structure(
        "minecraft:barrel[facing=north]{signal=14}",
        "barrel",
        vec!["Items"],
    );
}

#[test]
fn test_chest_block_entity_structure_v3() {
    test_block_entity_structure(
        r#"minecraft:chest[facing=north]{Items:[{"Count":"1b","Slot":"0b","id":"minecraft:diamond"}]}"#,
        "chest",
        vec!["Items"],
    );
}

#[test]
fn test_sign_block_entity_structure_v3() {
    test_block_entity_structure(
        "minecraft:sign[rotation=0]{CustomName:'{\"text\":\"Test Sign\"}'}",
        "sign",
        vec!["CustomName"],
    );
}

/// Generic test function to validate BlockEntity structure according to Sponge Schematic v3 spec
///
/// According to the spec (src/formats/sponge_schematic_v3.md lines 196-223):
/// BlockEntity structure should be:
/// {
///     "Pos": [x, y, z],     // Required
///     "Id": "type",          // Required  
///     "Data": {              // Optional - contains block-specific NBT
///         "Items": [...],
///         "CustomName": "...",
///         // ... other block-specific data
///     }
/// }
fn test_block_entity_structure(
    block_string: &str,
    block_type: &str,
    expected_data_fields: Vec<&str>,
) {
    println!("\n=== Testing Block Entity: {} ===", block_type);
    println!("Block string: {}", block_string);

    // Create a schematic with the block entity
    let mut schematic = UniversalSchematic::new("test".to_string());
    schematic
        .set_block_from_string(0, 0, 0, block_string)
        .expect("Failed to set block from string");

    // Convert to schematic bytes
    let schem_bytes = schematic
        .to_schematic()
        .expect("Failed to convert to schematic");

    // Parse the NBT structure
    let reader = BufReader::with_capacity(1 << 20, &schem_bytes[..]);
    let mut gz = GzDecoder::new(reader);
    let (root, _) = read_nbt(&mut gz, Flavor::Uncompressed).expect("Failed to read NBT");

    // Navigate to the Schematic tag
    let schematic_tag = root
        .get::<_, &NbtCompound>("Schematic")
        .expect("No Schematic tag");

    // Get the version
    let version = schematic_tag
        .get::<_, i32>("Version")
        .expect("No Version tag");
    println!("Schematic version: {}", version);
    assert_eq!(version, 3, "Test expects v3 schematic format");

    // Get the BlockEntities list from Blocks compound (v3 structure)
    let blocks = schematic_tag
        .get::<_, &NbtCompound>("Blocks")
        .expect("No Blocks compound in v3");
    let block_entities_list = blocks
        .get::<_, &quartz_nbt::NbtList>("BlockEntities")
        .expect("No BlockEntities list in Blocks compound");

    assert!(
        block_entities_list.len() > 0,
        "Expected at least one block entity"
    );

    // Get the first (and only) block entity
    let be_tag = &block_entities_list[0];
    let be_compound = match be_tag {
        NbtTag::Compound(c) => c,
        _ => panic!("BlockEntity should be a compound tag"),
    };

    println!("\nBlock entity structure:");
    print_nbt_structure(be_compound, 0);

    // Validate required fields according to spec
    println!("\n=== Structure Validation ===");

    // 1. Check for required 'Id' field
    assert!(
        be_compound.contains_key("Id"),
        "BlockEntity MUST have 'Id' field (spec requirement)"
    );
    let id = be_compound
        .get::<_, &String>("Id")
        .expect("Id should be a string");
    println!("✓ Has required 'Id' field: {}", id);
    assert!(
        id.contains(block_type),
        "Id should contain block type '{}', got '{}'",
        block_type,
        id
    );

    // 2. Check for required 'Pos' field
    assert!(
        be_compound.contains_key("Pos"),
        "BlockEntity MUST have 'Pos' field (spec requirement)"
    );
    let pos = be_compound
        .get::<_, &Vec<i32>>("Pos")
        .expect("Pos should be int array");
    println!("✓ Has required 'Pos' field: {:?}", pos);
    assert_eq!(pos.len(), 3, "Pos must contain exactly 3 integers");

    // 3. Check structure of block-specific data
    let has_data_compound = be_compound.contains_key("Data");
    let mut fields_at_root = Vec::new();

    for field in &expected_data_fields {
        if be_compound.contains_key(*field) {
            fields_at_root.push(*field);
        }
    }

    if !fields_at_root.is_empty() {
        println!("\n❌ VALIDATION FAILED");
        println!("According to Sponge Schematic v3 spec:");
        println!("  - Block-specific data MUST be inside a 'Data' compound");
        println!("  - Only 'Id' and 'Pos' should be at root level\n");
        println!(
            "Found block-specific fields at root level: {:?}",
            fields_at_root
        );
        println!("\nExpected structure:");
        println!("  BlockEntity {{");
        println!("    Id: \"minecraft:{}\",", block_type);
        println!("    Pos: [x, y, z],");
        println!("    Data: {{");
        for field in &expected_data_fields {
            println!("      {}: ...", field);
        }
        println!("    }}");
        println!("  }}\n");
        println!(
            "Actual structure has '{}' at root level instead of nested in 'Data'",
            fields_at_root.join(", ")
        );

        panic!(
            "BlockEntity structure violates Sponge Schematic v3 spec: fields {:?} should be in 'Data' compound",
            fields_at_root
        );
    }

    // 4. If Data compound exists, validate it contains the expected fields
    if has_data_compound {
        let data_compound = be_compound
            .get::<_, &NbtCompound>("Data")
            .expect("Data should be a compound tag");

        println!("✓ Has 'Data' compound");

        for field in &expected_data_fields {
            if data_compound.contains_key(*field) {
                println!("  ✓ Data.{} is present", field);
            } else {
                println!("  ⚠️  Data.{} is missing (may be optional)", field);
            }
        }

        println!("\n✅ VALIDATION PASSED");
        println!("BlockEntity structure conforms to Sponge Schematic v3 spec");
    } else {
        // No Data compound and no fields at root - this is also valid if block has no extra data
        if expected_data_fields.is_empty() {
            println!("\n✅ VALIDATION PASSED");
            println!("BlockEntity has no extra data (valid for blocks without additional NBT)");
        } else {
            println!("\n⚠️  WARNING");
            println!(
                "Expected fields {:?} but found neither 'Data' compound nor fields at root",
                expected_data_fields
            );
        }
    }
}

// Helper function to print NBT structure for debugging
fn print_nbt_structure(compound: &NbtCompound, indent: usize) {
    let indent_str = "  ".repeat(indent);
    for (key, value) in compound.inner() {
        match value {
            NbtTag::Compound(inner) => {
                println!("{}{}: Compound {{", indent_str, key);
                print_nbt_structure(inner, indent + 1);
                println!("{}}}", indent_str);
            }
            NbtTag::List(list) => {
                println!("{}{}: List[{}]", indent_str, key, list.len());
                if list.len() > 0 && list.len() <= 3 {
                    for (i, item) in list.iter().enumerate() {
                        if let NbtTag::Compound(c) = item {
                            println!("{}  [{}]:", indent_str, i);
                            print_nbt_structure(c, indent + 2);
                        } else {
                            println!("{}  [{}]: {:?}", indent_str, i, item);
                        }
                    }
                }
            }
            NbtTag::ByteArray(arr) => {
                println!("{}{}: ByteArray[{}]", indent_str, key, arr.len());
            }
            NbtTag::IntArray(arr) => {
                println!("{}{}: IntArray{:?}", indent_str, key, arr);
            }
            NbtTag::LongArray(arr) => {
                println!("{}{}: LongArray[{}]", indent_str, key, arr.len());
            }
            _ => {
                println!("{}{}: {:?}", indent_str, key, value);
            }
        }
    }
}
