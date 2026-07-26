//! Per-block behaviour, and the registry that dispatches to it.
//!
//! # The shape of a behaviour
//!
//! [`BlockBehaviour`] is **synchronous**. That is a deliberate constraint rather
//! than an omission: the whole product is a simulation you can single-step,
//! checkpoint and rewind, and an async behaviour would mean a block could be
//! mid-await when a checkpoint is taken. Determinism and steppability come first.
//!
//! Every method has a default, so adding a block means implementing only what
//! that block actually does. A pressure plate that never reacts to a neighbour
//! does not need to say so.
//!
//! # Unknown blocks fail loudly
//!
//! The registry does **not** silently treat unregistered blocks as inert. This
//! project grows behaviour incrementally, and the failure mode that would quietly
//! ruin it is a contraption containing one unimplemented component that simulates
//! as air, produces a plausible-looking answer, and is wrong. So an encounter with
//! an unregistered block is recorded and surfaced — see
//! [`BehaviourTable::unknown`].
//!
//! Air is the sole exception, registered as genuinely inert.

use crate::pos::{Dir, Pos};
use crate::schedule::{BlockEvent, EventQueue, TickPriority, TickQueue};
use crate::state::{StateId, StateRegistry};
use crate::world::World;
use std::collections::BTreeSet;

/// One recorded block change, for comparison against a captured vanilla trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockChange {
    /// The tick it happened on.
    pub tick: u64,
    /// Where.
    pub pos: Pos,
    /// State before.
    pub from: StateId,
    /// State after.
    pub to: StateId,
}

/// One recorded container-slot change, the inventory analog of [`BlockChange`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryChange {
    /// The tick it happened on.
    pub tick: u64,
    /// The container.
    pub pos: Pos,
    /// Which slot.
    pub slot: u8,
    /// `(item id, count)` before, `None` for empty.
    pub from: Option<(String, u8)>,
    /// `(item id, count)` after.
    pub to: Option<(String, u8)>,
}

/// Per-hopper transfer state — vanilla's `HopperBlockEntity` fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HopperState {
    /// `cooldownTime`: decremented each block-entity tick; transfers only run
    /// at zero or below.
    pub cooldown: i32,
    /// `tickedGameTime`: the last tick this hopper's block entity ran, signed
    /// so a hopper that has **never** ticked compares before tick 0. The
    /// destination-cooldown rule compares these across two hoppers, and the
    /// distinction decides cooldown 7 versus 8 on the very first tick.
    pub ticked_at: i64,
}

impl Default for HopperState {
    fn default() -> Self {
        Self { cooldown: 0, ticked_at: -1 }
    }
}

/// A block write scheduled to land in a later tick's block-entities phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingMove {
    /// Where the write lands.
    pub pos: Pos,
    /// What it becomes.
    pub state: StateId,
    /// The tick whose block-entities phase applies it.
    pub resolve_on: u64,
}

