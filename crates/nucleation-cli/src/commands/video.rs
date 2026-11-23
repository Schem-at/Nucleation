//! Standard CLI scene simulation and video export. Physics stays in mc-tick;
//! moving visuals use its carried-block records rather than inferring strokes.
use super::scene::{position, value};
use mc_test::mc_tick::{MovingBlock, Pos, Simulation, StateId, Structure};
use mc_test::SettleMode;
use nucleation::animation::{CameraPose, Frame, Pose};
use nucleation::meshing::{MeshConfig, MeshOutput, ResourcePackSource};
use nucleation::rendering::{RenderConfig, VideoConfig};
use nucleation::{Entity, NbtValue, UniversalSchematic};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

const HELP: &str = "nucleation render INPUT -o OUTPUT [--pack PACK.zip]\n\
  OUTPUT: .png, .mp4 (H.264), .webm (VP9 alpha), .mov (ProRes 4444)\n\
  --width 1280 --height 720 --yaw 45 --pitch 35.264 --zoom 1 --tight\n\
  --ticks 100 --tick-ms 50 --fps 60 --seed 0 --settle in-world\n\
  --use TICK:X,Y,Z              use a lever/button/note block\n\
  --place TICK:X,Y,Z=STATE      place any block; --break TICK:X,Y,Z\n\
  --camera TICK:YAW,PITCH,ZOOM[,PAN_X,PAN_Y,PAN_Z] (repeat keyframes)\n\
  --transparent --ffmpeg PATH --origin X,Y,Z --report result.json\n\
  Actions run at tick boundaries, before stepping that tick; 0 is the start.\n\
  Positions use input schematic coordinates; pan is relative to the camera target.\n\
  Video needs an output filename and external FFmpeg. PNG accepts - for stdout.";

#[derive(Clone, Debug)]
struct Action {
    tick: u64,
    pos: [i32; 3],
    state: Option<String>,
}
#[derive(Clone, Debug)]
struct CameraKey {
    tick: u64,
    values: [f32; 6],
}
struct Options {
    input: PathBuf,
    output: PathBuf,
    pack: PathBuf,
    config: RenderConfig,
    ticks: u64,
    tick_ms: f64,
    fps: f64,
    seed: i64,
    origin: Pos,
    settle: SettleMode,
    actions: Vec<Action>,
    camera: Vec<CameraKey>,
    ffmpeg: Option<PathBuf>,
    report: Option<PathBuf>,
    video: bool,
}

