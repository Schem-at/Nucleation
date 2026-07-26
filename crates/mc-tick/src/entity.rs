//! Item entities: the first floating-point citizens of the engine.
//!
//! Everything here mirrors `ItemEntity.tick` bytecode, arithmetic types
//! included — drag factors are `f32` widened to `f64` exactly where vanilla
//! widens them, because the goal is bit-identical trajectories, verified by
//! RNG-free captures (items authored directly in structure files).
//!
//! # The tick, from bytecode
//!
//! ```text
//! pickupDelay-- (when finite)
//! v.y -= 0.04                                  (applyGravity)
//! resting = onGround && horizSqr(v) <= 1e-5 && (tickCount + id) % 4 != 0
//! if !resting:
//!     move with collision                       (clipped axes zero their velocity)
//!     drag = 0.98f;  ground = drag * frictionOf(block below) when onGround
//!     v = (v.x * ground, v.y * drag, v.z * ground)
//!     if onGround && v.y < 0: v.y *= -0.5       (the landing bounce)
//! age++; despawn at 6000
//! ```
//!
//! The resting skip means a settled item's velocity keeps accumulating gravity
//! and is periodically flushed by a collision — position never changes, which
//! is why traces emit entity events on **position** change only.

use crate::pos::Pos;

/// Air drag per tick, `Entity.getAirDrag`.
pub const AIR_DRAG: f32 = 0.98;

/// Gravity per tick, `ItemEntity.getDefaultGravity`.
pub const ITEM_GRAVITY: f64 = 0.04;

/// Age at which an item despawns, `ItemEntity.LIFETIME`.
pub const ITEM_LIFETIME: u32 = 6000;

/// Half the item hitbox's width (`EntityType.ITEM`: 0.25 × 0.25).
pub const ITEM_HALF_WIDTH: f64 = 0.125;

/// The item hitbox's height.
pub const ITEM_HEIGHT: f64 = 0.25;

/// One item entity.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemEntityState {
    /// Trace-stable id, assigned in spawn order.
    pub id: u32,
    /// `(item id, count)`.
    pub item: (String, u8),
    /// Feet-centre position, vanilla's entity position convention.
    pub pos: [f64; 3],
    /// Velocity, blocks per tick.
    pub vel: [f64; 3],
    pub on_ground: bool,
    /// Ticks since spawn (`Entity.tickCount`), incremented at tick start.
    pub tick_count: u32,
    /// `ItemEntity.age`; despawn at [`ITEM_LIFETIME`].
    pub age: u32,
    /// Player-pickup delay. Tracked for fidelity; hoppers ignore it.
    pub pickup_delay: u32,
    /// Set when discarded; removed (and reported) at end of tick.
    pub removed: bool,
}

/// All item entities plus the id counter — one bundle so the tick context
/// carries a single handle.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ItemEntities {
    /// In spawn order, which is also vanilla's entity-list iteration order.
    pub items: Vec<ItemEntityState>,
    /// The next id to assign.
    pub next_id: u32,
}

impl ItemEntities {
    /// Spawn an item entity, returning its id.
    pub fn spawn(
        &mut self,
        item: (String, u8),
        pos: [f64; 3],
        vel: [f64; 3],
        pickup_delay: u32,
    ) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.items.push(ItemEntityState {
            id,
            item,
            pos,
            vel,
            on_ground: false,
            tick_count: 0,
            age: 0,
            pickup_delay,
            removed: false,
        });
        id
    }
}

/// What item physics needs to know about the world.
pub trait CollisionWorld {
    /// Whether the block at `pos` is a full collision cube.
    fn is_solid(&self, pos: Pos) -> bool;
    /// `Block.getFriction` of the block at `pos` (0.6 for almost everything).
    fn friction(&self, pos: Pos) -> f32;
}

/// Advance one item entity by one tick. Returns `true` while it lives.
pub fn tick_item(entity: &mut ItemEntityState, world: &dyn CollisionWorld) -> bool {
    entity.tick_count += 1;
    if entity.pickup_delay > 0 && entity.pickup_delay != 32767 {
        entity.pickup_delay -= 1;
    }

    entity.vel[1] -= ITEM_GRAVITY;

    let horizontal_sqr = entity.vel[0] * entity.vel[0] + entity.vel[2] * entity.vel[2];
    let resting = entity.on_ground
        && horizontal_sqr <= 1.0e-5
        && (entity.tick_count + entity.id) % 4 != 0;
    if !resting {
        move_with_collision(entity, world);
        let drag = AIR_DRAG;
        let mut ground_drag = drag;
        if entity.on_ground {
            let below = block_below_affecting_movement(entity);
            ground_drag *= world.friction(below);
        }
        entity.vel[0] *= f64::from(ground_drag);
        entity.vel[1] *= f64::from(drag);
        entity.vel[2] *= f64::from(ground_drag);
        if entity.on_ground && entity.vel[1] < 0.0 {
            entity.vel[1] *= -0.5;
        }
    }

    entity.age += 1;
    if entity.age >= ITEM_LIFETIME {
        entity.removed = true;
    }
    !entity.removed
}

