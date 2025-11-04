use nucleation::{litematic, schematic, BlockState, UniversalSchematic};
use std::fs;
use std::path::Path;

fn litematic_to_schem_conversion(name: &str) {
    // Path to the sample .litematic file
    let input_path_str = format!("tests/samples/{}.litematic", name);
    let litematic_path = Path::new(&input_path_str);

    // Ensure the sample file exists
    assert!(litematic_path.exists(), "Sample .litematic file not found");

    // Read the .litematic file
    let litematic_data =
        fs::read(litematic_path).expect(format!("Failed to read {}", input_path_str).as_str());

    // Parse the .litematic data into a UniversalSchematic
    let mut schematic =
        litematic::from_litematic(&litematic_data).expect("Failed to parse litematic");

    // let region_blocks = schematic.get_region_from_index(0).unwrap().blocks.clone();

    //print the length of the blocks list
    // println!("{:?}", region_blocks.len());
    //print the blocks list
    // println!("{:?}", region_blocks);

    // println!("{:?}", schematic.count_block_types());
    //place a diamond block at the center of the schematic
    // schematic.set_block(-1,-1,-1, BlockState::new("minecraft:diamond_block".to_string()));

    // let dimensions = schematic.get_dimensions();
    // let width = dimensions.0;
    // let height = dimensions.1;
    // let length = dimensions.2;
    // for x in 0..width {
    //     for z in 0..length {
    //         schematic.set_block(x, -1, z, BlockState::new("minecraft:gray_concrete".to_string()));
    //     }
    // }
    // print the schematic in json format
    // let json = print_json_schematic(&schematic);
    // println!("{}", json);

    // Convert the UniversalSchematic to .schem format
    let schem_data = schematic::to_schematic(&schematic).expect("Failed to convert to schem");

    // Save the .schem file
    let output_schem_path = format!("tests/output/{}.schem", name);
    let schem_path = Path::new(&output_schem_path);
    fs::write(schem_path, &schem_data).expect("Failed to write schem file");

    // Convert the UniversalSchematic back to .litematic format
    let litematic_data =
        litematic::to_litematic(&schematic).expect("Failed to convert to litematic");

    // Save the .litematic file
    let output_litematic_path = format!("tests/output/{}.litematic", name);
    let litematic_path = Path::new(&output_litematic_path);
    fs::write(litematic_path, &litematic_data).expect("Failed to write litematic file");

    // Optionally, read back the .schem file and compare
    let read_back_data = fs::read(schem_path).expect("Failed to read back schem file");
    let read_back_schematic =
        schematic::from_schematic(&read_back_data).expect("Failed to parse schem");

    // Compare original and converted schematics
    assert_eq!(schematic.metadata.name, read_back_schematic.metadata.name);

    // Clean up the generated file
    //fs::remove_file(schem_path).expect("Failed to remove converted schem file");

    println!("Successfully converted sample.litematic to .schem format and verified the contents.");
}

fn schema_to_litematic_conversion(name: &str) {
    // Path to the sample .schem file
    let input_path_str = format!("tests/samples/{}.schem", name);
    let schem_path = Path::new(&input_path_str);

    // Ensure the sample file exists
    assert!(schem_path.exists(), "Sample .schem file not found");

    // Read the .schem file
    let schem_data =
        fs::read(schem_path).expect(format!("Failed to read {}", input_path_str).as_str());

    // Parse the .schem data into a UniversalSchematic
    let schematic = schematic::from_schematic(&schem_data).expect("Failed to parse schem");
    let block_entities = schematic.get_block_entities_as_list();
    println!("{:?}", block_entities);
    // Convert the UniversalSchematic to .litematic format
    let litematic_data =
        litematic::to_litematic(&schematic).expect("Failed to convert to litematic");

    // Save the .litematic file
    let output_litematic_path = format!("tests/output/{}.litematic", name);
    let litematic_path = Path::new(&output_litematic_path);
    fs::write(litematic_path, &litematic_data).expect("Failed to write litematic file");
}

