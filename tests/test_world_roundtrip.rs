use nucleation::formats::manager::get_manager;
use nucleation::formats::world;

#[test]
#[ignore] // Requires local files: /Users/harrison/Downloads/55 3x3.zip
fn test_55_3x3_roundtrip() {
    // Step 1: Import the world zip
    let data = std::fs::read("/Users/harrison/Downloads/55 3x3.zip").unwrap();
    println!("Input zip size: {} bytes", data.len());

    let manager = get_manager();
    let manager = manager.lock().unwrap();

    let schematic = manager.read(&data).expect("Failed to import world zip");
    let bb = schematic.default_region.get_bounding_box();
    println!("Imported bounding box: {:?}", bb);
    println!("Regions: {:?}", schematic.get_region_names());

    // Count non-air blocks
    let mut block_count = 0;
    let (min, max) = (bb.min, bb.max);
    for y in min.1..=max.1 {
        for z in min.2..=max.2 {
            for x in min.0..=max.0 {
                if schematic.get_block(x, y, z).is_some() {
                    block_count += 1;
                }
            }
        }
    }
    println!("Block count: {}", block_count);

    // Step 2: Export using to_world (raw files, no zip wrapper)
    let files = world::to_world(&schematic, None).expect("Failed to export to world format");

    // Step 3: Write directly to MC saves directory
    let saves_dir = std::path::Path::new(
        "/Users/harrison/Library/Application Support/PrismLauncher/instances/1.21.11/minecraft/saves/Nucleation Export"
    );
    // Clean and recreate
    if saves_dir.exists() {
        std::fs::remove_dir_all(saves_dir).unwrap();
    }
    for (path, data) in &files {
        let full_path = saves_dir.join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&full_path, data).unwrap();
        println!("Wrote {} ({} bytes)", path, data.len());
    }
    println!("\nWorld written directly to: {}", saves_dir.display());

    // Also save zip for comparison
    let output = manager
        .write("world", &schematic, None)
        .expect("Failed to export to world format");
    std::fs::write("/tmp/nucleation_roundtrip_output.zip", &output).unwrap();

    // Step 4: Re-import the exported world
    match manager.read(&output) {
        Ok(reimported) => {
            let bb2 = reimported.default_region.get_bounding_box();
            println!("Re-imported bounding box: {:?}", bb2);

            let mut block_count2 = 0;
            let (min2, max2) = (bb2.min, bb2.max);
            for y in min2.1..=max2.1 {
                for z in min2.2..=max2.2 {
                    for x in min2.0..=max2.0 {
                        if reimported.get_block(x, y, z).is_some() {
                            block_count2 += 1;
                        }
                    }
                }
            }
            println!("Re-imported block count: {}", block_count2);
            assert_eq!(
                block_count, block_count2,
                "Block count mismatch after roundtrip"
            );
        }
        Err(e) => {
            panic!("Failed to re-import exported world: {}", e);
        }
    }

    // Step 5: Compare our level.dat with original
    let cursor = std::io::Cursor::new(&data);
    let mut orig_archive = zip::ZipArchive::new(cursor).unwrap();
    for i in 0..orig_archive.len() {
        let name = orig_archive.by_index_raw(i).unwrap().name().to_string();
        if name.ends_with("level.dat") {
            let mut file = orig_archive.by_index(i).unwrap();
            let mut orig_leveldat = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut orig_leveldat).unwrap();
            println!("\nOriginal level.dat: {} bytes", orig_leveldat.len());
            println!(
                "Exported level.dat: {} bytes",
                files.get("level.dat").unwrap().len()
            );
            break;
        }
    }
}
