//! Opt-in simulation-run recording, cycle detection, and range selection.
//!
//! A [`RunTimeline`] is deliberately richer than the animation mesher's event
//! format. It retains the player inputs that caused a run, the one whole-world
//! [`StateFrame`] recording started from, canonical state fingerprints, block
//! deltas, and the piston strokes needed to project a selected range into an
//! external animation format. Every other frame is rebuilt on demand from the
//! seed and the change log rather than stored.

use std::collections::{BTreeMap, HashMap};

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
    /// Capture the visible world at `tick`.
    pub fn of(tick: u64, world: &World, registry: &StateRegistry) -> Self {
        Self::from_blocks(tick, world.iter_non_air().collect(), registry)
    }

    /// Build a frame from a block list in any order.
    ///
    /// The canonical sort lives here rather than in the caller so that a frame
    /// rebuilt by replay is byte-identical to one captured from a `World` —
    /// two orderings would make every cycle comparison wrong.
    pub fn from_blocks(
        tick: u64,
        mut blocks: Vec<(Pos, StateId)>,
        registry: &StateRegistry,
    ) -> Self {
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

/// A frame without its blocks — enough to *find* a recurrence, not to confirm
/// one.
///
/// Cycle detection indexes on fingerprints and then verifies candidates
/// against full block vectors. Keeping only the index for the scan is what
/// lets a whole run be examined without ever holding more than one world.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameDigest {
    /// Tick whose boundary this digest represents.
    pub tick: u64,
    /// Absolute-position fingerprint.
    pub exact: StateFingerprint,
    /// Fingerprint after subtracting the non-air bounding-box minimum.
    pub translated: StateFingerprint,
    /// Non-air bounding-box minimum.
    pub origin: Pos,
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

/// A half-open tick range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimelineSelection {
    /// Included boundary tick.
    start_tick: u64,
    /// Excluded boundary tick.
    end_tick: u64,
}

impl TimelineSelection {
    /// First tick of the selection.
    pub fn start_tick(&self) -> u64 {
        self.start_tick
    }

    /// Last tick of the selection.
    pub fn end_tick(&self) -> u64 {
        self.end_tick
    }

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
    /// Tick at which recording stopped — the last completed tick.
    ///
    /// Held because replay needs to know where to stop, and the per-tick frame
    /// vector that used to answer that is what replay exists to remove.
    pub end_tick: u64,
    /// Ordered block deltas, shared with [`crate::Simulation::recorded`].
    pub changes: Vec<BlockChange>,
    /// Ordered external inputs.
    pub inputs: Vec<InputAction>,
    /// Ordered successfully dispatched piston strokes.
    pub pistons: Vec<PistonEvent>,
    /// The world as recording began.
    pub initial: StateFrame,
}

impl RunTimeline {
    /// Detect exact and translated cycles independently.
    ///
    /// Fingerprints are only an index. Every candidate is checked against the
    /// complete block/state vectors, and translated matches subtract only one
    /// bounding-box origin: rotations, reflections, and symmetric sub-builds
    /// cannot become false translation matches.
    ///
    /// The scan runs over one replay pass of digests; only the handful of
    /// candidates that survive the tick-only guards are ever rebuilt into full
    /// frames. A world that merely sits still therefore materialises nothing.
    pub fn detect_cycles(&self, registry: &StateRegistry) -> CycleReport {
        let digests = self.digests(registry);
        let mut exact_seen: HashMap<StateFingerprint, Vec<usize>> = HashMap::new();
        let mut translated_seen: HashMap<StateFingerprint, Vec<usize>> = HashMap::new();
        let mut report = CycleReport::default();

        for (index, digest) in digests.iter().enumerate() {
            if report.exact.is_none() {
                if let Some(previous) = exact_seen.get(&digest.exact) {
                    let candidates: Vec<u64> = previous
                        .iter()
                        .map(|&before| digests[before].tick)
                        .filter(|&tick| self.changed_between(tick, digest.tick))
                        .collect();
                    if let Some(found) =
                        self.verify(CycleKind::Exact, &candidates, digest.tick, registry)
                    {
                        report.exact = Some(found);
                    }
                }
            }
            exact_seen.entry(digest.exact).or_default().push(index);

            if report.translated.is_none() {
                if let Some(previous) = translated_seen.get(&digest.translated) {
                    let candidates: Vec<u64> = previous
                        .iter()
                        .map(|&before| &digests[before])
                        .filter(|first| {
                            first.origin != digest.origin
                                && self.changed_between(first.tick, digest.tick)
                        })
                        .map(|first| first.tick)
                        .collect();
                    if let Some(found) =
                        self.verify(CycleKind::Translated, &candidates, digest.tick, registry)
                    {
                        report.translated = Some(found);
                    }
                }
            }
            translated_seen
                .entry(digest.translated)
                .or_default()
                .push(index);
        }
        report
    }