/// What a behaviour is given when it runs.
///
/// Holds the world and the queues, but deliberately **not** the behaviour table:
/// a behaviour that could dispatch into other behaviours re-entrantly would make
/// ordering depend on call depth rather than on the phase, which is exactly what
/// this engine exists to get right.
pub struct TickCtx<'a> {
    /// Block storage.
    pub world: &'a mut World,
    /// Scheduled block ticks.
    pub ticks: &'a mut TickQueue,
    /// Block events for this tick.
    pub events: &'a mut EventQueue,
    /// State descriptors, for behaviours that need to inspect a block by name.
    pub states: &'a StateRegistry,
    /// The tick currently being executed.
    pub tick: u64,
    /// Whether this dispatch is happening *between* ticks rather than inside one.
    ///
    /// Structure placement, block breaks and player clicks all happen in the
    /// server loop, outside `ServerLevel.tick` — at a moment when the game time
    /// still reads the last *completed* tick. A tick scheduled from there fires
    /// one tick sooner than the same schedule made inside a phase: captured with
    /// an observer, whose placement-provoked pulse lands on tick 1, not tick 2.
    /// [`TickCtx::schedule`] folds this in so behaviours never have to know when
    /// they are being called.
    pub boundary: bool,
    /// Deferred block writes, applied in the block-entities phase.
    ///
    /// Vanilla's moving pistons work this way: the block event replaces the moved
    /// blocks with `moving_piston` placeholders, and a block entity resolves them
    /// two ticks later. Modelling it as a deferred write puts the resolution in the
    /// same phase the game uses, so door timings come out right.
    pub moves: &'a mut Vec<PendingMove>,
    /// Block changes made this tick, when recording is on.
    ///
    /// Populated by [`TickCtx::set`] so a run can be compared against a trace
    /// captured from the real game. `None` when recording is off, which is the
    /// default — the tick loop should not pay for observability nobody asked for.
    pub log: Option<&'a mut Vec<BlockChange>>,
    /// Each comparator's last emitted output strength.
    ///
    /// Vanilla keeps this in a `ComparatorBlockEntity`, because the block *state*
    /// only carries `powered` and `mode` — it cannot express "I am on at strength
    /// 9". Comparator priming is exactly the consequence: a comparator schedules a
    /// tick when its output *strength* changes even though `powered` does not.
    pub comparator_out: &'a mut std::collections::HashMap<Pos, u8>,
    /// Container contents by position — vanilla's inventory block entities.
    ///
    /// What a comparator reads and a hopper moves. Kept with the simulation
    /// for the same reason as `comparator_out`: block states cannot express
    /// "27 slots holding 40 redstone". Mutate through
    /// [`TickCtx::set_inventory_slot`], which records the change and updates
    /// the comparators around the container.
    pub inventories: &'a mut std::collections::HashMap<Pos, crate::inventory::Inventory>,
    /// Per-hopper cooldown and tick bookkeeping.
    pub hopper_state: &'a mut std::collections::HashMap<Pos, HopperState>,
    /// The world's item entities — what hoppers vacuum and droppers eject.
    pub item_entities: &'a mut crate::entity::ItemEntities,
    /// Container-slot changes made this tick, when recording is on.
    pub inv_log: Option<&'a mut Vec<InventoryChange>>,
    /// Recent redstone-torch toggles, for burnout detection.
    ///
    /// Burnout is the one behaviour that depends on *history* rather than on the
    /// current world, and behaviours are shared and immutable — so the record lives
    /// with the simulation and is reached through here.
    pub toggles: &'a mut Vec<(Pos, u64)>,
    /// Neighbour notifications raised by [`TickCtx::set`], drained by the driver.
    ///
    /// Collected rather than dispatched inline: a behaviour that could re-enter
    /// other behaviours would make ordering depend on call depth instead of on the
    /// phase, which is the one thing this engine exists to get right.
    pub updates: &'a mut Vec<(Pos, Dir)>,
}

