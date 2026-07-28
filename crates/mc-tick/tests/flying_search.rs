//! Exploratory search for self-propelled flying machines — run manually:
//!
//!     cargo test -p mc-tick --test flying_search -- --ignored --nocapture
//!
//! Enumerates 1-wide rows (and a few 2-tall variants) of piston/observer/slime
//! blocks in a long air corridor, settles placement (every observer pulses once
//! at placement — the standard flying-machine starter), runs 80 ticks, and
//! reports candidates whose whole block set travels east without splitting.

use mc_tick::{Pos, Simulation, Structure};

const CORRIDOR: i32 = 40;

/// One candidate cell.
#[derive(Clone, Copy, PartialEq)]
enum Cell {
    SpE, // sticky piston facing east
    SpW,
    PE, // regular piston facing east
    ObE, // observer facing east (watching +x)
    ObW,
    Slime,
    Honey,
}

impl Cell {
    fn descriptor(self) -> &'static str {
        match self {
            Cell::SpE => "minecraft:sticky_piston[extended=false,facing=east]",
            Cell::SpW => "minecraft:sticky_piston[extended=false,facing=west]",
            Cell::PE => "minecraft:piston[extended=false,facing=east]",
            Cell::ObE => "minecraft:observer[facing=east,powered=false]",
            Cell::ObW => "minecraft:observer[facing=west,powered=false]",
            Cell::Slime => "minecraft:slime_block",
            Cell::Honey => "minecraft:honey_block",
        }
    }
    fn tag(self) -> &'static str {
        match self {
            Cell::SpE => "SpE",
            Cell::SpW => "SpW",
            Cell::PE => "PE",
            Cell::ObE => "ObE",
            Cell::ObW => "ObW",
            Cell::Slime => "S",
            Cell::Honey => "H",
        }
    }
}

