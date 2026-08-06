//! Static structural analysis: the canonical engine, and the soundness of every
//! rejection.
//!
//! The rejections are the whole point of this module, and the only property that
//! matters for them is that they are never wrong. So the centrepiece here is not
//! a golden value — it is a sweep over a few thousand candidate machines that
//! are *simulated* to see which ones move, then filtered, with a hard assertion
//! that the filter rejected none of the movers.

use std::collections::BTreeSet;

use mc_tick::machine_graph::{analyse, DeviceKind, MachineGraph};
use mc_tick::{Pos, Simulation, Structure};

/// Build a world from SNBT and run the static analysis over it.
fn graph_of(snbt: &str) -> MachineGraph {
    let structure = Structure::parse(snbt).expect("structure parses");
    let mut sim = Simulation::new(structure.bounds(8));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    mc_tick::intern_companions(sim.registry_mut());
    let rules = {
        let mut table = std::mem::take(sim.behaviours_mut());
        let rules = mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
        *sim.behaviours_mut() = table;
        rules
    };
    analyse(sim.world(), sim.registry(), &rules)
}

fn corpus(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/corpus/structures/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("corpus structure")
}

fn cell_set(cells: &[Pos]) -> BTreeSet<(i32, i32, i32)> {
    cells.iter().map(|p| (p.x, p.y, p.z)).collect()
}

/* ------------------------------------------------------- the canonical pair */

/// The 6x1x1 wiki engine is canonical *because* it is minimal: every one of its
/// six blocks is engine, and it carries nothing.
#[test]
fn canonical_engine_is_the_whole_six_block_machine() {
    let graph = graph_of(&corpus("flying_machine.snbt"));
    assert!(
        !graph.rejected(),
        "the canonical flying machine must not be rejected"
    );
    assert_eq!(
        graph.engines.len(),
        1,
        "one engine, not {}",
        graph.engines.len()
    );

    let engine = cell_set(&graph.engines[0].cells);
    let expected: BTreeSet<(i32, i32, i32)> = (0..6).map(|x| (x, 0, 0)).collect();
    assert_eq!(engine, expected, "engine must be all six blocks");
    assert!(
        graph.payload.is_empty(),
        "a minimal engine carries nothing: {:?}",
        graph.payload
    );
    assert!(
        graph.dead_weight.is_empty(),
        "no dead weight: {:?}",
        graph.dead_weight
    );
}

/// The other half of the canonical test: bolt cargo on and the *engine must not
/// move*. If the engine drifts when only the payload changed, the definition is
/// measuring the wrong thing.
#[test]
fn engine_holds_fixed_while_the_payload_grows() {
    // The canonical row, plus two carried blocks east of it. One is slime, so it
    // joins the engine's adhesion group — the case that catches a definition
    // built on groups rather than on connectivity between drivers.
    let with_cargo = "{DataVersion: 4903, size: [8, 1, 1], palette: [\
        {Name: \"minecraft:slime_block\"}, \
        {Name: \"minecraft:sticky_piston\", Properties: {facing: \"east\", extended: \"false\"}}, \
        {Name: \"minecraft:sticky_piston\", Properties: {facing: \"west\", extended: \"false\"}}, \
        {Name: \"minecraft:observer\", Properties: {facing: \"east\", powered: \"false\"}}, \
        {Name: \"minecraft:observer\", Properties: {facing: \"west\", powered: \"false\"}}, \
        {Name: \"minecraft:white_concrete\"}], \
        blocks: [\
        {pos: [0, 0, 0], state: 4}, {pos: [1, 0, 0], state: 0}, {pos: [2, 0, 0], state: 1}, \
        {pos: [3, 0, 0], state: 2}, {pos: [4, 0, 0], state: 0}, {pos: [5, 0, 0], state: 3}, \
        {pos: [6, 0, 0], state: 0}, {pos: [7, 0, 0], state: 5}], \
        entities: []}";

    let bare = graph_of(&corpus("flying_machine.snbt"));
    let loaded = graph_of(with_cargo);

    assert!(
        !loaded.rejected(),
        "a loaded flying machine must not be rejected"
    );
    assert_eq!(loaded.engines.len(), 1);

    let bare_engine = cell_set(&bare.engines[0].cells);
    let loaded_engine = cell_set(&loaded.engines[0].cells);
    assert_eq!(
        loaded_engine, bare_engine,
        "the engine changed when only the payload did"
    );

    let payload = cell_set(&loaded.payload);
    assert_eq!(
        payload,
        BTreeSet::from([(6, 0, 0), (7, 0, 0)]),
        "the two added blocks are payload"
    );
}

