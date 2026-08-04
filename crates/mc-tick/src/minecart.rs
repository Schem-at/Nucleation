//! Minecarts on rails — `AbstractMinecart` + `OldMinecartBehavior` bytecode.
//!
//! 26.2 ships two behaviours; without the `minecart_improvements` experiment
//! the server runs **OldMinecartBehavior**, the physics unchanged since alpha,
//! and that is what this module implements:
//!
//! ```text
//! tick:  vy -= 0.04 (gravity)
//!        railPos = block position, or the block below if that is a rail
//!        on rail  -> moveAlongTrack
//!        off rail -> comeOffTrack (clamp ±0.4, ×0.5 grounded, move, ×0.95f airborne)
//!
//! moveAlongTrack:
//!        posBefore = getPos(x, y, z)            (snap point on the rail line)
//!        powered/braking from powered_rail's POWERED
//!        ascending shapes pull 0.0078125 downhill (×0.2 in water) and lift y+1
//!        velocity is projected onto the exit chord, speed capped at 2.0
//!        braking: |v| < 0.03 -> stop dead, else ×0.5
//!        position clamped onto the chord, then move (per-axis clamp ±0.4)
//!        corner fixups re-seat the cart when it crossed onto a sloped exit
//!        applyNaturalSlowdown: ×(0.997 ridden / 0.96 empty), vy zeroed
//!        height correction: (before.y − after.y) × 0.05 feeds the speed
//!        leaving the rail block redirects velocity at the neighbour
//!        boost rail: +0.06 along motion; from rest, launch 0.02 away from a
//!        conductor pressed against either end
//! ```
//!
//! `EXITS` is transcribed from the static initialiser, pair order included —
//! the order is observable through the dot-product flip and corner fixups.

use crate::entity::CollisionWorld;
use crate::pos::Pos;

/// Cart gravity per tick (`getDefaultGravity`).
pub const CART_GRAVITY: f64 = 0.04;
/// `MAX_SPEED_ON_LAND` — the per-axis movement clamp.
pub const MAX_SPEED: f64 = 0.4;
/// Downhill pull per tick on ascending rails.
pub const SLOPE_ACCELERATION: f64 = 0.0078125;
/// `getAirDrag` for carts, applied off-rail while airborne.
pub const CART_AIR_DRAG: f32 = 0.95;
/// Half the cart hitbox width — `EntityType.MINECART` is `scalable(0.98F, 0.7F)`.
///
/// The literal is a **float**, so the width is 0.9800000190734863 and this is
/// 0.49000000953674316, not 0.49. That eighth decimal is measurable: two carts
/// parked 0.99 apart have their approach clipped to 0.009999981, not to 0.01,
/// and the `cart_gap` golden reads exactly the former.
pub const CART_HALF_WIDTH: f64 = (0.98_f32 as f64) / 2.0;
/// The cart hitbox height.
pub const CART_HEIGHT: f64 = 0.7_f32 as f64;

/// The ten rail shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RailShape {
    /// `north_south`
    NorthSouth,
    /// `east_west`
    EastWest,
    /// `ascending_east`
    AscendingEast,
    /// `ascending_west`
    AscendingWest,
    /// `ascending_north`
    AscendingNorth,
    /// `ascending_south`
    AscendingSouth,
    /// `south_east`
    SouthEast,
    /// `south_west`
    SouthWest,
    /// `north_west`
    NorthWest,
    /// `north_east`
    NorthEast,
}

impl RailShape {
    /// Parse the block-state property value.
    pub fn from_name(name: &str) -> Option<RailShape> {
        Some(match name {
            "north_south" => RailShape::NorthSouth,
            "east_west" => RailShape::EastWest,
            "ascending_east" => RailShape::AscendingEast,
            "ascending_west" => RailShape::AscendingWest,
            "ascending_north" => RailShape::AscendingNorth,
            "ascending_south" => RailShape::AscendingSouth,
            "south_east" => RailShape::SouthEast,
            "south_west" => RailShape::SouthWest,
            "north_west" => RailShape::NorthWest,
            "north_east" => RailShape::NorthEast,
            _ => return None,
        })
    }

    /// `AbstractMinecart.EXITS`, pair order preserved from the bytecode:
    /// west/east/north/south are unit offsets, "below" variants carry y = −1.
    pub fn exits(self) -> ([i32; 3], [i32; 3]) {
        const WEST: [i32; 3] = [-1, 0, 0];
        const EAST: [i32; 3] = [1, 0, 0];
        const NORTH: [i32; 3] = [0, 0, -1];
        const SOUTH: [i32; 3] = [0, 0, 1];
        const WEST_BELOW: [i32; 3] = [-1, -1, 0];
        const EAST_BELOW: [i32; 3] = [1, -1, 0];
        const NORTH_BELOW: [i32; 3] = [0, -1, -1];
        const SOUTH_BELOW: [i32; 3] = [0, -1, 1];
        match self {
            RailShape::NorthSouth => (NORTH, SOUTH),
            RailShape::EastWest => (WEST, EAST),
            RailShape::AscendingEast => (WEST_BELOW, EAST),
            RailShape::AscendingWest => (WEST, EAST_BELOW),
            RailShape::AscendingNorth => (NORTH, SOUTH_BELOW),
            RailShape::AscendingSouth => (NORTH_BELOW, SOUTH),
            RailShape::SouthEast => (SOUTH, EAST),
            RailShape::SouthWest => (SOUTH, WEST),
            RailShape::NorthWest => (NORTH, WEST),
            RailShape::NorthEast => (NORTH, EAST),
        }
    }

    /// Whether this is one of the four sloped shapes.
    pub fn is_ascending(self) -> bool {
        matches!(
            self,
            RailShape::AscendingEast
                | RailShape::AscendingWest
                | RailShape::AscendingNorth
                | RailShape::AscendingSouth
        )
    }
}

/// A rail block, as cart physics sees it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rail {
    /// The track shape.
    pub shape: RailShape,
    /// Whether this is a powered (golden) rail.
    pub powered_rail: bool,
    /// The `powered` property, meaningful on powered rails.
    pub powered: bool,
}

/// One minecart.
#[derive(Debug, Clone, PartialEq)]
pub struct MinecartState {
    /// Trace-stable id, from the shared entity counter.
    pub id: u32,
    /// e.g. `minecraft:minecart`.
    pub kind: String,
    /// Entity position (feet centre).
    pub pos: [f64; 3],
    /// Velocity, blocks per tick.
    pub vel: [f64; 3],
    /// Whether the last move ended on the ground.
    pub on_ground: bool,
    /// Whether the cart sat on a rail last tick.
    pub on_rails: bool,
    /// Set when discarded.
    pub removed: bool,
    /// The cart's container — `AbstractMinecartContainer`'s inventory, for
    /// chest and hopper carts. `None` for cart kinds that carry no items.
    pub inventory: Option<crate::inventory::Inventory>,
    /// A TNT cart's lit fuse — `MinecartTNT.fuse`, `None` unprimed.
    pub fuse: Option<i32>,
    /// `yRot`, degrees, as the *polar angle in the XZ plane* — the sense
    /// `AbstractMinecart` writes and reads it, `atan2(dz, dx)`, so 0 points
    /// +X and 90 points +Z.
    ///
    /// Carried because cart-cart pushing gates on it: a pair only shoves each
    /// other when the line between them is within ~37° of the facing. An
    /// entity with no `Rotation` tag starts at 0, which is what a structure
    /// spawn gives — the structure reader does not carry rotation yet.
    pub yaw: f64,
}

