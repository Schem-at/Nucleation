//! Block storage for the simulated region.
//!
//! Dense `Vec<StateId>` over a bounded region: reads and writes are index
//! arithmetic, and a snapshot is a `clone`. At the scale this runs at — a piston
//! door is a few thousand blocks, a 128³ region is 4 MB — copying the whole
//! buffer to checkpoint is cheaper in both time and complexity than any
//! structural-sharing scheme. Reach for something cleverer only when a
//! measurement demands it.

use crate::pos::{Bounds, Dir, Pos};
use crate::state::StateId;

/// How far the region will enlarge itself before refusing, in cells.
///
/// A flying machine travels until something stops it, and nothing here does.
/// The cap is what keeps "runs off to the horizon" from becoming "allocates
/// until the process dies"; past it, writes are dropped exactly as they were
/// before the region could grow at all. 32 M cells is a 320x320x320 room —
/// far more than any contraption needs, and a few hundred MB at worst.
pub const DEFAULT_GROWTH_LIMIT: u64 = 32_000_000;

/// How much the region overshoots a write that lands outside it.
///
/// Growing to exactly the offending block would reallocate on every step of a
/// machine that moves one block at a time. A chunk's worth of slack amortises
/// that to one reallocation per 16 blocks travelled.
const GROWTH_STEP: i32 = 16;

/// The simulated region's blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    bounds: Bounds,
    states: Vec<StateId>,
    /// Ceiling on [`World::grow_to_include`]; see [`DEFAULT_GROWTH_LIMIT`].
    growth_limit: u64,
}

