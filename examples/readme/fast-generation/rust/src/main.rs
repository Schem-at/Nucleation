//! Executable Rust source for docs/features/fast-generation.md.

use std::collections::BTreeSet;
use std::error::Error;
use std::fs;

const WIDTH: i32 = 48;

fn light_positions() -> Vec<(i32, i32, i32)> {
    let mut positions = BTreeSet::new();
    for p in (0..WIDTH).step_by(4) {
        positions.extend([
            (p, 2, 0),
            (p, 2, WIDTH - 1),
            (0, 2, p),
            (WIDTH - 1, 2, p),
            (p, 2, WIDTH / 2),
            (WIDTH / 2, 2, p),
        ]);
    }
    positions.into_iter().collect()
}

fn towers() -> impl Iterator<Item = (i32, i32, i32)> {
    (4..44).step_by(8).flat_map(|gx| {
        (4..44)
            .step_by(8)
            .map(move |gz| (gx, gz, 6 + ((gx / 8 + gz / 8) % 5) * 2))
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:build]
    use nucleation::UniversalSchematic;

    let mut campus = UniversalSchematic::new("bulk_campus".into());

    // The cuboid path performs one bounds expansion and one palette lookup.
    campus.fill_cuboid_str(
        (0, 0, 0),
        (WIDTH - 1, 1, WIDTH - 1),
        "minecraft:polished_deepslate",
    );

    // Pre-size once, then resolve every material to its region palette index.
    campus.ensure_bounds((0, 0, 0), (WIDTH - 1, 15, WIDTH - 1));
    let region = &mut campus.default_region;
    let light = region.get_or_insert_palette_by_name("minecraft:sea_lantern");
    let brick = region.get_or_insert_palette_by_name("minecraft:deepslate_bricks");
    let glass = region.get_or_insert_palette_by_name("minecraft:light_blue_stained_glass");
    let cap = region.get_or_insert_palette_by_name("minecraft:oxidized_cut_copper");

    for (x, y, z) in light_positions() {
        region.set_block_at_index_unchecked(light, x, y, z);
    }

    for (gx, gz, height) in towers() {
        for y in 2..height + 2 {
            let material = if y == height + 1 {
                cap
            } else if y % 3 == 0 {
                glass
            } else {
                brick
            };
            for dx in 0..3 {
                for dz in 0..3 {
                    region.set_block_at_index_unchecked(material, gx + dx, y, gz + dz);
                }
            }
        }
    }
    // --8<-- [end:build]

    // --8<-- [start:inspect]
    println!("{}", campus.total_blocks()); // 6926
    println!("{:?}", campus.get_tight_dimensions()); // (48, 16, 48)
    println!("{}", campus.get_block(36, 15, 4).unwrap()); // minecraft:oxidized_cut_copper

    // --8<-- [end:inspect]

    assert_eq!(campus.total_blocks(), 6_926);
    assert_eq!(campus.get_tight_dimensions(), (48, 16, 48));
    assert_eq!(
        campus.get_block(36, 15, 4).unwrap().to_string(),
        "minecraft:oxidized_cut_copper"
    );

    let output =
        std::env::var("FAST_GENERATION_OUT").unwrap_or_else(|_| "bulk-campus.schem".to_string());
    fs::write(&output, campus.to_schematic()?)?;
    println!("Fast generation Rust example: OK ({output})");
    Ok(())
}
