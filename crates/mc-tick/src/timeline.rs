//! Opt-in simulation-run recording, cycle detection, and range selection.
//!
//! A [`RunTimeline`] is deliberately richer than the animation mesher's event
//! format. It retains the player inputs that caused a run, canonical state
//! fingerprints, complete state frames for collision-free cycle verification,
//! block deltas, and the piston strokes needed to project a selected range into
//! an external animation format.

use std::collections::HashMap;

use crate::behaviour::BlockChange;
use crate::piston::{TRIGGER_CONTRACT, TRIGGER_DROP, TRIGGER_EXTEND};
use crate::pos::{Dir, Pos};
use crate::schedule::BlockEvent;
use crate::state::{StateId, StateRegistry};
use crate::world::World;

/// A stable 128-bit digest of one visible world state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateFingerprint(pub u128);

impl StateFingerprint {
    /// Lowercase, fixed-width hexadecimal representation.
    pub fn to_hex(self) -> String {
        format!("{:032x}", self.0)
    }
}

impl std::fmt::Display for StateFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

/// The non-air world and its two fingerprints at one tick boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateFrame {
    /// Tick whose boundary this frame represents.
    pub tick: u64,
    /// Absolute-position fingerprint.
    pub exact: StateFingerprint,
    /// Fingerprint after subtracting the non-air bounding-box minimum.
    pub translated: StateFingerprint,
    /// Non-air bounding-box minimum, or `(0,0,0)` for an empty world.
    pub origin: Pos,
    /// Every non-air `(position, state)` in deterministic storage order.
    pub blocks: Vec<(Pos, StateId)>,
}

impl StateFrame {
    pub(crate) fn capture(tick: u64, world: &World, registry: &StateRegistry) -> Self {
        let mut blocks: Vec<(Pos, StateId)> = world.iter_non_air().collect();
        blocks.sort_unstable_by_key(|(pos, _)| *pos);
        let origin = blocks
            .first()
            .map(|(first, _)| {
                blocks.iter().skip(1).fold(*first, |origin, (pos, _)| {
                    Pos::new(
                        origin.x.min(pos.x),
                        origin.y.min(pos.y),
                        origin.z.min(pos.z),
                    )
                })
            })
            .unwrap_or_default();
        let exact = fingerprint(&blocks, Pos::default(), registry);
        let translated = fingerprint(&blocks, origin, registry);
        Self {
            tick,
            exact,
            translated,
            origin,
            blocks,
        }
    }

    fn same_exact(&self, other: &Self) -> bool {
        self.exact == other.exact && self.blocks == other.blocks
    }

    fn same_translated(&self, other: &Self) -> bool {
        self.translated == other.translated
            && self.blocks.len() == other.blocks.len()
            && self
                .blocks
                .iter()
                .zip(&other.blocks)
                .all(|((a_pos, a_state), (b_pos, b_state))| {
                    *a_state == *b_state
                        && a_pos.x - self.origin.x == b_pos.x - other.origin.x
                        && a_pos.y - self.origin.y == b_pos.y - other.origin.y
                        && a_pos.z - self.origin.z == b_pos.z - other.origin.z
                })
    }
}

fn fingerprint(
    blocks: &[(Pos, StateId)],
    origin: Pos,
    registry: &StateRegistry,
) -> StateFingerprint {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(blocks.len() as u64).to_le_bytes());
    for (pos, state) in blocks {
        hasher.update(&(pos.x - origin.x).to_le_bytes());
        hasher.update(&(pos.y - origin.y).to_le_bytes());
        hasher.update(&(pos.z - origin.z).to_le_bytes());
        let descriptor = registry.descriptor(*state).unwrap_or_default().as_bytes();
        hasher.update(&(descriptor.len() as u32).to_le_bytes());
        hasher.update(descriptor);
    }
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
    StateFingerprint(u128::from_le_bytes(bytes))
}

/// One external action applied between ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputAction {
    /// Right-click a block with an empty hand.
    UseBlock {
        /// Boundary tick at which the action ran.
        tick: u64,
        /// Clicked position.
        pos: Pos,
    },
    /// Write a block state, including air for a break.
    PlaceBlock {
        /// Boundary tick at which the action ran.
        tick: u64,
        /// Written position.
        pos: Pos,
        /// Requested state.
        state: StateId,
    },
}