impl TickCtx<'_> {
    /// Schedule a block tick at `pos`.
    ///
    /// From a boundary dispatch the effective "now" is the last completed tick —
    /// `self.tick` here is the tick the change will be *observed* in, which is one
    /// later. See [`TickCtx::boundary`].
    pub fn schedule(&mut self, pos: Pos, delay: u64, priority: TickPriority) {
        // Folded into the delay rather than the tick so that a boundary schedule
        // before tick 0 still lands on tick `delay - 1` instead of underflowing.
        // A boundary delay of 0 stays 0: it would fire in the upcoming tick's
        // block-ticks phase either way.
        let delay = if self.boundary { delay.saturating_sub(1) } else { delay };
        self.ticks.schedule(pos, self.tick, delay, priority);
    }

    /// Queue a block event for this tick's block-events phase.
    pub fn queue_event(&mut self, pos: Pos, id: u8, param: u8) {
        self.events.push(BlockEvent { pos, id, param });
    }

    /// The state at `pos`.
    pub fn get(&self, pos: Pos) -> StateId {
        self.world.get(pos)
    }

    /// Set the state at `pos` and notify its six neighbours.
    ///
    /// Notifications are queued, not dispatched here; see [`TickCtx::updates`].
    /// Nothing is queued if the write changed nothing, which is what stops two
    /// blocks that keep re-asserting the same state from looping forever.
    pub fn set(&mut self, pos: Pos, state: StateId) {
        let previous = self.world.get(pos);
        if previous == state {
            return;
        }
        self.world.set(pos, state);
        if let Some(log) = self.log.as_deref_mut() {
            log.push(BlockChange { tick: self.tick, pos, from: previous, to: state });
        }
        for dir in crate::pos::ALL_DIRS {
            // The neighbour is told which way the change came from, relative to it.
            self.updates.push((pos.offset(dir), dir.opposite()));
        }
    }

    /// Set a block without notifying anything, for loading a structure.
    pub fn set_silent(&mut self, pos: Pos, state: StateId) {
        self.world.set(pos, state);
    }

    /// Set the state at `pos`, recording it, without notifying its neighbours.
    ///
    /// Vanilla's piston moves write their placeholders and vacated slots with
    /// update flags that suppress neighbour block updates (`324`, `82`, `68`) —
    /// which is why a piston does not react to its own move until the blocks
    /// land. The write is still real and a snapshot capture sees it, so it is
    /// logged; only the notifications are withheld.
    pub fn set_quiet(&mut self, pos: Pos, state: StateId) {
        let previous = self.world.get(pos);
        if previous == state {
            return;
        }
        self.world.set(pos, state);
        if let Some(log) = self.log.as_deref_mut() {
            log.push(BlockChange { tick: self.tick, pos, from: previous, to: state });
        }
    }

    /// The output strength a comparator at `pos` last emitted.
    pub fn stored_comparator_output(&self, pos: Pos) -> u8 {
        self.comparator_out.get(&pos).copied().unwrap_or(0)
    }

    /// Remember the output strength a comparator at `pos` is now emitting.
    pub fn store_comparator_output(&mut self, pos: Pos, strength: u8) {
        self.comparator_out.insert(pos, strength);
    }

    /// Record that a torch at `pos` toggled on this tick.
    pub fn record_toggle(&mut self, pos: Pos) {
        self.toggles.push((pos, self.tick));
    }

    /// How many times a torch at `pos` toggled within the last `window` ticks.
    pub fn recent_toggles(&self, pos: Pos, window: u64) -> usize {
        self.toggles
            .iter()
            .filter(|(p, t)| *p == pos && self.tick.saturating_sub(*t) < window)
            .count()
    }

    /// The `(item id, count)` in a container slot, `None` when empty.
    pub fn inventory_slot(&self, pos: Pos, slot: u8) -> Option<(String, u8)> {
        self.inventories.get(&pos).and_then(|inv| {
            inv.stacks
                .iter()
                .find(|stack| stack.slot == slot && stack.count > 0)
                .map(|stack| (stack.id.clone(), stack.count))
        })
    }

    /// Write a container slot, recording the change and updating comparators.
    ///
    /// The comparator update mirrors `Level.updateNeighbourForOutputSignal`,
    /// which a container's `setChanged` reaches: every neighbour of the
    /// container is notified, and when a neighbour is a conductor, the block
    /// beyond it too — that is how a comparator reading a container *through* a
    /// solid block notices the contents change.
    pub fn set_inventory_slot(&mut self, pos: Pos, slot: u8, to: Option<(String, u8)>) {
        let from = self.inventory_slot(pos, slot);
        if from == to {
            return;
        }
        let inventory = self
            .inventories
            .entry(pos)
            .or_insert_with(|| crate::inventory::Inventory::empty(0));
        inventory.stacks.retain(|stack| stack.slot != slot);
        if let Some((id, count)) = &to {
            inventory.stacks.push(crate::inventory::ItemStack {
                slot,
                id: id.clone(),
                count: *count,
            });
        }
        if let Some(log) = self.inv_log.as_deref_mut() {
            log.push(InventoryChange { tick: self.tick, pos, slot, from, to });
        }
        for dir in crate::pos::ALL_DIRS {
            self.updates.push((pos.offset(dir), dir.opposite()));
        }
    }

    /// Schedule a block write for `delay` ticks from now, resolved in the
    /// block-entities phase.
    pub fn defer(&mut self, pos: Pos, state: StateId, delay: u64) {
        self.moves.push(PendingMove {
            pos,
            state,
            resolve_on: self.tick + delay,
        });
    }
}

