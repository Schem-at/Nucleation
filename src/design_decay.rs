//! SIGNAL-STRENGTH BUDGETING for the bus router: how much reach the carrier has
//! left, tracked as a running LEDGER instead of guessed at with one distance
//! constant.
//!
//! # Why a ledger and not a pitch
//!
//! [`crate::design`] used to decide refresh spacing from a single unconditional
//! constant (`REFRESH_AT = 7` dust cells, a repeater every ~8). Dust reaches 15,
//! so that is about twice as dense as the physics needs — and an adversarial
//! demolition sweep confirmed it: deleting every repeater closer than 15 to its
//! neighbour still delivered every word on 54 of 63 measured problems.
//!
//! But the same widening BROKE nine form-conversion / level-shift problems, so
//! "raise the constant" is not the fix. The reason is that 7 was doing two jobs
//! at once:
//!
//! 1. it bounded the decay of the run it was emitting (its stated job), and
//! 2. it silently reserved half of dust's reach as slack for the strength spent
//!    by geometry the run does not see — the form adapter upstream of the
//!    driver anchor, the form adapter downstream of the sink anchor, and the
//!    dust cells inside a crossing window on either side of the window's own
//!    repeater. None of those were debited anywhere.
//!
//! Halving the reach hid all three. Widening the constant to the real reach
//! exposed all three at once, which is what the nine failures are.
//!
//! This module makes job 2 explicit. A [`DecayLedger`] is seeded with the debt
//! the signal ARRIVES with, carries the headroom it must LEAVE for whatever
//! comes after, and is debited per cell by the carrier's real decay. A station
//! is then inserted only when the budget would genuinely run out — which leaves
//! the first and last stretch of a run as tight as they always were while the
//! interior stretches widen to the full reach.
//!
//! # Cross-checked against the measured physics
//!
//! The per-cell numbers here match `redstone-eda/materials.py` (`Mechanism`'s
//! `decay` / `refresh` fields) and [`crate::routing::transport::Mechanism`]'s
//! `decay()` / `refreshes()`. A divergence between those three tables has been
//! a real bug class in this repo, so [`Carrier::decay_per_cell`] is written to
//! be read side by side with them.
//!
//! # CORRECTION to `redstone-eda/adversarial/REPORT.md` §8
//!
//! That report is what motivated this module, and implementing it turned up two
//! places where its evidence does not say what it claims. Recorded here because
//! this is the code the claims are about; the report itself belongs to another
//! author and is left alone.
//!
//! **1. "The conservative pitch is load-bearing" is false as stated.** All nine
//! of §8's must-stay-dense problems pass every vector against this module, five
//! of them with FEWER repeaters than the constant emitted. What those nine
//! actually need is for the strength their adapters, crossing windows and level
//! shifts spend to be ACCOUNTED — not for the pitch to stay at 7. The pitch was
//! only ever a proxy for that accounting, and a bad one, because it charged
//! every straight run in the design for slack that three specific geometries
//! owed.
//!
//! **2. The demolition test is ANISOTROPIC, so 947 is a lower bound.**
//! `prune_check.py` buckets repeaters by `(y, z)` and thins along **x only**. A
//! run along z puts every repeater in its own single-element bucket, so it can
//! never be pruned and was never tested. It also selects on the block name
//! alone, so it does not distinguish a refresh station from a structural diode —
//! for an x-axis row (`p_form_4`, `flat_x`) that deletes the per-bit gather
//! diodes which are the only thing isolating one bit's lane from the next, and
//! the failure it records there is crosstalk rather than decay.
//!
//! Deliberately NOT claimed: that all nine failed for that reason. Six of them
//! (`p_form_2`, `t3_01`..`t3_05`) convert to or from a **z**-stepping row, whose
//! gather the x-only rule cannot touch, so their pruned repeaters really were
//! refresh stations and their failures really were strength. Which mechanism
//! killed which problem was not isolated here; only the conclusion above is
//! measured.
//!
//! **2. The ACCEPTED verdicts were optimistic too.** The sweep prunes a single
//! finished schematic, so it never exercises the ROUTE ORDER — and a crossing
//! amendment is stamped into a bus that was already planned. Four multi-bus
//! problems (`t4_00`, `t4_01`, `t5_02`, `t5_04`) route clean, pass DRC/LVS, and
//! deliver ZERO at 15 cells of pitch: the amendment moves the amended bus's own
//! station one cell later than it budgeted for. They pass at 14. That is
//! [`CROSSING_ALLOWANCE`], and it means a demolition test on one schematic
//! cannot certify a pitch for a design where buses are added incrementally.

