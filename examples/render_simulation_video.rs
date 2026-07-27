//! Animate an mc-tick simulation as a smooth video.
//!
//!     cargo run --release --example render_simulation_video --features rendering -- \
//!         <pack.zip|client.jar> <structure.snbt> <out.mp4> \
//!         [--ticks N] [--click x,y,z@T] [--frames-per-tick 6] [--fps 30]
//!
//! The simulation is the source of truth: the world's per-tick block changes are
//! reconstructed into a cast of `(position, state, lifetime)` members, each
//! meshed once and driven by animation poses. Blocks in flight — vanilla's
//! `moving_piston` block entities — are drawn as the block they carry, linearly
//! interpolated from source to destination across their two-tick move window.
//! That is g4mespeed's default "pause at end" piston animation
//! (`(progress * steps + tickDelta) / steps`, clamped), which is what makes
//! piston motion look continuous instead of stepping twice per move.
//!
//! A retracting piston additionally synthesises the vanilla visual: the base
//! renders as its extended shell while a piston head slides back into it.
use mc_tick::{Pos, Simulation, Structure};
use nucleation::animation::{Frame, Pose};
use nucleation::meshing::{MeshConfig, MeshOutput, ResourcePackSource};
use nucleation::rendering::{render_animation_to_video, RenderConfig, VideoConfig};
use nucleation::UniversalSchematic;
use std::collections::HashMap;

/// One drawable: a block state at a position, alive for a tick interval.
struct Member {
    pos: Pos,
    state: String,
    /// Alive for `start..end`, in game ticks (fractional times compare in this range).
    start: f64,
    end: f64,
    /// Where it travels from during `start..motion_end`, if it is in flight.
    motion: Option<Motion>,
}

