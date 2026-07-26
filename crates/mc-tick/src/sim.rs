//! The simulation driver: the phase walk and the control surface.
//!
//! This is the crate's public face. It owns the world and the queues, walks the
//! ten phases in order, and offers the controls the product needs: step, run,
//! reset, checkpoint, restore.
//!
//! No block behaviour lives here. Behaviour arrives in a later phase of the
//! project via a registry; this module's job is to be the thing whose ordering
//! is already right when that happens.

use crate::behaviour::{BehaviourTable, BlockChange, PendingMove, TickCtx};
use crate::phase::{Phase, PHASE_ORDER};
use crate::pos::{Bounds, Pos};
use crate::schedule::{BlockEvent, EventQueue, TickPriority, TickQueue};
use crate::state::{StateId, StateRegistry};
use crate::world::World;

/// How many times the block-events phase may chain within one tick before the
/// simulation gives up.
///
/// Events legitimately enqueue further events, so the phase loops until a drain
/// comes back empty. A contraption bug — or a bug of ours — could make that
/// never happen, and hanging is a worse failure than reporting. The number is
/// generous enough that no real build should reach it.
const MAX_EVENT_CHAIN: usize = 1024;

/// Why a run stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// The requested number of ticks completed.
    Completed,
    /// Nothing remained scheduled.
    Quiescent,
    /// The tick budget ran out before quiescence.
    BudgetExhausted,
    /// The block-events phase chained past [`MAX_EVENT_CHAIN`].
    ///
    /// Surfaced rather than swallowed: it means either a pathological build or a
    /// defect in our event handling, and both need to be seen.
    EventChainLimit,
}

/// A saved simulation state.
///
/// Opaque by design — a checkpoint is a value to hand back to
/// [`Simulation::restore`], not something to inspect or mutate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkpoint {
    tick: u64,
    world: World,
    ticks: TickQueue,
    events: EventQueue,
}

/// A controllable, deterministic simulation of a bounded region.
///
/// Not `Clone`: it owns trait-object behaviours, which cannot be duplicated
/// meaningfully. Duplicating a *run* is what [`Checkpoint`] is for, and that
/// carries the mutable state rather than the behaviour table — which is
/// immutable once built anyway.
#[derive(Debug)]
pub struct Simulation {
    registry: StateRegistry,
    world: World,
    ticks: TickQueue,
    events: EventQueue,
    behaviours: BehaviourTable,
    /// States encountered during a tick with no registered behaviour.
    ///
    /// Accumulated during dispatch, where the table cannot be mutated, and folded
    /// back in by [`Simulation::unknown_report`].
    unknown_seen: Vec<StateId>,
    /// Scratch buffer for neighbour notifications; reused every dispatch.
    updates: Vec<(Pos, crate::pos::Dir)>,
    /// Deferred writes awaiting their block-entities phase.
    moves: Vec<PendingMove>,
    /// Torch toggle history for burnout, pruned as it ages out.
    toggles: Vec<(Pos, u64)>,
    /// Stored comparator output strengths; vanilla's ComparatorBlockEntity.
    comparator_out: std::collections::HashMap<Pos, u8>,
    /// Recorded block changes, when recording is enabled.
    log: Option<Vec<BlockChange>>,
    /// Whether a tick's phase walk is currently executing.
    ///
    /// Dispatches outside of one — settling a freshly placed structure, a block
    /// broken or clicked from the control surface — are *boundary* actions: they
    /// happen in the server loop where the game time still reads the last
    /// completed tick, and anything they schedule fires one tick sooner than an
    /// in-phase schedule would. See [`TickCtx::boundary`].
    in_tick: bool,
    /// Where block entities actually tick, when narrower than the world.
    ///
    /// Vanilla freezes block entities outside the block-ticking chunk area: a
    /// moving piston pushed across that edge stays a `moving_piston` placeholder
    /// indefinitely — and, being immovable, it then *blocks* further pushes.
    /// Captured with the manual engine, whose flying machine crosses a chunk
    /// border after two steps and stops dead against its own frozen blocks.
    /// `None` means the whole world ticks.
    ticking: Option<Bounds>,
    tick: u64,
    /// The state to return to on [`Simulation::reset`].
    ///
    /// Held from construction so `reset` is exactly "as loaded" rather than
    /// "as I last remembered to snapshot".
    initial: Checkpoint,
}

