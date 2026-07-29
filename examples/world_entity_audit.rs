//! What does a world carry that the tick simulator never sees?
//!
//! Loads a world zip, reports its entities by type, and then reports how many
//! of them survive the conversion the browser path actually performs
//! (`TickSimulation::gametest_snbt`). The gap between the two numbers is the
//! answer: entities authored in a build that the simulator is never told about
//! do not fail loudly, they simply are not there.
//!
//! `cargo run --release --example world_entity_audit --features bridge,mc-tick -- <world.zip>`
use std::collections::BTreeMap;

use nucleation::bridge::mc_tick::ffi::TickSimulation;
use nucleation::bridge::schematic::ffi::Schematic;

/// Call a bridge function that writes into a `DiplomatWrite` and get a String.
///
/// Takes a closure rather than a `fn` pointer, because every call here needs to
/// capture the schematic. Borrows the same growable buffer the generated C
/// bindings use — from Rust the only public constructors are the C entry points.
fn read_out(f: impl FnOnce(&mut diplomat_runtime::DiplomatWrite)) -> String {
    unsafe {
        let write = diplomat_runtime::diplomat_buffer_write_create(0);
        f(&mut *write);
        let text = String::from_utf8_lossy((*write).as_bytes()).into_owned();
        diplomat_runtime::diplomat_buffer_write_destroy(write);
        text
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("a world zip");
    let bytes = std::fs::read(&path).expect("read the zip");
    println!("{} — {} bytes", path, bytes.len());

    let schem = match Schematic::from_world_zip(&bytes) {
        Ok(s) => s,
        Err(e) => {
            println!("LOAD FAILED: {e:?}");
            return;
        }
    };

    let (x, y, z) = {
        let d = schem.dimensions();
        (d.x, d.y, d.z)
    };
    println!("loaded: {x} x {y} x {z}, {} entities", schem.entity_count());

    // By type, so "stacked entities inside the door" is visible as a count
    // rather than a total.
    let json = read_out(|w| schem.get_entities_json(w));
    let mut by_id: BTreeMap<String, usize> = BTreeMap::new();
    // The JSON is a list of objects with an `id`; count without pulling in a
    // parser, since all we need is the type histogram.
    for chunk in json.split("\"id\"").skip(1) {
        let after = chunk.trim_start().trim_start_matches(':').trim_start();
        let Some(rest) = after.strip_prefix('"') else { continue };
        let Some(end) = rest.find('"') else { continue };
        *by_id.entry(rest[..end].to_string()).or_default() += 1;
    }
    for (id, n) in &by_id {
        println!("   {n:>6}  {id}");
    }

    // Now the conversion the app performs. `entities: []` in the emitted SNBT
    // means every one of the above is dropped before the engine ever runs.
    let snbt = read_out(|w| TickSimulation::gametest_snbt(&schem, w));
    let carried = snbt
        .split_once("entities:")
        .map(|(_, tail)| tail.trim_start().starts_with("[]"))
        .map(|empty| if empty { 0 } else { usize::MAX });
    println!(
        "\ngametest SNBT: {} bytes, entities carried into the simulator: {}",
        snbt.len(),
        match carried {
            Some(0) => "NONE — all dropped".to_string(),
            Some(_) => "some".to_string(),
            None => "no entities field?".to_string(),
        }
    );
}
