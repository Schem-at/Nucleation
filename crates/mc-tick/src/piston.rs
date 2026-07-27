//! Pistons: the reason the tick phases had to be right before any behaviour.
//!
//! # Why a piston is not just another block
//!
//! Every other component so far acts within one phase. A piston's motion is spread
//! across the tick:
//!
//! ```text
//! phase 3  BlockTicks     a diode changes, notifying the piston
//! phase 7  BlockEvents    the piston actually moves      <- Level.blockEvent
//! phase 9  BlockEntities  the moving blocks finish       <- not yet modelled
//! ```
//!
//! Verified from `PistonBaseBlock`: it calls `Level.blockEvent` and does **not**
//! schedule a block tick of its own. So a piston notified during phase 3 moves in
//! phase 7 of the *same* tick — not the next one. An engine that treated the move
//! as a scheduled tick would report every door a tick slow.
//!
//! # What is modelled here
//!
//! Extension and retraction (both of which travel: placeholders now, real states
//! two ticks later in phase 9, the retracting *base* included), the push limit,
//! quasi-connectivity, slime/honey adhesion with pulls, short-pulse dropping,
//! dispatch-time re-validation of queued events, and vanilla's silent move
//! writes. Every rule is backed by a captured trace or the class's bytecode; the
//! details and their captures are in `redstone_components.md`.

use crate::behaviour::{BlockBehaviour, TickCtx};
use crate::components::{PowerSource, StatePair};
use crate::pos::{Dir, Pos};
use crate::state::StateId;
use crate::world::World;

/// Block event id: extend. `PistonBaseBlock.TRIGGER_EXTEND`.
pub const TRIGGER_EXTEND: u8 = 0;
/// Block event id: retract. `PistonBaseBlock.TRIGGER_CONTRACT`.
pub const TRIGGER_CONTRACT: u8 = 1;
/// Block event id: drop. `PistonBaseBlock.TRIGGER_DROP`.
pub const TRIGGER_DROP: u8 = 2;

/// How many blocks a piston can push.
///
/// `PistonStructureResolver.MAX_PUSH_DEPTH`. Inlined by javac, so it was captured
/// rather than read: a twelve-block column moved, a thirteen-block column did not.
pub const MAX_PUSH_DEPTH: usize = 12;

/// Game ticks a piston's blocks spend in motion before landing.
///
/// Captured from vanilla, not assumed:
///
/// ```text
/// tick 0  piston -> extended;  stone -> moving_piston;  air -> moving_piston
/// tick 2  moving_piston -> piston_head;  moving_piston -> stone
/// ```
///
/// The blocks become `moving_piston` placeholders immediately and resolve two ticks
/// later in the block-entities phase. Moving them instantly — as a first
/// implementation naturally does — reports every door two ticks early.
pub const PISTON_MOVE_TICKS: u64 = 2;

/// The two sticky block kinds.
///
/// They behave identically when dragging ordinary blocks, and differ in one rule
/// that matters: **slime and honey do not stick to each other**. Builds rely on
/// that to separate two halves of a contraption, so collapsing the distinction
/// would quietly fuse structures the game keeps apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sticky {
    /// Slime block.
    Slime,
    /// Honey block.
    Honey,
}

/// Which blocks a piston may move, and which of them are sticky.
///
/// Supplied by the caller: obsidian and bedrock are immovable, most blocks are
/// pushable, and this crate holds no Minecraft block list of its own.
pub trait Movability: Send + Sync {
    /// Whether the block at `pos` can be pushed.
    ///
    /// Must return false for genuinely immovable blocks — obsidian, bedrock, a
    /// block currently in motion (`moving_piston`), and an extended piston base.
    /// One immovable block anywhere in a resolved structure cancels the whole push.
    fn is_movable(&self, world: &World, pos: Pos) -> bool;

    /// Whether a push *breaks* the block at `pos` instead of moving it —
    /// `PushReaction.DESTROY`, which is dust, torches, rails and plants.
    fn destroys(&self, _world: &World, _pos: Pos) -> bool {
        false
    }

    /// Whether the block at `pos` is air, i.e. free space for a push to end in.
    fn is_empty(&self, world: &World, pos: Pos) -> bool {
        world.get(pos) == StateId::AIR
    }

    /// The sticky kind of the block at `pos`, if it is one.
    fn sticky(&self, _world: &World, _pos: Pos) -> Option<Sticky> {
        None
    }
}

/// Whether two adjacent blocks drag one another.
///
/// A sticky block adheres to any movable neighbour, **except** that slime and
/// honey do not adhere to each other.
fn adheres(a: Option<Sticky>, b: Option<Sticky>) -> bool {
    match (a, b) {
        (None, None) => false,
        (Some(x), Some(y)) => x == y,
        _ => true,
    }
}

/// The outcome of working out what a piston would move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PushPlan {
    /// Positions to move, ordered from the far end back toward the piston.
    ///
    /// Far-end-first is what makes applying the plan a simple loop: each block is
    /// written into space the previous write has already vacated.
    pub to_push: Vec<Pos>,
    /// Positions the push destroys rather than moves, in collection order.
    ///
    /// `PistonStructureResolver.toDestroy`. They are broken, not carried, and
    /// `moveBlocks` notifies each of them before it notifies the vacated
    /// sources — dust breaking beside a piston is a power change like any
    /// other, and something has to hear it.
    pub to_destroy: Vec<Pos>,
    /// Whether the push is possible at all.
    pub possible: bool,
}

/// Work out what extending a piston at `piston` facing `dir` would move.
///
/// Mirrors `PistonStructureResolver.resolve` for the straight-line case: walk
/// forward while blocks are movable, stop at the first empty space, and refuse if
/// the column exceeds [`MAX_PUSH_DEPTH`] or ends in something immovable.
pub fn resolve_push(
    world: &World,
    movability: &dyn Movability,
    piston: Pos,
    dir: Dir,
) -> PushPlan {
    let start = piston.offset(dir);
    let mut to_push: Vec<Pos> = Vec::new();
    let mut to_destroy: Vec<Pos> = Vec::new();
    let failed = PushPlan { to_push: Vec::new(), to_destroy: Vec::new(), possible: false };

    // `resolve()`: the start block must be pushable at all, then one line from
    // it, then a branching pass over every sticky block collected.
    if !movability.is_empty(world, start) && !movability.is_movable(world, start) {
        // `resolve()`: a start block that breaks is collected and the push
        // still goes ahead — with nothing to carry.
        if movability.destroys(world, start) {
            return PushPlan { to_push, to_destroy: vec![start], possible: true };
        }
        return failed;
    }
    if !add_block_line(world, movability, piston, dir, start, dir, &mut to_push, &mut to_destroy) {
        return failed;
    }
    let mut index = 0;
    while index < to_push.len() {
        let pos = to_push[index];
        if movability.sticky(world, pos).is_some()
            && !add_branching_blocks(world, movability, piston, dir, pos, &mut to_push, &mut to_destroy)
        {
            return failed;
        }
        index += 1;
    }

    // No sort. `PistonStructureResolver` hands `moveBlocks` its list as built —
    // nearest the piston first, with slime branches interleaved by
    // `reorderListAtCollision` — and `moveBlocks` walks it *backwards*, so every
    // block is written into space an earlier write has already vacated.
    //
    // Sorting it by distance along the push axis looks equivalent and is not:
    // a branch pulled sideways by slime has no meaningful position on that axis,
    // and the order is observable anyway, because the moving block entities land
    // in creation order and each landing notifies its neighbours.
    PushPlan { to_push, to_destroy, possible: true }
}

