//! `nucleation animate` — drive an SNBT structure through `mc-tick`, select a
//! range, and pass that run to the existing animated-GLB mesher.

use std::path::{Path, PathBuf};

use mc_test::mc_tick::{
    vanilla::Descriptor, Cycle, CycleKind, Dir, PistonAction, Pos, RunTimeline, Simulation,
    StateRegistry, Structure, TimelineSelection,
};
use mc_test::SettleMode;
use serde_json::{json, Map, Value};

use crate::usage_and_exit;

#[derive(Debug, Clone)]
enum ActionKind {
    Use(Pos),
    Place(Pos, String),
}

#[derive(Debug, Clone)]
struct ScheduledAction {
    tick: u64,
    order: usize,
    kind: ActionKind,
}

#[derive(Debug, Clone, Copy)]
enum RangeRequest {
    All,
    Ticks(u64, u64),
    BetweenActions(usize),
    Cycle(CycleKind),
}

#[derive(Debug, Default, PartialEq, Eq)]
struct ProjectionWarnings {
    dropped_non_sticky_retractions: usize,
    dropped_short_pulse_retractions: usize,
    projected_pistons: usize,
}

pub(crate) fn animate_main(args: impl Iterator<Item = String>) {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut pack: Option<PathBuf> = None;
    let mut timeline_out: Option<PathBuf> = None;
    let mut container: Option<String> = None;
    let mut ticks = 100u64;
    let mut tick_ms = 50.0f32;
    let mut settle = SettleMode::Placement;
    let mut range = RangeRequest::All;
    let mut actions = Vec::new();
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--out" => {
                output = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--pack" => {
                pack = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--to" => container = Some(args.next().unwrap_or_else(|| usage_and_exit())),
            "--ticks" => {
                ticks = parse_num("ticks", &args.next().unwrap_or_else(|| usage_and_exit()))
            }
            "--tick-ms" => {
                tick_ms = parse_num("tick-ms", &args.next().unwrap_or_else(|| usage_and_exit()))
            }
            "--settle" => settle = parse_settle(&args.next().unwrap_or_else(|| usage_and_exit())),
            "--range" => {
                let value = args.next().unwrap_or_else(|| usage_and_exit());
                let (start, end) = parse_span(&value).unwrap_or_else(|error| fail(error));
                range = RangeRequest::Ticks(start, end);
            }
            "--between-actions" => {
                let index = parse_num(
                    "between-actions",
                    &args.next().unwrap_or_else(|| usage_and_exit()),
                );
                range = RangeRequest::BetweenActions(index);
            }
            "--cycle" => {
                range = RangeRequest::Cycle(parse_cycle(
                    &args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            "--use" => {
                let value = args.next().unwrap_or_else(|| usage_and_exit());
                let (tick, pos) = parse_timed_pos(&value).unwrap_or_else(|error| fail(error));
                actions.push(ScheduledAction {
                    tick,
                    order: actions.len(),
                    kind: ActionKind::Use(pos),
                });
            }
            "--place" => {
                let value = args.next().unwrap_or_else(|| usage_and_exit());
                let (tick, pos, state) = parse_place(&value).unwrap_or_else(|error| fail(error));
                actions.push(ScheduledAction {
                    tick,
                    order: actions.len(),
                    kind: ActionKind::Place(pos, state),
                });
            }
            "--break" => {
                let value = args.next().unwrap_or_else(|| usage_and_exit());
                let (tick, pos) = parse_timed_pos(&value).unwrap_or_else(|error| fail(error));
                actions.push(ScheduledAction {
                    tick,
                    order: actions.len(),
                    kind: ActionKind::Place(pos, "minecraft:air".to_string()),
                });
            }
            "--timeline-json" => {
                timeline_out = Some(PathBuf::from(
                    args.next().unwrap_or_else(|| usage_and_exit()),
                ))
            }
            other if other.starts_with('-') && other != "-" => usage_and_exit(),
            other => input = Some(PathBuf::from(other)),
        }
    }

    let Some(input) = input else { usage_and_exit() };
    if ticks == 0 {
        fail("ticks must be greater than zero".to_string());
    }
    if !tick_ms.is_finite() || tick_ms <= 0.0 {
        fail("tick-ms must be a finite positive number".to_string());
    }
    let output = output.unwrap_or_else(|| {
        if super::io::is_stdio(&input) {
            PathBuf::from("-")
        } else {
            input.with_extension("animated.glb")
        }
    });
    let container = container
        .or_else(|| {
            output
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
        })
        .unwrap_or_else(|| "glb".to_string());
    if !matches!(container.as_str(), "glb" | "usdz") {
        fail(format!(
            "animate writes glb or a static usdz snapshot, not {container:?}"
        ));
    }
    let Some(pack) = pack.or_else(super::pack::discover_pack) else {
        fail(
            "no resource pack found — pass --pack <zip or client jar>, set NUCLEATION_PACK, \
             or install Minecraft"
                .to_string(),
        );
    };
    if timeline_out
        .as_ref()
        .is_some_and(|p| super::io::is_stdio(p))
        && super::io::is_stdio(&output)
    {
        fail("GLB and timeline JSON cannot both own stdout".to_string());
    }

    let piping_out = super::io::is_stdio(&output)
        || timeline_out
            .as_ref()
            .is_some_and(|path| super::io::is_stdio(path));
    let result = run_export(RunConfig {
        input: &input,
        output: &output,
        pack: &pack,
        timeline_out: timeline_out.as_deref(),
        container: &container,
        ticks,
        tick_ms,
        settle,
        range,
        actions,
    });
    match result {
        Ok(summary) => {
            super::io::status(&summary.status, piping_out);
            for warning in summary.warnings {
                eprintln!("warning: {warning}");
            }
        }
        Err(error) => fail(error),
    }
}

struct RunConfig<'a> {
    input: &'a Path,
    output: &'a Path,
    pack: &'a Path,
    timeline_out: Option<&'a Path>,
    container: &'a str,
    ticks: u64,
    tick_ms: f32,
    settle: SettleMode,
    range: RangeRequest,
    actions: Vec<ScheduledAction>,
}

