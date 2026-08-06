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
        Self {
            cooldown: 0,
            ticked_at: -1,
        }
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

/// `MC_TICK_TRACE_WRITE=x,y,z[;...]` — every write landing on a position.
///
/// A divergence that is a *write* rather than an event is invisible in both the
/// event log and the notification log, and that is where these have been
/// hiding. This says who touched a block and with which flags.
pub(crate) fn trace_write(kind: &str, pos: Pos, states: &StateRegistry, to: StateId) {
    let Some(filter) = std::env::var_os("MC_TICK_TRACE_WRITE") else {
        return;
    };
    let wanted = filter.to_string_lossy().split(';').any(|t| {
        let c: Vec<i32> = t.split(',').filter_map(|v| v.trim().parse().ok()).collect();
        c.len() == 3 && c[0] == pos.x && c[1] == pos.y && c[2] == pos.z
    });
    if wanted {
        eprintln!(
            "[write] {kind:<14} {} {} {} -> {}",
            pos.x,
            pos.y,
            pos.z,
            states.descriptor(to).unwrap_or("?")
        );
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
    /// The state that is visibly *travelling*, which is not always the state
    /// that lands.
    ///
    /// Equal to [`state`](Self::state) for everything a piston carries. It
    /// differs for the one case vanilla's client singles out: a **retracting
    /// source piston**, where `PistonHeadRenderer.extractRenderState` puts a
    /// `piston_head` in the moving slot and draws the base separately. The
    /// piston body does not move when it retracts — its arm comes home — and a
    /// consumer given only the landing state slides the whole body a block.
    pub carried: StateId,
    /// [`carried`](Self::carried) with a piston arm's `short=true`, if it is
    /// one.
    ///
    /// The game draws a moving head shortened while it is within half a block
    /// of its body and full length beyond that, so a mover has to be able to
    /// pick. `None` for anything that is not a piston head.
    pub carried_short: Option<StateId>,
    /// A state that occupies `pos` for the whole move, if any.
    ///
    /// Vanilla's `base` slot: the retracting piston's own body, drawn
    /// `EXTENDED=true` and *outside* the interpolated translate that
    /// [`carried`](Self::carried) rides. `None` whenever the cell is simply
    /// empty until the move lands, which is every other move.
    pub remains: Option<StateId>,
    /// The tick that scheduled the write.
    ///
    /// Not used by the engine — [`resolve_on`](Self::resolve_on) is what it
    /// acts on. It is here because a *consumer* animating a move needs the
    /// window, not just its end: how far along a stroke is at tick `t` is
    /// `(t - started_on) / (resolve_on - started_on)`, and the alternative is
    /// every consumer assuming [`crate::piston::PISTON_MOVE_TICKS`] applies to
    /// a delay this struct deliberately leaves free.
    pub started_on: u64,
    /// The tick whose block-entities phase applies it.
    pub resolve_on: u64,
    /// `isSourcePiston`: the placeholder a piston writes over its *own* square —
    /// the head slot it is extending into, or its base while it retracts —
    /// rather than one carrying a block it pushes.
    ///
    /// It decides what an early `finalTick` lands. A move that runs to
    /// completion lands the moved state either way, but `finalTick` lands
    /// **air** for a source piston, and a retract that interrupts an extension
    /// goes through `finalTick`. Landing the head there instead leaves a piston
    /// head in a slot vanilla emptied, and that head then forwards every
    /// neighbour update it receives on to its base — updates vanilla never
    /// delivers.
    pub source_piston: bool,
    /// Which way the block is *travelling*, when a piston is carrying it.
    ///
    /// Extension moves toward the piston's facing, retraction away from it.
    /// `None` for a deferred write that is not a piston movement.
    ///
    /// Carried because a `moving_piston` is not only a delayed block write —
    /// while it is in flight its collision box sweeps forward, and entities in
    /// that sweep are shoved along. Without the direction the engine knows a
    /// block is moving but not which way, and cannot displace anything.
    /// See [`crate::piston::sweep_displacement`].
    pub sweep: Option<crate::piston::Sweep>,
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
    /// Every delivered notification, when update recording is on.
    ///
    /// Lives here rather than on [`TickCtx`] because a drain is the only place
    /// updates are ever *delivered* — everywhere else they are merely queued.
    /// `None` when recording is off, which is the default: the tick loop should
    /// not pay for observability nobody asked for.
    pub upd_log: Option<&'a mut Vec<UpdateRecord>>,
    /// The phase currently executing, or `None` for a boundary dispatch.
    pub phase: Option<crate::phase::Phase>,
}

/// One delivered update — the raw material of a propagation view.
///
/// Recorded at the moment of *delivery* rather than of request, which is why it
/// can carry `state`: the block as it stood when the notification reached it.
/// Which block sits at a position mid-tick is what decides whether an update
/// does anything, and it is invisible in a per-tick snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateRecord {
    /// The tick this was delivered in.
    pub tick: u64,
    /// Position within the tick, counting from 0 — the scrubber's sub-tick axis.
    pub seq: u32,
    /// The block being notified.
    pub pos: Pos,
    /// The side the notification arrived from.
    pub from: Dir,
    /// `neighborChanged` or `updateShape`.
    pub kind: UpdateKind,
    /// The phase it landed in; `None` outside a phase walk.
    pub phase: Option<crate::phase::Phase>,
    /// The state at `pos` at dispatch time.
    pub state: StateId,
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
    /// Pre-resolved command-block programs by position — set at build from
    /// each command block's `Command` NBT, immutable during a run.
    pub commands: &'a std::collections::HashMap<Pos, CommandProgram>,
    /// Command blocks' last-seen powered flags, for rising-edge detection.
    pub command_powered: &'a mut std::collections::HashMap<Pos, bool>,
    /// The block-entity tick list, reconciled on every block write: a hopper
    /// a command block setblocks into existence must tick, and one it
    /// removes must stop. Vanilla creates/destroys the block entity with the
    /// block; the engine keeps the tick list in step instead.
    pub tickers: &'a mut Vec<Pos>,
    /// The world's item entities — what hoppers vacuum and droppers eject.
    pub item_entities: &'a mut crate::entity::ItemEntities,
    /// The world's minecarts — for a hopper meeting a container cart
    /// (`HopperBlockEntity.getEntityContainer`) and a detector rail reading
    /// the cart parked on it.
    pub minecarts: &'a mut Vec<crate::minecart::MinecartState>,
    /// Conductor-per-state, for `updateNeighbourForOutputSignal`'s
    /// through-a-solid extension — a comparator reading a container through
    /// one conductor hears about the container's change via exactly this.
    pub conductors: &'a [bool],
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
        let Some(mut pump) = self.drain.take() else {
            return;
        };
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
            let Some(top) = pump.pending.last_mut() else {
                break;
            };
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
            // Recorded before the behaviour lookup, so a notification landing on
            // air or on an unregistered block still shows up: the question a
            // propagation view answers is "what did the update reach", not
            // "what reacted to it".
            if let Some(log) = pump.upd_log.as_deref_mut() {
                let tick = self.tick;
                let seq = match log.last() {
                    Some(last) if last.tick == tick => last.seq + 1,
                    _ => 0,
                };
                log.push(UpdateRecord {
                    tick,
                    seq,
                    pos,
                    from,
                    kind,
                    phase: pump.phase,
                    state,
                });
            }
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
        // `MC_TICK_TRACE_SCHED=x,y,z[;...]` — who books a tick, and in what
        // order. Two blocks scheduled for the same tick fire in insertion
        // order, so this is what decides races between them.
        if let Some(filter) = std::env::var_os("MC_TICK_TRACE_SCHED") {
            let wanted = filter.to_string_lossy().split(';').any(|t| {
                let c: Vec<i32> = t.split(',').filter_map(|n| n.trim().parse().ok()).collect();
                matches!(c[..], [x, y, z] if Pos::new(x, y, z) == pos)
            });
            if wanted {
                eprintln!(
                    "[sched] t{} {:?} delay={delay} prio={priority:?} boundary={}",
                    self.tick,
                    (pos.x, pos.y, pos.z),
                    self.boundary
                );
            }
        }
        // Folded into the delay rather than the tick so that a boundary schedule
        // before tick 0 still lands on tick `delay - 1` instead of underflowing.
        // A boundary delay of 0 stays 0: it would fire in the upcoming tick's
        // block-ticks phase either way.
        let delay = if self.boundary {
            delay.saturating_sub(1)
        } else {
            delay
        };
        self.ticks.schedule(pos, self.tick, delay, priority);
    }

    /// Schedule a fluid tick at `pos`, with the same boundary folding as
    /// [`TickCtx::schedule`].
    pub fn schedule_fluid(&mut self, pos: Pos, delay: u64) {
        let delay = if self.boundary {
            delay.saturating_sub(1)
        } else {
            delay
        };
        self.fluids
            .schedule(pos, self.tick, delay, TickPriority::Normal);
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
        self.events.push(BlockEvent {
            pos,
            id,
            param,
            block,
        });
    }

    /// The state at `pos`.
    pub fn get(&self, pos: Pos) -> StateId {
        self.world.get(pos)
    }

    /// `markAndNotifyBlock`'s indirect passes: the old state's, then — after the
    /// six-neighbour shape pass — the new state's. Both, because a connection
    /// that has just gone away still has to tell what it was reaching.
    fn indirect_shapes(&mut self, pos: Pos, state: StateId) {
        let Some(table) = self.behaviours else { return };
        let Some(behaviour) = table.get(state) else {
            return;
        };
        let targets = behaviour.indirect_shape_targets(self.world, pos);
        for (target, from) in targets {
            trace_update("shape", target);
            self.updates
                .push(UpdateEntry::new(vec![(target, from, UpdateKind::Shape)]));
        }
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
        trace_write("set(flag3)", pos, self.states, state);
        let previous = self.world.get(pos);
        if previous == state {
            return;
        }
        self.world.set(pos, state);
        if let Some(log) = self.log.as_deref_mut() {
            log.push(BlockChange {
                tick: self.tick,
                pos,
                from: previous,
                to: state,
            });
        }
        self.reconcile_ticker(pos, previous, state);
        // `LevelChunk.setBlockState`: `onRemove` runs first, and only when
        // the block *identity* changed — a wire flipping power stays a wire
        // and hears nothing.
        let block_of = |states: &StateRegistry, id: StateId| {
            states
                .descriptor(id)
                .map(|d| d.split('[').next().unwrap_or(d).to_string())
        };
        if block_of(self.states, previous) != block_of(self.states, state) {
            if let Some(table) = self.behaviours {
                if let Some(behaviour) = table.get(previous) {
                    behaviour.on_removed(self, pos);
                }
            }
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
        self.indirect_shapes(pos, previous);
        self.updates.push(UpdateEntry::neighbor_shapes(pos));
        self.indirect_shapes(pos, state);
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
        trace_write("set_quiet", pos, self.states, state);
        let previous = self.world.get(pos);
        if previous == state {
            return;
        }
        self.world.set(pos, state);
        if let Some(log) = self.log.as_deref_mut() {
            log.push(BlockChange {
                tick: self.tick,
                pos,
                from: previous,
                to: state,
            });
        }
        self.reconcile_ticker(pos, previous, state);
    }

    /// A **flag 2** write: no neighbour updates, but the shape pass still
    /// runs (`markAndNotifyBlock` only skips it for `UPDATE_KNOWN_SHAPE`).
    /// This is the dust evaluator's write — and it is how an observer
    /// watching redstone dust sees a power change at all.
    pub fn set_shape_only(&mut self, pos: Pos, state: StateId) {
        trace_write("set_shape_only", pos, self.states, state);
        let previous = self.world.get(pos);
        if previous == state {
            return;
        }
        self.world.set(pos, state);
        if let Some(log) = self.log.as_deref_mut() {
            log.push(BlockChange {
                tick: self.tick,
                pos,
                from: previous,
                to: state,
            });
        }
        self.reconcile_ticker(pos, previous, state);
        self.indirect_shapes(pos, previous);
        self.updates.push(UpdateEntry::neighbor_shapes(pos));
        self.indirect_shapes(pos, state);
    }

    /// Keep the block-entity tick list in step with a write — see
    /// [`TickCtx::tickers`]. Appended in write order, which is vanilla's
    /// creation order for freshly made block entities.
    fn reconcile_ticker(&mut self, pos: Pos, previous: StateId, state: StateId) {
        let Some(table) = self.behaviours else { return };
        let ticks = |s: StateId| table.get(s).is_some_and(|b| b.ticks_as_block_entity());
        let was = ticks(previous);
        let is = ticks(state);
        if is && !was {
            if !self.tickers.contains(&pos) {
                self.tickers.push(pos);
            }
        } else if was && !is {
            self.tickers.retain(|p| *p != pos);
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

    /// Detach and return the container contents carried by the stack in a
    /// slot — a shulker box's slots travelling with the item.
    pub fn take_slot_contents(
        &mut self,
        pos: Pos,
        slot: u8,
    ) -> Option<Vec<crate::inventory::ItemStack>> {
        self.inventories.get_mut(&pos).and_then(|inv| {
            inv.stacks
                .iter_mut()
                .find(|stack| stack.slot == slot)
                .and_then(|stack| stack.contents.take())
        })
    }

    /// Attach container contents to the stack in a slot. No-op when empty —
    /// contents only ever ride on a real item.
    pub fn set_slot_contents(
        &mut self,
        pos: Pos,
        slot: u8,
        contents: Option<Vec<crate::inventory::ItemStack>>,
    ) {
        let Some(contents) = contents else { return };
        if let Some(inv) = self.inventories.get_mut(&pos) {
            if let Some(stack) = inv.stacks.iter_mut().find(|stack| stack.slot == slot) {
                stack.contents = Some(contents);
            }
        }
    }

    /// `HopperBlockEntity.getEntityContainer`: the container cart whose box
    /// intersects the 1-cube centred at `center`. Vanilla picks a *random*
    /// entity when several overlap; the engine takes the lowest id, which is
    /// identical whenever at most one cart straddles the cell — true of every
    /// measured machine, and the honest place to look when one day it is not.
    pub fn cart_container_at(&self, center: [f64; 3]) -> Option<usize> {
        let mut found: Option<usize> = None;
        for (index, cart) in self.minecarts.iter().enumerate() {
            if cart.removed || cart.inventory.is_none() {
                continue;
            }
            let (emin, emax) = crate::minecart::cart_aabb(cart.pos);
            let hit = (0..3)
                .all(|axis| emin[axis] < center[axis] + 0.5 && emax[axis] > center[axis] - 0.5);
            if hit && found.is_none_or(|prev: usize| cart.id < self.minecarts[prev].id) {
                found = Some(index);
            }
        }
        found
    }

    /// Read a container cart's slot, hopper-style: `(id, count)` or `None`.
    pub fn cart_slot(&self, cart: usize, slot: u8) -> Option<(String, u8)> {
        self.minecarts[cart].inventory.as_ref().and_then(|inv| {
            inv.stacks
                .iter()
                .find(|stack| stack.slot == slot && stack.count > 0)
                .map(|stack| (stack.id.clone(), stack.count))
        })
    }

    /// Write a container cart's slot, recording the change (keyed at the
    /// cart's block cell) and poking `updateNeighbourForOutputSignal` there —
    /// how a comparator behind the detector rail under the cart notices.
    pub fn set_cart_slot(&mut self, cart: usize, slot: u8, to: Option<(String, u8)>) {
        let from = self.cart_slot(cart, slot);
        if from == to {
            return;
        }
        let cell = {
            let pos = self.minecarts[cart].pos;
            Pos::new(
                pos[0].floor() as i32,
                pos[1].floor() as i32,
                pos[2].floor() as i32,
            )
        };
        let inv = self.minecarts[cart]
            .inventory
            .as_mut()
            .expect("set_cart_slot on a cart with no container");
        inv.stacks.retain(|stack| stack.slot != slot);
        if let Some((id, count)) = &to {
            inv.stacks.push(crate::inventory::ItemStack {
                slot,
                id: id.clone(),
                count: *count,
                contents: None,
            });
        }
        if let Some(log) = self.inv_log.as_deref_mut() {
            log.push(InventoryChange {
                tick: self.tick,
                pos: cell,
                slot,
                from,
                to,
            });
        }
        // The same `Plane.HORIZONTAL` sweep as [`TickCtx::set_inventory_slot`].
        for dir in [Dir::North, Dir::East, Dir::South, Dir::West] {
            let neighbor = cell.offset(dir);
            self.notify(neighbor, dir.opposite());
            let state = self.world.get(neighbor);
            if self
                .conductors
                .get(state.raw() as usize)
                .copied()
                .unwrap_or(false)
            {
                self.notify(neighbor.offset(dir), dir.opposite());
            }
        }
    }

    /// Detach and return container contents riding a cart's slot — the cart
    /// half of [`TickCtx::take_slot_contents`].
    pub fn take_cart_slot_contents(
        &mut self,
        cart: usize,
        slot: u8,
    ) -> Option<Vec<crate::inventory::ItemStack>> {
        self.minecarts[cart].inventory.as_mut().and_then(|inv| {
            inv.stacks
                .iter_mut()
                .find(|stack| stack.slot == slot)
                .and_then(|stack| stack.contents.take())
        })
    }

    /// Attach container contents to a cart's slot — the cart half of
    /// [`TickCtx::set_slot_contents`].
    pub fn set_cart_slot_contents(
        &mut self,
        cart: usize,
        slot: u8,
        contents: Option<Vec<crate::inventory::ItemStack>>,
    ) {
        let Some(contents) = contents else { return };
        if let Some(inv) = self.minecarts[cart].inventory.as_mut() {
            if let Some(stack) = inv.stacks.iter_mut().find(|stack| stack.slot == slot) {
                stack.contents = Some(contents);
            }
        }
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
                contents: None,
            });
        }
        if let Some(log) = self.inv_log.as_deref_mut() {
            log.push(InventoryChange {
                tick: self.tick,
                pos,
                slot,
                from,
                to,
            });
        }
        // `Level.updateNeighbourForOutputSignal`: the four *horizontals*, in
        // `Direction.Plane.HORIZONTAL`'s declared order — NORTH, EAST,
        // SOUTH, WEST — not Direction.values(). The order is load-bearing:
        // lithium's comparator_update_collection hangs two comparators off
        // one chest, east and south, and their submission order into the
        // tick queue decides which of two pistons claims a shared redstone
        // block. The vanilla oracle's queue log shows the east one
        // scheduled first (`cuc_trace.json`, order 32 vs 33).
        //
        // Each horizontal is notified, and when it is a conductor the cell
        // one further along the same direction hears too — how the
        // comparator behind the concrete beside lithium's chest learns that
        // a hopper touched it (`comparator_update_collection`'s core).
        for dir in [Dir::North, Dir::East, Dir::South, Dir::West] {
            let neighbor = pos.offset(dir);
            self.notify(neighbor, dir.opposite());
            let state = self.world.get(neighbor);
            if self
                .conductors
                .get(state.raw() as usize)
                .copied()
                .unwrap_or(false)
            {
                self.notify(neighbor.offset(dir), dir.opposite());
            }
        }
    }

    /// `updateNeighbourForOutputSignal` with no slot write — what a hopper's
    /// *failed* take fires: vanilla's `tryTakeInItemFromSlot` removes the
    /// item and puts it back, and both halves call the container's
    /// `setChanged`. The no-op still schedules every comparator watching the
    /// container, which is the churn lithium's update collection exists to
    /// batch — and what its gametest measures.
    pub fn poke_container_output(&mut self, pos: Pos) {
        for dir in [Dir::North, Dir::East, Dir::South, Dir::West] {
            let neighbor = pos.offset(dir);
            self.notify(neighbor, dir.opposite());
            let state = self.world.get(neighbor);
            if self
                .conductors
                .get(state.raw() as usize)
                .copied()
                .unwrap_or(false)
            {
                self.notify(neighbor.offset(dir), dir.opposite());
            }
        }
    }

    /// Schedule a block write for `delay` ticks from now, resolved in the
    /// block-entities phase.
    pub fn defer(
        &mut self,
        pos: Pos,
        state: StateId,
        delay: u64,
        sweep: Option<crate::piston::Sweep>,
    ) {
        self.push_move(pos, state, state, None, None, delay, false, sweep);
    }

    /// `defer`, for the placeholder a piston writes over its own square.
    pub fn defer_source(
        &mut self,
        pos: Pos,
        state: StateId,
        delay: u64,
        sweep: Option<crate::piston::Sweep>,
    ) {
        self.push_move(pos, state, state, None, None, delay, true, sweep);
    }

    /// [`defer_source`](Self::defer_source) for a move whose travelling block
    /// is a piston **arm**.
    ///
    /// `head`/`head_short` are the two forms the game draws it in, and `body`
    /// is the piston left standing in place while the arm moves — which a
    /// retraction has and an extension does not, because an extending piston's
    /// body is already written into the world.
    ///
    /// Nothing in the simulation reads any of them. They exist so a consumer
    /// does not have to know that a retracting piston's body stays put, or
    /// assemble `piston_head` states out of a base's.
    #[allow(clippy::too_many_arguments)]
    pub fn defer_source_arm(
        &mut self,
        pos: Pos,
        state: StateId,
        head: StateId,
        head_short: StateId,
        body: Option<StateId>,
        delay: u64,
        sweep: Option<crate::piston::Sweep>,
    ) {
        self.push_move(pos, state, head, Some(head_short), body, delay, true, sweep);
    }

    #[allow(clippy::too_many_arguments)]
    fn push_move(
        &mut self,
        pos: Pos,
        state: StateId,
        carried: StateId,
        carried_short: Option<StateId>,
        remains: Option<StateId>,
        delay: u64,
        source_piston: bool,
        sweep: Option<crate::piston::Sweep>,
    ) {
        self.moves.push(PendingMove {
            pos,
            state,
            carried,
            carried_short,
            remains,
            started_on: self.tick,
            resolve_on: self.tick + delay,
            source_piston,
            sweep,
        });
    }
}