impl Simulation {
    /// A new simulation over `bounds`, all air.
    pub fn new(bounds: Bounds) -> Self {
        let world = World::new(bounds);
        let initial = Checkpoint {
            tick: 0,
            world: world.clone(),
            ticks: TickQueue::new(),
            events: EventQueue::new(),
        };
        Self {
            registry: StateRegistry::new(),
            world,
            ticks: TickQueue::new(),
            events: EventQueue::new(),
            behaviours: BehaviourTable::new(),
            unknown_seen: Vec::new(),
            updates: Vec::new(),
            moves: Vec::new(),
            toggles: Vec::new(),
            comparator_out: std::collections::HashMap::new(),
            log: None,
            in_tick: false,
            ticking: None,
            tick: 0,
            initial,
        }
    }

    /// Restrict block-entity ticking to `bounds`.
    ///
    /// A pending move whose position lies outside never resolves — the
    /// placeholder freezes in place exactly as a block entity in a
    /// loaded-but-not-ticking chunk does. Needed to conform against captures,
    /// whose ticking area ends at a chunk border the capture harness chose.
    /// Note that frozen pending moves also keep [`Simulation::is_quiescent`]
    /// false, so bounded runs should use [`Simulation::run`].
    pub fn set_ticking_bounds(&mut self, bounds: Bounds) {
        self.ticking = Some(bounds);
    }

    /// Start recording block changes, for comparison against a vanilla trace.
    ///
    /// Off by default: the tick loop should not pay for observability nobody asked
    /// for, and a simulation used for timing runs millions of ticks.
    pub fn record(&mut self) {
        self.log = Some(Vec::new());
    }

    /// The changes recorded since [`Simulation::record`], in order.
    pub fn recorded(&self) -> &[BlockChange] {
        self.log.as_deref().unwrap_or(&[])
    }

    /// The behaviour table.
    pub fn behaviours(&self) -> &BehaviourTable {
        &self.behaviours
    }

    /// The behaviour table, for registering block behaviour.
    pub fn behaviours_mut(&mut self) -> &mut BehaviourTable {
        &mut self.behaviours
    }

    /// A report of every block state encountered without a behaviour, or `None`.
    ///
    /// **Check this before trusting a result.** A contraption containing one
    /// unimplemented component simulates that component as nothing and produces a
    /// plausible, wrong answer — which is the one failure mode that would quietly
    /// undermine this whole project.
    pub fn unknown_report(&mut self) -> Option<String> {
        let seen = std::mem::take(&mut self.unknown_seen);
        for state in seen {
            self.behaviours.note_unknown(state);
        }
        self.behaviours.note_unknown_in(&self.world);
        self.behaviours.unknown_report(&self.registry)
    }

    /// Treat the current state as the baseline that [`Simulation::reset`] returns to.
    ///
    /// Call this after loading a structure, so `reset` means "back to the loaded
    /// build" rather than "back to an empty region".
    pub fn mark_initial(&mut self) {
        self.initial = self.checkpoint();
    }

    /// The number of **completed** ticks, i.e. the index of the tick that will
    /// run next.
    ///
    /// The distinction matters and is a classic source of off-by-one errors in
    /// timing work: a tick scheduled for tick 4 fires *during* tick 4, and once
    /// tick 4 has finished this reads **5**. So after running a contraption to
    /// quiescence, this is one greater than the last tick on which anything
    /// happened — not the tick the last event occurred on.
    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    /// The block storage.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// The block storage, mutably.
    ///
    /// Writing here does not schedule updates — it is for loading a structure
    /// and for tests. Once behaviour exists, in-tick changes must go through the
    /// tick context so neighbour updates happen.
    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// The state registry.
    pub fn registry(&self) -> &StateRegistry {
        &self.registry
    }

    /// The state registry, mutably, for interning while loading.
    pub fn registry_mut(&mut self) -> &mut StateRegistry {
        &mut self.registry
    }

    /// Registry and world together, for loaders that must intern while writing.
    pub fn registry_and_world_mut(&mut self) -> (&mut StateRegistry, &mut World) {
        (&mut self.registry, &mut self.world)
    }