struct ExportSummary {
    status: String,
    warnings: Vec<String>,
}

fn run_export(mut config: RunConfig<'_>) -> Result<ExportSummary, String> {
    config
        .actions
        .sort_by_key(|action| (action.tick, action.order));
    if let Some(action) = config
        .actions
        .iter()
        .find(|action| action.tick > config.ticks)
    {
        return Err(format!(
            "action at tick {} is beyond the recorded horizon {}",
            action.tick, config.ticks
        ));
    }

    let bytes = super::io::read_input(config.input)?;
    let text = std::str::from_utf8(&bytes).map_err(|e| {
        format!(
            "{}: mc-tick animation input must be an SNBT structure: {e}",
            super::io::display_name(config.input)
        )
    })?;
    let structure = Structure::parse(text).map_err(|e| {
        format!(
            "{}: invalid SNBT structure: {e}",
            super::io::display_name(config.input)
        )
    })?;
    let extra_states: Vec<String> = config
        .actions
        .iter()
        .filter_map(|action| match &action.kind {
            ActionKind::Place(_, state) => Some(state.clone()),
            ActionKind::Use(_) => None,
        })
        .collect();
    let mut sim = mc_test::try_build_sim(
        &structure,
        Pos::default(),
        config.settle,
        &extra_states,
        &[],
        None,
        &super::io::display_name(config.input),
    )?;

    sim.record_timeline();
    let mut next_action = 0;
    for boundary in 0..=config.ticks {
        while config
            .actions
            .get(next_action)
            .is_some_and(|action| action.tick == boundary)
        {
            apply_action(&mut sim, &config.actions[next_action])?;
            next_action += 1;
        }
        if boundary < config.ticks {
            sim.step();
        }
    }
    let timeline = sim
        .recorded_timeline()
        .ok_or_else(|| "timeline recorder unexpectedly stopped".to_string())?;
    let cycles = timeline.detect_cycles();
    let selection = select_range(&timeline, config.range)?;
    let initial = selected_schematic(&timeline, selection, sim.registry())?;
    let (timeline_json, projection) =
        project_timeline(&timeline, selection, sim.registry(), config.tick_ms)?;

    if let Some(path) = config.timeline_out {
        let pretty: Value = serde_json::from_str(&timeline_json)
            .map_err(|e| format!("internal timeline JSON did not round-trip: {e}"))?;
        let encoded = serde_json::to_vec_pretty(&pretty)
            .map_err(|e| format!("encoding timeline JSON: {e}"))?;
        super::io::write_output(path, &encoded)?;
    }

    let source =
        nucleation::meshing::ResourcePackSource::from_file(&config.pack.display().to_string())
            .map_err(|e| format!("loading resource pack {}: {e:?}", config.pack.display()))?;
    let (encoded, static_usdz) = match config.container {
        "usdz" => {
            let mesh = initial
                .to_mesh(&source, &nucleation::meshing::MeshConfig::default())
                .map_err(|e| format!("meshing static USDZ snapshot: {e:?}"))?;
            (
                mesh.to_usdz()
                    .map_err(|e| format!("encoding usdz: {e:?}"))?,
                true,
            )
        }
        _ => (
            initial
                .to_animated_glb(&source, &timeline_json)
                .map_err(|e| format!("animated GLB: {e:?}"))?,
            false,
        ),
    };
    let size = encoded.len();
    super::io::write_output(config.output, &encoded)?;

    let mut warnings = Vec::new();
    if projection.dropped_non_sticky_retractions > 0 {
        warnings.push(format!(
            "{} non-sticky piston retractions were left as STEP block changes because the \
             mesher's retract event always pulls",
            projection.dropped_non_sticky_retractions
        ));
    }
    if projection.dropped_short_pulse_retractions > 0 {
        warnings.push(format!(
            "{} piston drop retractions were left as STEP block changes because the mesher \
             timeline has no drop action",
            projection.dropped_short_pulse_retractions
        ));
    }
    if projection.projected_pistons > 0 {
        warnings.push(
            "the mesher's piston event reconstructs only a straight push column; slime/honey side \
             branches remain correct STEP state changes but are not translated motion nodes"
                .to_string(),
        );
    }
    if static_usdz {
        warnings.push(
            "the existing USDZ encoder is static; this file is the selected range's initial \
             snapshot, not an animation"
                .to_string(),
        );
    }

    let cycle_text = format!(
        "cycles: exact {}; translated {}",
        format_cycle(cycles.exact),
        format_cycle(cycles.translated)
    );
    let operation = if static_usdz {
        "exported static snapshot of"
    } else {
        "animated"
    };
    Ok(ExportSummary {
        status: format!(
            "{operation} {} ticks {}..{} -> {} ({}, {} bytes); {cycle_text}",
            super::io::display_name(config.input),
            selection.start_tick,
            selection.end_tick,
            super::io::display_out(config.output),
            config.container,
            size
        ),
        warnings,
    })
}