/// `getCurrentBlockPosOrRailBelow`: the cart's block, or the block below when
/// that one is the rail (a cart on a rail sits at the rail's top surface, so
/// its feet round into the block above on slopes).
fn rail_block_pos(cart: &MinecartState, world: &dyn CollisionWorld) -> Pos {
    let pos = Pos::new(
        cart.pos[0].floor() as i32,
        cart.pos[1].floor() as i32,
        cart.pos[2].floor() as i32,
    );
    let below = Pos::new(pos.x, pos.y - 1, pos.z);
    if world.rail(below).is_some() {
        below
    } else {
        pos
    }
}

/// `OldMinecartBehavior.getPos`: the exact point on the rail line beneath
/// `(x, y, z)`, or `None` off-rail. The 0.0625 is the rail's surface height.
pub fn rail_snap(world: &dyn CollisionWorld, x: f64, y: f64, z: f64) -> Option<[f64; 3]> {
    let bx = x.floor() as i32;
    let mut by = y.floor() as i32;
    let bz = z.floor() as i32;
    if world.rail(Pos::new(bx, by - 1, bz)).is_some() {
        by -= 1;
    }
    let rail = world.rail(Pos::new(bx, by, bz))?;
    let (first, second) = rail.shape.exits();
    let d0 = f64::from(bx) + 0.5 + f64::from(first[0]) * 0.5;
    let d1 = f64::from(by) + 0.0625 + f64::from(first[1]) * 0.5;
    let d2 = f64::from(bz) + 0.5 + f64::from(first[2]) * 0.5;
    let d3 = f64::from(bx) + 0.5 + f64::from(second[0]) * 0.5;
    let d4 = f64::from(by) + 0.0625 + f64::from(second[1]) * 0.5;
    let d5 = f64::from(bz) + 0.5 + f64::from(second[2]) * 0.5;
    let dx = d3 - d0;
    // The y-delta is doubled (bytecode: `(d4 − d1) × 2.0`) — a sloped rail
    // spans a full block of height across its one block of run.
    let dy = (d4 - d1) * 2.0;
    let dz = d5 - d2;
    let t = if dx == 0.0 {
        z - f64::from(bz)
    } else if dz == 0.0 {
        x - f64::from(bx)
    } else {
        ((x - d0) * dx + (z - d2) * dz) * 2.0
    };
    let out_x = d0 + dx * t;
    let mut out_y = d1 + dy * t;
    let out_z = d2 + dz * t;
    if dy < 0.0 {
        out_y += 1.0;
    } else if dy > 0.0 {
        out_y += 0.5;
    }
    Some([out_x, out_y, out_z])
}

/// The cart's AABB at `pos`.
/// Container slots by cart kind — `MinecartChest` carries 27, `MinecartHopper` 5.
pub fn cart_container_slots(kind: &str) -> Option<u32> {
    match kind {
        "minecraft:chest_minecart" => Some(27),
        "minecraft:hopper_minecart" => Some(5),
        _ => None,
    }
}

pub fn cart_aabb(pos: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    (
        [pos[0] - CART_HALF_WIDTH, pos[1], pos[2] - CART_HALF_WIDTH],
        [
            pos[0] + CART_HALF_WIDTH,
            pos[1] + CART_HEIGHT,
            pos[2] + CART_HALF_WIDTH,
        ],
    )
}

/// `Entity.move(SELF, movement)` for a cart: clip, apply (past the 1e-7
/// gate), zero clipped components, set `on_ground`.
fn move_cart(
    cart: &mut MinecartState,
    world: &dyn CollisionWorld,
    movement: [f64; 3],
    obstacles: &[([f64; 3], [f64; 3])],
) {
    let (min, max) = cart_aabb(cart.pos);
    let (clipped, hit) = crate::entity::collide_move_among(world, min, max, movement, obstacles);
    // Vanilla applies a movement once it exceeds 1e-7 in *length*. The obvious
    // reading — `lengthSqr() > 1.0E-7` — is too coarse by seven orders of
    // magnitude, and `cart_gap` catches it: a squeezed cart there creeps
    // 4.75e-5 a tick, which that reading would round away and the game does
    // not. Dropping the gate entirely is also wrong; the goldens want one.
    //
    // Any threshold keeps a NaN cart frozen, since every comparison against NaN
    // is false. That is what makes a nan cart immovable by anything but a
    // piston, and it survives this change.
    let sqr = clipped[0] * clipped[0] + clipped[1] * clipped[1] + clipped[2] * clipped[2];
    if sqr > 1.0e-14 {
        cart.pos[0] += clipped[0];
        cart.pos[1] += clipped[1];
        cart.pos[2] += clipped[2];
    }
    cart.on_ground = hit[1] && movement[1] < 0.0;
    // Through `set_delta`, because vanilla zeroes a clipped axis with
    // `setDeltaMovement` like everything else: if the cart's other components
    // are non-finite the write is refused and a NaN cart keeps its NaN rather
    // than being quietly converted into a stationary ordinary cart.
    //
    // And the zeroing is a **multiply by zero**, not an assignment of zero.
    // Two captures say so independently:
    //
    // * `piston_pull.entities.log` records a cart's velocity as `(-0.0, 0.0,
    //   0.0)`. A negative zero cannot come from assigning `0.0`; it is what
    //   `x * 0.0` gives for a negative `x`.
    // * `piston_entity.entities.log` has a cart resting off-rail on stone with
    //   `Motion.z` NaN, and it is still NaN thirty ticks later. Assigning
    //   `0.0` to the clipped z would make the vector finite, and the
    //   `setDeltaMovement` guard would then *accept* it — quietly turning the
    //   nan cart into an ordinary stationary one. `NaN * 0.0` is NaN, the
    //   guard refuses the write, and the cart stays dead. That is the whole
    //   mechanism the record doors are glued together with.
    //
    // For every finite velocity the two are indistinguishable, which is why
    // this went unnoticed until an entity was parked off-rail.
    if hit[0] {
        set_delta(cart, [cart.vel[0] * 0.0, cart.vel[1], cart.vel[2]]);
    }
    if hit[1] {
        set_delta(cart, [cart.vel[0], cart.vel[1] * 0.0, cart.vel[2]]);
    }
    if hit[2] {
        set_delta(cart, [cart.vel[0], cart.vel[1], cart.vel[2] * 0.0]);
    }
}

/// One cart tick — `OldMinecartBehavior.tick`, server side, riderless.
///
/// This is the movement half only. Vanilla's `AbstractMinecart.tick` then
/// shoves whatever shares its space; that half is [`push_neighbours`], which
/// the caller runs straight after this for the same cart, because it needs the
/// other carts and this function only has one.
pub fn tick_minecart(cart: &mut MinecartState, world: &dyn CollisionWorld) {
    tick_minecart_blocked(cart, world, &[]);
}

/// One cart tick, with the boxes of the other carts it can be stopped by.
///
/// Prefer [`tick_minecart_among`], which builds the list. This is the seam for
/// a caller that already has the boxes, and for tests that want one cart and an
/// explicit obstacle.
pub fn tick_minecart_blocked(
    cart: &mut MinecartState,
    world: &dyn CollisionWorld,
    obstacles: &[([f64; 3], [f64; 3])],
) {
    let before = [cart.pos[0], cart.pos[2]];
    // Gravity through `set_delta` like every other velocity write. Vanilla's
    // is `setDeltaMovement(getDeltaMovement().add(0.0, -0.04, 0.0))`, and the
    // `isFinite` guard refuses the whole vector when any component is
    // non-finite — so a NaN cart does not accumulate fall speed. The
    // `piston_entity` capture reads its nan cart's velocity as `(0, 0, NaN)`
    // on every one of thirty ticks, never `(0, -0.04, NaN)`, which is what a
    // direct subtraction here produced.
    set_delta(cart, [cart.vel[0], cart.vel[1] - CART_GRAVITY, cart.vel[2]]);
    let rail_pos = rail_block_pos(cart, world);
    match world.rail(rail_pos) {
        Some(rail) => {
            cart.on_rails = true;
            move_along_track(cart, world, rail_pos, rail, obstacles);
        }
        None => {
            cart.on_rails = false;
            come_off_track(cart, world, obstacles);
        }
    }
    // `yRot = atan2(zo - z, xo - x)`, but only once the tick's travel clears
    // 0.001 squared — a cart creeping slower than 0.0316 a tick keeps the
    // heading it already had, which is the whole reason a parked cart can hold
    // a stale yaw and refuse to be pushed.
    //
    // Vanilla then folds in a `flipped` flag that adds 180°. It is dropped
    // here: the only consumer of yaw in this engine is the push gate, which
    // takes `abs` of the dot product, and negating the facing cannot change
    // that. If a second consumer appears, `flipped` has to come with it.
    let dx = cart.pos[0] - before[0];
    let dz = cart.pos[2] - before[1];
    if dx * dx + dz * dz > 0.001 {
        cart.yaw = dz.atan2(dx).to_degrees();
    }
}

