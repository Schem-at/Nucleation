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

    // ── 1. Simulate, recording every block change ───────────────────────────
    let (initial, changes) = simulate(snbt_path, ticks, &clicks);
    println!("simulated {ticks} ticks, {} block changes", changes.len());

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
                one.set_block_from_string(member.pos.x, member.pos.y, member.pos.z, &member.state)
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
    config.sphere_fit = true; // bounds cover the whole run, so the camera never drifts
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
fn simulate(
    snbt_path: &str,
    ticks: u64,
    clicks: &[(Pos, u64)],
) -> (Vec<(Pos, String)>, Vec<(u64, Pos, String, String)>) {
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

    sim.record();
    for t in 0..ticks {
        for (pos, at) in clicks {
            if *at == t {
                sim.use_block(*pos);
            }
        }
        sim.step();
    }

    let describe = |id| sim.registry().descriptor(id).unwrap_or("?").to_string();
    let changes = sim
        .recorded()
        .iter()
        .map(|c| (c.tick, c.pos, describe(c.from), describe(c.to)))
        .collect();
    (initial, changes)
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
