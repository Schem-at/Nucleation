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

/// Whether a state descriptor names a comparator.
fn is_comparator(descriptor: &str) -> bool {
    descriptor.starts_with("minecraft:comparator")
}

use crate::pos::{Bounds, Dir, Pos};
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
    rails: &'a [Option<crate::minecart::Rail>],
    conductors: &'a [bool],
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
    fn rail(&self, pos: Pos) -> Option<crate::minecart::Rail> {
        let state = self.world.get(pos);
        self.rails.get(state.raw() as usize).copied().flatten()
    }
    fn is_conductor(&self, pos: Pos) -> bool {
        let state = self.world.get(pos);
        self.conductors.get(state.raw() as usize).copied().unwrap_or(false)
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

/// One block a piston is currently carrying between two cells.
///
/// The presentable view of a [`PendingMove`] that has a
/// [`Sweep`](crate::piston::Sweep): where it came from, what it looks like, and
/// the tick window it occupies. See [`Simulation::moving_blocks`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MovingBlock {
    /// The cell the block is travelling toward, and where it will be written.
    pub to: Pos,
    /// The cell it left, one step back along [`travel`](Self::travel).
    ///
    /// Given rather than left to the caller: a piston move is exactly one
    /// block, but "exactly one block, backwards along the sweep" is the sort of
    /// rule five bindings would each restate and one of them would get wrong.
    pub from: Pos,
    /// The state that lands at [`to`](Self::to) — the block being carried, not
    /// the `moving_piston` placeholder standing in for it.
    pub state: StateId,
    /// The state to *draw* travelling, which is [`state`](Self::state) for
    /// everything except a retracting piston's own square: there the body
    /// stays where it is and a `piston_head` comes home, so what lands and
    /// what moves are different blocks. See
    /// [`PendingMove::carried`](crate::behaviour::PendingMove::carried).
    ///
    pub carried: StateId,
    /// [`carried`](Self::carried) with a piston arm's `short=true`, or `None`
    /// if it is not an arm.
    ///
    /// The game draws a moving head **shortened while it is within half a
    /// block of its body** and full length beyond that — otherwise the shaft
    /// visibly passes through the back of the piston as it comes home.
    /// `PistonHeadRenderer` writes that as `SHORT = progress <= 0.5` when
    /// extending and `SHORT = progress >= 0.5` when retracting: one rule read
    /// from each end, since an extension travels away from the base and a
    /// retraction back to it. Which form to use is the mover's call — it holds
    /// the sub-tick progress — but the state is the engine's to name.
    pub carried_short: Option<StateId>,
    /// A state occupying [`to`](Self::to) for the duration of the move — the
    /// retracting piston's body, and `None` for every other move, whose
    /// destination is genuinely empty until it arrives.
    pub remains: Option<StateId>,
    /// The direction of travel.
    pub travel: Dir,
    /// Whether the piston is pushing (`true`) or pulling back (`false`).
    pub extending: bool,
    /// The tick the stroke was dispatched on.
    pub started_on: u64,
    /// The tick the block is written at [`to`](Self::to).
    ///
    /// A consumer animating the move should retire it here and not on a
    /// wall-clock timer: the landing is the simulation's decision, and any
    /// other clock will eventually disagree with it.
    pub lands_on: u64,
    /// Whether this is the piston's own square — the head sliding out of a base
    /// that stays put, rather than a block being carried.
    pub source_piston: bool,
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
    command_powered: std::collections::HashMap<Pos, bool>,
    item_entities: crate::entity::ItemEntities,
    minecarts: Vec<crate::minecart::MinecartState>,
    tnts: Vec<crate::explosion::PrimedTnt>,
    bodies: Vec<crate::entity::EntityInstance>,
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
    /// Entries queued during the current dispatch (`addedThisLayer`).
    updates: Vec<crate::behaviour::UpdateEntry>,
    /// The collector's stack of in-flight entries (`CollectingNeighborUpdater`).
    pending: Vec<crate::behaviour::UpdateEntry>,
    /// Deferred writes awaiting their block-entities phase.
    moves: Vec<PendingMove>,
    /// Torch toggle history for burnout, pruned as it ages out.
    toggles: Vec<(Pos, u64)>,
    /// Stored comparator output strengths; vanilla's ComparatorBlockEntity.
    comparator_out: std::collections::HashMap<Pos, u8>,
    /// Container contents by position; vanilla's inventory block entities.
    inventories: std::collections::HashMap<Pos, crate::inventory::Inventory>,
    /// Every position holding a block entity, contents modelled or not.
    block_entities: std::collections::HashSet<Pos>,
    /// Per-hopper cooldown and tick bookkeeping.
    hopper_state: std::collections::HashMap<Pos, crate::behaviour::HopperState>,
    /// Command-block programs by position — see [`Simulation::set_command`].
    commands: std::collections::HashMap<Pos, crate::behaviour::CommandProgram>,
    /// Command blocks' last-seen powered flags, for rising-edge detection.
    command_powered: std::collections::HashMap<Pos, bool>,
    /// `randomTickSpeed`: attempts per 4096-block volume per tick; 0 = off.
    random_tick_speed: u8,
    /// The random source feeding random-tick position picks. Reset by
    /// [`Simulation::set_rng_seed`] so a seeded run replays exactly.
    random_rng: crate::rng::JavaRandom,
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
    lava_kinds: Vec<Option<crate::fluid::WaterKind>>,
    /// Bubble columns per state (`Some(drag_down)`), indexed by `StateId`.
    bubble_kinds: Vec<Option<bool>>,
    /// Rails per state, indexed by `StateId`; see [`Simulation::set_rail_tables`].
    rails: Vec<Option<crate::minecart::Rail>>,
    /// Redstone conductivity per state, for the powered-rail launch check.
    conductors: Vec<bool>,
    /// The world's minecarts, in spawn order.
    minecarts: Vec<crate::minecart::MinecartState>,
    tnts: Vec<crate::explosion::PrimedTnt>,
    /// Every entity that is a hitbox and nothing else, in spawn order — the
    /// record doors' frozen fireballs, their villagers and boats, and the
    /// passengers riding their nan carts.
    ///
    /// One collection, not two: a standing body and a passenger differ in how
    /// they come by a position ([`crate::entity::BodyPhysics`]) and in nothing
    /// else, and splitting them into two `Vec`s of differently-shaped tuples
    /// meant every question — what is its box, does it stop a cart, does a
    /// piston shove it — had to be asked twice.
    ///
    /// Wherever order is observable the two are still walked in two passes,
    /// standing bodies before passengers, because that is the order the
    /// entity-box view was built in and a trace reads the same way.
    bodies: Vec<crate::entity::EntityInstance>,
    /// Which version's `Entity.load` Motion handling the authored spawns run
    /// under. See [`crate::motion`] — it decides whether a nan cart exists.
    motion_semantics: crate::motion::MotionSemantics,
    /// Entities whose box entered a *retracting* piston's sweep.
    ///
    /// `(tick, entity id)`, appended and never cleared within a run. A tripwire
    /// from when retraction was unmodelled: extension displacement was measured
    /// and implemented ([`piston_entity.json`]) while `piston_pull.json`'s
    /// sub-0.03 displacements, not uniformly backwards, had no model here.
    /// All three retraction geometries are measured and implemented now — see
    /// `Simulation::piston_push` — so nothing appends to this and it stays
    /// empty, including on the record 3x3 door, which used to name six. Kept
    /// rather than deleted because the next geometry that turns out not to be
    /// covered should be reported the same way, not guessed at: a non-zero
    /// value means a run leaned on behaviour we do not reproduce.
    ///
    /// [`piston_entity.json`]: crate::sim::Simulation::piston_retract_contacts
    retract_contacts: Vec<(u64, u32)>,
    /// Boxes a moving piston carried an entity *through* this tick, before the
    /// step's own correction and before collision clipped it: `(id, min, max)`.
    ///
    /// A piston shove is not a jump from one settled box to another. Vanilla
    /// calls `entity.move(MoverType.PISTON, …)` with the whole step, then pushes
    /// the entity back out of the piston's block — and `entityInside` fires at
    /// the far position, before the correction. That intermediate box is
    /// invisible in an entity log, which only prints settled positions, but it
    /// is perfectly visible in a **pressure plate's block state**, and the
    /// record 3x3 door is built on exactly that: `piston_head_transient` and
    /// `piston_plate_clip` power a light weighted plate on the retraction tick
    /// while the fireball's settled box never comes within 0.08 of the plate's
    /// touch box, and `plate_reach_flush` proves the same fireball parked there
    /// never presses it.
    ///
    /// Filled by [`Simulation::displace_entities_by_pistons`] and consumed in
    /// the same call, so it never outlives the tick that made it.
    piston_probes: Vec<(u32, [f64; 3], [f64; 3])>,
    /// `Entity.pistonDeltas`: how far each entity has already been carried by
    /// pistons this tick, per axis.
    ///
    /// `Entity.limitPistonMovement` clamps the *running total* of a tick's
    /// piston displacements to ±[`crate::piston::PISTON_MAX_STEP`] per axis and
    /// zeroes a residual under 1e-5, so two strokes shoving one entity in the
    /// same tick compose through this and not by addition — the "combination of
    /// two laws" the record door's captures kept being 0.02 away from. Cleared
    /// at the top of [`Simulation::displace_entities_by_pistons`], which is the
    /// once-a-tick that `pistonDeltasGameTime` implements in vanilla.
    piston_deltas: std::collections::HashMap<u32, [f64; 3]>,
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
    /// Recorded update deliveries, when update recording is enabled.
    ///
    /// Separate from [`Simulation::record`] because it is far larger — several
    /// updates per block change — and only a propagation view wants it.
    upd_log: Option<Vec<crate::behaviour::UpdateRecord>>,
    /// The log kept after recording was switched off, so that stopping the
    /// recorder does not destroy what it recorded. `None` while recording.
    upd_saved: Option<Vec<crate::behaviour::UpdateRecord>>,
    /// Rich run recording, independently opt-in from the conformance log.
    timeline: Option<crate::timeline::TimelineRecorder>,
    /// The phase currently executing, `None` between ticks.
    phase: Option<Phase>,
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

/// Whether a saved body is the one nonzero-motion frozen pose we can identify
/// without pretending to have general boat physics.
///
/// `boat_fence.litematic` is a boat hanging almost vertically from a fence
/// knot. Vanilla saves its tether-rest correction as
/// `(roundoff, 0.011681267954871115, roundoff)` even though its authored pose is
/// the stable collision geometry the build needs. The leash tag proves this is
/// that bounded state rather than a free boat. The envelope is intentionally
/// one-sided and tight: no horizontal travel beyond floating-point dust, and
/// less than 1/64 block/tick upward. A falling, rowing or swinging boat still
/// refuses until real boat/leash physics exists.
fn supported_parked_boat(body: &crate::structure::SpawnedBody, motion: [f64; 3]) -> bool {
    const HORIZONTAL_DUST: f64 = 2.0 * f64::EPSILON;
    const MAX_TETHER_REST_RISE: f64 = 1.0 / 64.0;

    if body.kind == "minecraft:oak_boat" {
        return body.leashed
            && motion[0].abs() <= HORIZONTAL_DUST
            && (0.0..=MAX_TETHER_REST_RISE).contains(&motion[1])
            && motion[2].abs() <= HORIZONTAL_DUST;
    }

    // `Elevator Decorated.litematic` contains 24 vertical chains of eight pale
    // oak boats. Each carries one silverfish; seven boats in each chain are
    // leashed to the next boat and the root is not. Vanilla 26.2 measures the
    // silverfish seat at [0, 0.1875, 0]. The settled save's tether corrections
    // are horizontal only and bounded below 1/4096 block/tick; every free root
    // contains floating-point dust only. This identifies that authored rest
    // network without accepting a free rowing, falling or drifting boat.
    let silverfish_rider = matches!(
        body.passengers.as_slice(),
        [crate::structure::SpawnedEntity::Body(rider)]
            if rider.kind == "minecraft:silverfish"
    );
    let tether_residue = body.leashed
        && motion[0].abs() <= 1.0 / 4096.0
        && motion[1] == 0.0
        && motion[2].abs() <= 1.0 / 4096.0;
    let free_root_at_rest = !body.leashed
        && motion[0].abs() <= HORIZONTAL_DUST
        && motion[1].abs() <= HORIZONTAL_DUST
        && motion[2].abs() <= HORIZONTAL_DUST;

    body.kind == "minecraft:pale_oak_boat"
        && silverfish_rider
        && (tether_residue || free_root_at_rest)
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
            command_powered: std::collections::HashMap::new(),
            item_entities: crate::entity::ItemEntities::default(),
            minecarts: Vec::new(),
            tnts: Vec::new(),
            bodies: Vec::new(),
        };
        Self {
            block_entities: std::collections::HashSet::new(),
            registry: StateRegistry::new(),
            world,
            ticks: TickQueue::new(),
            fluids: TickQueue::new(),
            events: EventQueue::new(),
            behaviours: BehaviourTable::new(),
            unknown_seen: Vec::new(),
            updates: Vec::new(),
            pending: Vec::new(),
            moves: Vec::new(),
            toggles: Vec::new(),
            comparator_out: std::collections::HashMap::new(),
            inventories: std::collections::HashMap::new(),
            hopper_state: std::collections::HashMap::new(),
            commands: std::collections::HashMap::new(),
            command_powered: std::collections::HashMap::new(),
            random_tick_speed: 0,
            random_rng: crate::rng::JavaRandom::new(0),
            item_entities: crate::entity::ItemEntities::default(),
            solidity: Vec::new(),
            frictions: Vec::new(),
            heights: Vec::new(),
            webs: Vec::new(),
            water_kinds: Vec::new(),
            lava_kinds: Vec::new(),
            bubble_kinds: Vec::new(),
            rails: Vec::new(),
            conductors: Vec::new(),
            minecarts: Vec::new(),
            tnts: Vec::new(),
            bodies: Vec::new(),
            motion_semantics: crate::motion::MotionSemantics::default(),
            retract_contacts: Vec::new(),
            piston_probes: Vec::new(),
            piston_deltas: std::collections::HashMap::new(),
            entity_snapshot: std::collections::HashMap::new(),
            ent_log: None,
            tickers: Vec::new(),
            log: None,
            inv_log: None,
            upd_log: None,
            upd_saved: None,
            timeline: None,
            phase: None,
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
        self.timeline = None;
        self.log = Some(Vec::new());
        self.inv_log = Some(Vec::new());
        self.ent_log = Some(Vec::new());
        self.entity_snapshot.clear();
        for item in &self.item_entities.items {
            self.entity_snapshot.insert(item.id, item.pos);
        }
        for cart in &self.minecarts {
            self.entity_snapshot.insert(cart.id, cart.pos);
        }
    }

    /// Start a rich run timeline, including block deltas, input markers,
    /// piston strokes, and a canonical visible-world fingerprint per tick.
    ///
    /// This is separate from [`Simulation::record`] because state snapshots
    /// deliberately cost work. With timeline recording off the tick loop only
    /// pays one predictable `Option` check at the tick boundary.
    pub fn record_timeline(&mut self) {
        self.record();
        self.timeline = Some(crate::timeline::TimelineRecorder::new(
            self.tick,
            &self.world,
            &self.registry,
        ));
    }

    /// Clone the timeline recorded since [`Simulation::record_timeline`].
    pub fn recorded_timeline(&self) -> Option<crate::timeline::RunTimeline> {
        self.timeline
            .as_ref()
            .map(|timeline| timeline.finish(self.log.as_deref().unwrap_or(&[]), self.tick))
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
    /// Container contents carried by an item entity — a dropped shulker box's
    /// slots. Empty for ordinary items.
    pub fn item_contents(&self, id: u32) -> Option<&[crate::inventory::ItemStack]> {
        self.item_entities.contents.get(&id).map(Vec::as_slice)
    }

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

    /// Set the rail and conductor tables, indexed by `StateId`.
    ///
    /// Built by `vanilla::rail_tables`. Cart physics reads these.
    /// The lava-per-state table — [`crate::vanilla::lava_table`]. Optional:
    /// a wiring recipe that never simulates an entity in lava can skip it.
    pub fn set_lava_table(&mut self, lava_kinds: Vec<Option<crate::fluid::WaterKind>>) {
        self.lava_kinds = lava_kinds;
    }

    pub fn set_rail_tables(
        &mut self,
        rails: Vec<Option<crate::minecart::Rail>>,
        conductors: Vec<bool>,
    ) {
        self.rails = rails;
        self.conductors = conductors;
    }

    /// Spawn a minecart. Ids come from the shared entity counter, so carts
    /// and items interleave exactly as a placed structure's entity list does.
    /// Seed the vanilla random source (`LegacyRandomSource`'s LCG).
    ///
    /// Behaviours that jitter — dispense trajectories, dispenser slot choice,
    /// destroy drops — draw from it in a fixed order, so a seeded run is
    /// exactly reproducible. Without a seed the engine uses each
    /// distribution's mean, which is what the conformance goldens compare
    /// against. Matching a live server draw-for-draw is out of scope: a real
    /// `ServerLevel.random` is shared with the whole world.
    pub fn set_rng_seed(&mut self, seed: i64) {
        self.item_entities.rng = Some(crate::rng::JavaRandom::new(seed));
        self.random_rng = crate::rng::JavaRandom::new(seed);
    }

    /// `randomTickSpeed`: how many random-tick attempts each 4096-block
    /// volume gets per tick (vanilla's default is 3; gametest environments
    /// set their own). Zero — the default — disables the pass entirely, so
    /// existing corpora never pay for it.
    ///
    /// Position selection is an approximation: vanilla draws per 16³ chunk
    /// section with one packed `nextInt`, this draws three bounded ints
    /// uniformly over the world box. The *rate* matches; the exact cells and
    /// draw order do not, so a seeded run is reproducible against itself but
    /// not against a vanilla capture.
    pub fn set_random_ticks(&mut self, speed: u8) {
        self.random_tick_speed = speed;
    }

    pub fn spawn_minecart(&mut self, kind: String, pos: [f64; 3], vel: [f64; 3]) -> u32 {
        let id = self.item_entities.next_id;
        self.item_entities.next_id += 1;
        self.push_minecart(id, kind, pos, vel);
        id
    }

    /// Spawn a minecart with the id a capture recorded for it.
    pub fn spawn_minecart_with_id(
        &mut self,
        id: u32,
        kind: String,
        pos: [f64; 3],
        vel: [f64; 3],
    ) -> u32 {
        self.item_entities.next_id = self.item_entities.next_id.max(id + 1);
        self.push_minecart(id, kind, pos, vel);
        id
    }

    /// Spawn an authored minecart, carrying everything the structure said about
    /// it — including its yaw, which the three-argument spawns cannot express
    /// and which decides whether the cart can be pushed at all.
    ///
    /// Takes the parsed entity rather than loose arguments so that a new field
    /// on [`crate::structure::SpawnedMinecart`] reaches the simulation by being
    /// read here, instead of by being silently dropped at a call site that was
    /// never updated. `id` is the id a capture recorded, when there is one.
    pub fn spawn_authored_minecart(
        &mut self,
        cart: &crate::structure::SpawnedMinecart,
        id: Option<u32>,
    ) -> u32 {
        let id = match id {
            Some(id) => {
                self.item_entities.next_id = self.item_entities.next_id.max(id + 1);
                id
            }
            None => {
                let id = self.item_entities.next_id;
                self.item_entities.next_id += 1;
                id
            }
        };
        let motion = self.motion_semantics.load_motion(cart.motion, [0.0; 3]);
        self.push_minecart(id, cart.kind.clone(), cart.pos, motion);
        if let Some(spawned) = self.minecarts.last_mut() {
            spawned.yaw = cart.yaw;
            if !cart.items.is_empty() {
                let inv = spawned.inventory.as_mut().unwrap_or_else(|| {
                    panic!(
                        "a {} at {:?} carries Items, but that cart kind has no \
                         container — the save is corrupt or the kind is unmodelled",
                        cart.kind, cart.pos
                    )
                });
                inv.stacks = cart.items.clone();
            }
        }
        id
    }

    /// Which version's `Entity.load` Motion handling authored spawns use.
    ///
    /// Set from the save's own DataVersion. Whoever loads a world knows it;
    /// the engine cannot, and guessing would decide whether a nan cart exists.
    pub fn set_motion_semantics(&mut self, semantics: crate::motion::MotionSemantics) {
        self.motion_semantics = semantics;
    }

    /// The Motion-load path this simulation is running under.
    ///
    /// Readable so that a door which only works under one of them cannot look
    /// identical to a door that works under both.
    pub fn motion_semantics(&self) -> crate::motion::MotionSemantics {
        self.motion_semantics
    }

    /// Entities that touched a **retracting** piston's sweep, as
    /// `(tick, entity id)`.
    ///
    /// Non-empty means the run depended on behaviour this engine does not
    /// model — see the field docs on `retract_contacts` and
    /// `tools/gametest/captures/piston_pull.entities.log`. Empty means no
    /// entity was ever in a retracting arm's way, which is a real answer
    /// rather than an absence of instrumentation.
    pub fn piston_retract_contacts(&self) -> &[(u64, u32)] {
        &self.retract_contacts
    }

    /// Spawn an authored frozen body: a hitbox, and nothing more.
    ///
    /// One method for fireballs, villagers, blazes, boats and armor stands,
    /// because the engine does exactly one thing with each of them and the
    /// refusal below is exactly the same sentence about each of them. What
    /// differs between them — box, solidity to a cart, seats — is entirely in
    /// [`crate::vanilla::entity_table`].
    ///
    /// Refuses one that is actually *moving*, with one measured exception: a
    /// leashed oak boat at tether rest may carry a sub-1/64 upward correction
    /// and floating-point dust horizontally. `boat_fence.litematic` is that
    /// state, and its authored pose is kept as the stationary hard collider.
    /// Free, falling, rowing and swinging boats still refuse.
    ///
    /// Otherwise the bar is lower than it looks. The record doors' fireballs
    /// are caught mid-flight by a
    /// piston-and-cobweb trick and have zero motion, so modelling them as a
    /// stationary box is what the game does with them — but a fireball with real
    /// velocity is a projectile and there is no projectile physics here; a
    /// villager is a wall and a floor, and one with velocity is *doing*
    /// something, with no AI, pathfinding or gravity here to do it with; `noai`
    /// is what holds a blaze still in `blaze_ride.entities.log`, and a blaze with
    /// AI hovers, strafes and shoots. In every case a stationary box would be a
    /// wrong answer that looks like a right one, so it refuses by name instead.
    ///
    /// The record 3x3 door has no free-standing blaze — both of its blazes are
    /// passengers, and go through [`Simulation::spawn_authored_rider`].
    pub fn spawn_authored_body(
        &mut self,
        body: &crate::structure::SpawnedBody,
    ) -> Result<u32, String> {
        let motion = self.motion_semantics.load_motion(body.motion, [0.0; 3]);
        if motion.iter().any(|v| *v != 0.0) && !supported_parked_boat(body, motion) {
            return Err(format!(
                "{} at {:?} has Motion {:?}: this engine models it only as a frozen \
                 hitbox, and has no projectile physics, mob AI, flight or gravity to \
                 move one with",
                body.kind, body.pos, motion
            ));
        }
        self.spawn_frozen_entity_leashed(body.kind.clone(), body.pos, body.leashed)
    }

    /// Seat a passenger on a measured vehicle.
    ///
    /// A rider has no physics and no position of its own. Vanilla's
    /// `Entity.rideTick` zeroes the passenger's velocity, ticks it, and then the
    /// vehicle's `positionRider` **hard-sets** the passenger to
    /// `vehicle.position() + seat` with no collision check — so the rider's
    /// state is exactly `(which vehicle, which seat)`, and that is all this
    /// stores. `refresh_bodies` re-derives the box from the vehicle whenever
    /// anything moves.
    ///
    /// Measured over twenty ticks in `blaze_ride.entities.log`, four ways:
    /// a cart at rest holds its rider at a fixed offset; a cart rolling east
    /// carries the rider's x with it to the last digit; a cart falling through
    /// air carries the rider down and lands it; and a **NaN** cart — whose own
    /// physics are dead — pins its rider forever, which is what the record door
    /// is built on. `blaze_ride_ai.entities.log` repeats the last two with AI
    /// enabled and gets the same positions, with the rider's velocity reading
    /// `(0, -0.0784000015258789, 0)` on every tick and never moving it: the
    /// velocity is real, it is one step of gravity, and it is overwritten before
    /// it can do anything. The record door's saved riders carry that exact
    /// number.
    ///
    /// The seat comes from [`crate::entity::passenger_attachment`], which
    /// refuses any vehicle/rider pair it has not measured. There is no fallback:
    /// a guessed seat puts a real hitbox in the wrong place.
    pub fn spawn_authored_rider(
        &mut self,
        vehicle_id: u32,
        rider: &crate::structure::SpawnedEntity,
    ) -> Result<u32, String> {
        let vehicle_kind = self
            .minecarts
            .iter()
            .find(|cart| cart.id == vehicle_id && !cart.removed)
            .map(|cart| cart.kind.clone())
            .or_else(|| {
                self.bodies
                    .iter()
                    .find(|body| body.id == vehicle_id)
                    .map(|body| body.kind.clone())
            });
        let Some(vehicle_kind) = vehicle_kind else {
            return Err(format!(
                "passenger seated on missing entity {vehicle_id}: its vehicle did not \
                 spawn, so its seat cannot be derived"
            ));
        };
        let rider_kind = rider.kind().to_string();
        let Some(seat) = crate::entity::passenger_attachment(&vehicle_kind, &rider_kind) else {
            return Err(format!(
                "{rider_kind} riding {vehicle_kind}: no seat offset has been measured \
                 for that pair, and it cannot be derived from the hitboxes — on one and \
                 the same minecart a blaze sits 0.1875 above it and a villager sits at \
                 0.0. Measure it with tools/gametest/capture.sh --spawn '...:ride' and \
                 add it to the vehicle's `seats` in vanilla::entity_table()."
            ));
        };
        // A rider's own `Motion` is not checked the way a free-standing
        // villager's is. It cannot mean the same thing: `rideTick` overwrites it
        // every tick, so a saved rider's non-zero velocity is a *result* of
        // being carried, not a statement that it is going somewhere.
        if crate::entity::entity_dimensions(&rider_kind).is_none() {
            return Err(format!(
                "unknown passenger {rider_kind}: no hitbox is known for it, and \
                 guessing one would silently change the simulation."
            ));
        }
        let id = self.item_entities.next_id;
        self.item_entities.next_id += 1;
        self.bodies.push(crate::entity::EntityInstance {
            id,
            kind: rider_kind,
            physics: crate::entity::BodyPhysics::Rider { vehicle: vehicle_id, seat },
            // A passenger is held by its seat, not by a rope.
            leashed: false,
        });
        self.refresh_bodies();
        Ok(id)
    }

    /// Spawn an authored furnace minecart.
    ///
    /// A furnace cart is dimensionally an ordinary cart — one hitbox for every
    /// variant, measured — so an *unfuelled* one needs nothing the plain cart
    /// does not already have. All fifteen in the record 3x3 door carry
    /// `Fuel: 0` with `PushX`/`PushZ` zero, which makes them pure mass.
    ///
    /// A fuelled one drives itself, and that is not implemented. It refuses
    /// rather than running as a plain cart, because a cart that should be
    /// accelerating and is not produces a plausible-looking wrong trace.
    ///
    /// `id` is the id a capture recorded, when there is one — same contract as
    /// [`Simulation::spawn_authored_minecart`], so a conformance run can line
    /// its carts up with the golden's.
    pub fn spawn_authored_furnace_minecart(
        &mut self,
        cart: &crate::structure::SpawnedFurnaceMinecart,
        id: Option<u32>,
    ) -> Result<u32, String> {
        if cart.fuel != 0 || cart.push != [0.0, 0.0] {
            return Err(format!(
                "minecraft:furnace_minecart at {:?} has Fuel {} and Push {:?}: \
                 self-propulsion is not implemented, and running a fuelled cart as \
                 an unfuelled one would silently drop its acceleration",
                cart.pos, cart.fuel, cart.push
            ));
        }
        let motion = self.motion_semantics.load_motion(cart.motion, [0.0; 3]);
        let id = match id {
            Some(id) => {
                self.item_entities.next_id = self.item_entities.next_id.max(id + 1);
                id
            }
            None => {
                let id = self.item_entities.next_id;
                self.item_entities.next_id += 1;
                id
            }
        };
        self.push_minecart(id, "minecraft:furnace_minecart".to_string(), cart.pos, motion);
        // Same reason as `spawn_authored_minecart`: yaw is the push gate, and a
        // furnace cart is an `AbstractMinecart` like any other. Leaving it at
        // the spawn default here is what made the record door's top row shove
        // itself apart — see `SpawnedFurnaceMinecart::yaw`.
        if let Some(spawned) = self.minecarts.last_mut() {
            spawned.yaw = cart.yaw;
        }
        Ok(id)
    }

    /// Materialise everything `summon` queued this tick.
    fn drain_pending_spawns(&mut self) {
        let pending: Vec<crate::entity::PendingSpawn> =
            self.item_entities.pending_spawns.drain(..).collect();
        for spawn in pending {
            match spawn {
                crate::entity::PendingSpawn::Tnt { pos, fuse } => {
                    let id = self.item_entities.next_id;
                    self.item_entities.next_id += 1;
                    self.tnts.push(crate::explosion::PrimedTnt {
                        id,
                        pos,
                        vel: [0.0; 3],
                        fuse,
                        removed: false,
                    });
                }
                crate::entity::PendingSpawn::Minecart { kind, pos } => {
                    self.spawn_minecart(kind.to_string(), pos, [0.0; 3]);
                }
                crate::entity::PendingSpawn::Body { kind, pos } => {
                    let Some((_, height)) = crate::entity::entity_dimensions(kind) else {
                        panic!(
                            "summon {kind} at {pos:?}: this engine has no measured \
                             dimensions for that kind — add its EntityType row first"
                        );
                    };
                    let _ = height;
                    let Some(hp) = crate::entity::mob_health(kind) else {
                        panic!(
                            "summon {kind} at {pos:?}: no measured max health — a \
                             body an explosion can hit needs one to die honestly"
                        );
                    };
                    let id = self.item_entities.next_id;
                    self.item_entities.next_id += 1;
                    self.bodies.push(crate::entity::EntityInstance {
                        id,
                        kind: kind.to_string(),
                        physics: crate::entity::BodyPhysics::Ballistic {
                            pos,
                            vel: [0.0; 3],
                            on_ground: false,
                            hp,
                        },
                        // `/summon` has no leash argument.
                        leashed: false,
                    });
                }
            }
        }
    }

    /// Tick primed TNT and ballistic bodies — the entity pass between carts
    /// and items. Fuses that run out explode *after* the loop, in trigger
    /// order: a chain where one explosion must be visible to a same-tick
    /// later entity is unmeasured.
    fn tick_tnt_and_bodies(&mut self) {
        let mut explosions: Vec<([f64; 3], f32, bool)> = Vec::new();
        {
            let collision = SimCollision {
                world: &self.world,
                solidity: &self.solidity,
                frictions: &self.frictions,
                heights: &self.heights,
                webs: &self.webs,
                water_kinds: &self.water_kinds,
                bubble_kinds: &self.bubble_kinds,
                rails: &self.rails,
                conductors: &self.conductors,
            };
            for tnt in &mut self.tnts {
                if tnt.removed {
                    continue;
                }
                if crate::explosion::tick_tnt(tnt, &collision) {
                    tnt.removed = true;
                    // `PrimedTnt.explode`: centred at `y + height * 0.0625`.
                    explosions.push((
                        [tnt.pos[0], tnt.pos[1] + 0.98 * 0.0625, tnt.pos[2]],
                        4.0,
                        false,
                    ));
                }
            }
            // `LivingEntity.travel` with no AI input: move, then
            // `x *= friction`, `y = (y - 0.08) * 0.98`, `z *= friction`,
            // friction 0.91 airborne and `0.6 * 0.91` on ordinary ground. In
            // lava the branch is different: the fluid current pushes first
            // (`updateFluidHeightAndDoFluidPushing`, scale 0.0023333333333333335),
            // then move, then a uniform 0.5 drag and a quarter of gravity.
            for body in &mut self.bodies {
                let kind = body.kind.clone();
                let crate::entity::BodyPhysics::Ballistic { pos, vel, on_ground, .. } =
                    &mut body.physics
                else {
                    continue;
                };
                let Some((min, max)) = crate::entity::body_aabb(&kind, *pos) else {
                    continue;
                };
                // Which lava cells the body stands in, and their net flow.
                let mut flow = [0.0f64; 3];
                let mut in_lava = false;
                for x in (min[0].floor() as i32)..=((max[0] - 1e-9).floor() as i32) {
                    for y in (min[1].floor() as i32)..=((max[1] - 1e-9).floor() as i32) {
                        for z in (min[2].floor() as i32)..=((max[2] - 1e-9).floor() as i32) {
                            let cell = Pos::new(x, y, z);
                            let state = self.world.get(cell);
                            let Some(kind) =
                                self.lava_kinds.get(state.raw() as usize).copied().flatten()
                            else {
                                continue;
                            };
                            in_lava = true;
                            // `FlowingFluid.getFlow`, reduced to the measured
                            // case: amount gradients across the four
                            // horizontals; an empty passable neighbour reads
                            // as amount 0.
                            let own = f64::from(kind.amount());
                            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                                let npos = Pos::new(x + dx, y, z + dz);
                                let nstate = self.world.get(npos);
                                let namount = match self
                                    .lava_kinds
                                    .get(nstate.raw() as usize)
                                    .copied()
                                    .flatten()
                                {
                                    Some(nkind) => f64::from(nkind.amount()),
                                    None if self
                                        .solidity
                                        .get(nstate.raw() as usize)
                                        .copied()
                                        .unwrap_or(false) =>
                                    {
                                        continue; // solid wall: no gradient
                                    }
                                    None => 0.0,
                                };
                                let delta = own - namount;
                                flow[0] += f64::from(dx) * delta;
                                flow[2] += f64::from(dz) * delta;
                            }
                        }
                    }
                }
                if in_lava {
                    let norm =
                        (flow[0] * flow[0] + flow[1] * flow[1] + flow[2] * flow[2]).sqrt();
                    if norm > 0.0 {
                        let scale = 0.002_333_333_333_333_333_5;
                        for axis in 0..3 {
                            vel[axis] += flow[axis] / norm * scale;
                        }
                    }
                }
                let (moved, grounded) = crate::explosion::move_box(&collision, min, max, *vel);
                for axis in 0..3 {
                    pos[axis] += moved[axis];
                    if moved[axis] != vel[axis] {
                        vel[axis] = 0.0;
                    }
                }
                *on_ground = grounded;
                if in_lava {
                    for axis in 0..3 {
                        vel[axis] *= 0.5;
                    }
                    vel[1] -= 0.08 / 4.0;
                } else {
                    let friction = if grounded { 0.6 * 0.91 } else { 0.91 };
                    vel[0] *= friction;
                    vel[1] = (vel[1] - 0.08) * 0.98;
                    vel[2] *= friction;
                }
                if grounded && vel[1] < 0.0 {
                    vel[1] = 0.0;
                }
            }
            // A primed TNT cart counts down here; the activator-rail check
            // that primes it lives in the cart loop's wake, below.
            for cart in &mut self.minecarts {
                if cart.removed || cart.kind != "minecraft:tnt_minecart" {
                    continue;
                }
                let cell = Pos::new(
                    cart.pos[0].floor() as i32,
                    cart.pos[1].floor() as i32,
                    cart.pos[2].floor() as i32,
                );
                let on_rail = [cell, Pos::new(cell.x, cell.y - 1, cell.z)]
                    .into_iter()
                    .find(|p| crate::entity::CollisionWorld::rail(&collision, *p).is_some());
                if cart.fuse.is_none() {
                    if let Some(rail_cell) = on_rail {
                        let descriptor =
                            self.registry.descriptor(self.world.get(rail_cell)).unwrap_or("");
                        // `ActivatorRailBlock` → `activateMinecart(powered)`
                        // → `MinecartTNT.primeFuse`: 80 ticks.
                        if descriptor.starts_with("minecraft:activator_rail")
                            && descriptor.contains("powered=true")
                        {
                            cart.fuse = Some(80);
                        }
                    }
                }
                if let Some(fuse) = cart.fuse.as_mut() {
                    *fuse -= 1;
                    if *fuse <= 0 {
                        cart.removed = true;
                        // `MinecartTNT.explode(speedSq)`: parked, the speed
                        // term is zero and the power is exactly 4.
                        explosions.push(([cart.pos[0], cart.pos[1], cart.pos[2]], 4.0, true));
                    }
                }
            }
        }
        for (center, power, rail_shielded) in explosions {
            self.explode(center, power, rail_shielded);
        }
    }

    /// `ServerExplosion.explode`: destruction rays, entity knockback and
    /// damage, then block removal — in that order, so line-of-sight reads
    /// the pre-blast world exactly as vanilla's does.
    ///
    /// `rail_shielded` carries `MinecartTNT`'s damage calculator: while
    /// primed, a TNT cart's blast reads every rail — and every block
    /// directly *under* a rail — as resistance 0 and indestructible
    /// (`BlockTags.RAILS` in both overrides). A block-drop TNT passes
    /// `false`.
    pub fn explode(&mut self, center: [f64; 3], power: f32, rail_shielded: bool) {
        // Phase A: read-only sweeps.
        let collision = SimCollision {
            world: &self.world,
            solidity: &self.solidity,
            frictions: &self.frictions,
            heights: &self.heights,
            webs: &self.webs,
            water_kinds: &self.water_kinds,
            bubble_kinds: &self.bubble_kinds,
            rails: &self.rails,
            conductors: &self.conductors,
        };
        let registry = &self.registry;
        let world = &self.world;
        let rng = &mut self.item_entities.rng;
        let rails = &self.rails;
        let is_rail = |pos: Pos| {
            rails.get(world.get(pos).raw() as usize).copied().flatten().is_some()
        };
        let shielded = |pos: Pos| {
            rail_shielded && (is_rail(pos) || is_rail(Pos::new(pos.x, pos.y + 1, pos.z)))
        };
        let cleared = crate::explosion::destruction_set(
            center,
            power,
            || match rng.as_mut() {
                Some(rng) => rng.next_float(),
                None => 0.5,
            },
            |pos| {
                let state = world.get(pos);
                if state == StateId::AIR {
                    return None;
                }
                if shielded(pos) {
                    return Some(0.0);
                }
                let descriptor = registry.descriptor(state).unwrap_or("minecraft:air");
                let name = descriptor.split('[').next().unwrap_or(descriptor);
                crate::vanilla::blast_resistance(name)
            },
            |pos| !shielded(pos),
        );
        // Entity sweep: `radius * 2` reach, feet-distance falloff, eye-height
        // direction (feet for another primed TNT), seen-percent scaling.
        let reach = f64::from(power) * 2.0;
        enum Hit {
            Item(usize),
            Cart(usize),
            Tnt(usize),
            Body(usize),
        }
        let mut hits: Vec<(Hit, [f64; 3], f32)> = Vec::new();
        {
            let mut consider =
                |slot: Hit, feet: [f64; 3], eye_y: f64, bb: ([f64; 3], [f64; 3])| {
                    let d = ((feet[0] - center[0]).powi(2)
                        + (feet[1] - center[1]).powi(2)
                        + (feet[2] - center[2]).powi(2))
                    .sqrt()
                        / reach;
                    if d > 1.0 {
                        return;
                    }
                    let dir = [feet[0] - center[0], eye_y - center[1], feet[2] - center[2]];
                    let norm = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                    if norm == 0.0 {
                        return;
                    }
                    let seen = crate::explosion::seen_percent(center, bb.0, bb.1, &collision);
                    let impact = (1.0 - d) * seen;
                    if impact <= 0.0 {
                        return;
                    }
                    let push = [
                        dir[0] / norm * impact,
                        dir[1] / norm * impact,
                        dir[2] / norm * impact,
                    ];
                    // `((impact² + impact) / 2) * 7 * diameter + 1`.
                    let damage =
                        ((impact * impact + impact) / 2.0 * 7.0 * reach + 1.0) as f32;
                    hits.push((slot, push, damage));
                };
            for (index, item) in self.item_entities.items.iter().enumerate() {
                if item.removed {
                    continue;
                }
                let bb = crate::entity::item_aabb(item.pos);
                consider(Hit::Item(index), item.pos, item.pos[1] + 0.25 * 0.85, bb);
            }
            for (index, cart) in self.minecarts.iter().enumerate() {
                if cart.removed {
                    continue;
                }
                let bb = crate::minecart::cart_aabb(cart.pos);
                consider(Hit::Cart(index), cart.pos, cart.pos[1] + 0.7 * 0.85, bb);
            }
            for (index, tnt) in self.tnts.iter().enumerate() {
                if tnt.removed {
                    continue;
                }
                let bb = crate::explosion::tnt_aabb(tnt.pos);
                // The one eye-height exception: another primed TNT is pushed
                // from its feet.
                consider(Hit::Tnt(index), tnt.pos, tnt.pos[1], bb);
            }
            for (index, body) in self.bodies.iter().enumerate() {
                let crate::entity::BodyPhysics::Ballistic { pos, .. } = body.physics else {
                    // A frozen body is scaffolding this engine does not blow
                    // around; nothing measured explodes near one.
                    continue;
                };
                let Some(bb) = crate::entity::body_aabb(&body.kind, pos) else { continue };
                let eye = pos[1] + (bb.1[1] - bb.0[1]) * 0.85;
                consider(Hit::Body(index), pos, eye, bb);
            }
        }
        // Phase B: apply. Deaths are collected and removed after the loop —
        // removing mid-loop would shift the indices later hits carry.
        let mut dead_bodies: Vec<u32> = Vec::new();
        for (slot, push, damage) in hits {
            match slot {
                Hit::Item(index) => {
                    // An item entity has 5 health and explosion damage is
                    // never below 1 at nonzero impact — it dies.
                    let _ = damage;
                    self.item_entities.items[index].removed = true;
                }
                Hit::Cart(index) => {
                    for axis in 0..3 {
                        self.minecarts[index].vel[axis] += push[axis];
                    }
                }
                Hit::Tnt(index) => {
                    for axis in 0..3 {
                        self.tnts[index].vel[axis] += push[axis];
                    }
                }
                Hit::Body(index) => {
                    let id = self.bodies[index].id;
                    let crate::entity::BodyPhysics::Ballistic { vel, hp, .. } =
                        &mut self.bodies[index].physics
                    else {
                        unreachable!("collected as ballistic above");
                    };
                    *hp -= damage;
                    if *hp <= 0.0 {
                        // Dead: the body leaves the world (drops unmodelled).
                        dead_bodies.push(id);
                        continue;
                    }
                    for axis in 0..3 {
                        vel[axis] += push[axis];
                    }
                }
            }
        }
        self.bodies.retain(|body| !dead_bodies.contains(&body.id));
        // Phase C: the destruction, in sorted order (vanilla shuffles with
        // world random; nothing measured reads the difference yet). Every
        // write goes through the context so neighbour updates, observers and
        // ticker reconciliation all fire as for any other removal.
        for pos in cleared {
            let mut ctx = TickCtx {
                drain: Some(crate::behaviour::Drain {
                    pending: &mut self.pending,
                    unknown_seen: &mut self.unknown_seen,
                    upd_log: self.upd_log.as_mut(),
                    phase: self.phase,
                }),
                behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
                log: self.log.as_mut(),
            };
            ctx.set(pos, StateId::AIR);
            drop(ctx);
            self.propagate();
        }
    }

    /// `MinecartHopper.tick`: an enabled hopper cart pulls **one item per
    /// game tick** — no transfer cooldown; the 8-a-second hopper cadence is a
    /// block-entity affair the cart does not share. The source is the
    /// container block above the cart's head (`suckInItems` checks the cell
    /// at `y + 1`).
    ///
    /// Not modelled, loudly: the activator-rail disable and item-entity
    /// vacuuming — nothing measured exercises them, and each belongs in here
    /// when something does. A double chest above reads as its own half only:
    /// the cart path does not walk segments.
    fn tick_hopper_carts(&mut self) {
        for index in 0..self.minecarts.len() {
            let above = {
                let cart = &self.minecarts[index];
                if cart.removed || cart.kind != "minecraft:hopper_minecart" {
                    continue;
                }
                // `Hopper.getLevelY` for a cart is `getY() + 0.5`, and
                // `suckInItems` looks one block above that — a cart parked
                // under a top-half trapdoor still reaches the container over
                // the gap.
                Pos::new(
                    cart.pos[0].floor() as i32,
                    (cart.pos[1] + 1.5).floor() as i32,
                    cart.pos[2].floor() as i32,
                )
            };
            let source_block = self
                .registry
                .descriptor(self.world.get(above))
                .map(|descriptor| descriptor.split('[').next().unwrap_or(descriptor))
                .and_then(crate::vanilla::container_slots)
                .is_some();
            // With no container *block* there, `getSourceContainer` falls to
            // the entity container in that cell — a chest cart resting where
            // the block was. Never the pulling cart itself.
            let source_cart = if source_block {
                None
            } else {
                let center = [
                    f64::from(above.x) + 0.5,
                    f64::from(above.y) + 0.5,
                    f64::from(above.z) + 0.5,
                ];
                let mut found: Option<usize> = None;
                for (other, cart) in self.minecarts.iter().enumerate() {
                    if other == index || cart.removed || cart.inventory.is_none() {
                        continue;
                    }
                    let (emin, emax) = crate::minecart::cart_aabb(cart.pos);
                    let hit = (0..3).all(|axis| {
                        emin[axis] < center[axis] + 0.5 && emax[axis] > center[axis] - 0.5
                    });
                    if hit
                        && found
                            .is_none_or(|prev: usize| cart.id < self.minecarts[prev].id)
                    {
                        found = Some(other);
                    }
                }
                if found.is_none() {
                    continue;
                }
                found
            };
            // First occupied source slot, in slot order.
            let source_stacks = match source_cart {
                Some(other) => self.minecarts[other].inventory.as_ref().map(|inv| &inv.stacks),
                None => self.inventories.get(&above).map(|inv| &inv.stacks),
            };
            let Some((source_slot, id, count)) = source_stacks.and_then(|stacks| {
                stacks
                    .iter()
                    .filter(|stack| stack.count > 0)
                    .min_by_key(|stack| stack.slot)
                    .map(|stack| (stack.slot, stack.id.clone(), stack.count))
            }) else {
                continue;
            };
            // First cart slot that takes it: stackable or empty, one walk —
            // `HopperBlockEntity.addItem`'s loop.
            let target_slot = {
                let inv = self.minecarts[index]
                    .inventory
                    .as_ref()
                    .expect("a hopper cart always has its 5 slots");
                let mut found = None;
                for slot in 0..inv.slots.min(255) as u8 {
                    match inv.stacks.iter().find(|s| s.slot == slot && s.count > 0) {
                        None => {
                            found = Some((slot, 1));
                            break;
                        }
                        Some(stack)
                            if stack.id == id
                                && stack.count < crate::vanilla::max_stack(&id) =>
                        {
                            found = Some((slot, stack.count + 1));
                            break;
                        }
                        Some(_) => {}
                    }
                }
                found
            };
            let Some((target_slot, new_count)) = target_slot else { continue };
            let mut ctx = TickCtx {
                drain: Some(crate::behaviour::Drain {
                    pending: &mut self.pending,
                    unknown_seen: &mut self.unknown_seen,
                    upd_log: self.upd_log.as_mut(),
                    phase: self.phase,
                }),
                behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
                log: self.log.as_mut(),
            };
            let remaining = count - 1;
            match source_cart {
                Some(other) => ctx.set_cart_slot(
                    other,
                    source_slot,
                    (remaining > 0).then(|| (id.clone(), remaining)),
                ),
                None => ctx.set_inventory_slot(
                    above,
                    source_slot,
                    (remaining > 0).then(|| (id.clone(), remaining)),
                ),
            }
            ctx.set_cart_slot(index, target_slot, Some((id, new_count)));
            drop(ctx);
            self.propagate();
        }
    }

    fn push_minecart(&mut self, id: u32, kind: String, pos: [f64; 3], vel: [f64; 3]) {
        let inventory =
            crate::minecart::cart_container_slots(&kind).map(crate::inventory::Inventory::empty);
        self.minecarts.push(crate::minecart::MinecartState {
            id,
            kind,
            pos,
            vel,
            on_ground: false,
            on_rails: false,
            removed: false,
            // Vanilla's default for an entity spawned without a `Rotation`
            // tag. The structure reader does not carry rotation, so a build
            // that means to park a cart on a stale heading cannot say so yet —
            // see `push_neighbours`, which gates on this.
            inventory,
            fuse: None,
            yaw: 0.0,
        });
        self.refresh_bodies();
    }

    /// Spawn an entity that is **only** a hitbox: it holds its position and
    /// takes part in every presence question, and has no physics of its own.
    ///
    /// This is what the record doors' frozen fireballs are. Caught mid-flight
    /// by a piston-and-cobweb trick, they have zero motion and no gravity, so
    /// vanilla's own physics leaves them exactly where they are — and a piston
    /// arm shoves them onto a pressure plate. Modelling them as a box that does
    /// not move is not a simplification here; it is what the game does with
    /// them.
    ///
    /// Refuses an unrecognised entity **by name** rather than inventing a
    /// hitbox for it. A wrong box silently produces a wrong door.
    pub fn spawn_frozen_entity(&mut self, kind: String, pos: [f64; 3]) -> Result<u32, String> {
        self.spawn_frozen_entity_leashed(kind, pos, false)
    }

    /// [`Simulation::spawn_frozen_entity`], recording whether the entity was
    /// authored on a leash.
    ///
    /// Separate rather than a fourth argument on the original because every
    /// existing caller means `false` and should keep saying so by saying
    /// nothing. Nothing in the simulation branches on the flag — see
    /// [`crate::entity::EntityBody::leashed`] for what it is for.
    pub fn spawn_frozen_entity_leashed(
        &mut self,
        kind: String,
        pos: [f64; 3],
        leashed: bool,
    ) -> Result<u32, String> {
        if crate::entity::entity_dimensions(&kind).is_none() {
            return Err(format!(
                "unknown entity {kind}: no hitbox is known for it, and guessing one \
                 would silently change the simulation. Measure it with \
                 tools/gametest/src/EntityDims.java, decide whether it stops a cart \
                 with tools/gametest/capture.sh, and add one row to \
                 vanilla::entity_table()."
            ));
        }
        let id = self.item_entities.next_id;
        self.item_entities.next_id += 1;
        self.bodies.push(crate::entity::EntityInstance {
            id,
            kind,
            physics: crate::entity::BodyPhysics::Frozen { pos },
            leashed,
        });
        self.refresh_bodies();
        Ok(id)
    }

    /// Shove entities standing in a moving piston's path — the one thing that
    /// can move an entity whose own physics are dead.
    ///
    /// This is `PistonMovingBlockEntity.moveCollidedEntities`, and the record
    /// doors are built on it: a NaN minecart ignores gravity, collisions and
    /// players, but a piston arm still displaces it, and frozen fireballs are
    /// pushed onto pressure plates by piston heads. The geometry is
    /// [`crate::piston::sweep_displacement`], fitted bit-exactly to
    /// `piston_entity.json`.
    ///
    /// **Displacement only.** Vanilla calls `entity.move(MoverType.PISTON, …)`
    /// and never touches `deltaMovement`, so a pushed cart ends the shove with
    /// exactly the velocity it started with — and a NaN cart keeps its NaN.
    /// The capture confirms all four lanes end at zero velocity.
    ///
    /// A move that is in flight but frozen outside the block-ticking bounds
    /// does not sweep, for the same reason it does not land: its block entity
    /// is not ticking.
    fn displace_entities_by_pistons(&mut self) {
        let tick = self.tick;
        let ticking = self.ticking;
        // `pistonDeltasGameTime`: the running per-axis totals reset once a tick.
        self.piston_deltas.clear();
        // Insertion order, which is what `tickBlockEntities` walks — and the
        // index rides along, because *when* another move's cell turns solid
        // depends on whether its block entity has already ticked this tick.
        let sweeps: Vec<(usize, Pos, crate::piston::Sweep, f64, bool)> = self
            .moves
            .iter()
            .enumerate()
            .filter(|(_, m)| ticking.as_ref().is_none_or(|bounds| bounds.contains(m.pos)))
            .filter_map(|(index, m)| {
                let sweep = m.sweep?;
                // `progress` before this step. A move resolves two ticks after
                // the event that made it, and the two half-block steps are the
                // two ticks in between — so the ticks remaining name the step.
                let progress = match m.resolve_on.checked_sub(tick)? {
                    2 => 0.0,
                    1 => crate::piston::PISTON_STEP,
                    _ => return None,
                };
                Some((index, m.pos, sweep, progress, m.source_piston))
            })
            .collect();
        if sweeps.is_empty() {
            return;
        }
        for (shover, destination, sweep, progress, source_piston) in sweeps {
            let boxes =
                crate::piston::shove_shape_boxes(destination, sweep, progress, source_piston);
            // Carts first, then frozen bodies — the order the entity-box view
            // is built in, so a trace reads the same way. Only a standing body
            // is shoved: a passenger has no position of its own — it is
            // re-derived from its vehicle every tick — so shoving one would be
            // overwritten before it could be observed, and its vehicle is in
            // the cart pass.
            enum Slot {
                Cart(usize),
                Body(usize),
            }
            let mut slots: Vec<Slot> = Vec::new();
            for index in 0..self.minecarts.len() {
                if !self.minecarts[index].removed {
                    slots.push(Slot::Cart(index));
                }
            }
            for index in 0..self.bodies.len() {
                if matches!(self.bodies[index].physics, crate::entity::BodyPhysics::Frozen { .. })
                {
                    slots.push(Slot::Body(index));
                }
            }
            for slot in slots {
                let (id, is_cart, min, max) = match &slot {
                    Slot::Cart(index) => {
                        let cart = &self.minecarts[*index];
                        let (min, max) = crate::minecart::cart_aabb(cart.pos);
                        (cart.id, true, min, max)
                    }
                    Slot::Body(index) => {
                        let body = &self.bodies[*index];
                        let crate::entity::BodyPhysics::Frozen { pos } = body.physics else {
                            continue;
                        };
                        let Some((min, max)) = crate::entity::body_aabb(&body.kind, pos) else {
                            continue;
                        };
                        (body.id, false, min, max)
                    }
                };
                // `moveCollidedEntities`: the shape's swept slabs, max
                // penetration, capped at the step plus the overshoot.
                let Some(distance) =
                    crate::piston::moved_shape_displacement(&boxes, sweep.travel, min, max)
                else {
                    continue;
                };
                if std::env::var_os("MC_TICK_TRACE_SHOVE").is_some() {
                    eprintln!(
                        "t{tick} SHOVE id={id} cart={is_cart} dest={destination:?} \
                         sweep={sweep:?} progress={progress} src={source_piston} \
                         dist={distance} min={min:?}",
                    );
                }
                let mut current = (min, max);
                let mut apply = |sim: &mut Self, from: ([f64; 3], [f64; 3]), travel, amount| {
                    let moved = sim.shove(from.0, from.1, travel, amount, id, is_cart, shover);
                    if std::env::var_os("MC_TICK_TRACE_SHOVE").is_some() {
                        eprintln!(
                            "        APPLY id={id} travel={travel:?} asked={amount} moved={moved:?}"
                        );
                    }
                    let mut nmin = from.0;
                    let mut nmax = from.1;
                    for axis in 0..3 {
                        nmin[axis] += moved[axis];
                        nmax[axis] += moved[axis];
                    }
                    match &slot {
                        Slot::Cart(index) => {
                            for axis in 0..3 {
                                sim.minecarts[*index].pos[axis] += moved[axis];
                            }
                        }
                        Slot::Body(index) => {
                            if let crate::entity::BodyPhysics::Frozen { pos } =
                                &mut sim.bodies[*index].physics
                            {
                                for axis in 0..3 {
                                    pos[axis] += moved[axis];
                                }
                            }
                        }
                    }
                    // `moveEntityByPiston` ends with `applyEffectsFromBlocks`:
                    // `entityInside` fires the moment the entity lands, within
                    // the same block-entity tick — which is how the record
                    // door's plate presses on the very tick the drag delivers
                    // its fireball, and why it counts 1 and not 2: the second
                    // fireball has not moved yet when the first one is counted.
                    // The body view is updated to this entity's new box only,
                    // leaving everyone else where this instant has them.
                    if moved.iter().any(|&d| d != 0.0) {
                        sim.notify_shoved(id, nmin, nmax);
                    }
                    (nmin, nmax)
                };
                if let Some(amount) = self.limit_piston_movement(id, sweep.travel, distance) {
                    current = apply(self, current, sweep.travel, amount);
                }
                // `fixEntityWithinPistonBase`: after every shove by a retracting
                // source piston, an entity still overlapping the piston's own
                // full cell is pushed back out of it, against the head's travel.
                // The box the drag reached is a real intermediate the fix takes
                // back, and `entityInside` has already fired at it above — that
                // transient is what presses the record door's plates.
                if !sweep.extending && source_piston {
                    if let Some(fix) = crate::piston::base_fix_displacement(
                        destination,
                        sweep.travel,
                        current.0,
                        current.1,
                    ) {
                        let out = sweep.travel.opposite();
                        if let Some(amount) = self.limit_piston_movement(id, out, fix) {
                            current = apply(self, current, out, amount);
                        }
                    }
                }
            }
            self.refresh_bodies();
        }
        self.notify_piston_probes();
    }

    /// `Entity.limitPistonMovement` + `applyPistonMovementRestriction`: clamp
    /// this tick's running piston displacement to ±0.51 per axis and zero a
    /// residual under 1e-5.
    ///
    /// `distance` is nonnegative along `travel`; the return value is too, and
    /// `None` means the move is swallowed entirely (`Vec3.ZERO`). The
    /// accumulator records what was *asked*, not what collision later grants —
    /// vanilla runs this at the top of `Entity.move`, before `collide`.
    fn limit_piston_movement(
        &mut self,
        id: u32,
        travel: crate::pos::Dir,
        distance: f64,
    ) -> Option<f64> {
        let (dx, dy, dz) = travel.delta();
        let axis = if dx != 0 {
            0
        } else if dy != 0 {
            1
        } else {
            2
        };
        let sign = f64::from([dx, dy, dz][axis]);
        let acc = self.piston_deltas.entry(id).or_insert([0.0; 3]);
        let requested = sign * distance;
        let clamped = (requested + acc[axis])
            .clamp(-crate::piston::PISTON_MAX_STEP, crate::piston::PISTON_MAX_STEP);
        let amount = clamped - acc[axis];
        acc[axis] = clamped;
        if amount.abs() <= f64::from(1.0e-5f32) {
            return None;
        }
        Some(amount * sign)
    }

    /// `applyEffectsFromBlocks` for one piston-moved entity: update its view
    /// box to where the move just put it and fire `entityInside` there.
    ///
    /// The view is *advanced*, not swapped and restored: the entity really is
    /// at the new box now, and anything counting bodies mid-tick — a weighted
    /// plate pressed by the first of two stacked fireballs — must see exactly
    /// this instant's arrangement, later movers still at their old boxes.
    fn notify_shoved(&mut self, id: u32, min: [f64; 3], max: [f64; 3]) {
        if let Some(slot) = self.item_entities.others.iter().position(|b| b.id == id) {
            self.item_entities.others[slot].min = min;
            self.item_entities.others[slot].max = max;
        }
        self.notify_entity_inside(min, max);
    }

    /// Fire `entityInside` at each intermediate box a piston carried an entity
    /// through this tick.
    ///
    /// This is `Entity.move(MoverType.PISTON, …)` running `applyEffectsFromBlocks`
    /// at the far end of the shove, inside `tickBlockEntities`, before the step's
    /// correction pulls the entity back. The body view is pointed at the probe
    /// box for the duration so that a plate *counts* the entity there and not in
    /// two places at once — `piston_plate_clip` reads `power=1`, never 2 — and
    /// restored from the authoritative lists afterwards.
    fn notify_piston_probes(&mut self) {
        let probes = std::mem::take(&mut self.piston_probes);
        for (id, min, max) in probes {
            let Some(slot) = self.item_entities.others.iter().position(|b| b.id == id) else {
                continue;
            };
            let settled = (self.item_entities.others[slot].min, self.item_entities.others[slot].max);
            self.item_entities.others[slot].min = min;
            self.item_entities.others[slot].max = max;
            self.notify_entity_inside(min, max);
            // `notify_entity_inside` can move blocks, which rebuilds the view.
            if let Some(slot) = self.item_entities.others.iter().position(|b| b.id == id) {
                self.item_entities.others[slot].min = settled.0;
                self.item_entities.others[slot].max = settled.1;
            }
        }
    }

    /// `entityInside` for every cell one box overlaps.
    fn notify_entity_inside(&mut self, emin: [f64; 3], emax: [f64; 3]) {
        let mut cells: Vec<Pos> = Vec::new();
        for x in (emin[0].floor() as i32)..=(emax[0].floor() as i32) {
            for y in (emin[1].floor() as i32)..=(emax[1].floor() as i32) {
                for z in (emin[2].floor() as i32)..=(emax[2].floor() as i32) {
                    cells.push(Pos::new(x, y, z));
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
                drain: Some(crate::behaviour::Drain {
                    pending: &mut self.pending,
                    unknown_seen: &mut self.unknown_seen,
                    upd_log: self.upd_log.as_mut(),
                    phase: self.phase,
                }),
                behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
                log: self.log.as_mut(),
            };
            behaviour.on_entity_inside(&mut ctx, cell);
            self.propagate();
        }
    }

    /// One `moveEntityByPiston`: displace a box `distance` along `travel`,
    /// clipped by whatever is in the way.
    ///
    /// Vanilla's is `entity.move(MoverType.PISTON, …)`, which collides against
    /// blocks and other entities like any other move — the NOCLIP flag exempts
    /// only moving blocks travelling the shove's own way, which is why the
    /// stroke's own blocks do not wall their cargo in.
    ///
    /// **Which bodies stop the entity depends on who is being shoved.**
    /// `Entity.collide` asks `getEntityCollisions`, whose predicate is
    /// `source.canCollideWith(other)` — and `AbstractMinecart` overrides it to
    /// the vehicle rule `other.canBeCollidedWith() || other.isPushable()`,
    /// which is [`crate::entity::blocks_a_cart`]'s measured table, while every
    /// other entity keeps `Entity`'s default of `other.canBeCollidedWith()`
    /// alone — true for a boat, false for a blaze, a cart, a villager and every
    /// fireball. So a piston-shoved cart is stopped by a blaze, and a
    /// piston-shoved fireball passes straight through one. The record 3x3 door
    /// depends on the second half: its dragon fireballs return east *through*
    /// the blaze wall its nan carts are seating.
    ///
    /// The block clipping **is** measured, and it was the last thing holding the
    /// record 3x3 door shut — see [`Simulation::blocks_in_flight`] for the
    /// surface that does it and the capture that pins it, and
    /// [`Simulation::retracting_base_boxes`] for the 12/16 slab a retracting
    /// piston's own square keeps solid.
    fn shove(
        &self,
        min: [f64; 3],
        max: [f64; 3],
        travel: crate::pos::Dir,
        distance: f64,
        moving_id: u32,
        moving_is_cart: bool,
        shover: usize,
    ) -> [f64; 3] {
        let (dx, dy, dz) = travel.delta();
        let movement = [
            f64::from(dx) * distance,
            f64::from(dy) * distance,
            f64::from(dz) * distance,
        ];
        let mut obstacles: Vec<([f64; 3], [f64; 3])> = self
            .item_entities
            .others
            .iter()
            .filter(|body| body.id != moving_id)
            .filter(|body| {
                if moving_is_cart {
                    crate::entity::blocks_a_cart(&body.kind) == Some(true)
                } else {
                    crate::entity::hard_collides(&body.kind)
                }
            })
            .map(|body| (body.min, body.max))
            .collect();
        // What each in-flight cell contributes depends on whether its block
        // entity has already ticked this tick, which is insertion order:
        //
        // * a move on its **second step** whose entity ran *before* the shover
        //   has set `progress = 1.0` by now, so `getCollisionShape` answers
        //   with the landed block at its destination — a full cube, if what
        //   lands is one. This is the "solid at its destination on the second
        //   step" the wide-body capture measured, made per-observer: to its own
        //   shove a cell is never its landed cube;
        // * every other in-flight cell still has `progress < 1.0`, and the
        //   shover's NOCLIP exemption reduces it to
        //   `getCollisionShape`'s early return: the extended base's 12/16 slab
        //   for a retracting source piston, nothing at all otherwise.
        for (index, m) in self.moves.iter().enumerate() {
            let Some(sw) = m.sweep else { continue };
            let steps_left = m.resolve_on.saturating_sub(self.tick);
            if !(1..=2).contains(&steps_left) {
                continue;
            }
            if steps_left == 1 && index < shover {
                if self.solidity.get(m.state.raw() as usize).copied().unwrap_or(false) {
                    let lo = [f64::from(m.pos.x), f64::from(m.pos.y), f64::from(m.pos.z)];
                    obstacles.push((lo, [lo[0] + 1.0, lo[1] + 1.0, lo[2] + 1.0]));
                }
            } else if m.source_piston && !sw.extending {
                obstacles.push(crate::piston::retracting_base_box(m.pos, sw.travel));
            }
        }
        let collision = SimCollision {
            world: &self.world,
            solidity: &self.solidity,
            frictions: &self.frictions,
            heights: &self.heights,
            webs: &self.webs,
            water_kinds: &self.water_kinds,
            bubble_kinds: &self.bubble_kinds,
            rails: &self.rails,
            conductors: &self.conductors,
        };
        let (clipped, _) = crate::entity::collide_move_among(&collision, min, max, movement, &obstacles);
        clipped
    }

    /// The in-flight cells' boxes, for an entity moving under its **own** power.
    ///
    /// [`Simulation::shove`] answers for the entity a piston is shoving, and
    /// there the cells turn solid one by one, in block-entity order, because
    /// vanilla exempts the shove's own stroke: `PistonMovingBlockEntity
    /// .getCollisionShape` skips the shape while `progress < 1.0` *and* the
    /// entity's NOCLIP direction matches the stroke. An entity falling of its
    /// own accord matches neither clause, so it gets the shape on both steps of
    /// every move — and that is the only difference between this list and the
    /// per-shover set `shove` builds.
    ///
    /// That was recorded as an open question here ("a minecart rolling of its
    /// own accord still passes through a moving block, which is a separate and
    /// unmeasured question"). It is measured now.
    /// `piston_cart_support.json` retracts a west-facing piston out from under a
    /// furnace cart resting on its top face: the base cell holds a
    /// `moving_piston` for the two ticks the head takes to come home, and the
    /// cart does not move a thousandth in either of them. Without this the cart
    /// took gravity for those two ticks, ended up inside the cell the retracted
    /// base was about to reoccupy, and fell out of the world from there — three
    /// of the record door's fifteen furnace carts, ids 14, 21 and 23, each
    /// standing over a `sticky_piston` that fires when the button is pressed.
    ///
    /// Like the shove path this counts only cells whose **landed** state is
    /// solid, so a stroke that lands a `piston_head` contributes nothing.
    fn blocks_in_flight_to_ordinary_motion(&self) -> Vec<([f64; 3], [f64; 3])> {
        self.moves
            .iter()
            .filter(|m| m.sweep.is_some())
            .filter(|m| self.solidity.get(m.state.raw() as usize).copied().unwrap_or(false))
            .map(|m| {
                let lo = [f64::from(m.pos.x), f64::from(m.pos.y), f64::from(m.pos.z)];
                (lo, [lo[0] + 1.0, lo[1] + 1.0, lo[2] + 1.0])
            })
            .collect()
    }

    /// The boxes of the non-cart entities a moving minecart is stopped by.
    ///
    /// Carts are **not** here — [`crate::minecart::tick_minecart_among`] adds
    /// those itself, and it has to, because it must leave out the cart doing the
    /// moving.
    ///
    /// This is the record doors' "the minecart rests on the villager's head":
    /// a mob's hitbox used as scaffolding to hold a cart at an exact height. It
    /// is a full box, not a ledge — a cart is stopped sideways by a blaze
    /// exactly as it is stopped from below (`cart_body2`). Which kinds qualify
    /// is [`crate::entity::blocks_a_cart`], which is a measured table and not a
    /// class test; the fireballs are in `frozen` too and are transparent.
    ///
    /// A rider's position is re-derived from its vehicle here rather than
    /// stored, for the same reason [`Simulation::refresh_bodies`] does it: a
    /// passenger has no position of its own. `cart_body2` drops a furnace cart
    /// onto a blaze seated on a NaN cart and it rests at 2.987499952316284 —
    /// the vehicle's y, plus the 0.1875 seat, plus the blaze's 1.8 — so a
    /// rider's box holds a cart up just as a standing body does.
    fn cart_obstacle_bodies(&self) -> Vec<([f64; 3], [f64; 3])> {
        let mut out = self.blocks_in_flight_to_ordinary_motion();
        for standing in [true, false] {
            for body in &self.bodies {
                if matches!(body.physics, crate::entity::BodyPhysics::Frozen { .. }) != standing {
                    continue;
                }
                if crate::entity::blocks_a_cart(&body.kind) != Some(true) {
                    continue;
                }
                let Some(pos) = self.body_position(body) else { continue };
                // `None` is unreachable: spawn refuses a kind with no dimensions.
                if let Some(aabb) = crate::entity::body_aabb(&body.kind, pos) {
                    out.push(aabb);
                }
            }
        }
        out
    }

    /// Where a body actually is: stored for a standing one, derived from its
    /// vehicle for a passenger.
    ///
    /// `None` for a rider whose vehicle has been removed — a removed vehicle
    /// takes its rider's box with it, which is why the lookup is a filter and
    /// not an unwrap.
    fn body_position(&self, body: &crate::entity::EntityInstance) -> Option<[f64; 3]> {
        match body.physics {
            crate::entity::BodyPhysics::Frozen { pos }
            | crate::entity::BodyPhysics::Ballistic { pos, .. } => Some(pos),
            crate::entity::BodyPhysics::Rider { vehicle, seat } => {
                let vehicle_pos = self
                    .minecarts
                    .iter()
                    .find(|cart| cart.id == vehicle && !cart.removed)
                    .map(|cart| cart.pos)
                    .or_else(|| {
                        self.bodies.iter().find_map(|candidate| {
                            if candidate.id != vehicle {
                                return None;
                            }
                            match candidate.physics {
                                crate::entity::BodyPhysics::Frozen { pos }
                                | crate::entity::BodyPhysics::Ballistic { pos, .. } => Some(pos),
                                crate::entity::BodyPhysics::Rider { .. } => None,
                            }
                        })
                    })?;
                Some([
                    vehicle_pos[0] + seat[0],
                    vehicle_pos[1] + seat[1],
                    vehicle_pos[2] + seat[2],
                ])
            }
        }
    }

    /// Rebuild the entity-box view block behaviours read
    /// ([`crate::entity::EntityBody`]).
    ///
    /// Vanilla answers `getEntitiesOfClass` against the live entity list, so
    /// this has to be current wherever a behaviour might ask: after the carts
    /// move, and after any spawn. It is a derived view — cheap to rebuild and
    /// never the authoritative copy.
    fn refresh_bodies(&mut self) {
        let bodies = &mut self.item_entities.others;
        bodies.clear();
        for cart in &self.minecarts {
            if cart.removed {
                continue;
            }
            let (min, max) = crate::minecart::cart_aabb(cart.pos);
            bodies.push(crate::entity::EntityBody {
                id: cart.id,
                kind: cart.kind.clone(),
                min,
                max,
                is_minecart: true,
                // A cart is never tethered: vanilla has no leashable minecart.
                leashed: false,
            });
        }
        // Standing bodies, then riders — two passes over the one collection,
        // because the order this view is built in is observable (it is the order
        // `entityInside` visits cells in) and this is the order it has always
        // had.
        //
        // A rider's box is derived rather than stored. This is
        // `Entity.positionRider`: the passenger is placed at the vehicle's
        // position plus its seat, every tick, unconditionally. Rebuilding it
        // here rather than keeping a rider position means a rider cannot drift
        // from its vehicle even by one tick — including a vehicle a piston just
        // shoved, and including one whose velocity is NaN and never moves at
        // all.
        for standing in [true, false] {
            for body in &self.bodies {
                if matches!(body.physics, crate::entity::BodyPhysics::Frozen { .. }) != standing {
                    continue;
                }
                let Some(pos) = self.body_position(body) else { continue };
                // Refused at spawn if the kind had no dimensions, so this cannot
                // fall back to a guessed box.
                let Some((min, max)) = crate::entity::body_aabb(&body.kind, pos) else { continue };
                self.item_entities.others.push(crate::entity::EntityBody {
                    id: body.id,
                    kind: body.kind.clone(),
                    min,
                    max,
                    // A blaze on a cart is not an `AbstractMinecart`, and a
                    // detector rail selects on that class — so a rider must not
                    // power one just because its vehicle would. No frozen body
                    // is a cart either: every cart is in `minecarts`.
                    is_minecart: false,
                    leashed: body.leashed,
                });
            }
        }
    }

    /// Where a passenger is sitting, derived from its vehicle.
    ///
    /// `(rider id, kind, position)`. Exposed for the same reason
    /// [`Simulation::entity_bodies`] is: a rider has no state beyond where it
    /// is, so its position *is* the entity.
    pub fn riders(&self) -> Vec<(u32, String, [f64; 3])> {
        self.bodies
            .iter()
            .filter(|body| matches!(body.physics, crate::entity::BodyPhysics::Rider { .. }))
            .filter_map(|body| Some((body.id, body.kind.clone(), self.body_position(body)?)))
            .collect()
    }

    /// Every non-item entity's world-space box, as block behaviours see it.
    ///
    /// The only public view of a frozen entity's position: it has no state
    /// beyond a box, so exposing the box *is* exposing the entity.
    pub fn entity_bodies(&self) -> &[crate::entity::EntityBody] {
        &self.item_entities.others
    }

    /// The live minecarts.
    pub fn minecarts(&self) -> &[crate::minecart::MinecartState] {
        &self.minecarts
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

    /// One random-tick pass — `randomTickSpeed` attempts per 4096 blocks.
    fn run_random_ticks(&mut self) {
        let bounds = self.world.bounds();
        let (min, max) = (bounds.min, bounds.max);
        let size = (
            (max.x - min.x + 1).max(1),
            (max.y - min.y + 1).max(1),
            (max.z - min.z + 1).max(1),
        );
        let volume = i64::from(size.0) * i64::from(size.1) * i64::from(size.2);
        let attempts = ((volume + 4095) / 4096) * i64::from(self.random_tick_speed);
        let mut picks = Vec::with_capacity(attempts as usize);
        for _ in 0..attempts {
            picks.push(Pos::new(
                min.x + self.random_rng.next_int(size.0),
                min.y + self.random_rng.next_int(size.1),
                min.z + self.random_rng.next_int(size.2),
            ));
        }
        for pos in picks {
            let state = self.world.get(pos);
            let Some(behaviour) = self.behaviours.get(state) else { continue };
            let mut ctx = TickCtx {
                drain: Some(crate::behaviour::Drain {
                    pending: &mut self.pending,
                    unknown_seen: &mut self.unknown_seen,
                    upd_log: self.upd_log.as_mut(),
                    phase: self.phase,
                }),
                behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
                log: self.log.as_mut(),
            };
            behaviour.on_random_tick(&mut ctx, pos);
            self.propagate();
        }
    }

    /// The changes recorded since [`Simulation::record`], in order.
    pub fn recorded(&self) -> &[BlockChange] {
        self.log.as_deref().unwrap_or(&[])
    }

    /// Start (or stop) recording every delivered update.
    ///
    /// Off by default and deliberately separate from [`Simulation::record`]:
    /// a door's cycle produces several updates per block change, and only a
    /// propagation view wants them.
    ///
    /// Turning it *off* stops recording but keeps what was recorded — a
    /// caller that records a cycle, switches the recorder off to run an
    /// unwatched settle, and only then reads the log gets the cycle rather
    /// than nothing. Turning it back *on* starts a fresh log; use
    /// [`Simulation::clear_updates`] to drop one without changing the
    /// recording state.
    pub fn record_updates(&mut self, on: bool) {
        if on {
            self.upd_saved = None;
            self.upd_log = Some(Vec::new());
        } else {
            // Retained, not dropped: `recorded_updates` reads through to it.
            self.upd_saved = self.upd_log.take();
        }
    }

    /// Discard the recorded updates, leaving recording as it was.
    ///
    /// The log is the largest thing a simulation holds — a 6x6 door cycle is
    /// tens of megabytes — so a long-lived instance that records many cycles
    /// needs a way to free one without stopping.
    pub fn clear_updates(&mut self) {
        if let Some(log) = self.upd_log.as_mut() {
            log.clear();
        }
        self.upd_saved = None;
    }

    /// The updates recorded since [`Simulation::record_updates`], in delivery
    /// order — which is `(tick, seq)` order. Survives the recorder being
    /// switched off; emptied by [`Simulation::clear_updates`].
    pub fn recorded_updates(&self) -> &[crate::behaviour::UpdateRecord] {
        self.upd_log
            .as_deref()
            .or(self.upd_saved.as_deref())
            .unwrap_or(&[])
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
    /// The block ticks this simulation has pending, as `(fires on, position)`.
    pub fn pending_ticks(&self) -> Vec<(u64, Pos)> {
        self.ticks.pending()
    }

    /// The block events queued for the next block-events phase, in order.
    pub fn pending_events(&self) -> Vec<(Pos, u8)> {
        self.events.peek().iter().map(|e| (e.pos, e.id)).collect()
    }

    pub fn tick_count(&self) -> u64 {
        self.tick
    }

    /// Every block a piston currently has in flight.
    ///
    /// The engine's own answer to the question a renderer otherwise guesses at.
    /// A block change says only that a cell became a `moving_piston`
    /// placeholder; it does not say which block set off, which cell it left,
    /// or which tick it arrives — so a consumer that reconstructs strokes from
    /// the change stream is reimplementing [`crate::piston`] downstream of it,
    /// and its animation lands on a different clock from the simulation's.
    /// Every artefact that follows (a block drawn twice, a gap where one
    /// should be, a piston head shearing off the block it pushes) is that
    /// disagreement made visible.
    ///
    /// Deferred writes with no [`Sweep`](crate::piston::Sweep) are not here:
    /// those are delayed *writes*, not blocks in motion, and nothing travels.
    pub fn moving_blocks(&self) -> Vec<MovingBlock> {
        self.moves
            .iter()
            .filter_map(|m| {
                let sweep = m.sweep?;
                Some(MovingBlock {
                    to: m.pos,
                    from: m.pos.offset(sweep.travel.opposite()),
                    state: m.state,
                    carried: m.carried,
                    carried_short: m.carried_short,
                    remains: m.remains,
                    travel: sweep.travel,
                    extending: sweep.extending,
                    started_on: m.started_on,
                    lands_on: m.resolve_on,
                    source_piston: m.source_piston,
                })
            })
            .collect()
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
                self.phase = None;
                return stop;
            }
        }
        self.emit_entity_events();
        self.in_tick = false;
        self.phase = None;
        self.tick += 1;
        if let Some(timeline) = self.timeline.as_mut() {
            timeline.frames.push(crate::timeline::StateFrame::of(
                self.tick,
                &self.world,
                &self.registry,
            ));
        }
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
            self.minecarts.retain(|cart| !cart.removed);
            return;
        }
        let mut events: Vec<(u64, EntityEvent)> = Vec::new();
        let mut seen: std::collections::HashSet<u32> = std::collections::HashSet::new();
        let mut observations: Vec<(u32, String, [f64; 3], [f64; 3])> = Vec::new();
        for item in &self.item_entities.items {
            if item.removed {
                continue;
            }
            observations.push((item.id, "minecraft:item".to_string(), item.pos, item.vel));
        }
        for cart in &self.minecarts {
            if cart.removed {
                continue;
            }
            observations.push((cart.id, cart.kind.clone(), cart.pos, cart.vel));
        }
        for (id, entity_type, pos, velocity) in observations {
            seen.insert(id);
            let moved = match self.entity_snapshot.get(&id) {
                None => true,
                Some(last) => last.iter().zip(&pos).any(|(a, b)| (a - b).abs() > 1.0e-9),
            };
            if moved {
                events.push((
                    self.tick,
                    EntityEvent::Moved { id, entity_type, pos, velocity },
                ));
                self.entity_snapshot.insert(id, pos);
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
        self.minecarts.retain(|cart| !cart.removed);
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
            command_powered: self.command_powered.clone(),
            item_entities: self.item_entities.clone(),
            minecarts: self.minecarts.clone(),
            bodies: self.bodies.clone(),
            tnts: self.tnts.clone(),
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
        self.command_powered = checkpoint.command_powered.clone();
        self.item_entities = checkpoint.item_entities.clone();
        self.minecarts = checkpoint.minecarts.clone();
        self.bodies = checkpoint.bodies.clone();
        self.tnts = checkpoint.tnts.clone();
        // Event emission restarts from the restored state.
        self.entity_snapshot.clear();
        for item in &self.item_entities.items {
            self.entity_snapshot.insert(item.id, item.pos);
        }
        for cart in &self.minecarts {
            self.entity_snapshot.insert(cart.id, cart.pos);
        }
    }

    /// Set the container contents at `pos`, as loading a structure does.
    /// Seed a comparator's stored output strength, as a saved build carries it.
    ///
    /// `ComparatorBlockEntity.outputSignal` is what the comparator emits until
    /// it re-evaluates, and a schematic saved mid-cycle carries it. Without
    /// this every loaded comparator starts at zero — true of a freshly placed
    /// one, false of a saved one.
    /// Give the command block at `pos` its program.
    ///
    /// Parsed from the block's `Command` NBT at load — see
    /// [`crate::vanilla::parse_command`] for the supported subset. A command
    /// block with no program still powers on and off; it just runs nothing,
    /// like an unparseable command in game.
    pub fn set_command(&mut self, pos: Pos, program: crate::behaviour::CommandProgram) {
        self.commands.insert(pos, program);
    }

    pub fn set_comparator_output(&mut self, pos: Pos, strength: u8) {
        self.comparator_out.insert(pos, strength);
    }

    /// Record that `pos` holds a block entity. `placeInWorld` calls
    /// `BlockEntity.setChanged` on each of these, so the placement pass has to
    /// know about them even when we model nothing of their contents.
    pub fn mark_block_entity(&mut self, pos: Pos) {
        self.block_entities.insert(pos);
    }

    /// Give the container at `pos` its contents, and record it as a block
    /// entity.
    pub fn set_inventory(&mut self, pos: Pos, inventory: crate::inventory::Inventory) {
        self.block_entities.insert(pos);
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

    /// Deliver queued neighbour updates exactly as `CollectingNeighborUpdater`
    /// does: after every single notification, entries queued during it join
    /// the stack in call order and run depth-first before the current entry's
    /// remaining notifications.
    ///
    /// Bounded (vanilla's `maxChainedNeighborUpdates` is a million): a circuit
    /// that keeps re-notifying itself reports rather than hangs.
    fn propagate(&mut self) {
        let mut ctx = self.ctx();
        ctx.drain();
    }

    /// A dispatch context over this simulation, with the update pump wired in
    /// so a behaviour can drain mid-handler.
    fn ctx(&mut self) -> TickCtx<'_> {
        TickCtx {
            drain: Some(crate::behaviour::Drain {
                pending: &mut self.pending,
                unknown_seen: &mut self.unknown_seen,
                upd_log: self.upd_log.as_mut(),
                phase: self.phase,
            }),
            behaviours: Some(&self.behaviours),
            world: &mut self.world,
            ticks: &mut self.ticks,
            fluids: &mut self.fluids,
            events: &mut self.events,
            states: &self.registry,
            tick: self.tick,
            // A cascade running outside a phase walk is a boundary action; its
            // schedules use last-completed-tick time. See TickCtx::boundary.
            boundary: !self.in_tick,
            updates: &mut self.updates,
            moves: &mut self.moves,
            toggles: &mut self.toggles,
            comparator_out: &mut self.comparator_out,
            inventories: &mut self.inventories,
            hopper_state: &mut self.hopper_state,
            commands: &self.commands,
            command_powered: &mut self.command_powered,
            tickers: &mut self.tickers,
            item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
            inv_log: self.inv_log.as_mut(),
            log: self.log.as_mut(),
        }
    }

    /// Notify the neighbours of `pos`, as an external change would.
    ///
    /// This is how a lever flip or a block break enters the simulation.
    pub fn notify_neighbors(&mut self, pos: Pos) {
        self.updates
            .push(crate::behaviour::UpdateEntry::neighbors_at(pos));
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
        self.settle_with_order(&occupied);
    }

    /// Give every placed block its `onPlace`, in placement order.
    ///
    /// `LevelChunk.setBlockState` calls `onPlace` on the new state, and that
    /// happens for **every** flag combination a structure placement uses —
    /// including `knownShape`, which suppresses the neighbour and shape passes
    /// but not this. So even a "silent" placement wakes a piston that is
    /// already powered (`PistonBaseBlock.onPlace` → `checkIfExtend`) and lets
    /// an observer saved mid-pulse clear itself.
    ///
    /// Without it a quietly loaded community build never starts: the engine
    /// sat at zero events while the game did twelve on tick 0.
    /// The walk is *incremental*: the structure's blocks are lifted back out of
    /// the world and written one at a time, so a block's `onPlace` sees only
    /// what vanilla had written by that point. It matters more than it sounds —
    /// a comparator's `updateNeighborsInFront` fires into the repeater beside
    /// it, and whether that repeater exists yet decides whether it books a tick
    /// during placement or waits to be woken later.
    pub fn place_on_place(&mut self, order: &[Pos]) {
        let index: std::collections::HashMap<Pos, usize> =
            order.iter().enumerate().map(|(i, pos)| (*pos, i)).collect();
        let placed: Vec<StateId> = order.iter().map(|pos| self.world.get(*pos)).collect();
        for pos in order {
            self.world.set(*pos, StateId::AIR);
        }
        for (i, pos) in order.iter().enumerate() {
            self.world.set(*pos, placed[i]);
            // The write's own shape propagation: `markAndNotifyBlock` skips it
            // only for `UPDATE_KNOWN_SHAPE`, which a structure placement does
            // not set on its block writes — so this happens even under
            // `knownShape`, and it is how an observer gets its pulse scheduled
            // in a build the game places "silently". Only blocks already down
            // can hear it.
            let items: Vec<(Pos, crate::pos::Dir, crate::behaviour::UpdateKind)> =
                crate::pos::UPDATE_SHAPE_ORDER
                    .iter()
                    .filter(|dir| {
                        index
                            .get(&pos.offset(**dir))
                            .is_some_and(|placed| *placed < i)
                    })
                    .map(|dir| {
                        (
                            pos.offset(*dir),
                            dir.opposite(),
                            crate::behaviour::UpdateKind::Shape,
                        )
                    })
                    .collect();
            if !items.is_empty() {
                self.updates.push(crate::behaviour::UpdateEntry::new(items));
                self.propagate();
            }

            let state = self.world.get(*pos);
            if state == StateId::AIR {
                continue;
            }
            let Some(behaviour) = self.behaviours.get(state) else {
                self.unknown_seen.push(state);
                continue;
            };
            let mut ctx = TickCtx {
                drain: Some(crate::behaviour::Drain {
                    pending: &mut self.pending,
                    unknown_seen: &mut self.unknown_seen,
                    upd_log: self.upd_log.as_mut(),
                    phase: self.phase,
                }),
                behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
                log: self.log.as_mut(),
            };
            behaviour.on_placed(&mut ctx, *pos);
            self.propagate();

            // `markAndNotifyBlock` after the write: flag 1 is set (the harness
            // places with flags 3), so every block notifies its neighbours as
            // it lands. `knownShape` gates the *update pass* that follows the
            // placement loop, not this — which is why a "silent" placement
            // still schedules torches and wakes pistons.
            //
            // The order within a write is `onPlace`, then this: a block reacts
            // to its own arrival before the neighbourhood hears about it.
            self.updates
                .push(crate::behaviour::UpdateEntry::neighbors_at(*pos));
            self.propagate();
        }

        // `placeInWorld` ends each block's turn with `BlockEntity.setChanged()`
        // for anything carrying NBT — and that runs even under `knownShape`,
        // where nothing else does. `setChanged` calls
        // `Level.updateNeighbourForOutputSignal`, so every placed container
        // pokes the comparators around it. Without it a comparator saved
        // `powered=true` over a barrel is never told to re-read, sits lit, and
        // locks the repeater beside it — which was the whole of the tick-1 gap.
        let containers: Vec<Pos> = order
            .iter()
            .copied()
            .filter(|pos| self.block_entities.contains(pos))
            .collect();
        for pos in containers {
            self.update_neighbour_for_output_signal(pos);
            self.propagate();
        }
    }

    /// Place one block the way a player's hand does.
    ///
    /// Vanilla's placement derives the block's state from its surroundings
    /// *before* anything reacts — `getStateForPlacement` plus the
    /// `updateShape` chain — so a wire arrives already connected, and only
    /// then does `onPlace` run and the neighbourhood hear about it. The
    /// engine's equivalent, in order:
    ///
    /// 1. write the state, and deliver a shape update *to* the placed block
    ///    from all six sides — the placed state re-derives itself in place;
    /// 2. `on_placed` — the genuine-placement hook (a wire recomputes its
    ///    power, a repeater its lock);
    /// 3. the flag-3 notification: neighbour updates plus the shape pass.
    ///
    /// This differs from [`Self::place_block`], which is an *actuator write*
    /// (a dropped-in redstone block, a test harness poke): that path runs
    /// `on_state_changed` and never the placement derivation, which is why a
    /// wire written through it stays shaped as authored.
    pub fn place_block_by_hand(&mut self, pos: Pos, state: StateId) {
        let previous = self.world.get(pos);
        if previous == state {
            return;
        }
        self.world.set(pos, state);
        self.inventories.remove(&pos);
        if let Some(log) = self.log.as_mut() {
            log.push(BlockChange { tick: self.tick, pos, from: previous, to: state });
        }

        // 1. The placement derivation: the placed block hears a shape update
        //    from every side, in vanilla's own shape order.
        let items: Vec<(Pos, crate::pos::Dir, crate::behaviour::UpdateKind)> =
            crate::pos::UPDATE_SHAPE_ORDER
                .iter()
                .map(|dir| (pos, *dir, crate::behaviour::UpdateKind::Shape))
                .collect();
        self.updates.push(crate::behaviour::UpdateEntry::new(items));
        self.propagate();

        // 2. `onPlace`, on whatever the derivation left in the cell.
        let state = self.world.get(pos);
        if state != StateId::AIR {
            if let Some(behaviour) = self.behaviours.get(state) {
                let mut ctx = TickCtx {
                    drain: Some(crate::behaviour::Drain {
                        pending: &mut self.pending,
                        unknown_seen: &mut self.unknown_seen,
                        upd_log: self.upd_log.as_mut(),
                        phase: self.phase,
                    }),
                    behaviours: Some(&self.behaviours),
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
                    commands: &self.commands,
                    command_powered: &mut self.command_powered,
                    tickers: &mut self.tickers,
                    item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                    inv_log: self.inv_log.as_mut(),
                    log: self.log.as_mut(),
                };
                behaviour.on_placed(&mut ctx, pos);
            } else {
                self.unknown_seen.push(state);
            }
            self.propagate();
        }

        // 2b. Vanilla's `getStateForPlacement` also reads the *inputs* — a
        //     repeater arrives already knowing whether it should turn on. The
        //     engine derives that lazily instead: one neighbour-changed poke
        //     to the placed block, so anything input-driven schedules its own
        //     check. Idempotent for everything else (a wire just recomputes
        //     the power it already has).
        self.updates.push(crate::behaviour::UpdateEntry::new(vec![(
            pos,
            crate::pos::Dir::Down,
            crate::behaviour::UpdateKind::Neighbor,
        )]));
        self.propagate();

        // 3. markAndNotifyBlock, flag 3: neighbours and the observer pass.
        self.updates.push(crate::behaviour::UpdateEntry::neighbors_at(pos));
        self.updates.push(crate::behaviour::UpdateEntry::neighbor_shapes(pos));
        self.propagate();
    }

    /// `Level.updateNeighbourForOutputSignal`: the *comparator* notification a
    /// changed container sends.
    ///
    /// Narrower than its name suggests, and the narrowness is the point: only
    /// the four **horizontal** neighbours, and only if the block there is a
    /// comparator — or, past a block that conducts, the comparator one step
    /// beyond it. Everything else adjacent hears nothing at all.
    ///
    /// Reading it as "notify all six neighbours" put four comparators on the
    /// schedule that vanilla leaves alone, the moment the door fixtures
    /// regained their block entities.
    fn update_neighbour_for_output_signal(&mut self, pos: Pos) {
        const HORIZONTAL: [crate::pos::Dir; 4] = [
            crate::pos::Dir::North,
            crate::pos::Dir::South,
            crate::pos::Dir::West,
            crate::pos::Dir::East,
        ];
        for dir in HORIZONTAL {
            let side = pos.offset(dir);
            let state = self.world.get(side);
            if self.registry.descriptor(state).is_some_and(is_comparator) {
                self.updates.push(crate::behaviour::UpdateEntry::new(vec![(
                    side,
                    dir.opposite(),
                    crate::behaviour::UpdateKind::Neighbor,
                )]));
            } else if self
                .conductors
                .get(state.raw() as usize)
                .copied()
                .unwrap_or(false)
            {
                let beyond = side.offset(dir);
                let state = self.world.get(beyond);
                if self.registry.descriptor(state).is_some_and(is_comparator) {
                    self.updates.push(crate::behaviour::UpdateEntry::new(vec![(
                        beyond,
                        dir.opposite(),
                        crate::behaviour::UpdateKind::Neighbor,
                    )]));
                }
            }
        }
    }


    /// [`Simulation::settle`] with an explicit placement order — the structure
    /// file's block list, which is the order `StructureTemplate.placeInWorld`
    /// walks.
    ///
    /// Per block vanilla runs `updateFromNeighbourShapes` and then
    /// `updateNeighborsAt(pos)`, each cascade fully drained before the next
    /// block. The shape pass is **not** a neighbour dispatch — it calls
    /// `updateShape`, which only rewrites the block's own state (fence
    /// connections, dust shapes) — so this engine, whose shapes come from the
    /// structure file, has nothing to do for it. Dispatching it as a
    /// neighbour-change was wrong and fired every piston in a door a tick
    /// early; a block hears about placement only through its *neighbours'*
    /// `updateNeighborsAt` passes, which is also how an observer facing a
    /// placed block pulses.
    pub fn settle_with_order(&mut self, order: &[Pos]) {
        // Pass 1 (the `setBlock` loop's shape propagation and `onPlace`) is
        // [`Simulation::place_on_place`], which runs for quiet placement too.
        self.update_shape_at_edge(order);
        // Pass 3 — the update pass: `updateFromNeighbourShapes` then a fully
        // drained `updateNeighborsAt`, per block.
        for pos in order {
            if self.world.get(*pos) == StateId::AIR {
                continue;
            }
            self.updates
                .push(crate::behaviour::UpdateEntry::own_shapes(*pos));
            self.propagate();
            self.updates
                .push(crate::behaviour::UpdateEntry::neighbors_at(*pos));
            self.propagate();
        }
    }

    /// Pass 2 of `placeInWorld`: `StructureTemplate.updateShapeAtEdge`.
    ///
    /// Before the per-block update pass, the game walks the *surface* of what it
    /// just placed and shape-updates each boundary pair — the placed block and
    /// the world block outside it, both ways. `DiscreteVoxelShape.forAllFaces`
    /// drives it, and its order has nothing to do with placement order: three
    /// sweeps, one per axis (`AxisCycle` NONE, FORWARD, BACKWARD), each scanning
    /// along that axis and emitting a face wherever filled meets empty —
    /// negative-facing on entering a run, positive-facing on leaving it.
    ///
    /// That is the order two observers on opposite faces of a build pulse in,
    /// so it decides races that placement order has no say in.
    fn update_shape_at_edge(&mut self, order: &[Pos]) {
        let filled: std::collections::HashSet<Pos> = order.iter().copied().collect();
        let (Some(min), Some(max)) = (
            order.iter().copied().reduce(|a, b| Pos::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))),
            order.iter().copied().reduce(|a, b| Pos::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))),
        ) else {
            return;
        };

        // (outer axis, middle axis, scan axis) per sweep, with the face pair the
        // scan axis emits.
        let sweeps: [(usize, usize, usize, crate::pos::Dir, crate::pos::Dir); 3] = [
            (0, 1, 2, crate::pos::Dir::North, crate::pos::Dir::South),
            (2, 0, 1, crate::pos::Dir::Down, crate::pos::Dir::Up),
            (1, 2, 0, crate::pos::Dir::West, crate::pos::Dir::East),
        ];
        let lo = [min.x, min.y, min.z];
        let hi = [max.x, max.y, max.z];
        for (outer, middle, scan, negative, positive) in sweeps {
            for l in lo[outer]..=hi[outer] {
                for m in lo[middle]..=hi[middle] {
                    let mut inside = false;
                    for s in lo[scan]..=hi[scan] + 1 {
                        let at = |v: i32| {
                            let mut c = [0i32; 3];
                            c[outer] = l;
                            c[middle] = m;
                            c[scan] = v;
                            Pos::new(c[0], c[1], c[2])
                        };
                        let full = s <= hi[scan] && filled.contains(&at(s));
                        if !inside && full {
                            self.shape_update_pair(at(s), negative);
                        }
                        if inside && !full {
                            self.shape_update_pair(at(s - 1), positive);
                        }
                        inside = full;
                    }
                }
            }
        }
    }

    /// One face of the edge walk: the placed block hears from the world block
    /// across the face, then that block hears back.
    fn shape_update_pair(&mut self, pos: Pos, dir: crate::pos::Dir) {
        self.updates.push(crate::behaviour::UpdateEntry::new(vec![(
            pos,
            dir,
            crate::behaviour::UpdateKind::Shape,
        )]));
        self.propagate();
        self.updates.push(crate::behaviour::UpdateEntry::new(vec![(
            pos.offset(dir),
            dir.opposite(),
            crate::behaviour::UpdateKind::Shape,
        )]));
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
        if let Some(timeline) = self.timeline.as_mut() {
            timeline.inputs.push(crate::timeline::InputAction::PlaceBlock {
                tick: self.tick,
                pos,
                state,
            });
        }
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
        // `LevelChunk.setBlockState` runs `onPlace` before `markAndNotifyBlock`
        // reaches the neighbours — same as `TickCtx::set`, at the boundary.
        if let Some(behaviour) = self.behaviours.get(state) {
            // Field-disjoint borrows, exactly as `use_block` builds its ctx —
            // `self.ctx()` would reborrow `behaviours` mutably.
            let mut ctx = TickCtx {
                drain: Some(crate::behaviour::Drain {
                    pending: &mut self.pending,
                    unknown_seen: &mut self.unknown_seen,
                    upd_log: self.upd_log.as_mut(),
                    phase: self.phase,
                }),
                behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
                log: self.log.as_mut(),
            };
            behaviour.on_state_changed(&mut ctx, pos);
        }
        // markAndNotifyBlock, flag 3: the neighbour updates AND the shape pass
        // observers listen to. Sending only the neighbour half left every
        // observer blind to placed and removed blocks — the repro is
        // `a_placed_block_pulses_the_watching_observer`, and vanilla's own
        // record of the same write (`--at` uses setBlock flag 3) shows the
        // pulse. Queue both, drain once: the documented flag-3 order.
        self.updates
            .push(crate::behaviour::UpdateEntry::neighbors_at(pos));
        self.updates
            .push(crate::behaviour::UpdateEntry::neighbor_shapes(pos));
        self.propagate();
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
        if let Some(timeline) = self.timeline.as_mut() {
            timeline.inputs.push(crate::timeline::InputAction::UseBlock {
                tick: self.tick,
                pos,
            });
        }
        let state = self.world.get(pos);
        let Some(behaviour) = self.behaviours.get(state) else {
            if state != StateId::AIR {
                self.unknown_seen.push(state);
            }
            return;
        };
        let mut ctx = TickCtx {
            drain: Some(crate::behaviour::Drain {
                pending: &mut self.pending,
                unknown_seen: &mut self.unknown_seen,
                upd_log: self.upd_log.as_mut(),
                phase: self.phase,
            }),
            behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
            log: self.log.as_mut(),
        };
        behaviour.on_used(&mut ctx, pos);
        self.propagate();
    }

    /// Run one phase. `Some(stop)` aborts the tick.
    fn run_phase(&mut self, phase: Phase) -> Option<StopReason> {
        self.phase = Some(phase);
        if std::env::var_os("MC_TICK_TRACE_EVENTS").is_some() {
            eprintln!("[t{}] --- phase {}", self.tick, phase.name());
        }
        match phase {
            // Not simulated. Named and walked so the order stays structurally
            // right and filling one in never re-plumbs its neighbours.
            Phase::WorldBorder | Phase::Weather | Phase::Raids => None,

            // Random ticks live in chunk ticking. See `set_random_ticks` for
            // what is exact (the rate) and what is approximate (the cells).
            Phase::ChunkManager => {
                if self.random_tick_speed > 0 {
                    self.run_random_ticks();
                }
                None
            }

            Phase::BlockTicks => {
                // Drain first, then dispatch: a behaviour may schedule further
                // ticks, and those belong to a later tick rather than extending
                // this drain. Draining into a Vec is what keeps that boundary
                // sharp.
                let due = self.ticks.collect_due(self.tick);
                if std::env::var("MC_TICK_TRACE_SCHEDULE").is_ok() {
                    let names: Vec<String> = due
                        .iter()
                        .map(|e| {
                            let d = self.registry.descriptor(self.world.get(e.pos)).unwrap_or("?");
                            format!("{:?}{}", (e.pos.x, e.pos.y, e.pos.z), &d[10..d.len().min(28)])
                        })
                        .collect();
                    eprintln!("[t{}] block ticks: {}", self.tick, names.join(" | "));
                }
                for entry in due {
                    let state = self.world.get(entry.pos);
                    let Some(behaviour) = self.behaviours.get(state) else {
                        // Unregistered: record it rather than treating the block
                        // as inert. See BehaviourTable's module docs.
                        self.unknown_seen.push(state);
                        continue;
                    };
                    let mut ctx = TickCtx {
                        drain: Some(crate::behaviour::Drain {
                            pending: &mut self.pending,
                            unknown_seen: &mut self.unknown_seen,
                            upd_log: self.upd_log.as_mut(),
                            phase: self.phase,
                        }),
                        behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
                        log: self.log.as_mut(),
                    };
                    behaviour.on_scheduled_tick(&mut ctx, entry.pos);
                    self.propagate();
                    // Off the "will tick this tick" set only once it has run,
                    // so the ticks still queued behind it stay visible.
                    self.ticks.finished(entry.pos);
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
                        drain: Some(crate::behaviour::Drain {
                            pending: &mut self.pending,
                            unknown_seen: &mut self.unknown_seen,
                            upd_log: self.upd_log.as_mut(),
                            phase: self.phase,
                        }),
                        behaviours: Some(&self.behaviours),
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
                        commands: &self.commands,
                        command_powered: &mut self.command_powered,
                        tickers: &mut self.tickers,
                        item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
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
                        drain: Some(crate::behaviour::Drain {
                            pending: &mut self.pending,
                            unknown_seen: &mut self.unknown_seen,
                            upd_log: self.upd_log.as_mut(),
                            phase: self.phase,
                        }),
                        behaviours: Some(&self.behaviours),
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
                        commands: &self.commands,
                        command_powered: &mut self.command_powered,
                        tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                        inv_log: self.inv_log.as_mut(),
                        log: self.log.as_mut(),
                    };
                    behaviour.on_block_entity_tick(&mut ctx, pos);
                    self.propagate();
                }
                // Entities in the way of anything still in flight. A moving
                // piston's block-entity tick moves entities *before* the tick
                // on which it lands, so this runs alongside the landings and
                // not after them.
                self.displace_entities_by_pistons();
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
                // Insertion order, which is what `tickBlockEntities` walks: the
                // moving block entities tick in the order they were added to the
                // level, and `moveBlocks` creates them farthest-from-the-piston
                // first, head slot last.
                //
                // The order is observable, because each landing notifies its
                // neighbours. A column landing top-down tells the block above it
                // while everything below is still a `moving_piston` placeholder —
                // immovable — so a piston up there resolves *false* and queues
                // nothing. Landing bottom-up instead hands that piston a fully
                // solid column and it extends into a door vanilla leaves shut.
                //
                // A stable sort on the due tick alone preserves that order.
                due.sort_by_key(|m| m.resolve_on);
                // Landing order is this list's order, and this list's order is
                // `moveBlocks`': each stroke appends `to_push` reversed, so the
                // far end of a pushed line lands first. Both a blanket reverse
                // and a per-stroke reverse were measured against the corpus:
                // each makes BB's t34 piston race come out vanilla's way and
                // each breaks `door_4x4_vault_cycle` and `door_6x6_cycle` — so
                // the direction is not the variable. What differs is the
                // *sequence* `resolve_push` builds for some structures, which
                // is where a divergence from `getToPush` has to be chased.
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
                    if std::env::var_os("MC_TICK_TRACE_EVENTS").is_some() {
                        eprintln!(
                            "[t{}] land   ({}, {}, {}) -> {}",
                            self.tick,
                            entry.pos.x,
                            entry.pos.y,
                            entry.pos.z,
                            self.registry.descriptor(entry.state).unwrap_or_default()
                        );
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
                    self.updates
                        .push(crate::behaviour::UpdateEntry::own_shapes(entry.pos));
                    // Drained per landing, not once at the end. Each moving
                    // block entity's tick completes — shapes, onPlace, neighbour
                    // updates — before the next one starts, so a neighbour woken
                    // by the first landing sees the rest of the column still in
                    // flight. Draining once at the end showed it an already-solid
                    // column instead, and a piston above it extended into a door
                    // vanilla leaves shut.
                    self.propagate();
                    landed.push(entry.pos);
                // `onPlace` for each landed block, *after* that block's own
                // landing updates have run but still inside its own landing.
                //
                // Ordering matters twice over. Within one landing it comes
                // last: vanilla's shape update reaches a moved observer while
                // it still carries its mid-pulse powered state, so it does not
                // re-schedule, and only then does onPlace clear the flag.
                // Dispatching it first would start a pulse vanilla never
                // starts.
                //
                // Across landings it must not be batched to the end. `onPlace`
                // is where a landed piston runs `checkIfExtend`, and that only
                // queues a block event if the push actually resolves — so a
                // piston must see the column above it as vanilla does, still
                // holding `moving_piston` placeholders that have not landed
                // yet. Immovable, so the push fails and no event is queued.
                // Running every onPlace after every landing showed it a solid
                // column instead, and the 6x6 door fired a piston vanilla
                // leaves alone.
                for pos in std::mem::take(&mut landed) {
                    let state = self.world.get(pos);
                    let Some(behaviour) = self.behaviours.get(state) else {
                        if state != StateId::AIR {
                            self.unknown_seen.push(state);
                        }
                        continue;
                    };
                    let mut ctx = TickCtx {
                        drain: Some(crate::behaviour::Drain {
                            pending: &mut self.pending,
                            unknown_seen: &mut self.unknown_seen,
                            upd_log: self.upd_log.as_mut(),
                            phase: self.phase,
                        }),
                        behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
                        log: self.log.as_mut(),
                    };
                    behaviour.on_placed(&mut ctx, pos);
                    self.propagate();
                    }
                    // Only now `markAndNotifyBlock`. The write is
                    // `setBlock(pos, state, 3)`, and flag 3 leaves
                    // `UPDATE_KNOWN_SHAPE` clear, so the neighbour updates run
                    // and then the shape pass over the same six neighbours.
                    // Both come *after* onPlace, because onPlace happens inside
                    // `LevelChunk.setBlockState` and `markAndNotifyBlock` after
                    // it returns — which is how a landed piston queues its own
                    // extend before it wakes the piston underneath it.
                    self.updates
                        .push(crate::behaviour::UpdateEntry::neighbors_at(entry.pos));
                    self.propagate();
                    self.updates
                        .push(crate::behaviour::UpdateEntry::neighbor_shapes(entry.pos));
                    self.propagate();
                    // `finalTick` ends with `level.neighborChanged(pos, block,
                    // null)` — the landed block hearing a neighbour update at
                    // its *own* position, after everything else. Vanilla passes
                    // no orientation; nothing that reacts to this reads the
                    // direction, so Down stands in for the absent one.
                    self.updates.push(crate::behaviour::UpdateEntry::new(vec![(
                        entry.pos,
                        crate::pos::Dir::Down,
                        crate::behaviour::UpdateKind::Neighbor,
                    )]));
                    crate::behaviour::trace_update("neighbor", entry.pos);
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
                    rails: &self.rails,
                    conductors: &self.conductors,
                };
                // Carts tick before items here. Vanilla interleaves by entity
                // list order; nothing implemented couples the two yet, so the
                // simplification is invisible — documented in case it stops
                // being.
                for index in 0..self.minecarts.len() {
                    if self.minecarts[index].removed {
                        continue;
                    }
                    // Rebuilt per cart rather than once per tick, because a
                    // rider's box is derived from its vehicle and a vehicle
                    // that ticked earlier in this very loop has already moved.
                    // Vanilla reads live positions; so does this.
                    let bodies = self.cart_obstacle_bodies();
                    crate::minecart::tick_minecart_among(
                        &mut self.minecarts,
                        index,
                        &collision,
                        &bodies,
                    );
                    // The push half of the same cart's tick, in the same pass:
                    // a cart shoves its neighbours the moment it has finished
                    // moving, before the next cart ticks. That interleaving is
                    // observable — in `cart_collide` the pushed cart travels on
                    // its own tick, later the same tick it did the pushing.
                    let _ = crate::minecart::push_neighbours(&mut self.minecarts, index);
                }
                // A hopper cart's own pull, part of the same entity pass —
                // vanilla runs it inside `MinecartHopper.tick`, right after
                // the movement above. The item loop below rebuilds its
                // collision view because this can write the world's
                // inventories (never its blocks).
                drop(collision);
                self.tick_hopper_carts();
                self.tick_tnt_and_bodies();
                let collision = SimCollision {
                    world: &self.world,
                    solidity: &self.solidity,
                    frictions: &self.frictions,
                    heights: &self.heights,
                    webs: &self.webs,
                    water_kinds: &self.water_kinds,
                    bubble_kinds: &self.bubble_kinds,
                    rails: &self.rails,
                    conductors: &self.conductors,
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
                // Entities summoned this tick materialise now — they first
                // *act* next tick, which is when vanilla's entity list picks
                // up a mid-tick addition.
                self.drain_pending_spawns();
                // The carts have finished moving, so the box view behaviours
                // read has to catch up before entityInside runs.
                self.refresh_bodies();
                // entityInside: every cell an entity overlaps hears about it —
                // how a wooden pressure plate notices the item on it, and how a
                // detector rail notices the cart.
                let mut cells: Vec<Pos> = Vec::new();
                let mut boxes: Vec<([f64; 3], [f64; 3])> = Vec::new();
                for item in &self.item_entities.items {
                    if item.removed {
                        continue;
                    }
                    boxes.push(crate::entity::item_aabb(item.pos));
                }
                for body in &self.item_entities.others {
                    boxes.push((body.min, body.max));
                }
                for (emin, emax) in boxes {
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
                        drain: Some(crate::behaviour::Drain {
                            pending: &mut self.pending,
                            unknown_seen: &mut self.unknown_seen,
                            upd_log: self.upd_log.as_mut(),
                            phase: self.phase,
                        }),
                        behaviours: Some(&self.behaviours),
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
                        commands: &self.commands,
                        command_powered: &mut self.command_powered,
                        tickers: &mut self.tickers,
                        item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
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
        // `ServerLevel.runBlockEvents` reschedules only what it never *tried*:
        //
        //     if (shouldTickBlocksAt(pos)) { doBlockEvent(data); }
        //     else { blockEventsToReschedule.add(data); }
        //
        // An event whose handler returns false — a piston that has lost its
        // power, or whose position no longer holds the block the event names —
        // is simply dropped. Re-queueing those instead gave a piston a second
        // attempt a tick later, and a retracting piston re-extended into a door
        // that vanilla leaves shut.
        //
        // Everything this engine simulates is inside a ticking region, so
        // nothing is rescheduled at all; the distinction is recorded here for
        // when ticking bounds start suspending block events too.
        let mut refused: Vec<BlockEvent> = Vec::new();
        self.drain_block_events(&mut refused)
    }

    fn drain_block_events(&mut self, refused: &mut Vec<BlockEvent>) -> Option<StopReason> {
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
                // `doBlockEvent`: an event whose position no longer holds the
                // block it was queued for is refused and dropped. Properties may
                // differ; only the block has to match.
                let trace_events = std::env::var("MC_TICK_TRACE_EVENTS").is_ok();
                if !self.registry.same_block(state, event.block) {
                    if trace_events {
                        eprintln!(
                            "[t{}] refuse(identity) {:?} id={} want={} have={}",
                            self.tick,
                            (event.pos.x, event.pos.y, event.pos.z),
                            event.id,
                            self.registry.descriptor(event.block).unwrap_or("?"),
                            self.registry.descriptor(state).unwrap_or("?")
                        );
                    }
                    refused.push(event);
                    continue;
                }
                let Some(behaviour) = self.behaviours.get(state) else {
                    self.unknown_seen.push(state);
                    continue;
                };
                let piston = self.timeline.as_ref().and_then(|_| {
                    crate::timeline::piston_event(
                        self.tick,
                        event,
                        self.registry.descriptor(state).unwrap_or_default(),
                        self.log.as_ref().map_or(0, Vec::len),
                    )
                });
                let mut ctx = TickCtx {
                    drain: Some(crate::behaviour::Drain {
                        pending: &mut self.pending,
                        unknown_seen: &mut self.unknown_seen,
                        upd_log: self.upd_log.as_mut(),
                        phase: self.phase,
                    }),
                    behaviours: Some(&self.behaviours),
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
                commands: &self.commands,
                command_powered: &mut self.command_powered,
                tickers: &mut self.tickers,
                item_entities: &mut self.item_entities,
                minecarts: &mut self.minecarts,
                conductors: &self.conductors,
                inv_log: self.inv_log.as_mut(),
                        log: self.log.as_mut(),
                };
                // Logged *before* dispatch, because the game logs its own at
                // `removeFirst` — before `doBlockEvent` runs. Logging after
                // instead puts every notification the event causes ahead of the
                // line announcing it, and the two traces stop being comparable
                // exactly where a divergence hunt needs them to be. That cost an
                // hour of chasing a divergence that was only in the logging.
                if trace_events {
                    eprintln!(
                        "[t{}] run    {:?} id={} on {}",
                        self.tick,
                        (event.pos.x, event.pos.y, event.pos.z),
                        event.id,
                        self.registry.descriptor(state).unwrap_or("?")
                    );
                }
                let handled = behaviour.on_block_event(&mut ctx, event.pos, event.id, event.param);
                if trace_events && !handled {
                    eprintln!(
                        "[t{}] refuse {:?} id={}",
                        self.tick,
                        (event.pos.x, event.pos.y, event.pos.z),
                        event.id
                    );
                }
                if !handled {
                    refused.push(event);
                }
                if handled {
                    if let (Some(timeline), Some(piston)) = (self.timeline.as_mut(), piston) {
                        timeline.pistons.push(piston);
                    }
                }
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

    /// Switching the recorder off must not destroy what it recorded.
    ///
    /// The propagation view records a cycle, then runs an unwatched settle
    /// with the recorder off before reading the log — under the old
    /// behaviour that read back empty, silently.
    #[test]
    fn stopping_the_update_recorder_keeps_the_log() {
        let mut s = sim();
        s.record_updates(true);
        let stone = s.registry_mut().intern("minecraft:stone").unwrap();
        s.place_block(Pos::new(1, 1, 1), stone);
        s.run(2);
        let recorded = s.recorded_updates().len();
        assert!(recorded > 0, "nothing was recorded, so the test proves nothing");

        s.record_updates(false);
        assert_eq!(s.recorded_updates().len(), recorded, "the log was dropped");

        // ...and the recorder really is off: further placements add nothing.
        s.place_block(Pos::new(2, 1, 1), stone);
        s.run(2);
        assert_eq!(s.recorded_updates().len(), recorded);

        s.clear_updates();
        assert!(s.recorded_updates().is_empty());
    }

    #[test]
    fn restarting_the_recorder_starts_a_fresh_log() {
        let mut s = sim();
        s.record_updates(true);
        let stone = s.registry_mut().intern("minecraft:stone").unwrap();
        s.place_block(Pos::new(1, 1, 1), stone);
        s.run(2);
        assert!(!s.recorded_updates().is_empty());

        s.record_updates(false);
        s.record_updates(true);
        assert!(s.recorded_updates().is_empty(), "a stale log survived a restart");
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
        assert!(s.ticks.is_pending(Pos::new(2, 2, 2)));
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
                    contents: None,
                }],
                blocked_slots: 0,
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

    /// `boat_fence.litematic` saves a leashed boat at tether rest with a small
    /// upward correction rather than zero Motion. That exact bounded state is
    /// a stationary hard collider; removing the leash or adding real travel
    /// must still refuse rather than broadening this into guessed boat physics.
    #[test]
    fn a_parked_leashed_boat_accepts_only_the_measured_rest_residue() {
        let mut s = sim();
        let fixture = crate::structure::SpawnedBody {
            kind: "minecraft:oak_boat".to_string(),
            pos: [1.488_202_109_234_411, 0.045_043_285_781_332_54, 2.812_601_257_115_602_5],
            motion: [
                -4.323_888_315_084_626_6e-16,
                0.011_681_267_954_871_115,
                -3.611_030_807_389_778_4e-85,
            ],
            leashed: true,
            passengers: Vec::new(),
        };

        let id = s
            .spawn_authored_body(&fixture)
            .expect("the supplied tether-rest boat is supported");
        let boat = s.entity_bodies().iter().find(|body| body.id == id).expect("spawned body");
        assert_eq!(boat.kind, "minecraft:oak_boat");
        assert_eq!(
            [(boat.min[0] + boat.max[0]) / 2.0, boat.min[1], (boat.min[2] + boat.max[2]) / 2.0],
            fixture.pos,
            "the authored tether-rest pose is the frozen collision pose"
        );

        let mut free = fixture.clone();
        free.leashed = false;
        assert!(
            s.spawn_authored_body(&free).expect_err("a free moving boat has no physics").contains(
                "has Motion"
            )
        );

        let mut swinging = fixture;
        swinging.motion[0] = 0.001;
        assert!(
            s.spawn_authored_body(&swinging)
                .expect_err("horizontal boat travel is not a tether-rest residue")
                .contains("has Motion")
        );
    }

    /// The tether survives into the public entity view.
    ///
    /// A leashed boat and a boat parked on the ground are the same box, so a
    /// viewer that only reads the box draws the second when it should draw the
    /// first. The flag was previously consumed by `supported_parked_boat` and
    /// dropped, which made the distinction unobservable from outside.
    #[test]
    fn a_spawned_boat_reports_whether_it_is_leashed() {
        let mut s = sim();
        let tethered = crate::structure::SpawnedBody {
            // Pale oak because it is the one boat with a measured seat, and
            // this test needs a vehicle that can actually carry a passenger.
            kind: "minecraft:pale_oak_boat".to_string(),
            pos: [1.5, 0.0, 2.5],
            motion: [0.0; 3],
            leashed: true,
            passengers: Vec::new(),
        };
        let mut grounded = tethered.clone();
        grounded.pos = [4.5, 0.0, 2.5];
        grounded.leashed = false;

        let tethered_id = s.spawn_authored_body(&tethered).expect("a still boat is supported");
        let grounded_id = s.spawn_authored_body(&grounded).expect("a still boat is supported");

        let leashed_of = |s: &Simulation, id: u32| {
            s.entity_bodies().iter().find(|body| body.id == id).expect("spawned body").leashed
        };
        assert!(leashed_of(&s, tethered_id), "the leash tag was dropped on the way out");
        assert!(!leashed_of(&s, grounded_id), "an unleashed boat must not claim a tether");

        // A rider is held by its seat; the flag must not leak from the vehicle.
        let passenger = crate::structure::SpawnedEntity::Body(crate::structure::SpawnedBody {
            kind: "minecraft:silverfish".to_string(),
            pos: tethered.pos,
            motion: [0.0; 3],
            leashed: false,
            passengers: Vec::new(),
        });
        let rider =
            s.spawn_authored_rider(tethered_id, &passenger).expect("a boat seats a passenger");
        assert!(!leashed_of(&s, rider), "a passenger is not itself on a leash");
        assert!(leashed_of(&s, tethered_id), "seating a rider dropped the vehicle's tether");
    }

    /// The decorated elevator's vehicle/rider pair is admitted only in the
    /// measured settled envelope, and the silverfish is seated from the boat
    /// rather than from Litematica's stale source-world passenger position.
    #[test]
    fn the_elevator_boat_seats_its_silverfish_and_rejects_real_travel() {
        let mut s = sim();
        let rider = crate::structure::SpawnedEntity::Body(crate::structure::SpawnedBody {
            kind: "minecraft:silverfish".to_string(),
            pos: [-103.3125, 4.25, 128.748_073_537_581_1],
            motion: [
                -0.004_550_000_029_429_79,
                -0.078_400_001_525_878_9,
                4.362_258_015_156_135e-7,
            ],
            leashed: false,
            passengers: Vec::new(),
        });
        let fixture = crate::structure::SpawnedBody {
            kind: "minecraft:pale_oak_boat".to_string(),
            pos: [5.6875, 61.0625, 16.748_073_537_581_092],
            motion: [-5e-324, 0.0, -5.059_650_645_427_016e-6],
            leashed: true,
            passengers: vec![rider.clone()],
        };

        let boat = s.spawn_authored_body(&fixture).expect("the tether-rest boat is supported");
        let silverfish = s
            .spawn_authored_rider(boat, &rider)
            .expect("the vanilla-measured pale-oak boat seat is supported");
        let body = s
            .entity_bodies()
            .iter()
            .find(|body| body.id == silverfish)
            .expect("silverfish body");
        assert_eq!(
            [(body.min[0] + body.max[0]) / 2.0, body.min[1], (body.min[2] + body.max[2]) / 2.0],
            [fixture.pos[0], fixture.pos[1] + 0.1875, fixture.pos[2]]
        );

        let mut root = fixture.clone();
        root.leashed = false;
        root.motion = [0.0, 0.0, -2e-323];
        s.spawn_authored_body(&root)
            .expect("an elevator chain root contains floating-point dust only");

        let mut riderless = fixture.clone();
        riderless.passengers.clear();
        assert!(
            s.spawn_authored_body(&riderless)
                .expect_err("the passenger relationship identifies the elevator fixture")
                .contains("has Motion")
        );

        let mut drifting = fixture;
        drifting.motion[2] = 1.0 / 2048.0;
        assert!(
            s.spawn_authored_body(&drifting)
                .expect_err("a boat outside the captured tether-rest envelope must refuse")
                .contains("has Motion")
        );
    }

    /// A rider's box is its vehicle's position plus the seat, always.
    ///
    /// This is `Entity.positionRider`, and the assertion is that the rider has
    /// no position of its own to drift with: move the cart by hand and the box
    /// has already followed. The frozen blaze beside it is the control — it is
    /// the *same entity type* with the same box, seated at the same place, and
    /// it must **not** move when the cart does. Without it this test would pass
    /// on an engine that simply never moves anything.
    #[test]
    fn a_rider_follows_its_vehicle_and_a_frozen_body_does_not() {
        let mut s = sim();
        let cart = s.spawn_minecart("minecraft:minecart".into(), [4.0, 2.0625, 8.0], [0.0; 3]);
        let rider = s
            .spawn_authored_rider(
                cart,
                &crate::structure::SpawnedEntity::Body(crate::structure::SpawnedBody {
                    kind: "minecraft:blaze".to_string(),
                    // Deliberately nonsense, and deliberately ignored: a rider is
                    // seated from its vehicle, not from the file.
                    pos: [999.0, 999.0, 999.0],
                    motion: [0.0, -0.0784, 0.0],
                    leashed: false,
                    passengers: Vec::new(),
                }),
            )
            .expect("a blaze on a plain minecart is measured");
        let frozen = s
            .spawn_frozen_entity("minecraft:blaze".into(), [4.0, 2.25, 8.0])
            .expect("a free-standing blaze is a hitbox");

        let seat_of = |s: &Simulation, id: u32| {
            s.entity_bodies().iter().find(|b| b.id == id).expect("in the box view").min
        };
        // Half the *float* width the registry holds, which is not 0.3.
        let half = (0.6_f32 as f64) / 2.0;
        assert_eq!(seat_of(&s, rider), [4.0 - half, 2.25, 8.0 - half]);
        assert_eq!(seat_of(&s, frozen), [4.0 - half, 2.25, 8.0 - half]);

        s.minecarts[0].pos = [4.5, 3.0625, 8.25];
        s.refresh_bodies();
        assert_eq!(
            seat_of(&s, rider),
            [4.5 - half, 3.25, 8.25 - half],
            "the rider is re-seated from the vehicle every time anything moves"
        );
        assert_eq!(
            seat_of(&s, frozen),
            [4.0 - half, 2.25, 8.0 - half],
            "the control: a body that is not riding must stay where it was, or the \
             assertion above is measuring a refresh that moves everything"
        );

        // And a rider whose vehicle leaves the world leaves with it.
        s.minecarts[0].removed = true;
        s.refresh_bodies();
        assert!(s.entity_bodies().iter().all(|b| b.id != rider));
        assert!(s.entity_bodies().iter().any(|b| b.id == frozen));
    }

    /// An unmeasured seat refuses instead of being approximated.
    #[test]
    fn a_rider_with_no_measured_seat_refuses_by_name() {
        let mut s = sim();
        let cart =
            s.spawn_minecart("minecraft:furnace_minecart".into(), [4.0, 2.0, 8.0], [0.0; 3]);
        let why = s
            .spawn_authored_rider(
                cart,
                &crate::structure::SpawnedEntity::Body(crate::structure::SpawnedBody {
                    kind: "minecraft:blaze".to_string(),
                    pos: [4.0, 2.2, 8.0],
                    motion: [0.0; 3],
                    leashed: false,
                    passengers: Vec::new(),
                }),
            )
            .expect_err("no furnace-cart seat has been measured");
        assert!(why.contains("minecraft:furnace_minecart"), "{why}");
        assert!(why.contains("0.1875"), "the refusal cites what was measured: {why}");
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