fn apply_action(sim: &mut Simulation, action: &ScheduledAction) -> Result<(), String> {
    match &action.kind {
        ActionKind::Use(pos) => sim.use_block(*pos),
        ActionKind::Place(pos, descriptor) => {
            let state = sim
                .registry_mut()
                .intern(descriptor)
                .map_err(|e| format!("tick {}: interning {descriptor}: {e}", action.tick))?;
            sim.place_block(*pos, state);
        }
    }
    Ok(())
}

fn select_range(
    timeline: &RunTimeline,
    request: RangeRequest,
) -> Result<TimelineSelection, String> {
    match request {
        RangeRequest::All => {
            let end = timeline
                .frames
                .last()
                .ok_or_else(|| "recording has no frames".to_string())?
                .tick;
            timeline.select_ticks(timeline.start_tick, end)
        }
        RangeRequest::Ticks(start, end) => timeline.select_ticks(start, end),
        RangeRequest::BetweenActions(index) => timeline.select_between_actions(index),
        RangeRequest::Cycle(kind) => timeline.select_cycle(kind),
    }
    .map_err(|e| e.to_string())
}

fn selected_schematic(
    timeline: &RunTimeline,
    selection: TimelineSelection,
    registry: &StateRegistry,
) -> Result<nucleation::UniversalSchematic, String> {
    let frame = timeline.initial_frame(selection);
    let mut schematic = nucleation::UniversalSchematic::new(format!(
        "mc-tick {}..{}",
        selection.start_tick, selection.end_tick
    ));
    for (pos, state) in &frame.blocks {
        let descriptor = registry
            .descriptor(*state)
            .ok_or_else(|| format!("state {} has no descriptor", state.raw()))?;
        schematic.set_block_from_string(
            pos.x - frame.origin.x,
            pos.y - frame.origin.y,
            pos.z - frame.origin.z,
            descriptor,
        )?;
    }
    Ok(schematic)
}

