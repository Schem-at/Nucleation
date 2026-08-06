//! Convert a schematic to the structure SNBT dialect the tick tooling consumes.
//!
//! Both `tools/gametest` (via the game's own `TagParser`) and
//! `crates/mc-tick/src/structure.rs` read the *binary structure schema* written
//! as SNBT text — `palette` as a list of `{Name, Properties}` compounds and
//! `blocks` as palette indices. That is not what
//! `formats::structure_snbt::to_structure_snbt` emits (its `data` entries carry
//! state strings), so this walks the blocks directly.
use nucleation::formats::manager::get_manager;
use std::collections::BTreeMap;
use std::fmt::Write as _;

fn main() {
    let mut args = std::env::args().skip(1);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        eprintln!("usage: schematic_to_snbt <in.litematic|...> <out.snbt>");
        std::process::exit(2);
    };
    let data = std::fs::read(&input).expect("read input");
    let schematic = get_manager().lock().unwrap().read(&data).expect("parse");

    let bounds = schematic.get_bounding_box();
    let (min, max) = (bounds.min, bounds.max);
    let size = (max.0 - min.0 + 1, max.1 - min.1 + 1, max.2 - min.2 + 1);

    // BTreeMap keys the palette on the rendered entry, so identical states share
    // an index and the palette order is deterministic across runs.
    let mut palette: BTreeMap<String, usize> = BTreeMap::new();
    let mut blocks: Vec<((i32, i32, i32), usize)> = Vec::new();
    for (pos, state) in schematic.iter_blocks() {
        if state.name == "minecraft:air" {
            continue;
        }
        let entry = render_palette_entry(&state.name, &state.properties);
        let next = palette.len();
        let index = *palette.entry(entry).or_insert(next);
        blocks.push(((pos.x - min.0, pos.y - min.1, pos.z - min.2), index));
    }
    blocks.sort_by_key(|((x, y, z), _)| (*y, *z, *x));

    let mut ordered: Vec<&str> = vec![""; palette.len()];
    for (entry, index) in &palette {
        ordered[*index] = entry;
    }

    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  DataVersion: 4903,");
    let _ = writeln!(out, "  size: [{}, {}, {}],", size.0, size.1, size.2);
    let _ = writeln!(out, "  palette: [");
    let _ = writeln!(out, "    {}", ordered.join(",\n    "));
    let _ = writeln!(out, "  ],");
    let _ = writeln!(out, "  blocks: [");
    let rendered: Vec<String> = blocks
        .iter()
        .map(|((x, y, z), state)| format!("    {{pos: [{x}, {y}, {z}], state: {state}}}"))
        .collect();
    let _ = writeln!(out, "{}", rendered.join(",\n"));
    let _ = writeln!(out, "  ],");
    let _ = writeln!(out, "  entities: []");
    let _ = writeln!(out, "}}");

    std::fs::write(&output, &out).expect("write output");
    println!(
        "{input}: {} x {} x {} -> {output} ({} blocks, {} palette entries)",
        size.0,
        size.1,
        size.2,
        blocks.len(),
        palette.len()
    );
}

fn render_palette_entry(
    name: &str,
    properties: &[(smol_str::SmolStr, smol_str::SmolStr)],
) -> String {
    if properties.is_empty() {
        return format!("{{Name: \"{name}\"}}");
    }
    let mut sorted: Vec<_> = properties.to_vec();
    sorted.sort();
    let rendered: Vec<String> = sorted
        .iter()
        .map(|(k, v)| format!("{k}: \"{v}\""))
        .collect();
    format!(
        "{{Name: \"{name}\", Properties: {{{}}}}}",
        rendered.join(", ")
    )
}