/// Tick the cart at `index`, stopped by every other live cart **and** by
/// `bodies` — the boxes of the non-cart entities that stop a cart.
///
/// Carts block each other's movement. That single fact is what turns the
/// two-body push into the behaviour of a *chain*: shove a cart into a
/// neighbour it is already flush against and it does not move, and its velocity
/// on that axis is zeroed — so it goes on to push its own neighbours from a
/// standstill rather than from the velocity it was handed. Every chain golden
/// falls out of it, and none of them falls out without it.
///
/// `bodies` is the caller's business because this module owns only carts;
/// [`crate::sim::Simulation`] builds it from the frozen entities and the seated
/// riders, filtered by [`crate::entity::blocks_a_cart`]. Pass an empty slice for
/// a world that has none, which is what every pre-existing cart golden is.
///
/// The bodies go in through the *same* obstacle list as the carts, so a body is
/// a full AABB and not a floor: `cart_body2` measures a furnace cart rolling
/// east into a blaze and it stops with its east face at the blaze's west face,
/// where a support-only model would have driven straight through. And the boxes
/// are one list rather than two because vanilla's `Entity.collide` takes one
/// list: whichever is nearer wins, per axis, with no ordering between them.
///
/// A cart's own passenger may be in `bodies` and is harmless: its box overlaps
/// the cart's on every axis, and [`crate::entity::collide_move_among`] only
/// clips a box the mover is *outside* of. Pinned by
/// `a_cart_is_not_stopped_by_the_rider_sitting_on_it`.
pub fn tick_minecart_among(
    carts: &mut [MinecartState],
    index: usize,
    world: &dyn CollisionWorld,
    bodies: &[([f64; 3], [f64; 3])],
) {
    let obstacles: Vec<([f64; 3], [f64; 3])> = carts
        .iter()
        .enumerate()
        .filter(|(other, cart)| *other != index && !cart.removed)
        .map(|(_, cart)| cart_aabb(cart.pos))
        .chain(bodies.iter().copied())
        .collect();
    tick_minecart_blocked(&mut carts[index], world, &obstacles);
}

/// `Mth.SIN`: 65536 **float** samples of one turn, the table `Mth.sin`/`Mth.cos`
/// read instead of calling `Math.sin`. Vanilla's push gate goes through it, and
/// it is only accurate to about 1e-4, so reproducing the structure rather than
/// calling `f64::sin` is what keeps the threshold comparison honest.
fn sin_table() -> &'static [f32; 65536] {
    use std::sync::OnceLock;
    static TABLE: OnceLock<Box<[f32; 65536]>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = Box::new([0.0f32; 65536]);
        for (index, slot) in table.iter_mut().enumerate() {
            *slot = (index as f64 * std::f64::consts::TAU / 65536.0).sin() as f32;
        }
        table
    })
}

fn mth_sin(radians: f32) -> f32 {
    sin_table()[((radians * 10430.378_f32) as i32 as usize) & 0xFFFF]
}

fn mth_cos(radians: f32) -> f32 {
    sin_table()[((radians * 10430.378_f32 + 16384.0) as i32 as usize) & 0xFFFF]
}

/// `comeOffTrack`: clamp, halve when grounded, move, air-drag when airborne.
fn come_off_track(
    cart: &mut MinecartState,
    world: &dyn CollisionWorld,
    obstacles: &[([f64; 3], [f64; 3])],
) {
    cart.vel[0] = cart.vel[0].clamp(-MAX_SPEED, MAX_SPEED);
    cart.vel[2] = cart.vel[2].clamp(-MAX_SPEED, MAX_SPEED);
    if cart.on_ground {
        for axis in &mut cart.vel {
            *axis *= 0.5;
        }
    }
    move_cart(cart, world, cart.vel, obstacles);
    if !cart.on_ground {
        for axis in &mut cart.vel {
            *axis *= f64::from(CART_AIR_DRAG);
        }
    }
}