struct Motion {
    from: Pos,
    /// Interpolation finishes here (the landing tick).
    until: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(pack_path), Some(snbt_path), Some(out_path)) =
        (args.first(), args.get(1), args.get(2))
    else {
        eprintln!(
            "usage: render_simulation_video <pack> <structure.snbt> <out.mp4> \
             [--ticks N] [--click x,y,z@T] [--frames-per-tick 6] [--fps 30]"
        );
        std::process::exit(2);
    };
    let ticks: u64 = flag(&args, "--ticks").map_or(40, |v| v.parse().expect("--ticks N"));
    let frames_per_tick: u32 = flag(&args, "--frames-per-tick")
        .map_or(6, |v| v.parse().expect("--frames-per-tick N"));
    let fps: f64 = flag(&args, "--fps").map_or(30.0, |v| v.parse().expect("--fps N"));
    let transparent = args.iter().any(|a| a == "--transparent");
    // Run vanilla's placement pass first — a community build saved mid-cycle
    // needs it before it means anything.
    let settle = args.iter().any(|a| a == "--settle");
    // Framing. A door is wide and tall but thin, so the rotation-invariant
    // sphere fit wastes most of the frame on empty air; --tight uses the
    // silhouette fit instead.
    let yaw: f32 = flag(&args, "--yaw").map_or(45.0, |v| v.parse().expect("--yaw deg"));
    let pitch: f32 = flag(&args, "--pitch").map_or(35.264, |v| v.parse().expect("--pitch deg"));
    let zoom: f32 = flag(&args, "--zoom").map_or(1.0, |v| v.parse().expect("--zoom f"));
    let tight = args.iter().any(|a| a == "--tight");
    // Every --click occurrence is an actuation; a moving machine's note block
    // moves with it, so successive clicks target successive positions.
    let clicks: Vec<(Pos, u64)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--click")
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|v| {
            let (xyz, t) = v.split_once('@').expect("--click x,y,z@T");
            let p: Vec<i32> = xyz.split(',').map(|c| c.parse().expect("coord")).collect();
            (Pos::new(p[0], p[1], p[2]), t.parse().expect("tick"))
        })
        .collect();

    // --pulse x,y,z@T: a redstone block for two ticks, like the capture tool.
    let pulses: Vec<(Pos, u64)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--pulse")
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|v| {
            let (xyz, t) = v.split_once('@').expect("--pulse x,y,z@T");
            let p: Vec<i32> = xyz.split(',').map(|c| c.parse().expect("coord")).collect();
            (Pos::new(p[0], p[1], p[2]), t.parse().expect("tick"))
        })
        .collect();

    // --break x,y,z@T: replace a block with air, the capture tool's --break.
    let breaks: Vec<(Pos, u64)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--break")
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|v| {
            let (xyz, t) = v.split_once('@').expect("--break x,y,z@T");
            let p: Vec<i32> = xyz.split(',').map(|c| c.parse().expect("coord")).collect();
            (Pos::new(p[0], p[1], p[2]), t.parse().expect("tick"))
        })
        .collect();

    // --place x,y,z@T: put a redstone block down and leave it there.
    let places: Vec<(Pos, u64)> = args
        .iter()
        .enumerate()
        .filter(|(_, a)| *a == "--place")
        .filter_map(|(i, _)| args.get(i + 1))
        .map(|v| {
            let (xyz, t) = v.split_once('@').expect("--place x,y,z@T");
            let p: Vec<i32> = xyz.split(',').map(|c| c.parse().expect("coord")).collect();
            (Pos::new(p[0], p[1], p[2]), t.parse().expect("tick"))
        })
        .collect();

    // ── 1. Simulate, or replay a capture ────────────────────────────────────
    //
    // `--trace <capture.json>` renders what *vanilla* did instead of what the
    // engine does. Same pipeline either way: a capture is already a list of
    // per-tick block changes, which is exactly what the simulation produces.
    // Being able to watch the two side by side is the point — a divergence in
    // a 200-event tick is far easier to recognise than to read.
    let (initial, changes, item_tracks) = match flag(&args, "--trace") {
        Some(trace_path) => {
            let (initial, changes) = replay(snbt_path, trace_path, ticks);
            println!("replayed {ticks} ticks, {} block changes", changes.len());
            (initial, changes, Vec::new())
        }
        None => {
            let out = simulate(snbt_path, ticks, &clicks, &pulses, &breaks, &places, settle);
            println!("simulated {ticks} ticks, {} block changes", out.1.len());
            out
        }
    };

    println!(
        "item tracks: {} ({} entity events)",
        item_tracks.len(),
        item_tracks.iter().map(|t| t.positions.iter().flatten().count()).sum::<usize>()
    );

    // ── 2. Reconstruct the cast ─────────────────────────────────────────────
    let members = build_cast(&initial, &changes, ticks);
    println!("cast: {} members", members.len());

    // ── 3. Mesh each member once ────────────────────────────────────────────
    let pack = ResourcePackSource::from_file(pack_path)?;
    let mesh_config = MeshConfig::default();
    let mut mesh_cache: HashMap<(Pos, String), usize> = HashMap::new();
    let mut meshes: Vec<MeshOutput> = Vec::new();
    // member index -> mesh index (members sharing (pos, state) share a mesh).
    let mut mesh_of: Vec<usize> = Vec::with_capacity(members.len());
    for member in &members {
        let key = (member.pos, member.state.clone());
        let index = match mesh_cache.get(&key) {
            Some(i) => *i,
            None => {
                let mut one = UniversalSchematic::new("member".to_string());
                // Fluids have no blockstate model; draw them as tinted glass
                // boxes, squashed to the fluid's surface height by their pose.
                let drawn = fluid_proxy(&member.state).unwrap_or(member.state.as_str());
                one.set_block_from_string(member.pos.x, member.pos.y, member.pos.z, drawn)
                    .map_err(|e| format!("{}: {e}", member.state))?;
                let mesh = one.to_mesh(&pack, &mesh_config)?;
                meshes.push(mesh);
                mesh_cache.insert(key, meshes.len() - 1);
                meshes.len() - 1
            }
        };
        mesh_of.push(index);
    }
    println!("meshes: {} (deduplicated)", meshes.len());

    // Item entities: one mesh per entity (each needs its own pose), the item's
    // block form scaled to vanilla's item-cube proportions, positioned along
    // the simulation's per-tick track with sub-tick interpolation.
    let mut item_mesh_range = Vec::new();
    for track in &item_tracks {
        let mut one = UniversalSchematic::new("item".to_string());
        if track.item == "minecraft:minecart" {
            // A cart-shaped mini-structure: a 3×3 iron floor with a one-high
            // rim, scaled down to the cart's 0.98 × 0.7 box by its pose.
            for x in 0..3 {
                for z in 0..3 {
                    one.set_block_from_string(x, 0, z, "minecraft:polished_deepslate").ok();
                    if x != 1 || z != 1 {
                        one.set_block_from_string(x, 1, z, "minecraft:polished_deepslate").ok();
                    }
                }
            }
        } else {
            // Item ids are not block ids; ingots at least have a block to show.
            let drawn = track.item.replace("_ingot", "_block");
            if one.set_block_from_string(0, 0, 0, &drawn).is_err() {
                one.set_block_from_string(0, 0, 0, "minecraft:stone").ok();
            }
        }
        item_mesh_range.push(meshes.len());
        let mesh = one.to_mesh(&pack, &mesh_config)?;
        println!(
            "item mesh {} ({}): {} opaque verts",
            meshes.len(),
            track.item,
            mesh.opaque.positions.len()
        );
        meshes.push(mesh);
    }

    // ── 4. Pose every mesh for every frame ──────────────────────────────────
    let hidden = Pose {
        scale: [0.0; 3],
        opacity: 0.0,
        ..Pose::IDENTITY
    };
    let frame_count = ticks as u32 * frames_per_tick;
    let mut frames = Vec::with_capacity(frame_count as usize);
    for f in 0..frame_count {
        let t = f as f64 / frames_per_tick as f64;
        // One pose per MESH: visible members overwrite the hidden default.
        // Members sharing a mesh never overlap in time, so last-write wins is safe.
        let mut poses: Vec<Pose> = vec![hidden; meshes.len()];
        for (index, member) in members.iter().enumerate() {
            if t < member.start || t >= member.end {
                continue;
            }
            let mut pose = Pose::IDENTITY;
            if let Some(height) = fluid_height(&member.state) {
                pose.scale = [1.0, height, 1.0];
                pose.pivot = [
                    member.pos.x as f32 + 0.5,
                    member.pos.y as f32,
                    member.pos.z as f32 + 0.5,
                ];
            }
            if let Some(motion) = &member.motion {
                // g4mespeed PAUSE_END: linear across the move window, clamped.
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
        for (track_index, track) in item_tracks.iter().enumerate() {
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
            if let Some(p) = position {
                let mut pose = Pose::IDENTITY;
                let cart = track.item == "minecraft:minecart";
                // The cart mesh is 3×2×3, squeezed to the real 0.98 × 0.7 box.
                pose.scale = if cart {
                    [0.98 / 3.0, 0.7 / 2.0, 0.98 / 3.0]
                } else {
                    [0.25; 3]
                };
                // Scale in place about the block's centre, then translate that
                // centre to the entity position (plus half the scaled height).
                if cart {
                    // Scale about the mesh's bottom centre, then put that
                    // point at the entity position.
                    pose.pivot = [1.5, 0.0, 1.5];
                    pose.translate = [p[0] as f32 - 1.5, p[1] as f32, p[2] as f32 - 1.5];
                } else {
                    pose.pivot = [0.5, 0.5, 0.5];
                    pose.translate = [
                        p[0] as f32 - 0.5,
                        p[1] as f32 + 0.125 - 0.5,
                        p[2] as f32 - 0.5,
                    ];
                }
                poses[item_mesh_range[track_index]] = pose;
            }
        }
        frames.push(Frame {
            time_ms: f as f32 * (1000.0 / fps as f32),
            poses: poses
                .into_iter()
                .enumerate()
                .map(|(i, p)| (i as u32, p))
                .collect(),
            camera: None,
            gizmos: Vec::new(),
        });
    }

    // ── 5. Render straight to video ─────────────────────────────────────────
    let mut config = RenderConfig::isometric();
    config.width = 1280;
    config.height = 720;
    config.yaw = yaw;
    config.pitch = pitch;
    config.zoom = zoom;
    // Bounds cover the whole run either way, so the camera never drifts.
    config.sphere_fit = !tight;
    if transparent {
        config.background = Some([0.0, 0.0, 0.0, 0.0]);
    }
    // The container picks the codec: .webm keeps the alpha channel and still
    // plays inline in chat clients; .mov is ProRes 4444 for compositing.
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

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .map(String::as_str)
}

/// Quiet placement (no settle), record, click, run. Returns the initial world
/// and the change log, both as descriptor strings.
/// One item entity's life for the renderer: its block id and a per-tick
/// position track (`None` once removed).
struct ItemTrack {
    item: String,
    positions: Vec<Option<[f64; 3]>>,
}

/// Rebuild `(initial, changes)` from a captured vanilla trace.
///
/// The structure supplies the starting world — a capture records only what
/// *changed* — and the trace supplies every change after it. Positions the
/// capture mentions but the structure does not (a block pushed into empty air
/// beyond the build) come in as air, which `build_cast` already handles.
fn replay(
    snbt_path: &str,
    trace_path: &str,
    ticks: u64,
) -> (Vec<(Pos, String)>, Vec<(u64, Pos, String, String)>) {
    let text = std::fs::read_to_string(snbt_path).expect("read structure");
    let structure = Structure::parse(&text).expect("parse structure");
    let initial: Vec<(Pos, String)> = structure
        .blocks
        .iter()
        .map(|(pos, entry)| (*pos, structure.palette[*entry].clone()))
        .filter(|(_, state)| state != "minecraft:air")
        .collect();

    let mut initial: std::collections::HashMap<Pos, String> = initial.into_iter().collect();

    let json = std::fs::read_to_string(trace_path).expect("read trace");
    let trace = mc_tick_trace::Trace::from_json(&json).expect("parse trace");
    let mut changes: Vec<(u64, Pos, String, String)> = Vec::new();
    let mut seen: std::collections::HashSet<Pos> = std::collections::HashSet::new();
    for record in &trace.ticks {
        if record.tick >= ticks {
            break;
        }
        for event in &record.events {
            if let mc_tick_trace::EventKind::BlockChanged { pos, from, to } = &event.kind {
                let at = Pos::new(pos.0, pos.1, pos.2);
                // The capture's baseline is taken *after* placement, so for a
                // loud capture the world it starts from is not the file: a
                // placement settle may already have extended pistons and lit
                // dust. Each position's first `from` is what the capture says
                // was there, and it wins over the schematic — otherwise the
                // cast is built on states that never existed and blocks
                // animate out of the wrong pose.
                if !seen.contains(&at) {
                    seen.insert(at);
                    if from == "minecraft:air" {
                        initial.remove(&at);
                    } else {
                        initial.insert(at, from.clone());
                    }
                }
                changes.push((record.tick, at, from.clone(), to.clone()));
            }
        }
    }
    let mut initial: Vec<(Pos, String)> = initial.into_iter().collect();
    initial.sort_by_key(|(p, _)| (p.y, p.x, p.z));
    (initial, changes)
}

fn simulate(
    snbt_path: &str,
    ticks: u64,
    clicks: &[(Pos, u64)],
    pulses: &[(Pos, u64)],
    breaks: &[(Pos, u64)],
    places: &[(Pos, u64)],
    settle: bool,
) -> (
    Vec<(Pos, String)>,
    Vec<(u64, Pos, String, String)>,
    Vec<ItemTrack>,
) {
    let text = std::fs::read_to_string(snbt_path).expect("read structure");
    let structure = Structure::parse(&text).expect("parse structure");

    let mut sim = Simulation::new(structure.bounds(4));
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
    }
    if let Some(report) = sim.unknown_report() {
        panic!("unimplemented blocks: {report}");
    }

    let initial: Vec<(Pos, String)> = sim
        .world()
        .iter_non_air()
        .filter_map(|(pos, state)| {
            sim.registry()
                .descriptor(state)
                .map(|d| (pos, d.to_string()))
        })
        .collect();

    {
        let (solidity, frictions, heights, webs) = mc_tick::vanilla::physics_tables(sim.registry());
        sim.set_physics_tables(solidity, frictions, heights, webs);
        let (water_kinds, bubble_kinds) = mc_tick::vanilla::fluid_tables(sim.registry());
        sim.set_fluid_tables(water_kinds, bubble_kinds);
        let (rails, conductors) = mc_tick::vanilla::rail_tables(sim.registry());
        sim.set_rail_tables(rails, conductors);
    }
    for spawned in &structure.entities {
        match spawned {
            mc_tick::structure::SpawnedEntity::Item(item) => {
                sim.spawn_item(item.item.clone(), item.pos, item.motion, item.pickup_delay);
            }
            mc_tick::structure::SpawnedEntity::Minecart(cart) => {
                sim.spawn_minecart(cart.kind.clone(), cart.pos, cart.motion);
            }
        }
    }
    for (pos, stacks) in &structure.inventories {
        let entry = structure
            .blocks
            .iter()
            .find(|(p, _)| p == pos)
            .map(|(_, e)| *e)
            .expect("inventory with no block");
        let name = structure.palette[entry].split('[').next().unwrap_or_default();
        let slots = mc_tick::vanilla::container_slots(name).expect("container slot count");
        sim.set_inventory(*pos, mc_tick::Inventory { slots, stacks: stacks.clone() });
    }
    let redstone_block = sim
        .registry_mut()
        .intern("minecraft:redstone_block")
        .expect("intern redstone block");
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;
    }
    for (pos, entry) in &structure.blocks {
        let state = sim.registry().get(&structure.palette[*entry]);
        let is_ticker = state
            .and_then(|s| sim.behaviours().get(s))
            .is_some_and(|b| b.ticks_as_block_entity());
        if is_ticker {
            sim.add_block_entity_ticker(*pos);
        }
    }

    if settle {
        let order = structure.placement_order(
            mc_tick::vanilla::is_collision_full_cube,
            mc_tick::vanilla::has_dynamic_shape,
        );
        sim.settle_with_order(&order);
    }
    sim.record();
    for t in 0..ticks {
        for (pos, at) in clicks {
            if *at == t {
                sim.use_block(*pos);
            }
        }
        for (pos, at) in pulses {
            if *at == t {
                sim.place_block(*pos, redstone_block);
            }
            if *at + 2 == t {
                sim.place_block(*pos, mc_tick::StateId::AIR);
            }
        }
        for (pos, at) in breaks {
            if *at == t {
                sim.place_block(*pos, mc_tick::StateId::AIR);
            }
        }
        for (pos, at) in places {
            if *at == t {
                sim.place_block(*pos, redstone_block);
            }
        }
        sim.step();
    }

    // Entity events -> dense per-tick position tracks. Positions persist
    // between movement events; a removed id's track ends.
    let mut tracks: Vec<ItemTrack> = Vec::new();
    let mut index_of: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    let mut cursor: std::collections::HashMap<u32, [f64; 3]> = std::collections::HashMap::new();
    let mut removed: std::collections::HashMap<u32, u64> = std::collections::HashMap::new();
    let mut item_names: std::collections::HashMap<u32, String> =
        sim.item_name_log().iter().cloned().collect();
    for cart in sim.minecarts() {
        item_names.insert(cart.id, cart.kind.clone());
    }
    for (tick, event) in sim.recorded_entities() {
        match event {
            mc_tick::sim::EntityEvent::Moved { id, pos, .. } => {
                if !index_of.contains_key(id) {
                    index_of.insert(*id, tracks.len());
                    tracks.push(ItemTrack {
                        item: item_names
                            .get(id)
                            .cloned()
                            .unwrap_or_else(|| "minecraft:redstone".to_string()),
                        positions: vec![None; ticks as usize],
                    });
                }
                cursor.insert(*id, *pos);
                let index = index_of[id];
                tracks[index].positions[*tick as usize] = Some(*pos);
            }
            mc_tick::sim::EntityEvent::Removed { id } => {
                removed.insert(*id, *tick);
            }
        }
    }
    // Fill gaps: a live, unmoving item keeps its last position.
    for (id, index) in &index_of {
        let end = removed.get(id).copied().unwrap_or(ticks);
        let mut last: Option<[f64; 3]> = None;
        for t in 0..ticks as usize {
            if (t as u64) >= end {
                tracks[*index].positions[t] = None;
                continue;
            }
            match tracks[*index].positions[t] {
                Some(p) => last = Some(p),
                None => tracks[*index].positions[t] = last,
            }
        }
    }

    let describe = |id| sim.registry().descriptor(id).unwrap_or("?").to_string();
    let changes = sim
        .recorded()
        .iter()
        .map(|c| (c.tick, c.pos, describe(c.from), describe(c.to)))
        .collect();
    (initial, changes, tracks)
}