/// The east-facing capture is the same machine, and must classify the same way.
#[test]
fn the_east_variant_classifies_as_one_engine_too() {
    let graph = graph_of(&corpus("flying_machine_east.snbt"));
    assert!(
        !graph.rejected(),
        "flying_machine_east must not be rejected"
    );
    assert!(
        !graph.engines.is_empty(),
        "flying_machine_east has an engine"
    );
}

/* ------------------------------------------------------------- the graph */

/// Slime and honey do not stick to each other, and the graph must say so.
#[test]
fn honey_and_slime_are_separate_groups() {
    let snbt = "{DataVersion: 4903, size: [4, 1, 1], palette: [\
        {Name: \"minecraft:slime_block\"}, {Name: \"minecraft:honey_block\"}], \
        blocks: [{pos: [0, 0, 0], state: 0}, {pos: [1, 0, 0], state: 1}], entities: []}";
    let graph = graph_of(snbt);
    assert_eq!(
        graph.groups.len(),
        2,
        "slime and honey must not share a group"
    );
}

/// A piston's `pushes` edge comes from `resolve_push`, so the twelve-block limit
/// is inherited rather than re-implemented.
#[test]
fn the_push_limit_is_inherited_not_reimplemented() {
    // A piston with thirteen solid blocks ahead of it cannot extend.
    let mut blocks = String::from("{pos: [0, 0, 0], state: 0}");
    for x in 1..=13 {
        blocks.push_str(&format!(", {{pos: [{x}, 0, 0], state: 1}}"));
    }
    let snbt = format!(
        "{{DataVersion: 4903, size: [20, 1, 1], palette: [\
        {{Name: \"minecraft:piston\", Properties: {{facing: \"east\", extended: \"false\"}}}}, \
        {{Name: \"minecraft:white_concrete\"}}], blocks: [{blocks}], entities: []}}"
    );
    let graph = graph_of(&snbt);
    let piston = graph
        .devices
        .iter()
        .find(|d| matches!(d.kind, DeviceKind::Piston { .. }))
        .expect("one piston");
    assert!(!piston.can_extend, "thirteen blocks is over MAX_PUSH_DEPTH");
    assert!(
        graph
            .rejections
            .iter()
            .any(|r| r.code == "all_pistons_blocked"),
        "a machine whose only piston is over the limit is provably immobile"
    );
}

/// A piston aimed at obsidian in a build with no other piston is immobile.
#[test]
fn a_piston_facing_immovable_is_rejected() {
    let snbt = "{DataVersion: 4903, size: [6, 1, 1], palette: [\
        {Name: \"minecraft:piston\", Properties: {facing: \"east\", extended: \"false\"}}, \
        {Name: \"minecraft:obsidian\"}, \
        {Name: \"minecraft:observer\", Properties: {facing: \"west\", powered: \"false\"}}], \
        blocks: [{pos: [1, 0, 0], state: 0}, {pos: [2, 0, 0], state: 1}, \
        {pos: [0, 0, 0], state: 2}], entities: []}";
    let graph = graph_of(snbt);
    assert!(graph.rejected(), "a piston that cannot extend cannot fly");
    assert!(graph
        .rejections
        .iter()
        .any(|r| r.code == "all_pistons_blocked"));
}

/// A build with no piston at all cannot move.
#[test]
fn a_pistonless_build_is_rejected() {
    let snbt = "{DataVersion: 4903, size: [6, 1, 1], palette: [\
        {Name: \"minecraft:slime_block\"}, \
        {Name: \"minecraft:observer\", Properties: {facing: \"east\", powered: \"false\"}}], \
        blocks: [{pos: [0, 0, 0], state: 0}, {pos: [1, 0, 0], state: 1}], entities: []}";
    let graph = graph_of(snbt);
    assert!(graph.rejections.iter().any(|r| r.code == "no_piston"));
    assert!(graph.rejected());
}

