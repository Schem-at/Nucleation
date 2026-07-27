//! Water, from `FlowingFluid` + `WaterFluid` bytecode.
//!
//! # The vanilla model
//!
//! A water cell is a `FluidState`: source (amount 8), flowing (amount 1-7) or
//! falling (amount 8 with `FALLING`). The block state carries the legacy
//! `level` property: source 0, flowing `8 - amount`, falling 8. Every 5 game
//! ticks (`WaterFluid.getTickDelay`) a scheduled **fluid** tick — a separate
//! queue from block ticks, drained in its own phase — runs:
//!
//! - A non-source recomputes itself (`getNewLiquid`): the strongest same-fluid
//!   horizontal neighbour minus the drop-off of 1; **falling 8** whenever any
//!   water sits directly above; a full **source** when two or more horizontal
//!   sources flank it and the block below is solid or source water (infinite
//!   water). Zero or less means the block empties to air.
//! - Then it spreads (`spread`): **down first** — anything that can hold fluid
//!   below receives falling water and side spread is skipped (unless three or
//!   more neighbouring sources feed this cell); otherwise sideways, but only
//!   if this is a source or the cell below is not already a hole being fed.
//! - Side spread (`spreadToSides`) does the slope search: each passable
//!   horizontal direction is scored by its distance to the nearest hole within
//!   `getSlopeFindDistance` = 4, and only minimum-distance directions receive
//!   water — which is why a stream on a table runs straight for the edge.
//!
//! Writes are **loud** (vanilla flag 3), and every placed fluid schedules its
//! own next tick (`LiquidBlock.onPlace`).
//!
//! # Approximations, documented
//!
//! `canPassThroughWall` is a shape computation over collision faces; this
//! engine approximates it with the full-cube table — water flows between any
//! two non-full-cube cells. Waterlogged blocks count as *sources for
//! neighbours* (their `getFluidState` is a full source) but do not run their
//! own spread tick here; a build that relies on water leaking out of a
//! waterlogged stair will diverge and the capture that shows it becomes the
//! fixture for doing walls properly.

use crate::behaviour::{BlockBehaviour, TickCtx};
use crate::pos::{Dir, Pos};
use crate::state::StateId;
use crate::world::World;

/// `WaterFluid.getTickDelay`.
pub const WATER_TICK_DELAY: u64 = 5;
/// `WaterFluid.getDropOff`.
pub const WATER_DROP_OFF: u8 = 1;
/// `WaterFluid.getSlopeFindDistance`.
pub const SLOPE_FIND_DISTANCE: u8 = 4;

const HORIZONTALS: [Dir; 4] = [Dir::North, Dir::South, Dir::West, Dir::East];

/// One cell's worth of water, `FluidState` distilled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterKind {
    /// A still source, amount 8.
    Source,
    /// Flowing, amount 1-7 (higher is fuller).
    Flowing(u8),
    /// Falling, amount 8 with the `FALLING` flag.
    Falling,
}

impl WaterKind {
    /// `FluidState.getAmount`.
    pub fn amount(self) -> u8 {
        match self {
            WaterKind::Source | WaterKind::Falling => 8,
            WaterKind::Flowing(amount) => amount,
        }
    }

    /// `FluidState.getOwnHeight` = amount / 9.
    pub fn own_height(self) -> f32 {
        f32::from(self.amount()) / 9.0
    }

    /// Whether this is a source.
    pub fn is_source(self) -> bool {
        matches!(self, WaterKind::Source)
    }

    /// The block-state `level` property: source 0, flowing `8 - amount`,
    /// falling 8 (`FlowingFluid.getLegacyLevel`).
    pub fn legacy_level(self) -> u8 {
        match self {
            WaterKind::Source => 0,
            WaterKind::Flowing(amount) => 8 - amount,
            WaterKind::Falling => 8,
        }
    }

    /// From the block-state `level` property.
    pub fn from_level(level: u8) -> WaterKind {
        match level {
            0 => WaterKind::Source,
            1..=7 => WaterKind::Flowing(8 - level),
            _ => WaterKind::Falling,
        }
    }
}