/// Dust cells a strongly-powered source lights before the signal dies.
///
/// A repeater's output strongly powers the block in front of it, so the first
/// dust cell downstream reads 15 and the `n`-th reads `16 - n`. The last cell
/// that still reads at least 1 is therefore the 15th. This is the PHYSICS
/// ceiling, not the number the router plans against.
pub const DUST_REACH: u32 = 15;

/// The dust budget the router actually plans against.
///
/// One cell short of [`DUST_REACH`], which is the spacing the adversarial
/// demolition sweep verified empirically (`--pitch 15`, i.e. repeater centres
/// 15 apart, i.e. 14 dust cells between them) across 54 problems and 18-42
/// vectors each. Keeping a cell in hand also means an off-by-one anywhere in
/// the accounting degrades a margin instead of dropping a word.
pub const DUST_BUDGET: u32 = DUST_REACH - 1;

/// Cells a run must leave in hand for a CROSSING IT DOES NOT KNOW ABOUT YET.
///
/// A bus routed later can cross this one, and the crossing stamps a
/// through-bus station (entry block / repeater / exit block) into the already
/// routed bus at the crossing column — see `Design::plan_station_amendment`.
/// The amended bus is NOT re-planned, so if the station it had chosen for
/// itself falls inside that three-cell window, the amendment MOVES its refresh
/// to the window's own repeater — up to one cell later than the run budgeted
/// for. With the old half-reach constant that cost nothing; against a budget
/// spent to the last cell it kills the word outright.
///
/// MEASURED, not guessed: on the adversarial corpus, four multi-bus problems
/// with crossings (`t4_00`, `t4_01`, `t5_02`, `t5_04`) route and pass DRC but
/// deliver ZERO at 15 cells of pitch and pass every vector at 14. One cell,
/// which is exactly the displacement above.
pub const CROSSING_ALLOWANCE: u32 = 1;

/// What is carrying the signal along a segment.
///
/// Only [`Carrier::Dust`] exists in the design surface today; the other two
/// arms are the seam the analog carriers land on, and they are here because
/// getting them wrong is not a small error — see [`Carrier::plans_stations`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Carrier {
    /// Redstone dust on a sturdy support: the only DECAYING carrier, 1 per
    /// cell, no delay, no refresh. (`materials.py` `_m("dust", …, decay=1)`;
    /// `transport::Mechanism::Dust.decay() == 1`.)
    #[default]
    Dust,
    /// A carrier whose every stage re-emits 15 regardless of its input — a
    /// repeater line, a comparator chain. Strength is not a resource here.
    Refreshing,
    /// A carrier that GAINS strength per stage, e.g. the measured hex comb:
    /// `out = min(15, v + (15 - comb_len))`, so a short comb is a free level
    /// shifter (`redstone-eda/notes-hex-transport.md` H6.3).
    Gaining {
        /// Strength gained per stage (`15 - comb_len` for the hex comb).
        per_stage: u8,
    },
}

impl Carrier {
    /// Signal strength lost per cell traversed.
    ///
    /// Mirrors `transport::Mechanism::decay()` and `materials.py`'s `decay`
    /// field. A gaining carrier is NOT modelled as negative decay: see
    /// [`Carrier::plans_stations`].
    pub fn decay_per_cell(self) -> u32 {
        match self {
            Carrier::Dust => 1,
            Carrier::Refreshing | Carrier::Gaining { .. } => 0,
        }
    }