impl InputAction {
    /// Boundary tick at which this action ran.
    pub fn tick(self) -> u64 {
        match self {
            Self::UseBlock { tick, .. } | Self::PlaceBlock { tick, .. } => tick,
        }
    }
}

/// The piston operation the simulator actually dispatched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PistonAction {
    /// Push outward.
    Extend,
    /// Retract, pulling when sticky.
    Retract,
    /// Short-pulse retraction that deliberately drops the moved block.
    Drop,
}

/// One successfully dispatched piston stroke.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PistonEvent {
    /// Tick of the block event.
    pub tick: u64,
    /// Piston base position.
    pub pos: Pos,
    /// Operation dispatched by the engine.
    pub action: PistonAction,
    /// Direction the piston faces.
    pub dir: Dir,
    /// Whether the base is sticky.
    pub sticky: bool,
    /// Index into [`RunTimeline::changes`] immediately before this stroke.
    ///
    /// This preserves the stroke's exact position among same-tick block
    /// changes when projecting to another ordered event format.
    pub change_index: usize,
}

pub(crate) fn piston_event(
    tick: u64,
    event: BlockEvent,
    descriptor: &str,
    change_index: usize,
) -> Option<PistonEvent> {
    let name = descriptor.split('[').next().unwrap_or(descriptor);
    let sticky = match name {
        "minecraft:piston" => false,
        "minecraft:sticky_piston" => true,
        _ => return None,
    };
    let action = match event.id {
        TRIGGER_EXTEND => PistonAction::Extend,
        TRIGGER_CONTRACT => PistonAction::Retract,
        TRIGGER_DROP => PistonAction::Drop,
        _ => return None,
    };
    let dir = match event.param {
        0 => Dir::Down,
        1 => Dir::Up,
        2 => Dir::North,
        3 => Dir::South,
        4 => Dir::West,
        5 => Dir::East,
        _ => return None,
    };
    Some(PistonEvent {
        tick,
        pos: event.pos,
        action,
        dir,
        sticky,
        change_index,
    })
}

/// Whether a repeated visible state is stationary or translated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleKind {
    /// The absolute block configuration repeated.
    Exact,
    /// The bounding-box-normalized configuration repeated at another origin.
    Translated,
}

/// One verified repeated state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cycle {
    /// Match category.
    pub kind: CycleKind,
    /// First matching state boundary.
    pub start_tick: u64,
    /// Second matching state boundary.
    pub end_tick: u64,
    /// `end_tick - start_tick`.
    pub period: u64,
    /// Bounding-box-origin movement across the period.
    pub drift: Pos,
}

/// Exact and translated cycle results, reported independently.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CycleReport {
    /// First verified exact recurrence containing at least one block change.
    pub exact: Option<Cycle>,
    /// First verified translated recurrence with non-zero drift.
    pub translated: Option<Cycle>,
}

/// A half-open tick range and the frame used as its initial scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineSelection {
    /// Included boundary tick.
    pub start_tick: u64,
    /// Excluded boundary tick.
    pub end_tick: u64,
    /// Index of `start_tick` in [`RunTimeline::frames`].
    pub frame_index: usize,
}

impl TimelineSelection {
    /// Whether an event at `tick` belongs to the half-open range.
    pub fn contains(self, tick: u64) -> bool {
        tick >= self.start_tick && tick < self.end_tick
    }
}

/// Invalid timeline-range request.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TimelineError {
    /// A range must advance time.
    #[error("timeline range must satisfy start < end (got {start}..{end})")]
    EmptyRange {
        /// Requested start.
        start: u64,
        /// Requested end.
        end: u64,
    },
    /// A requested boundary was not recorded.
    #[error("tick {0} is outside the recorded frame boundaries")]
    MissingTick(u64),
    /// Toggle-to-toggle selection needs an action and its successor.
    #[error("input action {0} has no following action")]
    MissingNextAction(usize),
    /// No cycle of the requested kind was found.
    #[error("no {0:?} cycle was detected")]
    MissingCycle(CycleKind),
}