/// `Entity.getBlockPosBelowThatAffectsMyMovement`: 0.500001 below the feet.
fn block_below_affecting_movement(entity: &ItemEntityState) -> Pos {
    Pos::new(
        entity.pos[0].floor() as i32,
        (entity.pos[1] - 0.500001).floor() as i32,
        entity.pos[2].floor() as i32,
    )
}

/// The item's AABB: `(min, max)`.
pub fn item_aabb(pos: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    (
        [pos[0] - ITEM_HALF_WIDTH, pos[1], pos[2] - ITEM_HALF_WIDTH],
        [
            pos[0] + ITEM_HALF_WIDTH,
            pos[1] + ITEM_HEIGHT,
            pos[2] + ITEM_HALF_WIDTH,
        ],
    )
}

/// Move with axis-clipped collision against full-cube blocks: Y first, then
/// the larger horizontal axis — `Entity.collideBoundingBox`'s order. A clipped
/// axis zeroes its velocity component; a downward Y clip sets `on_ground`.
fn move_with_collision(entity: &mut ItemEntityState, world: &dyn CollisionWorld) {
    let (mut min, mut max) = item_aabb(entity.pos);
    let mut movement = entity.vel;

    let clipped_y = {
        let clipped = clip_axis(world, min, max, 1, movement[1]);
        let hit = clipped != movement[1];
        min[1] += clipped;
        max[1] += clipped;
        movement[1] = clipped;
        hit
    };
    let x_first = movement[0].abs() > movement[2].abs();
    let order = if x_first { [0usize, 2] } else { [2, 0] };
    let mut clipped_horizontal = [false, false];
    for (index, &axis) in order.iter().enumerate() {
        let clipped = clip_axis(world, min, max, axis, movement[axis]);
        clipped_horizontal[index] = clipped != movement[axis];
        min[axis] += clipped;
        max[axis] += clipped;
        movement[axis] = clipped;
    }

    entity.pos[0] += movement[0];
    entity.pos[1] += movement[1];
    entity.pos[2] += movement[2];

    // Vanilla zeroes collided components and derives onGround from a downward
    // vertical collision.
    entity.on_ground = clipped_y && entity.vel[1] < 0.0;
    if clipped_y {
        entity.vel[1] = 0.0;
    }
    if clipped_horizontal[0] || clipped_horizontal[1] {
        if order[0] == 0 && clipped_horizontal[0] || order[1] == 0 && clipped_horizontal[1] {
            entity.vel[0] = 0.0;
        }
        if order[0] == 2 && clipped_horizontal[0] || order[1] == 2 && clipped_horizontal[1] {
            entity.vel[2] = 0.0;
        }
    }
}

/// Clip a single-axis movement of the box `(min, max)` against solid blocks.
fn clip_axis(
    world: &dyn CollisionWorld,
    min: [f64; 3],
    max: [f64; 3],
    axis: usize,
    mut delta: f64,
) -> f64 {
    if delta == 0.0 {
        return 0.0;
    }
    // The swept region: everywhere the box passes through along this axis.
    let mut sweep_min = min;
    let mut sweep_max = max;
    if delta > 0.0 {
        sweep_max[axis] += delta;
    } else {
        sweep_min[axis] += delta;
    }
    const EPSILON: f64 = 1.0e-7;

    let lo = |v: f64| (v + EPSILON).floor() as i32;
    let hi = |v: f64| (v - EPSILON).floor() as i32;
    for x in lo(sweep_min[0])..=hi(sweep_max[0]) {
        for y in lo(sweep_min[1])..=hi(sweep_max[1]) {
            for z in lo(sweep_min[2])..=hi(sweep_max[2]) {
                if !world.is_solid(Pos::new(x, y, z)) {
                    continue;
                }
                let block_min = [f64::from(x), f64::from(y), f64::from(z)];
                let block_max = [block_min[0] + 1.0, block_min[1] + 1.0, block_min[2] + 1.0];
                // Must overlap on the other two axes to matter.
                let others: [usize; 2] = match axis {
                    0 => [1, 2],
                    1 => [0, 2],
                    _ => [0, 1],
                };
                let overlaps = others.iter().all(|&other| {
                    min[other] + EPSILON < block_max[other]
                        && max[other] - EPSILON > block_min[other]
                });
                if !overlaps {
                    continue;
                }
                if delta > 0.0 {
                    let gap = block_min[axis] - max[axis];
                    if gap >= -EPSILON && gap < delta {
                        delta = gap.max(0.0);
                    }
                } else {
                    let gap = block_max[axis] - min[axis];
                    if gap <= EPSILON && gap > delta {
                        delta = gap.min(0.0);
                    }
                }
            }
        }
    }
    delta
}

