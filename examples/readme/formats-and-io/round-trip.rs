//! README illustration: the round-trip proof.
//!
//! One build, written to every writable format, read back, and fingerprinted.
//! The fingerprint is content-exact, so an identical hash *is* proof the
//! round-trip lost nothing — and a differing hash shows exactly where a format
//! can't represent something. This is self-verifying: if a format regresses,
//! regenerating this output changes.
//!
//!     cargo run --release --example readme_formats
//!
//! No resource pack needed — this is data, not pixels.

use nucleation::fingerprint::{fingerprint, FingerprintSpec};
use nucleation::formats::{gametest_snbt, litematic, mcstructure, schematic as sponge, snapshot};
use nucleation::UniversalSchematic;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "render_work/formats".to_string());
    std::fs::create_dir_all(&out)?;

    // A build that exercises the things formats tend to drop: many block types,
    // block-state properties, and a block entity carrying NBT.
    let mut s = UniversalSchematic::new("round-trip".to_string());
    for x in 0..4 {
        for z in 0..4 {
            s.set_block_from_string(x, 0, z, "minecraft:stone_bricks")?;
        }
    }
    s.set_block_from_string(1, 1, 1, "minecraft:oak_stairs[facing=east,half=bottom]")?;
    s.set_block_from_string(2, 1, 1, "minecraft:lever[face=floor,facing=east]")?;
    // A chest carrying block-entity NBT — the `exact` fingerprint includes it,
    // so this genuinely tests whether each format preserves tile-entity data.
    let mut nbt = std::collections::HashMap::new();
    nbt.insert("CustomName".to_string(), "Treasure".to_string());
    s.set_block_with_nbt(0, 1, 0, "minecraft:chest[facing=south]", nbt)?;

    // Save the downloads that sit beside the illustration.
    std::fs::write(format!("{out}/round-trip.schem"), sponge::to_schematic(&s)?)?;
    std::fs::write(
        format!("{out}/round-trip.litematic"),
        litematic::to_litematic(&s)?,
    )?;

    let spec = FingerprintSpec::exact();
    let original = fingerprint(&s, &spec).to_hex();
    println!("original          {}  ({} blocks)\n", original, count(&s));

    // Each writer, its extension, and the bytes it produced.
    let writers: Vec<(&str, &str, Vec<u8>)> = vec![
        ("litematic", ".litematic", litematic::to_litematic(&s)?),
        ("sponge", ".schem", sponge::to_schematic(&s)?),
        (
            "gametest_snbt",
            ".snbt",
            gametest_snbt::to_gametest_snbt(&s)?,
        ),
        ("snapshot", ".nusn", snapshot::to_snapshot(&s)?),
        (
            "mcstructure",
            ".mcstructure",
            mcstructure::to_mcstructure(&s)?,
        ),
    ];

    let mgr = nucleation::formats::manager::get_manager();
    println!(
        "{:<13} {:>8}  {:<16}  round-trip",
        "format", "bytes", "fingerprint"
    );
    println!("{}", "-".repeat(56));
    for (name, ext, bytes) in &writers {
        std::fs::write(format!("{out}/round-trip{ext}"), bytes)?;
        // Read it back through the same auto-detecting path a user would.
        let back = mgr.lock().unwrap().read(bytes)?;
        let hex = fingerprint(&back, &spec).to_hex();
        let verdict = if hex == original {
            "IDENTICAL"
        } else {
            "differs (see notes)"
        };
        println!("{:<13} {:>8}  {}  {}", name, bytes.len(), hex, verdict);
    }
    println!("\nwrote {out}/round-trip.{{schem,litematic,snbt,nusn,mcstructure}}");
    Ok(())
}

fn count(s: &UniversalSchematic) -> usize {
    s.iter_blocks()
        .filter(|(_, b)| b.get_name() != "minecraft:air")
        .count()
}