fn snbt_for(rows: &[&[Cell]]) -> String {
    // rows[y] is the row at height y; each row starts at x=0.
    let mut palette: Vec<String> = Vec::new();
    let mut blocks = String::new();
    for (y, row) in rows.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let descriptor = cell.descriptor();
            let (name, props) = match descriptor.split_once('[') {
                Some((n, p)) => (n, Some(p.trim_end_matches(']'))),
                None => (descriptor, None),
            };
            let palette_entry = match props {
                Some(p) => {
                    let props_snbt = p
                        .split(',')
                        .map(|kv| {
                            let (k, v) = kv.split_once('=').unwrap();
                            format!("{k}: \"{v}\"")
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{{Name: \"{name}\", Properties: {{{props_snbt}}}}}")
                }
                None => format!("{{Name: \"{name}\"}}"),
            };
            let entry = match palette.iter().position(|e| *e == palette_entry) {
                Some(i) => i,
                None => {
                    palette.push(palette_entry.clone());
                    palette.len() - 1
                }
            };
            blocks.push_str(&format!("{{pos: [{x}, {y}, 0], state: {entry}}}, "));
        }
    }
    let height = rows.len();
    format!(
        "{{DataVersion: 4903, size: [{CORRIDOR}, {height}, 1], palette: [{}], blocks: [{}], entities: []}}",
        palette.join(", "),
        blocks.trim_end_matches(", ")
    )
}

/// Settle, run, and score: (min_x displacement, span at end, block count at end).
fn evaluate(snbt: &str, ticks: u64) -> (i32, i32, usize) {
    let structure = Structure::parse(snbt).expect("candidate parses");
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
        *sim.behaviours_mut() = table;
    }
    if sim.unknown_report().is_some() {
        return (0, 0, 0);
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    sim.settle_with_order(&order);
    for _ in 0..ticks {
        sim.step();
    }
    let cells: Vec<Pos> = sim.world().iter_non_air().map(|(p, _)| p).collect();
    if cells.is_empty() {
        return (0, 0, 0);
    }
    let min_x = cells.iter().map(|p| p.x).min().unwrap();
    let max_x = cells.iter().map(|p| p.x).max().unwrap();
    (min_x, max_x - min_x + 1, cells.len())
}

/// Like `evaluate`, but with a redstone-block kick placed then removed.
fn evaluate_with_kick(
    snbt: &str,
    kick: Pos,
    kick_on: u64,
    kick_off: u64,
    ticks: u64,
) -> (i32, i32, usize) {
    let structure = Structure::parse(snbt).expect("candidate parses");
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    let rb = sim.registry_mut().intern("minecraft:redstone_block").unwrap();
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
        *sim.behaviours_mut() = table;
    }
    if sim.unknown_report().is_some() {
        return (0, 0, 0);
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    sim.settle_with_order(&order);
    for t in 0..ticks {
        if t == kick_on {
            sim.place_block(kick, rb);
        }
        if t == kick_off {
            sim.place_block(kick, mc_tick::StateId::AIR);
        }
        sim.step();
    }
    let cells: Vec<Pos> = sim
        .world()
        .iter_non_air()
        .filter(|(p, _)| *p != kick)
        .map(|(p, _)| p)
        .collect();
    if cells.is_empty() {
        return (0, 0, 0);
    }
    let min_x = cells.iter().map(|p| p.x).min().unwrap();
    let max_x = cells.iter().map(|p| p.x).max().unwrap();
    (min_x, max_x - min_x + 1, cells.len())
}

/// Quiet placement (no settle transient), then a stone blink at `kick` on
/// ticks 2..4 — the player's "place the last block" start.
fn evaluate_quiet_kicked(snbt: &str, kick: Pos, ticks: u64) -> (i32, i32, usize) {
    let structure = Structure::parse(snbt).expect("candidate parses");
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    let stone = sim.registry_mut().intern("minecraft:stone").unwrap();
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
        *sim.behaviours_mut() = table;
    }
    if sim.unknown_report().is_some() {
        return (0, 0, 0);
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    for t in 0..ticks {
        if t == 2 {
            sim.place_block(kick, stone);
        }
        if t == 4 {
            sim.place_block(kick, mc_tick::StateId::AIR);
        }
        sim.step();
    }
    let cells: Vec<Pos> = sim
        .world()
        .iter_non_air()
        .filter(|(p, _)| *p != kick)
        .map(|(p, _)| p)
        .collect();
    if cells.is_empty() {
        return (0, 0, 0);
    }
    let min_x = cells.iter().map(|p| p.x).min().unwrap();
    let max_x = cells.iter().map(|p| p.x).max().unwrap();
    (min_x, max_x - min_x + 1, cells.len())
}

/// Primitive: an extended sticky piston retracting must PULL the block that
/// sits in front of its head — the pull stroke every flying machine rests on.
#[test]
fn sticky_retraction_pulls_the_block_ahead() {
    // slime at x0; sticky piston base at x2 facing west. A redstone block at
    // (2,1,0) over the piston powers it: extend puts the head at x1, adjacent
    // to the slime. Removing the power retracts: the head withdraws and the
    // slime must come east with it, landing at x1.
    let snbt = "{DataVersion: 4903, size: [8, 2, 1], palette: [\
        {Name: \"minecraft:slime_block\"}, \
        {Name: \"minecraft:sticky_piston\", Properties: {extended: \"false\", facing: \"west\"}}], \
        blocks: [{pos: [0, 0, 0], state: 0}, {pos: [2, 0, 0], state: 1}], entities: []}";
    let (min_x, span, count) = evaluate_with_kick(snbt, Pos::new(2, 1, 0), 2, 8, 20);
    assert_eq!(
        (min_x, span, count),
        (1, 2, 2),
        "slime pulled from x0 to x1 beside the piston base at x2"
    );
}

/// Air in a row: the cell is simply skipped when building the structure.
fn snbt_for_sparse(rows: &[&[Option<Cell>]]) -> String {
    let dense: Vec<Vec<Cell>> = Vec::new();
    let _ = dense;
    let mut palette: Vec<String> = Vec::new();
    let mut blocks = String::new();
    for (y, row) in rows.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let Some(cell) = cell else { continue };
            let descriptor = cell.descriptor();
            let (name, props) = match descriptor.split_once('[') {
                Some((n, p)) => (n, Some(p.trim_end_matches(']'))),
                None => (descriptor, None),
            };
            let palette_entry = match props {
                Some(p) => {
                    let props_snbt = p
                        .split(',')
                        .map(|kv| {
                            let (k, v) = kv.split_once('=').unwrap();
                            format!("{k}: \"{v}\"")
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{{Name: \"{name}\", Properties: {{{props_snbt}}}}}")
                }
                None => format!("{{Name: \"{name}\"}}"),
            };
            let entry = match palette.iter().position(|e| *e == palette_entry) {
                Some(i) => i,
                None => {
                    palette.push(palette_entry.clone());
                    palette.len() - 1
                }
            };
            blocks.push_str(&format!("{{pos: [{x}, {y}, 0], state: {entry}}}, "));
        }
    }
    let height = rows.len();
    format!(
        "{{DataVersion: 4903, size: [{CORRIDOR}, {height}, 1], palette: [{}], blocks: [{}], entities: []}}",
        palette.join(", "),
        blocks.trim_end_matches(", ")
    )
}

fn is_piston(c: Cell) -> bool {
    matches!(c, Cell::SpE | Cell::SpW | Cell::PE)
}
fn is_observer(c: Cell) -> bool {
    matches!(c, Cell::ObE | Cell::ObW)
}

fn run_candidates(candidates: Vec<Vec<Vec<Option<Cell>>>>, label: &str) -> usize {
    let total = candidates.len();
    let workers = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(10);
    let chunk = total.div_ceil(workers);
    let winners = std::sync::Mutex::new(Vec::<String>::new());
    let winners_ref = &winners;
    std::thread::scope(|scope| {
        for part in candidates.chunks(chunk.max(1)) {
            scope.spawn(move || {
                for rows in part {
                    let borrowed: Vec<&[Option<Cell>]> =
                        rows.iter().map(|r| r.as_slice()).collect();
                    let snbt = snbt_for_sparse(&borrowed);
                    let blocks: usize = rows.iter().flatten().flatten().count();
                    let len = rows[0].len();
                    // Quiet placement + a kick: a player builds the machine
                    // inert and starts it with one block update. Try a stone
                    // blink at the front face and at the rear face.
                    let front = Pos::new(len as i32, 0, 0);
                    let rear = Pos::new(-1, 0, 0);
                    let (min_x, span, count) = [front, rear]
                        .into_iter()
                        .map(|kick| evaluate_quiet_kicked(&snbt, kick, 80))
                        .max_by_key(|(m, _, _)| *m)
                        .unwrap();
                    if min_x >= 3 && span <= len as i32 + 2 && count >= blocks {
                        let mut desc = String::new();
                        for row in rows {
                            let tags: Vec<&str> =
                                row.iter().map(|c| c.map_or("_", Cell::tag)).collect();
                            desc.push_str(&format!("[{}]", tags.join(",")));
                        }
                        winners_ref.lock().unwrap().push(format!(
                            "WINNER {desc} min_x={min_x} span={span} count={count}"
                        ));
                    }
                }
            });
        }
    });
    let found = winners.into_inner().unwrap();
    for w in &found {
        println!("{w}");
    }
    println!("-- {label}: {} winners / {total} candidates", found.len());
    found.len()
}

/// Primitive: an observer MOVED by a piston must pulse when it lands — the
/// heartbeat of every flying machine. A pushed observer (output facing east)
/// lands beside a lamp; the landing pulse must light it.
#[test]
fn a_moved_observer_pulses_on_landing() {
    // x0: piston facing east. x1: observer facing west (output east). x3: lamp.
    // Kick the piston: observer is pushed to x2, output now adjacent to the
    // lamp at x3. If landing pulses, the lamp lights within a few ticks.
    let snbt = "{DataVersion: 4903, size: [8, 2, 1], palette: [\
        {Name: \"minecraft:piston\", Properties: {extended: \"false\", facing: \"east\"}}, \
        {Name: \"minecraft:observer\", Properties: {facing: \"west\", powered: \"false\"}}, \
        {Name: \"minecraft:redstone_lamp\", Properties: {lit: \"false\"}}], \
        blocks: [{pos: [0, 0, 0], state: 0}, {pos: [1, 0, 0], state: 1}, {pos: [3, 0, 0], state: 2}], entities: []}";
    let structure = Structure::parse(snbt).unwrap();
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    let rb = sim.registry_mut().intern("minecraft:redstone_block").unwrap();
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
        *sim.behaviours_mut() = table;
    }
    assert_eq!(sim.unknown_report(), None);
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    sim.settle_with_order(&order);
    let mut lamp_lit_at = None;
    for t in 0..30u64 {
        if t == 2 {
            sim.place_block(Pos::new(0, 1, 0), rb);
        }
        sim.step();
        let lamp = sim.world().get(Pos::new(3, 0, 0));
        let descriptor = sim.registry().descriptor(lamp).unwrap_or("");
        if descriptor.contains("lit=true") && lamp_lit_at.is_none() {
            lamp_lit_at = Some(t);
        }
    }
    assert!(
        lamp_lit_at.is_some(),
        "a pushed observer landed at x2 next to the lamp and never pulsed — \
         the flying-machine heartbeat is missing"
    );
}

#[test]
#[ignore = "exploratory search; run with --ignored --nocapture"]
fn search_single_row_len6() {
    use Cell::*;
    let alphabet = [SpE, SpW, PE, ObE, ObW, Slime, Honey];
    let len = 6usize;
    let combos = alphabet.len().pow(len as u32);
    let mut candidates = Vec::new();
    for index in 0..combos {
        let mut row = Vec::with_capacity(len);
        let mut rest = index;
        for _ in 0..len {
            row.push(Some(alphabet[rest % alphabet.len()]));
            rest /= alphabet.len();
        }
        let cells: Vec<Cell> = row.iter().map(|c| c.unwrap()).collect();
        if cells.iter().copied().any(is_piston) && cells.iter().copied().any(is_observer) {
            candidates.push(vec![row]);
        }
    }
    run_candidates(candidates, "single row len 6");
}

/// Primitive: placing a block in front of an observer must pulse it — the
/// "place the last block" start every flying machine uses.
///
/// KNOWN GAP: `Simulation::place_block` sends neighbour (power) updates but
/// not the shape updates observers listen for, so this fails. Vanilla's
/// `setBlock` flag 3 sends both. The fix belongs in `sim.rs`'s write path;
/// until it lands, kick machines by powering a piston with a redstone block
/// (neighbour updates work) — the first stroke's landing pulse takes over.
#[test]
fn a_placed_block_pulses_the_watching_observer() {
    // Observer at x1 facing west (watches x0), lamp at x2 (its output side).
    let snbt = "{DataVersion: 4903, size: [6, 1, 1], palette: [\
        {Name: \"minecraft:observer\", Properties: {facing: \"west\", powered: \"false\"}}, \
        {Name: \"minecraft:redstone_lamp\", Properties: {lit: \"false\"}}], \
        blocks: [{pos: [1, 0, 0], state: 0}, {pos: [2, 0, 0], state: 1}], entities: []}";
    let structure = Structure::parse(snbt).unwrap();
    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    let stone = sim.registry_mut().intern("minecraft:stone").unwrap();
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
        *sim.behaviours_mut() = table;
    }
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    sim.place_on_place(&order);
    let mut lit_at = None;
    for t in 0..12u64 {
        if t == 2 {
            sim.place_block(Pos::new(0, 0, 0), stone);
        }
        sim.step();
        let lamp = sim.registry().descriptor(sim.world().get(Pos::new(2, 0, 0))).unwrap_or("");
        if lamp.contains("lit=true") && lit_at.is_none() {
            lit_at = Some(t);
        }
    }
    assert!(
        lit_at.is_some(),
        "placing stone in the observer's watched face never pulsed it — \
         place_block is not delivering the update observers listen for"
    );
}