pub(crate) fn render_main(args: impl Iterator<Item = String>) {
    let args: Vec<_> = args.collect();
    if args.iter().any(|s| s == "--help") {
        println!("{HELP}");
        return;
    }
    if let Err(error) = parse(args.into_iter()).and_then(render) {
        eprintln!("{error}");
        std::process::exit(2);
    }
}
fn number<T: std::str::FromStr>(s: &str, label: &str) -> Result<T, String> {
    s.parse().map_err(|_| format!("Invalid {label}: {s:?}"))
}
fn timed(s: &str) -> Result<(u64, [i32; 3]), String> {
    let (t, p) = s.split_once(':').ok_or("Action needs TICK:X,Y,Z")?;
    Ok((number(t, "tick")?, position(p)?))
}
fn parse(mut args: impl Iterator<Item = String>) -> Result<Options, String> {
    let (mut input, mut output, mut pack) = (None, None, None);
    let mut config = RenderConfig::isometric();
    let (mut width_set, mut height_set) = (false, false);
    let (mut ticks, mut tick_ms, mut fps, mut seed) = (100u64, 50.0f64, 60.0f64, 0i64);
    let mut origin = Pos::default();
    let mut settle = SettleMode::InWorld;
    let (mut actions, mut camera) = (Vec::new(), Vec::new());
    let (mut ffmpeg, mut report) = (None, None);
    let mut video_only = false;
    let mut tight = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--out" => output = Some(PathBuf::from(value(&mut args, &arg)?)),
            "--pack" => pack = Some(PathBuf::from(value(&mut args, &arg)?)),
            "--width" => {
                config.width = number(&value(&mut args, &arg)?, "width")?;
                width_set = true;
            }
            "--height" => {
                config.height = number(&value(&mut args, &arg)?, "height")?;
                height_set = true;
            }
            "--yaw" => config.yaw = number(&value(&mut args, &arg)?, "yaw")?,
            "--pitch" => config.pitch = number(&value(&mut args, &arg)?, "pitch")?,
            "--zoom" => config.zoom = number(&value(&mut args, &arg)?, "zoom")?,
            "--tight" => tight = true,
            "--transparent" => config.background = Some([0.0; 4]),
            "--ticks" => {
                ticks = number(&value(&mut args, &arg)?, "ticks")?;
                video_only = true;
            }
            "--tick-ms" => {
                tick_ms = number(&value(&mut args, &arg)?, "tick-ms")?;
                video_only = true;
            }
            "--fps" => {
                fps = number(&value(&mut args, &arg)?, "fps")?;
                video_only = true;
            }
            "--seed" => {
                seed = number(&value(&mut args, &arg)?, "seed")?;
                video_only = true;
            }
            "--origin" => {
                let p = position(&value(&mut args, &arg)?)?;
                origin = Pos::new(p[0], p[1], p[2]);
                video_only = true;
            }
            "--settle" => {
                video_only = true;
                settle = match value(&mut args, &arg)?.as_str() {
                    "placement" => SettleMode::Placement,
                    "quiet" => SettleMode::Quiet,
                    "in-world" => SettleMode::InWorld,
                    _ => return Err("--settle must be placement, quiet, or in-world".into()),
                };
            }
            "--use" | "--break" | "--place" => {
                let v = value(&mut args, &arg)?;
                let (target, state) = if arg == "--place" {
                    let (p, s) = v.split_once('=').ok_or("--place needs TICK:X,Y,Z=STATE")?;
                    if s.is_empty() {
                        return Err("Empty block state".into());
                    }
                    (p, Some(s.to_string()))
                } else {
                    (
                        v.as_str(),
                        if arg == "--break" {
                            Some("minecraft:air".into())
                        } else {
                            None
                        },
                    )
                };
                let (tick, pos) = timed(target)?;
                actions.push(Action { tick, pos, state });
                video_only = true;
            }
            "--camera" => {
                let v = value(&mut args, &arg)?;
                let (t, xyz) = v
                    .split_once(':')
                    .ok_or("--camera needs TICK:YAW,PITCH,ZOOM[,PAN_X,PAN_Y,PAN_Z]")?;
                let v: Vec<f32> = xyz
                    .split(',')
                    .map(|s| number(s, "camera coordinate"))
                    .collect::<Result<_, _>>()?;
                if v.len() != 3 && v.len() != 6 {
                    return Err("Camera needs 3 or 6 values".into());
                }
                let mut values = [0.0; 6];
                values[..v.len()].copy_from_slice(&v);
                if !values.iter().all(|n| n.is_finite()) || values[2] <= 0.0 {
                    return Err("Camera values must be finite and zoom positive".into());
                }
                camera.push(CameraKey {
                    tick: number(t, "camera tick")?,
                    values,
                });
                video_only = true;
            }
            "--ffmpeg" => ffmpeg = Some(PathBuf::from(value(&mut args, &arg)?)),
            "--report" => report = Some(PathBuf::from(value(&mut args, &arg)?)),
            "-" => {
                if input.replace(PathBuf::from(arg)).is_some() {
                    return Err("Only one input is allowed".into());
                }
            }
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown render option {arg}; see render --help"))
            }
            _ => {
                if input.replace(PathBuf::from(arg)).is_some() {
                    return Err("Only one input is allowed".into());
                }
            }
        }
    }
    let input = input.ok_or("render requires INPUT; see render --help")?;
    let output = output.ok_or("render requires -o OUTPUT")?;
    let ext = output
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let video = match ext.as_str() {
        "mp4" | "webm" | "mov" => true,
        "png" => false,
        _ if super::io::is_stdio(&output) => false,
        _ => return Err("Output must be .png, .mp4, .webm or .mov".into()),
    };
    if video {
        if !width_set {
            config.width = 1280;
        }
        if !height_set {
            config.height = 720;
        }
        config.sphere_fit = !tight;
    }
    if !video && (video_only || report.is_some() || ffmpeg.is_some()) {
        return Err(
            "Simulation, camera animation, reports and FFmpeg options require video output".into(),
        );
    }
    if config.width == 0 || config.height == 0 || config.width > 8192 || config.height > 8192 {
        return Err("Dimensions must be in 1..8192".into());
    }
    if ![config.yaw, config.pitch, config.zoom]
        .iter()
        .all(|x| x.is_finite())
        || config.zoom <= 0.0
    {
        return Err("Camera values must be finite and zoom positive".into());
    }
    if ticks == 0
        || ticks > 100_000
        || !tick_ms.is_finite()
        || tick_ms <= 0.0
        || !fps.is_finite()
        || !(1.0..=240.0).contains(&fps)
    {
        return Err("Use 1..100000 ticks, positive finite tick-ms and 1..240 fps".into());
    }
    if (ticks as f64 * tick_ms * fps / 1000.0).ceil() > 200_000.0 {
        return Err("Video exceeds 200000 frames; shorten the run or lower fps".into());
    }
    if actions.iter().any(|a| a.tick >= ticks) {
        return Err("Actions must precede the end tick".into());
    }
    actions.sort_by_key(|a| a.tick); // stable: preserve CLI order within a tick
    camera.sort_by_key(|c| c.tick);
    if camera.iter().any(|c| c.tick > ticks) || camera.windows(2).any(|w| w[0].tick == w[1].tick) {
        return Err("Camera ticks must be unique and within the run".into());
    }
    if camera.first().is_some_and(|c| c.tick != 0) {
        camera.insert(
            0,
            CameraKey {
                tick: 0,
                values: [config.yaw, config.pitch, config.zoom, 0.0, 0.0, 0.0],
            },
        );
    }
    if ext == "mp4"
        && (config.width % 2 != 0 || config.height % 2 != 0 || config.background.is_some())
    {
        return Err(
            "H.264 requires even dimensions and opaque output; use WebM/MOV for transparency"
                .into(),
        );
    }
    let pack = pack
        .or_else(super::pack::discover_pack)
        .ok_or("Pass --pack PACK.zip, set NUCLEATION_PACK, or install Minecraft")?;
    if report
        .as_ref()
        .is_some_and(|p| p == &input || p == &output || super::io::is_stdio(p))
    {
        return Err("Report needs a separate file path".into());
    }
    if input == output && !super::io::is_stdio(&input) {
        return Err("Input and output must be different files".into());
    }
    Ok(Options {
        input,
        output,
        pack,
        config,
        ticks,
        tick_ms,
        fps,
        seed,
        origin,
        settle,
        actions,
        camera,
        ffmpeg,
        report,
        video,
    })
}