/// A complete opt-in recording of one simulation run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunTimeline {
    /// Tick at which recording began.
    pub start_tick: u64,
    /// Ordered block deltas, shared with [`crate::Simulation::recorded`].
    pub changes: Vec<BlockChange>,
    /// Ordered external inputs.
    pub inputs: Vec<InputAction>,
    /// Ordered successfully dispatched piston strokes.
    pub pistons: Vec<PistonEvent>,
    /// Initial frame followed by one frame after each completed tick.
    pub frames: Vec<StateFrame>,
}

impl RunTimeline {
    /// Detect exact and translated cycles independently.
    ///
    /// Fingerprints are only an index. Every candidate is checked against the
    /// complete block/state vectors, and translated matches subtract only one
    /// bounding-box origin: rotations, reflections, and symmetric sub-builds
    /// cannot become false translation matches.
    pub fn detect_cycles(&self) -> CycleReport {
        let mut exact_seen: HashMap<StateFingerprint, Vec<usize>> = HashMap::new();
        let mut translated_seen: HashMap<StateFingerprint, Vec<usize>> = HashMap::new();
        let mut report = CycleReport::default();

        for (index, frame) in self.frames.iter().enumerate() {
            if report.exact.is_none() {
                if let Some(previous) = exact_seen.get(&frame.exact) {
                    for &before in previous {
                        let first = &self.frames[before];
                        if self.changed_between(first.tick, frame.tick) && first.same_exact(frame) {
                            report.exact = Some(cycle(CycleKind::Exact, first, frame));
                            break;
                        }
                    }
                }
            }
            exact_seen.entry(frame.exact).or_default().push(index);

            if report.translated.is_none() {
                if let Some(previous) = translated_seen.get(&frame.translated) {
                    for &before in previous {
                        let first = &self.frames[before];
                        if first.origin != frame.origin
                            && self.changed_between(first.tick, frame.tick)
                            && first.same_translated(frame)
                        {
                            report.translated = Some(cycle(CycleKind::Translated, first, frame));
                            break;
                        }
                    }
                }
            }
            translated_seen
                .entry(frame.translated)
                .or_default()
                .push(index);
        }
        report
    }

    fn changed_between(&self, start: u64, end: u64) -> bool {
        self.changes
            .iter()
            .any(|change| change.tick >= start && change.tick < end)
    }

    /// Select an explicit half-open tick span.
    pub fn select_ticks(
        &self,
        start_tick: u64,
        end_tick: u64,
    ) -> Result<TimelineSelection, TimelineError> {
        if start_tick >= end_tick {
            return Err(TimelineError::EmptyRange {
                start: start_tick,
                end: end_tick,
            });
        }
        let frame_index = self
            .frames
            .binary_search_by_key(&start_tick, |frame| frame.tick)
            .map_err(|_| TimelineError::MissingTick(start_tick))?;
        self.frames
            .binary_search_by_key(&end_tick, |frame| frame.tick)
            .map_err(|_| TimelineError::MissingTick(end_tick))?;
        Ok(TimelineSelection {
            start_tick,
            end_tick,
            frame_index,
        })
    }

    /// Select from input action `index` up to, but not including, its successor.
    pub fn select_between_actions(&self, index: usize) -> Result<TimelineSelection, TimelineError> {
        let Some(first) = self.inputs.get(index) else {
            return Err(TimelineError::MissingNextAction(index));
        };
        let Some(next) = self.inputs.get(index + 1) else {
            return Err(TimelineError::MissingNextAction(index));
        };
        self.select_ticks(first.tick(), next.tick())
    }

    /// Select one detected cycle of `kind`.
    pub fn select_cycle(&self, kind: CycleKind) -> Result<TimelineSelection, TimelineError> {
        let report = self.detect_cycles();
        let found = match kind {
            CycleKind::Exact => report.exact,
            CycleKind::Translated => report.translated,
        }
        .ok_or(TimelineError::MissingCycle(kind))?;
        self.select_ticks(found.start_tick, found.end_tick)
    }

    /// Initial state frame for `selection`.
    pub fn initial_frame(&self, selection: TimelineSelection) -> &StateFrame {
        &self.frames[selection.frame_index]
    }
}