/// `OldMinecartBehavior.moveAlongTrack`, transcribed.
fn move_along_track(
    cart: &mut MinecartState,
    world: &dyn CollisionWorld,
    rail_pos: Pos,
    rail: Rail,
    obstacles: &[([f64; 3], [f64; 3])],
) {
    let x = cart.pos[0];
    let z = cart.pos[2];
    let before = rail_snap(world, cart.pos[0], cart.pos[1], cart.pos[2]);
    let mut y = f64::from(rail_pos.y);

    let powered = rail.powered_rail && rail.powered;
    let braking = rail.powered_rail && !rail.powered;

    // Water fifths the slope pull; carts in water are not captured yet, so the
    // dry constant stands alone here.
    let slope = SLOPE_ACCELERATION;
    match rail.shape {
        RailShape::AscendingEast => {
            cart.vel[0] -= slope;
            y += 1.0;
        }
        RailShape::AscendingWest => {
            cart.vel[0] += slope;
            y += 1.0;
        }
        RailShape::AscendingNorth => {
            cart.vel[2] += slope;
            y += 1.0;
        }
        RailShape::AscendingSouth => {
            cart.vel[2] -= slope;
            y += 1.0;
        }
        _ => {}
    }

    let (first, second) = rail.shape.exits();
    let mut dx = f64::from(second[0] - first[0]);
    let mut dz = f64::from(second[2] - first[2]);
    let length = (dx * dx + dz * dz).sqrt();
    let dot = cart.vel[0] * dx + cart.vel[2] * dz;
    if dot < 0.0 {
        dx = -dx;
        dz = -dz;
    }
    let speed = jmin(horizontal(cart.vel), 2.0);
    cart.vel = [speed * dx / length, cart.vel[1], speed * dz / length];

    // (The rider kick-start lives here in vanilla; this engine has no riders.)

    if braking {
        if horizontal(cart.vel) < 0.03 {
            cart.vel = [0.0; 3];
        } else {
            cart.vel[0] *= 0.5;
            cart.vel[1] = 0.0;
            cart.vel[2] *= 0.5;
        }
    }

    // Clamp the cart onto the exit chord.
    let fx = f64::from(rail_pos.x) + 0.5 + f64::from(first[0]) * 0.5;
    let fz = f64::from(rail_pos.z) + 0.5 + f64::from(first[2]) * 0.5;
    let sx = f64::from(rail_pos.x) + 0.5 + f64::from(second[0]) * 0.5;
    let sz = f64::from(rail_pos.z) + 0.5 + f64::from(second[2]) * 0.5;
    let cx = sx - fx;
    let cz = sz - fz;
    let t = if cx == 0.0 {
        z - f64::from(rail_pos.z)
    } else if cz == 0.0 {
        x - f64::from(rail_pos.x)
    } else {
        ((x - fx) * cx + (z - fz) * cz) * 2.0
    };
    cart.pos = [fx + cx * t, y, fz + cz * t];

    // Riderless factor is 1.0; the 0.75 applies with a passenger.
    move_cart(
        cart,
        world,
        [
            cart.vel[0].clamp(-MAX_SPEED, MAX_SPEED),
            0.0,
            cart.vel[2].clamp(-MAX_SPEED, MAX_SPEED),
        ],
        obstacles,
    );

    // Corner fixups: the cart crossed onto a sloped exit's block.
    if first[1] != 0
        && (cart.pos[0].floor() as i32) - rail_pos.x == first[0]
        && (cart.pos[2].floor() as i32) - rail_pos.z == first[2]
    {
        cart.pos[1] += f64::from(first[1]);
    } else if second[1] != 0
        && (cart.pos[0].floor() as i32) - rail_pos.x == second[0]
        && (cart.pos[2].floor() as i32) - rail_pos.z == second[2]
    {
        cart.pos[1] += f64::from(second[1]);
    }

    // applyNaturalSlowdown: empty-cart 0.96 (0.997 ridden), vy zeroed on rail.
    cart.vel[0] *= 0.96;
    cart.vel[1] = 0.0;
    cart.vel[2] *= 0.96;

    // Height correction against the snapped rail line.
    let after = rail_snap(world, cart.pos[0], cart.pos[1], cart.pos[2]);
    if let (Some(before), Some(after)) = (before, after) {
        let lift = (before[1] - after[1]) * 0.05;
        let hs = horizontal(cart.vel);
        if hs > 0.0 {
            let factor = (hs + lift) / hs;
            cart.vel[0] *= factor;
            cart.vel[2] *= factor;
        }
        cart.pos[1] = after[1];
    }

    // Crossing out of the rail's column redirects velocity at the neighbour.
    let bx = cart.pos[0].floor() as i32;
    let bz = cart.pos[2].floor() as i32;
    if bx != rail_pos.x || bz != rail_pos.z {
        let hs = horizontal(cart.vel);
        cart.vel[0] = hs * f64::from(bx - rail_pos.x);
        cart.vel[2] = hs * f64::from(bz - rail_pos.z);
    }

    if powered {
        let hs = horizontal(cart.vel);
        if hs > 0.01 {
            cart.vel[0] += cart.vel[0] / hs * 0.06;
            cart.vel[2] += cart.vel[2] / hs * 0.06;
        } else {
            match rail.shape {
                RailShape::EastWest => {
                    if world.is_conductor(Pos::new(rail_pos.x - 1, rail_pos.y, rail_pos.z)) {
                        cart.vel[0] = 0.02;
                    } else if world.is_conductor(Pos::new(rail_pos.x + 1, rail_pos.y, rail_pos.z)) {
                        cart.vel[0] = -0.02;
                    }
                }
                RailShape::NorthSouth => {
                    if world.is_conductor(Pos::new(rail_pos.x, rail_pos.y, rail_pos.z - 1)) {
                        cart.vel[2] = 0.02;
                    } else if world.is_conductor(Pos::new(rail_pos.x, rail_pos.y, rail_pos.z + 1)) {
                        cart.vel[2] = -0.02;
                    }
                }
                _ => {}
            }
        }
    }
}

fn horizontal(vel: [f64; 3]) -> f64 {
    (vel[0] * vel[0] + vel[2] * vel[2]).sqrt()
}

/// The push search inflates the hitbox by this on X and Z (`inflate(0.2, 0, 0.2)`),
/// so two carts shove each other out to 0.98 + 0.2 = 1.18 apart.
pub const PUSH_INFLATE: f64 = 0.2;

/// The push impulse, and the single most important constant here: vanilla
/// writes `0.05F`, a **float** literal, widened to double. That is
/// `0.05000000074505806`, not `0.05` — and the difference is not cosmetic.
/// With plain `0.05` the `cart_collide` golden reproduces to about 1e-8 and
/// drifts; with the float literal it reproduces **bit for bit** across all 80
/// ticks, which is how the constant was identified in the first place.
const PUSH_SCALE: f64 = 0.05_f32 as f64;

/// Two carts only shove each other when the line between them is within this
/// much of one cart's facing, as `|dot| >= 0.8` — about 37°.
const PUSH_ALIGNMENT: f64 = 0.8;

/// Whether `a`'s push search box reaches `b` — `getBoundingBox().inflate(0.2, 0, 0.2)`
/// against `b`'s box, vanilla's strict `AABB.intersects`.
fn push_boxes_overlap(a: &MinecartState, b: &MinecartState) -> bool {
    let (amin, amax) = cart_aabb(a.pos);
    let (bmin, bmax) = cart_aabb(b.pos);
    let inflate = [PUSH_INFLATE, 0.0, PUSH_INFLATE];
    (0..3).all(|axis| amin[axis] - inflate[axis] < bmax[axis] && amax[axis] + inflate[axis] > bmin[axis])
}

/// `AbstractMinecart.push(Entity)` for a cart pushing a cart.
///
/// # The law
///
/// ```text
/// n     = normalize(other.pos - this.pos)   in XZ, then scaled by min(1/dist, 1)
/// n    *= 0.05F
/// gate  = |dot(normalize(other.pos - this.pos), (cos yaw, 0, sin yaw))| >= 0.8
/// mid   = (this.vel + other.vel) / 2
/// this.vel  = this.vel  * 0.2 + (mid - n)
/// other.vel = other.vel * 0.2 + (mid + n)
/// ```
///
/// **This does not conserve momentum, it amplifies it**, and that is not a bug
/// in the transcription — it is the documented mechanism the record doors abuse.
/// Each cart keeps a fifth of its own velocity *and* is handed the full average
/// of the pair, so the sum comes out larger than it went in: the `cart_collide`
/// golden shows 0.1100 becoming 0.1460 (+33%) on one collision and 0.0984
/// becoming 0.1103 (+12%) on another. Iterate that on a slope and the velocity
/// saturates to ±Infinity; collide a +Infinity cart with a -Infinity one and
/// the `mid` term computes `(+Inf + -Inf)/2` = NaN. That is where nan carts
/// come from, and why nothing here may clamp, guard or sanitise a velocity.
///
/// # Reading it against Java
///
/// Every comparison is written so NaN falls the way Java drops it:
/// `if (sq >= 1.0E-4)` skips a NaN separation, `if (scale > 1.0)` leaves a NaN
/// scale alone, and `if (dot < 0.8) return` does **not** return on NaN, so a
/// NaN-positioned cart still pushes. Do not "simplify" any of these into
/// `f64::min`/`max` or `clamp`, which discard NaN where Java propagates it.
fn push(this: &mut MinecartState, other: &mut MinecartState) {
    let raw_x = other.pos[0] - this.pos[0];
    let raw_z = other.pos[2] - this.pos[2];
    let sq = raw_x * raw_x + raw_z * raw_z;
    // Java: `if (d2 >= 1.0E-4)`. NaN fails it, so a NaN separation pushes
    // nothing. Written negated on purpose — `sq < 1.0e-4` is *not* the same
    // thing, because it is false for NaN and would let the push through.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    if !(sq >= 1.0e-4) {
        return;
    }
    let dist = sq.sqrt();
    let mut d0 = raw_x / dist;
    let mut d1 = raw_z / dist;
    let mut scale = 1.0 / dist;
    // Java: `if (d3 > 1.0) d3 = 1.0`. NaN fails it and stays NaN — deliberately
    // not `scale.min(1.0)`, which would launder it.
    if scale > 1.0 {
        scale = 1.0;
    }
    d0 *= scale;
    d1 *= scale;
    d0 *= PUSH_SCALE;
    d1 *= PUSH_SCALE;

    // The alignment gate. `cart_yaw` pins this: two carts at rest 0.98 apart
    // along +Z, one lane holding yaw 0 and one holding yaw 90, and only the
    // yaw-90 lane ever moves. The yaw-0 lane sits there for all 80 ticks.
    let length = (raw_x * raw_x + raw_z * raw_z).sqrt();
    let (ux, uz) = (raw_x / length, raw_z / length);
    let radians = (this.yaw as f32) * (std::f32::consts::PI / 180.0);
    let (fx, fz) = (f64::from(mth_cos(radians)), f64::from(mth_sin(radians)));
    let facing = (fx * fx + fz * fz).sqrt();
    let dot = (ux * (fx / facing) + uz * (fz / facing)).abs();
    // Java: `if (d6 < 0.8F) return`. NaN fails it, so the push goes ahead —
    // matching, not correcting, the game.
    if dot < PUSH_ALIGNMENT {
        return;
    }

    let (tx, tz) = (this.vel[0], this.vel[2]);
    let (ox, oz) = (other.vel[0], other.vel[2]);
    let mid_x = (ox + tx) / 2.0;
    let mid_z = (oz + tz) / 2.0;
    // Vanilla writes this as two calls each, and the split is observable
    // through `set_delta`: the scale lands, the add is refused.
    //   this.setDeltaMovement(v.multiply(0.2, 1.0, 0.2));
    //   this.push(mid - d0, 0.0, mid - d1);   // = setDeltaMovement(v.add(..))
    set_delta(this, [tx * 0.2, this.vel[1], tz * 0.2]);
    set_delta(this, [this.vel[0] + (mid_x - d0), this.vel[1], this.vel[2] + (mid_z - d1)]);
    set_delta(other, [ox * 0.2, other.vel[1], oz * 0.2]);
    set_delta(other, [other.vel[0] + (mid_x + d0), other.vel[1], other.vel[2] + (mid_z + d1)]);
}