/// How one kind of block behaves.
pub trait BlockBehaviour: Send + Sync {
    /// A neighbour of `pos` changed, in direction `from` relative to `pos`.
    fn on_neighbor_changed(&self, _ctx: &mut TickCtx<'_>, _pos: Pos, _from: Dir) {}

    /// A scheduled block tick fired at `pos`.
    fn on_scheduled_tick(&self, _ctx: &mut TickCtx<'_>, _pos: Pos) {}

    /// A block event fired at `pos`. Returns whether it was handled.
    ///
    /// Vanilla's equivalent returns a boolean that decides whether the event
    /// counts as consumed, which matters for pistons.
    fn on_block_event(&self, _ctx: &mut TickCtx<'_>, _pos: Pos, _id: u8, _param: u8) -> bool {
        false
    }

    /// A player right-clicked this block with an empty hand.
    ///
    /// Mirrors vanilla's `useWithoutItem`. Most blocks do nothing, hence the
    /// default; a note block cycles its pitch, which is what makes a *manual*
    /// contraption manual.
    fn on_used(&self, _ctx: &mut TickCtx<'_>, _pos: Pos) {}

    /// This block's entity ticks every game tick in the block-entities phase.
    ///
    /// Only hoppers so far. The dispatch order is the ticker registration
    /// order — vanilla's `tickBlockEntities` walks its list in insertion order,
    /// which for a placed structure is block order.
    fn on_block_entity_tick(&self, _ctx: &mut TickCtx<'_>, _pos: Pos) {}

    /// Whether this block registers a ticking block entity.
    fn ticks_as_block_entity(&self) -> bool {
        false
    }

    /// This block was just written into the world by a completed piston move.
    ///
    /// Mirrors the `onPlace` a landed block receives from vanilla's `setBlock`.
    /// The one behaviour that needs it so far: an observer that lands still
    /// mid-pulse, with its turn-off tick stranded at its old position, clears
    /// its own powered flag (captured: `flying_machine.json`, tick 3).
    fn on_placed(&self, _ctx: &mut TickCtx<'_>, _pos: Pos) {}

    /// Redstone power this block emits toward `dir`, 0-15.
    fn redstone_power(&self, _world: &World, _pos: Pos, _dir: Dir) -> u8 {
        0
    }

    /// A short name for traces and diagnostics.
    fn name(&self) -> &'static str;
}

/// A block that does nothing at all.
///
/// Used for air, and for genuinely inert building blocks once they are known to
/// be inert. Registering something here is an assertion that it has no
/// behaviour — not a shortcut for "not implemented yet", which is what
/// [`BehaviourTable::unknown`] is for.
#[derive(Debug, Clone, Copy)]
pub struct Inert(&'static str);

impl Inert {
    /// An inert behaviour reported under `name`.
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }
}

impl BlockBehaviour for Inert {
    fn name(&self) -> &'static str {
        self.0
    }
}

/// Dispatch from a [`StateId`] to its behaviour.
///
/// Lookup is a flat `Vec` index rather than a map: it happens on every event of
/// every tick, and this is the hot path.
pub struct BehaviourTable {
    /// Indexed by `StateId`; `None` means unregistered.
    entries: Vec<Option<Box<dyn BlockBehaviour>>>,
    /// States seen with no behaviour, in a deterministic order.
    ///
    /// Interior mutability is avoided on purpose — recording goes through
    /// `&mut self` at registration/report time, so the hot dispatch path stays a
    /// plain immutable index.
    unknown: BTreeSet<StateId>,
}

impl BehaviourTable {
    /// A table with air registered inert and nothing else.
    pub fn new() -> Self {
        let mut table = Self {
            entries: Vec::new(),
            unknown: BTreeSet::new(),
        };
        table.register(StateId::AIR, Box::new(Inert::new("air")));
        table
    }

    /// Register `behaviour` for `state`, replacing any previous entry.
    pub fn register(&mut self, state: StateId, behaviour: Box<dyn BlockBehaviour>) {
        let index = state.raw() as usize;
        if self.entries.len() <= index {
            self.entries.resize_with(index + 1, || None);
        }
        self.entries[index] = Some(behaviour);
        self.unknown.remove(&state);
    }

    /// The behaviour for `state`, if registered.
    pub fn get(&self, state: StateId) -> Option<&dyn BlockBehaviour> {
        self.entries
            .get(state.raw() as usize)
            .and_then(|slot| slot.as_deref())
    }

    /// Whether `state` has a behaviour.
    pub fn is_registered(&self, state: StateId) -> bool {
        self.get(state).is_some()
    }

    /// Record that `state` was encountered with no behaviour.
    pub fn note_unknown(&mut self, state: StateId) {
        if !self.is_registered(state) {
            self.unknown.insert(state);
        }
    }

    /// Scan a world and record every unregistered state in it.
    ///
    /// Call this after loading a structure and before trusting any result. It is
    /// the difference between "this contraption simulated correctly" and "this
    /// contraption contained three components we do not implement".
    pub fn note_unknown_in(&mut self, world: &World) {
        let seen: Vec<StateId> = world
            .iter_non_air()
            .map(|(_, state)| state)
            .filter(|state| !self.is_registered(*state))
            .collect();
        for state in seen {
            self.unknown.insert(state);
        }
    }

    /// Every state encountered without a behaviour, ascending.
    pub fn unknown(&self) -> impl Iterator<Item = StateId> + '_ {
        self.unknown.iter().copied()
    }

    /// Whether anything unregistered has been encountered.
    pub fn has_unknown(&self) -> bool {
        !self.unknown.is_empty()
    }

    /// A human-readable report of unregistered states, or `None` if clean.
    ///
    /// Resolved through `states` so the message names blocks rather than
    /// integers — `minecraft:observer` is actionable, `StateId(37)` is not.
    pub fn unknown_report(&self, states: &StateRegistry) -> Option<String> {
        if self.unknown.is_empty() {
            return None;
        }
        let mut names: Vec<&str> = self
            .unknown
            .iter()
            .map(|state| states.descriptor(*state).unwrap_or("<unregistered state>"))
            .collect();
        names.sort_unstable();
        Some(format!(
            "{} block state(s) have no behaviour and were simulated as nothing: {}",
            names.len(),
            names.join(", ")
        ))
    }

    /// How many states have behaviours.
    pub fn len(&self) -> usize {
        self.entries.iter().filter(|slot| slot.is_some()).count()
    }

    /// Whether only air is registered.
    pub fn is_empty(&self) -> bool {
        self.len() <= 1
    }
}

impl Default for BehaviourTable {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for BehaviourTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BehaviourTable")
            .field("registered", &self.len())
            .field("unknown", &self.unknown.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos::Bounds;

    /// A block that emits a fixed power in every direction.
    struct Source(u8);
    impl BlockBehaviour for Source {
        fn redstone_power(&self, _world: &World, _pos: Pos, _dir: Dir) -> u8 {
            self.0
        }
        fn name(&self) -> &'static str {
            "source"
        }
    }

    #[test]
    fn air_is_registered_inert_from_the_start() {
        let table = BehaviourTable::new();
        assert!(table.is_registered(StateId::AIR));
        assert_eq!(table.get(StateId::AIR).unwrap().name(), "air");
        assert!(!table.has_unknown());
    }

    #[test]
    fn registration_and_lookup_round_trip() {
        let mut table = BehaviourTable::new();
        table.register(StateId(5), Box::new(Source(15)));
        let behaviour = table.get(StateId(5)).expect("registered");
        assert_eq!(behaviour.name(), "source");
        assert_eq!(
            behaviour.redstone_power(&World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(1, 1, 1))), Pos::new(0, 0, 0), Dir::Up),
            15
        );
    }

    #[test]
    fn unregistered_states_are_recorded_not_silently_inert() {
        // The failure this project must never tolerate: an unimplemented block
        // simulating as nothing and yielding a plausible, wrong answer.
        let mut table = BehaviourTable::new();
        assert!(table.get(StateId(9)).is_none());
        table.note_unknown(StateId(9));
        assert!(table.has_unknown());
        assert_eq!(table.unknown().collect::<Vec<_>>(), vec![StateId(9)]);
    }

    #[test]
    fn registering_clears_a_previously_unknown_state() {
        let mut table = BehaviourTable::new();
        table.note_unknown(StateId(9));
        assert!(table.has_unknown());
        table.register(StateId(9), Box::new(Source(1)));
        assert!(!table.has_unknown(), "registering must resolve the gap");
    }

    #[test]
    fn scanning_a_world_finds_every_unimplemented_block() {
        let mut states = StateRegistry::new();
        let stone = states.intern("minecraft:stone").unwrap();
        let observer = states.intern("minecraft:observer[facing=up]").unwrap();

        let mut world = World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(7, 7, 7)));
        world.set(Pos::new(1, 1, 1), stone);
        world.set(Pos::new(2, 1, 1), observer);

