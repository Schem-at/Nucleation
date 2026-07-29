//! Actuate the record 3x3 piston door and render what the engine actually does.
//!
//! ```text
//! cargo run --release --example door55_render --features bridge,mc-tick,rendering -- \
//!     <world.zip> <client.jar> <out.mp4> [--ticks N] [--press T] [--crop x0,y0,z0,x1,y1,z1] ...
//! ```
//!
//! The simulation half is `examples/door55_sim.rs`: the world is loaded through
//! the bridge with `TickSettleMode::InWorld`, because that is the only mode that
//! leaves a build cut out of a running save where it was found. The rendering
//! half is `examples/render_simulation_video.rs`: per-tick block changes are
//! reconstructed into a cast of `(position, state, lifetime)` members and posed
//! per frame, with `moving_piston` placeholders interpolated along their flight.
//!
//! Nothing here is tuned to make the door look right. If it tears itself apart,
//! that is what the video shows.
use mc_tick::{Dir, Pos};
use nucleation::animation::{Frame, Pose};
use nucleation::bridge::mc_tick::ffi::{TickSettleMode, TickSimulation};
use nucleation::bridge::schematic::ffi::Schematic;
use nucleation::meshing::{MeshConfig, MeshOutput, ResourcePackSource};
use nucleation::rendering::{render_animation_to_video, RenderConfig, VideoConfig};
use nucleation::UniversalSchematic;
use std::collections::{BTreeMap, HashMap};

