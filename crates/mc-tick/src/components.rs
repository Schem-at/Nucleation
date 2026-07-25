//! Torches, repeaters and comparators: the components that make redstone *timed*.
//!
//! Dust settles instantly (see [`crate::redstone`]). Everything that gives a
//! contraption its timing lives here, and the timing comes from two things —
//! **delay** and **tick priority**. Both are taken from Minecraft 26.2's own
//! compiled code, which ships unobfuscated; see `redstone_components.md` for the
//! full reading.
//!
//! # Verified delays
//!
//! | component | delay | source |
//! |---|---|---|
//! | repeater | `delay` property (1–4) **× 2** game ticks | `RepeaterBlock.getDelay` |
//! | comparator | 2 game ticks | `ComparatorBlock` |
//! | torch | [`TORCH_DELAY`] game ticks | `RedstoneTorchBlock` |
//!
//! # Verified priorities, and why they matter more than the delays
//!
//! `DiodeBlock.checkTickOnNeighbor` picks one of three:
//!
//! - [`TickPriority::ExtremelyHigh`] when `shouldPrioritize` holds — the block
//!   *behind* the diode is itself a diode facing it. This is "repeater priority",
//!   and it is what makes chains of repeaters resolve in a stable order.
//! - [`TickPriority::VeryHigh`] when the diode is currently powered, i.e. about to
//!   turn **off**.
//! - [`TickPriority::High`] otherwise, i.e. turning **on**.
//!
//! A redstone torch schedules with **no priority argument at all**, so it runs at
//! [`TickPriority::Normal`] — strictly after every diode in the same tick. That
//! difference is not cosmetic: it decides which component observes which, and it is
//! the kind of detail that makes a piston fire one tick early or late.
//!
//! # What is *not* verified here
//!
//! Comparator priming, repeater locking, and torch burnout are described in
//! `redstone_components.md` but deliberately **not** implemented, because they are
//! exactly the behaviours where a plausible reading goes subtly wrong. They need
//! traces first.

use crate::behaviour::{BlockBehaviour, TickCtx};
use crate::pos::{Dir, Pos};
use crate::schedule::TickPriority;
use crate::state::StateId;
use crate::world::World;

/// Game ticks between a redstone torch seeing a change and acting on it.
///
/// One "redstone tick". `RedstoneTorchBlock` schedules with the plain
/// `scheduleTick(pos, block, delay)` overload — no priority — so torches always run
/// at [`TickPriority::Normal`].
pub const TORCH_DELAY: u64 = 2;

/// Game ticks a comparator takes to act. Fixed, unlike a repeater's.
pub const COMPARATOR_DELAY: u64 = 2;

/// Game ticks per unit of a repeater's `delay` property.
///
/// The property counts *redstone* ticks (1–4); the scheduler counts game ticks.
/// Forgetting this factor is a silent doubling or halving of every repeater in a
/// build.
pub const REPEATER_TICKS_PER_DELAY: u64 = 2;

/// Which way a diode's signal flows.
///
/// Determined empirically from a captured trace rather than assumed: a repeater
/// with `facing=east` and a redstone block to its **east** scheduled a tick, while
/// `facing=west` with the same block did not. So the input side is the side the
/// block's `facing` names.
///
/// This is recorded here because it is easy to get backwards, and backwards means
/// every diode in a build reads the wrong neighbour.
pub const INPUT_IS_FACING_SIDE: bool = true;

/// A powered/unpowered pair of states for one block configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatePair {
    /// The state when unpowered.
    pub off: StateId,
    /// The state when powered.
    pub on: StateId,
}

impl StatePair {
    /// The state for a given powered flag.
    pub fn get(&self, powered: bool) -> StateId {
        if powered {
            self.on
        } else {
            self.off
        }
    }
}

/// Reads whether a position is emitting a redstone signal.
///
/// Supplied by the caller so this module stays independent of any particular
/// power model — the same reasoning that keeps [`crate::state::StateRegistry`] free
/// of Minecraft's block list.
pub trait PowerSource: Send + Sync {
    /// Whether `pos` currently emits a signal toward `toward`.
    fn is_powered(&self, world: &World, pos: Pos, toward: Dir) -> bool;

    /// Whether the block at `pos` is a diode, for repeater-priority purposes.
    fn is_diode(&self, world: &World, pos: Pos) -> bool;

    /// Which way the diode at `pos` faces, if it is one.
    fn diode_facing(&self, world: &World, pos: Pos) -> Option<Dir>;
}

