//! Observe two uses of a door control and compare with an unactivated replay.
use super::scene::{load, position, value};
use mc_test::mc_tick::{Pos, Simulation, StateId, Structure};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

const HELP: &str = "nucleation door-stats FILE [--list-inputs | --input X,Y,Z] [--json]
  --warmup N       ticks before measurement (default 10)
  --ticks N        ticks to observe after EACH of two uses (default 160)
  --aperture A:B   inclusive corners x,y,z:x,y,z of the passage to watch
  --origin X,Y,Z   world hash origin (defaults to schematic minimum)
  --timeline FILE  save per-tick JSON Lines, including ordered block writes and piston events";
type Point = [i32; 3];
type Layout = BTreeMap<Pos, StateId>;
fn pt(p: Pos) -> Point {
    [p.x, p.y, p.z]
}
fn pos(p: Point) -> Pos {
    Pos::new(p[0], p[1], p[2])
}
fn source_pos(p: Pos, offset: Point) -> [i64; 3] {
    std::array::from_fn(|i| i64::from(pt(p)[i]) + i64::from(offset[i]))
}
fn local(p: Point, offset: Point) -> Result<Pos, String> {
    let mut q = [0; 3];
    for i in 0..3 {
        q[i] = p[i].checked_sub(offset[i]).ok_or("coordinate overflow")?;
    }
    Ok(pos(q))
}
fn control(s: &str) -> bool {
    let name = s.split('[').next().unwrap_or(s);
    matches!(name, "minecraft:lever" | "minecraft:note_block")
        || name.starts_with("minecraft:") && name.ends_with("_button")
}
fn state(sim: &Simulation, id: StateId) -> &str {
    sim.registry().descriptor(id).unwrap_or("<unknown>")
}
fn layout(sim: &Simulation) -> Layout {
    sim.world().iter_non_air().collect()
}
fn difference(a: &Layout, b: &Layout) -> usize {
    a.iter().filter(|(p, s)| b.get(p) != Some(*s)).count()
        + b.keys().filter(|p| !a.contains_key(p)).count()
}
fn step(sim: &mut Simulation) -> Result<(), String> {
    let before = sim.tick_count();
    let reason = sim.step();
    if sim.tick_count() != before + 1 {
        return Err(format!("simulation stopped at tick {before}: {reason:?}"));
    }
    Ok(())
}
fn aperture(raw: &str, offset: Point) -> Result<Vec<Pos>, String> {
    let (a, b) = raw
        .split_once(':')
        .ok_or("aperture needs two inclusive corners")?;
    let a = pt(local(position(a)?, offset)?);
    let b = pt(local(position(b)?, offset)?);
    let lo: Point = std::array::from_fn(|i| a[i].min(b[i]));
    let hi: Point = std::array::from_fn(|i| a[i].max(b[i]));
    let volume = (0..3)
        .map(|i| i128::from(hi[i]) - i128::from(lo[i]) + 1)
        .product::<i128>();
    if volume > 65536 {
        return Err("aperture exceeds 65536 cells".into());
    }
    let mut cells = Vec::new();
    for x in lo[0]..=hi[0] {
        for y in lo[1]..=hi[1] {
            for z in lo[2]..=hi[2] {
                cells.push(Pos::new(x, y, z));
            }
        }
    }
    Ok(cells)
}
fn occupied(sim: &Simulation, cells: &[Pos]) -> usize {
    cells
        .iter()
        .filter(|&&p| {
            !matches!(
                state(sim, sim.world().get(p)),
                "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
            )
        })
        .count()
}