/// A piston with no observer and no power source is never told to fire.
#[test]
fn a_driverless_piston_is_rejected() {
    let snbt = "{DataVersion: 4903, size: [8, 1, 1], palette: [\
        {Name: \"minecraft:sticky_piston\", Properties: {facing: \"east\", extended: \"false\"}}, \
        {Name: \"minecraft:slime_block\"}], \
        blocks: [{pos: [0, 0, 0], state: 0}, {pos: [1, 0, 0], state: 1}], entities: []}";
    let graph = graph_of(snbt);
    assert!(graph.rejections.iter().any(|r| r.code == "no_driver"));
}

/* ------------------------------------------------------------ soundness */

/// The GA's own fitness for one machine, computed in-engine.
///
/// Deliberately not "did anything change" — it is `evalCore`'s number: signed
/// centre-of-mass displacement along +x, minus the debris penalty that docks a
/// machine for leaving its tail behind. Matching it exactly is the whole point:
/// a false reject is only a false reject against the score the search actually
/// uses, and a west-facing piston poking its head out is *negative* displacement
/// under this measure, not flight.
fn ga_fitness(snbt: &str, kicks: &[Pos], ticks: u64) -> (f64, bool) {
    let structure = match Structure::parse(snbt) {
        Ok(s) => s,
        Err(_) => return (0.0, false),
    };
    let mut best: f64 = 0.0;
    let mut best_sustained = false;
    // `None` is the placement transient — every observer pulses once as the
    // build lands, which is the standard flying-machine starter and the protocol
    // the oracle-verified corpus capture uses. The rest are explicit kicks.
    let starts: Vec<Option<Pos>> = std::iter::once(None)
        .chain(kicks.iter().map(|&k| Some(k)))
        .collect();
    // A *corridor*, not a cube. `Structure::bounds(margin)` grows all three axes,
    // and a margin wide enough to fly down turns a 6x1x1 machine into a
    // hundred-thousand-cell world that takes longer to allocate than to tick.
    let inner = structure.bounds(0);
    let corridor = mc_tick::Bounds::new(
        Pos::new(inner.min.x - 8, inner.min.y - 3, inner.min.z - 3),
        Pos::new(inner.max.x + 60, inner.max.y + 3, inner.max.z + 3),
    );
    for start_kick in starts {
        let mut sim = Simulation::new(corridor);
        {
            let (registry, world) = sim.registry_and_world_mut();
            structure.place(world, registry, Pos::new(0, 0, 0));
        }
        let Ok(rb) = sim.registry_mut().intern("minecraft:redstone_block") else {
            continue;
        };
        mc_tick::intern_companions(sim.registry_mut());
        {
            let mut table = std::mem::take(sim.behaviours_mut());
            mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
            *sim.behaviours_mut() = table;
        }
        if sim.unknown_report().is_some() {
            continue;
        }
        let order = structure.placement_order(
            mc_tick::vanilla::is_collision_full_cube,
            mc_tick::vanilla::has_dynamic_shape,
        );
        sim.place_on_place(&order);
        let kick = start_kick.unwrap_or(Pos::new(i32::MIN / 2, i32::MIN / 2, i32::MIN / 2));
        if start_kick.is_none() {
            sim.settle_with_order(&order);
        }
        let Some((start_com, start_min, _, _)) = census(&sim, kick) else {
            continue;
        };
        let mut mid_com = start_com;
        for t in 0..ticks {
            if start_kick.is_some() {
                if t == 2 {
                    sim.place_block(kick, rb);
                }
                if t == 4 {
                    sim.place_block(kick, mc_tick::StateId::AIR);
                }
            }
            sim.step();
            if t == ticks / 2 {
                if let Some((c, _, _, _)) = census(&sim, kick) {
                    mid_com = c;
                }
            }
        }
        let Some((end_com, end_min, end_max, n1)) = census(&sim, kick) else {
            continue;
        };
        let disp = end_com - start_com;
        let mut penalty = 0.0;
        if disp > 0.5 && f64::from(end_min) < f64::from(start_min) + disp / 2.0 {
            let travel = f64::from(end_max - start_min);
            if travel > 0.5 {
                penalty = (f64::from(n1) * (1.0 - disp / travel)).max(0.0).round();
            }
        }
        let fit = (disp - penalty).max(0.0);
        // `metrics.ts`: kept propelling itself through the second half.
        let late = end_com - mid_com;
        let sustained = fit > 0.5 && late >= 1.0 && late >= 0.25 * disp;
        if fit > best {
            best = fit;
        }
        best_sustained |= sustained;
    }
    (best, best_sustained)
}