fn camera_at(keys: &[CameraKey], t: f64, base: &RenderConfig) -> CameraPose {
    let values = if keys.is_empty() {
        [base.yaw, base.pitch, base.zoom, 0.0, 0.0, 0.0]
    } else {
        let right = keys.partition_point(|k| k.tick as f64 <= t);
        if right == 0 {
            keys[0].values
        } else if right == keys.len() {
            keys[right - 1].values
        } else {
            let a = &keys[right - 1];
            let b = &keys[right];
            let f = ((t - a.tick as f64) / (b.tick - a.tick) as f64) as f32;
            std::array::from_fn(|i| a.values[i] + (b.values[i] - a.values[i]) * f)
        }
    };
    CameraPose {
        yaw: values[0] - base.yaw,
        pitch: values[1] - base.pitch,
        zoom: values[2] / base.zoom,
        target_offset: [values[3], values[4], values[5]],
    }
}

// One constant state span, or one engine-owned moving-block span. The same
// position/state mesh is reused across spans. Pixel frames are never retained.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
struct Key {
    pos: Pos,
    state: StateId,
    from: Pos,
    started: u64,
    lands: u64,
    kind: u8,
    extending: bool,
}
impl Key {
    fn block(pos: Pos, state: StateId) -> Self {
        Self {
            pos,
            state,
            from: pos,
            started: 0,
            lands: 0,
            kind: 0,
            extending: false,
        }
    }
    fn flight(m: MovingBlock, state: StateId, kind: u8) -> Self {
        Self {
            pos: m.to,
            state,
            from: m.from,
            started: m.started_on,
            lands: m.lands_on,
            kind,
            extending: m.extending,
        }
    }
}
struct Span {
    key: Key,
    start: u64,
    end: u64,
    mesh: usize,
}
#[derive(Default)]
struct RunVisuals {
    spans: Vec<Span>,
    active: HashMap<Key, usize>,
    entities: BTreeMap<u32, EntityTrack>,
}
struct EntityTrack {
    kind: String,
    item: Option<String>,
    samples: BTreeMap<u64, [f64; 3]>,
}
impl RunVisuals {
    fn capture(&mut self, sim: &Simulation, tick: u64) {
        let mut keys = HashSet::new();
        for (pos, state) in sim.world().iter_non_air() {
            if !sim
                .registry()
                .descriptor(state)
                .unwrap_or("")
                .starts_with("minecraft:moving_piston")
            {
                keys.insert(Key::block(pos, state));
            }
        }
        for m in sim.moving_blocks() {
            if let Some(s) = m.remains {
                keys.insert(Key::block(m.to, s));
            }
            keys.insert(Key::flight(
                m,
                m.carried,
                if m.carried_short.is_some() { 1 } else { 3 },
            ));
            if let Some(s) = m.carried_short {
                keys.insert(Key::flight(m, s, 2));
            }
        }
        self.active.retain(|k, _| keys.contains(k));
        let mut ordered: Vec<_> = keys.into_iter().collect();
        ordered.sort_by_key(|k| (k.pos, k.state.raw(), k.kind, k.started));
        for key in ordered {
            if let Some(i) = self.active.get(&key) {
                self.spans[*i].end = tick + 1;
            } else {
                let index = self.spans.len();
                self.spans.push(Span {
                    key,
                    start: tick,
                    end: tick + 1,
                    mesh: 0,
                });
                self.active.insert(key, index);
            }
        }
        let mut entities = Vec::new();
        for c in sim.minecarts().iter().filter(|c| !c.removed) {
            entities.push((c.id, c.kind.clone(), None, c.pos));
        }
        for e in sim.item_entities().iter().filter(|e| !e.removed) {
            entities.push((e.id, "minecraft:item".into(), Some(e.item.0.clone()), e.pos));
        }
        for e in sim.entity_bodies().iter().filter(|e| !e.is_minecart) {
            entities.push((
                e.id,
                e.kind.clone(),
                None,
                [
                    (e.min[0] + e.max[0]) / 2.0,
                    e.min[1],
                    (e.min[2] + e.max[2]) / 2.0,
                ],
            ));
        }
        for (id, kind, item, pos) in entities {
            self.entities
                .entry(id)
                .or_insert_with(|| EntityTrack {
                    kind,
                    item,
                    samples: BTreeMap::new(),
                })
                .samples
                .insert(tick, pos);
        }
    }
}
fn frame_pose(span: &Span, t: f64) -> Option<Pose> {
    if t < span.start as f64 || t >= span.end as f64 {
        return None;
    }
    let mut pose = Pose::IDENTITY;
    if span.key.kind != 0 {
        let k = span.key;
        let span = k.lands.saturating_sub(k.started);
        let p = if span == 0 {
            1.0
        } else {
            ((t - k.started as f64) / span as f64).clamp(0.0, 1.0)
        };
        let short = if k.extending { p <= 0.5 } else { p >= 0.5 };
        if (k.kind == 1 && short) || (k.kind == 2 && !short) {
            return None;
        }
        pose.translate = [
            (k.from.x - k.pos.x) as f32 * (1.0 - p as f32),
            (k.from.y - k.pos.y) as f32 * (1.0 - p as f32),
            (k.from.z - k.pos.z) as f32 * (1.0 - p as f32),
        ];
    }
    Some(pose)
}