fn read_out(f: impl FnOnce(&mut diplomat_runtime::DiplomatWrite)) -> String {
    unsafe {
        let write = diplomat_runtime::diplomat_buffer_write_create(0);
        f(&mut *write);
        let text = String::from_utf8_lossy((*write).as_bytes()).into_owned();
        diplomat_runtime::diplomat_buffer_write_destroy(write);
        text
    }
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// `[a,b,c]` following `key` inside `chunk`.
fn triple_i32(chunk: &str, key: &str) -> Option<Pos> {
    let rest = chunk.split(key).nth(1)?.split(']').next()?;
    let v: Vec<i32> = rest.split(',').filter_map(|n| n.trim().parse().ok()).collect();
    (v.len() == 3).then(|| Pos::new(v[0], v[1], v[2]))
}

fn triple_f64(chunk: &str, key: &str) -> Option<[f64; 3]> {
    let rest = chunk.split(key).nth(1)?.split(']').next()?;
    let v: Vec<f64> = rest.split(',').filter_map(|n| n.trim().parse().ok()).collect();
    (v.len() == 3).then(|| [v[0], v[1], v[2]])
}

fn string_field(chunk: &str, key: &str) -> Option<String> {
    Some(chunk.split(key).nth(1)?.split('"').next()?.to_string())
}

/// `[{"pos":[x,y,z],"state":"..."}]` -> the world.
fn parse_snapshot(json: &str) -> Vec<(Pos, String)> {
    json.split("{\"pos\":")
        .skip(1)
        .filter_map(|chunk| {
            let pos = triple_i32(&format!("\"pos\":{chunk}"), "\"pos\":[")?;
            let state = string_field(chunk, "\"state\":\"")?;
            Some((pos, state))
        })
        .collect()
}

/// `[{"tick":N,"pos":[..],"from":"..","to":".."}]` -> the change log.
fn parse_changes(json: &str) -> Vec<(u64, Pos, String, String)> {
    json.split("{\"tick\":")
        .skip(1)
        .filter_map(|chunk| {
            let tick: u64 = chunk.split(',').next()?.trim().parse().ok()?;
            let pos = triple_i32(chunk, "\"pos\":[")?;
            let from = string_field(chunk, "\"from\":\"")?;
            let to = string_field(chunk, "\"to\":\"")?;
            Some((tick, pos, from, to))
        })
        .collect()
}

/// One live entity at one tick: `(kind, position, velocity)`.
type EntitySnapshot = BTreeMap<(u8, u32), (String, [f64; 3], [f64; 3])>;

/// Both halves of `item_entities_json`. Items are tagged `0`, minecarts `1`,
/// because the two id spaces are independent.
fn parse_entities(json: &str) -> EntitySnapshot {
    let (items, carts) = json.split_once("],\"minecarts\":[").unwrap_or((json, ""));
    let mut out = EntitySnapshot::new();
    for (tag, section, name_key) in [(0u8, items, "\"item\":\""), (1, carts, "\"kind\":\"")] {
        for chunk in section.split("{\"id\":").skip(1) {
            let Some(id) = chunk.split(',').next().and_then(|t| t.trim().parse().ok()) else {
                continue;
            };
            let kind = string_field(chunk, name_key).unwrap_or_else(|| "?".into());
            let pos = triple_f64(chunk, "\"pos\":[").unwrap_or([f64::NAN; 3]);
            // A NaN component is not legal JSON and does not parse; that is the
            // mechanism, so an unreadable velocity is reported as NaN and not
            // silently zeroed.
            let vel = triple_f64(chunk, "\"vel\":[").unwrap_or([f64::NAN; 3]);
            out.insert((tag, id), (kind, pos, vel));
        }
    }
    out
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(world_path), Some(pack_path), Some(out_path)) =
        (args.first(), args.get(1), args.get(2))
    else {
        eprintln!(
            "usage: door55_render <world.zip> <pack|client.jar> <out.mp4> \
             [--ticks N] [--press T] [--button x,y,z] [--crop x0,y0,z0,x1,y1,z1] \
             [--frames-per-tick N] [--fps N] [--yaw d] [--pitch d] [--zoom f] [--tight] \
             [--report-only]"
        );
        std::process::exit(2);
    };
    let ticks: u64 = flag(&args, "--ticks").map_or(120, |v| v.parse().expect("--ticks N"));
    let press: u64 = flag(&args, "--press").map_or(5, |v| v.parse().expect("--press T"));
    let presses: Vec<u64> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--press")
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|v| v.parse().expect("--press T"))
        .collect();
    let presses = if presses.is_empty() { vec![press] } else { presses };
    let frames_per_tick: u32 =
        flag(&args, "--frames-per-tick").map_or(4, |v| v.parse().expect("N"));
    let fps: f64 = flag(&args, "--fps").map_or(30.0, |v| v.parse().expect("N"));
    let report_only = args.iter().any(|a| a == "--report-only");
    let tight = args.iter().any(|a| a == "--tight");
    // Every entity in this build sits *inside* the one-block-thick mechanism
    // plane, so a straight render occludes all 22 of them. `--ghost` fades the
    // blocks only; it changes nothing about the simulation, and the entity
    // positions it reveals are the ones the engine computed.
    let ghost: f32 = flag(&args, "--ghost").map_or(1.0, |v| v.parse().expect("--ghost f"));
    let yaw: f32 = flag(&args, "--yaw").map_or(45.0, |v| v.parse().expect("deg"));
    let pitch: f32 = flag(&args, "--pitch").map_or(35.264, |v| v.parse().expect("deg"));
    let zoom: f32 = flag(&args, "--zoom").map_or(1.0, |v| v.parse().expect("f"));
    let crop: Option<(Pos, Pos)> = flag(&args, "--crop").map(|v| {
        let c: Vec<i32> = v.split(',').map(|p| p.parse().expect("--crop 6 ints")).collect();
        assert_eq!(c.len(), 6, "--crop x0,y0,z0,x1,y1,z1");
        (
            Pos::new(c[0].min(c[3]), c[1].min(c[4]), c[2].min(c[5])),
            Pos::new(c[0].max(c[3]), c[1].max(c[4]), c[2].max(c[5])),
        )
    });
    let inside = |p: Pos| match crop {
        None => true,
        Some((lo, hi)) => {
            p.x >= lo.x && p.x <= hi.x && p.y >= lo.y && p.y <= hi.y && p.z >= lo.z && p.z <= hi.z
        }
    };

    // ── 1. Load the world exactly as `door55_sim` does ──────────────────────
    let bytes = std::fs::read(world_path)?;
    let schem = Schematic::from_world_zip(&bytes).map_err(|_| "world zip did not load")?;
    let mut sim = TickSimulation::from_schematic(&schem, TickSettleMode::InWorld, 0, 0, 0, b"")
        .map_err(|_| {
            format!(
                "the engine refused this build: {}",
                read_out(|w| TickSimulation::last_error_detail(w))
            )
        })?;
    println!("settle mode: InWorld");
    println!("Motion semantics: {}", read_out(|w| sim.motion_semantics(w)));

    let initial = parse_snapshot(&read_out(|w| sim.world_snapshot_json(w)));
    println!("world: {} non-air blocks", initial.len());

    // ── 2. Find the control in the simulation's own frame ───────────────────
    //
    // The capture log, the schematic and the simulation each number this build
    // differently, so the button is *searched for* rather than computed from a
    // remembered offset.
    let buttons: Vec<&(Pos, String)> = initial
        .iter()
        .filter(|(_, s)| s.contains("_button"))
        .collect();
    for (pos, state) in &buttons {
        println!("  control candidate {pos:?} {state}");
    }
    for (pos, state) in initial.iter().filter(|(_, s)| s.contains("pressure_plate")) {
        println!("  (plate, internal to the mechanism) {pos:?} {state}");
    }
    let button = match flag(&args, "--button") {
        Some(v) => {
            let c: Vec<i32> = v.split(',').map(|p| p.parse().expect("--button x,y,z")).collect();
            Pos::new(c[0], c[1], c[2])
        }
        None => buttons.first().map(|(p, _)| *p).ok_or("no button in this world")?,
    };
    println!("pressing {button:?} at tick(s) {presses:?}, running {ticks} ticks");

    // ── 3. Run ──────────────────────────────────────────────────────────────
    let mut entity_ticks: Vec<EntitySnapshot> = Vec::with_capacity(ticks as usize);
    let start_entities = parse_entities(&read_out(|w| sim.item_entities_json(w)));
    println!("entities in the simulator: {}", start_entities.len());
    for ((tag, id), (kind, pos, vel)) in &start_entities {
        let tag = if *tag == 0 { "item" } else { "cart" };
        println!("  {tag} id={id:<3} {kind:<28} pos={pos:?} vel={vel:?}");
    }

    let mut per_tick_changes: Vec<u32> = Vec::new();
    let mut before = sim.changes_count();
    for t in 0..ticks {
        if presses.contains(&t) {
            sim.use_block(button.x, button.y, button.z);
        }
        sim.step();
        let now = sim.changes_count();
        per_tick_changes.push(now - before);
        before = now;
        entity_ticks.push(parse_entities(&read_out(|w| sim.item_entities_json(w))));
    }

    // ── 4. Report before rendering ──────────────────────────────────────────
    println!("\n--- what happened ---");
    println!("block changes: {}", sim.changes_count());
    println!("quiescent after {ticks} ticks: {}", sim.is_quiescent());
    println!("piston_retract_contacts: {}", sim.piston_retract_contacts());
    let active: Vec<(usize, u32)> = per_tick_changes
        .iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .map(|(t, n)| (t, *n))
        .collect();
    match (active.first(), active.last()) {
        (Some((first, _)), Some((last, _))) => {
            println!("changes span ticks {first}..={last} ({} active ticks)", active.len());
            let line: Vec<String> =
                active.iter().map(|(t, n)| format!("t{t}:{n}")).collect();
            println!("per-tick: {}", line.join(" "));
        }
        _ => println!("no block changed at all"),
    }

    let changes = parse_changes(&read_out(|w| sim.changes_json(w)));
    // Did it come back? Compare the end world against the start world.
    let final_world: HashMap<Pos, String> =
        parse_snapshot(&read_out(|w| sim.world_snapshot_json(w))).into_iter().collect();
    let start_world: HashMap<Pos, String> = initial.iter().cloned().collect();
    let differing = start_world
        .iter()
        .filter(|(p, s)| final_world.get(*p) != Some(*s))
        .count()
        + final_world.keys().filter(|p| !start_world.contains_key(*p)).count();
    println!(
        "cells differing from the start state at tick {ticks}: {differing} \
         (0 means it returned home)"
    );

    if report_only {
        let mut lo = initial[0].0;
        let mut hi = initial[0].0;
        for (p, _) in &initial {
            lo = Pos::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
            hi = Pos::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
        }
        println!("bounds {lo:?} .. {hi:?}");
        let mut by_state: BTreeMap<&str, usize> = BTreeMap::new();
        for (_, s) in &initial {
            *by_state.entry(s.split('[').next().unwrap_or(s)).or_default() += 1;
        }
        println!("palette: {by_state:?}");
        println!("world, by layer (z slices, x across, y up):");
        for z in lo.z..=hi.z {
            println!("  z={z}");
            for y in (lo.y..=hi.y).rev() {
                let row: Vec<String> = (lo.x..=hi.x)
                    .map(|x| {
                        start_world
                            .get(&Pos::new(x, y, z))
                            .map(|s| {
                                let n = s.split('[').next().unwrap_or(s);
                                n.trim_start_matches("minecraft:").chars().take(6).collect()
                            })
                            .unwrap_or_else(|| ".".into())
                    })
                    .collect();
                println!("    y={y:<3} {}", row.iter().map(|c| format!("{c:<7}")).collect::<String>());
            }
        }
    }

    // The whole change log, and the passage over time. A door is judged by what
    // is *missing* while it is open, so the world is replayed cell by cell and
    // the cells that are air now but were solid at rest are counted each tick.
    if report_only {
        println!("\nchange log:");
        for (tick, pos, from, to) in &changes {
            println!("  t{tick:<4} {:>3},{:>2},{:>3}  {from} -> {to}", pos.x, pos.y, pos.z);
        }
        let mut live: HashMap<Pos, String> = start_world.clone();
        let mut cursor = 0usize;
        println!("\npassage (cells solid at rest that are air on this tick):");
        let mut last = usize::MAX;
        for t in 0..ticks {
            while cursor < changes.len() && changes[cursor].0 == t {
                let (_, pos, _, to) = &changes[cursor];
                if to == "minecraft:air" {
                    live.remove(pos);
                } else {
                    live.insert(*pos, to.clone());
                }
                cursor += 1;
            }
            let open: Vec<Pos> = start_world
                .keys()
                .filter(|p| !live.contains_key(*p))
                .copied()
                .collect();
            if open.len() != last {
                let mut sorted = open.clone();
                sorted.sort_by_key(|p| (p.y, p.x, p.z));
                let cells: Vec<String> =
                    sorted.iter().map(|p| format!("{},{},{}", p.x, p.y, p.z)).collect();
                println!("  t{t:<4} {} open: {}", open.len(), cells.join(" "));
                last = open.len();
            }
        }
    }

    // Which cells opened — a passage is cells that became air and stayed air.
    let opened: Vec<Pos> = start_world
        .keys()
        .filter(|p| !final_world.contains_key(*p))
        .copied()
        .collect();
    println!("cells that ended empty which began solid: {}", opened.len());
    if !opened.is_empty() {
        let (mut lo, mut hi) = (opened[0], opened[0]);
        for p in &opened {
            lo = Pos::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
            hi = Pos::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
        }
        println!("  their bounding box: {lo:?} .. {hi:?}");
    }

    // Entity motion, by the tick each first moved on.
    let mut first_move: BTreeMap<(u8, u32), (u64, String)> = BTreeMap::new();
    let mut previous = start_entities.clone();
    for (t, now) in entity_ticks.iter().enumerate() {
        for (key, (kind, pos, _)) in now {
            if let Some((_, was, _)) = previous.get(key) {
                if pos != was && !first_move.contains_key(key) {
                    first_move.insert(*key, (t as u64, kind.clone()));
                }
            }
        }
        previous = now.clone();
    }
    if first_move.is_empty() {
        println!("no entity moved");
    } else {
        println!("entities that moved ({}):", first_move.len());
        for (key, (t, kind)) in &first_move {
            let tag = if key.0 == 0 { "item" } else { "cart" };
            let now = entity_ticks.last().and_then(|s| s.get(key));
            let end = now.map(|(_, p, _)| format!("{p:?}")).unwrap_or_else(|| "gone".into());
            println!("  tick {t:<4} {tag} id={:<3} {kind:<28} ended {end}", key.1);
        }
    }
    let survivors = entity_ticks.last().map(|s| s.len()).unwrap_or(0);
    println!("entities alive at the end: {survivors} of {}", start_entities.len());
    println!(
        "NOTE: passengers are never instantiated, so the two blazes riding nan \
         carts are absent from this run; piston *retraction* does not displace \
         entities, it is only counted above."
    );

    if report_only {
        return Ok(());
    }

    // ── 5. Build the cast (render_simulation_video's reconstruction) ────────
    let initial: Vec<(Pos, String)> =
        initial.into_iter().filter(|(p, _)| inside(*p)).collect();
    let changes: Vec<(u64, Pos, String, String)> =
        changes.into_iter().filter(|(_, p, _, _)| inside(*p)).collect();
    let members = build_cast(&initial, &changes, ticks);
    println!("cast: {} members (crop {:?})", members.len(), crop);

    let pack = ResourcePackSource::from_file(pack_path)?;
    let mesh_config = MeshConfig::default();
    let mut mesh_cache: HashMap<(Pos, String), usize> = HashMap::new();
    let mut meshes: Vec<MeshOutput> = Vec::new();
    let mut mesh_of: Vec<usize> = Vec::with_capacity(members.len());
    for member in &members {
        let key = (member.pos, member.state.clone());
        let index = match mesh_cache.get(&key) {
            Some(i) => *i,
            None => {
                let mut one = UniversalSchematic::new("member".to_string());
                one.set_block_from_string(member.pos.x, member.pos.y, member.pos.z, &member.state)
                    .map_err(|e| format!("{}: {e}", member.state))?;
                meshes.push(one.to_mesh(&pack, &mesh_config)?);
                mesh_cache.insert(key, meshes.len() - 1);
                meshes.len() - 1
            }
        };
        mesh_of.push(index);
    }
    println!("meshes: {} (deduplicated)", meshes.len());

    // Entity tracks, cropped the same way: an entity is drawn if it is ever in
    // frame. Minecarts get the real vanilla hull; items get their block form.
    struct Track {
        kind: String,
        /// A real minecart gets the vanilla hull. Everything else is drawn as a
        /// cube the size of its **measured hitbox** — the frozen fireballs are
        /// mechanism, and their dimensions are the mechanism, so a cube of the
        /// right size says more than a wrong model would.
        cart: bool,
        size: f32,
        positions: Vec<Option<[f64; 3]>>,
    }
    /// Vanilla's width for the kinds this build contains (`crates/mc-tick`
    /// measured these against the game's registry).
    fn hitbox(kind: &str) -> f32 {
        match kind {
            "minecraft:dragon_fireball" => 1.0,
            "minecraft:small_fireball" => 0.3125,
            _ => 0.25,
        }
    }
    let mut tracks: Vec<Track> = Vec::new();
    let mut order: Vec<(u8, u32)> = start_entities.keys().copied().collect();
    for snapshot in &entity_ticks {
        for key in snapshot.keys() {
            if !order.contains(key) {
                order.push(*key);
            }
        }
    }
    for key in &order {
        let kind = entity_ticks
            .iter()
            .find_map(|s| s.get(key))
            .or_else(|| start_entities.get(key))
            .map(|(k, _, _)| k.clone())
            .unwrap_or_else(|| "minecraft:minecart".into());
        let mut positions: Vec<Option<[f64; 3]>> = Vec::with_capacity(ticks as usize);
        for snapshot in &entity_ticks {
            positions.push(snapshot.get(key).map(|(_, p, _)| *p));
        }
        // A NaN position cannot be drawn, and a body outside the crop is not
        // the subject of this shot.
        let visible = positions.iter().flatten().any(|p| {
            p.iter().all(|c| c.is_finite())
                && inside(Pos::new(
                    p[0].floor() as i32,
                    p[1].floor() as i32,
                    p[2].floor() as i32,
                ))
        });
        if visible {
            let cart = kind.contains("minecart");
            let size = hitbox(&kind);
            tracks.push(Track { kind, cart, size, positions });
        }
    }
    println!("entity tracks in frame: {}", tracks.len());

    // Entity meshes are one-block schematics whose pose carries them to the
    // entity's position, and the camera fits the *geometry*, not the pose — so
    // a mesh authored at the origin drags the frame back to 0,0,0 and leaves a
    // build standing at x=70 as a speck in the corner. Author them at a cell
    // inside the shot instead and subtract that cell from every translation.
    let anchor = members
        .first()
        .map(|m| m.pos)
        .unwrap_or_else(|| Pos::new(0, 0, 0));
    let mut track_mesh = Vec::new();
    for track in &tracks {
        let mut one = UniversalSchematic::new("entity".to_string());
        if track.cart {
            let points: Vec<[f64; 3]> = track.positions.iter().flatten().copied().collect();
            let yaw: f32 = match (points.first(), points.last()) {
                (Some(a), Some(b)) => {
                    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
                    if dx.abs() >= dz.abs() {
                        if dx >= 0.0 { -90.0 } else { 90.0 }
                    } else if dz >= 0.0 {
                        0.0
                    } else {
                        180.0
                    }
                }
                _ => 0.0,
            };
            let mut cart = nucleation::Entity::new(
                "minecraft:minecart".to_string(),
                (anchor.x as f64 + 0.5, anchor.y as f64, anchor.z as f64 + 0.5),
            );
            cart.nbt.insert(
                "Rotation".to_string(),
                nucleation::NbtValue::List(vec![
                    nucleation::NbtValue::Float(yaw),
                    nucleation::NbtValue::Float(0.0),
                ]),
            );
            one.add_entity(cart);
        } else {
            let drawn = match track.kind.as_str() {
                // A frozen fireball. Not a block; drawn as one so its hitbox
                // can be seen where it sits.
                "minecraft:dragon_fireball" | "minecraft:small_fireball" => {
                    "minecraft:magma_block".to_string()
                }
                "minecraft:diamond" | "minecraft:emerald" | "minecraft:lapis_lazuli"
                | "minecraft:coal" | "minecraft:redstone" => format!("{}_block", track.kind),
                other => other.replace("_ingot", "_block"),
            };
            if one.set_block_from_string(anchor.x, anchor.y, anchor.z, &drawn).is_err() {
                one.set_block_from_string(anchor.x, anchor.y, anchor.z, "minecraft:stone").ok();
            }
        }
        track_mesh.push(meshes.len());
        meshes.push(one.to_mesh(&pack, &mesh_config)?);
    }

    // ── 6. Pose and encode ──────────────────────────────────────────────────
    let hidden = Pose { scale: [0.0; 3], opacity: 0.0, ..Pose::IDENTITY };
    let frame_count = ticks as u32 * frames_per_tick;
    let mut frames = Vec::with_capacity(frame_count as usize);
    for f in 0..frame_count {
        let t = f as f64 / frames_per_tick as f64;
        let mut poses: Vec<Pose> = vec![hidden; meshes.len()];
        for (index, member) in members.iter().enumerate() {
            if t < member.start || t >= member.end {
                continue;
            }
            let mut pose = Pose::IDENTITY;
            pose.opacity = ghost;
            if let Some(motion) = &member.motion {
                let progress = ((t - member.start) / (motion.until - member.start)).clamp(0.0, 1.0);
                let remaining = 1.0 - progress as f32;
                pose.translate = [
                    (motion.from.x - member.pos.x) as f32 * remaining,
                    (motion.from.y - member.pos.y) as f32 * remaining,
                    (motion.from.z - member.pos.z) as f32 * remaining,
                ];
            }
            poses[mesh_of[index]] = pose;
        }
        for (i, track) in tracks.iter().enumerate() {
            let t0 = t.floor() as usize;
            let alpha = t - t.floor();
            let here = track.positions.get(t0).copied().flatten();
            let next = track.positions.get(t0 + 1).copied().flatten();
            let position = match (here, next) {
                (Some(a), Some(b)) => Some([
                    a[0] + (b[0] - a[0]) * alpha,
                    a[1] + (b[1] - a[1]) * alpha,
                    a[2] + (b[2] - a[2]) * alpha,
                ]),
                (Some(a), None) => Some(a),
                _ => None,
            };
            let Some(p) = position else { continue };
            if !p.iter().all(|c| c.is_finite()) {
                continue;
            }
            let (ax, ay, az) = (anchor.x as f32, anchor.y as f32, anchor.z as f32);
            let mut pose = Pose::IDENTITY;
            if track.cart {
                pose.pivot = [ax + 0.5, ay, az + 0.5];
                pose.translate = [
                    p[0] as f32 - ax - 0.5,
                    p[1] as f32 - ay,
                    p[2] as f32 - az - 0.5,
                ];
            } else {
                // The entity position is the bottom centre of its box, so the
                // cube is lifted by half its own height.
                let s = track.size;
                pose.scale = [s; 3];
                pose.pivot = [ax + 0.5, ay + 0.5, az + 0.5];
                pose.translate = [
                    p[0] as f32 - ax - 0.5,
                    p[1] as f32 + s / 2.0 - ay - 0.5,
                    p[2] as f32 - az - 0.5,
                ];
            }
            poses[track_mesh[i]] = pose;
        }
        frames.push(Frame {
            time_ms: f as f32 * (1000.0 / fps as f32),
            poses: poses.into_iter().enumerate().map(|(i, p)| (i as u32, p)).collect(),
            camera: None,
            gizmos: Vec::new(),
        });
    }

    let mut config = RenderConfig::isometric();
    config.width = 1280;
    config.height = 720;
    config.yaw = yaw;
    config.pitch = pitch;
    config.zoom = zoom;
    config.sphere_fit = !tight;
    let video = match out_path.rsplit('.').next() {
        Some("webm") => VideoConfig::vp9_alpha(fps),
        Some("mov") => VideoConfig::prores_4444(fps),
        _ => VideoConfig::h264(fps),
    }
    .map_err(|e| e.to_string())?;
    render_animation_to_video(
        &meshes,
        &frames,
        &config,
        None,
        &video,
        std::path::Path::new(out_path),
    )?;
    println!("{} frames -> {out_path}", frames.len());
    Ok(())
}

