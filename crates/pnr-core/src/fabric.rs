//! The `Fabric` trait: the seam between generic P&R algorithms and a
//! concrete technology (redstone, or a synthetic test grid).
//!
//! The search state is `(position, fabric memory)`. The memory generalizes
//! the `(stair_count, prev_stair_dir)` pattern the Python router converged
//! on: any rule that constrains a move based on *how the path arrived* lives
//! in the fabric's `Memory` type, so the A* core never learns about stairs,
//! switchbacks or vias.

use crate::grid::Pos;
use core::fmt::Debug;
use core::hash::Hash;

/// A search state: where the head of the route is, plus whatever the fabric
/// needs to remember about the recent path (stair chains, previous stair
/// direction, ...).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct State<M> {
    /// Head position.
    pub pos: Pos,
    /// Fabric-defined path memory.
    pub mem: M,
}

/// One candidate transition out of a state.
#[derive(Clone, Debug)]
pub struct Candidate<M, T> {
    /// The state this move leads to.
    pub to: State<M>,
    /// Base cost of the move (the fabric may refine it in [`Fabric::cost`]).
    pub base_cost: u32,
    /// Fabric move tag, recorded in the emitted path (e.g. horizontal /
    /// stair-up / via-climb + parameters).
    pub tag: T,
    /// Every cell this move newly occupies, including the destination.
    /// Congestion negotiation accounts usage over this footprint, so a via
    /// that occupies a whole column contests all of it.
    pub footprint: Vec<Pos>,
}

/// Signal-budget summary the router core needs from a fabric.
///
/// The full budget semantics (repeater insertion, decay arithmetic) belong to
/// the fabric's emitter; the core only needs the refresh interval to reason
/// about run lengths.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Budget {
    /// Maximum straight run length before the signal must be refreshed.
    pub refresh_every: u32,
    /// Maximum consecutive "unrefreshable" moves (stairs cannot host
    /// repeaters; the Python prototype capped chains at 4).
    pub max_unrefreshable_chain: u32,
}

impl Default for Budget {
    fn default() -> Self {
        Budget {
            refresh_every: 5,
            max_unrefreshable_chain: 4,
        }
    }
}

/// Routing context handed to the fabric on every query: which net is being
/// routed (an index into the caller's net table — the fabric maps it to
/// labels / friendly sets itself).
#[derive(Copy, Clone, Debug)]
pub struct RouteCtx {
    /// Index of the net being routed, assigned by the caller.
    pub net: usize,
}

/// The technology seam. Implementations must be deterministic: `moves` must
/// enumerate candidates in a stable order for identical inputs.
pub trait Fabric {
    /// Path memory carried in the search state.
    type Memory: Clone + Eq + Hash + Ord + Debug;
    /// Move tag recorded per path step.
    type Tag: Clone + Debug;

    /// Memory of a route's starting state.
    fn start_memory(&self) -> Self::Memory;

    /// Enumerate candidate moves out of `from`. Candidates need not be
    /// legal; the core filters through [`Fabric::legal`].
    fn moves(
        &self,
        from: &State<Self::Memory>,
        ctx: &RouteCtx,
    ) -> Vec<Candidate<Self::Memory, Self::Tag>>;

    /// Whether the candidate is legal for this net in the current fabric
    /// state (design rules, occupancy, bounds, clearances).
    fn legal(
        &self,
        from: &State<Self::Memory>,
        cand: &Candidate<Self::Memory, Self::Tag>,
        ctx: &RouteCtx,
    ) -> bool;

    /// Final cost of a candidate. Defaults to the candidate's base cost.
    fn cost(&self, cand: &Candidate<Self::Memory, Self::Tag>, _ctx: &RouteCtx) -> u32 {
        cand.base_cost
    }

    /// The signal budget this fabric routes under.
    fn budget(&self) -> Budget;
}
