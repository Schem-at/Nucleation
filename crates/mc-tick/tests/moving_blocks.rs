//! What the engine knows about the blocks it currently has in flight.
//!
//! A renderer cannot animate a piston stroke from the block-change stream: the
//! stream says a cell became a `moving_piston` placeholder, not which block set
//! off, which way it is travelling, or which tick it arrives. Reconstructing
//! that downstream is a reimplementation of piston mechanics, and it desyncs —
//! the flight and the landing are then decided by two different clocks.
//!
//! [`Simulation::moving_blocks`] is the engine answering instead.

use std::collections::BTreeSet;

use mc_test::SettleMode;
use mc_tick::{Pos, Simulation, Structure};

const FLYER: &str = include_str!("corpus/structures/flying_machine_east.snbt");
/// Four sticky pistons facing east, extended, each held out by a redstone
/// block behind it. Break one and that piston retracts with nothing to pull.
const PULL: &str = include_str!("corpus/structures/piston_pull.snbt");

fn sim(snbt: &str) -> Simulation {
    let structure = Structure::parse(snbt).expect("fixture parses");
    let actuators = [
        "minecraft:redstone_block".to_string(),
        "minecraft:air".to_string(),
    ];
    mc_test::build_sim(
        &structure,
        Pos::new(0, 0, 0),
        SettleMode::InWorld,
        &actuators,
        &[],
        None,
        "moving_blocks",
    )
}

/// Kick the flying machine and step until it first has something in flight.
fn kicked() -> Simulation {
    let mut sim = sim(FLYER);
    let redstone = sim
        .registry_mut()
        .intern("minecraft:redstone_block")
        .expect("redstone block");
    sim.place_block(Pos::new(2, 1, 1), redstone);
    for _ in 0..32 {
        sim.step();
        if !sim.moving_blocks().is_empty() {
            return sim;
        }
    }
    panic!("a kicked flying machine should put blocks in flight within 32 ticks");
}

#[test]
fn a_stroke_reports_every_block_it_carries_over_one_shared_flight_window() {
    let sim = kicked();
    let flying = sim.moving_blocks();

    let started: BTreeSet<u64> = flying.iter().map(|m| m.started_on).collect();
    let lands: BTreeSet<u64> = flying.iter().map(|m| m.lands_on).collect();
    assert_eq!(
        started.len(),
        1,
        "blocks dispatched by one stroke set off together: {flying:?}"
    );
    assert_eq!(
        lands.len(),
        1,
        "and they land together: {flying:?}"
    );
    // `tick_count` counts *completed* ticks, so the tick that just ran — and
    // dispatched this stroke — is the one before it.
    assert_eq!(
        started.into_iter().next(),
        Some(sim.tick_count() - 1),
        "a flight begins on the tick its block event ran"
    );
    assert_eq!(
        lands.into_iter().next(),
        Some(sim.tick_count() - 1 + mc_tick::PISTON_MOVE_TICKS),
        "and lands two ticks later, as vanilla does"
    );
}

/// A retracting piston is the one move where the state that *lands* is not the
/// state that *travels*, and vanilla's client says so explicitly.
/// `PistonHeadRenderer.extractRenderState` (26.2) has a branch for
/// `isSourcePiston() && !isExtending()`: it puts a **`piston_head`** in the
/// moving slot and the base — with `EXTENDED=true` — in a separate `base`
/// slot, and `submit` applies the interpolated offset to the moving slot only,
/// outside of which `base` is drawn. The body stays in its cell; the arm comes
/// home.
///
/// Reporting only the landing state (`piston[extended=false]`) makes a
/// consumer slide the whole piston body a block backwards out of the head
/// slot, which is what the sim lab did until this test existed.
#[test]
fn a_retracting_piston_walks_its_head_home_and_leaves_its_body_where_it_is() {
    let mut sim = sim(PULL);
    let air = sim.registry_mut().intern("minecraft:air").expect("air");
    // Break the redstone block holding the z=1 piston out.
    sim.place_block(Pos::new(1, 1, 1), air);
    let mut flying = Vec::new();
    for _ in 0..32 {
        sim.step();
        flying = sim.moving_blocks();
        if !flying.is_empty() {
            break;
        }
    }
    let base = flying
        .iter()
        .find(|m| m.source_piston)
        .expect("the retracting piston's own square is in flight");
    let d = |s| sim.registry().descriptor(s).unwrap_or("?").to_string();

    assert!(!base.extending, "this is a retraction");
    assert_eq!(base.to, Pos::new(2, 1, 1), "the piston's own cell");
    assert_eq!(base.from, Pos::new(3, 1, 1), "the head slot it comes home from");
    assert!(
        d(base.state).contains("sticky_piston") && d(base.state).contains("extended=false"),
        "the retracted base is what lands: {}",
        d(base.state)
    );
    assert!(
        d(base.carried).starts_with("minecraft:piston_head"),
        "but the head is what travels: {}",
        d(base.carried)
    );
    assert!(
        d(base.carried).contains("facing=east"),
        "pointing the way the piston faces: {}",
        d(base.carried)
    );
    let remains = base.remains.expect("the body is drawn while the arm returns");
    assert!(
        d(remains).contains("sticky_piston") && d(remains).contains("extended=true"),
        "and it is drawn extended, as vanilla's `base` slot is: {}",
        d(remains)
    );
}