// ── the cast reconstruction, as in `render_simulation_video` ────────────────

struct Member {
    pos: Pos,
    state: String,
    start: f64,
    end: f64,
    motion: Option<Motion>,
}

struct Motion {
    from: Pos,
    until: f64,
}

fn build_cast(
    initial: &[(Pos, String)],
    changes: &[(u64, Pos, String, String)],
    ticks: u64,
) -> Vec<Member> {
    let mut segments: HashMap<Pos, Vec<(String, u64, u64)>> = HashMap::new();
    for (pos, state) in initial {
        segments.insert(*pos, vec![(state.clone(), 0, ticks)]);
    }
    for (tick, pos, from, to) in changes {
        let list = segments.entry(*pos).or_default();
        if let Some(last) = list.last_mut() {
            last.2 = *tick;
        } else if from != "minecraft:air" {
            list.push((from.clone(), 0, *tick));
        }
        list.push((to.clone(), *tick, ticks));
    }

    let mut vacated: HashMap<(u64, String), Vec<Pos>> = HashMap::new();
    for (tick, pos, from, _) in changes {
        vacated.entry((*tick, from.clone())).or_default().push(*pos);
    }

    let mut members = Vec::new();
    for (pos, list) in &segments {
        for (index, (state, start, end)) in list.iter().enumerate() {
            if state == "minecraft:air" || start == end {
                continue;
            }
            if !state.starts_with("minecraft:moving_piston") {
                members.push(Member {
                    pos: *pos,
                    state: state.clone(),
                    start: *start as f64,
                    end: *end as f64,
                    motion: None,
                });
                continue;
            }
            let facing = facing_of(state);
            let landed = list.get(index + 1).map(|(s, _, _)| s.as_str());
            let previous = index.checked_sub(1).map(|i| list[i].0.as_str());
            match landed {
                Some(next) if is_extended_false_of(previous, next) => {
                    members.push(Member {
                        pos: *pos,
                        state: previous.unwrap().to_string(),
                        start: *start as f64,
                        end: *end as f64,
                        motion: None,
                    });
                    let head_slot = pos.offset(facing);
                    members.push(Member {
                        pos: *pos,
                        state: head_state_for(next, facing),
                        start: *start as f64,
                        end: *end as f64,
                        motion: Some(Motion { from: head_slot, until: *end as f64 }),
                    });
                }
                Some(next)
                    if next != "minecraft:air"
                        && !next.starts_with("minecraft:moving_piston") =>
                {
                    let from = vacated
                        .get(&(*start, next.to_string()))
                        .and_then(|l| l.iter().find(|p| adjacent(**p, *pos)))
                        .copied()
                        .unwrap_or_else(|| pos.offset(facing.opposite()));
                    members.push(Member {
                        pos: *pos,
                        state: next.to_string(),
                        start: *start as f64,
                        end: list[index + 1].2 as f64,
                        motion: Some(Motion { from, until: *end as f64 }),
                    });
                }
                _ => {
                    members.push(Member {
                        pos: *pos,
                        state: format!(
                            "minecraft:piston_head[facing={},short=false,type={}]",
                            dir_name(facing),
                            type_of(state)
                        ),
                        start: *start as f64,
                        end: *end as f64,
                        motion: Some(Motion {
                            from: pos.offset(facing.opposite()),
                            until: *end as f64,
                        }),
                    });
                }
            }
        }
    }

    let flights: Vec<(Pos, String, f64, f64)> = members
        .iter()
        .filter(|m| m.motion.is_some())
        .map(|m| (m.pos, m.state.clone(), m.start, m.end))
        .collect();
    members.retain(|m| {
        m.motion.is_some()
            || !flights.iter().any(|(p, s, start, end)| {
                *p == m.pos && *s == m.state && m.start >= *start && m.end <= *end
            })
    });
    members
}