/// Shared scheduling logic for repeaters and comparators.
///
/// Mirrors `DiodeBlock.checkTickOnNeighbor`: decide whether the output should
/// change, and if so schedule at the priority the game would choose.
fn schedule_diode(
    ctx: &mut TickCtx<'_>,
    power: &dyn PowerSource,
    pos: Pos,
    facing: Dir,
    powered: bool,
    delay: u64,
) {
    let input_side = if INPUT_IS_FACING_SIDE {
        facing
    } else {
        facing.opposite()
    };
    let input = power.is_powered(ctx.world, pos.offset(input_side), input_side.opposite());

    if input == powered {
        return;
    }

    // The game refuses to double-schedule a position that already has a tick
    // pending. Skipping this check silently doubles every delay.
    if ctx.ticks.has_pending_at(pos, ctx.tick) {
        return;
    }

    let priority = if should_prioritize(ctx.world, power, pos, facing) {
        TickPriority::ExtremelyHigh
    } else if powered {
        // Turning off outranks turning on.
        TickPriority::VeryHigh
    } else {
        TickPriority::High
    };

    ctx.schedule(pos, delay, priority);
}

/// `DiodeBlock.shouldPrioritize`: is the block behind us a diode facing us?
fn should_prioritize(world: &World, power: &dyn PowerSource, pos: Pos, facing: Dir) -> bool {
    let behind = pos.offset(facing);
    if !power.is_diode(world, behind) {
        return false;
    }
    // A diode behind us only prioritises if it is aimed our way.
    power.diode_facing(world, behind) == Some(facing)
}

/// A redstone repeater.
///
/// One instance is registered per distinct block state, so it knows its own facing,
/// delay and powered flag without needing to parse a descriptor at tick time.
pub struct Repeater<P: PowerSource> {
    /// The `facing` property.
    pub facing: Dir,
    /// The `delay` property, 1–4 in redstone ticks.
    pub delay: u8,
    /// Whether this state is the powered one.
    pub powered: bool,
    /// The powered/unpowered states for this facing and delay.
    pub states: StatePair,
    /// How power is read.
    pub power: P,
}

impl<P: PowerSource> Repeater<P> {
    /// Delay in game ticks.
    pub fn delay_ticks(&self) -> u64 {
        u64::from(self.delay) * REPEATER_TICKS_PER_DELAY
    }
}

impl<P: PowerSource> BlockBehaviour for Repeater<P> {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let delay = self.delay_ticks();
        schedule_diode(ctx, &self.power, pos, self.facing, self.powered, delay);
    }

    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let input_side = if INPUT_IS_FACING_SIDE {
            self.facing
        } else {
            self.facing.opposite()
        };
        let input =
            self.power
                .is_powered(ctx.world, pos.offset(input_side), input_side.opposite());
        if input != self.powered {
            ctx.set(pos, self.states.get(input));
        }
    }

    fn redstone_power(&self, _world: &World, _pos: Pos, dir: Dir) -> u8 {
        // A repeater emits only from its output side, at full strength.
        let output = if INPUT_IS_FACING_SIDE {
            self.facing.opposite()
        } else {
            self.facing
        };
        if self.powered && dir == output {
            15
        } else {
            0
        }
    }

    fn name(&self) -> &'static str {
        "repeater"
    }
}

/// A redstone torch.
///
/// Inverts the block it is attached to, after [`TORCH_DELAY`] game ticks, and always
/// at [`TickPriority::Normal`] — torches schedule without a priority argument, so
/// they run strictly after every diode in the same tick.
pub struct Torch<P: PowerSource> {
    /// The direction of the block this torch is attached to.
    pub attached: Dir,
    /// Whether this state is lit.
    pub lit: bool,
    /// Lit/unlit states.
    pub states: StatePair,
    /// How power is read.
    pub power: P,
}

impl<P: PowerSource> BlockBehaviour for Torch<P> {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        let support = pos.offset(self.attached);
        let powered = self
            .power
            .is_powered(ctx.world, support, self.attached.opposite());
        // A torch is lit exactly when its support is *not* powered.
        if self.lit == powered && !ctx.ticks.has_pending_at(pos, ctx.tick) {
            ctx.schedule(pos, TORCH_DELAY, TickPriority::Normal);
        }
    }

    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let support = pos.offset(self.attached);
        let powered = self
            .power
            .is_powered(ctx.world, support, self.attached.opposite());
        if self.lit == powered {
            ctx.set(pos, self.states.get(!powered));
        }
    }

    fn redstone_power(&self, _world: &World, _pos: Pos, dir: Dir) -> u8 {
        // A lit torch powers every side except the one it is attached to.
        if self.lit && dir != self.attached {
            15
        } else {
            0
        }
    }

    fn name(&self) -> &'static str {
        "redstone_torch"
    }
}