// Mesher region bounds describe block cells; vertices are block-centred and
// some models extend beyond their cell. Fit to geometry, then include every
// translated endpoint once before rendering so the camera never follows a
// changing per-frame silhouette.
fn fit_mesh_bounds(mesh: &mut MeshOutput) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for p in mesh
        .opaque
        .positions
        .iter()
        .chain(&mesh.cutout.positions)
        .chain(&mesh.transparent.positions)
        .chain(
            mesh.greedy_materials
                .iter()
                .flat_map(|m| m.opaque.positions.iter().chain(&m.transparent.positions)),
        )
    {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    if min[0].is_finite() {
        mesh.bounds.min = min;
        mesh.bounds.max = max;
    }
}

fn include_translation(mesh: &mut MeshOutput, base: ([f32; 3], [f32; 3]), delta: [f32; 3]) {
    for axis in 0..3 {
        mesh.bounds.min[axis] = mesh.bounds.min[axis].min(base.0[axis] + delta[axis]);
        mesh.bounds.max[axis] = mesh.bounds.max[axis].max(base.1[axis] + delta[axis]);
    }
}

fn simulate(
    source: &UniversalSchematic,
    o: &Options,
) -> Result<(Simulation, RunVisuals, [i32; 3]), String> {
    let bb = source.get_bounding_box();
    let offset = [bb.min.0, bb.min.1, bb.min.2];
    let snbt = nucleation::formats::gametest::to_gametest_snbt(source);
    let structure = Structure::parse(&snbt).map_err(|e| format!("Cannot simulate input: {e}"))?;
    let extra: Vec<_> = o.actions.iter().filter_map(|a| a.state.clone()).collect();
    let version = source
        .metadata
        .source_data_version
        .or(source.metadata.mc_version);
    let mut sim = mc_test::try_build_sim(
        &structure,
        o.origin,
        o.settle,
        &extra,
        &[],
        version,
        "CLI render",
    )?;
    sim.set_rng_seed(o.seed);
    sim.discard_recording();
    let mut visuals = RunVisuals::default();
    for tick in 0..o.ticks {
        for a in o.actions.iter().filter(|a| a.tick == tick) {
            let mut p = [0; 3];
            for i in 0..3 {
                p[i] = a.pos[i]
                    .checked_sub(offset[i])
                    .ok_or("Action coordinate overflow")?;
            }
            let p = Pos::new(p[0], p[1], p[2]);
            if let Some(s) = &a.state {
                let state = sim
                    .registry()
                    .get(s)
                    .ok_or_else(|| format!("Unregistered action state {s}"))?;
                sim.place_block(p, state);
            } else {
                sim.use_block(p);
            }
        }
        sim.step();
        if sim.tick_count() != tick + 1 {
            return Err(format!("Simulation stopped before tick {}", tick + 1));
        }
        visuals.capture(&sim, tick);
    }
    Ok((sim, visuals, offset))
}
fn render(o: Options) -> Result<(), String> {
    let source = super::scene::load(&o.input)?;
    let pack = ResourcePackSource::from_file(&o.pack.display().to_string())
        .map_err(|e| format!("Loading pack: {e:?}"))?;
    if !o.video {
        let mesh = source
            .to_mesh(&pack, &MeshConfig::default())
            .map_err(|e| format!("Meshing: {e:?}"))?;
        let bytes = nucleation::rendering::render_meshes_png(&[mesh], &o.config, None)
            .map_err(|e| e.to_string())?;
        super::io::write_output(&o.output, &bytes)?;
        super::io::status(
            &format!(
                "Rendered {} -> {}",
                o.input.display(),
                super::io::display_out(&o.output)
            ),
            super::io::is_stdio(&o.output),
        );
        return Ok(());
    }
    let (sim, mut visuals, offset) = simulate(&source, &o)?;
    eprintln!(
        "Simulated {} ticks; meshing {} visible spans",
        o.ticks,
        visuals.spans.len()
    );
    let mut meshes: Vec<MeshOutput> = Vec::new();
    let mut cache = HashMap::new();
    let mut static_scene = UniversalSchematic::new("constant blocks".into());
    for span in &visuals.spans {
        if span.start == 0 && span.end == o.ticks && span.key.kind == 0 {
            let p = span.key.pos;
            static_scene.set_block_from_string(
                p.x,
                p.y,
                p.z,
                sim.registry()
                    .descriptor(span.key.state)
                    .ok_or("Missing registry state")?,
            )?;
            let original = nucleation::block_position::BlockPosition {
                x: p.x + offset[0],
                y: p.y + offset[1],
                z: p.z + offset[2],
            };
            if let Some(be) = source.get_block_entity(original) {
                let mut be = be.clone();
                be.position = (p.x, p.y, p.z);
                static_scene.set_block_entity(
                    nucleation::block_position::BlockPosition {
                        x: p.x,
                        y: p.y,
                        z: p.z,
                    },
                    be,
                );
            }
        }
    }
    let has_static = static_scene.iter_blocks().next().is_some();
    if has_static {
        meshes.push(
            static_scene
                .to_mesh(&pack, &MeshConfig::default())
                .map_err(|e| format!("Meshing constant blocks: {e:?}"))?,
        );
        visuals
            .spans
            .retain(|s| !(s.start == 0 && s.end == o.ticks && s.key.kind == 0));
    }
    for span in &mut visuals.spans {
        let key = (span.key.pos, span.key.state);
        if let Some(i) = cache.get(&key) {
            span.mesh = *i;
            continue;
        }
        let mut one = UniversalSchematic::new("video block".into());
        let p = span.key.pos;
        let state = sim
            .registry()
            .descriptor(span.key.state)
            .ok_or("Missing registry state")?;
        one.set_block_from_string(p.x, p.y, p.z, state)?;
        let original = nucleation::block_position::BlockPosition {
            x: p.x + offset[0],
            y: p.y + offset[1],
            z: p.z + offset[2],
        };
        if let Some(be) = source.get_block_entity(original) {
            let mut be = be.clone();
            be.position = (p.x, p.y, p.z);
            one.set_block_entity(
                nucleation::block_position::BlockPosition {
                    x: p.x,
                    y: p.y,
                    z: p.z,
                },
                be,
            );
        }
        let mesh = one
            .to_mesh(&pack, &MeshConfig::default())
            .map_err(|e| format!("Meshing {state}: {e:?}"))?;
        span.mesh = meshes.len();
        cache.insert(key, span.mesh);
        meshes.push(mesh);
    }
    let mut entity_meshes = Vec::new();
    for track in visuals.entities.values() {
        let first = *track.samples.values().next().ok_or("Empty entity track")?;
        let mut one = UniversalSchematic::new("video entity".into());
        let mut entity = Entity::new(track.kind.clone(), (first[0], first[1], first[2]));
        if let Some(item) = &track.item {
            entity.nbt.insert(
                "Item".into(),
                NbtValue::Compound(HashMap::from([
                    ("id".into(), NbtValue::String(item.clone())),
                    ("Count".into(), NbtValue::Byte(1)),
                ])),
            );
        }
        one.add_entity(entity);
        let mesh = one
            .to_mesh(&pack, &MeshConfig::default())
            .map_err(|e| format!("Meshing {}: {e:?}", track.kind))?;
        if mesh.total_vertices() == 0 {
            eprintln!("warning: no visual model for {}", track.kind);
        }
        entity_meshes.push((meshes.len(), first, track));
        meshes.push(mesh);
    }
    if meshes.is_empty() {
        return Err("Nothing to render".into());
    }
    for mesh in &mut meshes {
        fit_mesh_bounds(mesh);
    }
    let rest_bounds: Vec<_> = meshes
        .iter()
        .map(|m| (m.bounds.min, m.bounds.max))
        .collect();
    for span in &visuals.spans {
        let k = span.key;
        if k.kind != 0 {
            include_translation(
                &mut meshes[span.mesh],
                rest_bounds[span.mesh],
                [
                    (k.from.x - k.pos.x) as f32,
                    (k.from.y - k.pos.y) as f32,
                    (k.from.z - k.pos.z) as f32,
                ],
            );
        }
    }
    for (i, first, track) in &entity_meshes {
        for position in track.samples.values() {
            include_translation(
                &mut meshes[*i],
                rest_bounds[*i],
                std::array::from_fn(|a| (position[a] - first[a]) as f32),
            );
        }
    }
    let frame_count = (o.ticks as f64 * o.tick_ms * o.fps / 1000.0).ceil() as usize;
    let hidden = Pose {
        scale: [0.0; 3],
        opacity: 0.0,
        ..Pose::IDENTITY
    };
    let frames = (0..frame_count).map(|index| {
        let t = index as f64 * 1000.0 / o.fps / o.tick_ms;
        let mut poses = vec![hidden; meshes.len()];
        if has_static {
            poses[0] = Pose::IDENTITY;
        }
        for span in &visuals.spans {
            if let Some(p) = frame_pose(span, t) {
                poses[span.mesh] = p;
            }
        }
        for (i, base, track) in &entity_meshes {
            let tick = t.floor() as u64;
            if let Some(a) = track.samples.get(&tick) {
                let b = track.samples.get(&(tick + 1)).unwrap_or(a);
                let f = t - t.floor();
                let mut p = Pose::IDENTITY;
                p.translate = std::array::from_fn(|axis| {
                    (a[axis] + (b[axis] - a[axis]) * f - base[axis]) as f32
                });
                poses[*i] = p;
            }
        }
        Frame {
            time_ms: (index as f64 * 1000.0 / o.fps) as f32,
            poses: poses
                .into_iter()
                .enumerate()
                .map(|(i, p)| (i as u32, p))
                .collect(),
            camera: Some(camera_at(&o.camera, t, &o.config)),
            gizmos: Vec::new(),
            anchors: Vec::new(),
        }
    });
    let mut video = match o
        .output
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "webm" => VideoConfig::vp9_alpha(o.fps),
        "mov" => VideoConfig::prores_4444(o.fps),
        _ => VideoConfig::h264(o.fps),
    }?;
    if let Some(path) = &o.ffmpeg {
        video.set_ffmpeg_path(path);
    }
    nucleation::rendering::render_animation_to_video_iter(
        &meshes, frames, &o.config, None, &video, &o.output,
    )
    .map_err(|e| e.to_string())?;
    if let Some(path) = &o.report {
        let blocks:Vec<_>=sim.world().iter_non_air().map(|(p,s)|serde_json::json!({"pos":[p.x+offset[0],p.y+offset[1],p.z+offset[2]],"state":sim.registry().descriptor(s)})).collect();
        let report = serde_json::json!({"ticks":sim.tick_count(),"quiescent":sim.is_quiescent(),"frames":frame_count,"fps":o.fps,"tick_ms":o.tick_ms,"input_origin":offset,"meshes":meshes.len(),"spans":visuals.spans.len(),"final_blocks":blocks,"camera_start":camera_at(&o.camera,0.0,&o.config),"camera_end":camera_at(&o.camera,o.ticks as f64,&o.config)});
        super::io::write_output(
            path,
            &serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
        )?;
    }
    println!("Rendered {frame_count} frames -> {}", o.output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn video_fit_uses_vertices_and_includes_motion_endpoints() {
        let mut pack = ResourcePackSource::from_resource_pack(Default::default());
        pack.add_blockstate_json(
            "minecraft:stone",
            r#"{"variants":{"":{"model":"minecraft:block/test"}}}"#,
        )
        .unwrap();
        pack.add_model_json("minecraft:block/test", r##"{"textures":{"all":"minecraft:block/test"},"elements":[{"from":[0,0,0],"to":[16,16,16],"faces":{"north":{"texture":"#all"},"south":{"texture":"#all"},"east":{"texture":"#all"},"west":{"texture":"#all"},"up":{"texture":"#all"},"down":{"texture":"#all"}}}]}"##).unwrap();
        pack.add_texture("minecraft:block/test", 1, 1, vec![255; 4])
            .unwrap();
        let mut scene = UniversalSchematic::new("bounds".into());
        scene
            .set_block_from_string(-8, 1, 0, "minecraft:stone")
            .unwrap();
        let mut mesh = scene.to_mesh(&pack, &MeshConfig::default()).unwrap();
        fit_mesh_bounds(&mut mesh);
        assert_eq!(mesh.bounds.min, [-8.5, 0.5, -0.5]);
        assert_eq!(mesh.bounds.max, [-7.5, 1.5, 0.5]);
        let base = (mesh.bounds.min, mesh.bounds.max);
        include_translation(&mut mesh, base, [-1.0, 0.0, 0.0]);
        include_translation(&mut mesh, base, [10.0, 2.0, -3.0]);
        assert_eq!(mesh.bounds.min, [-9.5, 0.5, -3.5]);
        assert_eq!(mesh.bounds.max, [2.5, 3.5, 0.5]);
    }

    fn options(extra: &[&str]) -> Options {
        let mut args = vec!["scene.litematic", "-o", "out.mp4", "--pack", "pack.zip"];
        args.extend_from_slice(extra);
        parse(args.into_iter().map(str::to_string)).unwrap()
    }
    fn piston_scene(x: i32) -> UniversalSchematic {
        let mut s = UniversalSchematic::new("piston test".into());
        s.metadata.mc_version = Some(4671);
        for (dx, y, z, state) in [
            (0, 0, 0, "minecraft:stone"),
            (0, 0, 1, "minecraft:stone"),
            (
                0,
                1,
                0,
                "minecraft:sticky_piston[facing=east,extended=false]",
            ),
            (
                0,
                1,
                1,
                "minecraft:lever[face=floor,facing=north,powered=false]",
            ),
            (1, 1, 0, "minecraft:stone"),
        ] {
            s.set_block_from_string(x + dx, y, z, state).unwrap();
        }
        s
    }
    #[test]
    fn lever_drives_full_extension_and_return_in_authored_coordinates() {
        for x in [0, -8] {
            let pos = format!("10:{x},1,1");
            let o = options(&["--ticks", "24", "--use", &pos]);
            let (sim, visuals, offset) = simulate(&piston_scene(x), &o).unwrap();
            let local = Pos::new(x - offset[0], 1 - offset[1], -offset[2]);
            assert!(sim
                .registry()
                .descriptor(sim.world().get(local))
                .unwrap()
                .contains("extended=true"));
            assert_eq!(
                sim.registry()
                    .descriptor(sim.world().get(Pos::new(local.x + 2, local.y, local.z))),
                Some("minecraft:stone")
            );
            assert!(
                visuals.spans.iter().any(|s| s.key.kind == 2),
                "actual shortened heads recorded"
            );
            let release = format!("40:{x},1,1");
            let o = options(&["--ticks", "64", "--use", &pos, "--use", &release]);
            let (sim, visuals, _) = simulate(&piston_scene(x), &o).unwrap();
            assert!(sim.is_quiescent());
            assert!(sim
                .registry()
                .descriptor(sim.world().get(local))
                .unwrap()
                .contains("extended=false"));
            assert_eq!(
                sim.registry()
                    .descriptor(sim.world().get(Pos::new(local.x + 1, local.y, local.z))),
                Some("minecraft:stone")
            );
            assert!(visuals
                .spans
                .iter()
                .any(|s| s.key.kind != 0 && !s.key.extending));
        }
    }
    #[test]
    fn camera_interpolates_angle_zoom_and_pan_and_holds_endpoints() {
        let o = options(&[
            "--camera",
            "0:45,30,1,0,0,0",
            "--camera",
            "80:225,50,2,4,2,-2",
        ]);
        let p = camera_at(&o.camera, 40.0, &o.config);
        assert_eq!(p.yaw, 135.0 - o.config.yaw);
        assert_eq!(p.zoom, 1.5);
        assert_eq!(p.target_offset, [2.0, 1.0, -1.0]);
        assert_eq!(
            camera_at(&o.camera, 90.0, &o.config),
            camera_at(&o.camera, 80.0, &o.config)
        );
    }
    #[test]
    fn same_tick_actions_keep_cli_order_and_invalid_controls_fail() {
        assert!(
            parse(
                ["-", "-o", "-", "--pack", "pack.zip"]
                    .into_iter()
                    .map(str::to_string)
            )
            .is_ok(),
            "stdin and stdout are distinct streams"
        );
        let o = options(&[
            "--place",
            "10:0,0,0=minecraft:stone",
            "--use",
            "10:0,0,0",
            "--break",
            "10:0,0,0",
        ]);
        assert_eq!(o.actions[0].state.as_deref(), Some("minecraft:stone"));
        assert!(o.actions[1].state.is_none());
        assert_eq!(o.actions[2].state.as_deref(), Some("minecraft:air"));
        for extra in [
            vec!["--ticks", "0"],
            vec!["--fps", "NaN"],
            vec!["--camera", "0:45,30,0"],
            vec!["--camera", "0:0,30,1", "--camera", "0:90,30,1"],
            vec!["--use", "100:0,0,0"],
            vec!["--bogus"],
        ] {
            let mut a = vec!["scene.litematic", "-o", "out.mp4", "--pack", "pack.zip"];
            a.extend(extra);
            assert!(parse(a.into_iter().map(str::to_string)).is_err());
        }
    }
}