/// `(centre x, min x, max x, count)` over every non-air cell but the kick.
fn census(sim: &Simulation, ignore: Pos) -> Option<(f64, i32, i32, u32)> {
    let xs: Vec<i32> = sim
        .world()
        .iter_non_air()
        .filter(|(p, _)| *p != ignore)
        .map(|(p, _)| p.x)
        .collect();
    if xs.is_empty() {
        return None;
    }
    let n = xs.len();
    Some((
        f64::from(xs.iter().sum::<i32>()) / n as f64,
        *xs.iter().min().unwrap(),
        *xs.iter().max().unwrap(),
        n as u32,
    ))
}

/// Every state the GA's alphabet can place, in one row.
const ALPHABET: &[&str] = &[
    "",
    "minecraft:slime_block",
    "minecraft:honey_block",
    "minecraft:sticky_piston[extended=false,facing=east]",
    "minecraft:sticky_piston[extended=false,facing=west]",
    "minecraft:piston[extended=false,facing=east]",
    "minecraft:piston[extended=false,facing=west]",
    "minecraft:observer[facing=east,powered=false]",
    "minecraft:observer[facing=west,powered=false]",
    "minecraft:white_concrete",
];

fn row_snbt(row: &[usize], width: i32) -> String {
    grid_snbt(std::slice::from_ref(&row), width)
}

/// SNBT for a `z`-indexed stack of rows. Blocks start at x = 2 so a kick has
/// somewhere to stand on either side.
fn grid_snbt<R: AsRef<[usize]>>(rows: &[R], width: i32) -> String {
    let mut palette: Vec<String> = Vec::new();
    let mut blocks: Vec<String> = Vec::new();
    let depth = rows.len().max(1) as i32;
    for (z, row) in rows.iter().enumerate() {
        for (x, &cell) in row.as_ref().iter().enumerate() {
            if cell == 0 {
                continue;
            }
            let descriptor = ALPHABET[cell];
            let (name, props) = match descriptor.split_once('[') {
                Some((n, p)) => (n, Some(p.trim_end_matches(']'))),
                None => (descriptor, None),
            };
            let entry = match props {
                Some(p) => {
                    let kvs: Vec<String> = p
                        .split(',')
                        .map(|kv| {
                            let (k, v) = kv.split_once('=').unwrap();
                            format!("{k}: \"{v}\"")
                        })
                        .collect();
                    format!("{{Name: \"{name}\", Properties: {{{}}}}}", kvs.join(", "))
                }
                None => format!("{{Name: \"{name}\"}}"),
            };
            let index = match palette.iter().position(|e| *e == entry) {
                Some(i) => i,
                None => {
                    palette.push(entry);
                    palette.len() - 1
                }
            };
            blocks.push(format!("{{pos: [{}, 0, {z}], state: {index}}}", x + 2));
        }
    }
    format!(
        "{{DataVersion: 4903, size: [{width}, 1, {depth}], palette: [{}], blocks: [{}], \
         entities: []}}",
        palette.join(", "),
        blocks.join(", ")
    )
}