    /// Rebuild `second` and each candidate `first` and return the earliest
    /// candidate that really is a recurrence of `kind`.
    ///
    /// Candidates arrive in ascending tick order and the first that verifies
    /// wins, which is what makes the earlier frame the cycle's start.
    fn verify(
        &self,
        kind: CycleKind,
        candidates: &[u64],
        tick: u64,
        registry: &StateRegistry,
    ) -> Option<Cycle> {
        if candidates.is_empty() {
            return None;
        }
        let second = self.frame_at(tick, registry)?;
        for &candidate in candidates {
            let Some(first) = self.frame_at(candidate, registry) else {
                continue;
            };
            let matched = match kind {
                CycleKind::Exact => first.same_exact(&second),
                CycleKind::Translated => first.same_translated(&second),
            };
            if matched {
                return Some(cycle(kind, &first, &second));
            }
        }
        None
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
        for tick in [start_tick, end_tick] {
            if tick < self.start_tick || tick > self.end_tick {
                return Err(TimelineError::MissingTick(tick));
            }
        }
        Ok(TimelineSelection {
            start_tick,
            end_tick,
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
    pub fn select_cycle(
        &self,
        kind: CycleKind,
        registry: &StateRegistry,
    ) -> Result<TimelineSelection, TimelineError> {
        let report = self.detect_cycles(registry);
        let found = match kind {
            CycleKind::Exact => report.exact,
            CycleKind::Translated => report.translated,
        }
        .ok_or(TimelineError::MissingCycle(kind))?;
        self.select_ticks(found.start_tick, found.end_tick)
    }

    /// Initial state frame for `selection`.
    ///
    /// Infallible by construction: `TimelineSelection` can only be built by
    /// the `select_*` methods, which reject any tick outside the recorded
    /// span. That is why the fields are private — an out-of-range selection
    /// used to be reachable, and the choice was between a wrong scene and a
    /// panic, in a bridge where a panic aborts the host page.
    pub fn initial_frame(
        &self,
        selection: TimelineSelection,
        registry: &StateRegistry,
    ) -> StateFrame {
        self.frame_at(selection.start_tick, registry)
            .expect("a selection is validated against the recorded span when it is built")
    }

    /// The visible world at `tick`, rebuilt from the initial frame and the
    /// change log.
    ///
    /// `None` outside `start_tick..=end_tick`. Every change carries `from` as
    /// well as `to`, so the log is a complete description of the run — the
    /// recorder does not need to keep a snapshot per tick to be able to answer
    /// this, it only needs the one it started from.
    pub fn frame_at(&self, tick: u64, registry: &StateRegistry) -> Option<StateFrame> {
        if tick < self.start_tick || tick > self.end_tick {
            return None;
        }
        let mut blocks = self.seed();
        self.apply_through(&mut blocks, tick);
        Some(StateFrame::from_blocks(
            tick,
            blocks.into_iter().collect(),
            registry,
        ))
    }

    /// One digest per tick boundary, in order, in a single replay pass.
    pub fn digests(&self, registry: &StateRegistry) -> Vec<FrameDigest> {
        let mut blocks = self.seed();
        let mut out = Vec::with_capacity((self.end_tick - self.start_tick + 1) as usize);
        let mut next = 0usize;
        for tick in self.start_tick..=self.end_tick {
            while let Some(change) = self.changes.get(next) {
                if change.tick >= tick {
                    break;
                }
                apply(&mut blocks, change);
                next += 1;
            }
            let frame = StateFrame::from_blocks(
                tick,
                blocks.iter().map(|(pos, state)| (*pos, *state)).collect(),
                registry,
            );
            out.push(FrameDigest {
                tick: frame.tick,
                exact: frame.exact,
                translated: frame.translated,
                origin: frame.origin,
            });
        }
        out
    }

    /// The world as recording began.
    fn seed(&self) -> BTreeMap<Pos, StateId> {
        self.initial
            .blocks
            .iter()
            .map(|(pos, state)| (*pos, *state))
            .collect()
    }

    /// Apply every change that ran *before* `tick`.
    ///
    /// A frame labelled `t` is the boundary after tick `t - 1` completed, so a
    /// change recorded during tick `t` belongs to the frame after it.
    fn apply_through(&self, blocks: &mut BTreeMap<Pos, StateId>, tick: u64) {
        for change in &self.changes {
            if change.tick >= tick {
                break;
            }
            apply(blocks, change);
        }
    }
}

fn apply(blocks: &mut BTreeMap<Pos, StateId>, change: &BlockChange) {
    if change.to == StateId::AIR {
        blocks.remove(&change.pos);
    } else {
        blocks.insert(change.pos, change.to);
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
    pub(crate) initial: StateFrame,
    pub(crate) inputs: Vec<InputAction>,
    pub(crate) pistons: Vec<PistonEvent>,
}

impl TimelineRecorder {
    pub(crate) fn new(tick: u64, world: &World, registry: &StateRegistry) -> Self {
        Self {
            start_tick: tick,
            initial: StateFrame::of(tick, world, registry),
            inputs: Vec::new(),
            pistons: Vec::new(),
        }
    }

    pub(crate) fn finish(&self, changes: &[BlockChange], end_tick: u64) -> RunTimeline {
        RunTimeline {
            start_tick: self.start_tick,
            end_tick,
            changes: changes.to_vec(),
            inputs: self.inputs.clone(),
            pistons: self.pistons.clone(),
            initial: self.initial.clone(),
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
        StateFrame::of(tick, &world, registry)
    }

    #[test]
    fn a_frame_built_from_blocks_matches_one_captured_from_a_world() {
        let mut registry = StateRegistry::new();
        let stone = registry.intern("minecraft:stone").unwrap();
        let bounds = Bounds::new(Pos::new(-8, -2, -8), Pos::new(8, 2, 8));
        let mut world = World::new(bounds);
        world.set(Pos::new(1, 0, 0), stone);
        world.set(Pos::new(-3, 1, 2), stone);

        let captured = StateFrame::of(7, &world, &registry);
        // Deliberately unsorted and in a different order from storage order: the
        // constructor is responsible for canonicalising, or a replayed frame can
        // never equal a recorded one.
        let built = StateFrame::from_blocks(
            7,
            vec![(Pos::new(-3, 1, 2), stone), (Pos::new(1, 0, 0), stone)],
            &registry,
        );
        assert_eq!(captured, built);
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
        let timeline = RunTimeline {
            start_tick: 0,
            end_tick: 2,
            // The block moves one step east during tick 0 and then sits still.
            // Detection replays this log, so the log — not just the frame
            // vector — has to describe that motion.
            changes: vec![
                BlockChange {
                    tick: 0,
                    pos: Pos::new(0, 0, 0),
                    from: stone,
                    to: StateId::AIR,
                },
                BlockChange {
                    tick: 0,
                    pos: Pos::new(1, 0, 0),
                    from: StateId::AIR,
                    to: stone,
                },
            ],
            inputs: Vec::new(),
            pistons: Vec::new(),
            initial,
        };
        let report = timeline.detect_cycles(&registry);
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
    }

    #[test]
    fn a_timeline_knows_the_tick_its_run_ended_on() {
        let mut sim = Simulation::new(Bounds::new(Pos::new(-2, -2, -2), Pos::new(2, 2, 2)));
        let stone = sim.registry_mut().intern("minecraft:stone").unwrap();
        sim.record_timeline();
        sim.place_block(Pos::new(0, 0, 0), stone);
        sim.step();
        sim.step();
        let timeline = sim.recorded_timeline().expect("timeline");
        assert_eq!(timeline.start_tick, 0);
        assert_eq!(timeline.end_tick, 2, "two completed ticks");
    }
}
