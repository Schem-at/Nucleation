use nucleation::formats::manager::get_manager;
use nucleation::formats::world;

#[test]
fn test_uss_texas_to_world() {
    let data = std::fs::read("/Users/harrison/Downloads/uss-texas.schem").unwrap();
    println!("Schematic size: {} bytes", data.len());

    let manager = get_manager();
    let manager = manager.lock().unwrap();

    let schematic = manager.read(&data).expect("Failed to read schematic");
    let bb = schematic.default_region.get_bounding_box();
    println!("Bounding box: {:?}", bb);

    // Count blocks
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

    // Export to world files using our generated level.dat
    let opts = world::WorldExportOptions {
        world_name: "USS Texas".to_string(),
        ..Default::default()
    };
    let files = world::to_world(&schematic, Some(opts)).expect("Failed to export");

    // Write to saves dir
    let saves_dir = std::path::Path::new(
        "/Users/harrison/Library/Application Support/PrismLauncher/instances/1.21.11/minecraft/saves/USS Texas"
    );
    if saves_dir.exists() {
        std::fs::remove_dir_all(saves_dir).unwrap();
    }

    for (path, file_data) in &files {
        let full_path = saves_dir.join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&full_path, file_data).unwrap();
        println!("Wrote {} ({} bytes)", path, file_data.len());
    }

    println!("\nWorld written to: {}", saves_dir.display());
    println!("Block count: {}", block_count);
}
