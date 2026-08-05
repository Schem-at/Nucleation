//! Diagnostic: run a build and report what it did.
//!
//! ```sh
//! MC_MACHINE=path.litematic MC_TICKS=20 MC_SETTLE=placement \
//!   cargo test -p nucleation-cli --test machine_probe -- --ignored --nocapture
//! ```
//!
//! | var | meaning |
//! |---|---|
//! | `MC_MACHINE` | the build to load (any format nucleation reads) |
//! | `MC_TICKS` | how many ticks to run (default 20) |
//! | `MC_SETTLE` | `placement` (default), `quiet`, `in-world` |
//! | `MC_USE` / `MC_USE_TICK` | right-click `x,y,z` on these ticks (comma list) |
//! | `MC_BREAK` / `MC_BREAK_TICK` | break `x,y,z` on these ticks — the trigger for a machine started by mining |
//! | `MC_FIND` | print every position whose state contains this, then stop |
//! | `MC_LAYOUT` | force the layout dump (automatic under 2000 cells) |
//! | `MC_CHANGES` | dump every block change instead of the per-tick summary |
use mc_test::mc_tick::{Pos, Structure};
use nucleation::formats::gametest::to_gametest_snbt;

/// The extent of everything non-air, which is how you tell a machine that
/// *moved* from one that merely twitched.
fn footprint(sim: &mc_test::mc_tick::Simulation) -> Option<(Pos, Pos)> {
    let mut it = sim.world().iter_non_air().map(|(p, _)| p);
    let first = it.next()?;
    Some(it.fold((first, first), |(lo, hi), p| {
        (
            Pos::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z)),
            Pos::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z)),
        )
    }))
}

fn coords(var: &str) -> Option<(i32, i32, i32)> {
    coord_list(var).first().copied()
}

/// `x,y,z;x,y,z;…` — several positions, for a build with more than one input.
///
/// An adder has seventeen levers; driving it one `MC_USE` at a time is not a
/// probe, it is a typing exercise.
fn coord_list(var: &str) -> Vec<(i32, i32, i32)> {
    let Ok(v) = std::env::var(var) else { return Vec::new() };
    v.split(';')
        .filter_map(|part| {
            let c: Vec<i32> = part.split(',').filter_map(|n| n.trim().parse().ok()).collect();
            if let [x, y, z] = c[..] { Some((x, y, z)) } else { None }
        })
        .collect()
}

fn ticks_list(var: &str, default: &str) -> Vec<u64> {
    std::env::var(var)
        .unwrap_or_else(|_| default.into())
        .split(',')
        .filter_map(|t| t.trim().parse().ok())
        .collect()
}