fn adjacent(a: Pos, b: Pos) -> bool {
    (a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs() == 1
}

fn facing_of(descriptor: &str) -> Dir {
    for (name, dir) in [
        ("facing=down", Dir::Down),
        ("facing=up", Dir::Up),
        ("facing=north", Dir::North),
        ("facing=south", Dir::South),
        ("facing=west", Dir::West),
        ("facing=east", Dir::East),
    ] {
        if descriptor.contains(name) {
            return dir;
        }
    }
    Dir::North
}

fn dir_name(dir: Dir) -> &'static str {
    match dir {
        Dir::Down => "down",
        Dir::Up => "up",
        Dir::North => "north",
        Dir::South => "south",
        Dir::West => "west",
        Dir::East => "east",
    }
}

fn type_of(moving_descriptor: &str) -> &'static str {
    if moving_descriptor.contains("type=sticky") { "sticky" } else { "normal" }
}

fn is_extended_false_of(previous: Option<&str>, next: &str) -> bool {
    let Some(previous) = previous else { return false };
    (next.starts_with("minecraft:piston[") || next.starts_with("minecraft:sticky_piston["))
        && next.contains("extended=false")
        && previous.replace("extended=true", "extended=false") == next
}

fn head_state_for(base: &str, facing: Dir) -> String {
    let kind = if base.starts_with("minecraft:sticky_piston") { "sticky" } else { "normal" };
    format!("minecraft:piston_head[facing={},short=false,type={kind}]", dir_name(facing))
}