/// **The soundness test.** A rejection is a claim no simulation can contradict.
///
/// Sweeps a deterministic slice of the GA's own search space, simulates every
/// candidate, and asserts that the filter rejected none of the ones that moved.
/// A single false reject fails this test: a filter that throws away a working
/// machine is worse than no filter, because the search never learns what it lost.
#[test]
fn no_rejection_ever_discards_a_machine_that_moves() {
    const LEN: usize = 5;
    const WIDTH: i32 = LEN as i32 + 3;
    // A fixed stride over the full 10^5 product keeps the sweep deterministic,
    // wide, and cheap enough to sit in the default suite.
    const STRIDE: usize = 11;

    let total = ALPHABET.len().pow(LEN as u32);
    let mut moved = 0usize;
    let mut sustained_count = 0usize;
    let mut rejected_movers: Vec<String> = Vec::new();
    let mut sustained_false_rejects: Vec<String> = Vec::new();
    let mut rejected_total = 0usize;
    let mut sustained_rejected_total = 0usize;
    let mut checked = 0usize;

    let mut index = 0usize;
    while index < total {
        let mut row = [0usize; LEN];
        let mut rest = index;
        for slot in row.iter_mut() {
            *slot = rest % ALPHABET.len();
            rest /= ALPHABET.len();
        }
        index += STRIDE;
        if row.iter().all(|&c| c == 0) {
            continue;
        }
        checked += 1;
        let snbt = row_snbt(&row, WIDTH);
        let graph = graph_of(&snbt);
        let verdict = graph.rejected();
        let sustained_verdict = graph.rejected_for_sustained();
        if verdict {
            rejected_total += 1;
        }
        if sustained_verdict {
            sustained_rejected_total += 1;
        }
        // Only simulate what one of the two filters would have thrown away, plus
        // a control sample — simulating all of them would make this test minutes
        // long.
        if sustained_verdict || checked.is_multiple_of(11) {
            // Every cell above the row is a candidate kick: a machine that flies
            // from *any* start counts as flying, which makes the filter's job
            // harder, not easier.
            let kicks: Vec<Pos> = (0..LEN as i32).map(|i| Pos::new(i + 2, 1, 0)).collect();
            let (fit, sustained) = ga_fitness(&snbt, &kicks, 80);
            let names: Vec<&str> = row.iter().map(|&c| ALPHABET[c]).map(short).collect();
            let codes = graph.rejections.iter().map(|r| r.code).collect::<Vec<_>>();
            if fit > 0.5 {
                moved += 1;
                if verdict {
                    rejected_movers.push(format!(
                        "[{}] scored {fit:.2} but was rejected outright: {codes:?}",
                        names.join(",")
                    ));
                }
            }
            if sustained {
                sustained_count += 1;
                if sustained_verdict {
                    sustained_false_rejects.push(format!(
                        "[{}] sustained flight at {fit:.2} but was rejected: {codes:?}",
                        names.join(",")
                    ));
                }
            }
        }
    }

    println!(
        "swept {checked} candidates\n  unconditional tier: {rejected_total} rejected, \
         {moved} simulated machines scored > 0.5, {} false rejects\n  sustained tier: \
         {sustained_rejected_total} rejected, {sustained_count} sustained fliers, {} false rejects",
        rejected_movers.len(),
        sustained_false_rejects.len()
    );
    assert!(
        rejected_movers.is_empty(),
        "the unconditional filter discarded {} machine(s) that score above the GA's \
         flight threshold:\n{}",
        rejected_movers.len(),
        rejected_movers.join("\n")
    );
    assert!(
        sustained_false_rejects.is_empty(),
        "the sustained filter discarded {} machine(s) that sustain flight:\n{}",
        sustained_false_rejects.len(),
        sustained_false_rejects.join("\n")
    );
    assert!(
        rejected_total > 0,
        "a filter that never rejects anything is not being tested"
    );
}

/// The harness must be able to fly a machine that is known to fly — otherwise a
/// sweep reporting "no fliers" is measuring the kick protocol, not the filter.
///
/// `flying_machine_east` is the directional engine-B capture and travels exactly
/// one block per ten ticks. `flying_machine` is its symmetric sibling: the same
/// six parts, but with the two observers pointing outward, it cycles without
/// going anywhere. Both must survive both filters — the second one is the more
/// interesting case, because a filter that only kept *fliers* would be free to
/// throw it away, and a filter that is sound must not.
#[test]
fn known_good_machines_survive_both_filters() {
    let kicks: Vec<Pos> = (-1..=6)
        .flat_map(|x| [Pos::new(x, 0, 0), Pos::new(x, 1, 0)])
        .collect();

    let east = corpus("flying_machine_east.snbt");
    let (fit, sustained) = ga_fitness(&east, &kicks, 100);
    assert!(
        fit > 9.0,
        "engine B travels a block every ten ticks; scored {fit:.2}"
    );
    assert!(sustained, "engine B sustains flight");

    for name in ["flying_machine.snbt", "flying_machine_east.snbt"] {
        let graph = graph_of(&corpus(name));
        assert!(
            !graph.rejected(),
            "{name} rejected outright: {:?}",
            graph.rejections
        );
        assert!(
            !graph.rejected_for_sustained(),
            "{name} rejected by the sustained filter: {:?}",
            graph.rejections
        );
    }
}