#[test]
fn load_evaluator_schematic() {
    let input_path_str = format!("tests/samples/Evaluator.schem");
    let schem_path = Path::new(&input_path_str);
    let schem_data =
        fs::read(schem_path).expect(format!("Failed to read {}", input_path_str).as_str());
    let schematic = schematic::from_schematic(&schem_data).expect("Failed to parse schem");
    let block_entities = schematic.get_block_entities_as_list();
    let entities = schematic.get_entities_as_list();
    println!("{:?}", block_entities);
    println!("{:?}", entities);
}

#[test]
fn test_cube_schematic() {
    let input_path_str = format!("tests/samples/test_cube.schem");

    let schem_path = Path::new(&input_path_str);
    let schem_data =
        fs::read(schem_path).expect(format!("Failed to read {}", input_path_str).as_str());
    let schematic = schematic::from_schematic(&schem_data).expect("Failed to parse schem");

    //save the schematic as litematic
    let litematic_data =
        litematic::to_litematic(&schematic).expect("Failed to convert to litematic");
    let output_litematic_path = format!("tests/output/test_cube.litematic");
    let litematic_path = Path::new(&output_litematic_path);
    fs::write(litematic_path, &litematic_data).expect("Failed to write litematic file");
}

// #[test]
fn time_load_large_schematic() {
    let input_path_str = format!("tests/samples/large_schematic.schem");
    let schem_path = Path::new(&input_path_str);
    let schem_data =
        fs::read(schem_path).expect(format!("Failed to read {}", input_path_str).as_str());
    let start_time = std::time::Instant::now();
    let _schematic = schematic::from_schematic(&schem_data).expect("Failed to parse schem");
    let elapsed_time = start_time.elapsed();
    println!("Time taken to load large schematic: {:?}", elapsed_time);
}

#[test]
fn test_expand_schematic() {
    // Create a new empty schematic
    let mut schematic = UniversalSchematic::new("Default".to_string());

    // Print initial state
    println!("Initial dimensions: {:?}", schematic.get_dimensions());
    println!("Initial bbox: {:?}", schematic.get_bounding_box());

    // Set a block
    let result = schematic.set_block(
        4,
        4,
        4,
        BlockState::new("minecraft:sea_lantern".to_string()),
    );
    println!("Block set result: {}", result);

    // Print region info
    if let Some(region) = schematic.get_region("Main") {
        println!("Region position: {:?}", region.position);
        println!("Region size: {:?}", region.size);
        println!("Region bbox: {:?}", region.get_bounding_box());
    }

    // Check if block was set
    let block = schematic.get_block(4, 4, 4);
    println!("Block at (4,4,4): {:?}", block.map(|b| b.get_name()));

    // Print final state
    println!("Final dimensions: {:?}", schematic.get_dimensions());
    println!("Final bbox: {:?}", schematic.get_bounding_box());

    // Assert expected dimensions
    assert_eq!(schematic.get_dimensions(), (1, 1, 1));
}

// convert litematic-rose-farm from samples to schem
#[test]
fn test_litematic_to_schem_conversion() {
    // convert only litematic-rose-farm.litematic to schem
    let test_files = list_test_file("litematic");
    for file_name in test_files {
        if file_name == "litematic-rose-farm" {
            litematic_to_schem_conversion(&file_name);
        }
    }
    // Clean up the generated file
    // fs::remove_file("tests/output/litematic-rose-farm.schem").expect("Failed to remove converted schem file");
    println!("Successfully converted litematic-rose-farm.litematic to .schem format and verified the contents.");
}

struct TestFiles<'a> {
    extension: &'a str,
    reader: fs::ReadDir,
}

impl<'a> Iterator for TestFiles<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.reader.next() {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file()
                    && path.extension().and_then(|ext| ext.to_str()) == Some(self.extension)
                {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        return Some(stem.to_string());
                    }
                }
            }
        }
        None
    }
}

fn list_test_file(extension: &str) -> TestFiles {
    const DIR_PATH: &str = "./tests/samples";
    TestFiles {
        extension,
        reader: fs::read_dir(DIR_PATH).unwrap(),
    }
}