/// `Entity.setDeltaMovement`: **a non-finite vector is silently dropped.**
///
/// ```java
/// public void setDeltaMovement(Vec3 vec3) {
///     if (vec3.isFinite()) this.deltaMovement = vec3;
/// }
/// ```
///
/// `Vec3.isFinite` is all-or-nothing over the three components, so one NaN
/// refuses the whole write and the previous velocity stands. Two consequences,
/// both load-bearing for the record doors and both measured rather than
/// reasoned:
///
/// * **A NaN cart stays NaN.** Every arithmetic path out of a NaN velocity is
///   itself non-finite, so every attempt to overwrite it is refused. NaN is a
///   fixed point — which is exactly why the builders can use one as glue.
/// * **NaN does not spread.** A finite cart colliding with a NaN one computes a
///   NaN mean, and that write is refused too, leaving it with the 0.2 scaling
///   that landed just before. The oracle capture shows a striker going
///   0.069 -> 0.0027 across one tick, which is 0.2 squared: two pushes, each
///   keeping the multiply and dropping the add. The "zombie minecart" in
///   `docs/history/entity-abuse-in-record-doors.md` is folklore, and the document says
///   the oracle wins.
fn set_delta(cart: &mut MinecartState, velocity: [f64; 3]) {
    if velocity.iter().all(|component| component.is_finite()) {
        cart.vel = velocity;
    }
}

/// The push half of `AbstractMinecart.tick` for the cart at `index`: every other
/// cart its inflated box reaches gets `entity.push(this)` — note the direction,
/// the *found* entity is the receiver `this` and the ticking cart is the
/// argument. For two ordinary carts the law is symmetric in the pair, so the
/// direction is invisible; it would stop being with a furnace cart, which is
/// not implemented.
///
/// Runs immediately after [`tick_minecart`] for the same cart, and reads the
/// cart's **post-move** position. The `cart_collide` golden settles that order
/// on its own: the first tick's numbers only come out right if the pusher has
/// already moved when it pushes.
///
/// # Chains
///
/// Returns how many carts were pushed. Nothing here special-cases two, and it
/// does not need to: what makes a *chain* behave unlike a pair is not the push
/// at all, it is that carts block each other's movement — see
/// [`tick_minecart_among`]. A cart shoved into a neighbour it is already flush
/// against does not move and has that axis zeroed, so it pushes its own
/// neighbours from a standstill instead of from the velocity it was handed.
///
/// That was worth chasing rather than approximating. `cart_group` puts a pair,
/// a triple and a quad of touching carts on one line; vanilla moves only the
/// far cart of each group on the first tick, by 1, 1.25, 1.3375 and (in
/// `cart_chain`, five carts) 1.368125 times the impulse — a geometric series in
/// 0.35, which is the push matrix's own 0.7 self-retention times its 0.5
/// transfer. An exhaustive search over every interleaving of moves and pushes
/// showed no composition of the two-body law could reach that state at all, and
/// the missing ingredient turned out to be the collision.
///
/// The whole family now reproduces vanilla bit for bit: `cart_collide`,
/// `cart_offrail`, `cart_yaw`, `cart_group` (9 carts), `cart_chain` (6),
/// `cart_triad` (11) and `cart_gap` (21).
#[must_use]
pub fn push_neighbours(carts: &mut [MinecartState], index: usize) -> usize {
    if carts[index].removed {
        return 0;
    }
    let mut pushed = 0;
    for other in 0..carts.len() {
        if other == index || carts[other].removed {
            continue;
        }
        if !push_boxes_overlap(&carts[index], &carts[other]) {
            continue;
        }
        pushed += 1;
        let (this, arg) = pair_mut(carts, other, index);
        push(this, arg);
    }
    pushed
}

/// Two distinct elements of a slice, mutably.
fn pair_mut<T>(slice: &mut [T], a: usize, b: usize) -> (&mut T, &mut T) {
    assert_ne!(a, b);
    if a < b {
        let (left, right) = slice.split_at_mut(b);
        (&mut left[a], &mut right[0])
    } else {
        let (left, right) = slice.split_at_mut(a);
        (&mut right[0], &mut left[b])
    }
}