/// Wiki "Two-way engine B" mapped onto +x travel (z = the 2-wide axis):
///   (0,0): observer output +x   (1,0): slime          (2,0): sticky piston B
///   (1,1): sticky piston A      (2,1): slime          (3,1): observer output -x
/// Piston facings are ambiguous in the wiki sprites — try all four combos,
/// kicked at either observer's watched face, and print movement telemetry.
#[test]
#[ignore = "exploratory probe; run with --ignored --nocapture"]
fn engine_b_variants() {
    for a_facing in ["east", "west"] {
        for b_facing in ["east", "west"] {
            let snbt = format!(
                "{{DataVersion: 4903, size: [40, 1, 2], palette: [\
                {{Name: \"minecraft:observer\", Properties: {{facing: \"west\", powered: \"false\"}}}}, \
                {{Name: \"minecraft:slime_block\"}}, \
                {{Name: \"minecraft:sticky_piston\", Properties: {{extended: \"false\", facing: \"{a_facing}\"}}}}, \
                {{Name: \"minecraft:sticky_piston\", Properties: {{extended: \"false\", facing: \"{b_facing}\"}}}}, \
                {{Name: \"minecraft:observer\", Properties: {{facing: \"east\", powered: \"false\"}}}}], \
                blocks: [\
                {{pos: [10, 0, 0], state: 0}}, {{pos: [11, 0, 0], state: 1}}, {{pos: [12, 0, 0], state: 3}}, \
                {{pos: [11, 0, 1], state: 2}}, {{pos: [12, 0, 1], state: 1}}, {{pos: [13, 0, 1], state: 4}}], \
                entities: []}}"
            );
            for (kick_name, kick) in [
                ("beside-A", Pos::new(11, 1, 1)),
                ("beside-B", Pos::new(12, 1, 0)),
            ] {
                let structure = Structure::parse(&snbt).unwrap();
                let mut sim = Simulation::new(structure.bounds(4));
                {
                    let (registry, world) = sim.registry_and_world_mut();
                    structure.place(world, registry, Pos::new(0, 0, 0));
                }
                let stone = sim.registry_mut().intern("minecraft:redstone_block").unwrap();
                mc_tick::intern_companions(sim.registry_mut());
                {
                    let mut table = std::mem::take(sim.behaviours_mut());
                    mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0, 0, 0));
                    *sim.behaviours_mut() = table;
                }
                let order = structure.placement_order(
                    mc_tick::vanilla::is_collision_full_cube,
                    mc_tick::vanilla::has_dynamic_shape,
                );
                sim.place_on_place(&order);
                let mut telemetry = String::new();
                for t in 0..60u64 {
                    if t == 2 {
                        sim.place_block(kick, stone);
                    }
                    if t == 4 {
                        sim.place_block(kick, mc_tick::StateId::AIR);
                    }
                    sim.step();
                    if t % 10 == 9 {
                        let xs: Vec<i32> = sim
                            .world()
                            .iter_non_air()
                            .filter(|(p, _)| *p != kick)
                            .map(|(p, _)| p.x)
                            .collect();
                        let (lo, hi) = (
                            xs.iter().min().copied().unwrap_or(0),
                            xs.iter().max().copied().unwrap_or(0),
                        );
                        telemetry.push_str(&format!(" t{}:[{lo},{hi}]n{}", t + 1, xs.len()));
                    }
                }
                println!("A={a_facing} B={b_facing} kick={kick_name}:{telemetry}");
            }
        }
    }
}

