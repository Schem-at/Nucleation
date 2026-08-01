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
    let in_world = args.iter().any(|a| a == "--in-world");
    // Where the build sits in the game's coordinates; `updatePowerStrength`
    // iterates a HashSet whose order follows from absolute position.
    let hash_origin = flag(&args, "--origin")
        .map(|v| {
            let c: Vec<i32> = v.split(',').map(|p| p.parse().expect("--origin x,y,z")).collect();
            Pos::new(c[0], c[1], c[2])
        })
        .unwrap_or_default();
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
    let (initial, changes, mut item_tracks) = match flag(&args, "--trace") {
        Some(trace_path) => {
            let (initial, changes) = replay(snbt_path, trace_path, ticks);
            println!("replayed {ticks} ticks, {} block changes", changes.len());
            (initial, changes, Vec::new())
        }
        None => {
            let seed: Option<i64> = flag(&args, "--seed").map(|v| v.parse().expect("--seed N"));
            let out = simulate(snbt_path, ticks, &clicks, &pulses, &breaks, &places, settle, in_world, hash_origin, seed);
            println!("simulated {ticks} ticks, {} block changes", out.1.len());
            out
        }
    };

    // --entity-log <file>: per-tick entity positions in TraceCapture's
    // `E tN id=K kind pos=(x, y, z)` lines — carts, fireballs and blazes ride
    // along as posed meshes. Works for a capture and for an engine dump alike,
    // which is what makes the side-by-side videos carry their entities.
    if let Some(log_path) = flag(&args, "--entity-log") {
        let mut tracks = parse_entity_log(log_path, ticks);
        println!("entity log: {} tracks from {log_path}", tracks.len());
        item_tracks.append(&mut tracks);
    }
    let item_tracks = item_tracks;

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
                // Shulker boxes pass through unchanged: the mesher runs the
                // real block-entity model (base + lid, entity/shulker/*.png).
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
    // Measured world-space bounds of each track's mesh, so posing never rests
    // on assumptions about where a mesher path puts its geometry.
    let mut item_mesh_bounds: Vec<([f32; 3], [f32; 3])> = Vec::new();
    for track in &item_tracks {
        let mut one = UniversalSchematic::new("item".to_string());
        let cart_kind = track.kind.ends_with("minecart");
        let fireball_item = match track.kind.as_str() {
            // Vanilla renders both fireball entities as exactly these item
            // sprites, so the mesher's `entity:item` path *is* the real look.
            "minecraft:small_fireball" | "minecraft:fireball" => Some("minecraft:fire_charge"),
            "minecraft:dragon_fireball" => Some("minecraft:dragon_fireball"),
            _ => None,
        };
        if cart_kind {
            // The cart hull, oriented along its direction of travel; a cart
            // that never moves lies along x, which is every one of the record
            // door's parked furnace carts (`Rotation: [±90, 0]`).
            let pts: Vec<[f64; 3]> = track.positions.iter().flatten().copied().collect();
            let yaw: f32 = match (pts.first(), pts.last()) {
                (Some(a), Some(b)) if (b[0] - a[0]).abs() + (b[2] - a[2]).abs() > 1e-6 => {
                    let (dx, dz) = (b[0] - a[0], b[2] - a[2]);
                    if dx.abs() >= dz.abs() {
                        if dx >= 0.0 { -90.0 } else { 90.0 }
                    } else if dz >= 0.0 {
                        0.0
                    } else {
                        180.0
                    }
                }
                _ => -90.0,
            };
            let mut cart =
                nucleation::Entity::new("minecraft:minecart".to_string(), (0.5, 0.0, 0.5));
            cart.nbt.insert(
                "Rotation".to_string(),
                nucleation::NbtValue::List(vec![
                    nucleation::NbtValue::Float(yaw),
                    nucleation::NbtValue::Float(0.0),
                ]),
            );
            one.add_entity(cart);
        } else if let Some(item_id) = fireball_item {
            let mut ball = nucleation::Entity::new("minecraft:item".to_string(), (0.5, 0.0, 0.5));
            let mut item = std::collections::HashMap::new();
            item.insert("id".to_string(), nucleation::NbtValue::String(item_id.to_string()));
            ball.nbt.insert("Item".to_string(), nucleation::NbtValue::Compound(item));
            one.add_entity(ball);
        } else if !track.kind.is_empty() && track.kind != "minecraft:item" {
            // Any other entity kind goes to the mesher's entity path as-is —
            // zombies, villagers, boats, the whole mob roster. A kind without
            // a model meshes empty and simply does not draw.
            one.add_entity(nucleation::Entity::new(track.kind.clone(), (0.5, 0.0, 0.5)));
        } else if track.item == "minecraft:minecart" {
            // The real cart: meshed through the mesher's entity path — the
            // vanilla MinecartModel hull, UV'd from entity/minecart.png in
            // the client jar — oriented along its direction of travel.
            let pts: Vec<[f64; 3]> = track.positions.iter().flatten().copied().collect();
            let yaw: f32 = match (pts.first(), pts.last()) {
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
            let mut cart =
                nucleation::Entity::new("minecraft:minecart".to_string(), (0.5, 0.0, 0.5));
            cart.nbt.insert(
                "Rotation".to_string(),
                nucleation::NbtValue::List(vec![
                    nucleation::NbtValue::Float(yaw),
                    nucleation::NbtValue::Float(0.0),
                ]),
            );
            one.add_entity(cart);
        } else {
            // Shulker boxes pass through as blocks: the mesher's block-entity
            // path draws the real base + lid from entity/shulker/*.png. Other
            // item ids are not block ids; ingots and gems have a block form.
            let drawn = match track.item.as_str() {
                "minecraft:diamond" | "minecraft:emerald" | "minecraft:lapis_lazuli"
                | "minecraft:coal" | "minecraft:redstone" => format!("{}_block", track.item),
                other => other.replace("_ingot", "_block"),
            };
            if one.set_block_from_string(0, 0, 0, &drawn).is_err() {
                one.set_block_from_string(0, 0, 0, "minecraft:stone").ok();
            }
        }
        item_mesh_range.push(meshes.len());
        let mut mesh = one.to_mesh(&pack, &mesh_config)?;
        if mesh.opaque.positions.is_empty()
            && mesh.cutout.positions.is_empty()
            && mesh.transparent.positions.is_empty()
            && fireball_item.is_some()
        {
            // Not every fireball's own item resolves through the item-model
            // path (`minecraft:dragon_fireball` comes back empty); the
            // fire charge is the same class of sprite and scales to size.
            let mut retry = UniversalSchematic::new("item".to_string());
            let mut ball =
                nucleation::Entity::new("minecraft:item".to_string(), (0.5, 0.0, 0.5));
            let mut item = std::collections::HashMap::new();
            item.insert(
                "id".to_string(),
                nucleation::NbtValue::String("minecraft:fire_charge".to_string()),
            );
            ball.nbt.insert("Item".to_string(), nucleation::NbtValue::Compound(item));
            retry.add_entity(ball);
            mesh = retry.to_mesh(&pack, &mesh_config)?;
        }
        let mut lo = [f32::INFINITY; 3];
        let mut hi = [f32::NEG_INFINITY; 3];
        for layer in [&mesh.opaque, &mesh.cutout, &mesh.transparent] {
            for v in &layer.positions {
                for axis in 0..3 {
                    lo[axis] = lo[axis].min(v[axis]);
                    hi[axis] = hi[axis].max(v[axis]);
                }
            }
        }
        println!(
            "item mesh {} ({} {}): {} verts, bounds {:?}..{:?}",
            meshes.len(),
            track.kind,
            track.item,
            mesh.opaque.positions.len() + mesh.cutout.positions.len()
                + mesh.transparent.positions.len(),
            lo,
            hi
        );
        item_mesh_bounds.push((lo, hi));
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
                let (lo, hi) = item_mesh_bounds[track_index];
                let center = [
                    (lo[0] + hi[0]) * 0.5,
                    (lo[1] + hi[1]) * 0.5,
                    (lo[2] + hi[2]) * 0.5,
                ];
                let size = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
                let cart =
                    track.kind.ends_with("minecart") || track.item == "minecraft:minecart";
                // Anchor the *measured* mesh box onto the entity's hitbox:
                // scale about the mesh centre, then move that centre onto the
                // target point. No assumptions about any mesher path's origin.
                let (scale, target): (f32, [f32; 3]) = if cart {
                    // The vanilla cart model, bottom-centre on the feet,
                    // unscaled — the hull is wider than the 0.98 box in the
                    // game too. A hair down so an entombed cart's floor-flush
                    // rim does not z-fight its block.
                    (1.0, [p[0] as f32, p[1] as f32 - 0.005 + size[1] * 0.5, p[2] as f32])
                } else if track.kind == "minecraft:small_fireball"
                    || track.kind == "minecraft:fireball"
                    || track.kind == "minecraft:dragon_fireball"
                {
                    // The item sprite scaled to the fireball's height, centred
                    // on its box.
                    let height: f32 =
                        if track.kind == "minecraft:dragon_fireball" { 1.0 } else { 0.3125 };
                    let extent = size[0].max(size[1]).max(size[2]).max(1.0e-4);
                    (height / extent, [p[0] as f32, p[1] as f32 + height * 0.5, p[2] as f32])
                } else if !track.kind.is_empty() && track.kind != "minecraft:item" {
                    // A mob model draws at its authored size, anchored by the
                    // model's own ground plane — mesh-frame y = -0.5, where a
                    // zombie's soles sit — not by its bounding box. The
                    // difference is every mob whose visual floats: a blaze's
                    // lowest rod hangs 0.2525 above its feet, and bbox
                    // anchoring would plant it on the ground.
                    (1.0, [p[0] as f32, p[1] as f32 + 0.5 + center[1], p[2] as f32])
                } else {
                    // Dropped items draw at vanilla's quarter block scale,
                    // hovering an eighth above the entity position.
                    let extent = size[0].max(size[1]).max(size[2]).max(1.0e-4);
                    (0.25 / extent, [p[0] as f32, p[1] as f32 + 0.125, p[2] as f32])
                };
                pose.scale = [scale; 3];
                pose.pivot = center;
                // The mesher emits entity geometry offset from the block grid
                // by (+0.5, +0.25, +0.5) relative to its own vertex data —
                // measured on the calibration grid, where every kind sat half
                // a block east of its marker and a quarter block above it.
                pose.translate = [
                    target[0] - center[0] - 0.5,
                    target[1] - center[1] - 0.25,
                    target[2] - center[2] - 0.5,
                ];
                // A sprite meshed lying flat (thin in y) is stood upright so
                // the camera sees it face-on; rotation is about the mesh
                // centre, so the anchoring above is unaffected.
                if track.kind.contains("fireball") && size[1] < size[0].min(size[2]) * 0.5 {
                    pose.rotate_deg = [90.0, 0.0, 0.0];
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
    /// The entity type, e.g. `minecraft:furnace_minecart`; `minecraft:item`
    /// for a dropped item, whose id is then in `item`.
    kind: String,
    item: String,
    positions: Vec<Option<[f64; 3]>>,
}

/// Parse a capture's `--entity-log` lines into dense per-tick tracks.
///
/// Both sides of a comparison speak this format: TraceCapture prints every
/// entity every tick as `  E tN id=K minecraft:kind pos=(x, y, z) ...`, and
/// the engine-side trace dumper emits the same lines. Positions are absolute,
/// riders included, so no seat math is needed here.
fn parse_entity_log(path: &str, ticks: u64) -> Vec<ItemTrack> {
    let text = std::fs::read_to_string(path).expect("read entity log");
    let mut index_of: HashMap<u32, usize> = HashMap::new();
    let mut tracks: Vec<ItemTrack> = Vec::new();
    for line in text.lines() {
        let line = line.trim_start();
        let Some(rest) = line.strip_prefix("E t") else { continue };
        let mut parts = rest.split_whitespace();
        let Some(tick) = parts.next().and_then(|v| v.parse::<u64>().ok()) else { continue };
        let Some(id) = parts
            .next()
            .and_then(|v| v.strip_prefix("id="))
            .and_then(|v| v.parse::<u32>().ok())
        else {
            continue;
        };
        let Some(kind) = parts.next() else { continue };
        let Some(pos_text) = line.split("pos=(").nth(1).and_then(|v| v.split(')').next())
        else {
            continue;
        };
        let coords: Vec<f64> = pos_text
            .split(',')
            .filter_map(|v| v.trim().parse::<f64>().ok())
            .collect();
        if coords.len() != 3 || tick >= ticks {
            continue;
        }
        let index = *index_of.entry(id).or_insert_with(|| {
            tracks.push(ItemTrack {
                kind: kind.to_string(),
                item: String::new(),
                positions: vec![None; ticks as usize],
            });
            tracks.len() - 1
        });
        tracks[index].positions[tick as usize] = Some([coords[0], coords[1], coords[2]]);
    }
    // A logged entity holds its last position through any gap.
    for track in &mut tracks {
        let mut last: Option<[f64; 3]> = None;
        for slot in &mut track.positions {
            match slot {
                Some(p) => last = Some(*p),
                None => *slot = last,
            }
        }
    }
    tracks
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
    in_world: bool,
    hash_origin: Pos,
    seed: Option<i64>,
) -> (
    Vec<(Pos, String)>,
    Vec<(u64, Pos, String, String)>,
    Vec<ItemTrack>,
) {
    let text = std::fs::read_to_string(snbt_path).expect("read structure");
    let structure = Structure::parse(&text).expect("parse structure");

    let mut sim = Simulation::new(structure.bounds(4));
    if let Some(seed) = seed {
        sim.set_rng_seed(seed);
    }
    {
        let (registry, world) = sim.registry_and_world_mut();
        structure.place(world, registry, Pos::new(0, 0, 0));
    }
    // A dispenser can *place* a shulker box it holds as an item; behaviours
    // bind only to interned states, so intern every facing up front.
    for (_, stacks) in &structure.inventories {
        for stack in stacks {
            let base = stack.id.split('[').next().unwrap_or(&stack.id);
            if base.ends_with("_shulker_box") || base == "minecraft:shulker_box" {
                for facing in ["up", "down", "north", "south", "west", "east"] {
                    let _ = sim.registry_mut().intern(&format!("{base}[facing={facing}]"));
                }
            }
        }
    }
    mc_tick::intern_companions(sim.registry_mut());
    {
        let mut table = std::mem::take(sim.behaviours_mut());
        mc_tick::register_all_at(sim.registry_mut(), &mut table, hash_origin);
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
                let vehicle = sim.spawn_authored_minecart(cart, None);
                for rider in &cart.passengers {
                    sim.spawn_authored_rider(vehicle, rider).expect("rider spawn");
                }
            }
            mc_tick::structure::SpawnedEntity::FurnaceMinecart(cart) => {
                sim.spawn_authored_furnace_minecart(cart, None).expect("furnace cart spawn");
            }
            mc_tick::structure::SpawnedEntity::Body(body) => {
                sim.spawn_authored_body(body).expect("body spawn");
            }
        }
    }
    // The same block-entity setup the conformance harness does. Without it a
    // comparator has no stored `OutputSignal`, so a door whose memory cell is
    // held by one comes up unlatched and never closes — which is the very
    // failure that made pasted captures useless, reproduced here in the
    // renderer while the engine itself was exact.
    for pos in &structure.block_entities {
        sim.mark_block_entity(*pos);
    }
    for (pos, strength) in &structure.comparator_outputs {
        sim.set_comparator_output(*pos, *strength);
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
        mc_tick::register_all_at(sim.registry_mut(), &mut table, hash_origin);
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

    // `onPlace` runs on every write whatever the flags, so a placed build gets
    // it even for a quiet placement; `--in-world` is for a recording taken where
    // the build already stood, which is placed by nothing at all.
    let order = structure.placement_order(
        mc_tick::vanilla::is_collision_full_cube,
        mc_tick::vanilla::has_dynamic_shape,
    );
    if !in_world {
        sim.place_on_place(&order);
        if settle {
            sim.settle_with_order(&order);
        }
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
                        kind: "minecraft:item".to_string(),
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