/// The sustained tier, tested against a population that actually contains
/// sustained fliers.
///
/// The five-wide sweep above is too narrow for one: a self-propelling engine
/// needs two pistons and an observer, which is six blocks. This sweep is six
/// wide over the engine alphabet only, and every candidate is simulated. Without
/// it the sustained tier's zero-false-reject result would be vacuously true —
/// nothing it could have got wrong was in the sample.
#[test]
fn the_sustained_filter_keeps_every_machine_that_sustains_flight() {
    // Engine B, the two-row layout the corpus capture uses, laid out on a 2x4
    // grid. Row z=0 runs obsW, slime, stickyW; row z=1 runs stickyE, slime, obsE
    // one cell east. A one-row sweep cannot contain a flier at all, which is why
    // an earlier version of this test was measuring nothing.
    const W: usize = 4;
    const ENGINE_B: [[usize; W]; 2] = [[8, 1, 4, 0], [0, 3, 1, 7]];
    // air, slime, sticky east/west, observer east/west.
    const PARTS: [usize; 6] = [0, 1, 3, 4, 7, 8];

    let mut population: Vec<[[usize; W]; 2]> = vec![ENGINE_B];
    // Every single-cell mutation of a known flier. This is the population that
    // matters: right on the boundary, where a filter that is subtly wrong will
    // reject a machine that still works.
    for z in 0..2 {
        for x in 0..W {
            for &part in &PARTS {
                let mut m = ENGINE_B;
                if m[z][x] == part {
                    continue;
                }
                m[z][x] = part;
                population.push(m);
            }
        }
    }
    // A strided uniform sample of the same 2x4 space, for breadth.
    let total = PARTS.len().pow((2 * W) as u32);
    let mut index = 0usize;
    while index < total {
        let mut rest = index;
        index += 379;
        let mut grid = [[0usize; W]; 2];
        for row in grid.iter_mut() {
            for slot in row.iter_mut() {
                *slot = PARTS[rest % PARTS.len()];
                rest /= PARTS.len();
            }
        }
        let flat: Vec<usize> = grid.iter().flatten().copied().collect();
        if !flat.iter().any(|&c| (3..=6).contains(&c))
            || !flat.iter().any(|&c| (7..=8).contains(&c))
        {
            continue;
        }
        population.push(grid);
    }

    let kicks: Vec<Pos> = (0..W as i32)
        .flat_map(|x| [Pos::new(x + 2, 1, 0), Pos::new(x + 2, 1, 1)])
        .collect();

    let mut sustained_fliers = 0usize;
    let mut movers = 0usize;
    let mut false_rejects: Vec<String> = Vec::new();
    let mut sustained_false_rejects: Vec<String> = Vec::new();
    let mut rejected = 0usize;
    let checked = population.len();

    for grid in &population {
        let snbt = grid_snbt(grid, W as i32 + 3);
        let graph = graph_of(&snbt);
        if graph.rejected_for_sustained() {
            rejected += 1;
        }
        let (fit, sustained) = ga_fitness(&snbt, &kicks, 100);
        let names: Vec<&str> = grid
            .iter()
            .flatten()
            .map(|&c| ALPHABET[c])
            .map(short)
            .collect();
        let codes = graph.rejections.iter().map(|r| r.code).collect::<Vec<_>>();
        if fit > 0.5 {
            movers += 1;
            if graph.rejected() {
                false_rejects.push(format!(
                    "[{}] scored {fit:.2}, rejected: {codes:?}",
                    names.join(",")
                ));
            }
        }
        if sustained {
            sustained_fliers += 1;
            if graph.rejected_for_sustained() {
                sustained_false_rejects.push(format!(
                    "[{}] sustains flight at {fit:.2}, rejected: {codes:?}",
                    names.join(",")
                ));
            }
        }
    }

    println!(
        "engine sweep: {checked} candidates, {rejected} rejected for sustained, \
         {movers} moved, {sustained_fliers} sustained, {} / {} false rejects",
        false_rejects.len(),
        sustained_false_rejects.len()
    );
    assert!(
        sustained_fliers > 0,
        "this sweep found no sustained fliers, so it proves nothing about the filter"
    );
    assert!(
        false_rejects.is_empty(),
        "unconditional filter discarded movers:\n{}",
        false_rejects.join("\n")
    );
    assert!(
        sustained_false_rejects.is_empty(),
        "the sustained filter discarded {} machine(s) that sustain flight:\n{}",
        sustained_false_rejects.len(),
        sustained_false_rejects.join("\n")
    );
}

fn short(state: &str) -> &str {
    if state.is_empty() {
        return "_";
    }
    state
        .split_once('[')
        .map_or(state, |(n, _)| n)
        .trim_start_matches("minecraft:")
}
