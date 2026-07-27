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

/// Each comparator's last emitted output strength, by position — vanilla's
/// `ComparatorBlockEntity.outputSignal`.
///
/// Emitted power genuinely depends on this: a comparator whose block state
/// says `powered=true` but whose (freshly placed) block entity still holds 0
/// emits **nothing**, which is exactly what a community door's placement
/// looks like before its first comparator tick.
pub type ComparatorOutputs = std::collections::HashMap<Pos, u8>;

/// Which of vanilla's two update callbacks a queued notification carries.
///
/// `Level.setBlock` sends **both**: `updateNeighborsAt` (→ `neighborChanged`,
/// how a piston notices power) and `updateNeighbourShapes` (→ `updateShape`,
/// how an **observer** notices a change — `ObserverBlock` overrides only
/// `updateShape`, and that is exactly why placement pulses observers while
/// leaving pistons alone).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateKind {
    /// `neighborChanged`.
    Neighbor,
    /// `updateShape`.
    Shape,
}

/// One collected neighbour-update entry — vanilla's `NeighborUpdates` unit.
///
/// An `updateNeighborsAt` call is one entry of up to six notifications run in
/// `UPDATE_ORDER`; `CollectingNeighborUpdater` runs entries depth-first:
/// entries queued while a notification dispatches run to completion (in call
/// order) before the current entry's remaining notifications. The driver's
/// `propagate` reproduces exactly that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateEntry {
    items: Vec<(Pos, Dir, UpdateKind)>,
    cursor: usize,
}


/// `MC_TICK_TRACE_UPDATES=1` — every neighbour update the engine asks for.
///
/// Logged where the update is *requested*, which is where the game logs its
/// own, so the two sequences line up call for call. Granularity is the call,
/// not the notification: one `neighbors_at` covers six neighbours.
pub(crate) fn trace_update(kind: &str, pos: Pos) {
    if std::env::var_os("MC_TICK_TRACE_UPDATES").is_some() {
        eprintln!("[upd] {kind} {} {} {}", pos.x, pos.y, pos.z);
    }
}

impl UpdateEntry {
    /// An entry dispatching `items` in order.
    pub fn new(items: Vec<(Pos, Dir, UpdateKind)>) -> Self {
        Self { items, cursor: 0 }
    }

    /// One `updateNeighborsAt(pos)`: the six neighbours in `UPDATE_ORDER`.
    pub fn neighbors_at(pos: Pos) -> Self {
        trace_update("neighbors_at", pos);
        Self::new(
            crate::pos::UPDATE_ORDER
                .iter()
                .map(|dir| (pos.offset(*dir), dir.opposite(), UpdateKind::Neighbor))
                .collect(),
        )
    }

    /// One `updateNeighbourShapes(pos)`: the six neighbours in
    /// `UPDATE_SHAPE_ORDER`, each hearing a shape update from this side.
    pub fn neighbor_shapes(pos: Pos) -> Self {
        for dir in crate::pos::UPDATE_SHAPE_ORDER {
            trace_update("shape", pos.offset(dir));
        }
        Self::new(
            crate::pos::UPDATE_SHAPE_ORDER
                .iter()
                .map(|dir| (pos.offset(*dir), dir.opposite(), UpdateKind::Shape))
                .collect(),
        )
    }

    /// `updateFromNeighbourShapes(pos)`: the block at `pos` hears a shape
    /// update from every side, in `UPDATE_SHAPE_ORDER`.
    pub fn own_shapes(pos: Pos) -> Self {
        for _ in crate::pos::UPDATE_SHAPE_ORDER {
            trace_update("shape", pos);
        }
        Self::new(
            crate::pos::UPDATE_SHAPE_ORDER
                .iter()
                .map(|dir| (pos, *dir, UpdateKind::Shape))
                .collect(),
        )
    }