impl World {
    /// An all-air world covering `bounds`.
    pub fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            states: vec![StateId::AIR; bounds.volume() as usize],
            growth_limit: DEFAULT_GROWTH_LIMIT,
        }
    }

    /// Cap how far this world will enlarge itself, in cells.
    ///
    /// Zero pins the region to its current extent, restoring the original
    /// behaviour of dropping every write that lands outside. Useful when the
    /// region is meant to be the whole story — a conformance run whose capture
    /// has a world of its own, or a search that would rather a machine died at
    /// a known wall than wandered off and kept allocating.
    pub fn set_growth_limit(&mut self, cells: u64) {
        self.growth_limit = cells;
    }

    /// Enlarge the region so `pos` falls inside it.
    ///
    /// Returns whether it now does: growth stops at [`World::set_growth_limit`],
    /// and a refusal leaves the region exactly as it was.
    pub fn grow_to_include(&mut self, pos: Pos) -> bool {
        if self.bounds.contains(pos) {
            return true;
        }
        // Round outwards to a whole step, so travel costs one reallocation per
        // `GROWTH_STEP` blocks rather than one per block. Saturating, so a
        // coordinate near the end of the range cannot wrap the overshoot round
        // into a small number.
        let floor_step = |v: i32| v.div_euclid(GROWTH_STEP) * GROWTH_STEP;
        let lo = |current: i32, p: i32| current.min(floor_step(p));
        let hi = |current: i32, p: i32| current.max(floor_step(p).saturating_add(GROWTH_STEP - 1));
        let (min, max) = (
            Pos::new(
                lo(self.bounds.min.x, pos.x),
                lo(self.bounds.min.y, pos.y),
                lo(self.bounds.min.z, pos.z),
            ),
            Pos::new(
                hi(self.bounds.max.x, pos.x),
                hi(self.bounds.max.y, pos.y),
                hi(self.bounds.max.z, pos.z),
            ),
        );
        // Measured before the region is built, and in `u128`: `Bounds::size`
        // subtracts in `i32` and `volume` multiplies into a `u64`, so a write
        // far enough away overflows *both* — and a wrapped volume would look
        // small, slip past the cap, and allocate whatever the garbage said.
        let span = |a: i32, b: i32| u128::from((i64::from(b) - i64::from(a) + 1) as u64);
        if span(min.x, max.x) * span(min.y, max.y) * span(min.z, max.z)
            > u128::from(self.growth_limit)
        {
            return false;
        }
        let grown = Bounds::new(min, max);
        let mut states = vec![StateId::AIR; grown.volume() as usize];
        // Only non-air needs carrying over, and skipping air makes the copy
        // proportional to what the build actually contains rather than to the
        // volume it rattles around in.
        let old = self.bounds;
        for (index, state) in self.states.iter().enumerate() {
            if *state == StateId::AIR {
                continue;
            }
            if let Some(index) = old.position_of(index).and_then(|p| grown.index(p)) {
                states[index] = *state;
            }
        }
        self.bounds = grown;
        self.states = states;
        true
    }

    /// The region this world covers.
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }

    /// The state at `pos`.
    ///
    /// # Out of bounds
    ///
    /// Returns [`StateId::AIR`] outside the region rather than panicking or
    /// returning an error, because the tick loop reads neighbours constantly and
    /// every edge block would otherwise need a branch at each call site.
    ///
    /// This is a real divergence from the game, where a schematic's neighbour is
    /// whatever the world contains. A door flush against the region edge can
    /// therefore simulate differently than it would in-game. The fix is to load
    /// structures with a margin of padding, not to complicate this call — and it
    /// is the trace differ's job to catch it if we forget.
    pub fn get(&self, pos: Pos) -> StateId {
        match self.bounds.index(pos) {
            Some(index) => self.states[index],
            None => StateId::AIR,
        }
    }

    /// Set the state at `pos`, returning the previous one.
    ///
    /// # Outside the region
    ///
    /// A block written outside the region **grows the region to fit it**. The
    /// game's world does not end where a schematic does, and pretending it
    /// does is not a neutral simplification: a flying machine that reached the
    /// edge used to have whichever of its blocks crossed first deleted, and
    /// what was left — a piston here, a slime block two cells away — was
    /// wreckage that could not fly, sitting in a world that reported no
    /// further changes. Silent, and indistinguishable from a redstone bug.
    ///
    /// Growth is capped ([`World::set_growth_limit`]); past the cap the write
    /// is dropped as it always was. Writing **air** outside never grows the
    /// region — outside is already air, and a machine clears the cells behind
    /// itself every step, which would otherwise enlarge the world in the
    /// direction it is travelling away from.
    ///
    /// Reads still answer air outside the region rather than growing it; see
    /// [`get`].
    ///
    /// [`get`]: World::get
    pub fn set(&mut self, pos: Pos, state: StateId) -> Option<StateId> {
        if !self.bounds.contains(pos) {
            if state == StateId::AIR || !self.grow_to_include(pos) {
                return None;
            }
        }
        let index = self.bounds.index(pos)?;
        let previous = self.states[index];
        self.states[index] = state;
        Some(previous)
    }

    /// The state of the neighbour one block `dir` from `pos`.
    pub fn neighbor(&self, pos: Pos, dir: Dir) -> StateId {
        self.get(pos.offset(dir))
    }

    /// Whether `pos` is inside the simulated region.
    pub fn contains(&self, pos: Pos) -> bool {
        self.bounds.contains(pos)
    }

    /// Every position holding a state other than air, in storage order.
    ///
    /// Storage order is deterministic, which matters: anything derived from this
    /// feeds traces, and traces are compared byte for byte.
    pub fn iter_non_air(&self) -> impl Iterator<Item = (Pos, StateId)> + '_ {
        self.states
            .iter()
            .enumerate()
            .filter(|(_, state)| **state != StateId::AIR)
            .filter_map(|(index, state)| {
                self.bounds.position_of(index).map(|pos| (pos, *state))
            })
    }

    /// How many blocks are not air.
    pub fn non_air_count(&self) -> usize {
        self.states
            .iter()
            .filter(|state| **state != StateId::AIR)
            .count()
    }

    /// The raw backing slice, in storage order.
    pub fn raw_states(&self) -> &[StateId] {
        &self.states
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small() -> World {
        World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(3, 3, 3)))
    }

    #[test]
    fn a_new_world_is_entirely_air() {
        let world = small();
        assert_eq!(world.non_air_count(), 0);
        assert_eq!(world.get(Pos::new(1, 1, 1)), StateId::AIR);
        assert_eq!(world.raw_states().len(), 64);
    }

    #[test]
    fn set_returns_the_previous_state() {
        let mut world = small();
        let stone = StateId(7);
        assert_eq!(world.set(Pos::new(1, 1, 1), stone), Some(StateId::AIR));
        assert_eq!(world.get(Pos::new(1, 1, 1)), stone);
        assert_eq!(world.set(Pos::new(1, 1, 1), StateId(8)), Some(stone));
    }

    #[test]
    fn out_of_bounds_reads_air() {
        // Documented divergence: neighbours outside the region read as air so the
        // tick loop needs no edge branches. Pinned here so it stays a decision.
        // Reading does *not* grow the region — only writing does.
        let world = small();
        assert_eq!(world.get(Pos::new(99, 0, 0)), StateId::AIR);
        assert_eq!(world.bounds(), small().bounds());
    }

    #[test]
    fn a_block_written_outside_grows_the_region_to_hold_it() {
        let mut world = small();
        let outside = Pos::new(99, 0, 0);
        assert_eq!(world.set(outside, StateId(7)), Some(StateId::AIR));
        assert_eq!(world.get(outside), StateId(7));
        assert_eq!(world.non_air_count(), 1);
        assert!(world.bounds().contains(outside));
    }

    #[test]
    fn growing_keeps_everything_already_placed() {
        let mut world = small();
        let (a, b) = (Pos::new(1, 1, 1), Pos::new(3, 2, 0));
        world.set(a, StateId(7));
        world.set(b, StateId(9));
        world.set(Pos::new(-40, 0, 0), StateId(5));
        assert_eq!(world.get(a), StateId(7));
        assert_eq!(world.get(b), StateId(9));
        assert_eq!(world.non_air_count(), 3);
    }

    #[test]
    fn air_written_outside_does_not_grow_the_region() {
        // A machine clears the cells behind itself every step. Growing for
        // those would enlarge the world in the direction it is leaving.
        let mut world = small();
        let before = world.bounds();
        assert_eq!(world.set(Pos::new(99, 0, 0), StateId::AIR), None);
        assert_eq!(world.bounds(), before);
    }

    #[test]
    fn growth_stops_at_the_limit_and_changes_nothing_when_it_refuses() {
        let mut world = small();
        world.set_growth_limit(0);
        let before = world.bounds();
        assert_eq!(world.set(Pos::new(99, 0, 0), StateId(7)), None);
        assert_eq!(world.bounds(), before);
        assert_eq!(world.non_air_count(), 0);
    }

    #[test]
    fn a_wild_coordinate_is_refused_rather_than_overflowing_the_extent() {
        // `Bounds::size` subtracts in i32 and `volume` multiplies into u64;
        // both overflow long before this coordinate, so the cap has to be
        // decided without going through either.
        let mut world = small();
        for far in [i32::MAX, i32::MIN, 2_000_000_000, -2_000_000_000] {
            assert_eq!(world.set(Pos::new(far, far, far), StateId(7)), None);
            assert_eq!(world.bounds(), small().bounds());
            assert_eq!(world.non_air_count(), 0);
        }
    }

    #[test]
    fn growth_overshoots_so_travel_does_not_reallocate_every_block() {
        let mut world = small();
        world.set(Pos::new(4, 0, 0), StateId(7));
        let grown = world.bounds();
        // The next few blocks along are already covered by the overshoot.
        world.set(Pos::new(5, 0, 0), StateId(7));
        assert_eq!(world.bounds(), grown);
    }

    #[test]
    fn neighbor_reads_follow_the_direction_deltas() {
        let mut world = small();
        let marker = StateId(3);
        world.set(Pos::new(1, 2, 1), marker);
        assert_eq!(world.neighbor(Pos::new(1, 1, 1), Dir::Up), marker);
        assert_eq!(world.neighbor(Pos::new(1, 3, 1), Dir::Down), marker);
    }

    #[test]
    fn iter_non_air_is_deterministic_and_complete() {
        let mut world = small();
        world.set(Pos::new(3, 0, 0), StateId(1));
        world.set(Pos::new(0, 0, 0), StateId(2));
        world.set(Pos::new(0, 2, 1), StateId(3));

        let first: Vec<_> = world.iter_non_air().collect();
        let second: Vec<_> = world.iter_non_air().collect();
        assert_eq!(first, second, "iteration order must be stable for traces");
        assert_eq!(first.len(), 3);
        // Storage order is y-major, so (0,0,0) precedes (3,0,0) precedes (0,2,1).
        assert_eq!(first[0].0, Pos::new(0, 0, 0));
        assert_eq!(first[1].0, Pos::new(3, 0, 0));
        assert_eq!(first[2].0, Pos::new(0, 2, 1));
    }

    #[test]
    fn cloning_gives_an_independent_world() {
        // Checkpoints are clones, so this property is the checkpoint guarantee.
        let mut world = small();
        world.set(Pos::new(1, 1, 1), StateId(5));
        let snapshot = world.clone();
        world.set(Pos::new(1, 1, 1), StateId(9));
        assert_eq!(snapshot.get(Pos::new(1, 1, 1)), StateId(5));
        assert_eq!(world.get(Pos::new(1, 1, 1)), StateId(9));
    }
}