#[test]
#[ignore = "diagnostic, run by hand"]
fn run_machine() {
    // `MC_STRUCTURE=<file.snbt>` — parse a gametest structure directly, skipping
    // the schematic loader.
    //
    // A `.litematic` saved from a world is a snapshot of *when it was saved*. BB's
    // is two blocks and one piston-pair displaced from the same machine as it
    // stands in the save — so running the schematic and comparing against a
    // capture of the world compares two different builds, and every divergence
    // after that is noise. This is how you feed the engine exactly what the
    // oracle recorded.
    let from_snbt = std::env::var("MC_STRUCTURE").ok();
    let file = if from_snbt.is_some() {
        String::new()
    } else {
        std::env::var("MC_MACHINE").expect("set MC_MACHINE or MC_STRUCTURE")
    };
    let ticks: u64 = std::env::var("MC_TICKS").ok().and_then(|t| t.parse().ok()).unwrap_or(20);
    let settle = match std::env::var("MC_SETTLE").unwrap_or_else(|_| "placement".into()).as_str() {
        "quiet" => mc_test::SettleMode::Quiet,
        "in-world" => mc_test::SettleMode::InWorld,
        _ => mc_test::SettleMode::Placement,
    };
    let snbt = match &from_snbt {
        Some(path) => std::fs::read_to_string(path).expect("readable"),
        None => {
            let bytes = std::fs::read(&file).expect("readable");
            let manager = nucleation::formats::manager::get_manager();
            let schematic = manager.lock().unwrap().read(&bytes).expect("imports");
            to_gametest_snbt(&schematic)
        }
    };
    let structure = Structure::parse(&snbt).expect("engine parses");

    let (sx, sy, sz) = structure.size;
    eprintln!("=== {sx}x{sy}x{sz}, {} blocks", structure.blocks.len());

    // The same SNBT the engine just parsed, written where the oracle's pack
    // can load it — so the vanilla run and this one are the same build by
    // construction rather than by hand-conversion.
    if let Ok(path) = std::env::var("MC_DUMP_SNBT") {
        std::fs::write(&path, &snbt).expect("writable");
        eprintln!("=== wrote {} bytes of SNBT to {path}", snbt.len());
    }

    // Find first: on a build of any size, "where is the obsidian" is the
    // question you have before you can ask anything else.
    if let Ok(needle) = std::env::var("MC_FIND") {
        let mut hits = 0;
        for (pos, entry) in &structure.blocks {
            let state = structure.palette[*entry].as_str();
            if state.contains(&needle) {
                eprintln!("  [{},{},{}] {}", pos.x, pos.y, pos.z, state.replace("minecraft:", ""));
                hits += 1;
            }
        }
        eprintln!("=== {hits} match(es) for {needle:?}");
        panic!("probe output above");
    }

    // The full grid is only readable for something door-sized.
    if std::env::var_os("MC_LAYOUT").is_some() || (sx * sy * sz) <= 2000 {
        for y in (0..sy).rev() {
            for z in 0..sz {
                let mut row = format!("  y{y} z{z}: ");
                for x in 0..sx {
                    let at = structure
                        .blocks
                        .iter()
                        .find(|(p, _)| *p == Pos::new(x, y, z))
                        .map(|(_, e)| structure.palette[*e].as_str())
                        .unwrap_or("air");
                    row.push_str(&format!("[{x}]{} ", at.replace("minecraft:", "")));
                }
                eprintln!("{row}");
            }
        }
    }

    let mut sim =
        mc_test::build_sim(&structure, Pos::new(0, 0, 0), settle, &[], &[], None, "machine");
    sim.record();
    let air = sim.registry_mut().intern("minecraft:air").expect("air interns");

    let use_at = coord_list("MC_USE");
    let use_ticks = ticks_list("MC_USE_TICK", "5");
    // `MC_READ=x,y,z;…` — print these cells' states when the run ends. For a
    // build whose answer is a row of lamps rather than a change count.
    let read_at = coord_list("MC_READ");
    let break_at = coords("MC_BREAK");
    let break_ticks = ticks_list("MC_BREAK_TICK", "5");
    eprintln!(
        "=== {ticks} ticks (settle {settle:?}, use {use_at:?}@{use_ticks:?}, break {break_at:?}@{break_ticks:?})"
    );

    let dump_changes = std::env::var_os("MC_CHANGES").is_some();
    let mut seen = 0usize;
    let mut last_print: Option<String> = None;
    for t in 0..ticks {
        if use_ticks.contains(&t) {
            for (x, y, z) in &use_at {
                sim.use_block(Pos::new(*x, *y, *z));
                eprintln!("  -- used ({x},{y},{z}) at t{t}");
            }
        }
        if break_ticks.contains(&t) {
            if let Some((x, y, z)) = break_at {
                let was = sim.world().get(Pos::new(x, y, z));
                sim.place_block(Pos::new(x, y, z), air);
                eprintln!(
                    "  -- broke ({x},{y},{z}) at t{t} (was {})",
                    sim.registry().descriptor(was).unwrap_or("?").replace("minecraft:", "")
                );
            }
        }
        sim.step();
        // `MC_DUMP_AT=<tick>` — the whole non-air world after that tick.
        //
        // Reconstructing our state by replaying the change log is *wrong*: a
        // piston clears its move sources with `set_quiet`, which writes no
        // `BlockChange`, so a replay leaves blocks standing where the engine
        // has none. Diffs built that way accuse the wrong cells. Dump the
        // world and compare that.
        if std::env::var("MC_DUMP_AT").ok().and_then(|v| v.parse::<u64>().ok()) == Some(t) {
            let mut cells: Vec<_> = sim.world().iter_non_air().collect();
            cells.sort_by_key(|(p, _)| (p.x, p.y, p.z));
            for (p, id) in cells {
                println!(
                    "DUMP [{},{},{}] {}",
                    p.x, p.y, p.z,
                    sim.registry().descriptor(id).unwrap_or("?")
                );
            }
        }
        if !dump_changes {
            // One line per tick that did anything, and only when the picture
            // changed — a machine that is running steadily says so in a dozen
            // lines instead of ten thousand.
            let new = sim.recorded().len() - seen;
            seen = sim.recorded().len();
            if new > 0 {
                if let Some((lo, hi)) = footprint(&sim) {
                    let line = format!(
                        "x {}..{}  y {}..{}  z {}..{}  ({} non-air)",
                        lo.x, hi.x, lo.y, hi.y, lo.z, hi.z,
                        sim.world().non_air_count()
                    );
                    if last_print.as_deref() != Some(line.as_str()) {
                        eprintln!("  t{t:<4} {new:>4} change(s)   {line}");
                        last_print = Some(line);
                    }
                }
            }
        }
    }
    if dump_changes {
        for c in sim.recorded() {
            eprintln!(
                "  t{:<3} [{},{},{}] {} -> {}",
                c.tick, c.pos.x, c.pos.y, c.pos.z,
                sim.registry().descriptor(c.from).unwrap_or("?").replace("minecraft:", ""),
                sim.registry().descriptor(c.to).unwrap_or("?").replace("minecraft:", ""),
            );
        }
    }
    if let Some((lo, hi)) = footprint(&sim) {
        eprintln!(
            "=== final footprint x {}..{} y {}..{} z {}..{}",
            lo.x, hi.x, lo.y, hi.y, lo.z, hi.z
        );
    }
    // A `moving_piston` left standing is the signature of a machine that
    // stalled rather than finished: the placeholder never resolved, and being
    // immovable it then blocks every push through that cell.
    let stuck: Vec<Pos> = sim
        .world()
        .iter_non_air()
        .filter(|(_, id)| {
            sim.registry().descriptor(*id).is_some_and(|d| d.contains("moving_piston"))
        })
        .map(|(p, _)| p)
        .collect();
    if !stuck.is_empty() {
        eprintln!("=== {} unresolved moving_piston cell(s):", stuck.len());
        for p in stuck.iter().take(12) {
            eprintln!("      [{},{},{}]", p.x, p.y, p.z);
        }
    }
    if !read_at.is_empty() {
        eprintln!("=== read:");
        for (x, y, z) in &read_at {
            let state = sim.world().get(Pos::new(*x, *y, *z));
            eprintln!(
                "  READ [{x},{y},{z}] {}",
                sim.registry().descriptor(state).unwrap_or("?")
            );
        }
    }
    eprintln!("=== {} change(s) total over {ticks} ticks", sim.recorded().len());
    panic!("probe output above");
}
