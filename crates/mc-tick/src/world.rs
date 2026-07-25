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

/// The simulated region's blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    bounds: Bounds,
    states: Vec<StateId>,
}

impl World {
    /// An all-air world covering `bounds`.
    pub fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            states: vec![StateId::AIR; bounds.volume() as usize],
        }
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
    /// Returns `None` and changes nothing if `pos` is outside the region —
    /// silently dropping the write, deliberately, for the same reason [`get`]
    /// reads air: callers push blocks around and would otherwise all need edge
    /// handling.
    ///
    /// [`get`]: World::get
    pub fn set(&mut self, pos: Pos, state: StateId) -> Option<StateId> {
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
    fn out_of_bounds_reads_air_and_drops_writes() {
        // Documented divergence: neighbours outside the region read as air so the
        // tick loop needs no edge branches. Pinned here so it stays a decision.
        let mut world = small();
        let outside = Pos::new(99, 0, 0);
        assert_eq!(world.get(outside), StateId::AIR);
        assert_eq!(world.set(outside, StateId(7)), None);
        assert_eq!(world.non_air_count(), 0);
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