/// What the water tick needs to know about everything that is not water.
pub trait FluidWorld: Send + Sync {
    /// The water at `pos`, if any — plain water blocks, waterlogged blocks and
    /// bubble columns all answer (vanilla `getFluidState`).
    fn water(&self, world: &World, pos: Pos) -> Option<WaterKind>;
    /// Whether flow can enter `pos` by replacing what is there. Air only, for
    /// now — vanilla also floods replaceable plants.
    fn can_flow_into(&self, world: &World, pos: Pos) -> bool;
    /// Whether `pos` is a full collision cube (`blocksMotion` / sturdy faces).
    fn is_solid(&self, world: &World, pos: Pos) -> bool;
    /// The `minecraft:water` state with the given legacy `level`.
    fn water_state(&self, level: u8) -> Option<StateId>;
}

/// Whether flow can pass into `pos`: not solid, not a source, and either
/// already flowing water or replaceable — vanilla's `canPassThrough`
/// (`canMaybePassThrough` + `!isSourceBlockOfThisType` + `canHoldFluid`).
fn can_pass_through(rules: &dyn FluidWorld, world: &World, pos: Pos) -> bool {
    if rules.is_solid(world, pos) {
        return false;
    }
    match rules.water(world, pos) {
        Some(kind) => !kind.is_source(),
        None => rules.can_flow_into(world, pos),
    }
}

/// `isWaterHole`: the cell below a candidate can swallow flow — same fluid or
/// a cell water could enter.
fn is_hole(rules: &dyn FluidWorld, world: &World, below: Pos) -> bool {
    if rules.is_solid(world, below) {
        return false;
    }
    rules.water(world, below).is_some() || rules.can_flow_into(world, below)
}

/// How many of the four horizontal neighbours are sources (`sourceNeighborCount`).
fn source_neighbour_count(rules: &dyn FluidWorld, world: &World, pos: Pos) -> usize {
    HORIZONTALS
        .iter()
        .filter(|dir| {
            rules
                .water(world, pos.offset(**dir))
                .is_some_and(WaterKind::is_source)
        })
        .count()
}

/// `getNewLiquid`: what the cell at `pos` should contain, judged from its
/// neighbours. `None` means empty.
pub fn new_liquid(rules: &dyn FluidWorld, world: &World, pos: Pos) -> Option<WaterKind> {
    let mut strongest = 0u8;
    let mut sources = 0usize;
    for dir in HORIZONTALS {
        if let Some(kind) = rules.water(world, pos.offset(dir)) {
            if kind.is_source() {
                sources += 1;
            }
            strongest = strongest.max(kind.amount());
        }
    }
    // Infinite water: two flanking sources over a solid or source floor.
    if sources >= 2 {
        let below = pos.offset(Dir::Down);
        if rules.is_solid(world, below)
            || rules.water(world, below).is_some_and(WaterKind::is_source)
        {
            return Some(WaterKind::Source);
        }
    }
    // Any water directly above makes this cell falling, full strength.
    if rules.water(world, pos.offset(Dir::Up)).is_some() {
        return Some(WaterKind::Falling);
    }
    let amount = strongest.saturating_sub(WATER_DROP_OFF);
    (amount > 0).then_some(WaterKind::Flowing(amount))
}

/// Write water into `pos` and schedule its own tick — `spreadTo` plus the
/// `LiquidBlock.onPlace` schedule that a real placement gets.
fn spread_to(rules: &dyn FluidWorld, ctx: &mut TickCtx<'_>, pos: Pos, kind: WaterKind) {
    let Some(state) = rules.water_state(kind.legacy_level()) else {
        return;
    };
    ctx.set(pos, state);
    schedule_fluid(ctx, pos);
}

/// Schedule a water tick. The queue dedupes per position on its own, exactly as
/// the game's fluid ticks do.
pub fn schedule_fluid(ctx: &mut TickCtx<'_>, pos: Pos) {
    ctx.schedule_fluid(pos, WATER_TICK_DELAY);
}

