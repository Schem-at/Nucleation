use nucleation::{litematic, UniversalSchematic};

fn main() {
    let mut schematic = UniversalSchematic::new("simple".to_string());

    // Lever, 3 wires, lamp - total 5 blocks in a line
    schematic.set_block_str(
        0,
        0,
        0,
        "minecraft:lever[facing=east,powered=false,face=floor]",
    );
    schematic.set_block_str(1, 0, 0, "minecraft:redstone_wire[power=0]");
    schematic.set_block_str(2, 0, 0, "minecraft:redstone_wire[power=0]");
    schematic.set_block_str(3, 0, 0, "minecraft:redstone_wire[power=0]");
    schematic.set_block_str(4, 0, 0, "minecraft:redstone_lamp[lit=false]");

    println!("Schematic dimensions: {:?}", schematic.get_dimensions());

    // Export to litematic
    let data = litematic::to_litematic(&schematic).unwrap();

    // Compress and base64 encode
    use base64::{engine::general_purpose, Engine as _};
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&data).unwrap();
    let compressed = encoder.finish().unwrap();

    let b64 = general_purpose::STANDARD.encode(&compressed);

    println!(
        "\nBase64 litematic ({} bytes uncompressed, {} compressed):",
        data.len(),
        compressed.len()
    );
    println!("{}", b64);
}