fn project_timeline(
    timeline: &RunTimeline,
    selection: TimelineSelection,
    registry: &StateRegistry,
    tick_ms: f32,
) -> Result<(String, ProjectionWarnings), String> {
    let origin = timeline.initial_frame(selection).origin;
    let mut events = Vec::new();
    let mut warnings = ProjectionWarnings::default();
    let mut piston_index = 0;

    for change_index in 0..=timeline.changes.len() {
        while let Some(piston) = timeline.pistons.get(piston_index) {
            if piston.change_index != change_index {
                break;
            }
            if selection.contains(piston.tick) {
                match piston.action {
                    PistonAction::Extend => {
                        events.push(json!({
                            "kind": "piston",
                            "tick": relative_tick(piston.tick, selection.start_tick)?,
                            "pos": [piston.pos.x, piston.pos.y, piston.pos.z],
                            "action": "extend",
                            "dir": dir_name(piston.dir),
                        }));
                        warnings.projected_pistons += 1;
                    }
                    PistonAction::Retract if piston.sticky => {
                        events.push(json!({
                            "kind": "piston",
                            "tick": relative_tick(piston.tick, selection.start_tick)?,
                            "pos": [piston.pos.x, piston.pos.y, piston.pos.z],
                            "action": "retract",
                            "dir": dir_name(piston.dir),
                        }));
                        warnings.projected_pistons += 1;
                    }
                    PistonAction::Retract => warnings.dropped_non_sticky_retractions += 1,
                    PistonAction::Drop => warnings.dropped_short_pulse_retractions += 1,
                }
            }
            piston_index += 1;
        }
        let Some(change) = timeline.changes.get(change_index) else {
            continue;
        };
        if !selection.contains(change.tick) {
            continue;
        }
        let descriptor = registry
            .descriptor(change.to)
            .ok_or_else(|| format!("state {} has no descriptor", change.to.raw()))?;
        let parsed = Descriptor::parse(descriptor);
        let props: Map<String, Value> = parsed
            .properties
            .into_iter()
            .map(|(key, value)| (key, Value::String(value)))
            .collect();
        events.push(json!({
            "kind": "set_block",
            "tick": relative_tick(change.tick, selection.start_tick)?,
            "pos": [change.pos.x, change.pos.y, change.pos.z],
            "block": parsed.name,
            "props": props,
        }));
    }

    serde_json::to_string(&json!({
        "origin": [origin.x, origin.y, origin.z],
        "tick_ms": tick_ms,
        "events": events,
    }))
    .map(|json| (json, warnings))
    .map_err(|e| format!("encoding mesher timeline: {e}"))
}

fn relative_tick(tick: u64, start: u64) -> Result<u32, String> {
    u32::try_from(tick - start)
        .map_err(|_| format!("selected range is too long for the mesher's u32 ticks"))
}

fn format_cycle(cycle: Option<Cycle>) -> String {
    match cycle {
        None => "none".to_string(),
        Some(cycle) => format!(
            "{}..{} period {} drift ({},{},{})",
            cycle.start_tick,
            cycle.end_tick,
            cycle.period,
            cycle.drift.x,
            cycle.drift.y,
            cycle.drift.z
        ),
    }
}

fn dir_name(dir: Dir) -> &'static str {
    dir.name()
}

fn parse_settle(value: &str) -> SettleMode {
    match value {
        "placement" => SettleMode::Placement,
        "quiet" => SettleMode::Quiet,
        "in-world" => SettleMode::InWorld,
        _ => fail(format!(
            "settle must be placement, quiet, or in-world, not {value:?}"
        )),
    }
}

fn parse_cycle(value: &str) -> CycleKind {
    match value {
        "exact" => CycleKind::Exact,
        "translated" => CycleKind::Translated,
        _ => fail(format!("cycle must be exact or translated, not {value:?}")),
    }
}

