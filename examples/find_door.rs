//! Locate a redstone door in a world save by looking for its distinctive parts.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args().nth(1).unwrap();
    let path = std::path::Path::new(&dir);
    let schematic = nucleation::formats::world::from_world_directory(path)?;
    let mut hits: Vec<(i32, i32, i32, String)> = Vec::new();
    for (pos, state) in schematic.iter_blocks() {
        let name = state.get_name();
        if name.ends_with("barrel") || name.ends_with("sticky_piston") || name.ends_with("comparator") {
            hits.push((pos.x, pos.y, pos.z, name.to_string()));
        }
    }
    println!("candidate blocks: {}", hits.len());
    // cluster by rounding to 16
    use std::collections::BTreeMap;
    let mut buckets: BTreeMap<(i32, i32, i32), usize> = BTreeMap::new();
    for (x, y, z, _) in &hits {
        *buckets.entry((x >> 4, y >> 4, z >> 4)).or_default() += 1;
    }
    let mut ranked: Vec<_> = buckets.into_iter().collect();
    ranked.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    for ((cx, cy, cz), n) in ranked.into_iter().take(8) {
        println!("  chunk-ish ({}, {}, {}) -> {} parts  [blocks {}..{}, {}..{}, {}..{}]",
            cx, cy, cz, n, cx * 16, cx * 16 + 15, cy * 16, cy * 16 + 15, cz * 16, cz * 16 + 15);
    }
    Ok(())
}
