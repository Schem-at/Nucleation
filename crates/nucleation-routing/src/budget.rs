//! The signal budget: decay, refresh, and what may host a repeater.
//!
//! First-class per the design doc: decay is tracked per path cell, repeater
//! insertion is the router's job, stairs can't host repeaters, and the
//! refresh interval comes from here — checked invariants of the emitted
//! route, not conventions.

/// Signal-strength budget for routed nets.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SignalBudget {
    /// Full signal strength (a fresh source or a via cap emits this).
    pub full: u8,
    /// Insert a repeater after this many straight refreshless cells.
    /// The prototype refreshed every 5.
    pub refresh: u32,
    /// Maximum consecutive stairs. Stairs cannot host repeaters, so long
    /// runs decay to nothing — the 15-cell staircase that decayed to 0 is
    /// the provenance. Long verticals must use ladder vias, whose cap
    /// emits a fresh 15.
    pub stair_cap: u8,
}

impl Default for SignalBudget {
    fn default() -> Self {
        SignalBudget {
            full: 15,
            refresh: 5,
            stair_cap: 4,
        }
    }
}

impl SignalBudget {
    /// The `pnr-core` view of this budget.
    pub fn core(&self) -> pnr_core::Budget {
        pnr_core::Budget {
            refresh_every: self.refresh,
            max_unrefreshable_chain: self.stair_cap as u32,
        }
    }
}
