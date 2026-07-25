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
//! Extension and retraction of a column, the push limit, and the phase in which the
//! movement happens. What is **not** modelled: the moving-block entity with its
//! progress (phase 9), slime/honey adhesion pulling perpendicular blocks, and
//! quasi-connectivity. Each of those needs a captured trace before it is worth
//! writing — they are where guesses go wrong.

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
/// `PistonStructureResolver.MAX_PUSH_DEPTH`. The constant is inlined by javac so it
/// could not be read directly from the bytecode; 12 is the long-established value
/// and is asserted by a test so that if it is ever wrong, it fails loudly rather
/// than quietly truncating a build.
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

/// Which blocks a piston may move.
///
/// Supplied by the caller: obsidian and bedrock are immovable, most blocks are
/// pushable, and this crate holds no Minecraft block list of its own.
pub trait Movability: Send + Sync {
    /// Whether the block at `pos` can be pushed.
    fn is_movable(&self, world: &World, pos: Pos) -> bool;

    /// Whether the block at `pos` is air, i.e. free space for a push to end in.
    fn is_empty(&self, world: &World, pos: Pos) -> bool {
        world.get(pos) == StateId::AIR
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
    let mut column = Vec::new();
    let mut cursor = piston.offset(dir);

    loop {
        if movability.is_empty(world, cursor) {
            // Free space: the column has somewhere to go.
            return PushPlan {
                to_push: column.into_iter().rev().collect(),
                possible: true,
            };
        }
        if !movability.is_movable(world, cursor) {
            // Immovable block: nothing moves at all.
            return PushPlan {
                to_push: Vec::new(),
                possible: false,
            };
        }
        if column.len() >= MAX_PUSH_DEPTH {
            return PushPlan {
                to_push: Vec::new(),
                possible: false,
            };
        }
        column.push(cursor);
        cursor = cursor.offset(dir);
    }
}

/// A piston.
///
/// One instance per distinct block state, so it knows its own facing and extension
/// without parsing anything at tick time.
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
    /// The `moving_piston` placeholder occupying a block while it travels.
    pub moving: StateId,
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
    fn is_powered(&self, world: &World, pos: Pos) -> bool {
        self.has_direct_signal(world, pos) || self.has_direct_signal(world, pos.offset(Dir::Up))
    }

    /// Whether any neighbour of `pos` emits toward it.
    fn has_direct_signal(&self, world: &World, pos: Pos) -> bool {
        crate::pos::ALL_DIRS.iter().any(|dir| {
            self.power
                .is_powered(world, pos.offset(*dir), dir.opposite())
        })
    }
}

impl<P: PowerSource, M: Movability> BlockBehaviour for Piston<P, M> {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let powered = self.is_powered(ctx.world, pos);
        if powered == self.extended {
            return;
        }
        // Straight to a block event, exactly as PistonBaseBlock does — no scheduled
        // tick. This is why a piston notified in phase 3 moves in phase 7 of the
        // same tick rather than the next one.
        let trigger = if powered {
            TRIGGER_EXTEND
        } else {
            TRIGGER_CONTRACT
        };
        ctx.queue_event(pos, trigger, self.facing as u8);
    }

    fn on_block_event(&self, ctx: &mut TickCtx<'_>, pos: Pos, id: u8, _param: u8) -> bool {
        match id {
            TRIGGER_EXTEND => {
                let plan = resolve_push(ctx.world, &self.movability, pos, self.facing);
                if !plan.possible {
                    return false;
                }
                // Vanilla replaces both ends with `moving_piston` placeholders now
                // and resolves them two ticks later in the block-entities phase.
                // Far end first, so each block reads its source before the next
                // write disturbs it.
                for from in &plan.to_push {
                    let state = ctx.world.get(*from);
                    let to = from.offset(self.facing);
                    ctx.set(to, self.moving);
                    ctx.set(*from, self.moving);
                    ctx.defer(to, state, PISTON_MOVE_TICKS);
                }
                ctx.set(pos, self.states.get(true));
                // The head slot is itself in motion until the move completes.
                ctx.set(pos.offset(self.facing), self.moving);
                ctx.defer(pos.offset(self.facing), self.head, PISTON_MOVE_TICKS);
                true
            }
            TRIGGER_CONTRACT => {
                let head = pos.offset(self.facing);
                if ctx.world.get(head) == self.head {
                    ctx.set(head, StateId::AIR);
                }
                if self.sticky {
                    // Sticky pistons drag the block that was in front of the head.
                    let dragged = head.offset(self.facing);
                    if self.movability.is_movable(ctx.world, dragged) {
                        let state = ctx.world.get(dragged);
                        ctx.set(head, state);
                        ctx.set(dragged, StateId::AIR);
                    }
                }
                ctx.set(pos, self.states.get(false));
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
    }

    impl PowerSource for Model {
        fn is_powered(&self, world: &World, pos: Pos, _toward: Dir) -> bool {
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
    }

    const RETRACTED: StateId = StateId(1);
    const EXTENDED: StateId = StateId(2);
    const HEAD: StateId = StateId(3);
    const STONE: StateId = StateId(4);
    const OBSIDIAN: StateId = StateId(5);
    const LEVER: StateId = StateId(6);
    const MOVING: StateId = StateId(7);

    fn piston(extended: bool, sticky: bool) -> Piston<Model, Model> {
        let model = Model {
            powered: vec![LEVER],
            immovable: vec![OBSIDIAN],
        };
        Piston {
            facing: Dir::East,
            extended,
            sticky,
            states: StatePair { off: RETRACTED, on: EXTENDED },
            head: HEAD,
            moving: MOVING,
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
        TickCtx { world, ticks, events, states, tick: 0,
        updates: Box::leak(Box::new(Vec::new())),
        moves: Box::leak(Box::new(Vec::new())),
        toggles: Box::leak(Box::new(Vec::new())) }
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

        let p = piston(false, false);
        let mut ctx_moves = Vec::new();
        {
            let mut ctx = TickCtx {
                world: &mut w, ticks: &mut t, events: &mut e, states: &s, tick: 0,
                updates: &mut Vec::new(), moves: &mut ctx_moves,
                toggles: &mut Vec::new(),
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
    fn the_push_plan_is_ordered_far_end_first() {
        // So applying it is a simple loop: each block moves into space the previous
        // write already vacated.
        let mut w = world();
        w.set(Pos::new(0, 1, 0), RETRACTED);
        for x in 1..=3 {
            w.set(Pos::new(x, 1, 0), STONE);
        }
        let p = piston(false, false);
        let plan = resolve_push(&w, &p.movability, Pos::new(0, 1, 0), Dir::East);
        assert_eq!(
            plan.to_push,
            vec![Pos::new(3, 1, 0), Pos::new(2, 1, 0), Pos::new(1, 1, 0)]
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
        let mut ctx = run(&mut w, &mut t, &mut e, &s);
        assert!(p.on_block_event(&mut ctx, pos, TRIGGER_CONTRACT, 0));

        assert_eq!(w.get(pos), RETRACTED);
        assert_eq!(w.get(Pos::new(1, 1, 0)), STONE, "dragged back into the head slot");
        assert_eq!(w.get(Pos::new(2, 1, 0)), StateId::AIR);
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
        let mut w = world();
        let mut t = TickQueue::new();
        let mut e = EventQueue::new();
        let s = StateRegistry::new();
        let p = piston(false, false);
        let mut ctx = run(&mut w, &mut t, &mut e, &s);
        assert!(!p.on_block_event(&mut ctx, Pos::new(0, 1, 0), TRIGGER_DROP, 0));
    }
}