// A complete rectangular sheet that appeared/disappeared as a unit is a
// candidate door panel. Isolated mechanism blocks and partial sheets cannot
// establish a passage. Keep uncertainty when more than one sheet qualifies.
fn inferred_panels(sim: &Simulation, initial: &Layout, idle: &Layout) -> Vec<Vec<Pos>> {
    let endpoint = layout(sim);
    let mut changed = BTreeMap::new();
    for p in initial.keys().chain(endpoint.keys()) {
        let before = initial.get(p).copied().unwrap_or(StateId::AIR);
        let after = endpoint.get(p).copied().unwrap_or(StateId::AIR);
        if before == after || idle.get(p) != initial.get(p) {
            continue;
        }
        let material = if before == StateId::AIR {
            after
        } else if after == StateId::AIR {
            before
        } else {
            continue;
        };
        if state(sim, material).contains('[') {
            continue;
        }
        changed.insert(*p, (before, after));
    }
    let mut panels = Vec::new();
    while let Some((seed, transition)) = changed.pop_first() {
        let mut cells = vec![seed];
        let mut index = 0;
        let mut lo = pt(seed);
        let mut hi = lo;
        while index < cells.len() {
            let here = pt(cells[index]);
            index += 1;
            for axis in 0..3 {
                lo[axis] = lo[axis].min(here[axis]);
                hi[axis] = hi[axis].max(here[axis]);
                for d in [-1, 1] {
                    let mut n = here;
                    let Some(v) = n[axis].checked_add(d) else {
                        continue;
                    };
                    n[axis] = v;
                    let neighbor = pos(n);
                    if changed.get(&neighbor) == Some(&transition) {
                        changed.remove(&neighbor);
                        cells.push(neighbor);
                    }
                }
            }
        }
        let dims: [i128; 3] = std::array::from_fn(|i| i128::from(hi[i]) - i128::from(lo[i]) + 1);
        if dims.iter().filter(|&&d| d == 1).count() == 1
            && dims.iter().product::<i128>() == cells.len() as i128
            && cells.len() <= 65536
        {
            cells.sort();
            panels.push(cells);
        }
    }
    panels
}

struct Observation<'a> {
    offset: Point,
    cells: &'a [Pos],
    ticks: u64,
    input: Pos,
}
impl Observation<'_> {
    fn phase(
        &self,
        sim: &mut Simulation,
        phase: u64,
        mut timeline: Option<&mut BufWriter<std::fs::File>>,
    ) -> Result<Value, String> {
        let start = sim.tick_count();
        sim.record_timeline();
        let mut positions = HashSet::new();
        let mut extensions = 0usize;
        let mut retractions = 0usize;
        let (mut writes, mut strokes) = (0, 0);
        let mut last_write = None;
        let mut first_stroke = None;
        let initial = occupied(sim, self.cells);
        let mut last_states: Vec<_> = self.cells.iter().map(|&p| sim.world().get(p)).collect();
        let mut states_since = 0;
        let mut last_occupancy = initial;
        let mut occupancy_since = 0;
        let mut last_busy = 0;
        let before = state(sim, sim.world().get(self.input)).to_owned();
        sim.use_block(self.input);
        let after = state(sim, sim.world().get(self.input)).to_owned();
        for elapsed in 1..=self.ticks {
            if !sim.is_quiescent() {
                last_busy = elapsed;
            }
            step(sim)?;
            let view = sim.timeline_view().ok_or("timeline unavailable")?;
            let delta = &view.changes[writes..];
            let events = &view.pistons[strokes..];
            for c in delta {
                positions.insert(c.pos);
                last_write = Some(c.tick - start);
            }
            for e in events {
                first_stroke.get_or_insert(e.tick - start);
                if matches!(e.action, mc_test::mc_tick::PistonAction::Extend) {
                    extensions += 1
                } else {
                    retractions += 1
                }
            }
            for (&p, previous) in self.cells.iter().zip(&mut last_states) {
                let next = sim.world().get(p);
                if next != *previous {
                    states_since = elapsed;
                    *previous = next;
                }
            }
            let count = occupied(sim, self.cells);
            if count != last_occupancy {
                last_occupancy = count;
                occupancy_since = elapsed;
            }
            if let Some(w) = timeline.as_deref_mut() {
                let changes:Vec<_>=delta.iter().map(|c|json!({"tick":c.tick,"pos":source_pos(c.pos,self.offset),"from":state(sim,c.from),"to":state(sim,c.to)})).collect();
                let pistons:Vec<_>=events.iter().map(|e|json!({"tick":e.tick,"pos":source_pos(e.pos,self.offset),"action":format!("{:?}",e.action),"direction":format!("{:?}",e.dir),"sticky":e.sticky,"after_writes_in_row":e.change_index-writes})).collect();
                let row = json!({"use":phase,"elapsed_ticks":elapsed,"tick_end":sim.tick_count(),"clicked":elapsed==1,"quiescent":sim.is_quiescent(),"occupied_aperture_cells":if self.cells.is_empty(){None}else{Some(count)},"changes":changes,"pistons":pistons});
                serde_json::to_writer(&mut *w, &row).map_err(|e| e.to_string())?;
                w.write_all(b"\n").map_err(|e| e.to_string())?;
            }
            writes = view.changes.len();
            strokes = view.pistons.len();
        }
        let aperture=(!self.cells.is_empty()).then(|| json!({"cells":self.cells.len(),"occupied_before":initial,"occupied_after":last_occupancy,
        "endpoint":if last_occupancy==0{"clear"}else if last_occupancy==self.cells.len(){"filled"}else{"partial"},
        "endpoint_states_since_elapsed_tick":states_since,"endpoint_occupancy_since_elapsed_tick":occupancy_since}));
        Ok(
            json!({"use":phase,"input_at_tick":start,"control_before":before,"control_immediately_after":after,
        "block_writes":writes,"unique_positions_written":positions.len(),"first_stroke_tick_offset":first_stroke,
        "last_block_write_tick_offset":last_write,"piston_extensions":extensions,"piston_retractions_including_drops":retractions,
        "inventory_slot_changes":sim.recorded_inventory().len(),"entity_events":sim.recorded_entities().len(),
        "quiescent_at_end":sim.is_quiescent(),"queues_quiet_since_elapsed_tick":if sim.is_quiescent(){Some(last_busy)}else{None},
        "aperture":aperture}),
        )
    }
}