/// `getSlopeDistance`: nearest hole within [`SLOPE_FIND_DISTANCE`], walking
/// passable cells and never doubling back.
fn slope_distance(
    rules: &dyn FluidWorld,
    world: &World,
    pos: Pos,
    depth: u8,
    excluding: Dir,
) -> u16 {
    let mut best = 1000u16;
    for dir in HORIZONTALS {
        if dir == excluding {
            continue;
        }
        let target = pos.offset(dir);
        if !can_pass_through(rules, world, target) {
            continue;
        }
        if is_hole(rules, world, target.offset(Dir::Down)) {
            return u16::from(depth);
        }
        if depth < SLOPE_FIND_DISTANCE {
            best = best.min(slope_distance(rules, world, target, depth + 1, dir.opposite()));
        }
    }
    best
}

/// `getSpread`: the minimum-hole-distance directions and what lands in each —
/// the landing state is `getNewLiquid` at the target, not merely amount − 1,
/// which is how converging flows take the stronger value.
fn spread_directions(
    rules: &dyn FluidWorld,
    world: &World,
    pos: Pos,
) -> Vec<(Dir, WaterKind)> {
    let mut best = 1000u16;
    let mut chosen: Vec<(Dir, WaterKind)> = Vec::new();
    for dir in HORIZONTALS {
        let target = pos.offset(dir);
        if !can_pass_through(rules, world, target) {
            continue;
        }
        let distance = if is_hole(rules, world, target.offset(Dir::Down)) {
            0
        } else {
            slope_distance(rules, world, target, 1, dir.opposite())
        };
        if distance < best {
            chosen.clear();
        }
        if distance <= best {
            best = distance;
            if let Some(kind) = new_liquid(rules, world, target) {
                chosen.push((dir, kind));
            }
        }
    }
    chosen
}

/// `spreadToSides`: hand the slope-search winners their water.
fn spread_to_sides(rules: &dyn FluidWorld, ctx: &mut TickCtx<'_>, pos: Pos, kind: WaterKind) {
    let spread_amount = kind.amount().saturating_sub(WATER_DROP_OFF);
    if spread_amount == 0 {
        return;
    }
    for (dir, landing) in spread_directions(rules, ctx.world, pos) {
        // A target already holding exactly this is a no-op; ctx.set guards it.
        spread_to(rules, ctx, pos.offset(dir), landing);
    }
}

/// `spread`: down first and stop (side spread only with 3+ source neighbours);
/// otherwise sideways unless the flow is already pouring into a hole.
pub fn spread(rules: &dyn FluidWorld, ctx: &mut TickCtx<'_>, pos: Pos, kind: WaterKind) {
    let below = pos.offset(Dir::Down);
    let below_receives = !rules.is_solid(ctx.world, below)
        && rules.water(ctx.world, below).is_none()
        && rules.can_flow_into(ctx.world, below);
    if below_receives {
        spread_to(rules, ctx, below, WaterKind::Falling);
        if source_neighbour_count(rules, ctx.world, pos) >= 3 {
            spread_to_sides(rules, ctx, pos, kind);
        }
        return;
    }
    if kind.is_source() || !is_hole(rules, ctx.world, below) {
        spread_to_sides(rules, ctx, pos, kind);
    }
}

/// A water block state. One instance per `level`; the fluid tick recomputes and
/// spreads exactly as `FlowingFluid.tick` does.
pub struct Water<R: FluidWorld> {
    /// What this state holds.
    pub kind: WaterKind,
    /// The world rules.
    pub rules: R,
}

impl<R: FluidWorld + 'static> BlockBehaviour for Water<R> {
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        // LiquidBlock reacts to any neighbour change by scheduling a fluid tick.
        schedule_fluid(ctx, pos);
    }

    fn on_fluid_tick(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        let mut kind = self.kind;
        if !kind.is_source() {
            match new_liquid(&self.rules, ctx.world, pos) {
                None => {
                    // Emptied. Vanilla sets air (loud) and the subsequent
                    // spread of an empty fluid does nothing.
                    ctx.set(pos, StateId::AIR);
                    return;
                }
                Some(new_kind) if new_kind != kind => {
                    if let Some(state) = self.rules.water_state(new_kind.legacy_level()) {
                        ctx.set(pos, state);
                        ctx.schedule_fluid(pos, WATER_TICK_DELAY);
                        kind = new_kind;
                    }
                }
                _ => {}
            }
        }
        spread(&self.rules, ctx, pos, kind);
    }

    fn name(&self) -> &'static str {
        "water"
    }
}