    /// Does a route on this carrier need refresh stations at all?
    ///
    /// **A gaining carrier must suppress station insertion.** The gain is a
    /// CAPABILITY, not a discount: the stage already solves the strength
    /// problem a station would solve, so a planner that inserts one anyway
    /// double-spends the same headroom — it pays a station's footprint and
    /// delay for reach the carrier was going to hand it for free, and on a
    /// gaining stage it also clamps an analog value to 15 and destroys it.
    /// `redstone-eda/BUS_CRATE_DESIGN.md` states the same rule: *a bus on a
    /// gaining carrier plans zero refresh stations.*
    pub fn plans_stations(self) -> bool {
        matches!(self, Carrier::Dust)
    }

    /// Strength gained per stage (0 unless [`Carrier::Gaining`]).
    pub fn gain_per_stage(self) -> u8 {
        match self {
            Carrier::Gaining { per_stage } => per_stage,
            _ => 0,
        }
    }
}

/// A running signal-strength budget for one segment of one bus.
///
/// `spent` is measured in DUST CELLS traversed since the last refresh, which is
/// the same unit `design.rs` has always threaded across joints as `since`, so
/// entry and exit values remain interchangeable with it. The invariant the
/// router maintains is `spent <= budget` at every cell.
#[derive(Clone, Copy, Debug)]
pub struct DecayLedger {
    carrier: Carrier,
    budget: u32,
    spent: u32,
    reserve: u32,
}

impl DecayLedger {
    /// A ledger for a segment the signal enters FRESH (straight out of a
    /// repeater, a driver, or a station).
    pub fn fresh(carrier: Carrier) -> Self {
        Self::entering(carrier, 0)
    }

    /// A ledger for a segment the signal enters having already spent
    /// `spent` dust cells since its last refresh.
    ///
    /// Seeding this honestly is the whole point: a run out of a form adapter's
    /// gather column starts up to 7 cells down, and planning it as if it
    /// started at 15 is exactly the error the old constant was hiding.
    pub fn entering(carrier: Carrier, spent: u32) -> Self {
        DecayLedger {
            carrier,
            budget: DUST_BUDGET,
            spent,
            reserve: 0,
        }
    }

    /// Require `reserve` cells of reach to still be available when the segment
    /// ENDS, because geometry the segment cannot see spends them — a sink's
    /// form adapter, typically.
    pub fn reserving(mut self, reserve: u32) -> Self {
        self.reserve = reserve;
        self
    }

    /// Hold `cells` back from the budget EVERYWHERE along the segment, for
    /// geometry that may be stamped into it after it is planned. See
    /// [`CROSSING_ALLOWANCE`], the only current caller.
    ///
    /// Unlike [`DecayLedger::reserving`] this is not an end-of-segment cost: a
    /// crossing can land anywhere, so the allowance has to be live at every
    /// cell.
    pub fn allowing(mut self, cells: u32) -> Self {
        self.budget = self.budget.saturating_sub(cells);
        self
    }

    /// The carrier this ledger is tracking.
    pub fn carrier(&self) -> Carrier {
        self.carrier
    }

    /// Dust cells spent since the last refresh.
    pub fn spent(&self) -> u32 {
        self.spent
    }

    /// The exit headroom this ledger is holding back.
    pub fn reserve(&self) -> u32 {
        self.reserve
    }

    /// Debit one carried cell.
    pub fn carry_cell(&mut self) {
        self.spent = self.spent.saturating_add(self.carrier.decay_per_cell());
    }

    /// A refresh: the signal leaves at full strength again.
    pub fn refresh(&mut self) {
        self.spent = 0;
    }