#[test]
#[ignore = "exploratory search; run with --ignored --nocapture"]
fn search_two_rows() {
    use Cell::*;
    let bottom_alpha: [Option<Cell>; 5] = [Some(SpE), Some(SpW), Some(Slime), Some(Honey), None];
    let top_alpha: [Option<Cell>; 7] =
        [Some(ObE), Some(ObW), Some(Slime), Some(Honey), Some(SpE), Some(SpW), None];
    for len in 2..=3usize {
        let combos = (bottom_alpha.len() * top_alpha.len()).pow(len as u32);
        let mut candidates = Vec::new();
        for index in 0..combos {
            let mut bottom = Vec::with_capacity(len);
            let mut top = Vec::with_capacity(len);
            let mut rest = index;
            for _ in 0..len {
                let pair = rest % (bottom_alpha.len() * top_alpha.len());
                rest /= bottom_alpha.len() * top_alpha.len();
                bottom.push(bottom_alpha[pair % bottom_alpha.len()]);
                top.push(top_alpha[pair / bottom_alpha.len()]);
            }
            let all: Vec<Cell> = bottom.iter().chain(top.iter()).filter_map(|c| *c).collect();
            if all.len() >= 3
                && all.iter().copied().any(is_piston)
                && all.iter().copied().any(is_observer)
            {
                candidates.push(vec![bottom, top]);
            }
        }
        run_candidates(candidates, &format!("two rows len {len}"));
    }
}