fn parse_span(value: &str) -> Result<(u64, u64), String> {
    let Some((start, end)) = value.split_once("..") else {
        return Err(format!("range must be START..END, not {value:?}"));
    };
    Ok((
        parse_num_result("range start", start)?,
        parse_num_result("range end", end)?,
    ))
}

fn parse_timed_pos(value: &str) -> Result<(u64, Pos), String> {
    let Some((tick, pos)) = value.split_once(':') else {
        return Err(format!("action must be TICK:X,Y,Z, not {value:?}"));
    };
    Ok((parse_num_result("action tick", tick)?, parse_pos(pos)?))
}

fn parse_place(value: &str) -> Result<(u64, Pos, String), String> {
    let Some((timed_pos, state)) = value.split_once('=') else {
        return Err(format!(
            "place must be TICK:X,Y,Z=BLOCK_STATE, not {value:?}"
        ));
    };
    if state.is_empty() {
        return Err("place block state cannot be empty".to_string());
    }
    let (tick, pos) = parse_timed_pos(timed_pos)?;
    Ok((tick, pos, state.to_string()))
}

fn parse_pos(value: &str) -> Result<Pos, String> {
    let values: Vec<&str> = value.split(',').collect();
    if values.len() != 3 {
        return Err(format!("position must be X,Y,Z, not {value:?}"));
    }
    Ok(Pos::new(
        parse_num_result("x", values[0])?,
        parse_num_result("y", values[1])?,
        parse_num_result("z", values[2])?,
    ))
}

fn parse_num<T>(what: &str, value: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    parse_num_result(what, value).unwrap_or_else(|error| fail(error))
}

fn parse_num_result<T>(what: &str, value: &str) -> Result<T, String>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    value
        .parse()
        .map_err(|e| format!("invalid {what} {value:?}: {e}"))
}

fn fail(message: String) -> ! {
    eprintln!("{message}");
    std::process::exit(2);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_test::mc_tick::{Bounds, InputAction, StateId};

    #[test]
    fn action_and_range_syntax_is_unambiguous() {
        assert_eq!(
            parse_timed_pos("4:2,1,-3").unwrap(),
            (4, Pos::new(2, 1, -3))
        );
        assert_eq!(
            parse_place("2:2,1,1=minecraft:repeater[delay=2,facing=east]").unwrap(),
            (
                2,
                Pos::new(2, 1, 1),
                "minecraft:repeater[delay=2,facing=east]".to_string()
            )
        );
        assert_eq!(parse_span("12..40").unwrap(), (12, 40));
    }

    #[test]
    fn projection_preserves_same_tick_piston_before_block_delta_order() {
        let mut sim = Simulation::new(Bounds::new(Pos::new(-3, -1, -1), Pos::new(3, 1, 1)));
        let piston = sim
            .registry_mut()
            .intern("minecraft:sticky_piston[facing=east,extended=false]")
            .unwrap();
        let stone = sim.registry_mut().intern("minecraft:stone").unwrap();
        sim.world_mut().set(Pos::new(0, 0, 0), piston);
        sim.record_timeline();
        sim.place_block(Pos::new(1, 0, 0), stone);
        sim.step();
        let mut timeline = sim.recorded_timeline().unwrap();
        timeline.pistons.push(mc_test::mc_tick::PistonEvent {
            tick: 0,
            pos: Pos::new(0, 0, 0),
            action: PistonAction::Extend,
            dir: Dir::East,
            sticky: true,
            change_index: 0,
        });
        let selection = timeline.select_ticks(0, 1).unwrap();
        let (encoded, warnings) =
            project_timeline(&timeline, selection, sim.registry(), 50.0).unwrap();
        let decoded: Value = serde_json::from_str(&encoded).unwrap();
        let events = decoded["events"].as_array().unwrap();
        assert_eq!(events[0]["kind"], "piston");
        assert_eq!(events[1]["kind"], "set_block");
        assert_eq!(
            warnings,
            ProjectionWarnings {
                projected_pistons: 1,
                ..ProjectionWarnings::default()
            }
        );
        assert_eq!(
            timeline.inputs[0],
            InputAction::PlaceBlock {
                tick: 0,
                pos: Pos::new(1, 0, 0),
                state: stone,
            }
        );
        assert_ne!(stone, StateId::AIR);
    }
}
