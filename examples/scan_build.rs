//! Bounding box of the technical blocks in a region of a world save.
//!
//!     cargo run --release --example scan_build -- <overworld dir> x0 x1 z0 z1
//!
//! Finding a build in a world by eye means loading the world. This asks the
//! region files instead: everything that is plainly machinery rather than
//! landscape, and the box that holds it.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let path = std::path::Path::new(&args[0]);
    let (x0, x1): (i32, i32) = (args[1].parse()?, args[2].parse()?);
    let (z0, z1): (i32, i32) = (args[3].parse()?, args[4].parse()?);
    let schematic = nucleation::formats::world::from_world_directory(path)?;

    const MACHINE: &[&str] = &[
        "piston", "observer", "redstone", "repeater", "comparator", "note_block", "target",
        "tripwire", "obsidian", "glass", "slime", "honey", "concrete", "wool", "quartz",
        "lever", "button", "barrel", "hopper", "dropper", "slab", "lamp", "wood", "leaves",
    ];
    let (mut lo, mut hi) = ((i32::MAX, i32::MAX, i32::MAX), (i32::MIN, i32::MIN, i32::MIN));
    let mut counts: std::collections::BTreeMap<String, usize> = Default::default();
    for (pos, state) in schematic.iter_blocks() {
        if pos.x < x0 || pos.x > x1 || pos.z < z0 || pos.z > z1 {
            continue;
        }
        let name = state.get_name();
        // The platform a build stands on is not the build. Excluded by name
        // rather than by guessing at what is structural.
        let excluded: Vec<&str> = std::env::var("SCAN_EXCLUDE")
            .map(|v| Box::leak(v.into_boxed_str()) as &str)
            .map(|v| v.split(',').collect())
            .unwrap_or_default();
        if name == "minecraft:air"
            || !MACHINE.iter().any(|m| name.contains(m))
            || excluded.iter().any(|e| !e.is_empty() && name.contains(e))
        {
            continue;
        }
        *counts.entry(name.to_string()).or_default() += 1;
        lo = (lo.0.min(pos.x), lo.1.min(pos.y), lo.2.min(pos.z));
        hi = (hi.0.max(pos.x), hi.1.max(pos.y), hi.2.max(pos.z));
    }
    println!("box: {lo:?} .. {hi:?}   ({} x {} x {})", hi.0 - lo.0 + 1, hi.1 - lo.1 + 1, hi.2 - lo.2 + 1);
    if std::env::var("SCAN_PROFILE").is_ok() {
        let mut per_x: std::collections::BTreeMap<i32, usize> = Default::default();
        for (pos, state) in schematic.iter_blocks() {
            if pos.x < x0 || pos.x > x1 || pos.z < z0 || pos.z > z1 { continue }
            let name = state.get_name();
            if name == "minecraft:air" || name.contains("gray_concrete") { continue }
            if !MACHINE.iter().any(|m| name.contains(m)) { continue }
            *per_x.entry(pos.x).or_default() += 1;
        }
        let line: Vec<String> = per_x.iter().map(|(x, n)| format!("{x}:{n}")).collect();
        println!("per-x: {}", line.join(" "));
    }
    let mut ranked: Vec<_> = counts.into_iter().collect();
    ranked.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    for (name, n) in ranked.into_iter().take(14) {
        println!("  {n:>5}  {name}");
    }
    Ok(())
}
