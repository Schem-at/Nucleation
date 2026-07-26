//! Observers: the component most piston doors are built around.
//!
//! An observer watches the block it **faces** and emits a short pulse when that
//! block changes. Captured from vanilla — a redstone block placed in front of a
//! west-facing observer:
//!
//! ```text
//! tick 1  observer powered=false -> true    (dust beyond lights)
//! tick 3  observer powered=true  -> false   (dust goes out)
//! ```
//!
//! So the pulse is [`OBSERVER_PULSE_TICKS`] long and begins on the tick after the
//! change. Watching the *facing* side matches repeaters and comparators, which take
//! their input from the side `facing` names.
//!
//! # Why this matters for doors
//!
//! The pulse is short enough to interact with piston block-dropping: a two-tick
//! pulse into a sticky piston is longer than the one-tick pulse that drops a block,
//! but only just. Door designs sit right on that boundary, which is why an observer
//! whose pulse length is off by a tick would silently break them.

use crate::behaviour::{BlockBehaviour, TickCtx};
use crate::components::StatePair;
use crate::pos::{Dir, Pos};
use crate::schedule::TickPriority;
use crate::world::World;

/// How long an observer stays powered, in game ticks.
///
/// Captured: powered at tick 1, unpowered at tick 3.
pub const OBSERVER_PULSE_TICKS: u64 = 2;

/// An observer.
///
/// One instance per block state, so it knows its own facing and powered flag
/// without parsing anything at tick time.
pub struct Observer {
    /// The direction it watches — the block at `pos.offset(facing)`.
    pub facing: Dir,
    /// Whether this state is the powered one.
    pub powered: bool,
    /// Unpowered/powered states.
    pub states: StatePair,
}

impl Observer {
    /// The position this observer watches.
    pub fn watched(&self, pos: Pos) -> Pos {
        pos.offset(self.facing)
    }

    /// The face the pulse leaves from — the back, opposite what it watches.
    pub fn output_side(&self) -> Dir {
        self.facing.opposite()
    }

    /// `ObserverBlock.updateNeighborsInFront`: notify the block behind the
    /// observer (the one it strongly powers) and that block's other neighbours.
    ///
    /// The neighbour back toward the observer is excluded, as vanilla's
    /// `updateNeighborsAtExceptFromFacing` excludes it.
    fn update_neighbors_in_front(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let front = pos.offset(self.output_side());
        // From the front block's perspective, the observer sits on its `facing`
        // side.
        ctx.updates.push((front, self.facing));
        for dir in crate::pos::ALL_DIRS {
            if dir == self.facing {
                continue; // that neighbour is the observer itself
            }
            ctx.updates.push((front.offset(dir), dir.opposite()));
        }
    }
}

