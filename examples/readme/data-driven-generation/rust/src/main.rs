//! Executable Rust source for docs/features/data-driven-generation.md.

// --8<-- [start:example]
use std::error::Error;
use std::fs;

fn barrel_position(x: u32, y: u32, channel: u32) -> (i32, i32, i32) {
    let alternate = (x & 1) as i32;
    let z = if channel & 1 == y & 1 {
        5 + alternate
    } else {
        5 * alternate
    };
    (
        -(((channel + y) & 1) as i32),
        -2 - channel as i32 - 3 * y as i32,
        6 * (x / 2) as i32 + z - 2,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    use nucleation::UniversalSchematic;

    let image = image::open("rom-input.png")?.to_rgb8();
    let mut rom = UniversalSchematic::new("image_rom".to_owned());

    for (x, y, pixel) in image.enumerate_pixels() {
        let [red, green, blue] = pixel.0;

        for (channel, signal) in [blue >> 4, green >> 4, red >> 4].into_iter().enumerate() {
            let (bx, by, bz) = barrel_position(x, y, channel as u32);
            rom.set_block_from_string(
                bx,
                by,
                bz,
                &format!("minecraft:barrel{{signal={signal}}}"),
            )?;
        }
    }

    fs::write("image-rom.schem", rom.to_schematic()?)?;

    println!("Data-driven Rust example: OK");
    Ok(())
}
// --8<-- [end:example]
