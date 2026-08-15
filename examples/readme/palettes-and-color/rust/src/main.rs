//! Executable Rust source for docs/features/palettes-and-color.md.

use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:choose]
    use nucleation::building::{BlockPalette, PaletteBuilder};

    let safe_green = PaletteBuilder::new()
        .full_blocks_only()
        .exclude_transparent()
        .exclude_falling()
        .survival_obtainable_only()
        .color_near(42, 132, 92, 0.20)
        .build();

    let concrete = BlockPalette::new_concrete();
    let gray_ids = [
        "minecraft:black_concrete",
        "minecraft:gray_concrete",
        "minecraft:light_gray_concrete",
        "minecraft:white_concrete",
    ];
    let gray = BlockPalette::from_block_ids(gray_ids);
    assert!(!safe_green.is_empty());
    assert_eq!(concrete.len(), 16);
    assert_eq!(gray.len(), 4);
    // --8<-- [end:choose]

    // --8<-- [start:build]
    use nucleation::blockpedia::ExtendedColorData;
    use nucleation::{BlockState, UniversalSchematic};

    let mut atlas = UniversalSchematic::new("color_atlas".into());

    // A distinct 12-block ramp. No block id may repeat.
    let ramp = concrete
        .ramp_ids((20, 50, 150), (250, 200, 30), 12)
        .expect("concrete has enough distinct blocks");
    for x in 0..32 {
        atlas.set_block_str(x, 15, 0, &ramp[x as usize * ramp.len() / 32]);
    }

    // A 32-sample lookup table. Repeated ids are expected on a 16-color palette.
    let gradient = concrete.gradient_ids((20, 50, 150), (250, 200, 30), 32);
    for (x, block) in gradient.iter().enumerate() {
        atlas.set_block_str(x as i32, 13, 0, block);
    }

    // Ordered dithering extends a four-block grayscale palette across 32 values.
    for y in 0..12 {
        for x in 0..32 {
            let value = (x * 255 / 31) as u8;
            let target = ExtendedColorData::from_rgb(value, value, value);
            let block = gray
                .find_closest_dithered(&target, x, y, 0)
                .expect("gray palette is not empty");
            atlas.set_block(x, y, 0, &BlockState::new(block));
        }
    }
    // --8<-- [end:build]

    // --8<-- [start:inspect]
    assert_eq!(atlas.total_blocks(), 448);
    assert_eq!(atlas.get_tight_dimensions(), (32, 16, 1));
    let mut unique_ramp = ramp.clone();
    unique_ramp.sort();
    unique_ramp.dedup();
    assert_eq!(ramp.len(), unique_ramp.len());
    let mut unique_gradient = gradient.clone();
    unique_gradient.sort();
    unique_gradient.dedup();
    assert_eq!(gradient.len(), 32);
    assert!(unique_gradient.len() < gradient.len());
    // --8<-- [end:inspect]

    let output =
        std::env::var("PALETTES_COLOR_OUT").unwrap_or_else(|_| "color-atlas.schem".into());
    fs::write(&output, atlas.to_schematic()?)?;
    println!("Palettes and color Rust example: OK ({output})");
    Ok(())
}
