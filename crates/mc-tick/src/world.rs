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
#[derive(Clone, PartialEq, Eq)]
pub struct World {
    bounds: Bounds,
    states: Vec<StateId>,
    size: [u32; 3],
    layer_stride: usize,
    /// Ceiling on [`World::grow_to_include`]; see [`DEFAULT_GROWTH_LIMIT`].
    growth_limit: u64,
    #[cfg(feature = "experiment-wire-plan")]
    plan_journal: crate::world_journal::MutationJournal,
    #[cfg(feature = "experiment-world-plan-cache")]
    plan_cache: Box<crate::wire_plan::Cache>,
}

// Optimization bookkeeping is deliberately absent from semantic/debug snapshots.
impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World").field("bounds", &self.bounds).field("states", &self.states)
            .field("size", &self.size).field("layer_stride", &self.layer_stride)
            .field("growth_limit", &self.growth_limit).finish()
    }
}

#[cfg(feature = "experiment-cursor")]
pub(crate) struct LocalReadCursor<'a> {
    states: &'a [StateId],
    base: usize,
    row: isize,
    layer: isize,
}

#[cfg(feature = "experiment-cursor")]
impl LocalReadCursor<'_> {
    /// Offsets are restricted to the radius-two cube proven interior at creation.
    #[inline]
    pub(crate) fn get(&self, offset: [i32; 3]) -> StateId {
        assert!(offset.into_iter().all(|v| (-2..=2).contains(&v)));
        let delta = offset[0] as isize + offset[1] as isize * self.layer + offset[2] as isize * self.row;
        self.states[self.base.wrapping_add_signed(delta)]
    }
}

impl World {
    /// Per-world scratch storage, isolated even when multiple simulations share
    /// the same immutable rules. The cache also checks an owned rules identity.
    #[cfg(feature = "experiment-world-plan-cache")]
    pub(crate) fn wire_plan_cache(&self) -> &crate::wire_plan::Cache { &self.plan_cache }

    /// An all-air world covering `bounds`.
    pub fn new(bounds: Bounds) -> Self {
        let (x, y, z) = bounds.size();
        let size = [x, y, z];
        Self {
            size,
            layer_stride: size[0] as usize * size[2] as usize,
            bounds,
            states: vec![StateId::AIR; bounds.volume() as usize],
            growth_limit: DEFAULT_GROWTH_LIMIT,
            #[cfg(feature = "experiment-wire-plan")]
            plan_journal: Default::default(),
            #[cfg(feature = "experiment-world-plan-cache")]
            plan_cache: Default::default(),
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
        let (x, y, z) = grown.size();
        self.size = [x, y, z];
        self.layer_stride = self.size[0] as usize * self.size[2] as usize;
        self.bounds = grown;
        self.states = states;
        #[cfg(feature = "experiment-wire-plan")]
        self.plan_journal.reset();
        true
    }

    #[inline]
    fn index(&self, pos: Pos) -> Option<usize> {
        let x = pos.x.wrapping_sub(self.bounds.min.x) as u32;
        let y = pos.y.wrapping_sub(self.bounds.min.y) as u32;
        let z = pos.z.wrapping_sub(self.bounds.min.z) as u32;
        if x >= self.size[0] || y >= self.size[1] || z >= self.size[2] {
            return None;
        }
        Some(y as usize * self.layer_stride + z as usize * self.size[0] as usize + x as usize)
    }

    #[cfg(feature = "experiment-wire-plan")]
    pub(crate) fn plan_identity(&self) -> &std::sync::Arc<()> { self.plan_journal.identity() }

    #[cfg(feature = "experiment-wire-plan")]
    pub(crate) fn plan_revision(&self) -> u64 { self.plan_journal.revision() }

    #[cfg(feature = "experiment-wire-plan")]
    pub(crate) fn plan_changes_since(&self, revision: u64)
        -> Option<impl Iterator<Item = (Pos, StateId, StateId)> + '_>
    { self.plan_journal.changes_since(revision) }

    /// A borrow-scoped address calculation; no derived power or state is cached.
    #[cfg(feature = "experiment-cursor")]
    #[inline]
    pub(crate) fn local_reader(&self, pos: Pos) -> Option<LocalReadCursor<'_>> {
        let x = pos.x.wrapping_sub(self.bounds.min.x) as u32;
        let y = pos.y.wrapping_sub(self.bounds.min.y) as u32;
        let z = pos.z.wrapping_sub(self.bounds.min.z) as u32;
        if x.wrapping_sub(2) < self.size[0].saturating_sub(4)
            && y.wrapping_sub(2) < self.size[1].saturating_sub(4)
            && z.wrapping_sub(2) < self.size[2].saturating_sub(4)
        {
            Some(LocalReadCursor { states: &self.states,
                base: y as usize * self.layer_stride + z as usize * self.size[0] as usize + x as usize,
                row: self.size[0] as isize, layer: self.layer_stride as isize })
        } else { None }
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
        match self.index(pos) {
            Some(index) => self.states[index],
            None => StateId::AIR,
        }
    }