    /// Schedule a block tick, as a block would.
    pub fn schedule_tick(&mut self, pos: Pos, delay: u64, priority: TickPriority) {
        self.ticks.schedule(pos, self.tick, delay, priority);
    }

    /// Queue a block event for this tick's block-events phase.
    pub fn queue_event(&mut self, event: BlockEvent) {
        self.events.push(event);
    }

    /// Whether anything is scheduled.
    ///
    /// Quiescence is the natural stopping condition for timing a contraption:
    /// run until the build stops doing anything.
    pub fn is_quiescent(&self) -> bool {
        self.ticks.is_empty() && self.events.is_empty() && self.moves.is_empty()
    }

    /// Run exactly one tick, walking all ten phases in order.
    pub fn step(&mut self) -> StopReason {
        self.in_tick = true;
        for phase in PHASE_ORDER {
            if let Some(stop) = self.run_phase(phase) {
                // A phase-level failure ends the tick immediately; continuing
                // would build further state on top of a known-bad tick.
                self.in_tick = false;
                return stop;
            }
        }
        self.in_tick = false;
        self.tick += 1;
        // Burnout only looks back a fixed window, so anything older is dead weight.
        let horizon = self.tick.saturating_sub(crate::components::TORCH_BURNOUT_WINDOW);
        self.toggles.retain(|(_, t)| *t >= horizon);
        StopReason::Completed
    }

    /// Run `ticks` ticks, stopping early only on failure.
    pub fn run(&mut self, ticks: u64) -> StopReason {
        for _ in 0..ticks {
            match self.step() {
                StopReason::Completed => {}
                other => return other,
            }
        }
        StopReason::Completed
    }

    /// Run until nothing is scheduled, or `budget` ticks elapse.
    pub fn run_until_quiescent(&mut self, budget: u64) -> StopReason {
        for _ in 0..budget {
            if self.is_quiescent() {
                return StopReason::Quiescent;
            }
            match self.step() {
                StopReason::Completed => {}
                other => return other,
            }
        }
        if self.is_quiescent() {
            StopReason::Quiescent
        } else {
            StopReason::BudgetExhausted
        }
    }