    /// The next notification, if any.
    pub fn next(&mut self) -> Option<(Pos, Dir, UpdateKind)> {
        let item = self.items.get(self.cursor).copied();
        self.cursor += 1;
        item
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
/// What a [`TickCtx`] needs to deliver queued updates itself, rather than
/// handing them back to the driver when the handler returns.
///
/// `CollectingNeighborUpdater.addAndRun` runs its queue immediately when
/// nothing else is running it, and merely queues when a cascade is already in
/// flight. Handing this out as an `Option` reproduces both halves: the driver
/// passes it in, [`TickCtx::drain`] takes it for the duration of the drain, and
/// a nested `drain()` therefore finds `None` and only queues — which is the
/// count guard, not an approximation of it.
pub struct Drain<'a> {
    /// The driver's entry stack — vanilla's queue of pending notifications.
    pub pending: &'a mut Vec<UpdateEntry>,
    /// States met with no behaviour registered, for the unknown-block report.
    pub unknown_seen: &'a mut Vec<StateId>,
}

pub struct TickCtx<'a> {
    /// The driver's update pump, when this dispatch is allowed to run it.
    ///
    /// `None` inside a drain (so nested calls queue instead of recursing) and
    /// in unit tests that only want to observe what a behaviour queues.
    pub drain: Option<Drain<'a>>,
    /// Every registered behaviour: for dispatching `onPlace` on a state write,
    /// and for the update pump. Kept out of [`Drain`] so it stays reachable
    /// while a drain is in flight.
    pub behaviours: Option<&'a BehaviourTable>,
    /// Block storage.
    pub world: &'a mut World,
    /// Scheduled block ticks.
    pub ticks: &'a mut TickQueue,
    /// Scheduled **fluid** ticks — vanilla keeps these in a separate queue
    /// drained in their own phase, after block ticks.
    pub fluids: &'a mut TickQueue,
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
    pub comparator_out: &'a mut ComparatorOutputs,
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
    /// Neighbour-update entries raised during this dispatch, in call order —
    /// vanilla's `addedThisLayer`. The driver moves them onto its stack after
    /// each single notification, which is what makes cascades depth-first in
    /// call order without any re-entrant dispatch.
    pub updates: &'a mut Vec<UpdateEntry>,
}

impl<'a> TickCtx<'a> {
    /// Deliver every queued update now, exactly as [`Simulation::propagate`]
    /// does — because it *is* that loop, reached from inside a handler.
    ///
    /// `moveBlocks` needs this: it notifies the positions its push emptied
    /// while it is still running, before `triggerEvent` writes the piston base
    /// as extended. Queueing those and letting the driver drain them afterwards
    /// shows the notified blocks a piston that has already finished moving.
    ///
    /// Re-entrant calls are no-ops that leave the entries queued, which is what
    /// vanilla's `count` guard does inside `CollectingNeighborUpdater`.
    pub fn drain(&mut self) {
        // Vanilla's `maxChainedNeighborUpdates`: a circuit that keeps
        // re-notifying itself reports rather than hangs.
        const MAX_UPDATE_CASCADE: usize = 1_000_000;
        let Some(mut pump) = self.drain.take() else { return };
        let Some(table) = self.behaviours else {
            self.drain = Some(pump);
            return;
        };
        let mut delivered = 0usize;
        loop {
            // addedThisLayer joins the stack reversed, so the first-queued
            // entry ends on top and runs first.
            while let Some(entry) = self.updates.pop() {
                pump.pending.push(entry);
            }
            let Some(top) = pump.pending.last_mut() else { break };
            let Some((pos, from, kind)) = top.next() else {
                pump.pending.pop();
                continue;
            };
            delivered += 1;
            if delivered > MAX_UPDATE_CASCADE {
                self.updates.clear();
                pump.pending.clear();
                break;
            }
            let state = self.world.get(pos);
            if let Some(filter) = std::env::var_os("MC_TICK_TRACE_NOTIFY") {
                let filter = filter.to_string_lossy().to_string();
                let key = format!("{},{},{}", pos.x, pos.y, pos.z);
                if filter.split(';').any(|want| want.trim() == key) {
                    eprintln!(
                        "[t{}] notify ({key}) {kind:?} from {from:?}  {}",
                        self.tick,
                        self.states.descriptor(state).unwrap_or("minecraft:air")
                    );
                }
            }
            let Some(behaviour) = table.get(state) else {
                if state != StateId::AIR {
                    pump.unknown_seen.push(state);
                }
                continue;
            };
            match kind {
                UpdateKind::Neighbor => behaviour.on_neighbor_changed(self, pos, from),
                UpdateKind::Shape => behaviour.on_shape_update(self, pos, from),
            }
        }
        self.drain = Some(pump);
    }

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