impl BlockBehaviour for Observer {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, from: Dir) {
        // Only a change to the watched block matters. An observer is deliberately
        // deaf to everything else, which is what lets it sit inside a contraption
        // without reacting to its neighbours' comings and goings.
        if from != self.facing || self.powered {
            return;
        }
        if ctx.ticks.has_pending_at(pos, ctx.tick) {
            return;
        }
        ctx.schedule(pos, OBSERVER_PULSE_TICKS, TickPriority::Normal);
    }

    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if self.powered {
            // End of the pulse.
            ctx.set(pos, self.states.get(false));
        } else {
            // Start of the pulse, and schedule its end.
            ctx.set(pos, self.states.get(true));
            ctx.schedule(pos, OBSERVER_PULSE_TICKS, TickPriority::Normal);
        }
        // `ObserverBlock.tick` ends with `updateNeighborsInFront` on both edges:
        // the block the pulse strongly powers, and *that block's* neighbours,
        // are told about the change. This is the extra block of reach that lets
        // an observer drive a piston through a slime block — the piston is not
        // adjacent to the observer and would otherwise never re-check.
        self.update_neighbors_in_front(ctx, pos);
    }

    /// `ObserverBlock.onPlace`: a powered observer written into the world with
    /// no pending tick clears its own powered flag, silently.
    ///
    /// This is how a moved mid-pulse observer lands unpowered: its turn-off
    /// tick is stranded at the position it was pushed out of, so without this
    /// it would stay lit forever. Captured — the flying machine's east
    /// observer is pushed while pulsing and lands `powered=false`
    /// (`flying_machine.json`, tick 3).
    fn on_placed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        if !self.powered || ctx.ticks.has_pending_at(pos, ctx.tick) {
            return;
        }
        // Vanilla writes with flag 18 — visible, but no neighbour updates —
        // and then updates the front only.
        ctx.set_quiet(pos, self.states.get(false));
        self.update_neighbors_in_front(ctx, pos);
    }

    fn redstone_power(&self, _world: &World, _pos: Pos, dir: Dir) -> u8 {
        // An observer powers only out of its back face.
        if self.powered && dir == self.output_side() {
            15
        } else {
            0
        }
    }

    fn name(&self) -> &'static str {
        "observer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateId;
    use crate::pos::Bounds;
    use crate::schedule::{EventQueue, TickQueue};
    use crate::state::StateRegistry;

    const OFF: StateId = StateId(1);
    const ON: StateId = StateId(2);

    fn observer(powered: bool) -> Observer {
        Observer {
            facing: Dir::West,
            powered,
            states: StatePair { off: OFF, on: ON },
        }
    }

    fn parts() -> (World, TickQueue, EventQueue, StateRegistry) {
        (
            World::new(Bounds::new(Pos::new(-4, 0, -4), Pos::new(8, 4, 4))),
            TickQueue::new(),
            EventQueue::new(),
            StateRegistry::new(),
        )
    }

    #[test]
    fn an_observer_watches_the_block_it_faces() {
        // Same convention as repeaters and comparators: the input side is the one
        // `facing` names. Captured with a west-facing observer reacting to a block
        // placed to its west.
        let o = observer(false);
        assert_eq!(o.watched(Pos::new(1, 1, 0)), Pos::new(0, 1, 0));
        assert_eq!(o.output_side(), Dir::East, "and pulses out of its back");
    }

    #[test]
    fn a_change_in_front_schedules_a_pulse() {
        let (mut w, mut t, mut e, s) = parts();
        let pos = Pos::new(1, 1, 0);
        let o = observer(false);
        let mut ctx = TickCtx {
            world: &mut w, ticks: &mut t, events: &mut e, states: &s, tick: 0,
            boundary: false,
            updates: &mut Vec::new(), moves: &mut Vec::new(),
            toggles: &mut Vec::new(), comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        o.on_neighbor_changed(&mut ctx, pos, Dir::West);
        assert_eq!(t.len(), 1, "a watched change must schedule");
    }

    #[test]
    fn changes_behind_or_beside_are_ignored() {
        // An observer sits inside contraptions and must not react to its other
        // neighbours, or every door containing one would fire spuriously.
        let (mut w, mut t, mut e, s) = parts();
        let pos = Pos::new(1, 1, 0);
        let o = observer(false);
        let mut ctx = TickCtx {
            world: &mut w, ticks: &mut t, events: &mut e, states: &s, tick: 0,
            boundary: false,
            updates: &mut Vec::new(), moves: &mut Vec::new(),
            toggles: &mut Vec::new(), comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        for dir in [Dir::East, Dir::North, Dir::South, Dir::Up, Dir::Down] {
            o.on_neighbor_changed(&mut ctx, pos, dir);
        }
        assert!(t.is_empty(), "only the watched side counts");
    }

    #[test]
    fn the_pulse_is_two_ticks_long() {
        // Captured: powered at tick 1, unpowered at tick 3.
        let (mut w, mut t, mut e, s) = parts();
        let pos = Pos::new(1, 1, 0);
        w.set(pos, OFF);

        // Rising edge.
        {
            let mut ctx = TickCtx {
                world: &mut w, ticks: &mut t, events: &mut e, states: &s, tick: 0,
            boundary: false,
                updates: &mut Vec::new(), moves: &mut Vec::new(),
                toggles: &mut Vec::new(), comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            inv_log: None,
                log: None,
            };
            observer(false).on_scheduled_tick(&mut ctx, pos);
        }
        assert_eq!(w.get(pos), ON, "pulse starts");
        let due = t.drain_due(OBSERVER_PULSE_TICKS);
        assert_eq!(due.len(), 1, "and schedules its own end");
        assert_eq!(due[0].target, OBSERVER_PULSE_TICKS);

        // Falling edge.
        {
            let mut ctx = TickCtx {
                world: &mut w, ticks: &mut t, events: &mut e, states: &s,
                tick: OBSERVER_PULSE_TICKS,
            boundary: false,
                updates: &mut Vec::new(), moves: &mut Vec::new(),
                toggles: &mut Vec::new(), comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            inv_log: None,
                log: None,
            };
            observer(true).on_scheduled_tick(&mut ctx, pos);
        }
        assert_eq!(w.get(pos), OFF, "pulse ends");
    }

    #[test]
    fn a_powered_observer_landing_without_a_pending_tick_clears_itself() {
        // ObserverBlock.onPlace: a moved mid-pulse observer's turn-off tick is
        // stranded at its old position, so on landing it un-powers itself —
        // silently, like vanilla's flag-18 write. Captured: the flying
        // machine's east observer lands powered=false.
        let (mut w, mut t, mut e, s) = parts();
        let pos = Pos::new(1, 1, 0);
        w.set(pos, ON);
        let o = observer(true);
        let mut ctx = TickCtx {
            world: &mut w, ticks: &mut t, events: &mut e, states: &s, tick: 5,
            boundary: false,
            updates: &mut Vec::new(), moves: &mut Vec::new(),
            toggles: &mut Vec::new(), comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        o.on_placed(&mut ctx, pos);
        assert_eq!(ctx.world.get(pos), OFF, "must clear its own powered flag");

        // With a pending tick the pulse is legitimate and must be left alone.
        let (mut w, mut t, mut e, s) = parts();
        w.set(pos, ON);
        t.schedule(pos, 5, 2, crate::schedule::TickPriority::Normal);
        let mut ctx = TickCtx {
            world: &mut w, ticks: &mut t, events: &mut e, states: &s, tick: 5,
            boundary: false,
            updates: &mut Vec::new(), moves: &mut Vec::new(),
            toggles: &mut Vec::new(), comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        o.on_placed(&mut ctx, pos);
        assert_eq!(ctx.world.get(pos), ON, "a scheduled pulse is left to finish");
    }

    #[test]
    fn an_observer_powers_only_out_of_its_back() {
        let w = World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(2, 2, 2)));
        let o = observer(true);
        assert_eq!(o.redstone_power(&w, Pos::new(1, 1, 1), Dir::East), 15);
        assert_eq!(
            o.redstone_power(&w, Pos::new(1, 1, 1), Dir::West),
            0,
            "never back at what it watches"
        );
        assert_eq!(o.redstone_power(&w, Pos::new(1, 1, 1), Dir::Up), 0);
    }

    #[test]
    fn an_already_pulsing_observer_does_not_restart() {
        let (mut w, mut t, mut e, s) = parts();
        let pos = Pos::new(1, 1, 0);
        let o = observer(true); // mid-pulse
        let mut ctx = TickCtx {
            world: &mut w, ticks: &mut t, events: &mut e, states: &s, tick: 0,
            boundary: false,
            updates: &mut Vec::new(), moves: &mut Vec::new(),
            toggles: &mut Vec::new(), comparator_out: &mut Default::default(),
            inventories: &mut Default::default(),
            hopper_state: &mut Default::default(),
            inv_log: None,
            log: None,
        };
        o.on_neighbor_changed(&mut ctx, pos, Dir::West);
        assert!(t.is_empty(), "a pulse in flight must not be retriggered");
    }

    #[test]
    fn the_pulse_outlasts_the_block_dropping_threshold() {
        // A one-tick pulse drops a block from a sticky piston; an observer's is two.
        // Door designs sit right on that boundary, so the relationship is worth
        // pinning rather than leaving implicit.
        const { assert!(OBSERVER_PULSE_TICKS > 1) };
        assert_eq!(OBSERVER_PULSE_TICKS, crate::piston::PISTON_MOVE_TICKS);
    }
}
