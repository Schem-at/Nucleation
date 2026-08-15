//! Executable Rust source for docs/features/formats-and-io.md.

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn Error>> {
    // --8<-- [start:build]
    let mut build = UniversalSchematic::new("round_trip".into());
    build.fill_cuboid_str((0, 0, 0), (3, 0, 3), "minecraft:stone_bricks");
    build.set_block_from_string(
        1, 1, 1,
        "minecraft:oak_stairs[facing=east,half=bottom]",
    )?;
    build.set_block_from_string(
        2, 1, 1,
        "minecraft:lever[face=floor,facing=east,powered=false]",
    )?;
    build.set_block_with_nbt(
        0, 1, 0,
        "minecraft:chest[facing=south]",
        HashMap::from([("CustomName".into(), "Treasure".into())]),
    )?;
    // --8<-- [end:build]

    // --8<-- [start:bytes]
    use nucleation::formats::manager::get_manager;

    let manager = get_manager();
    let manager = manager.lock().map_err(|_| "format manager lock")?;
    let payload = manager.write("litematic", &build, None)?;
    let loaded = manager.read(&payload)?; // content detection; no filename required
    assert_eq!(loaded.total_blocks(), 19);

    let v3 = manager.write("schematic", &loaded, Some("v3"))?;
    fs::write("round-trip.schem", v3)?;
    // --8<-- [end:bytes]

    let formats = [
        ("litematic", None, ".litematic"),
        ("schematic", Some("v3"), ".schem"),
        ("structure_snbt", None, ".snbt"),
        ("snapshot", None, ".nusn"),
        ("mcstructure", None, ".mcstructure"),
    ];
    let output = PathBuf::from(
        std::env::var("FORMATS_IO_OUT_DIR").unwrap_or_else(|_| "formats-output".into()),
    );
    fs::create_dir_all(&output)?;
    for (format_name, version, extension) in formats {
        let data = manager.write(format_name, &build, version)?;
        let back = manager.read(&data)?;
        assert_eq!(back.total_blocks(), 19);
        fs::write(output.join(format!("round-trip{extension}")), data)?;
    }

    println!("Formats and I/O Rust example: OK ({})", output.display());
    Ok(())
}