/// `Math.min` with **Java's** semantics: NaN propagates.
///
/// Rust's `f64::min` implements IEEE-754 `minNum`, which *discards* NaN and
/// returns the other operand; Java's `Math.min` returns NaN if either operand
/// is NaN. On ordinary numbers they agree, so the difference is invisible
/// until a NaN arrives — and in the record piston doors NaN velocities are the
/// mechanism, not an error. A cart whose velocity is NaN must stay NaN: that
/// is what freezes it, because `Entity.move` gates on
/// `lengthSqr() > 1.0E-7`, and `NaN > 1.0E-7` is false, so the move never
/// happens. Laundering the NaN into 2.0 here would hand that cart a real
/// speed and set the whole contraption walking.
fn jmin(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a < b {
        a
    } else {
        b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exits_match_the_static_initialiser() {
        assert_eq!(RailShape::NorthSouth.exits(), ([0, 0, -1], [0, 0, 1]));
        assert_eq!(RailShape::EastWest.exits(), ([-1, 0, 0], [1, 0, 0]));
        assert_eq!(RailShape::AscendingEast.exits(), ([-1, -1, 0], [1, 0, 0]));
        assert_eq!(RailShape::AscendingWest.exits(), ([-1, 0, 0], [1, -1, 0]));
        assert_eq!(RailShape::AscendingNorth.exits(), ([0, 0, -1], [0, -1, 1]));
        assert_eq!(RailShape::AscendingSouth.exits(), ([0, -1, -1], [0, 0, 1]));
        assert_eq!(RailShape::SouthEast.exits(), ([0, 0, 1], [1, 0, 0]));
        assert_eq!(RailShape::SouthWest.exits(), ([0, 0, 1], [-1, 0, 0]));
        assert_eq!(RailShape::NorthWest.exits(), ([0, 0, -1], [-1, 0, 0]));
        assert_eq!(RailShape::NorthEast.exits(), ([0, 0, -1], [1, 0, 0]));
    }

    /// Nothing but a rail line at y = 0: no floor at all, so anything with
    /// working physics falls.
    struct RailOnly;

    impl crate::entity::CollisionWorld for RailOnly {
        fn is_solid(&self, _pos: Pos) -> bool {
            false
        }
        fn friction(&self, _pos: Pos) -> f32 {
            0.6
        }
        fn rail(&self, pos: Pos) -> Option<Rail> {
            (pos.y == 0).then_some(Rail {
                shape: RailShape::EastWest,
                powered_rail: false,
                powered: false,
            })
        }
    }

    /// A "nan cart" holds still forever, and stays NaN.
    ///
    /// This is the glue the record 3x3 door is built on: carts driven to
    /// ±Infinity on sloped rails, then collided so that `+Inf + -Inf` yields
    /// NaN. `Entity.move` only applies a movement when
    /// `lengthSqr() > 1.0E-7`, and every comparison against NaN is false, so
    /// the cart never moves and nothing but a piston can shift it. The world
    /// `55_3x3.zip` carries four of them, saved with `Motion` z = NaN.
    ///
    /// The trap this guards is `f64::min`: Rust returns the non-NaN operand
    /// where Java's `Math.min` propagates NaN, so the speed projection would
    /// quietly hand a dead cart a real 2.0 and set it moving.
    #[test]
    fn a_nan_cart_never_moves_and_stays_nan() {
        let mut cart = MinecartState {
            id: 0,
            kind: "minecraft:minecart".into(),
            pos: [0.5, 0.0625, 0.5],
            vel: [f64::NAN, 0.0, 0.0],
            on_ground: false,
            on_rails: true,
            removed: false,
            inventory: None,
            fuse: None,
            yaw: 0.0,
        };
        // The first tick seats the cart on the rail chord — vanilla writes that
        // position unconditionally, whatever the velocity — so stability is
        // measured from there.
        tick_minecart(&mut cart, &RailOnly);
        let seated = cart.pos;
        for _ in 0..100 {
            tick_minecart(&mut cart, &RailOnly);
        }
        assert_eq!(cart.pos, seated, "a nan cart must not move");
        // Now asserted, where it once could not be. `move_cart` used to zero a
        // clipped axis by writing the field, which laundered a NaN cart into an
        // ordinary stationary one; it goes through `set_delta` now, and 26.2's
        // `setDeltaMovement` refuses any write whose result is non-finite. So
        // the NaN is not merely undisturbed, it is unreachable — a hundred
        // ticks of collision cannot clear it.
        assert!(
            cart.vel[0].is_nan(),
            "the nan must survive, or the cart stops being glue"
        );
    }

    /// The same rail, with an ordinary velocity, does move — so the test above
    /// is measuring NaN and not a world that simply cannot move anything.
    #[test]
    fn an_ordinary_cart_on_that_same_rail_does_move() {
        let mut cart = MinecartState {
            id: 0,
            kind: "minecraft:minecart".into(),
            pos: [0.5, 0.0625, 0.5],
            vel: [0.3, 0.0, 0.0],
            on_ground: false,
            on_rails: true,
            removed: false,
            inventory: None,
            fuse: None,
            yaw: 0.0,
        };
        tick_minecart(&mut cart, &RailOnly);
        assert!(cart.pos[0] > 0.5, "a finite cart moves along the rail");
    }

    /// `jmin` is Java's `Math.min`, not Rust's.
    #[test]
    fn jmin_propagates_nan_where_rust_would_discard_it() {
        assert!(jmin(f64::NAN, 2.0).is_nan());
        assert!(jmin(2.0, f64::NAN).is_nan());
        // Rust's own min is the behaviour we must *not* have here.
        assert_eq!(f64::NAN.min(2.0), 2.0);
        // Finite operands agree with both.
        assert_eq!(jmin(1.0, 2.0), 1.0);
        assert_eq!(jmin(2.0, 1.0), 1.0);
        // Infinity clamps like an ordinary large number, as Java does.
        assert_eq!(jmin(f64::INFINITY, 2.0), 2.0);
    }

    fn parked(id: u32, x: f64, z: f64, yaw: f64) -> MinecartState {
        MinecartState {
            id,
            kind: "minecraft:minecart".into(),
            pos: [x, 1.0625, z],
            vel: [0.0; 3],
            on_ground: false,
            on_rails: true,
            removed: false,
            inventory: None,
            fuse: None,
            yaw,
        }
    }

    /// One push phase over `carts`, cart 0 doing the pushing.
    fn shove(carts: &mut [MinecartState]) -> usize {
        push_neighbours(carts, 0)
    }

    /// Two carts touching at 0.98 shove themselves apart, symmetrically, by the
    /// `0.05F` impulse — and it really is the float literal, not 0.05.
    ///
    /// This is the first tick of the `cart_collide` golden, where the pair sits
    /// at 8.5 and 9.48 with no input and comes apart anyway.
    #[test]
    fn a_touching_pair_pushes_itself_apart_by_the_float_impulse() {
        let mut carts = vec![parked(0, 8.5, 1.5, 0.0), parked(1, 9.48, 1.5, 0.0)];
        assert_eq!(shove(&mut carts), 1);
        // Vanilla's `0.05F` widened, exactly: 0.05000000074505806.
        assert_eq!(carts[0].vel[0], -(0.05_f32 as f64));
        assert_eq!(carts[1].vel[0], 0.05_f32 as f64);
        // and it is *not* the double 0.05 — the difference is what makes the
        // golden reproduce bit for bit instead of drifting at 1e-8.
        assert_ne!(carts[1].vel[0], 0.05_f64);
    }

    /// The collision adds momentum instead of conserving it.
    ///
    /// Each cart keeps a fifth of its own velocity *and* is handed the pair's
    /// full average, so the sum comes out bigger than it went in — and by an
    /// exact factor. Summing the pair, the ±impulse cancels and the averages
    /// add back to the whole, leaving `sum' = 0.2 x sum + sum = 1.2 x sum`,
    /// whatever the velocities and however far apart the carts are.
    ///
    /// A full tick applies this twice, once in each cart's push phase, with the
    /// 0.96 rail drag in between — which is the +33% the `cart_collide` golden
    /// shows when the rammer lands, 0.1100 becoming 0.1460.
    ///
    /// Compounding 1.2 a tick is the road to ±Infinity, and from there to the
    /// NaN carts the record doors are glued with, so this must not be "fixed"
    /// into a conserving collision.
    #[test]
    fn colliding_carts_amplify_momentum_by_exactly_a_fifth() {
        let mut carts = vec![parked(0, 8.5, 1.5, 0.0), parked(1, 9.48, 1.5, 0.0)];
        carts[0].vel[0] = 0.13812576058731485; // the golden's t18 rammer
        carts[1].vel[0] = -0.02808734124118818; // and the cart it caught
        let before = carts[0].vel[0] + carts[1].vel[0];
        assert_eq!(shove(&mut carts), 1);
        let after = carts[0].vel[0] + carts[1].vel[0];
        assert!(
            (after - before * 1.2).abs() < 1.0e-15,
            "one push must multiply total momentum by 1.2: {before} -> {after}"
        );

        // It is a property of the law, not of these two numbers: at rest the
        // pair still gains, from nothing, in opposite directions.
        let mut rest = vec![parked(0, 8.5, 1.5, 0.0), parked(1, 9.48, 1.5, 0.0)];
        assert_eq!(shove(&mut rest), 1);
        assert!(rest[0].vel[0] < 0.0 && rest[1].vel[0] > 0.0);
    }

    /// A cart pushes only along its facing: the alignment gate.
    ///
    /// `cart_yaw` is two identical north-south lanes, each holding a pair
    /// parked 0.98 apart along +Z. The lane whose carts carry yaw 90 shoves
    /// itself apart exactly like the east-west pairs do; the lane carrying
    /// yaw 0 sits there, untouched, for all 80 ticks of the capture. The gate
    /// is `|dot(direction, facing)| >= 0.8`, and a +Z separation against an
    /// +X facing scores 0.
    #[test]
    fn carts_only_push_along_their_facing() {
        // Facing +Z, separated along +Z: aligned, so they shove.
        let mut aligned = vec![parked(0, 2.5, 7.5, 90.0), parked(1, 2.5, 8.48, 90.0)];
        assert_eq!(shove(&mut aligned), 1);
        assert_eq!(aligned[0].vel[2], -(0.05_f32 as f64));
        assert_eq!(aligned[1].vel[2], 0.05_f32 as f64);

        // Facing +X, separated along +Z: crosswise, so nothing happens at all.
        let mut crosswise = vec![parked(0, 2.5, 7.5, 0.0), parked(1, 2.5, 8.48, 0.0)];
        // The box search still finds it — the gate is inside the push, not the
        // search, which is why the count is 1 and the velocities are still 0.
        assert_eq!(shove(&mut crosswise), 1);
        assert_eq!(crosswise[0].vel, [0.0; 3], "yaw 0 must not push along Z");
        assert_eq!(crosswise[1].vel, [0.0; 3], "yaw 0 must not push along Z");
    }

    /// Carts further apart than hitbox + 0.2 never touch.
    #[test]
    fn the_push_search_reaches_exactly_one_hitbox_plus_two_tenths() {
        let mut near = vec![parked(0, 0.5, 0.5, 0.0), parked(1, 0.5 + 1.17, 0.5, 0.0)];
        assert_eq!(shove(&mut near), 1);
        assert!(near[1].vel[0] > 0.0);

        let mut far = vec![parked(0, 0.5, 0.5, 0.0), parked(1, 0.5 + 1.19, 0.5, 0.0)];
        assert_eq!(shove(&mut far), 0);
        assert_eq!(far[1].vel, [0.0; 3]);
    }

    /// A cart cannot move into a cart it is already flush against.
    ///
    /// This is what makes a chain a chain. The middle cart of a row spaced at
    /// 0.98 — the hitbox width, so the boxes touch with nothing between them —
    /// is handed the full impulse and still goes nowhere, and its velocity on
    /// that axis is zeroed. It then pushes its own neighbours from rest, which
    /// is where the far cart's 1.25x impulse in `cart_group` comes from.
    ///
    /// The control is the same test one hitbox further apart, where the cart
    /// has room and takes it.
    #[test]
    fn a_cart_is_stopped_by_a_cart_it_is_flush_against() {
        // RailOnly's track is at y = 0, so everything sits on the rail surface.
        let blocker = cart_aabb([9.48, 0.0625, 0.5]);
        let mut cart = parked(0, 8.5, 0.5, 0.0);
        cart.pos[1] = 0.0625;
        cart.vel[0] = 0.05;
        tick_minecart_blocked(&mut cart, &RailOnly, &[blocker]);
        assert_eq!(cart.pos[0], 8.5, "flush against a cart, so it cannot move");
        assert_eq!(cart.vel[0], 0.0, "and the blocked axis is zeroed");

        // Same push, same everything, one hitbox further out: it moves.
        let far = cart_aabb([10.48, 0.0625, 0.5]);
        let mut free = parked(1, 8.5, 0.5, 0.0);
        free.pos[1] = 0.0625;
        free.vel[0] = 0.05;
        tick_minecart_blocked(&mut free, &RailOnly, &[far]);
        assert!(free.pos[0] > 8.5, "with room, the same cart travels");
    }

    /// A cart with only part of the gap it wants takes exactly that much.
    ///
    /// `cart_gap` measures this in the game at 0.99 spacing and gets
    /// 0.009999981 rather than 0.01 — the difference being the float slop in
    /// `EntityType.MINECART`'s `0.98F` width, which is why [`CART_HALF_WIDTH`]
    /// is derived from the float and not written as 0.49.
    #[test]
    fn a_squeezed_cart_moves_by_exactly_the_gap() {
        // The geometry of `cart_gap`'s 0.99 lane, so the number below is the
        // game's own rather than this engine agreeing with itself.
        let blocker = cart_aabb([25.48, 0.0625, 0.5]);
        let mut cart = parked(0, 24.49, 0.5, 0.0);
        cart.pos[1] = 0.0625;
        cart.vel[0] = 0.05;
        tick_minecart_blocked(&mut cart, &RailOnly, &[blocker]);
        let travelled = cart.pos[0] - 24.49;

        // What vanilla did, out of `cart_gap.json`: 24.49 -> 24.499999980926514.
        const VANILLA: f64 = 0.009999980926515661;
        assert!(
            (travelled - VANILLA).abs() < 1.0e-12,
            "must clip to vanilla's gap {VANILLA}, got {travelled}"
        );
        // Write the hitbox as a round 0.49 and this comes out 0.0100000000000016
        // instead — 1.9e-8 away, far outside the tolerance above. That gap is
        // the entire evidence for the float width, and it is *below* the 1e-6
        // the conformance diff uses, so this unit test is the only thing that
        // holds it.
        assert!(
            (travelled - 0.01).abs() > 1.0e-9,
            "0.01 exactly would mean the hitbox was read as a decimal 0.98"
        );
    }

    /// A NaN cart stays NaN, and does **not** infect what hits it.
    ///
    /// The folklore — and `docs/history/entity-abuse-in-record-doors.md`, which says to
    /// believe the oracle over itself — has a NaN cart turning whatever touches
    /// it into a "zombie minecart". The capture refutes it, and 26.2's
    /// `Entity.setDeltaMovement` says why: a non-finite vector is dropped, so
    /// the NaN mean never reaches the striker. What lands instead is the 0.2
    /// scaling from the line before, twice a tick — the oracle watched a
    /// striker go 0.069 to 0.0027, which is 0.2 squared, and stay finite for
    /// the next 24 ticks.
    ///
    /// The NaN cart itself is untouched, because every write aimed at it is
    /// non-finite too. That is what makes it glue: inert, unmovable by any
    /// entity, and impossible to clear by accident.
    #[test]
    fn a_nan_cart_stays_nan_and_does_not_infect_the_cart_that_hits_it() {
        let mut carts = vec![parked(0, 8.5, 1.5, 0.0), parked(1, 9.48, 1.5, 0.0)];
        carts[0].vel[0] = f64::NAN;
        carts[1].vel[0] = 0.069;
        assert_eq!(shove(&mut carts), 1);
        assert!(carts[0].vel[0].is_nan(), "the nan cart must stay nan");
        assert!(
            carts[1].vel[0].is_finite(),
            "and must NOT infect the cart it hit: contagion is refuted"
        );
        // One push keeps the multiply and refuses the add.
        assert_eq!(carts[1].vel[0], 0.069 * 0.2);
    }

    /// The same thing a whole tick at a time: 0.2 per push, two pushes, and the
    /// oracle's 0.0027.
    #[test]
    fn a_striker_loses_a_fifth_of_its_speed_per_push_against_a_nan_cart() {
        let mut carts = vec![parked(0, 8.5, 1.5, 0.0), parked(1, 9.48, 1.5, 0.0)];
        carts[0].vel[0] = f64::NAN;
        carts[1].vel[0] = 0.069;
        // Each cart's tick runs a push phase, so a touching pair interacts twice.
        assert_eq!(shove(&mut carts), 1);
        assert_eq!(push_neighbours(&mut carts, 1), 1);
        let expected = 0.069 * 0.2 * 0.2;
        assert!(
            (carts[1].vel[0] - expected).abs() < 1.0e-12,
            "0.2 squared, the oracle's 0.069 -> 0.0027: got {}",
            carts[1].vel[0]
        );
        assert!(carts[0].vel[0].is_nan());
    }

    /// A NaN *position* pushes nothing: `sq >= 1e-4` is false for NaN in Java
    /// too, so the whole push is skipped rather than producing NaN velocities.
    #[test]
    fn a_nan_position_skips_the_push_the_way_java_does() {
        let mut carts = vec![parked(0, f64::NAN, 1.5, 0.0), parked(1, 9.48, 1.5, 0.0)];
        assert_eq!(shove(&mut carts), 0, "a nan box intersects nothing");
        assert_eq!(carts[1].vel, [0.0; 3]);
    }

    #[test]
    fn rail_shapes_parse_their_property_values() {
        assert_eq!(RailShape::from_name("east_west"), Some(RailShape::EastWest));
        assert_eq!(
            RailShape::from_name("ascending_north"),
            Some(RailShape::AscendingNorth)
        );
        assert_eq!(RailShape::from_name("weird"), None);
    }
}

use crate::behaviour::{BlockBehaviour, TickCtx};
use crate::components::{PowerSource, StatePair};
use crate::pos::Dir;

/// A golden rail's block behaviour: `PoweredRailBlock.updateState`.
///
/// The rail is powered when any neighbour signals it directly, or when a
/// chain of **already-powered** golden rails of compatible shape leads — in
/// at most 8 steps — to one that is. Chains conduct through powered rails
/// only, which is why a long line lights up as a cascade of neighbour
/// updates, one rail per wave, all inside one tick's propagation.
pub struct PoweredRail<P: PowerSource> {
    /// Which rail block this is — `minecraft:powered_rail` or
    /// `minecraft:activator_rail`.
    ///
    /// `isSameRailWithPower` tests `state.is(this)`, so a chain conducts only
    /// between rails of the *same* block: an activator rail beside a golden one
    /// does not extend it. Activator rails run this exact update — there is no
    /// separate class, only a second registration of `PoweredRailBlock`.
    pub block: &'static str,
    /// This state's shape.
    pub shape: RailShape,
    /// This state's `powered` flag.
    pub powered: bool,
    /// The unpowered/powered pair.
    pub states: StatePair,
    /// The world's power rules.
    pub power: P,
}

impl<P: PowerSource> PoweredRail<P> {
    fn has_neighbor_signal(&self, ctx: &TickCtx<'_>, pos: Pos) -> bool {
        crate::pos::ALL_DIRS
            .iter()
            .any(|dir| self.power.is_powered(ctx.world, ctx.comparator_out, pos.offset(*dir), dir.opposite()))
    }

    /// The powered rail at `pos`, from its descriptor.
    fn rail_at(&self, ctx: &TickCtx<'_>, pos: Pos) -> Option<(RailShape, bool)> {
        let descriptor = ctx.states.descriptor(ctx.world.get(pos))?;
        if !descriptor.starts_with(self.block) {
            return None;
        }
        let shape = descriptor
            .split("shape=")
            .nth(1)
            .and_then(|rest| RailShape::from_name(rest.split([',', ']']).next()?))?;
        Some((shape, descriptor.contains("powered=true")))
    }

    /// `findPoweredRailSignal`: walk one rail along the line and recurse.
    fn find_signal(
        &self,
        ctx: &TickCtx<'_>,
        pos: Pos,
        shape: RailShape,
        forward: bool,
        distance: u32,
    ) -> bool {
        if distance >= 8 {
            return false;
        }
        let (mut x, mut y, mut z) = (pos.x, pos.y, pos.z);
        let mut descend = true;
        let mut expect = shape;
        match shape {
            RailShape::NorthSouth => {
                if forward {
                    z += 1;
                } else {
                    z -= 1;
                }
            }
            RailShape::EastWest => {
                if forward {
                    x -= 1;
                } else {
                    x += 1;
                }
            }
            RailShape::AscendingEast => {
                if forward {
                    x -= 1;
                } else {
                    x += 1;
                    y += 1;
                    descend = false;
                }
                expect = RailShape::EastWest;
            }
            RailShape::AscendingWest => {
                if forward {
                    x -= 1;
                    y += 1;
                    descend = false;
                } else {
                    x += 1;
                }
                expect = RailShape::EastWest;
            }
            RailShape::AscendingNorth => {
                if forward {
                    z += 1;
                } else {
                    z -= 1;
                    y += 1;
                    descend = false;
                }
                expect = RailShape::NorthSouth;
            }
            RailShape::AscendingSouth => {
                if forward {
                    z += 1;
                    y += 1;
                    descend = false;
                } else {
                    z -= 1;
                }
                expect = RailShape::NorthSouth;
            }
            // Golden rails are never curves.
            _ => {}
        }
        let stepped = Pos::new(x, y, z);
        if self.same_rail_with_power(ctx, stepped, forward, distance, expect) {
            return true;
        }
        descend
            && self.same_rail_with_power(
                ctx,
                Pos::new(x, y - 1, z),
                forward,
                distance,
                expect,
            )
    }

    /// `isSameRailWithPower`: shape-compatible, already powered, and fed.
    fn same_rail_with_power(
        &self,
        ctx: &TickCtx<'_>,
        pos: Pos,
        forward: bool,
        distance: u32,
        expect: RailShape,
    ) -> bool {
        let Some((shape, powered)) = self.rail_at(ctx, pos) else {
            return false;
        };
        let incompatible = match expect {
            RailShape::EastWest => matches!(
                shape,
                RailShape::NorthSouth | RailShape::AscendingNorth | RailShape::AscendingSouth
            ),
            RailShape::NorthSouth => matches!(
                shape,
                RailShape::EastWest | RailShape::AscendingEast | RailShape::AscendingWest
            ),
            _ => false,
        };
        if incompatible || !powered {
            return false;
        }
        self.has_neighbor_signal(ctx, pos) || self.find_signal(ctx, pos, shape, forward, distance + 1)
    }
}

impl<P: PowerSource + 'static> BlockBehaviour for PoweredRail<P> {
    /// `BaseRailBlock.onPlace` ends in the same `updateState` the neighbour
    /// path runs — a rail a command block sets next to a redstone block must
    /// light up on placement, not wait for a poke that may never come.
    fn on_placed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        self.on_neighbor_changed(ctx, pos, Dir::Up);
    }

    /// The same hook off a *write* — `LevelChunk.setBlockState` runs
    /// `onPlace` for a `/setblock` too, and that is the only poke a rail a
    /// command placed ever gets. Idempotent when the write was the rail's
    /// own powered flip: the recomputed target equals the state just
    /// written.
    fn on_state_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos) {
        self.on_neighbor_changed(ctx, pos, Dir::Up);
    }

    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
        if crate::components::rail_pops_off(&self.power, ctx, pos, self.block) {
            return;
        }
        let target = self.has_neighbor_signal(ctx, pos)
            || self.find_signal(ctx, pos, self.shape, true, 0)
            || self.find_signal(ctx, pos, self.shape, false, 0);
        if target == self.powered {
            return;
        }
        ctx.set(pos, if target { self.states.on } else { self.states.off });
        // updateState also updates the neighbours of the block below (and
        // above, on slopes) — how the change reaches components hanging off
        // the rail's support block.
        ctx.update_neighbors_at(pos.offset(Dir::Down));
        if self.shape.is_ascending() {
            ctx.update_neighbors_at(pos.offset(Dir::Up));
        }
    }

    fn name(&self) -> &'static str {
        self.block
    }
}
