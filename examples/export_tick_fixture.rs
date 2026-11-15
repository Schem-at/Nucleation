//! Export any supported schematic through the production importer to mc-tick's
//! structure format. Usage: export_tick_fixture INPUT OUTPUT [SPEC_OUTPUT].
//! Save versions and block-entity contents are preserved by the shared exporter.
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert!((3..=4).contains(&args.len()), "INPUT OUTPUT [SPEC_OUTPUT]");
    let bytes = std::fs::read(&args[1]).expect("read schematic");
    let schematic = nucleation::formats::manager::get_manager()
        .lock().unwrap().read(&bytes).expect("import schematic");
    let snbt = nucleation::formats::gametest::to_gametest_snbt(&schematic);
    let structure = mc_tick::Structure::parse(&snbt).expect("parse exported fixture");
    std::fs::write(&args[2], snbt).expect("write fixture");
    if let Some(path) = args.get(3) {
        std::fs::write(path, schematic.metadata.embedded_test.as_deref().unwrap_or("null"))
            .expect("write embedded scenario");
    }
    eprintln!("{} blocks, {} inventories, {} entities, save version {:?}",
        structure.blocks.len(), structure.inventories.len(), structure.entities.len(),
        structure.data_version);
}
