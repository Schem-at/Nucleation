use nucleation::formats::manager::get_manager;
use nucleation::meshing::{build_resource_pack, ItemModelConfig, ResourcePackSource};

fn main() {
    let pack_path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MINECRAFT_RESOURCE_PACK").ok())
        .expect(
            "Usage: multi_item_model <resource_pack_path> [output.zip]\n  \
             Or set MINECRAFT_RESOURCE_PACK env var",
        );

    let output_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "artifacts/multi_model_pack.zip".to_string());

    // Schematics to include: (path, model_name, custom_model_data)
    let schematics: Vec<(&str, &str, &str)> = vec![
        ("tests/samples/and.schem", "and_gate", "1"),
        ("tests/samples/test_cube.schem", "test_cube", "2"),
        ("tests/samples/cutecounter.schem", "cute_counter", "3"),
        ("tests/samples/sample.schem", "sample", "4"),
        ("tests/samples/new_chest_test.schem", "chest_test", "5"),
        ("tests/output/3x3.schem", "three_by_three", "6"),
        ("tests/output/4x2x4.schem", "four_by_four", "7"),
        ("tests/output/3x2x3.schem", "three_by_two", "8"),
        ("tests/output/4x4x4+1.schem", "four_cubed_plus", "9"),
        ("tests/output/wool_palette.schem", "wool_palette", "10"),
        ("tests/output/door_plot.schem", "door_plot", "11"),
        ("schematic_builder/generated/xor.schem", "xor_gate", "12"),
    ];

    // Load resource pack
    println!("Loading resource pack: {}", pack_path);
    let pack = ResourcePackSource::from_file(&pack_path).expect("Failed to load resource pack");
    let stats = pack.stats();
    println!(
        "  {} blockstates, {} models, {} textures\n",
        stats.blockstate_count, stats.model_count, stats.texture_count
    );

    let manager = get_manager();
    let manager = manager.lock().unwrap();

    let mut results = Vec::new();
    let mut loaded = 0;
    let mut skipped = 0;

    for (path, name, cmd) in &schematics {
        let full_path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path);
        let data = match std::fs::read(&full_path) {
            Ok(d) => d,
            Err(e) => {
                println!("  SKIP {} - {}", path, e);
                skipped += 1;
                continue;
            }
        };

        let schematic = match manager.read(&data) {
            Ok(s) => s,
            Err(e) => {
                println!("  SKIP {} - parse error: {}", path, e);
                skipped += 1;
                continue;
            }
        };

        let dims = schematic.default_region.get_dimensions();
        println!("  [{}] {} ({}x{}x{})", cmd, name, dims.0, dims.1, dims.2);

        let mut config = ItemModelConfig::new(*name);
        config.custom_model_data = cmd.to_string();
        // Auto scale is the default — oversized schematics will be scaled down automatically

        match schematic.to_item_model(&pack, &config) {
            Ok(result) => {
                let (sx, sy, sz) = result.stats.scale;
                let scale_info = if sx > 1.0 || sy > 1.0 || sz > 1.0 {
                    format!(" (scale: {:.2}x{:.2}x{:.2})", sx, sy, sz)
                } else {
                    String::new()
                };
                println!(
                    "    -> {} elements, {} textures, {} planes{}",
                    result.stats.element_count,
                    result.stats.texture_count,
                    result.stats.plane_count,
                    scale_info
                );
                results.push(result);
                loaded += 1;
            }
            Err(e) => {
                println!("    -> SKIP: {}", e);
                skipped += 1;
            }
        }
    }

    println!(
        "\nBuilding merged resource pack: {} models ({} skipped)",
        loaded, skipped
    );
    let refs: Vec<_> = results.iter().collect();
    let zip_data = build_resource_pack(&refs).expect("Failed to build resource pack");

    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        std::fs::create_dir_all(parent).ok();
    }

    std::fs::write(&output_path, &zip_data).expect("Failed to write ZIP");
    println!(
        "Saved to {} ({:.1} KB)\n",
        output_path,
        zip_data.len() as f64 / 1024.0
    );

    println!("To use in Minecraft 1.21.4+:");
    println!("  1. Copy {} to .minecraft/resourcepacks/", output_path);
    println!("  2. Enable the resource pack in-game");
    println!("  3. Give yourself items:");
    for (_, name, cmd) in &schematics[..loaded] {
        println!(
            "     /give @s paper[custom_model_data={{strings:[\"{}\"]}}]  # {}",
            cmd, name
        );
    }
}
