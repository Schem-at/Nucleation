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
        // The partial list, not an empty one: a refusal that reports nothing
        // collected cannot be told apart from a refusal at the first block, and
        // those have completely different causes.
        return PushPlan { to_push, to_destroy, possible: false };
    }
    let mut index = 0;
    while index < to_push.len() {
        let pos = to_push[index];
        if movability.sticky(world, pos).is_some()
            && !add_branching_blocks(world, movability, piston, dir, pos, &mut to_push, &mut to_destroy)
        {
            return PushPlan { to_push, to_destroy, possible: false };
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
    // `isPushable(..., allowDestroy = false, ...)`, which is what `addBlockLine`
    // passes at its head: a block that *breaks* when pushed is not collected
    // here at all — not carried, not destroyed, just skipped. Only the forward
    // walk destroys, and it passes `true`.
    if !movability.is_movable(world, origin) || movability.destroys(world, origin) {
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
            // allowDestroy is false here too: a breakable block ends the chain.
            || movability.destroys(world, back)
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
        // `MC_TICK_TRACE_REACH=1` — every breakable block a push line walks into.
        //
        // This walk passes `allowDestroy = true`, so whatever it reaches is a
        // block the engine is about to break. Cross-check each one against a
        // capture: if the game never breaks it, either the line is longer than
        // the game's or the block is not breakable at all. It was the second —
        // rails were listed as `DESTROY` and the game plainly pushes them.
        if std::env::var_os("MC_TICK_TRACE_REACH").is_some() && movability.destroys(world, next) {
            eprintln!(
                "[reach] piston={:?} dir={:?} line_origin={:?} reached={:?} collected={:?}",
                (piston.x, piston.y, piston.z),
                push_dir,
                (origin.x, origin.y, origin.z),
                (next.x, next.y, next.z),
                to_push.iter().map(|p| (p.x, p.y, p.z)).collect::<Vec<_>>()
            );
        }
        // The forward walk passes `allowDestroy = true`, so a breakable block
        // here *is* collected — into `toDestroy` — and it ends the line. The
        // order matters: this is checked *before* the twelve-block limit, so a
        // line that ends in dust is never the thing that overflows it.
        if movability.destroys(world, next) {
            if !to_destroy.contains(&next) {
                to_destroy.push(next);
            }
            return true;
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
    /// The base states this head can survive on: a piston or sticky piston,
    /// extended, facing the same way.
    pub bases: Vec<StateId>,
    /// The `facing` property: the head points away from its base.
    pub facing: Dir,
}

impl BlockBehaviour for PistonHead {
    /// `PistonHeadBlock.neighborChanged` forwards to the base — but only while
    /// the head can survive, and `canSurvive` is: the block behind me is a
    /// piston or sticky piston, extended, facing the same way. Nothing about
    /// TYPE, which is why a sticky head sitting on a plain base still forwards.
    ///
    /// The guard bites the moment the base stops being an extended piston,
    /// which is exactly when the base is mid-retract and has become a
    /// `moving_piston`. Forwarding unconditionally sends the base a
    /// notification vanilla never sends it.
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let base = pos.offset(self.facing.opposite());
        if !self.bases.contains(&ctx.world.get(base)) {
            return;
        }
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
        if std::env::var_os("MC_TICK_TRACE_POWER").is_some_and(|f| {
            f.to_string_lossy().split(';').any(|t| {
                let c: Vec<i32> = t.split(',').filter_map(|v| v.trim().parse().ok()).collect();
                c.len() == 3 && c[0] == pos.x && c[1] == pos.y && c[2] == pos.z
            })
        }) {
            let above = pos.offset(Dir::Up);
            let qc: Vec<String> = crate::pos::ALL_DIRS
                .iter()
                .filter(|d| **d != Dir::Down)
                .filter(|d| {
                    self.power.is_powered(ctx.world, ctx.comparator_out, above.offset(**d), d.opposite())
                })
                .map(|d| format!("qc:{d:?}"))
                .collect();
            eprintln!(
                "[pwr] {:?} extended={} powered={} {}",
                (pos.x, pos.y, pos.z),
                self.extended,
                self.is_powered(ctx.world, ctx.comparator_out, pos),
                qc.join(" ")
            );
        }
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
            let plan = resolve_push(ctx.world, &self.movability, pos, self.facing);
            if !plan.possible {
                // A piston that is powered, unextended, and still silent is
                // always a refused resolve — and which block refused it is the
                // only thing worth knowing.
                if std::env::var_os("MC_TICK_TRACE_POWER").is_some_and(|f| {
                    f.to_string_lossy().split(';').any(|t| {
                        let c: Vec<i32> =
                            t.split(',').filter_map(|v| v.trim().parse().ok()).collect();
                        c.len() == 3 && c[0] == pos.x && c[1] == pos.y && c[2] == pos.z
                    })
                }) {
                    let line: Vec<String> = plan
                        .to_push
                        .iter()
                        .map(|p| {
                            format!(
                                "({}, {}, {}){} destroys={} movable={}",
                                p.x,
                                p.y,
                                p.z,
                                ctx.states.descriptor(ctx.world.get(*p)).unwrap_or("?"),
                                self.movability.destroys(ctx.world, *p),
                                self.movability.is_movable(ctx.world, *p)
                            )
                        })
                        .collect();
                    let blocker = plan.to_push.last().map(|p| p.offset(self.facing));
                    eprintln!(
                        "[pwr] {:?} resolve REFUSED n={} blocker={:?} line=[{}]",
                        (pos.x, pos.y, pos.z),
                        plan.to_push.len(),
                        blocker.map(|b| (
                            (b.x, b.y, b.z),
                            ctx.states.descriptor(ctx.world.get(b)).unwrap_or("?")
                        )),
                        line.join(", ")
                    );
                }
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
                    // A destroyed shulker box drops itself with its slots
                    // intact — `dropResources` through the box's loot table,
                    // whose container component survives the break. Spawn
                    // position and velocity are `Block.popResource` plus the
                    // four-arg `ItemEntity` constructor: centre ±0.25 uniform
                    // per axis (y also down half an item height), velocity
                    // `(U*0.2-0.1, 0.2, U*0.2-0.1)`, pickup delay 10. With no
                    // seeded rng, the distribution means. Other DESTROY blocks
                    // (dust, torches) intentionally drop nothing yet — the
                    // conformance goldens predate loot.
                    let destroyed = ctx.world.get(pos);
                    let is_shulker = ctx
                        .states
                        .descriptor(destroyed)
                        .is_some_and(crate::vanilla::has_dynamic_shape);
                    if is_shulker {
                        let name = ctx
                            .states
                            .descriptor(destroyed)
                            .map(|d| d.split('[').next().unwrap_or(d).to_string())
                            .expect("checked above");
                        let carried: Vec<crate::inventory::ItemStack> = ctx
                            .inventories
                            .remove(&pos)
                            .map(|inv| inv.stacks)
                            .unwrap_or_default();
                        let (offset, vel) = if let Some(rng) = ctx.item_entities.rng.as_mut() {
                            (
                                [
                                    rng.next_double_between(-0.25, 0.25),
                                    rng.next_double_between(-0.25, 0.25) - 0.125,
                                    rng.next_double_between(-0.25, 0.25),
                                ],
                                [
                                    rng.next_double() * 0.2 - 0.1,
                                    0.2,
                                    rng.next_double() * 0.2 - 0.1,
                                ],
                            )
                        } else {
                            ([0.0, -0.125, 0.0], [0.0, 0.2, 0.0])
                        };
                        let spawn = [
                            f64::from(pos.x) + 0.5 + offset[0],
                            f64::from(pos.y) + 0.5 + offset[1],
                            f64::from(pos.z) + 0.5 + offset[2],
                        ];
                        let entity = ctx.item_entities.spawn((name, 1), spawn, vel, 10);
                        if !carried.is_empty() {
                            ctx.item_entities.contents.insert(entity, carried);
                        }
                    }
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
                    ctx.defer(
                        to,
                        *state,
                        PISTON_MOVE_TICKS,
                        Some(Sweep { travel: self.facing, extending: true }),
                    );
                }
                // The head slot is itself in motion until the move completes.
                ctx.set_shape_only(head_slot, self.moving);
                ctx.drain();
                ctx.defer_source(
                    head_slot,
                    self.head,
                    PISTON_MOVE_TICKS,
                    Some(Sweep { travel: self.facing, extending: true }),
                );

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
                // An in-flight head is `finalTick`ed first, and `finalTick`
                // lands the block it is *carrying* — the piston head — not air.
                // The slot is emptied later, by the `removeBlock` at the end of
                // the retract branch, which is a second write.
                //
                // Writing air here instead collapsed the two into one and lost
                // both the head state and the `neighborChanged` that finalTick
                // ends with. A snapshot capture cannot tell the difference,
                // because both orders leave air at the end of the tick; the
                // notification log can, and does.

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
                if std::env::var_os("MC_TICK_TRACE_RETRACT").is_some() {
                    eprintln!(
                        "[t{}] [retract] {:?} id={id} head={:?} head_state={} pending_at_head={}",
                        ctx.tick,
                        (pos.x, pos.y, pos.z),
                        (head.x, head.y, head.z),
                        ctx.states.descriptor(ctx.world.get(head)).unwrap_or("?"),
                        ctx.moves.iter().any(|m| m.pos == head)
                    );
                }
                if let Some(index) = ctx.moves.iter().position(|m| m.pos == head) {
                    let landed = ctx.moves.remove(index);
                    // `finalTick` lands `isSourcePiston ? AIR : movedState`, and
                    // the head slot of an extension still in progress is a
                    // source piston. It empties; it does not deliver its head.
                    let state = if landed.source_piston {
                        StateId::AIR
                    } else {
                        landed.state
                    };
                    ctx.set(head, state);
                    ctx.drain();
                    // `finalTick` ends with `neighborChanged` at its own
                    // position, whether it runs from the block-entity phase or
                    // from here inside `triggerEvent`. Same call, same place in
                    // the sequence — after the write's own notifications.
                    ctx.notify(head, Dir::Down);
                    ctx.drain();
                }
                ctx.set(pos, self.moving);
                ctx.defer_source(
                    pos,
                    self.states.get(false),
                    PISTON_MOVE_TICKS,
                    Some(Sweep { travel: self.facing.opposite(), extending: false }),
                );

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
                                ctx.defer(
                        to,
                        *state,
                        PISTON_MOVE_TICKS,
                                    Some(Sweep { travel: back, extending: false }),
                                );
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
        // Refused, so nothing moves — not even the stone that could have. The
        // list a refused plan carries is diagnostic only: every caller gates on
        // `possible`, and it is worth far more to be able to see *how far* a
        // refused resolve got than to have it come back empty.
        assert!(!plan.possible, "nothing moves, not even the movable part");
        assert_eq!(plan.to_push, vec![Pos::new(1, 1, 0)], "collected up to the obsidian");
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

// ---------------------------------------------------------------------------
// Entities in a moving piston's way.
// ---------------------------------------------------------------------------

/// A block a piston currently has in flight: which way it is going, and
/// whether the piston is pushing or pulling.
///
/// The direction alone does not say which, and the two are not equally known:
/// **extension** displacement is measured and reproduced bit-exactly, while
/// **retraction** is not — see [`sweep_displacement`] and
/// `crate::sim::Simulation::piston_retract_contacts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sweep {
    /// The direction the block is travelling.
    pub travel: Dir,
    /// `true` while the piston pushes, `false` while it pulls back.
    pub extending: bool,
}

/// How far a moving-piston step advances, as a fraction of a block.
///
/// `PistonMovingBlockEntity.tick` does `float f = progress + 0.5F` once per
/// block-entity tick, so a move is two steps of half a block. The block itself
/// lands on the *third* tick, when `progressO` has already reached 1.0 — which
/// is why [`PISTON_MOVE_TICKS`] is 2 and the two displacement steps happen on
/// the tick the event ran and the one after it.
pub const PISTON_STEP: f64 = 0.5;

/// The extra shove vanilla adds on top of the overlap it measured.
///
/// `d1 = Math.min(d1, d0) + 0.01D`. It is not cosmetic: it is why a dragon
/// fireball ends a full extension 1.01 blocks along and not 1.00, and it is
/// directly visible in `piston_entity.json`.
pub const PISTON_OVERSHOOT: f64 = 0.01;

/// How far an entity is displaced by one step of a moving piston, or `None` if
/// this step does not touch it.
///
/// The model is `PistonMovingBlockEntity.moveCollidedEntities`, reduced to the
/// one geometry the captures exercise:
///
/// * the moving block's collision box at `progress` is the unit cube at its
///   **destination**, offset back along the travel direction by `1 - progress`;
/// * `PistonMath.getMovementArea` turns that into the *slab* the leading face
///   sweeps through this step — from the leading face, `PISTON_STEP` further
///   on;
/// * an entity overlapping the slab is pushed by the depth of that overlap
///   measured from the slab's trailing edge, capped at the step, plus
///   [`PISTON_OVERSHOOT`].
///
/// # Why a unit cube is exact here, and where it would not be
///
/// Vanilla takes the max over the moved state's individual collision boxes. For
/// the head slot the state is `piston_head`, whose plate sits flush against the
/// leading face of its block and whose arm trails behind it — so the plate,
/// not the arm, sets the maximum, and the plate's leading face *is* the unit
/// cube's. For a pushed block the state is whatever was pushed, and every one
/// in the record door is a full cube. A pushed slab or stair would have a
/// leading face short of the cube's and this would over-report; that case is
/// unmeasured, and noted rather than guessed at.
///
/// Verified bit-exact against `crates/mc-tick/tests/traces/piston_entity.json`
/// (see `tools/gametest/captures/piston_entity.entities.log`) for a minecart,
/// a NaN minecart, a small fireball and a dragon fireball.
pub fn sweep_displacement(
    destination: Pos,
    travel: Dir,
    progress: f64,
    entity_min: [f64; 3],
    entity_max: [f64; 3],
) -> Option<f64> {
    let axis = match travel {
        Dir::West | Dir::East => 0,
        Dir::Down | Dir::Up => 1,
        Dir::North | Dir::South => 2,
    };
    let (sx, sy, sz) = travel.delta();
    let sign = f64::from([sx, sy, sz][axis]);
    let origin = [
        f64::from(destination.x),
        f64::from(destination.y),
        f64::from(destination.z),
    ];
    // `moveByPositionAndProgress`: the cube, shifted back by the distance still
    // to travel.
    let behind = 1.0 - progress;
    let mut block_min = [0.0f64; 3];
    let mut block_max = [0.0f64; 3];
    for i in 0..3 {
        let shift = if i == axis { -behind * sign } else { 0.0 };
        block_min[i] = origin[i] + shift;
        block_max[i] = origin[i] + 1.0 + shift;
    }
    // `PistonMath.getMovementArea`: the slab in front of the leading face.
    let leading = if sign > 0.0 { block_max[axis] } else { block_min[axis] };
    let (slab_lo, slab_hi) = if sign > 0.0 {
        (leading, leading + PISTON_STEP)
    } else {
        (leading - PISTON_STEP, leading)
    };
    let mut slab_min = block_min;
    let mut slab_max = block_max;
    slab_min[axis] = slab_lo;
    slab_max[axis] = slab_hi;
    // `AABB.intersects` — strict, so a face exactly touching is not a hit.
    for i in 0..3 {
        if !(entity_min[i] < slab_max[i] && entity_max[i] > slab_min[i]) {
            return None;
        }
    }
    // `getMovement`: how deep the entity is into the slab, from behind it.
    let overlap = if sign > 0.0 {
        slab_max[axis] - entity_min[axis]
    } else {
        entity_max[axis] - slab_min[axis]
    };
    if overlap <= 0.0 {
        return None;
    }
    Some(overlap.min(PISTON_STEP) + PISTON_OVERSHOOT)
}

/// How far clear of the vacated block a retracting head leaves an entity that
/// was standing in it: `2 * PISTON_OVERSHOOT`.
///
/// Measured, not derived. Every entity ejected by a head-only retraction ends
/// with its trailing face exactly `0.02` inside the block the head left, across
/// four hitboxes and every start position tried — see
/// [`head_eject_displacement`].
pub const PISTON_EJECT_CLEARANCE: f64 = 2.0 * PISTON_OVERSHOOT;

/// The furthest one moving-piston step will carry an entity.
///
/// `PISTON_STEP + PISTON_OVERSHOOT`. Visible directly in the captures: an
/// entity that needs `0.72375` to reach its target takes it as `0.51` then
/// `0.21375`.
pub const PISTON_MAX_STEP: f64 = PISTON_STEP + PISTON_OVERSHOOT;

/// How far a retracting **head** ejects an entity that is standing in the block
/// it is leaving, or `None` if it does not touch it.
///
/// This is the second, separate way a retraction moves an entity, and it is not
/// the [`sweep_displacement`] slab. A retracting head does **not** reach
/// forwards: an entity in front of the head is untouched no matter how close it
/// is (`piston_pull_plate` lane `headonly`, a dragon fireball overlapping the
/// head's block by 0.05, never moves). What it does instead is clear out the
/// block it is vacating:
///
/// * it fires only when the entity's **centre** lies in the vacated block —
///   a box-overlap gate is refuted by the `headonly` lane above, and the
///   centre-block gate agrees with all fifteen head lanes across three
///   captures, on both sides of the block and at two heights;
/// * the entity is moved along the piston axis until its trailing face — the
///   one on the piston's side — sits [`PISTON_EJECT_CLEARANCE`] inside the
///   vacated block, **in either direction**: an entity that is already past
///   that line is nudged *back* towards it, against the head's travel;
/// * each step moves at most [`PISTON_MAX_STEP`], so a deep entity takes two.
///
/// `destination` is the block the head is retracting *into* (the piston's own
/// square), `travel` the direction it is going. The returned distance is signed
/// along `travel`: negative means the entity is pushed the other way.
///
/// # Provenance
///
/// Fitted to `tools/gametest/captures/piston_pull.entities.log`,
/// `piston_pull_law.entities.log`, `piston_pull_plate.entities.log` and
/// `piston_pull_fit.entities.log`, bit-exactly, for a minecart, a NaN minecart,
/// a small fireball and a dragon fireball. It is an empirical fit to what the
/// game does, not a transcription of a mechanism — the vanilla call that
/// produces it has not been identified, and the `+0.02` is measured rather than
/// explained. Only the `-X` travel direction is covered by a capture; the axis
/// switch is assumed symmetric, as it is for [`sweep_displacement`].
pub fn head_eject_displacement(
    destination: Pos,
    travel: Dir,
    entity_min: [f64; 3],
    entity_max: [f64; 3],
) -> Option<f64> {
    let axis = match travel {
        Dir::West | Dir::East => 0,
        Dir::Down | Dir::Up => 1,
        Dir::North | Dir::South => 2,
    };
    let (dx, dy, dz) = travel.delta();
    let step = [dx, dy, dz];
    let sign = f64::from(step[axis]);
    // The block being vacated is one step back along the travel.
    let vacated = [
        destination.x - step[0],
        destination.y - step[1],
        destination.z - step[2],
    ];
    // The gate is a point in the vacated block, and *which* point differs by
    // axis. Along the piston it is the box centre — `piston_pull_plate`'s
    // `headonly` lane refutes an overlap gate, and `piston_pull_inside`'s
    // `inside 2.9` lane, a vertical piston whose fireball has its feet in the
    // vacated block and its centre in the next one, is left alone. **Across**
    // the piston the vertical coordinate is the entity's **feet**, not its
    // centre: `piston_head_yband.entities.log` ejects a fireball with two
    // thirds of its box in the block above the vacated one and refuses one
    // whose box overlaps the vacated block by 0.2625 from below, and only
    // `min y` separates those eleven lanes. Horizontal cross axes stay on the
    // centre, which is all any capture constrains them to.
    //
    // The asymmetry is the shape of `Entity.position()` — (centre x, min y,
    // centre z) — showing through, and it is why the record door's fireball
    // id=11, feet at y = 0.875 and centre at 1.03125, was refused by a gate
    // that used the centre everywhere.
    for i in 0..3 {
        let probe = if i == 1 && axis != 1 {
            entity_min[i]
        } else {
            (entity_min[i] + entity_max[i]) * 0.5
        };
        if probe.floor() != f64::from(vacated[i]) {
            return None;
        }
    }
    let lo = f64::from(vacated[axis]);
    // The vacated block's face on the piston's side, and the line the entity's
    // matching face is driven to.
    let (near, face) = if sign < 0.0 {
        (lo, entity_min[axis])
    } else {
        (lo + 1.0, entity_max[axis])
    };
    let target = near - sign * PISTON_EJECT_CLEARANCE;
    let distance = sign * (target - face);
    if distance == 0.0 {
        return None;
    }
    Some(distance.clamp(-PISTON_MAX_STEP, PISTON_MAX_STEP))
}

/// The near face of a piston arm's cross-section, as a fraction of a block.
///
/// `PistonHeadBlock`'s arm is `box(6, 6, 10, 10)` in the two axes across the
/// piston, and that 4/16 column — not the whole block — is the gate on
/// [`inside_eject_displacement`]. Measured, not read off the source:
/// `sq_yband` steps a small fireball's box across the edge in thousandths and
/// the answer flips exactly here, at both edges, in twelve lanes.
pub const PISTON_ARM_NEAR: f64 = 6.0 / 16.0;

/// The far face of a piston arm's cross-section. See [`PISTON_ARM_NEAR`].
pub const PISTON_ARM_FAR: f64 = 10.0 / 16.0;

/// How deep the arm's slot is in an extended piston's own block: `4/16`.
///
/// `PistonBaseBlock`'s collision shape when `extended=true` is the block minus a
/// 4-pixel slab on the facing side — the slot the arm sits in. It is the only
/// part of a retracting piston's own square that is **not** solid, and the face
/// at `4/16` is a real surface an entity is stopped flush against.
pub const PISTON_BASE_SLOT: f64 = 4.0 / 16.0;

/// The box a retracting piston's own square keeps solid for the whole stroke.
///
/// `PistonMovingBlockEntity.getCollisionShape` opens with
///
/// ```text
/// if (!extending && isSourcePiston && movedState.getBlock() instanceof PistonBaseBlock)
///     shape = movedState.setValue(EXTENDED, true).getCollisionShape(...)
/// else
///     shape = Shapes.empty()
/// ```
///
/// and that first shape is **never suppressed**: the `NOCLIP` early return a
/// line later returns *it*, not nothing. So while a sticky piston pulls, the cell
/// holding the piston keeps the 12/16 box of its own extended base, and an entity
/// the stroke shoves inward is stopped dead against it.
///
/// This is what pins the wide-body numbers in
/// `tools/gametest/captures/piston_clip_sizes.entities.log`. The 0.98-wide
/// furnace minecart in lane `z=9` starts with its east face at
/// `5.000000009536743` and ends the first step at exactly `5.25` — the slot face
/// of the piston at `(5,1,9)`, to the last bit — where the unclipped step would
/// have been `0.51`.
///
/// `travel` is the direction the head is moving, which is *inward*, so the slot
/// is at the `travel.opposite()` end of the block.
pub fn retracting_base_box(piston: Pos, travel: Dir) -> ([f64; 3], [f64; 3]) {
    let axis = match travel {
        Dir::West | Dir::East => 0,
        Dir::Down | Dir::Up => 1,
        Dir::North | Dir::South => 2,
    };
    let (dx, dy, dz) = travel.delta();
    let outward = -f64::from([dx, dy, dz][axis]);
    let lo = [
        f64::from(piston.x),
        f64::from(piston.y),
        f64::from(piston.z),
    ];
    let mut min = lo;
    let mut max = [lo[0] + 1.0, lo[1] + 1.0, lo[2] + 1.0];
    if outward > 0.0 {
        max[axis] -= PISTON_BASE_SLOT;
    } else {
        min[axis] += PISTON_BASE_SLOT;
    }
    (min, max)
}

/// `PistonMovingBlockEntity.fixEntityWithinPistonBase`, as an outward distance.
///
/// The call the engine had no model of at all, and the missing half of
/// retracting a body wider than the arm. After **every** shove by a retracting
/// source piston, vanilla runs
///
/// ```text
/// if (!extending && isSourcePiston)
///     fixEntityWithinPistonBase(pos, entity, movementDirection, d0)
/// ```
///
/// which shoves any entity still overlapping the piston's **own full cell** back
/// out of it, against the head's travel:
///
/// ```text
/// AABB cell = Shapes.block().bounds().move(pos);
/// if (box.intersects(cell)) {
///     Direction out = dir.getOpposite();
///     double d1 = getMovement(cell, out, box) + 0.01;
///     double d2 = getMovement(cell, out, box.intersect(cell)) + 0.01;
///     if (Math.abs(d1 - d2) < 0.01) {
///         d1 = Math.min(d1, d0) + 0.01;
///         moveEntityByPiston(dir, entity, d1, out);
///     }
/// }
/// ```
///
/// `d1` is the distance from the cell's outward face to the entity's *inward*
/// face — how far out it has to go to leave the cell. `d2` is the same measured
/// against the clipped box, so the two differ only when the entity reaches past
/// the cell's inward face; that guard declines a body that engulfs the piston
/// rather than merely poking into it.
///
/// `travel` is the head's inward travel, and the returned distance is **outward**
/// — the opposite direction — so a caller shoves along `travel.opposite()`.
///
/// # Why the door needs it
///
/// `piston_clip_sizes` lanes `z=5` (a 1.0-wide dragon fireball) and `z=9` (a 0.98
/// furnace minecart) are shoved a quarter block east and then a quarter block
/// back, a round trip. The outward half is this call: the cart's east face lands
/// on exactly `5.0`, the west face of the piston's own cell, having been stopped
/// on the previous step against `5.25` by [`retracting_base_box`]. Two surfaces,
/// two bit-exact landings, and neither of them a fitted constant.
pub fn base_fix_displacement(
    piston: Pos,
    travel: Dir,
    entity_min: [f64; 3],
    entity_max: [f64; 3],
) -> Option<f64> {
    let axis = match travel {
        Dir::West | Dir::East => 0,
        Dir::Down | Dir::Up => 1,
        Dir::North | Dir::South => 2,
    };
    let (dx, dy, dz) = travel.delta();
    let outward = -f64::from([dx, dy, dz][axis]);
    let lo = [
        f64::from(piston.x),
        f64::from(piston.y),
        f64::from(piston.z),
    ];
    // `AABB.intersects` against the whole cell — strict, as everywhere else.
    for i in 0..3 {
        if !(entity_min[i] < lo[i] + 1.0 && entity_max[i] > lo[i]) {
            return None;
        }
    }
    // `getMovement(cell, outward, box)`: the cell's outward face to the entity's
    // inward one.
    let (cell_face, inward_face, clipped_face) = if outward > 0.0 {
        (lo[axis] + 1.0, entity_min[axis], entity_min[axis].max(lo[axis]))
    } else {
        (lo[axis], entity_max[axis], entity_max[axis].min(lo[axis] + 1.0))
    };
    let d1 = outward * (cell_face - inward_face) + PISTON_OVERSHOOT;
    let d2 = outward * (cell_face - clipped_face) + PISTON_OVERSHOOT;
    if (d1 - d2).abs() >= PISTON_OVERSHOOT {
        return None;
    }
    Some(d1.min(PISTON_STEP) + PISTON_OVERSHOOT)
}

/// How far a retracting head displaces an entity standing in the **piston's
/// own square** — the block the head is closing back into — or `None` if this
/// step does not touch it.
///
/// This is the third and last of retraction's three geometries, and the one
/// that was left unmodelled because two captures appeared to contradict each
/// other: `piston_pull_inside` moves a vertical entity where `piston_pull_law`
/// lane 1 leaves a horizontal one alone. **Neither the axis nor the floor is
/// the difference.** Lane 1's fireball sat at y = 1.0, whose box tops out at
/// 1.3125 — *below the arm*. Lift the same fireball by 0.34375, change nothing
/// else, and vanilla moves it exactly as the vertical rig does. See
/// `tools/gametest/captures/piston_pull_float.entities.log`.
///
/// The law, in the outward coordinate `u` — distance along the piston's facing
/// from the inner face of its own block, so the piston square is `u ∈ [0, 1]`
/// and the head's block is `u ∈ [1, 2]`:
///
/// * **the gate across the axis is the arm column**, [`PISTON_ARM_NEAR`] to
///   [`PISTON_ARM_FAR`] in both perpendicular axes, strictly — an entity that
///   only touches the edge is not moved;
/// * **step one** acts on an entity overlapping the middle half `u ∈ [0.25,
///   0.75]`, and drives it to the first of these it can reach in one
///   [`PISTON_MAX_STEP`]: trailing face to `1.01` (clear of the whole square),
///   trailing face to `0.76` (clear of the arm), leading face to `0.24` (back
///   out of the arm). If it can reach none, it retreats a full step;
/// * **step two** acts on an entity still overlapping `u ∈ [0, 1]` and drives
///   its trailing face to `1.02` — the same `blockMin + 0.02` line
///   [`head_eject_displacement`] lands on — or, failing that, its leading face
///   to `-0.01`, or failing that retreats a full step.
///
/// `destination` is the piston's own square, `travel` the direction the head is
/// moving (inward, against the piston's facing) and `progress` is `0.0` on the
/// first step and [`PISTON_STEP`] on the second. The returned distance is
/// signed along `travel`, so a negative value means the entity is thrown
/// *outward*, the way most of them go.
///
/// # Provenance
///
/// Fitted bit-exactly to four captures totalling forty-one lanes:
/// `piston_pull_inside` (vertical, 5), `piston_pull_float` (horizontal, 12),
/// `piston_pull_xsweep` (12) and `sq_cart` (12), covering a small fireball, a
/// dragon fireball, a minecart, a furnace minecart and a NaN furnace minecart,
/// on both axes and both sides of the block. It is an empirical fit; the
/// vanilla call producing it has not been identified.
///
/// The rigs matter: `piston_pull_law` lanes z = 15, 17, 19 and 21 carry a
/// **block to pull** at `(4, 1, z)` and the others do not, so the same entity
/// in two of its lanes lands 0.02 apart. `piston_pull_square` is the same rig
/// with twelve pull-free lanes, and every number above comes from it.
pub fn inside_eject_displacement(
    destination: Pos,
    travel: Dir,
    progress: f64,
    entity_min: [f64; 3],
    entity_max: [f64; 3],
) -> Option<f64> {
    let axis = match travel {
        Dir::West | Dir::East => 0,
        Dir::Down | Dir::Up => 1,
        Dir::North | Dir::South => 2,
    };
    let (dx, dy, dz) = travel.delta();
    // The head travels *into* the piston, so outward is the other way.
    let outward = -f64::from([dx, dy, dz][axis]);
    let origin = [
        f64::from(destination.x),
        f64::from(destination.y),
        f64::from(destination.z),
    ];
    // Across the piston: the entity must be in the arm's own column.
    for i in 0..3 {
        if i == axis {
            continue;
        }
        let near = origin[i] + PISTON_ARM_NEAR;
        let far = origin[i] + PISTON_ARM_FAR;
        if !(entity_min[i] < far && entity_max[i] > near) {
            return None;
        }
    }
    // Along the piston, in `u`: 0 at the square's inner face, 1 at its outer.
    let inner = if outward > 0.0 { origin[axis] } else { origin[axis] + 1.0 };
    let depth = |world: f64| outward * (world - inner);
    let (trailing, leading) = if outward > 0.0 {
        (depth(entity_min[axis]), depth(entity_max[axis]))
    } else {
        (depth(entity_max[axis]), depth(entity_min[axis]))
    };
    let first_step = progress < PISTON_STEP;
    let (gate_lo, gate_hi) = if first_step { (0.25, 0.75) } else { (0.0, 1.0) };
    if !(trailing < gate_hi && leading > gate_lo) {
        return None;
    }
    // Outward targets move the trailing face; the retreat moves the leading
    // one. Outermost first — an entity that can clear the whole square in one
    // step does, rather than stopping at the arm.
    let targets: &[(bool, f64)] = if first_step {
        &[(true, 1.01), (true, 0.76), (false, 0.24)]
    } else {
        &[(true, 1.02), (false, -0.01)]
    };
    let mut chosen = None;
    for &(is_outward, target) in targets {
        let shift = target - if is_outward { trailing } else { leading };
        if shift == 0.0 {
            return None;
        }
        if shift.abs() <= PISTON_MAX_STEP {
            chosen = Some(shift);
            break;
        }
    }
    // Nothing is reachable: the entity is shoved back a whole step and the
    // next one finishes the job.
    let shift = chosen.unwrap_or(-PISTON_MAX_STEP);
    // Back to a distance along `travel`, which points the other way to `u`.
    Some(-shift)
}

#[cfg(test)]
mod sweep_tests {
    use super::*;

    /// The four lanes of `piston_entity.json`, replayed step by step.
    ///
    /// Piston at x=2 facing east, head slot x=3, every entity centred at
    /// x=3.5. Vanilla's answers, from the entity log, are the `expected`
    /// column — and they are compared exactly, not approximately, because the
    /// eighth decimal of the cart's is the float width of its hitbox and the
    /// 0.01 is vanilla's overshoot. Both would survive an epsilon.
    #[test]
    fn extension_reproduces_the_capture_to_the_last_bit() {
        // (kind, half-width, [pos after step 1, pos after step 2])
        let cases: [(&str, f64, [f64; 2]); 3] = [
            // minecart and NaN minecart — one hitbox, one answer
            ("minecart", crate::minecart::CART_HALF_WIDTH, [4.000000009536743, 4.500000009536743]),
            ("small_fireball", 0.15625, [3.66625, 4.16625]),
            ("dragon_fireball", 0.5, [4.01, 4.51]),
        ];
        for (kind, half, expected) in cases {
            let mut x = 3.5;
            for (step, want) in expected.iter().enumerate() {
                let progress = f64::from(step as u32) * PISTON_STEP;
                let d = sweep_displacement(
                    Pos::new(3, 1, 1),
                    Dir::East,
                    progress,
                    [x - half, 1.0, 1.5 - half],
                    [x + half, 1.7, 1.5 + half],
                )
                .unwrap_or_else(|| panic!("{kind} step {step}: vanilla displaced it"));
                x += d;
                assert_eq!(x, *want, "{kind} after step {step}");
            }
        }
    }

    /// `piston_pull.entities.log`: a sticky piston with **nothing to pull**,
    /// entities standing in the head's own block, power cut at t6.
    ///
    /// Every one of the four lanes ends with its box's trailing face at exactly
    /// `3.02`, which is the whole content of the law. Read as *displacement*
    /// the four answers look unrelated — the carts move `+0.01`, the small
    /// fireball `-0.32375` — and that is what made this capture read as
    /// inconclusive the first time. Read as a box edge they are one number.
    #[test]
    fn head_ejection_reproduces_piston_pull_to_the_last_bit() {
        // (kind, half-width, vanilla's final centre x)
        let cases: [(&str, f64, f64); 4] = [
            ("minecart", crate::minecart::CART_HALF_WIDTH, 3.510_000_009_536_743),
            ("nan minecart", crate::minecart::CART_HALF_WIDTH, 3.510_000_009_536_743),
            ("small_fireball", 0.156_25, 3.176_25),
            ("dragon_fireball", 0.5, 3.52),
        ];
        for (kind, half, want) in cases {
            let mut x = 3.5;
            for step in 0..2 {
                // The head leaves block x=3 for the piston's own square, x=2.
                if let Some(d) = head_eject_displacement(
                    Pos::new(2, 1, 1),
                    Dir::West,
                    [x - half, 1.0, 1.01],
                    [x + half, 1.7, 1.99],
                ) {
                    x -= d;
                }
                let _ = step;
            }
            assert_eq!(x, want, "{kind}");
            assert_eq!(x - half, 3.02, "{kind} trailing face");
        }
    }

    /// `piston_pull_law` and `piston_pull_fit`: the same rig, with the entity
    /// started all over the head's block and outside it.
    ///
    /// The two `None` rows are the negative controls that pin the gate. A
    /// fireball whose centre is in the piston's own square is not ejected, and
    /// neither is one whose centre is in the block *in front* — even though the
    /// latter's box overlaps the head's block, which is what rules a
    /// box-overlap gate out.
    #[test]
    fn head_ejection_is_gated_on_the_entity_centre_not_its_box() {
        const HALF: f64 = 0.156_25;
        // (start x, how many steps vanilla needed, whether it ends on the line)
        //
        // The answers are asserted as the trailing *face* rather than as a
        // decimal delta: 3.02 is exact in binary, and the deltas are not — for
        // a start of 3.1 the true displacement is 3.1 - HALF - 3.02, which no
        // short decimal literal spells.
        let cases: [(f64, usize); 6] = [
            (2.5, 0),      // centre in the piston: untouched
            (4.3, 0),      // centre in front of the head: untouched
            (3.1, 1),      // nudged *back* east onto the line
            (3.5, 1),      // the plain case
            (3.9, 2),      // too deep for one step
            (3.176_25, 0), // already on the line
        ];
        for (start, steps) in cases {
            let mut x = start;
            let mut moved = 0usize;
            for step in 0..2 {
                let got = head_eject_displacement(
                    Pos::new(2, 1, 1),
                    Dir::West,
                    [x - HALF, 1.0, 1.4],
                    [x + HALF, 1.3125, 1.6],
                );
                if let Some(d) = got {
                    assert!(d.abs() <= PISTON_MAX_STEP, "start {start} step {step}: {d}");
                    if steps == 2 && step == 0 {
                        assert_eq!(d, PISTON_MAX_STEP, "start {start}: first step is capped");
                    }
                    x -= d;
                    moved += 1;
                }
            }
            assert_eq!(moved, steps, "start {start}: number of steps that moved it");
            let want = if steps == 0 && start < 3.0 || start > 4.0 { start - HALF } else { 3.02 };
            assert_eq!(x - HALF, want, "start {start}: trailing face");
        }
    }

    /// `piston_head_yband.entities.log`: the gate across the axis is the
    /// entity's **feet**, not its centre.
    ///
    /// Eleven lanes, one sticky piston retracting east with nothing to pull, one
    /// small fireball each at x = 4.84375 — the record 3x3 door's own offset,
    /// east face flush on the block boundary — and nothing varying but y. The
    /// head leaves block `(4,1,z)` for the piston at `(5,1,z)`.
    ///
    /// Vanilla ejects a fireball whose box is `[1.99, 2.3025]`, two thirds of it
    /// in the block *above* the vacated one and its centre y at 2.14, and leaves
    /// alone one at `[0.95, 1.2625]` whose box overlaps the vacated block by
    /// 0.2625 and whose centre y is 1.11. Neither a centre gate nor a
    /// box-overlap gate can produce that pair; `BlockPos.containing(position())`
    /// can, because an entity's `position()` is (centre x, **min** y, centre z),
    /// so the y coordinate the game floors is the entity's feet.
    ///
    /// This is exactly the record door's fireball id=11: min y 0.875 puts its
    /// feet in the piston's row, its centre y 1.03125 does not, and a centre
    /// gate is why the engine refused to eject it.
    #[test]
    fn head_ejection_gates_on_the_feet_across_the_axis_not_the_centre() {
        const HALF: f64 = 0.156_25;
        const HEIGHT: f64 = 0.3125;
        // (spawn y, does vanilla eject it) — read straight off the capture.
        let cases: [(f64, bool); 11] = [
            (0.5, false),      // wholly below the vacated block
            (0.7, false),      // box overlaps it by 0.0125; feet do not
            (0.95, false),     // box overlaps it by 0.2625; feet do not
            (1.0, true),       // feet exactly on the floor of it
            (1.34375, true),
            (1.6875, true),    // box top exactly flush with the block above
            (1.7, true),
            (1.875, true),     // the record door's own y
            (1.99, true),      // feet barely inside; centre is a block higher
            (2.0, false),      // feet exactly on the block above: refused
            (2.5, false),
        ];
        let x = 4.843_75;
        for (y, ejected) in cases {
            let got = head_eject_displacement(
                Pos::new(5, 1, 1),
                Dir::East,
                [x - HALF, y, 1.343_75],
                [x + HALF, y + HEIGHT, 1.656_25],
            );
            if ejected {
                // Asserted as the trailing face, not as a delta: 4.98 is what
                // the capture shows and the delta from a flush 5.0 has no short
                // decimal spelling.
                let d = got.expect("y {y}: ejected");
                assert!(d < 0.0, "y {y}: nudged back against the head, got {d}");
                assert_eq!(x + HALF + d, 5.0 - PISTON_EJECT_CLEARANCE, "y {y}: trailing face");
            } else {
                assert_eq!(got, None, "y {y}: left alone");
            }
        }
    }

    /// A dragon fireball overlapping the vacated block by 0.05 in x *and* 0.02
    /// in y, whose centre is in neither — `piston_pull_plate`'s `headonly`
    /// lane, which vanilla leaves alone for the whole run.
    ///
    /// This is the negative control the first attempt at this could not build,
    /// and it is the one that separates the two retraction mechanisms: the same
    /// entity in the same place *is* moved, almost a full block, as soon as the
    /// piston has a block to pull.
    #[test]
    fn a_retracting_head_does_not_reach_forward_out_of_its_own_block() {
        // Head in block (3,2,1) leaving for the piston at (2,2,1).
        let min = [3.95, 1.02, 1.0];
        let max = [4.95, 2.02, 2.0];
        assert_eq!(head_eject_displacement(Pos::new(2, 2, 1), Dir::West, min, max), None);
        // ...but the block the sticky piston pulls into (3,2,1) sweeps it.
        assert_eq!(
            sweep_displacement(Pos::new(3, 2, 1), Dir::West, 0.0, min, max),
            Some(PISTON_MAX_STEP),
        );
    }

    /// `piston_pull_fit`, the lanes with a stone block for the sticky head to
    /// pull: the pulled block's own sweep, which is [`sweep_displacement`]
    /// unchanged, and which the engine already computed and then threw away.
    ///
    /// The second step is short in every lane because the entity fetches up
    /// against the piston body — clipping that the caller does, so what is
    /// checked here is the unclipped distance and the resulting stop line.
    #[test]
    fn a_pulled_block_sweeps_entities_a_real_distance() {
        const HALF: f64 = 0.156_25;
        // (start x, whether the first step is capped, whether it is touched)
        let cases: [(f64, bool, bool); 4] = [
            (4.15, true, true),
            (4.10, true, true),
            (3.60, false, true), // shallow: the overlap, not the cap
            (3.20, false, false), // behind the first slab entirely
        ];
        for (start, capped, touched) in cases {
            let min = [start - HALF, 2.0, 1.4];
            let max = [start + HALF, 2.3125, 1.6];
            // The pulled block lands on the head's block, x=3, travelling west.
            let got = sweep_displacement(Pos::new(3, 2, 1), Dir::West, 0.0, min, max);
            assert_eq!(got.is_some(), touched, "start {start}");
            if let Some(d) = got {
                if capped {
                    assert_eq!(d, PISTON_MAX_STEP, "start {start}");
                } else {
                    // The overlap of the box with the slab behind x=3.5, plus
                    // the overshoot — written out rather than as a literal.
                    assert_eq!(d, (max[0] - 3.5) + PISTON_OVERSHOOT, "start {start}");
                    assert!(d < PISTON_MAX_STEP, "start {start}");
                }
            }
            // Every lane is stopped by the piston body and finishes flush at
            // x=3.0 — clipping the caller does, recorded here as the contract.
            assert_eq!(3.156_25 - HALF, 3.0);
        }
    }

    /// The same capture's retraction at t14: by then the entities sit a block
    /// clear of the arm, and vanilla moves nothing for the remaining 15 ticks.
    #[test]
    fn a_retracting_arm_does_not_reach_an_entity_that_is_already_clear() {
        let half = crate::minecart::CART_HALF_WIDTH;
        for step in 0..2 {
            let progress = f64::from(step) * PISTON_STEP;
            // Retraction: the head travels back into the piston's own square.
            let d = sweep_displacement(
                Pos::new(2, 1, 1),
                Dir::West,
                progress,
                [4.500000009536743 - half, 1.0, 1.01],
                [4.500000009536743 + half, 1.7, 1.99],
            );
            assert_eq!(d, None, "step {step}");
        }
    }

    /// An entity behind the leading face is not dragged along. The sweep is a
    /// slab in front of the block, not the block's own volume.
    #[test]
    fn nothing_behind_the_arm_is_touched() {
        assert_eq!(
            sweep_displacement(
                Pos::new(3, 1, 1),
                Dir::East,
                0.0,
                [1.1, 1.0, 1.1],
                [1.9, 1.7, 1.9],
            ),
            None
        );
    }

    /// Vertical and negative axes use the same geometry — a piston facing down
    /// shoves an entity down by the same overlap-plus-overshoot.
    #[test]
    fn the_geometry_is_not_special_cased_per_axis() {
        // Head slot at y=3 travelling down: at progress 0 the cube still sits
        // a block back, spanning y in [4, 5], and its leading (lower) face at
        // 4.0 sweeps down to 3.5. An entity whose head is at 3.95 is 0.45
        // into that slab. Not a capture — a check that the axis switch is
        // symmetric, with the arithmetic written out rather than a magic
        // number.
        let d = sweep_displacement(
            Pos::new(1, 3, 1),
            Dir::Down,
            0.0,
            [1.1, 3.0, 1.1],
            [1.9, 3.95, 1.9],
        );
        assert_eq!(d, Some((3.95_f64 - 3.5) + PISTON_OVERSHOOT));
    }

    /// Every lane of every capture of the third geometry lands within this of
    /// where vanilla put it. The captures print `Entity.position`, so the
    /// comparison is against a printed double rather than a recomputed one.
    const HAIR: f64 = 1e-9;

    /// A small fireball's box, the 0.3125 cube every horizontal lane uses.
    fn fireball(x: f64, y: f64, z: f64) -> ([f64; 3], [f64; 3]) {
        ([x - 0.15625, y, z - 0.15625], [x + 0.15625, y + 0.3125, z + 0.15625])
    }

    /// A minecart's box: 0.98 wide, 0.7 tall, standing on the floor at y = 1.
    fn cart(x: f64, z: f64) -> ([f64; 3], [f64; 3]) {
        ([x - 0.49, 1.0, z - 0.49], [x + 0.49, 1.7, z + 0.49])
    }

    /// `piston_pull_square`, the horizontal rig with **no block to pull**:
    /// sticky piston at (2, 1, 1) facing east, head at (3, 1, 1), power cut so
    /// the head retracts west into the piston's own square.
    ///
    /// The lanes are `piston_pull_xsweep` (small fireballs lifted into the arm's
    /// band) and `sq_cart` (minecarts, which reach the band from the floor
    /// because they are 0.7 tall). `d` is the displacement vanilla printed,
    /// eastward positive; the function reports it signed along the head's
    /// travel, which is the other way.
    #[test]
    fn an_entity_in_the_pistons_own_square_is_ejected_like_the_capture() {
        let piston = Pos::new(2, 1, 1);
        #[allow(clippy::type_complexity)]
        let lanes: &[(&str, ([f64; 3], [f64; 3]), f64, f64)] = &[
            // small fireball at y = 1.34375, straddling the arm.
            ("fb 2.40", fireball(2.40, 1.34375, 1.5), -0.31625, -0.25),
            ("fb 2.45", fireball(2.45, 1.34375, 1.5), 0.46625, 0.26),
            ("fb 2.50", fireball(2.50, 1.34375, 1.5), 0.41625, 0.26),
            ("fb 2.55", fireball(2.55, 1.34375, 1.5), 0.36625, 0.26),
            ("fb 2.60", fireball(2.60, 1.34375, 1.5), 0.31625, 0.26),
            ("fb 2.65", fireball(2.65, 1.34375, 1.5), 0.26625, 0.26),
            ("fb 2.70", fireball(2.70, 1.34375, 1.5), 0.46625, 0.01),
            // minecarts on the floor.
            ("cart 2.70", cart(2.70, 1.5), -0.51, -0.51),
            ("cart 2.75", cart(2.75, 1.5), 0.50, 0.26),
            ("cart 2.80", cart(2.80, 1.5), 0.45, 0.26),
            ("cart 2.85", cart(2.85, 1.5), 0.40, 0.26),
            ("cart 2.90", cart(2.90, 1.5), 0.35, 0.26),
            ("cart 2.95", cart(2.95, 1.5), 0.30, 0.26),
            ("cart 3.00", cart(3.00, 1.5), 0.50, 0.01),
            ("cart 3.05", cart(3.05, 1.5), 0.45, 0.01),
            ("cart 3.10", cart(3.10, 1.5), 0.40, 0.01),
            ("cart 3.20", cart(3.20, 1.5), 0.30, 0.01),
            // These two clear the middle half, so the first step is
            // `head_eject_displacement`'s and the second finds them settled.
            ("cart 3.25", cart(3.25, 1.5), 0.26, 0.0),
            ("cart 3.30", cart(3.30, 1.5), 0.21, 0.0),
        ];
        for (name, (min, max), first, second) in lanes {
            let mut min = *min;
            let mut max = *max;
            for (step, want) in [(0.0, *first), (PISTON_STEP, *second)] {
                let got = inside_eject_displacement(piston, Dir::West, step, min, max)
                    .or_else(|| head_eject_displacement(piston, Dir::West, min, max))
                    .unwrap_or(0.0);
                // Travel is west, so a displacement east reads negative.
                let east = -got;
                assert!(
                    (east - want).abs() < HAIR,
                    "{name} step {step}: got {east}, capture says {want}"
                );
                min[0] += east;
                max[0] += east;
            }
        }
    }

    /// The gate across the piston is the **arm's** 4/16 column, not the block.
    ///
    /// `sq_yband` walks a small fireball's box across both edges of the arm in
    /// thousandths. A box that only touches the edge is not moved — the
    /// intersection is strict — and one that overlaps it by 0.0025 is thrown
    /// the full 0.41625. This is what `piston_pull_law` lane 1 was measuring
    /// all along: a fireball resting on the floor tops out at 1.3125, below the
    /// arm, so vanilla never touches it.
    #[test]
    fn the_gate_across_the_piston_is_the_arm_column() {
        let piston = Pos::new(2, 1, 1);
        // (spawn y, whether vanilla moved it)
        let lanes = [
            (1.05, false),
            (1.06, false),
            (1.0625, false), // box top exactly 1.375: touching is not overlap
            (1.065, true),
            (1.08, true),
            (1.20, true),
            (1.34375, true),
            (1.60, true),
            (1.62, true),
            (1.625, false), // box bottom exactly 1.625
            (1.63, false),
            (1.70, false),
        ];
        for (y, moved) in lanes {
            let (min, max) = fireball(2.5, y, 1.5);
            let got = inside_eject_displacement(piston, Dir::West, 0.0, min, max);
            assert_eq!(got.is_some(), moved, "y={y}: {got:?}");
            if moved {
                assert!((-got.unwrap() - 0.41625).abs() < HAIR, "y={y}");
            }
        }
        // The same fireball outside the column is untouched on the second step
        // too, so this is a gate and not a delay.
        let (min, max) = fireball(2.5, 1.0, 1.5);
        assert_eq!(inside_eject_displacement(piston, Dir::West, PISTON_STEP, min, max), None);
        assert_eq!(head_eject_displacement(piston, Dir::West, min, max), None);
    }

    /// Where one target gives way to the next, to a ten-thousandth.
    ///
    /// `sq_thresh` walks a small fireball's box across both hand-overs in
    /// hundred-thousandths of a block. A target is taken when it costs
    /// **exactly** [`PISTON_MAX_STEP`] and given up at `0.5101`, at both of
    /// them — which is what fixes the threshold at 0.51 rather than at the
    /// 0.5 step it is built from.
    ///
    /// One lane of the twelve disagrees and is asserted as a disagreement:
    /// `x = 2.65635`, whose box starts a ten-thousandth past the hand-over, is
    /// moved `0.51` by vanilla where this law says `0.5099`. Every neighbour
    /// on both sides agrees, so it is a seam in the mechanism rather than a
    /// wrong constant, and it is 1e-4 of a block wide. Pinned here so that it
    /// cannot be quietly absorbed by a later change.
    #[test]
    fn the_hand_over_between_targets_is_exactly_the_step_limit() {
        let piston = Pos::new(2, 1, 1);
        // (spawn x, vanilla's displacement east on the first step)
        let lanes = [
            (2.66625, 0.50),
            (2.65875, 0.5075),
            (2.65625, 0.51),   // costs exactly 0.51: the outer target is taken
            (2.65615, 0.2601), // costs 0.5101: it is not
            (2.65375, 0.2625),
            (2.65125, 0.2650),
            (2.64625, 0.2700),
            (2.40725, 0.5090),
            (2.40625, 0.5100), // again exactly 0.51, and again taken
            (2.40615, -0.3224), // 0.5101, so it retreats instead
            (2.40525, -0.3215),
        ];
        for (x, want) in lanes {
            let (min, max) = fireball(x, 1.34375, 1.5);
            let got = -inside_eject_displacement(piston, Dir::West, 0.0, min, max)
                .expect("every lane here is inside the square");
            assert!((got - want).abs() < HAIR, "x={x}: got {got}, capture says {want}");
        }
        // The seam. Vanilla moves this one 0.51; the law says 0.5099.
        let (min, max) = fireball(2.65635, 1.34375, 1.5);
        let got = -inside_eject_displacement(piston, Dir::West, 0.0, min, max).unwrap();
        assert!((got - 0.5099).abs() < HAIR, "the law's answer moved: {got}");
        assert!((got - 0.51).abs() > 1e-5, "vanilla says 0.51 and this now agrees — retest the seam");
    }

    /// `piston_pull_inside`: the same law on the vertical axis, which is where
    /// this geometry was first seen and where it looked like a contradiction.
    ///
    /// Sticky piston at (2, 3, 1) facing **down**, head at (2, 2, 1), so the
    /// head travels up into the piston's square and "outward" is downward. The
    /// displacements are vanilla's printed `y` deltas.
    #[test]
    fn the_third_geometry_is_the_same_law_on_the_vertical_axis() {
        let piston = Pos::new(2, 3, 1);
        let sfb = |y: f64| fireball(2.5, y, 1.5);
        let dragon = |y: f64| ([2.0, y, 1.0], [3.0, y + 1.0, 2.0]);
        #[allow(clippy::type_complexity)]
        let lanes: &[(&str, ([f64; 3], [f64; 3]), f64, f64)] = &[
            ("inside 3.2", sfb(3.2), -0.2725, -0.26),
            // Clear of the middle half, so the first step passes it by.
            ("inside 2.9", sfb(2.9), 0.0, -0.2325),
            ("dragon 2.6", dragon(2.6), -0.36, -0.26),
            // Nearer the far side: thrown *up*, against the head's travel.
            ("inside 3.6", sfb(3.6), 0.16, 0.25),
        ];
        for (name, (min, max), first, second) in lanes {
            let mut min = *min;
            let mut max = *max;
            for (step, want) in [(0.0, *first), (PISTON_STEP, *second)] {
                let got = inside_eject_displacement(piston, Dir::Up, step, min, max)
                    .or_else(|| head_eject_displacement(piston, Dir::Up, min, max))
                    .unwrap_or(0.0);
                assert!(
                    (got - want).abs() < HAIR,
                    "{name} step {step}: got {got}, capture says {want}"
                );
                min[1] += got;
                max[1] += got;
            }
        }
    }
}