/// `PistonStructureResolver.addBlockLine`.
///
/// Walks **backwards** from `origin` along a sticky chain first — slime drags
/// what is behind it — then forwards along the push direction, and hands a
/// collision with an already-collected block to `reorder_at_collision`. A
/// simplified forward-only version passed every small golden and still let a
/// flying machine make a push the game refuses, which is what sent us here.
#[allow(clippy::too_many_arguments)]
fn add_block_line(
    world: &World,
    movability: &dyn Movability,
    piston: Pos,
    push_dir: Dir,
    origin: Pos,
    _face: Dir,
    to_push: &mut Vec<Pos>,
    to_destroy: &mut Vec<Pos>,
) -> bool {
    if movability.is_empty(world, origin) {
        return true;
    }
    if !movability.is_movable(world, origin) {
        if movability.destroys(world, origin) && !to_destroy.contains(&origin) {
            to_destroy.push(origin);
        }
        return true;
    }
    if origin == piston || to_push.contains(&origin) {
        return true;
    }

    // The backward sticky chain.
    let mut chain = 1usize;
    if chain + to_push.len() > MAX_PUSH_DEPTH {
        return false;
    }
    let mut previous = movability.sticky(world, origin);
    while previous.is_some() {
        let back = origin.offset_by(push_dir.opposite(), chain as i32);
        let back_sticky = movability.sticky(world, back);
        if movability.is_empty(world, back)
            || !adheres(previous, back_sticky)
            || !movability.is_movable(world, back)
            || back == piston
        {
            break;
        }
        chain += 1;
        if chain + to_push.len() > MAX_PUSH_DEPTH {
            return false;
        }
        previous = back_sticky;
    }

    // Collected from the far end of the chain back to the origin.
    let mut added = 0usize;
    for step in (0..chain).rev() {
        to_push.push(origin.offset_by(push_dir.opposite(), step as i32));
        added += 1;
    }

    // Then forwards.
    let mut step = 1i32;
    loop {
        let next = origin.offset_by(push_dir, step);
        if let Some(collision) = to_push.iter().position(|pos| *pos == next) {
            reorder_at_collision(to_push, added, collision);
            for index in 0..=(collision + added).min(to_push.len().saturating_sub(1)) {
                let pos = to_push[index];
                if movability.sticky(world, pos).is_some()
                    && !add_branching_blocks(world, movability, piston, push_dir, pos, to_push, to_destroy)
                {
                    return false;
                }
            }
            return true;
        }
        if movability.is_empty(world, next) {
            return true;
        }
        if !movability.is_movable(world, next) || next == piston {
            return false;
        }
        if to_push.len() >= MAX_PUSH_DEPTH {
            return false;
        }
        to_push.push(next);
        added += 1;
        step += 1;
    }
}

/// `PistonStructureResolver.reorderListAtCollision`: the collided run is moved
/// ahead of the line that ran into it.
fn reorder_at_collision(to_push: &mut Vec<Pos>, added: usize, collision: usize) {
    let head: Vec<Pos> = to_push[..collision].to_vec();
    let tail: Vec<Pos> = to_push[to_push.len() - added..].to_vec();
    let middle: Vec<Pos> = to_push[collision..to_push.len() - added].to_vec();
    to_push.clear();
    to_push.extend(head);
    to_push.extend(tail);
    to_push.extend(middle);
}

/// `PistonStructureResolver.addBranchingBlocks`: a sticky block starts a line
/// in every direction perpendicular to the push.
fn add_branching_blocks(
    world: &World,
    movability: &dyn Movability,
    piston: Pos,
    push_dir: Dir,
    pos: Pos,
    to_push: &mut Vec<Pos>,
    to_destroy: &mut Vec<Pos>,
) -> bool {
    let sticky = movability.sticky(world, pos);
    for dir in crate::pos::ALL_DIRS {
        if dir.axis() == push_dir.axis() {
            continue;
        }
        let neighbour = pos.offset(dir);
        if !adheres(sticky, movability.sticky(world, neighbour)) {
            continue;
        }
        if movability.is_empty(world, neighbour) {
            continue;
        }
        if !add_block_line(world, movability, piston, push_dir, neighbour, dir, to_push, to_destroy) {
            return false;
        }
    }
    true
}

/// Work out what a sticky piston retracting would pull back.
///
/// `start` is the block directly in front of the head and `dir` points back toward
/// the piston. Unlike a push there is no column ahead to shove — only the pulled
/// block and whatever adheres to it — so a blocked destination simply means that
/// piece stays put rather than cancelling the retraction.
pub fn resolve_pull(world: &World, movability: &dyn Movability, piston: Pos, facing: Dir) -> PushPlan {
    // `PistonStructureResolver` has no separate retract path. Constructed with
    // `extending = false` it sets `pushDirection = dir.getOpposite()` and
    // `startPos = pos.relative(dir, 2)`, and `resolve()` is then the same
    // method: one block line from the start, then a branching pass over every
    // sticky block it collected.
    //
    // A breadth-first walk over sticky neighbours stood in for this and is not
    // the same shape. It grows one block at a time where the game grows a whole
    // *line* — so a slime block dragged sideways brought only its immediate
    // neighbour along, not the row that neighbour is stuck to — and it filters
    // on the destination being free, which rejects a block whose destination is
    // occupied by another block moving out of the way in the same stroke. The
    // vault door opens on exactly that: a pulled slime block sticks to a second
    // slime block a row below, and *that* one carries a piston and a panel.
    let push_dir = facing.opposite();
    let start = piston.offset(facing).offset(facing);
    let failed = PushPlan { to_push: Vec::new(), to_destroy: Vec::new(), possible: false };

    if movability.is_empty(world, start) || !movability.is_movable(world, start) {
        return failed;
    }
    let mut to_push: Vec<Pos> = Vec::new();
    let mut to_destroy: Vec<Pos> = Vec::new();
    if !add_block_line(
        world,
        movability,
        piston,
        push_dir,
        start,
        push_dir,
        &mut to_push,
        &mut to_destroy,
    ) {
        return failed;
    }
    let mut index = 0;
    while index < to_push.len() {
        let pos = to_push[index];
        if movability.sticky(world, pos).is_some()
            && !add_branching_blocks(
                world,
                movability,
                piston,
                push_dir,
                pos,
                &mut to_push,
                &mut to_destroy,
            )
        {
            return failed;
        }
        index += 1;
    }

    PushPlan { possible: true, to_push, to_destroy }
}

