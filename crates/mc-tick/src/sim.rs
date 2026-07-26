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

/// One recorded entity observation.
#[derive(Debug, Clone, PartialEq)]
pub enum EntityEvent {
    /// The entity moved (or first appeared).
    Moved {
        /// Trace-stable id.
        id: u32,
        /// e.g. `minecraft:item`.
        entity_type: String,
        /// Position after the tick.
        pos: [f64; 3],
        /// Velocity after the tick.
        velocity: [f64; 3],
    },
    /// The entity left the world.
    Removed {
        /// Trace-stable id.
        id: u32,
    },
}

/// The simulation's world as item physics sees it.
struct SimCollision<'a> {
    world: &'a World,
    solidity: &'a [bool],
    frictions: &'a [f32],
    heights: &'a [f32],
    webs: &'a [bool],
    water_kinds: &'a [Option<crate::fluid::WaterKind>],
    bubble_kinds: &'a [Option<bool>],
}

impl crate::entity::CollisionWorld for SimCollision<'_> {
    fn is_solid(&self, pos: Pos) -> bool {
        let state = self.world.get(pos);
        self.solidity.get(state.raw() as usize).copied().unwrap_or(false)
    }
    fn friction(&self, pos: Pos) -> f32 {
        let state = self.world.get(pos);
        self.frictions.get(state.raw() as usize).copied().unwrap_or(0.6)
    }
    fn water(&self, pos: Pos) -> Option<crate::fluid::WaterKind> {
        let state = self.world.get(pos);
        self.water_kinds.get(state.raw() as usize).copied().flatten()
    }
    fn bubble(&self, pos: Pos) -> Option<bool> {
        let state = self.world.get(pos);
        self.bubble_kinds.get(state.raw() as usize).copied().flatten()
    }
    fn is_air(&self, pos: Pos) -> bool {
        self.world.get(pos) == StateId::AIR
    }
    fn solid_height(&self, pos: Pos) -> f64 {
        let state = self.world.get(pos);
        f64::from(self.heights.get(state.raw() as usize).copied().unwrap_or(1.0))
    }
    fn is_web(&self, pos: Pos) -> bool {
        let state = self.world.get(pos);
        self.webs.get(state.raw() as usize).copied().unwrap_or(false)
    }
}

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
#[derive(Debug, Clone, PartialEq)]
pub struct Checkpoint {
    tick: u64,
    world: World,
    ticks: TickQueue,
    fluids: TickQueue,
    events: EventQueue,
    moves: Vec<PendingMove>,
    toggles: Vec<(Pos, u64)>,
    comparator_out: std::collections::HashMap<Pos, u8>,
    inventories: std::collections::HashMap<Pos, crate::inventory::Inventory>,
    hopper_state: std::collections::HashMap<Pos, crate::behaviour::HopperState>,
    item_entities: crate::entity::ItemEntities,
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
    /// Scheduled fluid ticks — vanilla's separate queue, drained in
    /// `Phase::FluidTicks`.
    fluids: TickQueue,
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
    /// Container contents by position; vanilla's inventory block entities.
    inventories: std::collections::HashMap<Pos, crate::inventory::Inventory>,
    /// Per-hopper cooldown and tick bookkeeping.
    hopper_state: std::collections::HashMap<Pos, crate::behaviour::HopperState>,
    /// The world's item entities.
    item_entities: crate::entity::ItemEntities,
    /// Full-cube collision, indexed by `StateId`; see [`Simulation::set_physics_tables`].
    solidity: Vec<bool>,
    /// Block friction, indexed by `StateId`.
    frictions: Vec<f32>,
    /// Solid collision-box heights, indexed by `StateId`.
    heights: Vec<f32>,
    /// Cobwebs, indexed by `StateId`.
    webs: Vec<bool>,
    /// Water per state, indexed by `StateId`; see [`Simulation::set_fluid_tables`].
    water_kinds: Vec<Option<crate::fluid::WaterKind>>,
    /// Bubble columns per state (`Some(drag_down)`), indexed by `StateId`.
    bubble_kinds: Vec<Option<bool>>,
    /// Entity positions as of the last recorded tick, for event emission.
    entity_snapshot: std::collections::HashMap<u32, [f64; 3]>,
    /// Recorded entity events, when recording is enabled.
    ent_log: Option<Vec<(u64, EntityEvent)>>,
    /// Ticking block entities, in registration order.
    ///
    /// Vanilla's `tickBlockEntities` walks its list in insertion order — for a
    /// placed structure, block order. Only hoppers register so far. Blocks with
    /// block entities cannot be pushed, so the list is stable during a run.
    tickers: Vec<Pos>,
    /// Recorded block changes, when recording is enabled.
    log: Option<Vec<BlockChange>>,
    /// Recorded container-slot changes, when recording is enabled.
    inv_log: Option<Vec<crate::behaviour::InventoryChange>>,
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
            fluids: TickQueue::new(),
            events: EventQueue::new(),
            moves: Vec::new(),
            toggles: Vec::new(),
            comparator_out: std::collections::HashMap::new(),
            inventories: std::collections::HashMap::new(),
            hopper_state: std::collections::HashMap::new(),
            item_entities: crate::entity::ItemEntities::default(),
        };
        Self {
            registry: StateRegistry::new(),
            world,
            ticks: TickQueue::new(),
            fluids: TickQueue::new(),
            events: EventQueue::new(),
            behaviours: BehaviourTable::new(),
            unknown_seen: Vec::new(),
            updates: Vec::new(),
            moves: Vec::new(),
            toggles: Vec::new(),
            comparator_out: std::collections::HashMap::new(),
            inventories: std::collections::HashMap::new(),
            hopper_state: std::collections::HashMap::new(),
            item_entities: crate::entity::ItemEntities::default(),
            solidity: Vec::new(),
            frictions: Vec::new(),
            heights: Vec::new(),
            webs: Vec::new(),
            water_kinds: Vec::new(),
            bubble_kinds: Vec::new(),
            entity_snapshot: std::collections::HashMap::new(),
            ent_log: None,
            tickers: Vec::new(),
            log: None,
            inv_log: None,
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
        self.inv_log = Some(Vec::new());
        self.ent_log = Some(Vec::new());
        self.entity_snapshot.clear();
        for item in &self.item_entities.items {
            self.entity_snapshot.insert(item.id, item.pos);
        }
    }

    /// The entity events recorded since [`Simulation::record`].
    pub fn recorded_entities(&self) -> &[(u64, EntityEvent)] {
        self.ent_log.as_deref().unwrap_or(&[])
    }

    /// Spawn an item entity with the id a capture recorded for it — the id
    /// feeds vanilla's rest-flush phase, so conformance runs must match it.
    pub fn spawn_item_with_id(
        &mut self,
        id: u32,
        item: (String, u8),
        pos: [f64; 3],
        vel: [f64; 3],
        pickup_delay: u32,
    ) -> u32 {
        self.item_entities.spawn_with_id(id, item, pos, vel, pickup_delay)
    }

    /// Spawn an item entity, as loading a structure's entity list does.
    pub fn spawn_item(
        &mut self,
        item: (String, u8),
        pos: [f64; 3],
        vel: [f64; 3],
        pickup_delay: u32,
    ) -> u32 {
        self.item_entities.spawn(item, pos, vel, pickup_delay)
    }

    /// The live item entities.
    pub fn item_entities(&self) -> &[crate::entity::ItemEntityState] {
        &self.item_entities.items
    }

    /// Every item entity ever spawned: `(id, item id)`, surviving removal.
    pub fn item_name_log(&self) -> &[(u32, String)] {
        &self.item_entities.name_log
    }

    /// Set the collision and friction tables, indexed by `StateId`.
    ///
    /// Built by `vanilla::physics_tables` after every state is interned. Item
    /// physics reads these; without them everything but air is vacuum.
    pub fn set_physics_tables(
        &mut self,
        solidity: Vec<bool>,
        frictions: Vec<f32>,
        heights: Vec<f32>,
        webs: Vec<bool>,
    ) {
        self.solidity = solidity;
        self.frictions = frictions;
        self.heights = heights;
        self.webs = webs;
    }

    /// Set the fluid tables, indexed by `StateId`.
    ///
    /// Built by `vanilla::fluid_tables` after every state is interned. Item
    /// buoyancy, currents and bubble columns read these; without them water is
    /// scenery.
    pub fn set_fluid_tables(
        &mut self,
        water_kinds: Vec<Option<crate::fluid::WaterKind>>,
        bubble_kinds: Vec<Option<bool>>,
    ) {
        self.water_kinds = water_kinds;
        self.bubble_kinds = bubble_kinds;
    }

    /// The container-slot changes recorded since [`Simulation::record`].
    pub fn recorded_inventory(&self) -> &[crate::behaviour::InventoryChange] {
        self.inv_log.as_deref().unwrap_or(&[])
    }

    /// Register a ticking block entity at `pos`.
    ///
    /// Order matters and is preserved: it is vanilla's `tickBlockEntities`
    /// insertion order, which decides which of two hoppers moves first.
    pub fn add_block_entity_ticker(&mut self, pos: Pos) {
        if !self.tickers.contains(&pos) {
            self.tickers.push(pos);
        }
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
        self.ticks.is_empty()
            && self.fluids.is_empty()
            && self.events.is_empty()
            && self.moves.is_empty()
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
        self.emit_entity_events();
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

    /// Record entity movements and removals for this tick, then drop the dead.
    ///
    /// Emission is by **position** change, mirroring what a snapshot capture
    /// can see. A resting item's velocity oscillates as gravity accumulates and
    /// collisions flush it — invisible here, on both sides, by construction.
    fn emit_entity_events(&mut self) {
        if self.ent_log.is_none() {
            self.item_entities.items.retain(|item| !item.removed);
            return;
        }
        let mut events: Vec<(u64, EntityEvent)> = Vec::new();
        let mut seen: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for item in &self.item_entities.items {
            if item.removed {
                continue;
            }
            seen.insert(item.id);
            let moved = match self.entity_snapshot.get(&item.id) {
                None => true,
                Some(last) => last
                    .iter()
                    .zip(&item.pos)
                    .any(|(a, b)| (a - b).abs() > 1.0e-9),
            };
            if moved {
                events.push((
                    self.tick,
                    EntityEvent::Moved {
                        id: item.id,
                        entity_type: "minecraft:item".to_string(),
                        pos: item.pos,
                        velocity: item.vel,
                    },
                ));
                self.entity_snapshot.insert(item.id, item.pos);
            }
        }
        let vanished: Vec<u32> = self
            .entity_snapshot
            .keys()
            .copied()
            .filter(|id| !seen.contains(id))
            .collect();
        for id in vanished {
            self.entity_snapshot.remove(&id);
            events.push((self.tick, EntityEvent::Removed { id }));
        }
        if let Some(log) = self.ent_log.as_mut() {
            log.extend(events);
        }
        self.item_entities.items.retain(|item| !item.removed);
    }

    /// Capture the current state.
    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            tick: self.tick,
            world: self.world.clone(),
            ticks: self.ticks.clone(),
            fluids: self.fluids.clone(),
            events: self.events.clone(),
            moves: self.moves.clone(),
            toggles: self.toggles.clone(),
            comparator_out: self.comparator_out.clone(),
            inventories: self.inventories.clone(),
            hopper_state: self.hopper_state.clone(),
            item_entities: self.item_entities.clone(),
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
        self.fluids = checkpoint.fluids.clone();
        self.events = checkpoint.events.clone();
        self.moves = checkpoint.moves.clone();
        self.toggles = checkpoint.toggles.clone();
        self.comparator_out = checkpoint.comparator_out.clone();
        self.inventories = checkpoint.inventories.clone();
        self.hopper_state = checkpoint.hopper_state.clone();
        self.item_entities = checkpoint.item_entities.clone();
        // Event emission restarts from the restored state.
        self.entity_snapshot.clear();
        for item in &self.item_entities.items {
            self.entity_snapshot.insert(item.id, item.pos);
        }
    }

    /// Set the container contents at `pos`, as loading a structure does.
    pub fn set_inventory(&mut self, pos: Pos, inventory: crate::inventory::Inventory) {
        self.inventories.insert(pos, inventory);
    }

    /// The container contents at `pos`, if any.
    pub fn inventory(&self, pos: Pos) -> Option<&crate::inventory::Inventory> {
        self.inventories.get(&pos)
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
                fluids: &mut self.fluids,
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
                inventories: &mut self.inventories,
                hopper_state: &mut self.hopper_state,
                item_entities: &mut self.item_entities,
                inv_log: self.inv_log.as_mut(),
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
    ///
    /// # Quiet loading
    ///
    /// Vanilla also supports placement **without** the update pass
    /// (`StructurePlaceSettings.knownShape`), and a capture shows it dispatches
    /// *nothing* — no observer pulses, and even a quasi-connected piston stays
    /// put until something else touches it. The engine equivalent is simply not
    /// calling this method: load, and go. That is the mode a timing product
    /// wants — instantiate a contraption at rest, then actuate and measure —
    /// and `manual_engine_quiet_click.json` pins it: a quietly placed engine
    /// sits completely still until its note block is clicked.
    pub fn settle(&mut self) {
        let occupied: Vec<Pos> = self
            .world
            .iter_non_air()
            .map(|(pos, _)| pos)
            .collect();
        // The updates queue is a stack, so blocks are pushed in *reverse*: the
        // dispatch order then matches vanilla's placement pass, which walks the
        // block list in order. The order is observable — two pistons whose
        // triggers race queue their block events by it, and the first event to
        // run moves the other piston's blocks out from under its event
        // (captured: `flying_machine.json`, where the west observer's piston
        // wins and the east one's event finds only a placeholder).
        for pos in occupied.into_iter().rev() {
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
        // Breaking a container takes its contents with it.
        self.inventories.remove(&pos);
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
                fluids: &mut self.fluids,
            events: &mut self.events,
            states: &self.registry,
            tick: self.tick,
            boundary: true,
            updates: &mut self.updates,
            moves: &mut self.moves,
            toggles: &mut self.toggles,
            comparator_out: &mut self.comparator_out,
                inventories: &mut self.inventories,
                hopper_state: &mut self.hopper_state,
                item_entities: &mut self.item_entities,
                inv_log: self.inv_log.as_mut(),
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
                fluids: &mut self.fluids,
                        events: &mut self.events,
                        states: &self.registry,
                        tick: self.tick,
            boundary: false,
                        updates: &mut self.updates,
                moves: &mut self.moves,
                        toggles: &mut self.toggles,
                        comparator_out: &mut self.comparator_out,
                inventories: &mut self.inventories,
                hopper_state: &mut self.hopper_state,
                item_entities: &mut self.item_entities,
                inv_log: self.inv_log.as_mut(),
                        log: self.log.as_mut(),
                    };
                    behaviour.on_scheduled_tick(&mut ctx, entry.pos);
                    self.propagate();
                }
                None
            }

            Phase::FluidTicks => {
                // Identical mechanics to the block-ticks phase, from vanilla's
                // separate fluid queue — `LevelTicks<Fluid>` drains right after
                // `LevelTicks<Block>` and water spread happens here.
                let due = self.fluids.drain_due(self.tick);
                for entry in due {
                    let state = self.world.get(entry.pos);
                    let Some(behaviour) = self.behaviours.get(state) else {
                        self.unknown_seen.push(state);
                        continue;
                    };
                    let mut ctx = TickCtx {
                        world: &mut self.world,
                        ticks: &mut self.ticks,
                        fluids: &mut self.fluids,
                        events: &mut self.events,
                        states: &self.registry,
                        tick: self.tick,
                        boundary: false,
                        updates: &mut self.updates,
                        moves: &mut self.moves,
                        toggles: &mut self.toggles,
                        comparator_out: &mut self.comparator_out,
                        inventories: &mut self.inventories,
                        hopper_state: &mut self.hopper_state,
                        item_entities: &mut self.item_entities,
                        inv_log: self.inv_log.as_mut(),
                        log: self.log.as_mut(),
                    };
                    behaviour.on_fluid_tick(&mut ctx, entry.pos);
                    self.propagate();
                }
                None
            }

            Phase::BlockEvents => self.run_block_events(),

            Phase::BlockEntities => {
                // Ticking block entities first: they were registered at load
                // time, so they precede any moving-piston block entity in
                // vanilla's insertion-ordered list.
                for index in 0..self.tickers.len() {
                    let pos = self.tickers[index];
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
                fluids: &mut self.fluids,
                        events: &mut self.events,
                        states: &self.registry,
                        tick: self.tick,
                        boundary: false,
                        updates: &mut self.updates,
                        moves: &mut self.moves,
                        toggles: &mut self.toggles,
                        comparator_out: &mut self.comparator_out,
                        inventories: &mut self.inventories,
                        hopper_state: &mut self.hopper_state,
                item_entities: &mut self.item_entities,
                        inv_log: self.inv_log.as_mut(),
                        log: self.log.as_mut(),
                    };
                    behaviour.on_block_entity_tick(&mut ctx, pos);
                    self.propagate();
                }
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
                let mut landed: Vec<Pos> = Vec::new();
                for entry in due {
                    let previous = self.world.get(entry.pos);
                    if previous == entry.state {
                        continue;
                    }
                    landed.push(entry.pos);
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
                // `onPlace` for each landed block, *after* the landing updates
                // have run. Ordering matters: vanilla's shape update reaches a
                // moved observer while it still carries its mid-pulse powered
                // state (so it does not re-schedule), and only then does
                // onPlace clear that flag. Dispatching on_placed first would
                // let the self-update see an unpowered observer and start a
                // pulse vanilla never starts.
                for pos in landed {
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
                fluids: &mut self.fluids,
                        events: &mut self.events,
                        states: &self.registry,
                        tick: self.tick,
                        boundary: false,
                        updates: &mut self.updates,
                        moves: &mut self.moves,
                        toggles: &mut self.toggles,
                        comparator_out: &mut self.comparator_out,
                inventories: &mut self.inventories,
                hopper_state: &mut self.hopper_state,
                item_entities: &mut self.item_entities,
                inv_log: self.inv_log.as_mut(),
                        log: self.log.as_mut(),
                    };
                    behaviour.on_placed(&mut ctx, pos);
                    self.propagate();
                }
                None
            }

            Phase::Entities => {
                // Item entities tick in spawn order — vanilla's entity list
                // order. Merging runs at the captured intervals: every 2 ticks
                // while crossing block boundaries, every 40 at rest.
                let collision = SimCollision {
                    world: &self.world,
                    solidity: &self.solidity,
                    frictions: &self.frictions,
                    heights: &self.heights,
                    webs: &self.webs,
                    water_kinds: &self.water_kinds,
                    bubble_kinds: &self.bubble_kinds,
                };
                for index in 0..self.item_entities.items.len() {
                    if self.item_entities.items[index].removed {
                        continue;
                    }
                    let before = self.item_entities.items[index].pos;
                    let alive =
                        crate::entity::tick_item(&mut self.item_entities.items[index], &collision);
                    if !alive {
                        continue;
                    }
                    let after = self.item_entities.items[index].pos;
                    let crossed = before[0].floor() != after[0].floor()
                        || before[1].floor() != after[1].floor()
                        || before[2].floor() != after[2].floor();
                    let interval = if crossed { 2 } else { 40 };
                    if self.item_entities.items[index].tick_count % interval == 0 {
                        crate::entity::merge_neighbours(&mut self.item_entities, index);
                    }
                }
                // entityInside: every cell an item overlaps hears about it —
                // how a wooden pressure plate notices the item on it.
                let mut cells: Vec<Pos> = Vec::new();
                for item in &self.item_entities.items {
                    if item.removed {
                        continue;
                    }
                    let (emin, emax) = crate::entity::item_aabb(item.pos);
                    for x in (emin[0].floor() as i32)..=(emax[0].floor() as i32) {
                        for y in (emin[1].floor() as i32)..=(emax[1].floor() as i32) {
                            for z in (emin[2].floor() as i32)..=(emax[2].floor() as i32) {
                                let cell = Pos::new(x, y, z);
                                if !cells.contains(&cell) {
                                    cells.push(cell);
                                }
                            }
                        }
                    }
                }
                for cell in cells {
                    let state = self.world.get(cell);
                    if state == StateId::AIR {
                        continue;
                    }
                    let Some(behaviour) = self.behaviours.get(state) else { continue };
                    let mut ctx = TickCtx {
                        world: &mut self.world,
                        ticks: &mut self.ticks,
                fluids: &mut self.fluids,
                        events: &mut self.events,
                        states: &self.registry,
                        tick: self.tick,
                        boundary: false,
                        updates: &mut self.updates,
                        moves: &mut self.moves,
                        toggles: &mut self.toggles,
                        comparator_out: &mut self.comparator_out,
                        inventories: &mut self.inventories,
                        hopper_state: &mut self.hopper_state,
                        item_entities: &mut self.item_entities,
                        inv_log: self.inv_log.as_mut(),
                        log: self.log.as_mut(),
                    };
                    behaviour.on_entity_inside(&mut ctx, cell);
                    self.propagate();
                }
                None
            }

            Phase::PlayerInputs => None,
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
                fluids: &mut self.fluids,
                    events: &mut self.events,
                    states: &self.registry,
                    tick: self.tick,
            boundary: false,
                    updates: &mut self.updates,
                moves: &mut self.moves,
                        toggles: &mut self.toggles,
                        comparator_out: &mut self.comparator_out,
                inventories: &mut self.inventories,
                hopper_state: &mut self.hopper_state,
                item_entities: &mut self.item_entities,
                inv_log: self.inv_log.as_mut(),
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
    fn checkpoints_carry_inventories() {
        // Container contents are mutable simulation state — a checkpoint that
        // dropped them would restore a world whose comparators read differently.
        let mut s = sim();
        let pos = Pos::new(2, 2, 2);
        s.set_inventory(pos, crate::inventory::Inventory::empty(27));
        let saved = s.checkpoint();

        s.set_inventory(
            pos,
            crate::inventory::Inventory {
                slots: 27,
                stacks: vec![crate::inventory::ItemStack {
                    slot: 0,
                    id: "minecraft:redstone".to_string(),
                    count: 64,
                }],
            },
        );
        assert_eq!(s.inventory(pos).unwrap().analog_signal(), 1);

        s.restore(&saved);
        assert_eq!(
            s.inventory(pos).unwrap().analog_signal(),
            0,
            "restore must bring the container contents back"
        );
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
