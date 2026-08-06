//! Measure any build the way a `.litematic`-embedded scenario measures it.
//!
//! ```text
//! cargo run --release --example scenario_inspect -- <path> [options]
//!
//!   --settle in-world|quiet|placement   default in-world
//!   --press x,y,z                       what to right-click (default: the first button)
//!   --at T[,T...]                       tick(s) to press on (default: 5)
//!   --ticks N                           how long to run (default: 400)
//!   --cells x,y,z:x,y,z                 an inclusive cell box to report the fill of
//!   --every N                           also report the fill every N ticks
//!   --dump-test                         print the build's embedded scenario and stop
//!   --embed spec.json --write out.litematic
//!                                       attach a scenario to the build and save it
//!   --dump-trace out.json               write the run's block changes as a
//!                                       capture-shaped trace (render with
//!                                       render_simulation_video --trace)
//!   --dump-entities out.log             write per-tick entity positions in
//!                                       TraceCapture's `E t...` lines (render
//!                                       with --entity-log)
//! ```
//!
//! The last two are the authoring loop for a self-testing `.litematic`: measure
//! the build here, write the numbers into a descriptor, `--embed` it, and from
//! then on `cargo test --test litematic_cases` runs it. `--dump-test` gets the
//! descriptor back out for editing, so the only copy of a scenario is the one
//! inside the file it tests.
//!
//! This is the one diagnostic left where there used to be five door-specific
//! ones (`door55_sim`, `door55_doorway`, `door55_xray`, `world_entity_audit`,
//! `door_batch_load`). It takes a path rather than knowing about a door, and it
//! prints exactly the quantities a scenario descriptor can assert — entity
//! count, rider seats, block changes, quiescence, the lowest entity y, and the
//! fill of a named cell box — so authoring a case is reading numbers off this
//! and writing them down, and nothing measured here is measured *only* here.
//!
//! It deliberately reports end state plus a coarse timeline, never per-tick
//! internals: the same discipline the scenarios themselves keep, so that a
//! faster redstone backend changes nothing this prints that a test reads.

use mc_tick::{Pos, Structure};
use nucleation::formats::gametest::to_gametest_snbt;

// The harness the embedded scenarios run on. This was once a `#[path]` module
// pointing into mc-tick's test support; it is a real crate now, and sharing it
// is the point — an inspector that measures a build differently from the suite
// that judges it is an inspector that lies.
use mc_test as scenario;

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(String::as_str)
}

fn pos_arg(text: &str) -> Pos {
    let c: Vec<i32> = text.split(',').map(|p| p.trim().parse().expect("x,y,z")).collect();
    assert_eq!(c.len(), 3, "expected x,y,z, got {text:?}");
    Pos::new(c[0], c[1], c[2])
}

/// Every cell of an inclusive `x,y,z:x,y,z` box, in the order a picture reads:
/// top row first, then west to east.
fn cell_box(text: &str) -> Vec<Pos> {
    let (a, b) = text.split_once(':').expect("--cells x,y,z:x,y,z");
    let (lo, hi) = (pos_arg(a), pos_arg(b));
    let mut cells = Vec::new();
    for y in (lo.y.min(hi.y)..=lo.y.max(hi.y)).rev() {
        for z in lo.z.min(hi.z)..=lo.z.max(hi.z) {
            for x in lo.x.min(hi.x)..=lo.x.max(hi.x) {
                cells.push(Pos::new(x, y, z));
            }
        }
    }
    cells
}

