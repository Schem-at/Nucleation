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
/// The cart hitbox: 0.98 × 0.7 (`EntityType.MINECART`).
pub const CART_HALF_WIDTH: f64 = 0.49;
/// The cart hitbox height.
pub const CART_HEIGHT: f64 = 0.7;

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
fn move_cart(cart: &mut MinecartState, world: &dyn CollisionWorld, movement: [f64; 3]) {
    let (min, max) = cart_aabb(cart.pos);
    let (clipped, hit) = crate::entity::collide_move(world, min, max, movement);
    let sqr = clipped[0] * clipped[0] + clipped[1] * clipped[1] + clipped[2] * clipped[2];
    if sqr > 1.0e-7 {
        cart.pos[0] += clipped[0];
        cart.pos[1] += clipped[1];
        cart.pos[2] += clipped[2];
    }
    cart.on_ground = hit[1] && movement[1] < 0.0;
    if hit[0] {
        cart.vel[0] = 0.0;
    }
    if hit[1] {
        cart.vel[1] = 0.0;
    }
    if hit[2] {
        cart.vel[2] = 0.0;
    }
}

/// One cart tick — `OldMinecartBehavior.tick`, server side, riderless.
pub fn tick_minecart(cart: &mut MinecartState, world: &dyn CollisionWorld) {
    cart.vel[1] -= CART_GRAVITY;
    let rail_pos = rail_block_pos(cart, world);
    match world.rail(rail_pos) {
        Some(rail) => {
            cart.on_rails = true;
            move_along_track(cart, world, rail_pos, rail);
        }
        None => {
            cart.on_rails = false;
            come_off_track(cart, world);
        }
    }
}

/// `comeOffTrack`: clamp, halve when grounded, move, air-drag when airborne.
fn come_off_track(cart: &mut MinecartState, world: &dyn CollisionWorld) {
    cart.vel[0] = cart.vel[0].clamp(-MAX_SPEED, MAX_SPEED);
    cart.vel[2] = cart.vel[2].clamp(-MAX_SPEED, MAX_SPEED);
    if cart.on_ground {
        for axis in &mut cart.vel {
            *axis *= 0.5;
        }
    }
    move_cart(cart, world, cart.vel);
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
    let speed = horizontal(cart.vel).min(2.0);
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
    fn on_neighbor_changed(&self, ctx: &mut TickCtx<'_>, pos: Pos, _from: Dir) {
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