/// How one kind of block behaves.
pub trait BlockBehaviour: Send + Sync {
    /// `updateIndirectNeighbourShapes`: the *diagonal* partners a write must
    /// also shape-update, on top of its six neighbours.
    ///
    /// Only dust overrides this in the game, and it is what makes a staircase
    /// of wire propagate: a wire connected to a block it climbs tells the wire
    /// above and below that block. Without it a diagonal dust line is deaf to
    /// its own neighbours.
    ///
    /// `markAndNotifyBlock` runs it for the *old* state and the new one, so a
    /// connection that has just gone away still notifies what it was reaching.
    fn indirect_shape_targets(&self, _world: &World, _pos: Pos) -> Vec<(Pos, Dir)> {
        Vec::new()
    }

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

    /// This block was just replaced by a *different* block — vanilla's
    /// `onRemove`, which `LevelChunk.setBlockState` only calls when the
    /// block identity changes, never on a state flip of the same block.
    /// Dust needs it: `RedStoneWireBlock.onRemove` updates the neighbours
    /// of each of its six neighbours, and a torch two steps away relights
    /// on exactly that sweep when an explosion eats the wire.
    fn on_removed(&self, _ctx: &mut TickCtx<'_>, _pos: Pos) {}

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

    /// A random tick landed on this block — `Block.randomTick`. Only blocks
    /// with random-tick behaviour (ice melting) implement it; the pass runs
    /// only when [`randomTickSpeed`](crate::Simulation::set_random_ticks) is
    /// nonzero.
    fn on_random_tick(&self, _ctx: &mut TickCtx<'_>, _pos: Pos) {}

    /// A short name for traces and diagnostics.
    fn name(&self) -> &'static str;
}

/// One pre-resolved command-block program — the world-edit shapes an
/// impulse command block can run headlessly. Commands outside this set
/// (summon, data, queries) simply have no program: their block powers on
/// and executes nothing, which is also what an unparseable command does
/// in game.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandProgram {
    /// `setblock ~dx ~dy ~dz <state>`: one write, relative to the block.
    SetBlock {
        /// Offset from the command block.
        offset: (i32, i32, i32),
        /// The pre-interned state to write.
        state: StateId,
    },
    /// `fill ~.. ~.. ~.. <state>`: every cell of the inclusive box between
    /// the two relative corners.
    Fill {
        /// First corner, relative to the command block.
        a: (i32, i32, i32),
        /// Second corner, relative to the command block.
        b: (i32, i32, i32),
        /// The pre-interned state to write.
        state: StateId,
    },
    /// `summon <kind> ~dx ~dy ~dz [{fuse:N}]`: queue an entity spawn at the
    /// block's centre plus the offset. The spawn lands next entity pass.
    Summon {
        /// Normalised entity id, interned like the item names.
        kind: &'static str,
        /// Offset from the block centre.
        offset: [f64; 3],
        /// A `{fuse:N}` tag, when present.
        fuse: Option<i32>,
    },
    /// `data merge entity @e[type=item,distance=..N,limit=1] {Item:{id: X}}`:
    /// retype the nearest item entity within `radius` of the block.
    RetypeNearestItem {
        /// Selector radius in blocks.
        radius: f64,
        /// The item id to write. Boxed str keeps the enum `Copy`-adjacent
        /// cheap without a full `String` clone per dispatch.
        item: &'static str,
    },
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
            behaviour.redstone_power(
                &World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(1, 1, 1))),
                Pos::new(0, 0, 0),
                Dir::Up
            ),
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

        let report = table
            .unknown_report(&states)
            .expect("observer is unhandled");
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
        let piston = states
            .intern("minecraft:sticky_piston[facing=east]")
            .unwrap();
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
            commands: &Default::default(),
            command_powered: &mut Default::default(),
            tickers: &mut Default::default(),
            item_entities: &mut Default::default(),
            minecarts: Box::leak(Box::new(Vec::new())),
            conductors: &[],
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
            commands: &Default::default(),
            command_powered: &mut Default::default(),
            tickers: &mut Default::default(),
            item_entities: &mut Default::default(),
            minecarts: Box::leak(Box::new(Vec::new())),
            conductors: &[],
            inv_log: None,
            log: None,
        };
        ctx.schedule(Pos::new(1, 1, 1), 2, TickPriority::High);
        ctx.queue_event(Pos::new(2, 2, 2), 1, 3);

        assert_eq!(ticks.next_due(), Some(12), "delay is relative to ctx.tick");
        assert_eq!(events.len(), 1);
    }
}
