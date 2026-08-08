//! The verified sequential cell: the master-slave repeater-lock DFF from
//! `redstone-eda/seq_cells.py` (`build_dff`), frozen as template data.
//!
//! Mechanism (probed in seq_probe.py, characterized in seq_cells.py): a
//! repeater whose side is entered by a powered repeater is `locked=true` and
//! freezes. DFF = two data repeaters with side lockers on opposite clock
//! phases — master lock <- CLK, slave lock <- NOT(CLK) via a wall-torch
//! inverter. On the rising edge the master locks 2 gt BEFORE the slave
//! opens, so capture is glitch-free.
//!
//! Cell frame (local coords): x 0..12, z -1..5, y 0..3.
//! Ports: D (0,1,0) west, Q (12,1,0) east (buffered, over a y3 flyover that
//! caps the clock column — legal, that run is straight and flat),
//! clk_in (10,1,-1) north. `clk_out` (10,1,5) is DELIBERATELY omitted: the
//! PLA compiler distributes the clock on a spine north of the bank with one
//! branch per cell, not by z-abutment chaining.
//!
//! Characterization (empirical, exact, game ticks): setup 0, hold 3,
//! min pulse 3, clk->Q 10, min period (cell alone) 20, skew 2 gt per
//! chained repeater.
//!
//! Initial state is carried BY CONSTRUCTION: the state-bearing element (the
//! slave repeater) and its lock path are authored at the declared Q, so the
//! placement settle converges to that state before any clock edge — the
//! slave is locked at rest (CLK low -> NOT(CLK) high), which also cuts every
//! sequential feedback loop and makes the at-rest build quiescent.

/// DFF characterization: setup, in game ticks.
pub const DFF_SETUP_GT: u32 = 0;
/// DFF characterization: hold, in game ticks.
pub const DFF_HOLD_GT: u32 = 3;
/// DFF characterization: minimum clock pulse width, in game ticks.
pub const DFF_MIN_PULSE_GT: u32 = 3;
/// DFF characterization: clk -> Q through the Q buffer, in game ticks.
pub const DFF_CLK_TO_Q_GT: u32 = 10;
/// DFF characterization: minimum period of the cell alone, in game ticks.
pub const DFF_MIN_PERIOD_GT: u32 = 20;
/// Clock skew added per repeater on the distribution spine, in game ticks.
pub const SPINE_SKEW_PER_REPEATER_GT: u32 = 2;

/// D port, cell-local.
pub const DFF_D: (i32, i32, i32) = (0, 1, 0);
/// Q port, cell-local.
pub const DFF_Q: (i32, i32, i32) = (12, 1, 0);
/// Clock-in port, cell-local (fed from the north).
pub const DFF_CLK_IN: (i32, i32, i32) = (10, 1, -1);

/// One latch instance handed to the PLA compiler as part of a `seq` stage.
#[derive(Debug, Clone)]
pub struct SeqCell {
    /// Bank slice this DFF occupies (east of every combinational slice).
    pub slice: i32,
    /// The prim-graph vid computing D, raised to the top level so the bank
    /// is always a next-stage hop; `None` when D folded to a constant.
    pub d: Option<String>,
    /// The folded D value when `d` is `None`.
    pub d_const: u8,
    /// The stage-0 input rail this DFF's Q drives (`<q vid>.lv`).
    pub q_rail: String,
    /// Stage-0 slice of that rail (where the wrap corridor re-enters).
    pub q_slice: i32,
    /// Baked initial Q (0/1).
    pub init: u8,
    /// Label prefix for this cell's template dust.
    pub label: String,
}

fn dust_state(power: u8) -> String {
    format!("minecraft:redstone_wire[east=none,north=none,power={power},south=none,west=none]")
}

fn repeater_state(input_from: &str, locked: bool, powered: bool) -> String {
    format!("minecraft:repeater[facing={input_from},delay=1,locked={locked},powered={powered}]")
}

