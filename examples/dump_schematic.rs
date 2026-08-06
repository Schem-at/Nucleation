//! Inspect a schematic: dimensions and block palette.
//!
//! Used to work out what the sample builds actually contain before turning one
//! into a simulation test.
use nucleation::formats::manager::get_manager;
use std::collections::BTreeMap;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: dump_schematic <file>");
    let data = std::fs::read(&path).expect("read");
    let schematic = get_manager().lock().unwrap().read(&data).expect("parse");

    let (x, y, z) = schematic.get_dimensions();
    println!("{path}: {x} x {y} x {z}");

    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for (_pos, block) in schematic.iter_blocks() {
        *counts.entry(block.name.to_string()).or_default() += 1;
    }
    for (name, n) in counts.iter() {
        println!("  {n:6}  {name}");
    }
}