/// A piston.
///
/// One instance per distinct block state, so it knows its own facing and extension
/// without parsing anything at tick time.
/// A piston head — `PistonHeadBlock`.
///
/// It has one job here: `neighborChanged` forwards the update to the base
/// behind it. A head is the only face many circuits touch, so without the
/// forward an extended piston never hears that its power has gone and stays
/// out — which is exactly how a door that hands a redstone block along stalls
/// after its first move.
pub struct PistonHead {
    /// The `facing` property: the head points away from its base.
    pub facing: Dir,
}

impl BlockBehaviour for PistonHead {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let base = pos.offset(self.facing.opposite());
        ctx.notify(base, self.facing);
    }

    fn name(&self) -> &'static str {
        "piston_head"
    }
}

pub struct Piston<P: PowerSource, M: Movability> {
    /// The direction the piston pushes.
    pub facing: Dir,
    /// Whether this state is the extended one.
    pub extended: bool,
    /// Whether this is a sticky piston.
    pub sticky: bool,
    /// Retracted/extended states.
    pub states: StatePair,
    /// The block placed as the piston head when extended.
    pub head: StateId,
    /// The `moving_piston` placeholder for the head slot.
    ///
    /// Carries the piston's own type: `triggerEvent` builds it with
    /// `TYPE = sticky ? STICKY : DEFAULT`.
    pub moving: StateId,
    /// The `moving_piston` placeholder for a pushed or pulled block.
    ///
    /// **Always `type=normal`**, even for a sticky piston — `moveBlocks` sets only
    /// `FACING` on the placeholders it writes, leaving `TYPE` at its default.
    /// Captured: a sticky piston's pull wrote `moving_piston[...,type=normal]`
    /// over the sticky-typed head placeholder.
    pub moving_block: StateId,
    /// How power is read.
    pub power: P,
    /// Which blocks may be pushed.
    pub movability: M,
}

impl<P: PowerSource, M: Movability> Piston<P, M> {
    /// Whether the piston is powered, **including quasi-connectivity**.
    ///
    /// A piston reads power from its own neighbours *and* from the neighbours of the
    /// block directly above it. Confirmed by a captured trace: a redstone block
    /// placed adjacent only to the space above a piston — touching the piston
    /// nowhere — extended it anyway.
    ///
    /// QC is not a quirk to be tidied away; a great many door designs depend on it,
    /// and a simulator without it silently disagrees with the game on exactly the
    /// builds people care about.
    fn is_powered(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
    ) -> bool {
        // `getNeighborSignal` skips the direction the piston pushes at its own
        // position, and skips Down (back toward the piston) at the position
        // above. Read from the bytecode.
        self.has_direct_signal(world, outs, pos, Some(self.facing))
            || self.has_direct_signal(world, outs, pos.offset(Dir::Up), Some(Dir::Down))
    }

    /// Whether any neighbour of `pos` emits toward it, ignoring `skip`.
    fn has_direct_signal(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
        skip: Option<Dir>,
    ) -> bool {
        crate::pos::ALL_DIRS
            .iter()
            .filter(|dir| Some(**dir) != skip)
            .any(|dir| {
                self.power
                    .is_powered(world, outs, pos.offset(*dir), dir.opposite())
            })
    }
}

