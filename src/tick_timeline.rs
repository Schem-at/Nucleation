//! The projection from a recorded `mc-tick` run to the animated-GLB mesher's
//! `Timeline` JSON, and to the schematic a selection starts from.
//!
//! This is the single place that answers "what did this run do" for both the
//! `nucleation-cli animate` command and the Diplomat bridge. Two
//! implementations of that question is exactly the divergence this module
//! exists to close — do not fork this logic back into a caller.

use mc_tick::{vanilla::Descriptor, PistonAction, RunTimeline, StateRegistry, TimelineSelection};
use serde_json::{json, Map, Value};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ProjectionWarnings {
    pub dropped_non_sticky_retractions: usize,
    pub dropped_short_pulse_retractions: usize,
    pub projected_pistons: usize,
}

pub fn selection_schematic(
    timeline: &RunTimeline,
    selection: TimelineSelection,
    registry: &StateRegistry,
) -> Result<crate::UniversalSchematic, String> {
    let frame = timeline.initial_frame(selection, registry);
    let mut schematic = crate::UniversalSchematic::new(format!(
        "mc-tick {}..{}",
        selection.start_tick(), selection.end_tick()
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

pub fn mesher_timeline_json(
    timeline: &RunTimeline,
    selection: TimelineSelection,
    registry: &StateRegistry,
    tick_ms: f32,
) -> Result<(String, ProjectionWarnings), String> {
    let origin = timeline.initial_frame(selection, registry).origin;
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
                            "tick": relative_tick(piston.tick, selection.start_tick())?,
                            "pos": [piston.pos.x, piston.pos.y, piston.pos.z],
                            "action": "extend",
                            "dir": piston.dir.name(),
                        }));
                        warnings.projected_pistons += 1;
                    }
                    PistonAction::Retract if piston.sticky => {
                        events.push(json!({
                            "kind": "piston",
                            "tick": relative_tick(piston.tick, selection.start_tick())?,
                            "pos": [piston.pos.x, piston.pos.y, piston.pos.z],
                            "action": "retract",
                            "dir": piston.dir.name(),
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
            "tick": relative_tick(change.tick, selection.start_tick())?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use mc_tick::{Bounds, Dir, InputAction, PistonEvent, Pos, Simulation, StateId};

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
        timeline.pistons.push(PistonEvent {
            tick: 0,
            pos: Pos::new(0, 0, 0),
            action: PistonAction::Extend,
            dir: Dir::East,
            sticky: true,
            change_index: 0,
        });
        let selection = timeline.select_ticks(0, 1).unwrap();
        let (encoded, warnings) =
            mesher_timeline_json(&timeline, selection, sim.registry(), 50.0).unwrap();
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
