//! Convert a `.litematic` into the `.snbt` structure format the mc-tick corpus
//! and the gametest harness both read.
//!
//!     cargo run --example litematic_to_snbt -- <in.litematic> <out.snbt>
//!
//! The point of having this in the repo rather than doing it ad hoc: the
//! conversion is part of the conformance chain. A door is only as faithful as
//! the file it came from, and the first pass at these fixtures silently dropped
//! every block entity — so barrels lost their contents and comparators lost
//! their stored `OutputSignal`, which is what a comparator actually emits until
//! it next re-evaluates. Both are carried through here.
use nucleation::block_position::BlockPosition;
use nucleation::UniversalSchematic;
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let (Some(input), Some(output)) = (args.next(), args.next()) else {
        eprintln!("usage: litematic_to_snbt <in.litematic> <out.snbt>");
        std::process::exit(2);
    };

    let data = std::fs::read(&input)?;
    let schematic = nucleation::litematic::from_litematic(&data)?;

    // Normalise to a zero-based box: the corpus fixtures are all origin-anchored,
    // and the harness places them at its own origin anyway.
    let blocks: Vec<(BlockPosition, String)> = schematic
        .iter_blocks()
        .map(|(pos, state)| {
            let mut descriptor = state.get_name().to_string();
            if !state.properties.is_empty() {
                let mut props: Vec<String> = state
                    .properties
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect();
                props.sort();
                descriptor = format!("{descriptor}[{}]", props.join(","));
            }
            (pos, descriptor)
        })
        .collect();
    let (min_x, min_y, min_z) = blocks
        .iter()
        .fold((i32::MAX, i32::MAX, i32::MAX), |a, (p, _)| {
            (a.0.min(p.x), a.1.min(p.y), a.2.min(p.z))
        });

    // Block entities by (shifted) position, so a block can find its own tag.
    let mut tags: BTreeMap<(i32, i32, i32), String> = BTreeMap::new();
    for entity in schematic.get_block_entities_as_list() {
        let pos = (
            entity.position.0 - min_x,
            entity.position.1 - min_y,
            entity.position.2 - min_z,
        );
        if let Some(rendered) = render_nbt(&entity) {
            tags.insert(pos, rendered);
        }
    }

    let mut palette: Vec<String> = Vec::new();
    let mut entries: Vec<String> = Vec::new();
    let (mut size_x, mut size_y, mut size_z) = (0, 0, 0);
    for (pos, descriptor) in &blocks {
        if descriptor == "minecraft:air" {
            continue;
        }
        let index = match palette.iter().position(|p| p == descriptor) {
            Some(index) => index,
            None => {
                palette.push(descriptor.clone());
                palette.len() - 1
            }
        };
        let (x, y, z) = (pos.x - min_x, pos.y - min_y, pos.z - min_z);
        size_x = size_x.max(x + 1);
        size_y = size_y.max(y + 1);
        size_z = size_z.max(z + 1);
        match tags.get(&(x, y, z)) {
            Some(nbt) => entries.push(format!(
                "{{pos: [{x}, {y}, {z}], state: {index}, nbt: {nbt}}}"
            )),
            None => entries.push(format!("{{pos: [{x}, {y}, {z}], state: {index}}}")),
        }
    }

    let palette_text: Vec<String> = palette
        .iter()
        .map(|descriptor| match descriptor.split_once('[') {
            Some((name, rest)) => {
                let props: Vec<String> = rest
                    .trim_end_matches(']')
                    .split(',')
                    .filter(|p| !p.is_empty())
                    .map(|p| {
                        let (k, v) = p.split_once('=').unwrap_or((p, ""));
                        format!("{k}: \"{v}\"")
                    })
                    .collect();
                format!("{{Name: \"{name}\", Properties: {{{}}}}}", props.join(", "))
            }
            None => format!("{{Name: \"{descriptor}\"}}"),
        })
        .collect();

    let text = format!(
        "{{\n  DataVersion: 4903,\n  size: [{size_x}, {size_y}, {size_z}],\n  palette: [\n    {}\n  ],\n  blocks: [\n    {}\n  ],\n  entities: []\n}}\n",
        palette_text.join(",\n    "),
        entries.join(",\n    ")
    );
    std::fs::write(&output, text)?;
    println!(
        "{input} -> {output}: {} blocks, {} palette entries, {} block entities",
        entries.len(),
        palette.len(),
        tags.len()
    );
    Ok(())
}

/// Render the parts of a block entity the engine and the game both act on.
///
/// Deliberately narrow: `Items` for containers and `OutputSignal` for
/// comparators. Writing the whole tag back out would carry Bedrock-translated
/// and version-specific keys the structure loader then rejects, and everything
/// omitted here is either cosmetic or re-derived on placement.
fn render_nbt(entity: &nucleation::block_entity::BlockEntity) -> Option<String> {
    use nucleation::nbt::NbtValue;
    let mut parts: Vec<String> = Vec::new();

    if let Some(NbtValue::List(items)) = entity.nbt.get("Items") {
        let rendered: Vec<String> = items
            .iter()
            .filter_map(|item| {
                let NbtValue::Compound(fields) = item else {
                    return None;
                };
                let id = match fields.get("id") {
                    Some(NbtValue::String(id)) => id.clone(),
                    _ => return None,
                };
                let slot = match fields.get("Slot") {
                    Some(NbtValue::Byte(slot)) => *slot as i64,
                    Some(NbtValue::Int(slot)) => *slot as i64,
                    _ => 0,
                };
                let count = match fields.get("count").or_else(|| fields.get("Count")) {
                    Some(NbtValue::Byte(count)) => *count as i64,
                    Some(NbtValue::Int(count)) => *count as i64,
                    _ => 1,
                };
                Some(format!("{{Slot: {slot}b, id: \"{id}\", count: {count}}}"))
            })
            .collect();
        if !rendered.is_empty() {
            parts.push(format!("Items: [{}]", rendered.join(", ")));
        }
    }

    if let Some(value) = entity.nbt.get("OutputSignal") {
        let signal = match value {
            NbtValue::Byte(v) => Some(*v as i64),
            NbtValue::Int(v) => Some(*v as i64),
            _ => None,
        };
        if let Some(signal) = signal {
            parts.push(format!("OutputSignal: {signal}"));
        }
    }

    if parts.is_empty() {
        // Still a block entity, and the game still calls `setChanged` on it —
        // which is a neighbour notification. Keep the tag, empty.
        return Some(format!("{{id: \"{}\"}}", entity.id));
    }
    Some(format!("{{id: \"{}\", {}}}", entity.id, parts.join(", ")))
}