impl<P: PowerSource, M: Movability> BlockBehaviour for Piston<P, M> {
    /// `PistonBaseBlock.onPlace` runs `checkIfExtend` — a piston that is
    /// already powered when it is put down queues its extend immediately,
    /// without waiting for any neighbour to change.
    ///
    /// This is what starts a community build that was saved mid-cycle: under
    /// `knownShape` placement, where the game dispatches no neighbour or shape
    /// updates at all, the tick-0 activity comes entirely from here.
    fn on_placed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        self.on_neighbor_changed(ctx, pos, self.facing);
    }

    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let powered = self.is_powered(ctx.world, ctx.comparator_out, pos);
        if powered == self.extended {
            return;
        }
        // Straight to a block event, exactly as PistonBaseBlock does — no scheduled
        // tick. This is why a piston notified in phase 3 moves in phase 7 of the
        // same tick rather than the next one.
        let trigger = if powered {
            // `checkIfExtend` resolves the structure **before** queueing:
            // an extend that could not move anything is never queued at all.
            // Queueing it and letting dispatch refuse it looks equivalent only
            // while refused events are dropped — once they are rescheduled the
            // way the game reschedules them, a phantom event queued here
            // survives and fires a tick later.
            if !resolve_push(ctx.world, &self.movability, pos, self.facing).possible {
                return;
            }
            TRIGGER_EXTEND
        } else {
            // `checkIfExtend` retracts with TRIGGER_DROP instead of
            // TRIGGER_CONTRACT when the block beyond the head is still mid-flight
            // toward it — that is the short-pulse drop: the retraction refuses to
            // pull a block whose extension it interrupted.
            let target = pos.offset(self.facing).offset(self.facing);
            let target_in_flight = ctx.moves.iter().any(|m| m.pos == target);
            if target_in_flight {
                TRIGGER_DROP
            } else {
                TRIGGER_CONTRACT
            }
        };
        ctx.queue_event(pos, trigger, self.facing as u8);
    }

    fn on_block_event(&self, ctx: &mut TickCtx<'_>, pos: Pos, id: u8, _param: u8) -> bool {
        match id {
            TRIGGER_EXTEND => {
                // `triggerEvent` re-reads the signal at dispatch: an extend whose
                // power vanished between phase 3 (queueing) and phase 7 (here) is
                // simply dropped. Captured with the manual engine — a landed
                // piston queues an extend off an observer's pulse, the pulse ends
                // in the next tick's block-ticks phase, and the extend never runs.
                if !self.is_powered(ctx.world, ctx.comparator_out, pos) {
                    return false;
                }
                let plan = resolve_push(ctx.world, &self.movability, pos, self.facing);
                if std::env::var("MC_TICK_TRACE_EVENTS").is_ok() {
                    let names: Vec<String> = plan
                        .to_push
                        .iter()
                        .map(|p| {
                            let d = ctx.states.descriptor(ctx.world.get(*p)).unwrap_or("?");
                            format!("{:?}{}", (p.x, p.y, p.z), d.trim_start_matches("minecraft:"))
                        })
                        .collect();
                    eprintln!(
                        "[t{}] plan {:?} possible={} n={} [{}]",
                        ctx.tick,
                        (pos.x, pos.y, pos.z),
                        plan.possible,
                        plan.to_push.len(),
                        names.join(", ")
                    );
                }
                if !plan.possible {
                    return false;
                }
                // Read every source state *before* writing anything: the writes
                // below overwrite positions that later entries still need to read.
                let carried: Vec<(Pos, StateId)> = plan
                    .to_push
                    .iter()
                    .map(|from| (*from, ctx.world.get(*from)))
                    .collect();

                // Vanilla replaces both ends with `moving_piston` placeholders now
                // and resolves them two ticks later in the block-entities phase.
                let head_slot = pos.offset(self.facing);
                // A position can be both a source and a destination: in a column,
                // every block but the last moves into a slot another block just
                // vacated. Clearing those would wipe what had only just arrived, so
                // only positions that purely lose a block become air.
                let destinations: Vec<Pos> = carried
                    .iter()
                    .map(|(from, _)| from.offset(self.facing))
                    .collect();

                // No move *write* notifies neighbours. `moveBlocks` ends with
                // an explicit notification pass instead — `updateNeighborsAt`
                // for every position a block left, then the head slot — which
                // this engine does not run yet. Adding it here is not enough:
                // vanilla dispatches those updates *inside* `moveBlocks`, before
                // `triggerEvent` writes the base as extended, and queueing them
                // from here shows the notified blocks a piston that has already
                // finished moving. Doing it faithfully needs a mid-handler
                // drain; queued from here it reddens the manual-engine goldens.
                //
                // The two kinds of write differ in *shape* though, and it shows.
                // Vacating a source is flag 18 or 82, both carrying
                // `UPDATE_KNOWN_SHAPE`: fully silent. Writing a `moving_piston`
                // placeholder is flag 324, which does not — so the placeholder
                // appearing beside an observer gives it a shape update and it
                // pulses two ticks later, on the tick the block is still in
                // flight rather than the tick it lands.
                // A push breaks these outright — `setBlock(pos, AIR, 276)`,
                // which carries UPDATE_KNOWN_SHAPE and so says nothing on its
                // own. The notification comes from the tail pass below.
                for pos in plan.to_destroy.clone() {
                    ctx.set_quiet(pos, StateId::AIR);
                }
                for (from, state) in carried.iter().rev() {
                    let to = from.offset(self.facing);
                    // A vacated source becomes **air**, not a placeholder — captured
                    // from vanilla, where a slime block's dragged neighbours left
                    // `stone -> air` behind them. Only the slot directly in front of
                    // the piston holds a placeholder, because that one resolves into
                    // the head rather than emptying.
                    if *from != head_slot && !destinations.contains(from) {
                        ctx.set_quiet(*from, StateId::AIR);
                    }
                    ctx.set_shape_only(to, self.moving_block);
                    // Each placeholder's shape updates run before the next one
                    // is written. `moveBlocks` is not itself dispatching a
                    // neighbour update, so `CollectingNeighborUpdater.addAndRun`
                    // finds an empty stack and runs each write's shape pass on
                    // the spot. Batching them to the end of the loop instead
                    // shows a block its own slot already overwritten: an
                    // observer being pushed hears the placeholder land in front
                    // of it and books a tick *before* the head placeholder
                    // takes its square, and that booking then strands at a
                    // position the observer no longer occupies.
                    ctx.drain();
                    ctx.defer(to, *state, PISTON_MOVE_TICKS);
                }
                // The head slot is itself in motion until the move completes.
                ctx.set_shape_only(head_slot, self.moving);
                ctx.drain();
                ctx.defer(head_slot, self.head, PISTON_MOVE_TICKS);

                // `moveBlocks`' tail: `updateNeighborsAt` for every position a
                // block *left*, walked backwards like the write loop, then the
                // head slot. This is how a piston hears about its own push —
                // the block it shoved away may have been powering something,
                // and no move write carries a neighbour update.
                //
                // Drained here rather than left to the driver: vanilla
                // dispatches these from inside `moveBlocks`, while the base
                // still reads `extended=false`. Deferring them until after the
                // base write shows every notified block a piston that has
                // already finished moving, which is a different world.
                // Before any of that, the shape pass over the vacated slots.
                // They were cleared with flag 82 — `UPDATE_KNOWN_SHAPE` set, so
                // the write itself is silent — and `moveBlocks` then calls
                // `updateNeighbourShapes` on each of them explicitly, flag 2.
                // Without it a wire beside a slot a piston empties keeps a
                // connection to a block that is no longer there: it still hears
                // the neighbour update and drops its power, so the fault reads
                // as a stale connection property rather than a missing update.
                for (from, _) in carried.iter().rev() {
                    if *from != head_slot && !destinations.contains(from) {
                        ctx.update_neighbour_shapes(*from);
                        ctx.drain();
                    }
                }
                // Destroyed blocks go first, exactly as `moveBlocks` walks
                // them: the toDestroy loop runs before the vacated-source loop.
                //
                // Each call is dispatched before the next is made. `moveBlocks`
                // calls `Level.updateNeighborsAt` one position at a time, and
                // every one of those finds the updater's stack empty and runs
                // on the spot — so a piston head that forwards to its base does
                // so *between* two of these calls, not after all of them.
                // Batching them and draining once reorders the whole cascade
                // from that point on.
                for pos in plan.to_destroy.iter().rev() {
                    ctx.update_neighbors_at(*pos);
                    ctx.drain();
                }
                for (from, _) in carried.iter().rev() {
                    ctx.update_neighbors_at(*from);
                    ctx.drain();
                }
                ctx.update_neighbors_at(head_slot);
                ctx.drain();

                // The base state is written *after* the moves, with notifications
                // (vanilla flag 67) — the one loud write of the whole event.
                ctx.set(pos, self.states.get(true));
                true
            }
            TRIGGER_CONTRACT | TRIGGER_DROP => {
                // `MC_TICK_TRACE_POWER=x,y,z` — the whole of `getNeighborSignal`
                // at the instant a retract is dispatched, which is the instant
                // vanilla decides whether to refuse it. Both halves separately,
                // because "powered" and "quasi-powered" fail in different ways.
                if std::env::var_os("MC_TICK_TRACE_POWER").is_some_and(|f| {
                    f.to_string_lossy().split(';').any(|t| {
                        let c: Vec<i32> = t.split(',').filter_map(|v| v.trim().parse().ok()).collect();
                        c.len() == 3 && c[0] == pos.x && c[1] == pos.y && c[2] == pos.z
                    })
                }) {
                    eprintln!(
                        "[t{}] power at {:?} facing={:?} id={id}",
                        ctx.tick,
                        (pos.x, pos.y, pos.z),
                        self.facing
                    );
                    // The local layout at this instant, so the shape of the
                    // dust can be read rather than inferred.
                    for z in (pos.z - 1)..=(pos.z + 1) {
                        eprintln!("    --- z={z}");
                        for y in ((pos.y - 2)..=(pos.y + 2)).rev() {
                            let row: Vec<String> = ((pos.x - 2)..=(pos.x + 2))
                                .map(|x| {
                                    let d = ctx
                                        .states
                                        .descriptor(ctx.world.get(Pos::new(x, y, z)))
                                        .unwrap_or("?")
                                        .trim_start_matches("minecraft:");
                                    let mark = if (x, y, z) == (pos.x, pos.y, pos.z) { "*" } else { " " };
                                    format!("{mark}{:<38}", &d[..d.len().min(38)])
                                })
                                .collect();
                            eprintln!("    y{y:<3}{}", row.join(""));
                        }
                    }
                    for (label, base, skip) in
                        [("direct", pos, Some(self.facing)), ("qc", pos.offset(Dir::Up), Some(Dir::Down))]
                    {
                        for dir in crate::pos::ALL_DIRS {
                            if Some(dir) == skip {
                                continue;
                            }
                            let at = base.offset(dir);
                            let emits =
                                self.power.is_powered(ctx.world, ctx.comparator_out, at, dir.opposite());
                            eprintln!(
                                "    {label:<7}{dir:?} {:?} {} => {}",
                                (at.x, at.y, at.z),
                                ctx.states.descriptor(ctx.world.get(at)).unwrap_or("?"),
                                if emits { "POWERS" } else { "-" }
                            );
                        }
                    }
                }
                // Dispatch re-check, mirroring extend: if power returned before
                // the retract ran, vanilla re-marks the base extended and treats
                // the event as unhandled.
                //
                // The write is flag **2**, which is not silence. It withholds
                // the neighbour updates and still runs the shape pass over the
                // six neighbours, because `UPDATE_KNOWN_SHAPE` is clear — so an
                // observer beside a piston sees a refused retract even though no
                // block around it is told about one.
                if self.is_powered(ctx.world, ctx.comparator_out, pos) {
                    ctx.set_shape_only(pos, self.states.get(true));
                    return false;
                }
                let head = pos.offset(self.facing);
                // An in-flight head is `finalTick`ed first: a *source* block
                // entity resolves to air (not to its head state), loudly — which
                // is also what frees the slot for a pull to move into.
                let head_in_flight = ctx.moves.iter().any(|m| m.pos == head);
                ctx.moves.retain(|m| m.pos != head);
                if head_in_flight {
                    ctx.set(head, StateId::AIR);
                }

                // The *base* becomes a moving placeholder for the two ticks the
                // head takes to travel home — captured: `extended=true ->
                // moving_piston -> extended=false`. Vanilla writes it silently and
                // then fires updateNeighborsAt explicitly; ctx.set is exactly
                // that pair.
                // `triggerEvent` begins a retraction by finalising an
                // in-flight *head*: a piston that extended less than two ticks
                // ago still has a moving block entity in the head slot, and the
                // very first thing the retract branch does is `finalTick` it —
                // before the base is written and long before the block two
                // ahead is looked at.
                //
                // The order is observable. Landing the head writes it loudly,
                // and that notification reaches the block under the head while
                // the block two ahead is *still in flight*. Finalising that one
                // first instead — as this did — puts a solid block in the
                // quasi-connectivity ring a tick early, and a piston below
                // reads power vanilla never gives it.
                if let Some(index) = ctx.moves.iter().position(|m| m.pos == head) {
                    let landed = ctx.moves.remove(index);
                    ctx.set(head, landed.state);
                    ctx.drain();
                    // `finalTick` ends with `neighborChanged` at its own
                    // position, whether it runs from the block-entity phase or
                    // from here inside `triggerEvent`. Same call, same place in
                    // the sequence — after the write's own notifications.
                    ctx.notify(head, Dir::Down);
                    ctx.drain();
                }
                ctx.set(pos, self.moving);
                ctx.defer(pos, self.states.get(false), PISTON_MOVE_TICKS);

                if self.sticky {
                    let back = self.facing.opposite();
                    let target = head.offset(self.facing);
                    let target_pending = ctx.moves.iter().position(|m| m.pos == target);
                    let target_state = ctx.world.get(target);
                    let target_moving =
                        target_state == self.moving || target_state == self.moving_block;
                    if let (Some(index), true) = (target_pending, target_moving) {
                        // The block we would pull is still travelling toward the
                        // head: it is finalised where it is and *not* pulled — the
                        // short-pulse drop.
                        let landed = ctx.moves.remove(index);
                        ctx.set(target, landed.state);
                        ctx.set(head, StateId::AIR);
                    } else if id == TRIGGER_CONTRACT {
                        // `moveBlocks` begins a retraction by silently clearing a
                        // real head out of the slot the pulled block moves into.
                        if ctx.world.get(head) == self.head {
                            ctx.set_quiet(head, StateId::AIR);
                        }
                        // A pulled slime block drags its own neighbours exactly as
                        // a pushed one does. The return stroke matters as much as
                        // the push for doors.
                        let plan = resolve_pull(ctx.world, &self.movability, pos, self.facing);
                        if plan.possible {
                            let carried: Vec<(Pos, StateId)> = plan
                                .to_push
                                .iter()
                                .map(|from| (*from, ctx.world.get(*from)))
                                .collect();
                            let destinations: Vec<Pos> = carried
                                .iter()
                                .map(|(from, _)| from.offset(back))
                                .collect();
                            for (from, state) in &carried {
                                let to = from.offset(back);
                                // Flag 324, the same as a push: the placeholder
                                // write propagates shape, and does so before the
                                // next one is written.
                                ctx.set_shape_only(to, self.moving_block);
                                ctx.drain();
                                ctx.defer(to, *state, PISTON_MOVE_TICKS);
                            }
                            for (from, _) in &carried {
                                if !destinations.contains(from) {
                                    ctx.set_quiet(*from, StateId::AIR);
                                }
                            }
                            // Flag 82 is silent, so `moveBlocks` runs the shape
                            // pass over the vacated slots by hand, before any of
                            // the neighbour updates below.
                            for (from, _) in carried.iter().rev() {
                                if !destinations.contains(from) {
                                    ctx.update_neighbour_shapes(*from);
                                }
                            }
                            // `moveBlocks`' tail again — a pull vacates
                            // positions exactly as a push does, and the game
                            // notifies them the same way. Only the head-slot
                            // notification is skipped, which vanilla guards
                            // with `if (extending)`.
                            //
                            // This is the step that makes a 0-tick generator
                            // tick: the block pulled out from under a piston
                            // was powering it, and until those neighbours are
                            // told, the piston above it never learns its pulse ended.
                            for (from, _) in carried.iter().rev() {
                                ctx.update_neighbors_at(*from);
                                ctx.drain();
                            }
                        } else {
                            ctx.set(head, StateId::AIR);
                        }
                    } else {
                        // TRIGGER_DROP with nothing in flight: retract without
                        // pulling.
                        ctx.set(head, StateId::AIR);
                    }
                } else {
                    ctx.set(head, StateId::AIR);
                }
                true
            }
            _ => false,
        }
    }

    fn name(&self) -> &'static str {
        if self.sticky {
            "sticky_piston"
        } else {
            "piston"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phase::Phase;
    use crate::pos::Bounds;
    use crate::schedule::{EventQueue, TickQueue};
    use crate::state::StateRegistry;

    #[derive(Clone)]
    struct Model {
        powered: Vec<StateId>,
        immovable: Vec<StateId>,
        slime: Vec<StateId>,
        honey: Vec<StateId>,
    }

    impl PowerSource for Model {
        fn is_powered(
            &self,
            world: &World,
            _outs: &crate::behaviour::ComparatorOutputs,
            pos: Pos,
            _toward: Dir,
        ) -> bool {
            self.powered.contains(&world.get(pos))
        }
        fn is_diode(&self, _world: &World, _pos: Pos) -> bool {
            false
        }
        fn diode_facing(&self, _world: &World, _pos: Pos) -> Option<Dir> {
            None
        }
    }

    impl Movability for Model {
        fn is_movable(&self, world: &World, pos: Pos) -> bool {
            let state = world.get(pos);
            state != StateId::AIR && !self.immovable.contains(&state)
        }
        fn sticky(&self, world: &World, pos: Pos) -> Option<Sticky> {
            let state = world.get(pos);
            if self.slime.contains(&state) {
                Some(Sticky::Slime)
            } else if self.honey.contains(&state) {
                Some(Sticky::Honey)
            } else {
                None
            }
        }
    }

    const RETRACTED: StateId = StateId(1);
    const EXTENDED: StateId = StateId(2);
    const HEAD: StateId = StateId(3);
    const STONE: StateId = StateId(4);
    const OBSIDIAN: StateId = StateId(5);
    const LEVER: StateId = StateId(6);
    const MOVING: StateId = StateId(7);
    const SLIME: StateId = StateId(8);
    const HONEY: StateId = StateId(9);

    fn piston(extended: bool, sticky: bool) -> Piston<Model, Model> {
        let model = Model {
            powered: vec![LEVER],
            immovable: vec![OBSIDIAN, MOVING],
            slime: vec![SLIME],
            honey: vec![HONEY],
        };
        Piston {
            facing: Dir::East,
            extended,
            sticky,
            states: StatePair { off: RETRACTED, on: EXTENDED },
            head: HEAD,
            moving: MOVING,
            moving_block: MOVING,
            power: model.clone(),
            movability: model,
        }
    }

    fn world() -> World {
        World::new(Bounds::new(Pos::new(-2, 0, -2), Pos::new(24, 4, 2)))
    }

    fn run<'a>(
        world: &'a mut World,
        ticks: &'a mut TickQueue,
        events: &'a mut EventQueue,
        states: &'a StateRegistry,
    ) -> TickCtx<'a> {
        TickCtx { drain: None, behaviours: None, world, ticks, events, states, tick: 0,
            boundary: false,
        fluids: Box::leak(Box::new(TickQueue::new())),
        updates: Box::leak(Box::new(Vec::new())),
        moves: Box::leak(Box::new(Vec::new())),
        toggles: Box::leak(Box::new(Vec::new())),
        comparator_out: Box::leak(Box::new(Default::default())),
        inventories: Box::leak(Box::new(Default::default())),
        hopper_state: Box::leak(Box::new(Default::default())),
        item_entities: Box::leak(Box::new(Default::default())),
        inv_log: None, log: None }
    }

    #[test]
    fn max_push_depth_is_twelve() {
        // Inlined by javac so it could not be read from the bytecode. Asserted here
        // so a wrong value fails loudly rather than quietly truncating a build.
        assert_eq!(MAX_PUSH_DEPTH, 12);
    }

    #[test]
    fn a_piston_queues_a_block_event_rather_than_scheduling_a_tick() {
        // The load-bearing timing fact: PistonBaseBlock calls Level.blockEvent
        // directly, so the move lands in phase 7 of the same tick. Treating it as a
        // scheduled tick would report every door a tick slow.
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(pos.offset(Dir::Up), LEVER);

        let p = piston(false, false);
        let mut ctx = run(&mut w, &mut t, &mut e, &s);
        p.on_neighbor_changed(&mut ctx, pos, Dir::Up);

        assert_eq!(e.len(), 1, "must queue a block event");
        assert!(t.is_empty(), "must NOT schedule a block tick");
        assert!(Phase::BlockTicks < Phase::BlockEvents, "and events run later");
    }

    #[test]
    fn extending_pushes_a_column_and_places_the_head() {
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(Pos::new(1, 1, 0), STONE);
        w.set(Pos::new(2, 1, 0), STONE);
        // Power the piston: triggerEvent re-reads the signal at dispatch and
        // drops an extend whose power has vanished.
        w.set(pos.offset(Dir::Up), LEVER);

        let p = piston(false, false);
        let mut ctx_moves = Vec::new();
        {
            let mut ctx = TickCtx {
                drain: None,
                behaviours: None,
                world: &mut w, ticks: &mut t, fluids: &mut TickQueue::new(), events: &mut e, states: &s, tick: 0,
            boundary: false,
                updates: &mut Vec::new(), moves: &mut ctx_moves,
                toggles: &mut Vec::new(),
                comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
                log: None,
            };
            assert!(p.on_block_event(&mut ctx, pos, TRIGGER_EXTEND, 0));
        }

        // Immediately: the piston is extended and the moved blocks are placeholders.
        assert_eq!(w.get(pos), EXTENDED);
        assert_eq!(w.get(Pos::new(1, 1, 0)), MOVING, "head slot is in motion");
        assert_eq!(w.get(Pos::new(2, 1, 0)), MOVING);
        assert_eq!(w.get(Pos::new(3, 1, 0)), MOVING);

        // The real states are deferred to the block-entities phase, two ticks out,
        // exactly as the captured trace shows.
        let mut resolved: Vec<_> = ctx_moves.iter().map(|m| (m.pos, m.state, m.resolve_on)).collect();
        resolved.sort_by_key(|(p, _, _)| (p.x, p.y, p.z));
        assert!(resolved.iter().all(|(_, _, on)| *on == PISTON_MOVE_TICKS),
            "all writes land {PISTON_MOVE_TICKS} ticks later: {resolved:?}");
        assert_eq!(resolved[0].1, HEAD, "head resolves into the first slot");
        assert_eq!(resolved[1].1, STONE, "column lands one east");
    }

    #[test]
    fn an_immovable_block_blocks_the_whole_push() {
        let mut w = world();
        w.set(Pos::new(0, 1, 0), RETRACTED);
        w.set(Pos::new(1, 1, 0), STONE);
        w.set(Pos::new(2, 1, 0), OBSIDIAN);

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, Pos::new(0, 1, 0), Dir::East);
        assert!(!plan.possible);
        assert!(plan.to_push.is_empty(), "nothing moves, not even the movable part");
    }

    #[test]
    fn a_column_longer_than_the_push_limit_refuses_to_move() {
        let mut w = world();
        w.set(Pos::new(0, 1, 0), RETRACTED);
        for x in 1..=(MAX_PUSH_DEPTH as i32 + 1) {
            w.set(Pos::new(x, 1, 0), STONE);
        }

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, Pos::new(0, 1, 0), Dir::East);
        assert!(!plan.possible, "13 blocks is one too many");
    }

    #[test]
    fn exactly_the_push_limit_still_moves() {
        let mut w = world();
        w.set(Pos::new(0, 1, 0), RETRACTED);
        for x in 1..=(MAX_PUSH_DEPTH as i32) {
            w.set(Pos::new(x, 1, 0), STONE);
        }

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, Pos::new(0, 1, 0), Dir::East);
        assert!(plan.possible, "12 blocks is exactly allowed");
        assert_eq!(plan.to_push.len(), MAX_PUSH_DEPTH);
    }

    #[test]
    fn the_push_plan_is_ordered_nearest_first() {
        // `PistonStructureResolver` builds outward from the piston, and
        // `moveBlocks` then walks the list backwards — so applying it is still a
        // simple loop where each block moves into space the previous write
        // vacated, but the list itself is in vanilla's order. The distinction is
        // observable: moving block entities land in creation order, and each
        // landing notifies its neighbours.
        let mut w = world();
        w.set(Pos::new(0, 1, 0), RETRACTED);
        for x in 1..=3 {
            w.set(Pos::new(x, 1, 0), STONE);
        }
        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, Pos::new(0, 1, 0), Dir::East);
        assert_eq!(
            plan.to_push,
            vec![Pos::new(1, 1, 0), Pos::new(2, 1, 0), Pos::new(3, 1, 0)]
        );
    }

    #[test]
    fn a_sticky_piston_drags_its_block_back_on_retract() {
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, EXTENDED);
        w.set(Pos::new(1, 1, 0), HEAD);
        w.set(Pos::new(2, 1, 0), STONE);

        let p = piston(true, true);
        let mut pulled = Vec::new();
        {
            let mut ctx = TickCtx {
                drain: None,
                behaviours: None,
                world: &mut w, ticks: &mut t, fluids: &mut TickQueue::new(), events: &mut e, states: &s, tick: 0,
            boundary: false,
                updates: &mut Vec::new(), moves: &mut pulled, toggles: &mut Vec::new(),
                comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
                log: None,
            };
            assert!(p.on_block_event(&mut ctx, pos, TRIGGER_CONTRACT, 0));
        }

        // Retraction travels like extension: placeholders now, real states in the
        // block-entities phase two ticks later. The *base* is one of them —
        // captured: extended=true -> moving_piston -> extended=false.
        assert_eq!(w.get(pos), MOVING, "the base itself is in motion");
        assert!(
            pulled.iter().any(|m| m.pos == pos
                && m.state == RETRACTED
                && m.resolve_on == PISTON_MOVE_TICKS),
            "the base must resolve to retracted: {pulled:?}"
        );
        assert_eq!(w.get(Pos::new(1, 1, 0)), MOVING, "head slot is in motion");
        assert!(
            pulled.iter().any(|m| m.pos == Pos::new(1, 1, 0)
                && m.state == STONE
                && m.resolve_on == PISTON_MOVE_TICKS),
            "the stone must be scheduled into the head slot: {pulled:?}"
        );
        assert_eq!(
            w.get(Pos::new(2, 1, 0)),
            StateId::AIR,
            "the pulled block's old position empties, as the capture shows"
        );
    }

    #[test]
    fn a_sticky_piston_pulls_a_slime_structure_back_whole() {
        // The return stroke matters as much as the push: a pulled slime block drags
        // its neighbours exactly as a pushed one does.
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, EXTENDED);
        w.set(Pos::new(1, 1, 0), HEAD);
        w.set(Pos::new(2, 1, 0), SLIME);
        w.set(Pos::new(2, 2, 0), STONE); // stuck on top of the slime

        let p = piston(true, true);
        let mut pulled = Vec::new();
        {
            let mut ctx = TickCtx {
                drain: None,
                behaviours: None,
                world: &mut w, ticks: &mut t, fluids: &mut TickQueue::new(), events: &mut e, states: &s, tick: 0,
            boundary: false,
                updates: &mut Vec::new(), moves: &mut pulled, toggles: &mut Vec::new(),
                comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
                log: None,
            };
            p.on_block_event(&mut ctx, pos, TRIGGER_CONTRACT, 0);
        }

        assert!(
            pulled.iter().any(|m| m.pos == Pos::new(1, 1, 0) && m.state == SLIME),
            "slime pulled into the head slot: {pulled:?}"
        );
        assert!(
            pulled.iter().any(|m| m.pos == Pos::new(1, 2, 0) && m.state == STONE),
            "and the block stuck to it came along: {pulled:?}"
        );
    }

    #[test]
    fn a_sticky_piston_drops_its_block_on_a_short_pulse() {
        // Captured from vanilla with a one-tick pulse:
        //   tick 0  piston extends; stone -> moving_piston
        //   tick 1  piston starts retracting while the stone is STILL MOVING
        //   final   stone left at its pushed position, not pulled back
        // Against a four-tick pulse, where the extension completes at tick 2 and the
        // stone is pulled home at tick 6.
        //
        // The cause is PISTON_MOVE_TICKS: retraction begins before the extension
        // finishes, so there is no settled block to grab. Our model reproduces it
        // because a block in motion is immovable, so the pull simply finds nothing.
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, EXTENDED);
        w.set(Pos::new(1, 1, 0), HEAD);
        w.set(Pos::new(2, 1, 0), MOVING); // still in flight

        let p = piston(true, true);
        let mut pulled = Vec::new();
        {
            let mut ctx = TickCtx {
                drain: None,
                behaviours: None,
                world: &mut w, ticks: &mut t, fluids: &mut TickQueue::new(), events: &mut e, states: &s, tick: 0,
            boundary: false,
                updates: &mut Vec::new(), moves: &mut pulled, toggles: &mut Vec::new(),
                comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            item_entities: &mut Default::default(),
            inv_log: None,
                log: None,
            };
            p.on_block_event(&mut ctx, pos, TRIGGER_CONTRACT, 0);
        }

        assert!(
            pulled.iter().all(|m| m.pos == pos),
            "a block still in motion cannot be grabbed — only the base travels: {pulled:?}"
        );
        assert_eq!(w.get(pos), MOVING, "the piston still retracts, via its placeholder");
        assert_eq!(w.get(Pos::new(2, 1, 0)), MOVING, "the dropped block is left in flight");
    }

    #[test]
    fn a_plain_piston_leaves_the_block_behind_on_retract() {
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, EXTENDED);
        w.set(Pos::new(1, 1, 0), HEAD);
        w.set(Pos::new(2, 1, 0), STONE);

        let p = piston(true, false);
        let mut ctx = run(&mut w, &mut t, &mut e, &s);
        p.on_block_event(&mut ctx, pos, TRIGGER_CONTRACT, 0);

        assert_eq!(w.get(Pos::new(1, 1, 0)), StateId::AIR, "head just vanishes");
        assert_eq!(w.get(Pos::new(2, 1, 0)), STONE, "block stays put");
    }

    #[test]
    fn an_unpowered_retracted_piston_does_nothing() {
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);

        let p = piston(false, false);
        let mut ctx = run(&mut w, &mut t, &mut e, &s);
        p.on_neighbor_changed(&mut ctx, pos, Dir::Up);
        assert!(e.is_empty(), "no power, no event");
    }

    #[test]
    fn slime_drags_blocks_on_every_face() {
        // Captured from vanilla: pushing a slime block east also moved the stone
        // above it and the stone below it, neither of which the piston touched.
        let mut w = world();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(Pos::new(1, 1, 0), SLIME);
        w.set(Pos::new(1, 2, 0), STONE); // above the slime
        w.set(Pos::new(1, 0, 0), STONE); // below the slime

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, pos, Dir::East);

        assert!(plan.possible);
        assert!(plan.to_push.contains(&Pos::new(1, 1, 0)), "the slime itself");
        assert!(plan.to_push.contains(&Pos::new(1, 2, 0)), "dragged from above");
        assert!(plan.to_push.contains(&Pos::new(1, 0, 0)), "dragged from below");
    }

    #[test]
    fn a_dragged_block_pushes_whatever_it_runs_into() {
        // Also captured: the stone dragged from beneath the slime shoved the stone
        // already sitting in its path one further east. Adhesion starts new push
        // lines; it does not merely translate blocks.
        let mut w = world();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(Pos::new(1, 1, 0), SLIME);
        w.set(Pos::new(1, 0, 0), STONE);
        w.set(Pos::new(2, 0, 0), STONE); // in the dragged block's way

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, pos, Dir::East);

        assert!(plan.possible);
        assert!(
            plan.to_push.contains(&Pos::new(2, 0, 0)),
            "the block the dragged one runs into must move too: {:?}",
            plan.to_push
        );
    }

    #[test]
    fn slime_and_honey_do_not_stick_to_each_other() {
        // The one rule that separates them, and builds depend on it to keep two
        // halves of a contraption apart.
        let mut w = world();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(Pos::new(1, 1, 0), SLIME);
        w.set(Pos::new(1, 2, 0), HONEY);

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, pos, Dir::East);

        assert!(plan.possible);
        assert!(
            !plan.to_push.contains(&Pos::new(1, 2, 0)),
            "honey must not be dragged by slime: {:?}",
            plan.to_push
        );
    }

    #[test]
    fn honey_drags_ordinary_blocks_just_like_slime() {
        let mut w = world();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(Pos::new(1, 1, 0), HONEY);
        w.set(Pos::new(1, 2, 0), STONE);

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, pos, Dir::East);
        assert!(plan.to_push.contains(&Pos::new(1, 2, 0)));
    }

    #[test]
    fn slime_touching_the_same_kind_still_sticks() {
        let mut w = world();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(Pos::new(1, 1, 0), SLIME);
        w.set(Pos::new(1, 2, 0), SLIME);

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, pos, Dir::East);
        assert!(plan.to_push.contains(&Pos::new(1, 2, 0)), "slime sticks to slime");
    }

    #[test]
    fn an_immovable_block_beside_slime_is_simply_not_dragged() {
        // This test used to assert the opposite — that obsidian stuck to the
        // slime cancels the whole push — which was a guess, and wrong. Asked
        // directly (`capture.sh --probe-push`, structure `slime_obsidian`),
        // the game answers `resolve=true`: `addBlockLine` *returns true* for a
        // block it cannot push, so a branch that runs into obsidian is
        // abandoned, not fatal. Only an immovable block in the push **line**
        // stops a piston.
        let mut w = world();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(Pos::new(1, 1, 0), SLIME);
        w.set(Pos::new(1, 2, 0), OBSIDIAN); // stuck to the slime, cannot move

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, pos, Dir::East);
        assert!(plan.possible, "the branch is abandoned, the push survives");
        assert!(plan.to_push.contains(&Pos::new(1, 1, 0)), "the slime still moves");
        assert!(
            !plan.to_push.contains(&Pos::new(1, 2, 0)),
            "the obsidian is left behind"
        );
    }

    #[test]
    fn a_block_already_in_motion_is_immovable() {
        // moving_piston placeholders cannot be pushed; a piston cannot shove a
        // structure that is still travelling.
        let mut w = world();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(Pos::new(1, 1, 0), MOVING);

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, pos, Dir::East);
        assert!(!plan.possible, "a block in motion must block the push");
    }

    #[test]
    fn a_slime_branch_keeps_its_place_in_the_resolver_list() {
        // With adhesion the moved set is no longer a single line, and a branch
        // pulled sideways has no meaningful position along the push axis. The
        // list keeps the resolver's own order — the line first, then what the
        // slime brought with it — which is what vanilla hands to `moveBlocks`.
        let mut w = world();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        w.set(Pos::new(1, 1, 0), SLIME);
        w.set(Pos::new(2, 1, 0), STONE);
        w.set(Pos::new(1, 2, 0), STONE);

        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, pos, Dir::East);

        assert_eq!(
            plan.to_push,
            vec![Pos::new(1, 1, 0), Pos::new(2, 1, 0), Pos::new(1, 2, 0)],
            "the pushed line first, then the block the slime carries"
        );
    }

    #[test]
    fn quasi_connectivity_powers_a_piston_nothing_touches() {
        // Captured from vanilla: a redstone block adjacent only to the space *above*
        // a piston, touching the piston nowhere, extends it anyway. Many door
        // designs depend on this, so a simulator without it disagrees with the game
        // on exactly the builds people care about.
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        // Diagonally up-and-across: adjacent to pos.above(), not to pos.
        w.set(Pos::new(1, 2, 0), LEVER);

        let p = piston(false, false);
        let mut ctx = run(&mut w, &mut t, &mut e, &s);
        p.on_neighbor_changed(&mut ctx, pos, Dir::Up);

        assert_eq!(e.len(), 1, "QC must power the piston");
    }

    #[test]
    fn a_piston_with_no_signal_anywhere_stays_put() {
        // The other half of QC: it must not power pistons spuriously.
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let pos = Pos::new(0, 1, 0);
        w.set(pos, RETRACTED);
        // Two above: adjacent to neither the piston nor the block above it.
        w.set(Pos::new(1, 3, 0), LEVER);

        let p = piston(false, false);
        let mut ctx = run(&mut w, &mut t, &mut e, &s);
        p.on_neighbor_changed(&mut ctx, pos, Dir::Up);

        assert!(e.is_empty(), "QC reaches one block up, not two");
    }

    #[test]
    fn an_unknown_block_event_is_not_claimed() {
        // TRIGGER_DROP is a real retract variant, so an unknown id has to be an
        // actually-unused number.
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let p = piston(false, false);
        let mut ctx = run(&mut w, &mut t, &mut e, &s);
        assert!(!p.on_block_event(&mut ctx, Pos::new(0, 1, 0), 9, 0));
    }
}