    /// Six current states in `Dir` order. Interior reads share the coordinate
    /// conversion and boundary check; border reads keep ordinary air semantics.
    #[inline]
    pub(crate) fn neighbours(&self, pos: Pos) -> [StateId; 6] {
        let x = pos.x.wrapping_sub(self.bounds.min.x) as u32;
        let y = pos.y.wrapping_sub(self.bounds.min.y) as u32;
        let z = pos.z.wrapping_sub(self.bounds.min.z) as u32;
        if x.wrapping_sub(1) < self.size[0].saturating_sub(2)
            && y.wrapping_sub(1) < self.size[1].saturating_sub(2)
            && z.wrapping_sub(1) < self.size[2].saturating_sub(2)
        {
            let row = self.size[0] as usize;
            let i = y as usize * self.layer_stride + z as usize * row + x as usize;
            [self.states[i - self.layer_stride], self.states[i + self.layer_stride],
             self.states[i - row], self.states[i + row], self.states[i - 1], self.states[i + 1]]
        } else {
            crate::pos::ALL_DIRS.map(|dir| {
                let (x, y, z) = dir.delta();
                self.get(Pos::new(pos.x.wrapping_add(x), pos.y.wrapping_add(y), pos.z.wrapping_add(z)))
            })
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
        let index = self.index(pos)?;
        let previous = self.states[index];
        self.states[index] = state;
        #[cfg(feature = "experiment-wire-plan")]
        if previous != state { self.plan_journal.record(pos, previous, state); }
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
            .filter_map(|(index, state)| self.bounds.position_of(index).map(|pos| (pos, *state)))
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

    #[cfg(feature = "experiment-cursor")]
    #[test]
    fn local_cursor_matches_reads_for_every_offset_and_growth() {
        for origin in [Pos::new(-20, -30, -40), Pos::new(i32::MIN, 0, 0), Pos::new(i32::MAX - 8, 0, 0)] {
            let mut world = World::new(Bounds::new(origin, Pos::new(origin.x + 8, origin.y + 8, origin.z + 8)));
            for x in 0..9 { for y in 0..9 { for z in 0..9 {
                world.set(Pos::new(origin.x + x, origin.y + y, origin.z + z), StateId((1 + x + y * 9 + z * 81) as u16));
            } } }
            for grown in [false, true] {
                if grown { assert!(world.grow_to_include(Pos::new(origin.x, origin.y + 10, origin.z))); }
                for x in 0..9 { for y in 0..9 { for z in 0..9 {
                    let pos = Pos::new(origin.x + x, origin.y + y, origin.z + z);
                    if let Some(cursor) = world.local_reader(pos) {
                        for dx in -2..=2 { for dy in -2..=2 { for dz in -2..=2 {
                            assert_eq!(cursor.get([dx, dy, dz]), world.get(Pos::new(pos.x + dx, pos.y + dy, pos.z + dz)));
                        } } }
                    } else if !grown { assert!(x < 2 || y < 2 || z < 2 || x > 6 || y > 6 || z > 6); }
                } } }
            }
        }
        let world = World::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(3, 3, 3)));
        assert!(world.local_reader(Pos::new(2, 2, 2)).is_none());
    }

    #[cfg(feature = "experiment-wire-plan")]
    #[test]
    fn plan_journal_tracks_silent_writes_and_resets_on_clone_and_growth() {
        let mut world = small();
        let token = std::sync::Arc::clone(world.plan_identity());
        let pos = Pos::new(1, 1, 1);
        world.set(pos, StateId(7));
        world.set(pos, StateId(7));
        assert_eq!(world.plan_revision(), 1);
        assert_eq!(world.plan_changes_since(0).unwrap().collect::<Vec<_>>(), vec![(pos, StateId::AIR, StateId(7))]);
        let cloned = world.clone();
        assert_eq!(world, cloned);
        assert_eq!(format!("{world:?}"), format!("{cloned:?}"));
        assert!(!std::sync::Arc::ptr_eq(cloned.plan_identity(), &token));
        world.grow_to_include(Pos::new(-10, 1, 1));
        assert!(!std::sync::Arc::ptr_eq(world.plan_identity(), &token));
        assert_eq!(world.plan_revision(), 0);
    }

    #[test]
    fn neighbour_batch_matches_individual_reads_at_borders_and_after_growth() {
        for origin in [Pos::new(-7, -9, -11), Pos::new(i32::MIN, 0, 0),
            Pos::new(i32::MAX - 4, 0, 0)] {
            for width in 1..=5 {
                let mut world = World::new(Bounds::new(origin,
                    Pos::new(origin.x + (width - 1), origin.y + 4, origin.z + 4)));
                for x in 0..width { for y in 0..5 { for z in 0..5 {
                    world.set(Pos::new(origin.x + x, origin.y + y, origin.z + z),
                        StateId((1 + x + 5 * z + 25 * y) as u16));
                } } }
                let check = |world: &World| {
                    for x in -1..=width { for y in -1..=5 { for z in -1..=5 {
                        let pos = Pos::new(origin.x.wrapping_add(x), origin.y + y, origin.z + z);
                        let expected = crate::pos::ALL_DIRS.map(|dir| {
                            let (dx, dy, dz) = dir.delta();
                            world.get(Pos::new(pos.x.wrapping_add(dx), pos.y.wrapping_add(dy), pos.z.wrapping_add(dz)))
                        });
                        assert_eq!(world.neighbours(pos), expected, "at {pos:?}");
                    } } }
                };
                check(&world);
                assert!(world.grow_to_include(Pos::new(origin.x, origin.y + 6, origin.z)));
                check(&world);
            }
        }
    }

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

#[cfg(test)]
mod layout_tests {
    use super::*;
    #[test]
    fn cached_layout_matches_bounds_at_negative_and_extreme_coordinates() {
        for start in [-20, i32::MIN, i32::MAX - 8] {
            let bounds = Bounds::new(Pos::new(start, start, start), Pos::new(start+3, start+4, start+5));
            let world = World::new(bounds);
            for x in [i32::MIN, -21, -20, -18, 0, i32::MAX, start, start+3, start+4] {
                for y in [start, start+4, start+5, i32::MIN, i32::MAX] {
                    for z in [start, start+5, start+6, i32::MIN, i32::MAX] {
                        let p = Pos::new(x,y,z);
                        assert_eq!(world.index(p), bounds.index(p), "{p:?}");
                    }
                }
            }
        }
    }
}