    /// Schedule a fluid tick at `pos`, with the same boundary folding as
    /// [`TickCtx::schedule`].
    pub fn schedule_fluid(&mut self, pos: Pos, delay: u64) {
        let delay = if self.boundary { delay.saturating_sub(1) } else { delay };
        self.fluids.schedule(pos, self.tick, delay, TickPriority::Normal);
    }

    /// Queue a block event for this tick's block-events phase.
    pub fn queue_event(&mut self, pos: Pos, id: u8, param: u8) {
        let block = self.world.get(pos);
        if std::env::var("MC_TICK_TRACE_EVENTS").is_ok() {
            eprintln!(
                "[t{}] queue  {:?} id={} on {}",
                self.tick,
                (pos.x, pos.y, pos.z),
                id,
                self.states.descriptor(block).unwrap_or("?")
            );
        }
        self.events.push(BlockEvent { pos, id, param, block });
    }

    /// The state at `pos`.
    pub fn get(&self, pos: Pos) -> StateId {
        self.world.get(pos)
    }

    /// Queue one `updateNeighborsAt(pos)` entry.
    pub fn update_neighbors_at(&mut self, pos: Pos) {
        self.updates.push(UpdateEntry::neighbors_at(pos));
    }

    /// Queue one `updateNeighbourShapes(pos)` entry.
    ///
    /// Normally a consequence of a write's flags, but `moveBlocks` calls it
    /// directly: the slots a push vacates are cleared with flag 82, which
    /// carries `UPDATE_KNOWN_SHAPE` and so says nothing, and the shape pass is
    /// then run over them by hand.
    pub fn update_neighbour_shapes(&mut self, pos: Pos) {
        self.updates.push(UpdateEntry::neighbor_shapes(pos));
    }

    /// `updateNeighborsAtExceptFromFacing`: the same entry minus one side.
    pub fn update_neighbors_except(&mut self, pos: Pos, skip: Dir) {
        self.updates.push(UpdateEntry::new(
            crate::pos::UPDATE_ORDER
                .iter()
                .filter(|dir| **dir != skip)
                .map(|dir| (pos.offset(*dir), dir.opposite(), UpdateKind::Neighbor))
                .collect(),
        ));
    }

    /// `updateFromNeighbourShapes(pos)`.
    pub fn update_self_shapes(&mut self, pos: Pos) {
        self.updates.push(UpdateEntry::own_shapes(pos));
    }

    /// Queue a single notification (`level.neighborChanged` directly).
    pub fn notify(&mut self, pos: Pos, from: Dir) {
        trace_update("neighbor", pos);
        self.updates
            .push(UpdateEntry::new(vec![(pos, from, UpdateKind::Neighbor)]));
    }