fn fill(sim: &mc_tick::Simulation, cells: &[Pos]) -> (usize, String) {
    let mut filled = 0;
    let mut picture = String::new();
    for pos in cells {
        let id = sim.world().get(*pos);
        let solid =
            sim.registry().descriptor(id).is_some_and(|d| d != "minecraft:air");
        if solid {
            filled += 1;
        }
        picture.push(if solid { '#' } else { '.' });
    }
    (filled, picture)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).cloned().unwrap_or_else(|| {
        eprintln!(
            "usage: scenario_inspect <path> [--settle M] [--press x,y,z] [--at T,..] \
             [--ticks N] [--cells x,y,z:x,y,z] [--every N]"
        );
        std::process::exit(1);
    });
    let settle = match flag(&args, "--settle").unwrap_or("in-world") {
        "in-world" => scenario::SettleMode::InWorld,
        "quiet" => scenario::SettleMode::Quiet,
        "placement" => scenario::SettleMode::Placement,
        other => {
            eprintln!("unknown settle mode {other:?}");
            std::process::exit(1);
        }
    };
    let at: Vec<u64> = flag(&args, "--at")
        .map(|v| v.split(',').map(|t| t.trim().parse().expect("--at T,T")).collect())
        .unwrap_or_else(|| vec![5]);
    let ticks: u64 = flag(&args, "--ticks").map_or(400, |v| v.parse().expect("--ticks N"));
    let every: u64 = flag(&args, "--every").map_or(0, |v| v.parse().expect("--every N"));
    let cells: Vec<Pos> = flag(&args, "--cells").map(cell_box).unwrap_or_default();

    let bytes = std::fs::read(&path).expect("read the build");
    let manager = nucleation::formats::manager::get_manager();
    let mut schematic = manager.lock().unwrap().read(&bytes).expect("parse the build");

    if args.iter().any(|a| a == "--dump-test") {
        match &schematic.metadata.embedded_test {
            Some(spec) => println!("{spec}"),
            None => {
                eprintln!("{path} carries no embedded test");
                std::process::exit(1);
            }
        }
        return;
    }

    // Attach a scenario and save. The descriptor is stored verbatim so a
    // `--dump-test | edit | --embed` round trip is byte-stable.
    if let Some(spec_path) = flag(&args, "--embed") {
        let out = flag(&args, "--write").expect("--embed needs --write out.litematic");
        let spec = std::fs::read_to_string(spec_path).expect("read the descriptor");
        schematic.metadata.embedded_test = Some(spec.trim().to_string());
        let written = nucleation::formats::litematic::to_litematic(&schematic)
            .expect("the build writes as a litematic");
        std::fs::write(out, &written).expect("write the litematic");
        println!("wrote {out} ({} bytes) with its test inside", written.len());
        return;
    }

    println!("# {path}");
    println!("source_data_version: {:?}", schematic.metadata.source_data_version);
    println!(
        "embedded test: {}",
        match &schematic.metadata.embedded_test {
            Some(spec) => format!("{} bytes of descriptor", spec.len()),
            None => "none".to_string(),
        }
    );

    let snbt = to_gametest_snbt(&schematic);
    let structure = Structure::parse(&snbt).expect("the engine accepts this build");
    let mut sim = scenario::build_sim(
        &structure,
        Pos::new(0, 0, 0),
        settle,
        &[],
        &[],
        schematic.metadata.source_data_version,
        "inspect",
    );

    // Everything a scenario can assert about entities, before anything runs.
    println!("entities: {}", sim.entity_bodies().len());
    let mut seats: Vec<(String, f64)> =
        sim.riders().into_iter().map(|(_, kind, pos)| (kind, pos[1])).collect();
    seats.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    for (kind, y) in &seats {
        println!("  rider {kind} seated at y={y:.4}");
    }
    let non_finite = sim.minecarts().iter().filter(|c| c.vel.iter().any(|v| !v.is_finite())).count();
    println!("  carts with a non-finite velocity: {non_finite} of {}", sim.minecarts().len());

    // The actuator is searched for rather than assumed: two extractions of the
    // same build can land it in different places.
    let press = flag(&args, "--press").map(pos_arg).or_else(|| {
        sim.world()
            .iter_non_air()
            .find(|(_, id)| {
                sim.registry().descriptor(*id).is_some_and(|d| d.contains("_button"))
            })
            .map(|(pos, _)| pos)
    });
    match press {
        Some(pos) => println!("pressing {pos:?} at tick(s) {at:?}, running {ticks} ticks"),
        None => println!("nothing to press; running {ticks} ticks untouched"),
    }
    if !cells.is_empty() {
        let (filled, picture) = fill(&sim, &cells);
        println!("t0     fill {filled}/{} {picture}", cells.len());
    }

    let mut entity_lines = String::new();
    for tick in 0..=ticks {
        if at.contains(&tick) {
            if let Some(pos) = press {
                sim.use_block(pos);
            }
        }
        // TraceCapture's own `E t...` shape, absolute positions riders
        // included, so the render pipeline draws either side of a comparison
        // from the same kind of file.
        for body in sim.entity_bodies() {
            let x = (body.min[0] + body.max[0]) * 0.5;
            let z = (body.min[2] + body.max[2]) * 0.5;
            entity_lines.push_str(&format!(
                "  E t{tick} id={} {} pos=({}, {}, {})\n",
                body.id, body.kind, x, body.min[1], z
            ));
        }
        if tick < ticks {
            sim.step();
        }
        if every > 0 && !cells.is_empty() && tick > 0 && tick % every == 0 {
            let (filled, picture) = fill(&sim, &cells);
            println!("t{tick:<5} fill {filled}/{} {picture}", cells.len());
        }
    }

    println!("after {ticks} ticks:");
    println!("  block changes: {}", sim.recorded().len());
    println!("  quiescent: {}", sim.is_quiescent());
    println!("  entities: {}", sim.entity_bodies().len());
    let low = sim
        .entity_bodies()
        .iter()
        .map(|b| b.min[1])
        .chain(sim.item_entities().iter().filter(|e| !e.removed).map(|e| e.pos[1]))
        .fold(f64::INFINITY, f64::min);
    println!("  lowest entity y: {low}");
    if !cells.is_empty() {
        let (filled, picture) = fill(&sim, &cells);
        println!("  fill {filled}/{} {picture}", cells.len());
    }

    if let Some(path) = flag(&args, "--dump-entities") {
        std::fs::write(path, &entity_lines).expect("write entity log");
        println!("  wrote {path}");
    }
    if let Some(path) = flag(&args, "--dump-trace") {
        // The same JSON shape a TraceCapture recording has, minimally: enough
        // for `mc_tick_trace::Trace::from_json` and therefore for
        // `render_simulation_video --trace`.
        let mut ticks_out: Vec<String> = Vec::new();
        let mut current: Option<(u64, Vec<String>)> = None;
        let escape = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        for change in sim.recorded() {
            let from = sim.registry().descriptor(change.from).unwrap_or("?");
            let to = sim.registry().descriptor(change.to).unwrap_or("?");
            let event = format!(
                "{{\"phase\": \"tick_end\", \"kind\": \"block_changed\", \"pos\": [{}, {}, {}], \"from\": \"{}\", \"to\": \"{}\"}}",
                change.pos.x, change.pos.y, change.pos.z, escape(from), escape(to)
            );
            match &mut current {
                Some((tick, events)) if *tick == change.tick => events.push(event),
                _ => {
                    if let Some((tick, events)) = current.take() {
                        ticks_out.push(format!(
                            "{{\"tick\": {tick}, \"events\": [{}]}}",
                            events.join(", ")
                        ));
                    }
                    current = Some((change.tick, vec![event]));
                }
            }
        }
        if let Some((tick, events)) = current.take() {
            ticks_out
                .push(format!("{{\"tick\": {tick}, \"events\": [{}]}}", events.join(", ")));
        }
        let json = format!(
            "{{\"format_version\": 1, \"mc_version\": \"mc-tick\", \"structure\": \"inspect\", \"detail\": \"normal\", \"ticks\": [{}]}}",
            ticks_out.join(", ")
        );
        std::fs::write(path, json).expect("write trace");
        println!("  wrote {path}");
    }
}