    /// Capture the current state.
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            tick: self.tick,
            world: self.world.clone(),
            ticks: self.ticks.clone(),
            events: self.events.clone(),
        }
    }

    /// Return to a captured state.
    ///
    /// The registry is intentionally not restored: interning more states is
    /// additive and monotonic, and ids already handed out must keep meaning the
    /// same thing. Rolling it back would invalidate every `StateId` a caller
    /// holds.
    pub fn restore(&mut self, checkpoint: &Checkpoint) {
        self.tick = checkpoint.tick;
        self.world = checkpoint.world.clone();
        self.ticks = checkpoint.ticks.clone();
        self.events = checkpoint.events.clone();
    }

    /// Return to the state at construction, or at the last [`Simulation::mark_initial`].
    pub fn reset(&mut self) {
        let initial = self.initial.clone();
        self.restore(&initial);
    }

    /// Deliver queued neighbour notifications until none remain.
    ///
    /// Bounded: a circuit that keeps re-notifying itself would otherwise hang, and
    /// a reported limit is far better than a simulation that never returns. The
    /// `set` guard against no-op writes means real circuits terminate long before
    /// this.
    fn propagate(&mut self) {
        const MAX_UPDATE_CASCADE: usize = 8192;

        let mut delivered = 0;
        while let Some((pos, from)) = self.updates.pop() {
            delivered += 1;
            if delivered > MAX_UPDATE_CASCADE {
                self.updates.clear();
                break;
            }
            let state = self.world.get(pos);
            let Some(behaviour) = self.behaviours.get(state) else {
                if state != StateId::AIR {
                    self.unknown_seen.push(state);
                }
                continue;
            };
            let mut ctx = TickCtx {
                world: &mut self.world,
                ticks: &mut self.ticks,
                events: &mut self.events,
                states: &self.registry,
                tick: self.tick,
                // A cascade running outside a phase walk is a boundary action;
                // its schedules use last-completed-tick time. See TickCtx::boundary.
                boundary: !self.in_tick,
                updates: &mut self.updates,
                moves: &mut self.moves,
                toggles: &mut self.toggles,
                comparator_out: &mut self.comparator_out,
                log: self.log.as_mut(),
            };
            behaviour.on_neighbor_changed(&mut ctx, pos, from);
        }
    }

    /// Notify the neighbours of `pos`, as an external change would.
    ///
    /// This is how a lever flip or a block break enters the simulation.
    pub fn notify_neighbors(&mut self, pos: Pos) {
        for dir in crate::pos::ALL_DIRS {
            self.updates.push((pos.offset(dir), dir.opposite()));
        }
        self.propagate();
    }

    /// Give every non-air block a chance to react, as placing a build does.
    ///
    /// Vanilla blocks receive `onPlace` when they are put down, and several use it
    /// to evaluate their surroundings immediately — `PistonBaseBlock.onPlace` calls
    /// `checkIfExtend`, which is why a piston with a quasi-connectivity source
    /// diagonally above it extends the moment a structure is placed, without any
    /// neighbour of the piston having changed.
    ///
    /// Call this after loading a structure and before running. Without it a build
    /// sits inert until something happens to touch it, which is not what the game
    /// does and made an early conformance run produce no events at all.
    ///
    /// Each block is notified **from all six directions**, not once nominally.
    /// `StructureTemplate.placeInWorld` ends with an update pass that hands every
    /// placed block shape updates from every side, and observers depend on the
    /// difference: an observer pulses when the side it *faces* reports a change,
    /// so structure placement pulses every observer once. Captured with the
    /// manual-engine build — all its observers fire on placement, whatever they
    /// face. The duplicate piston events this produces are collapsed by the
    /// event queue's set semantics, as vanilla's are.
    pub fn settle(&mut self) {
        let occupied: Vec<Pos> = self
            .world
            .iter_non_air()
            .map(|(pos, _)| pos)
            .collect();
        for pos in occupied {
            for dir in crate::pos::ALL_DIRS {
                self.updates.push((pos, dir));
            }
        }
        self.propagate();
    }

    /// Write `state` at `pos` from outside the simulation — a placed or broken
    /// block, the way an actuation reaches a real world.
    ///
    /// A boundary action: the write is observed by the upcoming tick (and logged
    /// against it), while anything it schedules uses last-completed-tick time,
    /// exactly as a block placed between server ticks behaves. Breaking a block
    /// is placing air.
    pub fn place_block(&mut self, pos: Pos, state: StateId) {
        let previous = self.world.get(pos);
        if previous == state {
            return;
        }
        self.world.set(pos, state);
        if let Some(log) = self.log.as_mut() {
            log.push(BlockChange { tick: self.tick, pos, from: previous, to: state });
        }
        self.notify_neighbors(pos);
    }

    /// Right-click the block at `pos` with an empty hand, as a player would.
    ///
    /// This is the `Phase::PlayerInputs` input path. Vanilla processes use-block
    /// packets in the server loop *between* level ticks — which is why the phase
    /// list places player inputs last — so the click executes immediately as a
    /// boundary action rather than being queued into the next phase walk: its
    /// changes are observed by the upcoming tick, and anything it schedules uses
    /// last-completed-tick time. Captured with a note block and an observer: the
    /// note cycles on the click's tick, the observer pulses one tick later.
    pub fn use_block(&mut self, pos: Pos) {
        let state = self.world.get(pos);
        let Some(behaviour) = self.behaviours.get(state) else {
            if state != StateId::AIR {
                self.unknown_seen.push(state);
            }
            return;
        };
        let mut ctx = TickCtx {
            world: &mut self.world,
            ticks: &mut self.ticks,
            events: &mut self.events,
            states: &self.registry,
            tick: self.tick,
            boundary: true,
            updates: &mut self.updates,
            moves: &mut self.moves,
            toggles: &mut self.toggles,
            comparator_out: &mut self.comparator_out,
            log: self.log.as_mut(),
        };
        behaviour.on_used(&mut ctx, pos);
        self.propagate();
    }

    /// Run one phase. `Some(stop)` aborts the tick.
    fn run_phase(&mut self, phase: Phase) -> Option<StopReason> {
        match phase {
            // Not simulated. Named and walked so the order stays structurally
            // right and filling one in never re-plumbs its neighbours.
            Phase::WorldBorder | Phase::Weather | Phase::Raids | Phase::ChunkManager => None,

            Phase::BlockTicks => {
                // Drain first, then dispatch: a behaviour may schedule further
                // ticks, and those belong to a later tick rather than extending
                // this drain. Draining into a Vec is what keeps that boundary
                // sharp.
                let due = self.ticks.drain_due(self.tick);
                for entry in due {
                    let state = self.world.get(entry.pos);
                    let Some(behaviour) = self.behaviours.get(state) else {
                        // Unregistered: record it rather than treating the block
                        // as inert. See BehaviourTable's module docs.
                        self.unknown_seen.push(state);
                        continue;
                    };
                    let mut ctx = TickCtx {
                        world: &mut self.world,
                        ticks: &mut self.ticks,
                        events: &mut self.events,
                        states: &self.registry,
                        tick: self.tick,
            boundary: false,
                        updates: &mut self.updates,
                moves: &mut self.moves,
                        toggles: &mut self.toggles,
                        comparator_out: &mut self.comparator_out,
                        log: self.log.as_mut(),
                    };
                    behaviour.on_scheduled_tick(&mut ctx, entry.pos);
                    self.propagate();
                }
                None
            }

            Phase::FluidTicks => None,

            Phase::BlockEvents => self.run_block_events(),

            Phase::BlockEntities => {
                // Where a moving piston's blocks land, two ticks after the block
                // event that started them. Captured from vanilla:
                //   tick 0  stone -> moving_piston
                //   tick 2  moving_piston -> stone (at its destination)
                let ticking = self.ticking;
                let in_ticking =
                    move |pos: Pos| ticking.as_ref().is_none_or(|bounds| bounds.contains(pos));
                let mut due: Vec<PendingMove> = self
                    .moves
                    .iter()
                    .filter(|m| m.resolve_on <= self.tick && in_ticking(m.pos))
                    .copied()
                    .collect();
                // Resolve in the world's canonical order rather than the order the
                // moves happened to be queued. The captured trace lands the head
                // slot before the block beyond it, which insertion order gets
                // backwards — and an order-sensitive diff treats that as a
                // divergence, correctly, because update order is observable.
                due.sort_by_key(|m| (m.resolve_on, m.pos.y, m.pos.z, m.pos.x));
                // Frozen moves (outside the ticking bounds) stay pending: a
                // block entity in a non-ticking chunk is suspended, not gone.
                self.moves
                    .retain(|m| m.resolve_on > self.tick || !in_ticking(m.pos));
                for entry in due {
                    let previous = self.world.get(entry.pos);
                    if previous == entry.state {
                        continue;
                    }
                    self.world.set(entry.pos, entry.state);
                    // Record here too. These writes bypass TickCtx, and leaving
                    // them out of the log made a trace end two ticks early —
                    // the movement landed but the diff never saw it.
                    if let Some(log) = self.log.as_mut() {
                        log.push(BlockChange {
                            tick: self.tick,
                            pos: entry.pos,
                            from: previous,
                            to: entry.state,
                        });
                    }
                    // `PistonMovingBlockEntity.finalTick` runs the landed state
                    // through `updateFromNeighbourShapes` — a shape update from
                    // every side — and then `neighborChanged`s the position
                    // itself. The landed block re-examines its world, which is
                    // how an observer that was *moved* pulses two ticks after it
                    // lands, and how a landed piston notices power waiting for it.
                    for dir in crate::pos::ALL_DIRS {
                        self.updates.push((entry.pos, dir));
                    }
                    for dir in crate::pos::ALL_DIRS {
                        self.updates.push((entry.pos.offset(dir), dir.opposite()));
                    }
                }
                self.propagate();
                None
            }

            Phase::Entities | Phase::PlayerInputs => None,
        }
    }

    /// The block-events phase: drain, handle, repeat until empty.
    fn run_block_events(&mut self) -> Option<StopReason> {
        for _ in 0..MAX_EVENT_CHAIN {
            let batch = self.events.take();
            if batch.is_empty() {
                return None;
            }
            // Each batch may enqueue the next, which is why this loops rather
            // than draining once — the game chains block events within a tick and
            // piston contraptions depend on it.
            for event in batch {
                let state = self.world.get(event.pos);
                let Some(behaviour) = self.behaviours.get(state) else {
                    self.unknown_seen.push(state);
                    continue;
                };
                let mut ctx = TickCtx {
                    world: &mut self.world,
                    ticks: &mut self.ticks,
                    events: &mut self.events,
                    states: &self.registry,
                    tick: self.tick,
            boundary: false,
                    updates: &mut self.updates,
                moves: &mut self.moves,
                        toggles: &mut self.toggles,
                        comparator_out: &mut self.comparator_out,
                        log: self.log.as_mut(),
                };
                behaviour.on_block_event(&mut ctx, event.pos, event.id, event.param);
                self.propagate();
            }
        }
        // Still not empty after the cap: report instead of spinning.
        if self.events.is_empty() {
            None
        } else {
            Some(StopReason::EventChainLimit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateId;

    fn sim() -> Simulation {
        Simulation::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(7, 7, 7)))
    }

    #[test]
    fn a_fresh_simulation_is_quiescent_at_tick_zero() {
        let s = sim();
        assert_eq!(s.tick_count(), 0);
        assert!(s.is_quiescent());
    }

    #[test]
    fn stepping_n_times_equals_running_n() {
        // Part of the control-surface contract from the plan.
        let mut stepped = sim();
        let mut ran = sim();
        for _ in 0..17 {
            stepped.step();
        }
        ran.run(17);
        assert_eq!(stepped.tick_count(), ran.tick_count());
        assert_eq!(stepped.world(), ran.world());
    }

    #[test]
    fn checkpoint_restore_reproduces_state_exactly() {
        let mut s = sim();
        let stone = s.registry_mut().intern("minecraft:stone").unwrap();
        s.world_mut().set(Pos::new(1, 1, 1), stone);
        s.schedule_tick(Pos::new(2, 2, 2), 5, TickPriority::Normal);

        let saved = s.checkpoint();
        let saved_tick = s.tick_count();

        s.run(3);
        s.world_mut().set(Pos::new(1, 1, 1), StateId::AIR);
        assert_ne!(s.tick_count(), saved_tick);

        s.restore(&saved);
        assert_eq!(s.tick_count(), saved_tick);
        assert_eq!(s.world().get(Pos::new(1, 1, 1)), stone);
        assert!(s.ticks.has_pending_at(Pos::new(2, 2, 2), 0));
    }

    #[test]
    fn reset_returns_to_the_marked_baseline_not_an_empty_world() {
        let mut s = sim();
        let stone = s.registry_mut().intern("minecraft:stone").unwrap();
        s.world_mut().set(Pos::new(3, 3, 3), stone);
        s.mark_initial();

        s.world_mut().set(Pos::new(3, 3, 3), StateId::AIR);
        s.run(4);
        s.reset();

        assert_eq!(s.tick_count(), 0);
        assert_eq!(s.world().get(Pos::new(3, 3, 3)), stone, "reset is 'as loaded'");
    }

    #[test]
    fn reset_without_marking_returns_to_an_empty_region() {
        let mut s = sim();
        let stone = s.registry_mut().intern("minecraft:stone").unwrap();
        s.world_mut().set(Pos::new(3, 3, 3), stone);
        s.reset();
        assert_eq!(s.world().non_air_count(), 0);
    }

    #[test]
    fn restore_keeps_interned_ids_valid() {
        // Ids handed out before a checkpoint must still resolve after a restore,
        // or every StateId a caller holds becomes a dangling reference.
        let mut s = sim();
        let saved = s.checkpoint();
        let stone = s.registry_mut().intern("minecraft:stone").unwrap();
        s.restore(&saved);
        assert_eq!(s.registry().descriptor(stone), Some("minecraft:stone"));
    }

    #[test]
    fn run_until_quiescent_stops_immediately_when_nothing_is_scheduled() {
        let mut s = sim();
        assert_eq!(s.run_until_quiescent(100), StopReason::Quiescent);
        assert_eq!(s.tick_count(), 0, "no ticks burned on an idle world");
    }

    #[test]
    fn run_until_quiescent_drains_a_scheduled_tick() {
        let mut s = sim();
        s.schedule_tick(Pos::new(1, 1, 1), 3, TickPriority::Normal);
        assert!(!s.is_quiescent());
        let reason = s.run_until_quiescent(100);
        assert_eq!(reason, StopReason::Quiescent);
        assert!(s.tick_count() >= 3, "must reach the scheduled tick");
    }

    /// Turns itself into `becomes` when its scheduled tick fires.
    struct Transmute {
        becomes: StateId,
    }
    impl crate::behaviour::BlockBehaviour for Transmute {
        fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
            ctx.set(pos, self.becomes);
        }
        fn name(&self) -> &'static str {
            "transmute"
        }
    }

    /// Queues a block event when ticked, and records the event by setting a block.
    struct EventEcho {
        marker: StateId,
    }
    impl crate::behaviour::BlockBehaviour for EventEcho {
        fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
            ctx.queue_event(pos, 7, 3);
        }
        fn on_block_event(&self, ctx: &mut TickCtx<'_>, pos: Pos, id: u8, param: u8) -> bool {
            if id == 7 && param == 3 {
                ctx.set(pos.offset(crate::pos::Dir::Up), self.marker);
            }
            true
        }
        fn name(&self) -> &'static str {
            "event_echo"
        }
    }

    #[test]
    fn a_scheduled_tick_dispatches_to_the_registered_behaviour() {
        let mut s = sim();
        let before = s.registry_mut().intern("test:before").unwrap();
        let after = s.registry_mut().intern("test:after").unwrap();
        s.behaviours_mut()
            .register(before, Box::new(Transmute { becomes: after }));
        // The block we turn *into* needs a behaviour too. Without this the run
        // reports `test:after` as unimplemented — which is the loud-failure
        // mechanism working: a block produced mid-simulation is just as capable
        // of being an unhandled component as one placed at load.
        s.behaviours_mut()
            .register(after, Box::new(crate::behaviour::Inert::new("after")));

        let pos = Pos::new(2, 2, 2);
        s.world_mut().set(pos, before);
        s.schedule_tick(pos, 2, TickPriority::Normal);

        s.run_until_quiescent(50);
        assert_eq!(s.world().get(pos), after, "behaviour must have run");
        assert_eq!(s.unknown_report(), None, "everything present was registered");
    }

    #[test]
    fn a_block_event_reaches_its_behaviour_in_the_same_tick() {
        // The scheduled tick queues an event; the event runs in phase 7 of that
        // same tick, so one tick is enough for both to have happened.
        let mut s = sim();
        let echo = s.registry_mut().intern("test:echo").unwrap();
        let marker = s.registry_mut().intern("test:marker").unwrap();
        s.behaviours_mut()
            .register(echo, Box::new(EventEcho { marker }));
        s.behaviours_mut()
            .register(marker, Box::new(crate::behaviour::Inert::new("marker")));

        let pos = Pos::new(3, 3, 3);
        s.world_mut().set(pos, echo);
        s.schedule_tick(pos, 0, TickPriority::Normal);

        s.step();
        assert_eq!(
            s.world().get(pos.offset(crate::pos::Dir::Up)),
            marker,
            "block event must dispatch in the same tick as the scheduled tick"
        );
    }

    #[test]
    fn an_unregistered_block_is_reported_rather_than_simulated_as_nothing() {
        // The failure mode this project cannot tolerate: an unimplemented
        // component silently behaving as air and yielding a plausible, wrong
        // answer.
        let mut s = sim();
        let observer = s.registry_mut().intern("minecraft:observer[facing=up]").unwrap();
        let pos = Pos::new(1, 1, 1);
        s.world_mut().set(pos, observer);
        s.schedule_tick(pos, 1, TickPriority::Normal);

        s.run_until_quiescent(20);

        let report = s.unknown_report().expect("must report the gap");
        assert!(report.contains("minecraft:observer"), "{report}");
    }

    #[test]
    fn every_phase_is_walked_each_tick() {
        // Guards the invariant that the tick is the full declared sequence. If a
        // phase is ever dropped from run_phase's match, this and the exhaustive
        // match together are what catch it.
        assert_eq!(PHASE_ORDER.len(), 10);
        let mut s = sim();
        assert_eq!(s.step(), StopReason::Completed);
        assert_eq!(s.tick_count(), 1);
    }
}