pub(crate) fn door_main(args: impl Iterator<Item = String>) {
    if let Err(e) = run(args) {
        eprintln!("{e}");
        std::process::exit(2)
    }
}
fn run(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let (mut file, mut input, mut watch, mut timeline, mut origin) = (None, None, None, None, None);
    let (mut ticks, mut warmup) = (160u64, 10u64);
    let (mut json_out, mut list) = (false, false);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => {
                println!("{HELP}");
                return Ok(());
            }
            "--list-inputs" => list = true,
            "--json" => json_out = true,
            "--input" => input = Some(position(&value(&mut args, &arg)?)?),
            "--origin" => origin = Some(position(&value(&mut args, &arg)?)?),
            "--aperture" => watch = Some(value(&mut args, &arg)?),
            "--timeline" => timeline = Some(PathBuf::from(value(&mut args, &arg)?)),
            "--ticks" => {
                ticks = value(&mut args, &arg)?
                    .parse()
                    .map_err(|_| "invalid --ticks")?
            }
            "--warmup" => {
                warmup = value(&mut args, &arg)?
                    .parse()
                    .map_err(|_| "invalid --warmup")?
            }
            x if (x == "-" || !x.starts_with('-')) && file.is_none() => {
                file = Some(PathBuf::from(x))
            }
            _ => return Err(HELP.into()),
        }
    }
    if !(1..=100000).contains(&ticks) || warmup > 100000 {
        return Err("use 1..100000 observation ticks and 0..100000 warmup ticks".into());
    }
    let file = file.ok_or(HELP)?;
    let source = load(&file)?;
    let bounds = source.get_bounding_box();
    let offset: Point = bounds.min.into();
    let origin = origin.unwrap_or(offset);
    let structure = Structure::parse(&nucleation::formats::gametest::to_gametest_snbt(&source))
        .map_err(|e| e.to_string())?;
    let mut sim = mc_test::try_build_sim(
        &structure,
        pos(origin),
        mc_test::SettleMode::InWorld,
        &[],
        &[],
        source
            .metadata
            .source_data_version
            .or(source.metadata.mc_version),
        "door stats",
    )?;
    sim.set_rng_seed(0);
    sim.discard_recording();
    for _ in 0..warmup {
        step(&mut sim)?;
    }
    let initial = layout(&sim);
    let controls: Vec<_> = initial
        .iter()
        .filter(|(_, id)| control(state(&sim, **id)))
        .map(|(&p, &id)| (p, state(&sim, id).to_owned()))
        .collect();
    let candidates: Vec<_> = controls
        .iter()
        .map(|(p, id)| json!({"pos":source_pos(*p,offset),"state":id}))
        .collect();
    if list {
        println!("{}", json!(candidates));
        return Ok(());
    }
    let levers: Vec<_> = controls
        .iter()
        .filter(|(_, s)| s.split('[').next() == Some("minecraft:lever"))
        .collect();
    let (p, selection) = if let Some(input) = input {
        let p = local(input, offset)?;
        if !controls.iter().any(|(c, _)| *c == p) {
            return Err(format!(
                "no supported lever, button or note block at {input:?}; use --list-inputs"
            ));
        }
        (p, "explicit")
    } else if controls.len() == 1 {
        (controls[0].0, "only_candidate")
    } else if levers.len() == 1 {
        (levers[0].0, "only_lever")
    } else {
        return Err(format!(
            "{} input candidates ({} levers); use --list-inputs and --input X,Y,Z",
            controls.len(),
            levers.len()
        ));
    };
    sim.set_rng_seed(0);
    let checkpoint = sim.checkpoint();
    sim.record();
    let mut idle = Vec::new();
    for _ in 0..2 {
        let before = sim.recorded().len();
        for _ in 0..ticks {
            step(&mut sim)?;
        }
        idle.push((layout(&sim), sim.recorded().len() - before));
    }
    sim.restore(&checkpoint);
    sim.set_rng_seed(0);
    let (cells, aperture_selection) = if let Some(raw) = watch.as_deref() {
        (aperture(raw, offset)?, "explicit")
    } else {
        sim.discard_recording();
        sim.use_block(p);
        for _ in 0..ticks {
            step(&mut sim)?;
        }
        let mut panels = if sim.is_quiescent() {
            inferred_panels(&sim, &initial, &idle[0].0)
        } else {
            Vec::new()
        };
        sim.restore(&checkpoint);
        sim.set_rng_seed(0);
        match panels.len() {
            1 => (panels.pop().unwrap(), "inferred_rectangular_panel"),
            0 => (Vec::new(), "undetermined"),
            _ => (Vec::new(), "ambiguous"),
        }
    };
    let trial = Observation {
        offset,
        cells: &cells,
        ticks,
        input: p,
    };
    let mut dest = timeline
        .as_ref()
        .map(|path| {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map(BufWriter::new)
                .map_err(|e| format!("{}: {e}", path.display()))
        })
        .transpose()?;
    let execute = (|| -> Result<Vec<Value>, String> {
        let mut phases = Vec::new();
        for (i, (idle_layout, idle_writes)) in idle.iter().enumerate() {
            let mut phase = trial.phase(&mut sim, i as u64 + 1, dest.as_mut())?;
            let endpoint = layout(&sim);
            phase["endpoint_blocks_different_from_initial"] =
                json!(difference(&initial, &endpoint));
            phase["endpoint_blocks_different_from_idle"] =
                json!(difference(idle_layout, &endpoint));
            phase["idle_block_writes"] = json!(idle_writes);
            phases.push(phase);
        }
        if let Some(w) = &mut dest {
            w.flush().map_err(|e| e.to_string())?
        }
        Ok(phases)
    })();
    let phases = match execute {
        Ok(v) => v,
        Err(e) => {
            drop(dest);
            if let Some(path) = &timeline {
                let _ = std::fs::remove_file(path);
            }
            return Err(e);
        }
    };
    let report = json!({"dimensions":bounds.get_dimensions(),"initial_non_air_blocks":initial.len(),"input":source_pos(p,offset),"selection":selection,"input_candidates":candidates.len(),"hash_origin":origin,"warmup_ticks":warmup,"ticks_per_use":ticks,"aperture_selection":aperture_selection,"aperture_bounds":cells.first().zip(cells.last()).map(|(&a,&b)| [source_pos(a,offset),source_pos(b,offset)]),"phases":phases,"cycle_restores_block_layout":phases[1]["endpoint_blocks_different_from_initial"]==0,"timeline":timeline});
    if json_out {
        println!("{report}");
    } else {
        println!(
            "Door stats for {}\nDimensions {:?}, {} non-air blocks. Input {:?} ({selection}).",
            file.display(),
            bounds.get_dimensions(),
            initial.len(),
            source_pos(p, offset)
        );
        println!(
            "Aperture: {aperture_selection} {}",
            report["aperture_bounds"]
        );
        for phase in phases {
            println!("Use {}: {} block writes (idle {}), {} extensions, {} retractions. Last write at +{} game ticks. Queues quiet at end: {}.",phase["use"],phase["block_writes"],phase["idle_block_writes"],phase["piston_extensions"],phase["piston_retractions_including_drops"],phase["last_block_write_tick_offset"],phase["quiescent_at_end"]);
            if !cells.is_empty() {
                let a = &phase["aperture"];
                println!(
                    "  Aperture {} / {} occupied, {} since +{} elapsed ticks.",
                    a["occupied_after"],
                    cells.len(),
                    a["endpoint"],
                    a["endpoint_occupancy_since_elapsed_tick"]
                );
            }
        }
        println!(
            "Initial block layout restored: {}",
            report["cycle_restores_block_layout"]
        );
    }
    Ok(())
}
