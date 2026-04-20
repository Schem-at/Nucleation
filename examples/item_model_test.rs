use nucleation::formats::manager::get_manager;
use nucleation::meshing::{ItemModelConfig, ResourcePackSource};

fn main() {
    let pack_path = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("MINECRAFT_RESOURCE_PACK").ok())
        .expect(
            "Usage: item_model_test <resource_pack_path> [schematic_path] [output.zip]\n  \
             Or set MINECRAFT_RESOURCE_PACK env var",
        );

    let schem_path = std::env::args().nth(2).unwrap_or_else(|| {
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/samples/test_cube.schem").to_string()
    });

    let output_path = std::env::args()
        .nth(3)
        .unwrap_or_else(|| "item_model_pack.zip".to_string());

    // Load schematic
    println!("Loading schematic: {}", schem_path);
    let schem_data = std::fs::read(&schem_path).expect("Failed to read schematic");
    let manager = get_manager();
    let manager = manager.lock().unwrap();
    let schematic = manager
        .read(&schem_data)
        .expect("Failed to parse schematic");

    let bb = schematic.default_region.get_bounding_box();
    let dims = schematic.default_region.get_dimensions();
    println!(
        "Schematic: {}x{}x{} (bbox {:?}..{:?})",
        dims.0, dims.1, dims.2, bb.min, bb.max
    );

    // Load resource pack
    println!("Loading resource pack: {}", pack_path);
    let pack = ResourcePackSource::from_file(&pack_path).expect("Failed to load resource pack");
    let stats = pack.stats();
    println!(
        "Resource pack: {} blockstates, {} models, {} textures",
        stats.blockstate_count, stats.model_count, stats.texture_count
    );

    // Generate item model
    let config = ItemModelConfig::new("test_schematic");
    println!("Generating item model...");
    let result = schematic
        .to_item_model(&pack, &config)
        .expect("Failed to generate item model");

    println!("Item model generated:");
    println!("  Elements: {}", result.stats.element_count);
    println!("  Textures: {}", result.stats.texture_count);
    println!("  Planes:   {}", result.stats.plane_count);
    println!("  Dims:     {:?}", result.stats.dimensions);
    println!();

    // Print model JSON
    println!("=== Model JSON ===");
    println!("{}", result.model_json);
    println!();

    // Print texture info
    println!("=== Textures ===");
    for (name, data) in &result.textures {
        println!("  {} ({} bytes)", name, data.len());
    }
    println!();

    // Save resource pack ZIP
    println!("Saving resource pack to: {}", output_path);
    let zip_data = result
        .to_resource_pack_zip()
        .expect("Failed to create resource pack ZIP");
    std::fs::write(&output_path, &zip_data).expect("Failed to write ZIP");
    println!("Done! Resource pack saved ({} bytes)", zip_data.len());
    println!();
    println!("To use in Minecraft:");
    println!("  1. Copy {} to .minecraft/resourcepacks/", output_path);
    println!("  2. Enable the resource pack in Minecraft settings");
    println!("  3. Give yourself the item: /give @s minecraft:paper{{CustomModelData:1}}",);
    println!("     Or reference the model: nucleation:item/test_schematic");
}