/// Turn initial state + change log into drawable members.
fn build_cast(
    initial: &[(Pos, String)],
    changes: &[(u64, Pos, String, String)],
    ticks: u64,
) -> Vec<Member> {
    // Per-position state segments, in order: (state, start, end).
    let mut segments: HashMap<Pos, Vec<(String, u64, u64)>> = HashMap::new();
    for (pos, state) in initial {
        segments.insert(*pos, vec![(state.clone(), 0, ticks)]);
    }
    for (tick, pos, from, to) in changes {
        let list = segments.entry(*pos).or_default();
        if let Some(last) = list.last_mut() {
            debug_assert_eq!(&last.0, from, "log/timeline mismatch at {pos:?}");
            last.2 = *tick;
        } else if from != "minecraft:air" {
            // A position first mentioned by a change: it held `from` since tick 0.
            list.push((from.clone(), 0, *tick));
        }
        list.push((to.clone(), *tick, ticks));
    }

    // Who lost which state when — the source lookup for travelling blocks.
    let mut vacated: HashMap<(u64, String), Vec<Pos>> = HashMap::new();
    for (tick, pos, from, _) in changes {
        vacated.entry((*tick, from.clone())).or_default().push(*pos);
    }

    let mut members = Vec::new();
    for (pos, list) in &segments {
        for (index, (state, start, end)) in list.iter().enumerate() {
            if state == "minecraft:air" {
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

            // A flight: what does this placeholder become, and where from?
            let facing = facing_of(state);
            let landed = list.get(index + 1).map(|(s, _, _)| s.as_str());
            let previous = index.checked_sub(1).map(|i| list[i].0.as_str());
            match landed {
                Some(next) if is_extended_false_of(previous, next) => {
                    // A retracting base: the piston does not travel — its head
                    // slides back into it while the base shows its extended shell.
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
                        motion: Some(Motion {
                            from: head_slot,
                            until: *end as f64,
                        }),
                    });
                }
                Some(next) if next != "minecraft:air" && !next.starts_with("minecraft:moving_piston") => {
                    // A block in flight toward this position. Its source is the
                    // adjacent position that lost the same state on this tick;
                    // a piston head has no source block — it emerges from the base.
                    let from = vacated
                        .get(&(*start, next.to_string()))
                        .and_then(|list| list.iter().find(|p| adjacent(**p, *pos)))
                        .copied()
                        .unwrap_or_else(|| pos.offset(facing.opposite()));
                    members.push(Member {
                        pos: *pos,
                        state: next.to_string(),
                        start: *start as f64,
                        end: list[index + 1].2 as f64,
                        motion: Some(Motion {
                            from,
                            until: *end as f64,
                        }),
                    });
                    // Skip the landed segment: the flight member covers it.
                }
                _ => {
                    // Interrupted flight (finalTicked to air, or replaced by
                    // another placeholder): an emerging head that got yanked.
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

    // Landed segments already covered by a flight member (same pos, same state,
    // starting at the flight's landing) must not double-render: drop static
    // members fully contained in a flight member's interval.
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

fn facing_of(descriptor: &str) -> mc_tick::Dir {
    for (name, dir) in [
        ("facing=down", mc_tick::Dir::Down),
        ("facing=up", mc_tick::Dir::Up),
        ("facing=north", mc_tick::Dir::North),
        ("facing=south", mc_tick::Dir::South),
        ("facing=west", mc_tick::Dir::West),
        ("facing=east", mc_tick::Dir::East),
    ] {
        if descriptor.contains(name) {
            return dir;
        }
    }
    mc_tick::Dir::North
}

fn dir_name(dir: mc_tick::Dir) -> &'static str {
    match dir {
        mc_tick::Dir::Down => "down",
        mc_tick::Dir::Up => "up",
        mc_tick::Dir::North => "north",
        mc_tick::Dir::South => "south",
        mc_tick::Dir::West => "west",
        mc_tick::Dir::East => "east",
    }
}

fn type_of(moving_descriptor: &str) -> &'static str {
    if moving_descriptor.contains("type=sticky") {
        "sticky"
    } else {
        "normal"
    }
}

/// Whether `next` is `previous` with extended flipped true -> false — the
/// signature of a retracting piston base.
fn is_extended_false_of(previous: Option<&str>, next: &str) -> bool {
    let Some(previous) = previous else { return false };
    (next.starts_with("minecraft:piston[") || next.starts_with("minecraft:sticky_piston["))
        && next.contains("extended=false")
        && previous.replace("extended=true", "extended=false") == next
}

/// The head that visually retracts into `base`.
fn head_state_for(base: &str, facing: mc_tick::Dir) -> String {
    let kind = if base.starts_with("minecraft:sticky_piston") {
        "sticky"
    } else {
        "normal"
    };
    format!(
        "minecraft:piston_head[facing={},short=false,type={kind}]",
        dir_name(facing)
    )
}

/// The stand-in block for a fluid state, `None` for ordinary blocks.
fn fluid_proxy(state: &str) -> Option<&'static str> {
    if state.starts_with("minecraft:water") {
        Some("minecraft:blue_stained_glass")
    } else if state.starts_with("minecraft:bubble_column") {
        Some("minecraft:light_blue_stained_glass")
    } else {
        None
    }
}

/// The rendered surface height of a fluid state (vanilla's amount / 9).
fn fluid_height(state: &str) -> Option<f32> {
    if state.starts_with("minecraft:bubble_column") {
        return Some(1.0);
    }
    if !state.starts_with("minecraft:water") {
        return None;
    }
    let level: u8 = state
        .split("level=")
        .nth(1)
        .and_then(|rest| rest.trim_end_matches(']').parse().ok())
        .unwrap_or(0);
    Some(match level {
        0 => 8.0 / 9.0,
        1..=7 => f32::from(8 - level) / 9.0,
        _ => 1.0,
    })
}