/// Vanilla draws a moving piston head with a **shortened arm** while the head
/// is within half a block of its base, and the full arm once it is further
/// out. `PistonHeadRenderer.extractRenderState` (26.2) spells that as two
/// opposite comparisons — `SHORT = progress <= 0.5` on the extension branch
/// and `SHORT = progress >= 0.5` on the retraction branch — which are the same
/// rule read from each end, because extension travels away from the base and
/// retraction travels back to it.
///
/// Without the short form the shaft is drawn full length all the way home and
/// visibly pokes out the back of the piston body.
#[test]
fn a_moving_piston_head_offers_the_shortened_arm_drawn_next_to_the_body() {
    let mut sim = sim(PULL);
    let air = sim.registry_mut().intern("minecraft:air").expect("air");
    sim.place_block(Pos::new(1, 1, 1), air);
    let mut flying = Vec::new();
    for _ in 0..32 {
        sim.step();
        flying = sim.moving_blocks();
        if !flying.is_empty() {
            break;
        }
    }
    let head = flying
        .iter()
        .find(|m| m.source_piston)
        .expect("the retracting piston's arm");
    let d = |s| sim.registry().descriptor(s).unwrap_or("?").to_string();

    let short = head
        .carried_short
        .expect("a moving head has a shortened form");
    assert!(
        d(short).starts_with("minecraft:piston_head") && d(short).contains("short=true"),
        "the shortened arm is the same head with short=true: {}",
        d(short)
    );
    assert_eq!(
        d(short).replace("short=true", "short=false"),
        d(head.carried),
        "and differs from the long form in nothing else"
    );
}

#[test]
fn an_extending_head_offers_one_too() {
    let sim = kicked();
    let head = sim
        .moving_blocks()
        .into_iter()
        .find(|m| m.source_piston && m.extending)
        .expect("the flying machine extends a piston");
    let d = |s| sim.registry().descriptor(s).unwrap_or("?").to_string();
    assert!(
        head.carried_short.is_some_and(|s| d(s).contains("short=true")),
        "an extending arm is shortened near the base too: {:?}",
        head.carried_short.map(d)
    );
}

#[test]
fn an_ordinary_carried_block_travels_as_itself_and_leaves_nothing_behind() {
    let sim = kicked();
    for m in sim.moving_blocks().iter().filter(|m| !m.source_piston) {
        assert_eq!(m.carried, m.state, "a pushed block is drawn as what it is");
        assert_eq!(m.remains, None, "and its cell is empty until it arrives");
        assert_eq!(m.carried_short, None, "only a piston arm has a short form");
    }
}

#[test]
fn a_flight_names_the_cell_its_block_left() {
    let sim = kicked();
    for m in sim.moving_blocks() {
        assert_eq!(
            m.from,
            m.to.offset(m.travel.opposite()),
            "a piston move is exactly one cell along its travel: {m:?}"
        );
        assert_ne!(m.from, m.to, "an in-flight block is going somewhere");
    }
}

/// The contract a renderer polling after every `step` actually depends on:
/// a flight is listed for exactly as long as its cell is not yet written, and
/// the two swap over on one tick with no frame showing both or neither.
#[test]
fn a_flight_lives_until_its_block_lands_and_not_past_it() {
    let mut sim = kicked();
    let flight = sim
        .moving_blocks()
        .into_iter()
        .next()
        .expect("kicked() guarantees one");

    // Live for every completed tick up to and including the landing tick; the
    // last of those is the tick vanilla has the block visually arrived on while
    // the world still holds its `moving_piston` placeholder.
    while sim.tick_count() <= flight.lands_on {
        assert!(
            sim.moving_blocks().iter().any(|m| m.to == flight.to),
            "still in flight at tick {}, landing on {}",
            sim.tick_count(),
            flight.lands_on
        );
        assert_ne!(
            sim.world().get(flight.to),
            flight.state,
            "and its destination is not written until it retires"
        );
        sim.step();
    }

    assert!(
        !sim.moving_blocks().iter().any(|m| m.to == flight.to),
        "the flight retires once its landing tick has run"
    );
    assert_eq!(
        sim.world().get(flight.to),
        flight.state,
        "on the same tick the real block appears — the renderer never draws two, or none"
    );
}