/// The item-physics view of a cell's water surface: `FluidState.getHeight` —
/// 1.0 when the same fluid sits above, else `getOwnHeight`.
pub fn surface_height(kind: WaterKind, water_above: bool) -> f32 {
    if water_above {
        1.0
    } else {
        kind.own_height()
    }
}

/// `FlowingFluid.getFlow`: the flow vector at a water cell, from height
/// differences against the four horizontal neighbours; a falling cell walled
/// in on any side additionally pulls hard downward (−6 before normalizing).
/// Returned normalized, or zero. Height arithmetic stays in `f32`, as
/// vanilla's is, before widening into the vector.
pub fn flow_vector(
    water: &dyn Fn(Pos) -> Option<WaterKind>,
    solid: &dyn Fn(Pos) -> bool,
    pos: Pos,
) -> [f64; 3] {
    let Some(own) = water(pos) else {
        return [0.0; 3];
    };
    let own_height = own.own_height();
    let mut x = 0.0f64;
    let mut z = 0.0f64;
    for dir in HORIZONTALS {
        let neighbour = pos.offset(dir);
        let mut delta = 0.0f32;
        match water(neighbour) {
            Some(kind) => {
                delta = own_height - kind.own_height();
            }
            None => {
                // No fluid beside us: if the block there doesn't block motion
                // and water sits *below* it, the surface steps down a full
                // block — height − 8/9 against our own.
                if !solid(neighbour) {
                    if let Some(below) = water(neighbour.offset(Dir::Down)) {
                        let height = below.own_height();
                        if height > 0.0 {
                            delta = own_height - (height - 0.888_888_9);
                        }
                    }
                }
            }
        }
        if delta != 0.0 {
            let (dx, dz) = dir_step(dir);
            x += f64::from(dx) * f64::from(delta);
            z += f64::from(dz) * f64::from(delta);
        }
    }
    let mut vec = [x, 0.0, z];
    if own == WaterKind::Falling {
        for dir in HORIZONTALS {
            let neighbour = pos.offset(dir);
            if solid(neighbour) || solid(neighbour.offset(Dir::Up)) {
                vec = normalize(vec);
                vec[1] -= 6.0;
                break;
            }
        }
    }
    normalize(vec)
}

fn dir_step(dir: Dir) -> (i32, i32) {
    match dir {
        Dir::North => (0, -1),
        Dir::South => (0, 1),
        Dir::West => (-1, 0),
        Dir::East => (1, 0),
        Dir::Up | Dir::Down => (0, 0),
    }
}

fn normalize(vec: [f64; 3]) -> [f64; 3] {
    let length = (vec[0] * vec[0] + vec[1] * vec[1] + vec[2] * vec[2]).sqrt();
    if length < 1.0e-4 {
        return [0.0; 3];
    }
    [vec[0] / length, vec[1] / length, vec[2] / length]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_levels_round_trip() {
        for level in 0u8..=8 {
            let kind = WaterKind::from_level(level);
            assert_eq!(kind.legacy_level(), level, "level {level}");
        }
        assert_eq!(WaterKind::Source.amount(), 8);
        assert_eq!(WaterKind::Flowing(7).legacy_level(), 1);
        assert_eq!(WaterKind::Falling.amount(), 8);
    }

    #[test]
    fn own_height_is_amount_over_nine() {
        assert!((WaterKind::Source.own_height() - 8.0 / 9.0).abs() < 1e-6);
        assert!((WaterKind::Flowing(1).own_height() - 1.0 / 9.0).abs() < 1e-6);
    }
}
