//! Cut a live build out of a world save, block entities and all.
//!
//!     cargo run --example extract_world_door -- <world/dimension> <x0,y0,z0> <x1,y1,z1> <out.litematic>
//!
//! The point is fidelity a schematic export cannot give: a door in a world has
//! been *running*, so its comparators hold real output signals and its
//! repeaters are genuinely locked. Exported and re-pasted, that latched state
//! is recomputed away. Reading the world directly keeps it.
fn triple(text: &str) -> (i32, i32, i32) {
    let v: Vec<i32> = text.split(',').map(|p| p.trim().parse().unwrap()).collect();
    (v[0], v[1], v[2])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let dir = args.next().unwrap();
    let min = triple(&args.next().unwrap());
    let max = triple(&args.next().unwrap());
    let out = args.next().unwrap();

    let schematic =
        nucleation::formats::world::from_world_directory_bounded(
            std::path::Path::new(&dir), min.0, min.1, min.2, max.0, max.1, max.2,
        )?;

    let solid: Vec<_> = schematic
        .iter_blocks()
        .filter(|(_, s)| s.get_name() != "minecraft:air")
        .map(|(p, _)| (p.x, p.y, p.z))
        .collect();
    if solid.is_empty() {
        eprintln!("nothing in that box");
        std::process::exit(1);
    }
    let lo = solid.iter().fold((i32::MAX, i32::MAX, i32::MAX), |a, p| {
        (a.0.min(p.0), a.1.min(p.1), a.2.min(p.2))
    });
    let hi = solid.iter().fold((i32::MIN, i32::MIN, i32::MIN), |a, p| {
        (a.0.max(p.0), a.1.max(p.1), a.2.max(p.2))
    });
    println!("solid blocks {} in {:?}..{:?}", solid.len(), lo, hi);
    let entities = schematic.get_block_entities_as_list();
    println!("block entities: {}", entities.len());
    for e in &entities {
        let interesting: Vec<String> = e
            .nbt
            .iter()
            .filter(|(k, _)| *k == "OutputSignal" || *k == "Items")
            .map(|(k, v)| format!("{k}={v:?}"))
            .collect();
        if !interesting.is_empty() {
            println!("   {} at {:?}  {}", e.id, e.position, interesting.join(" "));
        }
    }
    std::fs::write(&out, nucleation::litematic::to_litematic(&schematic)?)?;
    println!("wrote {out}");
    Ok(())
}
