use nucleation::formats::world;
use std::collections::HashMap;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Usage: inspect_world <path_to_zip>");
    let data = std::fs::read(&path).expect("Failed to read file");

    println!("=== World Zip Analysis: {} ===", path);
    println!("File size: {} bytes\n", data.len());

    // Import the world
    let schematic = world::from_world_zip(&data).expect("Failed to import world");
    let region = &schematic.default_region;
    let bb = region.get_bounding_box();

    // Count blocks by type
    let mut block_counts: HashMap<String, usize> = HashMap::new();
    let mut block_positions: Vec<(i32, i32, i32, String)> = Vec::new();
    let (min_x, min_y, min_z) = bb.min;
    let (max_x, max_y, max_z) = bb.max;

    for y in min_y..=max_y {
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                if let Some(block) = region.get_block(x, y, z) {
                    if block.name != "minecraft:air" {
                        let key = if block.properties.is_empty() {
                            block.name.clone()
                        } else {
                            let mut props: Vec<_> = block.properties.iter().collect();
                            props.sort_by_key(|(k, _)| (*k).clone());
                            let prop_str: Vec<String> =
                                props.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                            format!("{}[{}]", block.name, prop_str.join(","))
                        };
                        block_positions.push((x, y, z, key.clone()));
                        *block_counts.entry(key).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // Find tight bounding box of actual blocks
    if block_positions.is_empty() {
        println!("No non-air blocks found!");
        return;
    }

    let tight_min_x = block_positions.iter().map(|(x, _, _, _)| *x).min().unwrap();
    let tight_max_x = block_positions.iter().map(|(x, _, _, _)| *x).max().unwrap();
    let tight_min_y = block_positions.iter().map(|(_, y, _, _)| *y).min().unwrap();
    let tight_max_y = block_positions.iter().map(|(_, y, _, _)| *y).max().unwrap();
    let tight_min_z = block_positions.iter().map(|(_, _, z, _)| *z).min().unwrap();
    let tight_max_z = block_positions.iter().map(|(_, _, z, _)| *z).max().unwrap();

    println!(
        "Build bounds: ({}, {}, {}) to ({}, {}, {})",
        tight_min_x, tight_min_y, tight_min_z, tight_max_x, tight_max_y, tight_max_z
    );
    println!(
        "Build size: {}x{}x{}",
        tight_max_x - tight_min_x + 1,
        tight_max_y - tight_min_y + 1,
        tight_max_z - tight_min_z + 1
    );

    // Sort by count
    let mut sorted: Vec<_> = block_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    let total: usize = sorted.iter().map(|(_, c)| *c).sum();

    println!(
        "\nBlock palette ({} types, {} total blocks):",
        sorted.len(),
        total
    );
    for (name, count) in &sorted {
        println!("  {:>5}x  {}", count, name);
    }

    // Block entities
    if !region.block_entities.is_empty() {
        println!("\nBlock entities ({}):", region.block_entities.len());
        for (pos, be) in &region.block_entities {
            println!("  {} at ({}, {}, {})", be.id, pos.0, pos.1, pos.2);
            for (key, value) in &be.nbt {
                println!("    {}: {:?}", key, value);
            }
        }
    }

    // Entities
    if !region.entities.is_empty() {
        println!("\nEntities ({}):", region.entities.len());
        for entity in &region.entities {
            println!(
                "  {} at ({:.1}, {:.1}, {:.1})",
                entity.id, entity.position.0, entity.position.1, entity.position.2
            );
            for (key, value) in &entity.nbt {
                println!("    {}: {:?}", key, value);
            }
        }
    } else {
        println!("\nNo entities found (this world has no mobs/items/etc.)");
    }

    // List every block with position
    println!("\n=== All {} blocks (sorted by position) ===", total);
    block_positions.sort_by_key(|(x, y, z, _)| (*y, *z, *x));
    for (x, y, z, name) in &block_positions {
        println!("  ({:>4}, {:>3}, {:>3})  {}", x, y, z, name);
    }

    // Layer view (tight bounds only)
    println!("\n=== Layer-by-layer view (tight bounds) ===");
    // Build a legend
    let mut legend: HashMap<String, char> = HashMap::new();
    let chars = "SsOoPpRrLlDdNnCcQqBbWwEeFfGgHhIiJjKkMm";
    let mut char_idx = 0;
    for (name, _) in &sorted {
        let short = name
            .strip_prefix("minecraft:")
            .unwrap_or(name)
            .split('[')
            .next()
            .unwrap();
        let c = if char_idx < chars.len() {
            chars.chars().nth(char_idx).unwrap()
        } else {
            '?'
        };
        legend.insert(name.to_string(), c);
        char_idx += 1;
    }

    println!("Legend:");
    for (name, _) in &sorted {
        println!("  {} = {}", legend[name.as_str()], name);
    }

    for y in tight_min_y..=tight_max_y {
        let mut has_blocks = false;
        for (bx, by, bz, _) in &block_positions {
            if *by == y {
                has_blocks = true;
                break;
            }
        }
        if !has_blocks {
            continue;
        }

        println!(
            "\nY={} (x={}..{}, z={}..{}):",
            y, tight_min_x, tight_max_x, tight_min_z, tight_max_z
        );
        for z in tight_min_z..=tight_max_z {
            let mut row = String::new();
            let mut has_block_in_row = false;
            for x in tight_min_x..=tight_max_x {
                if let Some(block) = region.get_block(x, y, z) {
                    if block.name == "minecraft:air" {
                        row.push('.');
                    } else {
                        let key = if block.properties.is_empty() {
                            block.name.clone()
                        } else {
                            let mut props: Vec<_> = block.properties.iter().collect();
                            props.sort_by_key(|(k, _)| (*k).clone());
                            let prop_str: Vec<String> =
                                props.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
                            format!("{}[{}]", block.name, prop_str.join(","))
                        };
                        row.push(*legend.get(&key).unwrap_or(&'?'));
                        has_block_in_row = true;
                    }
                } else {
                    row.push('.');
                }
            }
            if has_block_in_row {
                println!("  z={:>4}: {}", z, row);
            }
        }
    }
}
