//! The simulation driver: the phase walk and the control surface.
//!
//! This is the crate's public face. It owns the world and the queues, walks the
//! ten phases in order, and offers the controls the product needs: step, run,
//! reset, checkpoint, restore.
//!
//! No block behaviour lives here. Behaviour arrives in a later phase of the
//! project via a registry; this module's job is to be the thing whose ordering
//! is already right when that happens.

use crate::phase::{Phase, PHASE_ORDER};
use crate::pos::{Bounds, Pos};
use crate::schedule::{BlockEvent, EventQueue, TickPriority, TickQueue};
use crate::state::StateRegistry;
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
#[derive(Debug, Clone)]
pub struct Simulation {
    registry: StateRegistry,
    world: World,
    ticks: TickQueue,
    events: EventQueue,
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
            tick: 0,
            initial,
        }
    }

    /// Treat the current state as the baseline that [`Simulation::reset`] returns to.
    ///
    /// Call this after loading a structure, so `reset` means "back to the loaded
    /// build" rather than "back to an empty region".
    pub fn mark_initial(&mut self) {
        self.initial = self.checkpoint();
    }

    /// The current tick number.
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
        self.ticks.is_empty() && self.events.is_empty()
    }

    /// Run exactly one tick, walking all ten phases in order.
    pub fn step(&mut self) -> StopReason {
        for phase in PHASE_ORDER {
            if let Some(stop) = self.run_phase(phase) {
                // A phase-level failure ends the tick immediately; continuing
                // would build further state on top of a known-bad tick.
                return stop;
            }
        }
        self.tick += 1;
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

    /// Run one phase. `Some(stop)` aborts the tick.
    fn run_phase(&mut self, phase: Phase) -> Option<StopReason> {
        match phase {
            // Not simulated. Named and walked so the order stays structurally
            // right and filling one in never re-plumbs its neighbours.
            Phase::WorldBorder | Phase::Weather | Phase::Raids | Phase::ChunkManager => None,

            Phase::BlockTicks => {
                // Drained here; dispatch to block behaviour arrives with the
                // behaviour registry. Draining already exercises the ordering
                // contract, which is the part worth getting right first.
                let _due = self.ticks.drain_due(self.tick);
                None
            }

            Phase::FluidTicks => None,

            Phase::BlockEvents => self.run_block_events(),

            Phase::Entities | Phase::BlockEntities | Phase::PlayerInputs => None,
        }
    }

    /// The block-events phase: drain, handle, repeat until empty.
    fn run_block_events(&mut self) -> Option<StopReason> {
        for _ in 0..MAX_EVENT_CHAIN {
            let batch = self.events.take();
            if batch.is_empty() {
                return None;
            }
            // Handlers land here once behaviour exists. Each batch may enqueue
            // the next, which is why this loops rather than draining once.
            for _event in batch {}
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