fn cycle(kind: CycleKind, first: &StateFrame, second: &StateFrame) -> Cycle {
    Cycle {
        kind,
        start_tick: first.tick,
        end_tick: second.tick,
        period: second.tick - first.tick,
        drift: Pos::new(
            second.origin.x - first.origin.x,
            second.origin.y - first.origin.y,
            second.origin.z - first.origin.z,
        ),
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TimelineRecorder {
    pub(crate) start_tick: u64,
    pub(crate) inputs: Vec<InputAction>,
    pub(crate) pistons: Vec<PistonEvent>,
    pub(crate) frames: Vec<StateFrame>,
}

impl TimelineRecorder {
    pub(crate) fn new(tick: u64, world: &World, registry: &StateRegistry) -> Self {
        Self {
            start_tick: tick,
            inputs: Vec::new(),
            pistons: Vec::new(),
            frames: vec![StateFrame::capture(tick, world, registry)],
        }
    }

    pub(crate) fn finish(&self, changes: &[BlockChange]) -> RunTimeline {
        RunTimeline {
            start_tick: self.start_tick,
            changes: changes.to_vec(),
            inputs: self.inputs.clone(),
            pistons: self.pistons.clone(),
            frames: self.frames.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bounds, Simulation};

    fn frame(tick: u64, entries: &[(Pos, StateId)], registry: &StateRegistry) -> StateFrame {
        let bounds = Bounds::new(Pos::new(-8, -2, -8), Pos::new(8, 2, 8));
        let mut world = World::new(bounds);
        for (pos, state) in entries {
            world.set(*pos, *state);
        }
        StateFrame::capture(tick, &world, registry)
    }

    #[test]
    fn fingerprints_distinguish_absolute_from_translated_states() {
        let mut registry = StateRegistry::new();
        let stone = registry.intern("minecraft:stone").unwrap();
        let a = frame(
            0,
            &[(Pos::new(0, 0, 0), stone), (Pos::new(1, 0, 0), stone)],
            &registry,
        );
        let b = frame(
            4,
            &[(Pos::new(-3, 0, 0), stone), (Pos::new(-2, 0, 0), stone)],
            &registry,
        );
        assert_ne!(a.exact, b.exact);
        assert_eq!(a.translated, b.translated);
        assert!(a.same_translated(&b));
    }

    #[test]
    fn cycle_candidates_are_verified_and_stationary_ticks_are_ignored() {
        let mut registry = StateRegistry::new();
        let stone = registry.intern("minecraft:stone").unwrap();
        let initial = frame(0, &[(Pos::new(0, 0, 0), stone)], &registry);
        let moved = frame(1, &[(Pos::new(1, 0, 0), stone)], &registry);
        let still = frame(2, &[(Pos::new(1, 0, 0), stone)], &registry);
        let timeline = RunTimeline {
            start_tick: 0,
            changes: vec![BlockChange {
                tick: 0,
                pos: Pos::new(0, 0, 0),
                from: stone,
                to: StateId::AIR,
            }],
            inputs: Vec::new(),
            pistons: Vec::new(),
            frames: vec![initial, moved, still],
        };
        let report = timeline.detect_cycles();
        let translated = report.translated.expect("translated recurrence");
        assert_eq!(translated.period, 1);
        assert_eq!(translated.drift, Pos::new(1, 0, 0));
        assert!(
            report.exact.is_none(),
            "unchanged tick must not be called a cycle"
        );
    }

    #[test]
    fn timeline_recording_does_not_change_simulation_results() {
        fn run(record: bool) -> (crate::Checkpoint, Vec<BlockChange>, Option<RunTimeline>) {
            let mut sim = Simulation::new(Bounds::new(Pos::new(-2, -2, -2), Pos::new(2, 2, 2)));
            let stone = sim.registry_mut().intern("minecraft:stone").unwrap();
            if record {
                sim.record_timeline();
            } else {
                sim.record();
            }
            sim.place_block(Pos::new(0, 0, 0), stone);
            sim.step();
            (
                sim.checkpoint(),
                sim.recorded().to_vec(),
                sim.recorded_timeline(),
            )
        }
        let (plain, plain_changes, _) = run(false);
        let (recorded, recorded_changes, timeline) = run(true);
        assert_eq!(plain, recorded);
        assert_eq!(plain_changes, recorded_changes);
        let timeline = timeline.expect("timeline");
        assert_eq!(timeline.inputs.len(), 1);
        assert_eq!(timeline.changes.len(), 1);
        assert_eq!(timeline.frames.len(), 2);
    }
}