/// A redstone comparator.
///
/// Shares `DiodeBlock`'s scheduling with a fixed [`COMPARATOR_DELAY`]. Comparison
/// and subtraction modes, container reading, and priming are **not** implemented —
/// see the module docs.
pub struct Comparator<P: PowerSource> {
    /// The `facing` property.
    pub facing: Dir,
    /// Whether this state is powered.
    pub powered: bool,
    /// Powered/unpowered states.
    pub states: StatePair,
    /// How power is read.
    pub power: P,
}

impl<P: PowerSource> BlockBehaviour for Comparator<P> {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        schedule_diode(
            ctx,
            &self.power,
            pos,
            self.facing,
            self.powered,
            COMPARATOR_DELAY,
        );
    }

    fn on_scheduled_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let input_side = if INPUT_IS_FACING_SIDE {
            self.facing
        } else {
            self.facing.opposite()
        };
        let input =
            self.power
                .is_powered(ctx.world, pos.offset(input_side), input_side.opposite());
        if input != self.powered {
            ctx.set(pos, self.states.get(input));
        }
    }

    fn redstone_power(&self, _world: &World, _pos: Pos, dir: Dir) -> u8 {
        let output = if INPUT_IS_FACING_SIDE {
            self.facing.opposite()
        } else {
            self.facing
        };
        if self.powered && dir == output {
            15
        } else {
            0
        }
    }

    fn name(&self) -> &'static str {
        "comparator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos::Bounds;
    use crate::schedule::{EventQueue, TickQueue};
    use crate::state::StateRegistry;

    /// A power model where designated states emit in all directions.
    #[derive(Clone)]
    struct Sources {
        powered: Vec<StateId>,
        diodes: Vec<(StateId, Dir)>,
    }

    impl PowerSource for Sources {
        fn is_powered(&self, world: &World, pos: Pos, _toward: Dir) -> bool {
            self.powered.contains(&world.get(pos))
        }
        fn is_diode(&self, world: &World, pos: Pos) -> bool {
            let state = world.get(pos);
            self.diodes.iter().any(|(s, _)| *s == state)
        }
        fn diode_facing(&self, world: &World, pos: Pos) -> Option<Dir> {
            let state = world.get(pos);
            self.diodes
                .iter()
                .find(|(s, _)| *s == state)
                .map(|(_, d)| *d)
        }
    }

    fn ctx_parts() -> (World, TickQueue, EventQueue, StateRegistry) {
        (
            World::new(Bounds::new(Pos::new(-4, 0, -4), Pos::new(8, 4, 4))),
            TickQueue::new(),
            EventQueue::new(),
            StateRegistry::new(),
        )
    }

    #[test]
    fn repeater_delay_is_the_property_times_two() {
        // Verified from RepeaterBlock.getDelay: the property is in redstone ticks,
        // the scheduler in game ticks.
        for (property, expected) in [(1u8, 2u64), (2, 4), (3, 6), (4, 8)] {
            let repeater = Repeater {
                facing: Dir::East,
                delay: property,
                powered: false,
                states: StatePair { off: StateId(1), on: StateId(2) },
                power: Sources { powered: vec![], diodes: vec![] },
            };
            assert_eq!(repeater.delay_ticks(), expected, "delay={property}");
        }
    }

    #[test]
    fn a_repeater_schedules_at_high_when_turning_on() {
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        world.set(Pos::new(1, 1, 0), source);

        let repeater = Repeater {
            facing: Dir::East,
            delay: 1,
            powered: false,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            events: &mut events,
            states: &states,
            tick: 0,
            updates: &mut Vec::new(),
        };
        repeater.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::East);

        let due = ticks.drain_due(2);
        assert_eq!(due.len(), 1, "must schedule");
        assert_eq!(due[0].priority, TickPriority::High, "turning on is HIGH");
        assert_eq!(due[0].target, 2, "delay 1 == 2 game ticks");
    }

    #[test]
    fn a_powered_repeater_turning_off_schedules_at_very_high() {
        // Verified ordering: turning off outranks turning on.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let repeater = Repeater {
            facing: Dir::East,
            delay: 1,
            powered: true,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            events: &mut events,
            states: &states,
            tick: 0,
            updates: &mut Vec::new(),
        };
        repeater.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::East);

        let due = ticks.drain_due(2);
        assert_eq!(due[0].priority, TickPriority::VeryHigh);
    }

    #[test]
    fn a_repeater_fed_by_another_diode_jumps_the_queue() {
        // DiodeBlock.shouldPrioritize -> EXTREMELY_HIGH. This is what makes repeater
        // chains resolve in a stable order.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let upstream = StateId(5);
        world.set(Pos::new(1, 1, 0), upstream);

        let repeater = Repeater {
            facing: Dir::East,
            delay: 1,
            powered: false,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources {
                powered: vec![source, upstream],
                // The upstream diode faces east: the same way we look at it.
                diodes: vec![(upstream, Dir::East)],
            },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            events: &mut events,
            states: &states,
            tick: 0,
            updates: &mut Vec::new(),
        };
        repeater.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::East);

        let due = ticks.drain_due(2);
        assert_eq!(due[0].priority, TickPriority::ExtremelyHigh);
    }

    #[test]
    fn a_diode_never_double_schedules() {
        // The game checks for a pending tick first. Without it every delay doubles.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        world.set(Pos::new(1, 1, 0), source);

        let repeater = Repeater {
            facing: Dir::East,
            delay: 2,
            powered: false,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let pos = Pos::new(0, 1, 0);
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            events: &mut events,
            states: &states,
            tick: 0,
            updates: &mut Vec::new(),
        };
        repeater.on_neighbor_changed(&mut ctx, pos, Dir::East);
        repeater.on_neighbor_changed(&mut ctx, pos, Dir::East);
        repeater.on_neighbor_changed(&mut ctx, pos, Dir::East);

        assert_eq!(ticks.len(), 1, "three notifications, one scheduled tick");
    }

    #[test]
    fn a_torch_schedules_at_normal_so_it_runs_after_every_diode() {
        // RedstoneTorchBlock uses the scheduleTick overload without a priority, so
        // torches sit at NORMAL while diodes sit at HIGH or above. That ordering
        // decides which component observes which.
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let support = Pos::new(0, 0, 0);
        world.set(support, source);

        let torch = Torch {
            attached: Dir::Down,
            lit: true,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            events: &mut events,
            states: &states,
            tick: 0,
            updates: &mut Vec::new(),
        };
        torch.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::Down);

        let due = ticks.drain_due(TORCH_DELAY);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].priority, TickPriority::Normal);
        assert_eq!(due[0].target, TORCH_DELAY);
    }

    #[test]
    fn a_torch_inverts_its_support() {
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        let lit = StateId(2);
        let unlit = StateId(1);
        let torch_pos = Pos::new(0, 1, 0);
        world.set(Pos::new(0, 0, 0), source);
        world.set(torch_pos, lit);

        let torch = Torch {
            attached: Dir::Down,
            lit: true,
            states: StatePair { off: unlit, on: lit },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            events: &mut events,
            states: &states,
            tick: 0,
            updates: &mut Vec::new(),
        };
        torch.on_scheduled_tick(&mut ctx, torch_pos);

        assert_eq!(world.get(torch_pos), unlit, "powered support unlights the torch");
    }

    #[test]
    fn a_lit_torch_powers_every_side_but_its_support() {
        let torch = Torch {
            attached: Dir::Down,
            lit: true,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![], diodes: vec![] },
        };
        let world = World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(1, 1, 1)));
        assert_eq!(torch.redstone_power(&world, Pos::new(0, 1, 0), Dir::Up), 15);
        assert_eq!(torch.redstone_power(&world, Pos::new(0, 1, 0), Dir::North), 15);
        assert_eq!(
            torch.redstone_power(&world, Pos::new(0, 1, 0), Dir::Down),
            0,
            "never back into its own support"
        );
    }

    #[test]
    fn comparator_delay_is_fixed_at_two() {
        let (mut world, mut ticks, mut events, states) = ctx_parts();
        let source = StateId(9);
        world.set(Pos::new(1, 1, 0), source);

        let comparator = Comparator {
            facing: Dir::East,
            powered: false,
            states: StatePair { off: StateId(1), on: StateId(2) },
            power: Sources { powered: vec![source], diodes: vec![] },
        };
        let mut ctx = TickCtx {
            world: &mut world,
            ticks: &mut ticks,
            events: &mut events,
            states: &states,
            tick: 0,
            updates: &mut Vec::new(),
        };
        comparator.on_neighbor_changed(&mut ctx, Pos::new(0, 1, 0), Dir::East);

        let due = ticks.drain_due(COMPARATOR_DELAY);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].target, COMPARATOR_DELAY, "always 2, unlike a repeater");
    }

    #[test]
    fn diode_priorities_order_correctly_against_each_other() {
        // The ordering that actually matters when several fire in one tick.
        assert!(TickPriority::ExtremelyHigh < TickPriority::VeryHigh);
        assert!(TickPriority::VeryHigh < TickPriority::High);
        assert!(TickPriority::High < TickPriority::Normal);
    }
}