/// `ItemEntity.isMergable`.
fn is_mergable(entity: &ItemEntityState) -> bool {
    !entity.removed && entity.item.1 < 64 && entity.age < ITEM_LIFETIME
}

/// `mergeWithNeighbours` for the entity at `index`: absorb into the larger
/// stack within ±0.5 horizontally. The receiver keeps its id; an emptied
/// entity is discarded.
pub fn merge_neighbours(entities: &mut ItemEntities, index: usize) {
    if !is_mergable(&entities.items[index]) {
        return;
    }
    let (pos, item_id) = {
        let e = &entities.items[index];
        (e.pos, e.item.0.clone())
    };
    let (min, max) = item_aabb(pos);
    for other_index in 0..entities.items.len() {
        if other_index == index {
            continue;
        }
        let other = &entities.items[other_index];
        if !is_mergable(other) || other.item.0 != item_id {
            continue;
        }
        let (omin, omax) = item_aabb(other.pos);
        let intersects = omin[0] < max[0] + 0.5
            && omax[0] > min[0] - 0.5
            && omin[1] < max[1]
            && omax[1] > min[1]
            && omin[2] < max[2] + 0.5
            && omax[2] > min[2] - 0.5;
        if !intersects {
            continue;
        }
        let total = entities.items[index].item.1 as u32 + other.item.1 as u32;
        if total > 64 {
            continue; // areMergable refuses over-full merges
        }
        // The larger (or equal, favouring the ticking entity) stack receives.
        let (receiver, giver) = if entities.items[index].item.1 < entities.items[other_index].item.1
        {
            (other_index, index)
        } else {
            (index, other_index)
        };
        entities.items[receiver].item.1 = total as u8;
        entities.items[giver].item.1 = 0;
        entities.items[giver].removed = true;
        if entities.items[index].removed {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    struct Floor;
    impl CollisionWorld for Floor {
        fn is_solid(&self, pos: Pos) -> bool {
            pos.y < 0
        }
        fn friction(&self, _pos: Pos) -> f32 {
            0.6
        }
    }

    fn item_at(y: f64) -> ItemEntityState {
        ItemEntityState {
            id: 0,
            item: ("minecraft:redstone".to_string(), 1),
            pos: [0.5, y, 0.5],
            vel: [0.0; 3],
            on_ground: false,
            tick_count: 0,
            age: 0,
            pickup_delay: 0,
            removed: false,
        }
    }

    #[test]
    fn a_falling_item_accelerates_with_gravity_and_drag() {
        // First tick: v.y = (0 - 0.04) * 0.98, applied before the multiply —
        // position moves by the pre-drag velocity.
        let mut item = item_at(3.0);
        tick_item(&mut item, &Floor);
        assert!((item.pos[1] - (3.0 - 0.04)).abs() < 1e-12);
        assert!((item.vel[1] - (-0.04 * 0.98f32 as f64)).abs() < 1e-9);
    }

    #[test]
    fn an_item_lands_on_the_floor_and_stays() {
        let mut item = item_at(2.0);
        for _ in 0..200 {
            tick_item(&mut item, &Floor);
        }
        assert!(item.on_ground, "must come to rest");
        assert!(
            (item.pos[1] - 0.0).abs() < 1e-9,
            "rests exactly on the floor: {}",
            item.pos[1]
        );
    }

    #[test]
    fn resting_items_do_not_move() {
        let mut item = item_at(1.0);
        for _ in 0..100 {
            tick_item(&mut item, &Floor);
        }
        let rest = item.pos;
        for _ in 0..100 {
            tick_item(&mut item, &Floor);
        }
        assert_eq!(rest, item.pos, "a settled item's position is fixed");
    }

    #[test]
    fn merging_prefers_the_larger_stack_and_discards_the_other() {
        let mut entities = ItemEntities::default();
        entities.spawn(("minecraft:redstone".to_string(), 3), [0.4, 0.0, 0.5], [0.0; 3], 0);
        entities.spawn(("minecraft:redstone".to_string(), 5), [0.6, 0.0, 0.5], [0.0; 3], 0);
        merge_neighbours(&mut entities, 0);
        let alive: Vec<_> = entities.items.iter().filter(|e| !e.removed).collect();
        assert_eq!(alive.len(), 1);
        assert_eq!(alive[0].item.1, 8, "counts combine");
        assert_eq!(alive[0].id, 1, "the larger stack is the receiver");
    }

    #[test]
    fn ids_are_assigned_in_spawn_order() {
        let mut entities = ItemEntities::default();
        let a = entities.spawn(("minecraft:stone".to_string(), 1), [0.0; 3], [0.0; 3], 0);
        let b = entities.spawn(("minecraft:stone".to_string(), 1), [0.0; 3], [0.0; 3], 0);
        assert_eq!((a, b), (0, 1));
        let ids: HashSet<u32> = entities.items.iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), 2);
    }
}
