//! End-to-end: a real circuit driven through the public `Simulation` API.
//!
//! The unit tests exercise each component in isolation. This exercises the thing
//! that actually matters — components triggering each other across the tick phases,
//! with the delays and priorities that decide a contraption's timing.
//!
//! The circuit is the smallest one that proves the architecture:
//!
//! ```text
//!   lever ──► repeater (delay 1) ──► piston ──► [stone]
//! ```
//!
//! Flipping the lever must take the repeater's delay to reach the piston, and the
//! piston must then move **within the same tick** its repeater fired — because a
//! piston queues a block event (phase 7) rather than scheduling a tick.

use mc_tick::behaviour::Inert;
use mc_tick::components::{PowerSource, Repeater, StatePair};
use mc_tick::piston::{Movability, Piston};
use mc_tick::{Bounds, Dir, Pos, Simulation, StateId, World};

/// A power model over a fixed set of emitting states.
#[derive(Clone)]
struct Model {
    powered: Vec<StateId>,
    diodes: Vec<(StateId, Dir)>,
    immovable: Vec<StateId>,
}

impl PowerSource for Model {
    fn is_powered(
        &self,
        world: &World,
        _outs: &mc_tick::behaviour::ComparatorOutputs,
        pos: Pos,
        _toward: Dir,
    ) -> bool {
        self.powered.contains(&world.get(pos))
    }
    fn is_diode(&self, world: &World, pos: Pos) -> bool {
        let s = world.get(pos);
        self.diodes.iter().any(|(d, _)| *d == s)
    }
    fn diode_facing(&self, world: &World, pos: Pos) -> Option<Dir> {
        let s = world.get(pos);
        self.diodes.iter().find(|(d, _)| *d == s).map(|(_, f)| *f)
    }
}

impl Movability for Model {
    fn is_movable(&self, world: &World, pos: Pos) -> bool {
        let s = world.get(pos);
        s != StateId::AIR && !self.immovable.contains(&s)
    }
}

struct Circuit {
    sim: Simulation,
    lever_off: StateId,
    lever_on: StateId,
    piston_out: StateId,
    head: StateId,
    lever_at: Pos,
    piston_at: Pos,
}

/// Wire up lever → repeater → piston.
fn build(repeater_delay: u8) -> Circuit {
    let mut sim = Simulation::new(Bounds::new(Pos::new(-4, 0, -4), Pos::new(12, 4, 4)));

    let intern = |sim: &mut Simulation, s: &str| sim.registry_mut().intern(s).unwrap();
    let lever_off = intern(&mut sim, "minecraft:lever[powered=false]");
    let lever_on = intern(&mut sim, "minecraft:lever[powered=true]");
    let rep_off = intern(&mut sim, "minecraft:repeater[powered=false]");
    let rep_on = intern(&mut sim, "minecraft:repeater[powered=true]");
    let piston_in = intern(&mut sim, "minecraft:piston[extended=false]");
    let piston_out = intern(&mut sim, "minecraft:piston[extended=true]");
    let head = intern(&mut sim, "minecraft:piston_head");
    let moving = intern(&mut sim, "minecraft:moving_piston");
    let stone = intern(&mut sim, "minecraft:stone");

    let model = Model {
        powered: vec![lever_on, rep_on],
        diodes: vec![(rep_off, Dir::West), (rep_on, Dir::West)],
        immovable: vec![],
    };

    // Lever and stone are inert; they only matter as power sources and cargo.
    for (state, name) in [
        (lever_off, "lever"),
        (lever_on, "lever"),
        (stone, "stone"),
        (head, "piston_head"),
        (moving, "moving_piston"),
    ] {
        sim.behaviours_mut()
            .register(state, Box::new(Inert::new(name)));
    }

    for (state, powered) in [(rep_off, false), (rep_on, true)] {
        sim.behaviours_mut().register(
            state,
            Box::new(Repeater {
                facing: Dir::West,
                delay: repeater_delay,
                powered,
                states: StatePair {
                    off: rep_off,
                    on: rep_on,
                },
                locked: false,
                locked_twin: None,
                power: model.clone(),
            }),
        );
    }

    for (state, extended) in [(piston_in, false), (piston_out, true)] {
        sim.behaviours_mut().register(
            state,
            Box::new(Piston {
                facing: Dir::East,
                extended,
                sticky: false,
                states: StatePair {
                    off: piston_in,
                    on: piston_out,
                },
                head,
                head_short: head,
                moving,
                moving_block: moving,
                power: model.clone(),
                movability: model.clone(),
            }),
        );
    }

    // Layout along +X:  lever -> repeater -> piston -> stone
    //
    // The repeater sits *behind* the piston, not in front of it. An earlier version
    // put it in the push path, and the simulation faithfully reproduced the
    // consequence: the piston shoved its own power source out of range, lost power,
    // and retracted inside the same event chain. The engine was right and the
    // circuit was self-defeating — a good reminder that "the test failed" and "the
    // code is wrong" are different claims.
    //
    // A repeater outputs opposite its facing side, so facing=west outputs east into
    // the piston and takes its input from the west, where the lever sits.
    let lever_at = Pos::new(-2, 1, 0);
    let repeater_at = Pos::new(-1, 1, 0);
    let piston_at = Pos::new(0, 1, 0);
    let cargo_at = Pos::new(1, 1, 0);

    sim.world_mut().set(lever_at, lever_off);
    sim.world_mut().set(repeater_at, rep_off);
    sim.world_mut().set(piston_at, piston_in);
    sim.world_mut().set(cargo_at, stone);
    sim.mark_initial();

    Circuit {
        sim,
        lever_off,
        lever_on,
        piston_out,
        head,
        lever_at,
        piston_at,
    }
}

