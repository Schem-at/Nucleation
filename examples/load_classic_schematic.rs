//! Verification tool for the classic `.schematic` importer.
//!
//! Run: `cargo run --example load_classic_schematic -- <path.schematic>`
//! Default: ./angel_statue9925860.schematic

use nucleation::formats::manager::get_manager;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "angel_statue9925860.schematic".into());
    let bytes = std::fs::read(&path).expect("read input");

    let manager = get_manager();
    let manager = manager.lock().unwrap();

    println!("detected format: {:?}", manager.detect_format(&bytes));

    let schem = manager.read(&bytes).expect("parse classic schematic");

    let (w, h, l) = schem.get_dimensions();
    println!("dimensions: {} × {} × {}", w, h, l);
    println!("non-air blocks: {}", schem.total_blocks());

    let palette = schem.get_default_region_palette();
    println!("distinct palette entries: {}", palette.len());
    for state in &palette {
        if state.properties.is_empty() {
            println!("  {}", state.name);
        } else {
            let mut props: Vec<(String, String)> = state
                .properties
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            props.sort();
            let body = props
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(",");
            println!("  {}[{}]", state.name, body);
        }
    }
}