    /// Can the carrier pay for ONE more cell here, plus `mandatory_tail` cells
    /// after it that cannot host a station (a crossing window's approach, the
    /// two cells at each end of a run, a kept junction — and the exit reserve
    /// when the segment simply ends)?
    ///
    /// A non-decaying carrier always can; that is what makes
    /// [`DecayLedger::needs_station`] answer `false` for it.
    pub fn admits_cell(&self, mandatory_tail: u32) -> bool {
        let d = self.carrier.decay_per_cell();
        if d == 0 {
            return true;
        }
        self.spent + d * (1 + mandatory_tail) <= self.budget
    }

    /// Must a refresh station go HERE — i.e. would carrying this cell strand
    /// the signal before the next place a station could legally stand?
    pub fn needs_station(&self, mandatory_tail: u32) -> bool {
        self.carrier.plans_stations() && !self.admits_cell(mandatory_tail)
    }
}

/// The dust DEBT a form adapter (or any owned tile) imposes at `anchor`: how
/// many of its own dust cells lie between `anchor` and the nearest thing that
/// refreshes, measured from the stamped geometry rather than re-derived from
/// the code that stamped it.
///
/// `anchor` counts as the first cell, so a column cell one dust step from a
/// gather repeater has a debt of 1. Used two ways:
///
/// * on the DRIVER side, to seed the trunk's ledger — the trunk's first dust
///   cell is really the `debt + 1`-th cell since the adapter's last repeater;
/// * on the SINK side, as the trunk's exit reserve — the trunk's last dust cell
///   is followed by `debt` more adapter cells before anything refreshes.
///
/// The flood is over dust cells only and stops at anything else, so a repeater
/// (the adapter's mouth diode, its gather diodes, its own refresh stations)
/// bounds it. Neighbours include the vertical diagonals, because dust conducts
/// up and down a staircase and an adapter's lane climbs one.
pub fn tile_dust_debt<F>(anchor: (i32, i32, i32), is_dust: F) -> u32
where
    F: Fn((i32, i32, i32)) -> bool,
{
    if !is_dust(anchor) {
        return 0;
    }
    const STEPS: [(i32, i32, i32); 14] = [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
        // Dust conducts diagonally up and down a staircase.
        (1, 1, 0),
        (-1, 1, 0),
        (1, -1, 0),
        (-1, -1, 0),
        (0, 1, 1),
        (0, 1, -1),
        (0, -1, 1),
        (0, -1, -1),
    ];
    let mut seen = std::collections::BTreeSet::new();
    let mut frontier = vec![anchor];
    seen.insert(anchor);
    let mut depth = 0u32;
    // Breadth-first, so `depth` is the number of dust cells on the longest
    // chain reachable without passing through a refresher.
    while !frontier.is_empty() {
        depth += 1;
        let mut next = Vec::new();
        for p in frontier {
            for s in STEPS {
                let q = (p.0 + s.0, p.1 + s.1, p.2 + s.2);
                if seen.contains(&q) || !is_dust(q) {
                    continue;
                }
                seen.insert(q);
                next.push(q);
            }
        }
        frontier = next;
    }
    depth
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dust_budget_is_one_short_of_the_physics() {
        assert_eq!(DUST_REACH, 15);
        assert_eq!(DUST_BUDGET, 14);
    }

    #[test]
    fn a_fresh_dust_ledger_carries_the_whole_budget() {
        let mut l = DecayLedger::fresh(Carrier::Dust);
        let mut cells = 0;
        while !l.needs_station(0) {
            l.carry_cell();
            cells += 1;
            assert!(cells <= 100, "runaway");
        }
        assert_eq!(cells, DUST_BUDGET, "14 dust cells, then a station");
        assert_eq!(l.spent(), DUST_BUDGET);
    }

    #[test]
    fn an_entry_debt_shortens_the_first_stretch() {
        // Out of a form adapter 7 cells down, only 7 cells remain.
        let mut l = DecayLedger::entering(Carrier::Dust, 7);
        let mut cells = 0;
        while !l.needs_station(0) {
            l.carry_cell();
            cells += 1;
        }
        assert_eq!(cells, DUST_BUDGET - 7);
    }

    #[test]
    fn a_reserve_shortens_the_last_stretch_by_exactly_itself() {
        let mut l = DecayLedger::fresh(Carrier::Dust).reserving(7);
        let mut cells = 0;
        // `mandatory_tail` = the reserve once the run has nowhere left to put a
        // station; the caller folds it in, so exercise it the same way.
        while !l.needs_station(7) {
            l.carry_cell();
            cells += 1;
        }
        assert_eq!(cells, DUST_BUDGET - 7);
        assert_eq!(l.reserve(), 7);
    }

    #[test]
    fn a_refresh_returns_the_whole_budget() {
        let mut l = DecayLedger::entering(Carrier::Dust, 13);
        assert!(l.admits_cell(0));
        l.carry_cell();
        assert!(l.needs_station(0));
        l.refresh();
        assert_eq!(l.spent(), 0);
        assert!(l.admits_cell(0));
    }

    #[test]
    fn a_mandatory_tail_forces_the_station_earlier() {
        // 10 spent, 4 cells that cannot host a station after this one: this
        // cell would leave 15 spent at the last of them, one over budget.
        let l = DecayLedger::entering(Carrier::Dust, 10);
        assert!(l.needs_station(4));
        assert!(!l.needs_station(3));
    }

    #[test]
    fn a_crossing_allowance_holds_back_the_budget_everywhere() {
        let mut l = DecayLedger::fresh(Carrier::Dust).allowing(CROSSING_ALLOWANCE);
        let mut cells = 0;
        while !l.needs_station(0) {
            l.carry_cell();
            cells += 1;
            assert!(cells <= 100, "runaway");
        }
        assert_eq!(cells, DUST_BUDGET - CROSSING_ALLOWANCE);
        // And it composes with an entry debt rather than replacing it.
        let l = DecayLedger::entering(Carrier::Dust, DUST_BUDGET - CROSSING_ALLOWANCE)
            .allowing(CROSSING_ALLOWANCE);
        assert!(l.needs_station(0));
    }

    #[test]
    fn a_refreshing_carrier_never_plans_a_station() {
        let mut l = DecayLedger::fresh(Carrier::Refreshing);
        for _ in 0..1000 {
            assert!(!l.needs_station(50));
            l.carry_cell();
        }
        assert_eq!(l.spent(), 0, "a refreshing carrier spends nothing");
    }

    #[test]
    fn a_gaining_carrier_never_plans_a_station() {
        // The hex comb: out = min(15, v + (15 - comb_len)). A short comb is a
        // free level shifter, so a station here double-spends the headroom.
        let c = Carrier::Gaining { per_stage: 4 };
        assert_eq!(c.gain_per_stage(), 4);
        assert_eq!(c.decay_per_cell(), 0);
        assert!(!c.plans_stations());
        let mut l = DecayLedger::entering(c, 12).reserving(3);
        for _ in 0..1000 {
            assert!(!l.needs_station(50));
            l.carry_cell();
        }
    }

    #[test]
    fn debt_counts_dust_cells_up_to_a_refresher() {
        // A straight lane of 7 dust cells with a repeater at one end.
        let dust: std::collections::BTreeSet<(i32, i32, i32)> =
            (2..=8).map(|x| (x, 0, 0)).collect();
        assert_eq!(tile_dust_debt((8, 0, 0), |p| dust.contains(&p)), 7);
        assert_eq!(tile_dust_debt((5, 0, 0), |p| dust.contains(&p)), 4);
        // Nothing there at all: nothing owed.
        assert_eq!(tile_dust_debt((99, 0, 0), |p| dust.contains(&p)), 0);
    }

    #[test]
    fn debt_climbs_a_staircase() {
        // A diagonal staircase: dust conducts up it, so the debt must too.
        let dust: std::collections::BTreeSet<(i32, i32, i32)> =
            (0..5).map(|i| (i, i, 0)).collect();
        assert_eq!(tile_dust_debt((4, 4, 0), |p| dust.contains(&p)), 5);
    }
}