#[test]
fn a_lever_drives_a_repeater_which_drives_a_piston() {
    let mut c = build(1);

    // Flip the lever: an external change, so the world is told to notify.
    c.sim.world_mut().set(c.lever_at, c.lever_on);
    c.sim.notify_neighbors(c.lever_at);

    let reason = c.sim.run_until_quiescent(40);

    assert_eq!(
        c.sim.world().get(c.piston_at),
        c.piston_out,
        "the piston must have extended; stop reason was {reason:?}"
    );
    assert_eq!(
        c.sim.world().get(c.piston_at.offset(Dir::East)),
        c.head,
        "and placed its head"
    );
    assert_eq!(
        c.sim.unknown_report(),
        None,
        "every block must be implemented"
    );
}

#[test]
fn the_repeater_delay_sets_when_the_piston_fires() {
    // The point of the whole exercise: delay is observable in ticks, and a longer
    // repeater delays the piston by exactly the extra game ticks.
    let mut fast = build(1);
    fast.sim.world_mut().set(fast.lever_at, fast.lever_on);
    fast.sim.notify_neighbors(fast.lever_at);
    fast.sim.run_until_quiescent(60);
    let fast_ticks = fast.sim.tick_count();

    let mut slow = build(4);
    slow.sim.world_mut().set(slow.lever_at, slow.lever_on);
    slow.sim.notify_neighbors(slow.lever_at);
    slow.sim.run_until_quiescent(60);
    let slow_ticks = slow.sim.tick_count();

    assert_eq!(
        slow_ticks - fast_ticks,
        6,
        "delay 4 is 8 game ticks and delay 1 is 2, so exactly 6 later \
         (fast={fast_ticks}, slow={slow_ticks})"
    );
}

#[test]
fn the_piston_moves_in_the_same_tick_its_repeater_fires() {
    // A piston queues a block event (phase 7) rather than scheduling a tick, so it
    // must move during the very tick the repeater powered it — not the next one.
    // Stepping one tick at a time is what makes that observable.
    let mut c = build(1);
    c.sim.world_mut().set(c.lever_at, c.lever_on);
    c.sim.notify_neighbors(c.lever_at);

    let mut extended_on = None;
    for _ in 0..20 {
        c.sim.step();
        if c.sim.world().get(c.piston_at) == c.piston_out {
            extended_on = Some(c.sim.tick_count());
            break;
        }
    }

    let tick = extended_on.expect("the piston must extend");
    // The lever flip is a *boundary* action — it happens between ticks, where the
    // game time still reads the last completed tick — so the repeater's 2-game-tick
    // schedule fires during tick 1, not tick 2. Captured: a repeater scheduled at
    // the placement boundary turns on at trace tick 1 (`rep_boundary.json`), and an
    // observer clicked at a boundary pulses one tick after the click, not two.
    // The piston's block event resolves in phase 7 of that same tick 1. Since
    // tick_count reports *completed* ticks, it reads 2 immediately afterwards.
    //
    // The load-bearing claim is the relationship, not the number: the piston moves
    // in the repeater's own tick. Were the move modelled as a scheduled tick it
    // would land strictly later.
    assert_eq!(tick, 2, "extended after tick {tick}");
}

#[test]
fn releasing_the_lever_retracts_the_piston() {
    let mut c = build(1);
    c.sim.world_mut().set(c.lever_at, c.lever_on);
    c.sim.notify_neighbors(c.lever_at);
    c.sim.run_until_quiescent(40);
    assert_eq!(c.sim.world().get(c.piston_at), c.piston_out);

    c.sim.world_mut().set(c.lever_at, c.lever_off);
    c.sim.notify_neighbors(c.lever_at);
    c.sim.run_until_quiescent(40);

    assert_ne!(
        c.sim.world().get(c.piston_at),
        c.piston_out,
        "the piston must retract when power is removed"
    );
}

#[test]
fn reset_replays_the_circuit_identically() {
    // Determinism is the whole quality mechanism: the same inputs must give the
    // same result, or trace comparison means nothing.
    let mut c = build(2);

    c.sim.world_mut().set(c.lever_at, c.lever_on);
    c.sim.notify_neighbors(c.lever_at);
    c.sim.run_until_quiescent(40);
    let first_tick = c.sim.tick_count();
    let first_world = c.sim.world().clone();

    c.sim.reset();
    c.sim.world_mut().set(c.lever_at, c.lever_on);
    c.sim.notify_neighbors(c.lever_at);
    c.sim.run_until_quiescent(40);

    assert_eq!(c.sim.tick_count(), first_tick, "same tick count");
    assert_eq!(c.sim.world(), &first_world, "same final world");
}
