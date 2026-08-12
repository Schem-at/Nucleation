//! Read-only audit for the final connected-component schematic splitter.
//!
//! Usage:
//!   cargo run --release --example component_split_audit -- \
//!     build.schem [min-standalone-blocks] [max-air-gap]

use std::path::PathBuf;

use nucleation::formats::schematic::from_schematic;
use nucleation::Connectivity;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = PathBuf::from(args.next().ok_or("missing schematic path")?);
    let min_standalone_blocks = args
        .next()
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(16);
    let max_air_gap = args
        .next()
        .map(|value| value.parse::<u32>())
        .transpose()?
        .unwrap_or(3);
    if args.next().is_some() {
        return Err("too many arguments".into());
    }

    let bytes = std::fs::read(&path)?;
    let schematic = from_schematic(&bytes)?;
    let components = schematic.connected_components(Connectivity::Corner);
    let pieces = schematic.split_connected_attach_nearby(
        Connectivity::Corner,
        min_standalone_blocks,
        max_air_gap,
    );
    let component_sizes = components
        .iter()
        .map(|component| component.blocks.len())
        .collect::<Vec<_>>();
    let piece_sizes = pieces
        .iter()
        .map(|piece| piece.total_blocks())
        .collect::<Vec<_>>();
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "path": path,
            "blocks": schematic.total_blocks(),
            "components": components.len(),
            "component_sizes": component_sizes,
            "min_standalone_blocks": min_standalone_blocks,
            "max_air_gap": max_air_gap,
            "pieces": pieces.len(),
            "piece_sizes": piece_sizes,
        }))?
    );
    Ok(())
}