        let mut table = BehaviourTable::new();
        table.register(stone, Box::new(Inert::new("stone")));
        table.note_unknown_in(&world);

        let report = table.unknown_report(&states).expect("observer is unhandled");
        assert!(report.contains("minecraft:observer[facing=up]"), "{report}");
        assert!(!report.contains("minecraft:stone"), "{report}");
    }

    #[test]
    fn a_clean_table_reports_nothing() {
        let mut states = StateRegistry::new();
        let stone = states.intern("minecraft:stone").unwrap();
        let world = World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(3, 3, 3)));

        let mut table = BehaviourTable::new();
        table.register(stone, Box::new(Inert::new("stone")));
        table.note_unknown_in(&world);
        assert_eq!(table.unknown_report(&states), None);
    }

    #[test]
    fn unknown_report_names_blocks_not_integers() {
        // A report saying StateId(37) is not actionable; a name is.
        let mut states = StateRegistry::new();
        let piston = states.intern("minecraft:sticky_piston[facing=east]").unwrap();
        let mut table = BehaviourTable::new();
        table.note_unknown(piston);
        let report = table.unknown_report(&states).unwrap();
        assert!(report.contains("minecraft:sticky_piston"), "{report}");
    }

    #[test]
    fn default_methods_mean_a_block_implements_only_what_it_does() {
        // Source implements only redstone_power; everything else must be a no-op
        // rather than a compile error or a panic.
        let source = Source(7);
        let mut world = World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(3, 3, 3)));
        let mut ticks = TickQueue::new();
        let mut events = EventQueue::new();
        let states = StateRegistry::new();
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            events: &mut events,
            states: &states,
            tick: 0,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        source.on_neighbor_changed(&mut ctx, Pos::new(0, 0, 0), Dir::Up);
        source.on_scheduled_tick(&mut ctx, Pos::new(0, 0, 0));
        assert!(!source.on_block_event(&mut ctx, Pos::new(0, 0, 0), 0, 0));
        assert!(ctx.ticks.is_empty() && ctx.events.is_empty());
    }

    #[test]
    fn ctx_schedule_and_event_helpers_reach_the_queues() {
        let mut world = World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(3, 3, 3)));
        let mut ticks = TickQueue::new();
        let mut events = EventQueue::new();
        let states = StateRegistry::new();
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            events: &mut events,
            states: &states,
            tick: 10,
            boundary: false,
            updates: &mut Vec::new(),
            moves: &mut Vec::new(),
            toggles: &mut Vec::new(),
            comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        ctx.schedule(Pos::new(1, 1, 1), 2, TickPriority::High);
        ctx.queue_event(Pos::new(2, 2, 2), 1, 3);

        assert_eq!(ticks.next_due(), Some(12), "delay is relative to ctx.tick");
        assert_eq!(events.len(), 1);
    }
}