    /// Set the state at `pos` and notify its six neighbours.
    ///
    /// Notifications are queued as one `updateNeighborsAt` entry; see
    /// [`TickCtx::updates`]. Nothing is queued if the write changed nothing,
    /// which is what stops two blocks that keep re-asserting the same state
    /// from looping forever.
    pub fn set(&mut self, pos: Pos, state: StateId) {
        let previous = self.world.get(pos);
        if previous == state {
            return;
        }
        self.world.set(pos, state);
        if let Some(log) = self.log.as_deref_mut() {
            log.push(BlockChange { tick: self.tick, pos, from: previous, to: state });
        }
        // `LevelChunk.setBlockState` runs `onPlace` before `markAndNotifyBlock`
        // reaches the neighbours at all.
        if let Some(table) = self.behaviours {
            if let Some(behaviour) = table.get(state) {
                behaviour.on_state_changed(self, pos);
            }
        }
        // markAndNotifyBlock, flag 3: neighbour updates first, then the shape
        // pass that observers listen to.
        //
        // The neighbour updates are *dispatched* before the shape pass is even
        // requested. `blockUpdated` goes through `addAndRun`, which runs on the
        // spot whenever the updater's stack is empty, and only when it returns
        // does `markAndNotifyBlock` reach `updateNeighbourShapes`. Queueing both
        // and draining once puts every shape update ahead of anything the
        // neighbour updates set off — including a piston head forwarding to its
        // base, which is how this first showed up.
        //
        // Nested calls are unaffected: `drain` is a no-op while a drain is
        // already running, which is exactly what `addAndRun` does.
        self.updates.push(UpdateEntry::neighbors_at(pos));
        self.drain();
        self.updates.push(UpdateEntry::neighbor_shapes(pos));
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

    /// A **flag 2** write: no neighbour updates, but the shape pass still
    /// runs (`markAndNotifyBlock` only skips it for `UPDATE_KNOWN_SHAPE`).
    /// This is the dust evaluator's write — and it is how an observer
    /// watching redstone dust sees a power change at all.
    pub fn set_shape_only(&mut self, pos: Pos, state: StateId) {
        let previous = self.world.get(pos);
        if previous == state {
            return;
        }
        self.world.set(pos, state);
        if let Some(log) = self.log.as_deref_mut() {
            log.push(BlockChange { tick: self.tick, pos, from: previous, to: state });
        }
        self.updates.push(UpdateEntry::neighbor_shapes(pos));
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
        // updateNeighbourForOutputSignal: direct notifications in Java's
        // Direction.values() order.
        for dir in crate::pos::JAVA_DIRECTIONS {
            self.notify(pos.offset(dir), dir.opposite());
        }
        // (updateNeighbourForOutputSignal: direct neighborChanged calls.)
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

    /// A scheduled fluid tick fired at `pos` — `FluidState.tick`, dispatched
    /// from the fluid-ticks phase.
    fn on_fluid_tick(&self, _ctx: &mut TickCtx<'_>, _pos: Pos) {}

    /// A **shape** update reached `pos` from `from` — vanilla's `updateShape`.
    ///
    /// A different callback from [`BlockBehaviour::on_neighbor_changed`], and
    /// the distinction is load-bearing: `ObserverBlock` overrides *only* this
    /// one, which is why a structure placement (whose pass runs
    /// `updateFromNeighbourShapes` on every block) pulses every observer
    /// without triggering a single piston.
    fn on_shape_update(&self, _ctx: &mut TickCtx<'_>, _pos: Pos, _from: Dir) {}

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

    /// An entity is inside this block's cell — vanilla's `entityInside`,
    /// dispatched after entity movement for every cell an item overlaps.
    fn on_entity_inside(&self, _ctx: &mut TickCtx<'_>, _pos: Pos) {}

    /// Called after this block's *state* changes in place.
    ///
    /// `onPlace` runs on every `setBlockState`, not only on a genuine
    /// placement — which is why `PistonBaseBlock.onPlace` opens with
    /// `!oldState.is(block)` and `RedstoneTorchBlock.onPlace` does not. The
    /// guarded ones are modelled by leaving this at its default; the unguarded
    /// ones implement it, and it fires on every write.
    ///
    /// A torch lighting is the case that forced it: `notifyNeighbors` reaches
    /// two blocks out, and without it the piston standing on the block above a
    /// torch never hears that the torch came on.
    fn on_state_changed(&self, _ctx: &mut TickCtx<'_>, _pos: Pos) {}

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
            drain: None,
            behaviours: None,
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
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
            drain: None,
            behaviours: None,
            world: &mut world,
            ticks: &mut ticks,
            fluids: &mut TickQueue::new(),
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