/// Every cell of one DFF, local coords, authored at initial state `init`.
///
/// Returns `(pos, block state, dust net label)` — the label is `None` for
/// structure. States are the exact `seq_cells.build_dff` geometry; the init
/// bake sets the slave lock engaged (CLK low at rest), the NOT(CLK) feed
/// high, and — for `init == 1` — the slave + Q buffer powered with the Q
/// path dust at its settled signal levels.
pub fn dff_cells(init: u8) -> Vec<((i32, i32, i32), String, Option<&'static str>)> {
    let stone = |role: &str| -> String { crate::pla::palette(role).to_string() };
    let q1 = init == 1;
    let mut out: Vec<((i32, i32, i32), String, Option<&'static str>)> = Vec::new();
    let mut solid = |x: i32, y: i32, z: i32, role: &str| {
        out.push(((x, y, z), stone(role), None));
    };

    // data row z0 floors: D -> master -> slave -> buffer -> Q
    for x in [0, 1, 3, 5, 7, 12] {
        solid(x, 0, 0, "lane");
    }
    solid(2, 0, 0, "gate");
    solid(4, 0, 0, "gate");
    solid(6, 0, 0, "gate");
    // lock repeater floors (z1) + clk/nclk feed floors (z2, z3)
    solid(2, 0, 1, "gate");
    solid(4, 0, 1, "gate");
    solid(2, 0, 2, "route");
    solid(4, 0, 2, "inv");
    solid(6, 0, 3, "route");
    solid(2, 0, 3, "route");
    // clk row z4 floors x2..x10
    for x in 2..=10 {
        solid(x, 0, 4, "route");
    }
    // clock chain column x10 floors
    solid(10, 0, -1, "route");
    solid(10, 0, 0, "route");
    solid(10, 0, 1, "route");
    solid(10, 0, 2, "route");
    solid(10, 0, 3, "route");
    // Q flyover supports (the (10,2,0) cap sits ON the clock dust — legal,
    // that run is straight and flat)
    solid(8, 1, 0, "tap");
    solid(9, 2, 0, "tap");
    solid(10, 2, 0, "tap");
    solid(11, 1, 0, "tap");
    // NOT(CLK) torch base
    solid(6, 1, 2, "inv");

    // -- components ---------------------------------------------------------
    let mut dust = |x: i32, y: i32, z: i32, power: u8, lab: &'static str| {
        out.push(((x, y, z), dust_state(power), Some(lab)));
    };
    // data row
    dust(0, 1, 0, 0, "d");
    dust(1, 1, 0, 0, "d");
    dust(3, 1, 0, 0, "m");
    dust(5, 1, 0, if q1 { 15 } else { 0 }, "s");
    dust(7, 1, 0, if q1 { 15 } else { 0 }, "q");
    // clk / nclk feeds; nclk is HIGH at rest (CLK low)
    dust(2, 1, 2, 0, "clk");
    dust(4, 1, 2, 15, "nclk");
    dust(6, 1, 3, 0, "clk");
    dust(2, 1, 3, 0, "clk");
    for x in 2..=10 {
        dust(x, 1, 4, 0, "clk");
    }
    // clock chain column (clk_out omitted — spine-fed, see module docs)
    dust(10, 1, -1, 0, "clk");
    dust(10, 1, 0, 0, "clk");
    dust(10, 1, 1, 0, "clk");
    dust(10, 1, 3, 0, "clk");
    // Q flyover
    dust(8, 2, 0, if q1 { 14 } else { 0 }, "q");
    dust(9, 3, 0, if q1 { 13 } else { 0 }, "q");
    dust(10, 3, 0, if q1 { 12 } else { 0 }, "q");
    dust(11, 2, 0, if q1 { 11 } else { 0 }, "q");
    dust(12, 1, 0, if q1 { 10 } else { 0 }, "q");

    // repeaters: master data, slave data (the state bit), Q buffer,
    // master lock (clk), slave lock (nclk — ENGAGED at rest)
    out.push(((2, 1, 0), repeater_state("west", false, false), None));
    out.push(((4, 1, 0), repeater_state("west", true, q1), None));
    out.push(((6, 1, 0), repeater_state("west", false, q1), None));
    out.push(((2, 1, 1), repeater_state("south", false, false), None));
    out.push(((4, 1, 1), repeater_state("south", false, true), None));
    // clock chain refresh repeater
    out.push(((10, 1, 2), repeater_state("north", false, false), None));
    // NOT(CLK): wall torch on the west face of the (6,1,2) base, lit at rest
    out.push((
        (5, 1, 2),
        "minecraft:redstone_wall_torch[facing=west,lit=true]".to_string(),
        Some("nclk"),
    ));
    out
}