#[test]
#[ignore = "phase probe"]
fn engine_b_phase_probe() {
    let snbt = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus/structures/flying_machine_east.snbt")).unwrap();
    let structure = Structure::parse(&snbt).unwrap();
    let mut sim = Simulation::new(structure.bounds(4));
    { let (registry, world) = sim.registry_and_world_mut(); structure.place(world, registry, Pos::new(0,0,0)); }
    let rb = sim.registry_mut().intern("minecraft:redstone_block").unwrap();
    mc_tick::intern_companions(sim.registry_mut());
    { let mut table = std::mem::take(sim.behaviours_mut()); mc_tick::register_all_at(sim.registry_mut(), &mut table, Pos::new(0,0,0)); *sim.behaviours_mut() = table; }
    let order = structure.placement_order(mc_tick::vanilla::is_collision_full_cube, mc_tick::vanilla::has_dynamic_shape);
    sim.place_on_place(&order);
    for t in 0..80u64 {
        if t == 2 { sim.place_block(Pos::new(2,1,1), rb); }
        if t == 4 { sim.place_block(Pos::new(2,1,1), mc_tick::StateId::AIR); }
        sim.step();
        if t >= 58 && t <= 66 {
            let mut cells: Vec<String> = sim.world().iter_non_air().map(|(p,s)| format!("({},{},{})={}", p.x,p.y,p.z, sim.registry().descriptor(s).unwrap_or("?"))).collect();
            cells.sort();
            println!("t{}: {}", t+1, cells.join(" | "));
        }
    }
}
