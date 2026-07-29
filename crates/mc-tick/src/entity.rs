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
    /// Whether the last move ended on the ground (`Entity.onGround`).
    pub on_ground: bool,
    /// Ticks since spawn (`Entity.tickCount`), incremented at tick start.
    pub tick_count: u32,
    /// `ItemEntity.age`; despawn at [`ITEM_LIFETIME`].
    pub age: u32,
    /// Player-pickup delay. Tracked for fidelity; hoppers ignore it.
    pub pickup_delay: u32,
    /// Set when discarded; removed (and reported) at end of tick.
    pub removed: bool,
    /// `Entity.stuckSpeedMultiplier`, armed by a cobweb: the next move is
    /// scaled by it per axis and the velocity zeroed, then it clears.
    pub stuck: Option<[f64; 3]>,
}

/// A non-item entity as *blocks* see it: an id, a kind, and a world-space box.
///
/// Detector rails and weighted pressure plates ask the world "what is standing
/// here?", and the honest answer includes entities this engine does not
/// otherwise simulate — minecarts, and later fireballs and villagers, which
/// exist in the record doors purely as hitboxes. Mirroring their boxes here
/// keeps that question answerable through the single entity handle a
/// [`crate::behaviour::TickCtx`] already carries, rather than threading a new
/// borrow through forty context literals.
///
/// This is a *view*, rebuilt from the owning entity lists — never the
/// authoritative copy of an entity's position.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityBody {
    /// Trace-stable entity id.
    pub id: u32,
    /// Registry name, e.g. `minecraft:furnace_minecart`.
    pub kind: String,
    /// Box minimum corner.
    pub min: [f64; 3],
    /// Box maximum corner.
    pub max: [f64; 3],
    /// Whether this is an `AbstractMinecart`. Detector rails select on exactly
    /// this class (`DetectorRailBlock.checkPressed` passes
    /// `AbstractMinecart.class` to `getInteractingMinecartOfType`), so a
    /// fireball or a villager standing on one must *not* power it.
    pub is_minecart: bool,
}

/// An entity type's hitbox, as `EntityType.sized(width, height)` registered it.
///
/// Read out of the game's own registry (`tools/gametest/src/EntityDims.java`
/// prints `EntityType.getDimensions()` after `Bootstrap.bootStrap`), so these
/// cannot disagree with the game the way a remembered number can.
///
/// These are mechanism, not trivia. The record 3x3 door uses a **dragon**
/// fireball where a small one will not do, and the registry says why: a dragon
/// fireball is exactly one block tall, so resting at the bottom of a cell it
/// spans the whole cell — reaching a pressure plate at the floor *and* the
/// piston above. A small fireball is 5/16 and reaches neither.
///
/// Verified against the oracle in `fireball_reach.json`, which walks both
/// fireballs across a weighted plate's touch box and finds the edge exactly
/// where these widths put it: the dragon registers at 0.90 from centre and not
/// at 0.95, the small one at 0.55 and not at 0.65.
pub fn entity_dimensions(kind: &str) -> Option<(f64, f64)> {
    Some(match kind {
        // Every minecart variant shares one hitbox — furnace, chest, hopper and
        // TNT carts included. A furnace minecart is dimensionally an ordinary
        // cart.
        // `EntityDimensions.scalable(0.98F, 0.7F)` — float literals, so the
        // real box is 0.9800000190734863 by 0.699999988079071. The width's
        // eighth decimal is observable now that carts clip against each other:
        // `cart_gap` measures a squeezed approach as 0.009999981, which is the
        // float width and not the decimal one.
        "minecraft:minecart"
        | "minecraft:furnace_minecart"
        | "minecraft:chest_minecart"
        | "minecraft:hopper_minecart"
        | "minecraft:tnt_minecart" => (0.98_f32 as f64, 0.7_f32 as f64),
        "minecraft:dragon_fireball" | "minecraft:fireball" => (1.0, 1.0),
        "minecraft:small_fireball" => (0.3125, 0.3125),
        // `EntityType.VILLAGER` is `sized(0.6F, 1.95F)`, and both literals are
        // **floats**: 0.6000000238418579 by 1.9500000476837158. Written as
        // decimals until a cart could stand on one, at which point the eighth
        // decimal became observable and wrong — `blaze_ride_ai` rests a cart on
        // a villager at exactly `2.950000047683716`, which is 1.0 + 1.95f and
        // not 1.0 + 1.95. The width is float for the same reason the cart's is
        // (see above) even though no capture separates 0.6 from 0.6f yet.
        "minecraft:villager" => (0.6_f32 as f64, 1.95_f32 as f64),
        // The record 3x3 door's two riders. Registry says `sized(0.6F, 1.8F)`,
        // and `blaze_reach.entities.log` walks a blaze across a weighted plate's
        // touch box at twelve offsets and agrees at all twelve: clear at 1.76
        // and 11.24, touching at 5.77 and 15.23, which bounds the half-width in
        // (0.2925, 0.3025). The four baby-villager offsets are the cross-check —
        // a 0.49-wide body reads clear at 17.81 and 27.19 and a blaze reads
        // *touching*, so the width cannot be the baby's.
        //
        // Height is bounded the same way by a plate two blocks up: a blaze with
        // its feet at 1.205 reaches it and one at 1.195 does not, so the height
        // is in (1.795, 1.805). `blaze_reach_villager_control.entities.log` is
        // the negative control — the same rig, a 1.95-tall villager, and *both*
        // plates fire.
        //
        // Written as the float the registry holds rather than the decimal,
        // because the eighth decimal is observable: in `blaze_ride_ai` a cart
        // dropped onto a blaze settles at exactly 1.0 + 1.7999999523162842.
        "minecraft:blaze" => (0.6_f32 as f64, 1.8_f32 as f64),
        "minecraft:item" => (0.25, 0.25),
        // Anything else is *not* guessed. An unknown entity gets no box, and
        // the simulation refuses it by name rather than quietly giving it a
        // default hitbox — a wrong box in this corpus is a wrong door.
        _ => return None,
    })
}

/// The world-space box of an entity of `kind` standing at `pos`.
///
/// Vanilla centres an entity horizontally on its position and hangs the box
/// upward from its feet (`EntityDimensions.makeBoundingBox`). Confirmed by
/// `fireball_reach.json`: the fireballs there sit with their feet exactly at
/// plate level and register, and their horizontal edges land where a centred
/// box puts them.
pub fn body_aabb(kind: &str, pos: [f64; 3]) -> Option<([f64; 3], [f64; 3])> {
    let (width, height) = entity_dimensions(kind)?;
    let half = width / 2.0;
    Some((
        [pos[0] - half, pos[1], pos[2] - half],
        [pos[0] + half, pos[1] + height, pos[2] + half],
    ))
}

/// Whether a body of `kind` stops a **minecart** that moves into it.
///
/// `None` for a kind with no measured hitbox — unreachable from a loaded world,
/// because [`entity_dimensions`] refuses those at spawn.
///
/// This is not "living entities are solid". Ten lanes across four captures say
/// the split runs somewhere else entirely:
///
/// | body under / in front of the cart | cart | capture |
/// |---|---|---|
/// | `blaze` | **stopped** | `cart_body`, `cart_body2`, `cart_body3` |
/// | `villager` | **stopped** | `cart_body`, `cart_body2` |
/// | `zombie` | **stopped** | `cart_body` |
/// | `oak_boat` | **stopped** | `cart_body2` |
/// | `minecart` | **stopped** | `cart_body`, and five older goldens |
/// | `armor_stand` | passes through | `cart_body`, `cart_body2` |
/// | `small_fireball` | passes through | `blaze_ride_ai`, `cart_body` |
/// | `dragon_fireball` | passes through | `blaze_ride_ai`, `cart_body2` |
/// | `fireball` (ghast) | passes through | `cart_body4` |
/// | `item` | passes through | `cart_body4` |
///
/// An **armor stand is a `LivingEntity` and is transparent**; a **boat is not
/// living and is solid**. So "living" is refuted as the rule by both of its
/// edges at once. What fits all ten is vanilla's vehicle predicate — a cart's
/// collision set is `canBeCollidedWith() || isPushable()`, and an armor stand
/// answers false to both (`ArmorStand.isPushable` is overridden to `false`).
/// The table below is the measurement, not that reading; the reading is only
/// why the measurement is not two coincidences.
///
/// Two asymmetries are measured and are **not** in this function, because it
/// only answers "what stops a cart":
///
/// * A living body's *own* movement is not stopped by any of this. A blaze
///   dropped from y = 3 onto a minecart, a NaN minecart and a furnace minecart
///   lands on the floor at y = 1.0 on tick 19 in all three lanes — the same
///   tick as the empty control (`cart_body2`). Carts are transparent to a
///   falling mob. Nothing in this engine moves a mob, so there is nothing to
///   implement, but a future mob-physics pass must not reuse this table.
/// * A **rideable** cart that comes within `inflate(0.2, 0, 0.2)` of a free
///   living entity *mounts* it rather than being stopped by it: in `cart_body`
///   a plain cart rolling east picks the blaze up at t18, from 0.2 away, and
///   carries on with the ridden `0.997` slowdown instead of the empty `0.96`.
///   That is not modelled. It cannot fire on the record door — its rolling
///   stock is furnace carts, which are not rideable, and both of its blazes are
///   already passengers, which vanilla's gate excludes.
pub fn blocks_a_cart(kind: &str) -> Option<bool> {
    // Deliberately mirrors `entity_dimensions`' arms one for one, so a kind
    // cannot gain a hitbox without someone deciding what it does to a cart.
    Some(match kind {
        // Cart on cart: `cart_body` drops one onto another and it rests at
        // 1.699999988079071 — the lower cart's exact float top. The five
        // cart-cart goldens are the horizontal half of the same fact.
        "minecraft:minecart"
        | "minecraft:furnace_minecart"
        | "minecraft:chest_minecart"
        | "minecraft:hopper_minecart"
        | "minecraft:tnt_minecart" => true,
        // Every projectile measured is transparent, in both axes.
        "minecraft:dragon_fireball" | "minecraft:fireball" | "minecraft:small_fireball" => false,
        // `cart_body2`: a furnace cart rolling east stops with its east face at
        // 6.199999988079071, which is the blaze's and the villager's west face
        // to the last bit. `cart_body`/`blaze_ride_ai`: a cart dropped on them
        // rests at 2.799999952316284 and 2.950000047683716, their exact tops.
        "minecraft:villager" | "minecraft:blaze" => true,
        // `cart_body4`: an authored item on the rail, and a cart dropped on
        // one, both reproduce the empty control to the last digit.
        "minecraft:item" => false,
        _ => return None,
    })
}

/// Where a passenger sits relative to its vehicle's position, measured.
///
/// A rider is not a body with a position of its own. Vanilla's
/// `Entity.positionRider` hard-sets the passenger every tick to
/// `vehicle.position() + vehiclePassengerAttachment - riderVehicleAttachment`,
/// with no collision check, so this offset *is* the rider's position — see
/// [`crate::sim::Simulation::spawn_authored_rider`].
///
/// The two attachment points are properties of the two entity *types*, so the
/// offset is neither a constant nor derivable from the hitboxes, and this table
/// says so by refusing every pair it has not seen. `blaze_ride.entities.log`
/// measures three riders on one and the same minecart and gets two different
/// answers:
///
/// | vehicle | rider | seated y − vehicle y |
/// |---|---|---|
/// | `minecart` | `blaze` | **0.1875** |
/// | `minecart` | `small_fireball` | 0.1875 |
/// | `minecart` | `villager` | **0.0** |
///
/// A villager therefore rides *lower* than a blaze on the same cart despite
/// being taller, which rules out any rule of the form "half the vehicle's
/// height" or "derived from the rider's box". Horizontal offset is zero in all
/// three lanes, over twenty ticks and including a cart rolling east.
///
/// The blaze row is confirmed twice over: `55_3x3.zip` itself holds a blaze at
/// y = 2.2500 on a cart at 2.062 and another at 2.1875 on a cart at 2.000 —
/// 0.1875 both times.
pub fn passenger_attachment(vehicle_kind: &str, rider_kind: &str) -> Option<[f64; 3]> {
    // Only the plain cart is measured. The container and furnace variants share
    // its *hitbox*, but an attachment point is not a hitbox — none of them was
    // put under the oracle, so none of them is assumed to match.
    if vehicle_kind != "minecraft:minecart" {
        return None;
    }
    Some(match rider_kind {
        "minecraft:blaze" | "minecraft:small_fireball" => [0.0, 0.1875, 0.0],
        "minecraft:villager" => [0.0, 0.0, 0.0],
        _ => return None,
    })
}

#[cfg(test)]
mod hitbox_tests {
    use super::*;

    /// `BasePressurePlateBlock.TOUCH_AABB` at a cell, duplicated here so the
    /// geometry check does not depend on the block layer.
    fn touch(x: f64, y: f64, z: f64) -> ([f64; 3], [f64; 3]) {
        ([x + 0.0625, y, z + 0.0625], [x + 0.9375, y + 0.25, z + 0.9375])
    }

    fn presses(kind: &str, plate: [f64; 3], pos: [f64; 3]) -> bool {
        let (min, max) = body_aabb(kind, pos).expect("known entity");
        let body = EntityBody { id: 0, kind: kind.into(), min, max, is_minecart: false };
        let (tmin, tmax) = touch(plate[0], plate[1], plate[2]);
        body.intersects(tmin, tmax)
    }

    /// The twelve probes of `blaze_reach.entities.log`, replayed against the
    /// engine's own box.
    ///
    /// Nine plates on the floor walk a blaze across their touch box; the last
    /// three are the height rig. Four of the floor probes straddle the width
    /// edges at ±0.0025 and ±0.0075, so the assertion fails if the half-width is
    /// wrong by a hundredth in either direction. The four *baby-villager*
    /// offsets are the discriminator: a 0.49-wide body reads clear at 17.81 and
    /// 27.19 and vanilla read a blaze as touching both, so this cannot pass with
    /// the baby's width either.
    #[test]
    fn the_blaze_box_reproduces_the_captured_plate_probes() {
        // (plate cell x, blaze centre x, what vanilla's `power` said)
        const FLOOR: [(f64, f64, bool); 9] = [
            (2.0, 1.76, false),
            (6.0, 5.77, true),
            (10.0, 11.24, false),
            (14.0, 15.23, true),
            (18.0, 17.81, true),
            (22.0, 21.83, true),
            (26.0, 27.19, true),
            (30.0, 31.17, true),
            (34.0, 34.5, true),
        ];
        for (plate, at, expected) in FLOOR {
            assert_eq!(
                presses("minecraft:blaze", [plate, 1.0, 1.0], [at, 1.0, 1.5]),
                expected,
                "blaze at x={at} against the plate at {plate}"
            );
        }
        // The height rig: a plate two blocks up, and a blaze whose feet straddle
        // the height that just reaches it. 1.205 + h > 3.0 > 1.195 + h pins h to
        // (1.795, 1.805). `blaze_reach_villager_control.entities.log` is the
        // control — a 1.95-tall villager reaches both, so the rig can say yes.
        assert!(presses("minecraft:blaze", [44.0, 3.0, 2.0], [43.9, 1.205, 2.5]));
        assert!(!presses("minecraft:blaze", [48.0, 3.0, 2.0], [47.9, 1.195, 2.5]));
        assert!(
            presses("minecraft:villager", [48.0, 3.0, 2.0], [47.9, 1.195, 2.5]),
            "the control: a taller body must reach the plate the blaze misses, or \
             the assertion above is passing because the rig cannot reach at all"
        );
    }

    /// The seat is a property of the *pair*, and the table refuses the rest.
    ///
    /// The villager row is the control: same vehicle, taller rider, different
    /// answer. Any rule derived from the vehicle alone, or from the rider's box,
    /// would have to give these two the same offset — and vanilla does not.
    #[test]
    fn the_measured_seats_disagree_with_each_other() {
        assert_eq!(
            passenger_attachment("minecraft:minecart", "minecraft:blaze"),
            Some([0.0, 0.1875, 0.0])
        );
        assert_eq!(
            passenger_attachment("minecraft:minecart", "minecraft:villager"),
            Some([0.0, 0.0, 0.0])
        );
        // Unmeasured pairs get nothing rather than a plausible default.
        assert_eq!(passenger_attachment("minecraft:minecart", "minecraft:creeper"), None);
        assert_eq!(
            passenger_attachment("minecraft:furnace_minecart", "minecraft:blaze"),
            None,
            "a furnace cart shares the plain cart's hitbox, but an attachment point \
             is not a hitbox and this one was never put under the oracle"
        );
    }

    /// The six probes of `fireball_reach.json`, replayed against the engine's
    /// own boxes. Vanilla answered 1, 1, 0, 1, 1, 0 — the dragon fireball
    /// reaching 0.90 from a plate's centre but not 0.95, the small one 0.55 but
    /// not 0.65. Those pairs straddle the exact half-widths the registry gives
    /// (0.5 and 0.15625), so the test fails if either width is off by more than
    /// a couple of hundredths.
    #[test]
    fn fireball_boxes_reproduce_the_captured_plate_probes() {
        let plate = |x: f64| [x, 1.0, 1.0];
        let at = |x: f64| [x, 1.0, 1.5];
        assert!(presses("minecraft:dragon_fireball", plate(1.0), at(1.5)));
        assert!(presses("minecraft:dragon_fireball", plate(4.0), at(5.4)));
        assert!(!presses("minecraft:dragon_fireball", plate(7.0), at(8.45)));
        assert!(presses("minecraft:small_fireball", plate(10.0), at(10.5)));
        assert!(presses("minecraft:small_fireball", plate(13.0), at(14.05)));
        assert!(!presses("minecraft:small_fireball", plate(16.0), at(17.15)));
    }

    /// The reason the door uses a *dragon* fireball: it is exactly one block
    /// tall, so resting on a cell's floor it spans that whole cell — a plate at
    /// the bottom and the piston above. A small fireball spans 5/16 of it.
    #[test]
    fn a_dragon_fireball_spans_a_whole_block_and_a_small_one_does_not() {
        let (min, max) = body_aabb("minecraft:dragon_fireball", [0.5, 3.0, 0.5]).unwrap();
        assert_eq!(max[1] - min[1], 1.0);
        let (smin, smax) = body_aabb("minecraft:small_fireball", [0.5, 3.0, 0.5]).unwrap();
        assert!(smax[1] - smin[1] < 0.32);
    }

    /// Every cart variant is one hitbox, and it is exactly the 0.98 spacing the
    /// door's furnace-cart chain sits at.
    #[test]
    fn every_minecart_variant_shares_the_plain_cart_box() {
        for kind in [
            "minecraft:minecart",
            "minecraft:furnace_minecart",
            "minecraft:chest_minecart",
            "minecraft:hopper_minecart",
            "minecraft:tnt_minecart",
        ] {
            assert_eq!(
                entity_dimensions(kind),
                Some((0.98_f32 as f64, 0.7_f32 as f64)),
                "{kind}: the vanilla literals are floats"
            );
        }
        // The engine's cart box must agree with the generic table.
        let (min, max) = body_aabb("minecraft:minecart", [0.5, 1.0, 0.5]).unwrap();
        let (cmin, cmax) = crate::minecart::cart_aabb([0.5, 1.0, 0.5]);
        assert_eq!((min, max), (cmin, cmax));
    }

    /// An entity nobody has measured gets no box at all.
    #[test]
    fn an_unmeasured_entity_has_no_hitbox_rather_than_a_guessed_one() {
        assert_eq!(entity_dimensions("minecraft:creeper"), None);
        assert_eq!(body_aabb("minecraft:creeper", [0.0; 3]), None);
    }

    /// The collidability table, and its refusals.
    ///
    /// The two halves are asserted together on purpose: a table that answered
    /// `true` for everything would pass the first loop and fail the second, and
    /// one that answered `false` for everything the other way round. The
    /// numbers behind each row are in [`blocks_a_cart`]'s own documentation.
    #[test]
    fn only_the_measured_bodies_stop_a_cart() {
        for kind in [
            "minecraft:minecart",
            "minecraft:furnace_minecart",
            "minecraft:chest_minecart",
            "minecraft:hopper_minecart",
            "minecraft:tnt_minecart",
            "minecraft:villager",
            "minecraft:blaze",
        ] {
            assert_eq!(blocks_a_cart(kind), Some(true), "{kind} was measured solid");
        }
        for kind in [
            "minecraft:small_fireball",
            "minecraft:dragon_fireball",
            "minecraft:fireball",
            "minecraft:item",
        ] {
            assert_eq!(blocks_a_cart(kind), Some(false), "{kind} was measured transparent");
        }
        // Every kind with a hitbox has an answer here, and no kind without one
        // does — so a new entity cannot arrive with a box and no decision.
        for kind in ["minecraft:creeper", "minecraft:armor_stand", "minecraft:oak_boat"] {
            assert_eq!(blocks_a_cart(kind), None, "{kind} has no box in this engine");
            assert_eq!(entity_dimensions(kind), None);
        }
    }
}

impl EntityBody {
    /// Whether this body overlaps a box, by vanilla's strict `AABB.intersects`
    /// (touching faces do not count).
    pub fn intersects(&self, min: [f64; 3], max: [f64; 3]) -> bool {
        self.min[0] < max[0]
            && self.max[0] > min[0]
            && self.min[1] < max[1]
            && self.max[1] > min[1]
            && self.min[2] < max[2]
            && self.max[2] > min[2]
    }
}

/// All item entities plus the id counter — one bundle so the tick context
/// carries a single handle.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ItemEntities {
    /// In spawn order, which is also vanilla's entity-list iteration order.
    pub items: Vec<ItemEntityState>,
    /// Non-item entities' boxes, mirrored from the simulation's own lists so
    /// block behaviours can see every entity, not just the items. Rebuilt by
    /// [`crate::sim::Simulation::refresh_bodies`]; see [`EntityBody`].
    pub others: Vec<EntityBody>,
    /// The next id to assign.
    pub next_id: u32,
    /// Every id ever spawned with its item, surviving removal — a renderer
    /// needs to know what a vacuumed item *was*.
    pub name_log: Vec<(u32, String)>,
    /// Container contents carried *by an item* — a dropped shulker box keeps
    /// its slots. Keyed by entity id; moves with the item through hoppers.
    pub contents: std::collections::HashMap<u32, Vec<crate::inventory::ItemStack>>,
    /// The seeded random source, when the simulation opted in
    /// ([`crate::sim::Simulation::set_rng_seed`]). Lives here rather than on
    /// `TickCtx` because every consumer today — dispense jitter, dispenser
    /// slot choice, destroy drops — is an item-spawning path that already
    /// holds this handle. `None` keeps every behaviour on its deterministic
    /// mean, which is what the conformance goldens were recorded against.
    pub rng: Option<crate::rng::JavaRandom>,
}

impl ItemEntities {
    /// Spawn with an explicit id — the id participates in vanilla's rest-flush
    /// phase (`(tickCount + id) % 4`), so a conformance run must use the
    /// server's captured entity ids or its flush ticks land differently.
    pub fn spawn_with_id(
        &mut self,
        id: u32,
        item: (String, u8),
        pos: [f64; 3],
        vel: [f64; 3],
        pickup_delay: u32,
    ) -> u32 {
        self.next_id = self.next_id.max(id + 1);
        self.push_spawn(id, item, pos, vel, pickup_delay);
        id
    }

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
        self.push_spawn(id, item, pos, vel, pickup_delay);
        id
    }

    fn push_spawn(
        &mut self,
        id: u32,
        item: (String, u8),
        pos: [f64; 3],
        vel: [f64; 3],
        pickup_delay: u32,
    ) {
        self.name_log.push((id, item.0.clone()));
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
            stuck: None,
        });
    }
}

/// What item physics needs to know about the world.
pub trait CollisionWorld {
    /// Whether the block at `pos` is a full collision cube.
    fn is_solid(&self, pos: Pos) -> bool;
    /// `Block.getFriction` of the block at `pos` (0.6 for almost everything).
    fn friction(&self, pos: Pos) -> f32;
    /// The water at `pos`, if any (`getFluidState`) — waterlogged blocks and
    /// bubble columns included.
    fn water(&self, _pos: Pos) -> Option<crate::fluid::WaterKind> {
        None
    }
    /// The bubble column at `pos`: `Some(drag_down)` — `true` pulls entities
    /// down (magma), `false` pushes them up (soul sand).
    fn bubble(&self, _pos: Pos) -> Option<bool> {
        None
    }
    /// Whether `pos` is literally air — the bubble-column top check
    /// (`BubbleColumnBlock.entityInside` reads the state *above*).
    fn is_air(&self, _pos: Pos) -> bool {
        false
    }
    /// The collision-box height of a solid block at `pos` (1.0 for full
    /// cubes; soul sand answers 0.875).
    fn solid_height(&self, _pos: Pos) -> f64 {
        1.0
    }
    /// Whether `pos` is a cobweb (`WebBlock.entityInside`).
    fn is_web(&self, _pos: Pos) -> bool {
        false
    }
    /// The rail at `pos`, if any — what cart physics runs on.
    fn rail(&self, _pos: Pos) -> Option<crate::minecart::Rail> {
        None
    }
    /// `isRedstoneConductor` — the powered-rail launch check reads it.
    fn is_conductor(&self, _pos: Pos) -> bool {
        false
    }
}

/// The web's stuck-speed multiplier, `WebBlock.entityInside`'s
/// `Vec3(0.25, 0.05f, 0.25)`.
const WEB_MULTIPLIER: [f64; 3] = [0.25, 0.05000000074505806, 0.25];

/// Scan the cells the item's box overlaps and re-arm the stuck multiplier if
/// any is a cobweb — the `checkInsideBlocks` dispatch that runs with (and, via
/// `applyEffectsFromBlocksForLastMovements`, without) a move.
fn apply_webs(entity: &mut ItemEntityState, world: &dyn CollisionWorld) {
    let (min, max) = item_aabb(entity.pos);
    const EPSILON: f64 = 1.0e-7;
    for x in ((min[0] + EPSILON).floor() as i32)..=((max[0] - EPSILON).floor() as i32) {
        for y in ((min[1] + EPSILON).floor() as i32)..=((max[1] - EPSILON).floor() as i32) {
            for z in ((min[2] + EPSILON).floor() as i32)..=((max[2] - EPSILON).floor() as i32) {
                if world.is_web(Pos::new(x, y, z)) {
                    entity.stuck = Some(WEB_MULTIPLIER);
                    return;
                }
            }
        }
    }
}

/// The `EntityFluidInteraction.update` scan for water: the max surface height
/// above the box floor and the accumulated flow currents, walked over every
/// cell the interaction box (bounding box deflated 0.001) overlaps in
/// vanilla's x-outer/z-inner order.
fn water_interaction(entity: &ItemEntityState, world: &dyn CollisionWorld) -> (f64, [f64; 3]) {
    let (bmin, bmax) = item_aabb(entity.pos);
    let min = [bmin[0] + 0.001, bmin[1] + 0.001, bmin[2] + 0.001];
    let max = [bmax[0] - 0.001, bmax[1] - 0.001, bmax[2] - 0.001];
    let mut height = 0.0f64;
    let mut flow_sum = [0.0f64; 3];
    let water_at = |pos: Pos| world.water(pos);
    let solid_at = |pos: Pos| world.is_solid(pos);
    for x in (min[0].floor() as i32)..=((max[0].ceil() as i32) - 1) {
        for y in (min[1].floor() as i32)..=((max[1].ceil() as i32) - 1) {
            for z in (min[2].floor() as i32)..=((max[2].ceil() as i32) - 1) {
                let cell = Pos::new(x, y, z);
                let Some(kind) = world.water(cell) else { continue };
                let above = world.water(cell.offset(crate::pos::Dir::Up)).is_some();
                let surface =
                    f64::from(y) + f64::from(crate::fluid::surface_height(kind, above));
                if surface < min[1] {
                    continue;
                }
                // The skip check uses the deflated box; the height itself is
                // measured from the **raw** bounding-box floor (the bytecode
                // stores both, and the 0.001 difference decides the float-or-
                // sink branch in blocks-deep shallows).
                height = height.max(surface - bmin[1]);
                let mut flow = crate::fluid::flow_vector(&water_at, &solid_at, cell);
                if height < 0.4 {
                    for axis in &mut flow {
                        *axis *= height;
                    }
                }
                for (sum, axis) in flow_sum.iter_mut().zip(flow) {
                    *sum += axis;
                }
            }
        }
    }
    (height, flow_sum)
}

/// `Tracker.applyCurrentTo` for a non-player entity: normalize the summed
/// current and push with `waterPushSpeed` 0.014. Below the 1e-5 length² floor
/// nothing happens.
fn apply_water_current(entity: &mut ItemEntityState, flow_sum: [f64; 3]) {
    let length_sqr =
        flow_sum[0] * flow_sum[0] + flow_sum[1] * flow_sum[1] + flow_sum[2] * flow_sum[2];
    if length_sqr < 1.0e-5 {
        return;
    }
    let length = length_sqr.sqrt();
    if length < 1.0e-4 {
        return;
    }
    for (vel, axis) in entity.vel.iter_mut().zip(flow_sum) {
        *vel += axis / length * 0.014;
    }
}

/// `Entity.onInsideBubbleColumn` / `onAboveBubbleColumn`, dispatched for every
/// bubble-column cell the item's box overlaps — the clamps come straight from
/// the bytecode.
fn apply_bubble_columns(entity: &mut ItemEntityState, world: &dyn CollisionWorld) {
    let (min, max) = item_aabb(entity.pos);
    const EPSILON: f64 = 1.0e-7;
    for x in ((min[0] + EPSILON).floor() as i32)..=((max[0] - EPSILON).floor() as i32) {
        for y in ((min[1] + EPSILON).floor() as i32)..=((max[1] - EPSILON).floor() as i32) {
            for z in ((min[2] + EPSILON).floor() as i32)..=((max[2] - EPSILON).floor() as i32) {
                let cell = Pos::new(x, y, z);
                let Some(drag_down) = world.bubble(cell) else { continue };
                let vy = entity.vel[1];
                entity.vel[1] = if world.is_air(cell.offset(crate::pos::Dir::Up)) {
                    if drag_down {
                        (vy - 0.03).max(-0.9)
                    } else {
                        (vy + 0.1).min(1.8)
                    }
                } else if drag_down {
                    (vy - 0.03).max(-0.3)
                } else {
                    (vy + 0.06).min(0.7)
                };
            }
        }
    }
}

/// Advance one item entity by one tick. Returns `true` while it lives.
pub fn tick_item(entity: &mut ItemEntityState, world: &dyn CollisionWorld) -> bool {
    entity.tick_count += 1;
    if entity.pickup_delay > 0 && entity.pickup_delay != 32767 {
        entity.pickup_delay -= 1;
    }

    // Entity.baseTick → EntityFluidInteraction: fluid heights and current
    // pushing happen before the gravity/buoyancy decision. ItemEntity.tick
    // runs the same update **again** after movement and drag (so a second
    // 0.014 push lands every tick — the stream capture's velocities prove
    // both, exactly).
    let (water_height, flow_sum) = water_interaction(entity, world);
    apply_water_current(entity, flow_sum);

    if water_height > f64::from(0.1f32) {
        // setUnderwaterMovement: horizontal drag 0.99f, and a nudge of 5e-4f
        // upward while rising slower than 0.06f — items float, slowly.
        entity.vel[0] *= f64::from(0.99f32);
        entity.vel[2] *= f64::from(0.99f32);
        if entity.vel[1] < f64::from(0.06f32) {
            entity.vel[1] += f64::from(5.0e-4f32);
        }
    } else {
        entity.vel[1] -= ITEM_GRAVITY;
    }

    let horizontal_sqr = entity.vel[0] * entity.vel[0] + entity.vel[2] * entity.vel[2];
    let resting = entity.on_ground
        && horizontal_sqr <= 1.0e-5
        && (entity.tick_count + entity.id) % 4 != 0;
    if resting {
        // Even a rest-skipped tick re-applies block effects from the last
        // movements (`applyEffectsFromBlocksForLastMovements`): a sunk item in
        // a drag column keeps its velocity pinned downward, which is why the
        // flush tick never lifts it off the floor.
        apply_bubble_columns(entity, world);
        apply_webs(entity, world);
    } else {
        // Entity.move: an armed stuck multiplier scales this move per axis and
        // zeroes the velocity, then clears — a cobweb re-arms it every tick
        // the box still touches it.
        let movement = match entity.stuck.take() {
            Some(multiplier) => {
                let scaled = [
                    entity.vel[0] * multiplier[0],
                    entity.vel[1] * multiplier[1],
                    entity.vel[2] * multiplier[2],
                ];
                entity.vel = [0.0; 3];
                scaled
            }
            None => entity.vel,
        };
        move_with_collision(entity, world, movement);
        // Bubble columns and webs act during the move (`checkInsideBlocks`),
        // before drag.
        apply_bubble_columns(entity, world);
        apply_webs(entity, world);
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

    // The second fluid pass, `updateFluidInteraction()` near the tail of
    // ItemEntity.tick: heights and currents recomputed at the post-move
    // position, and the current push applied again.
    let (_, flow_sum) = water_interaction(entity, world);
    apply_water_current(entity, flow_sum);

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

/// Move with axis-clipped collision against solid blocks: Y first, then the
/// larger horizontal axis — `Entity.collideBoundingBox`'s order. A clipped
/// axis zeroes its velocity component; a downward Y clip sets `on_ground`.
/// `movement` is usually the velocity, but a stuck multiplier scales it.
fn move_with_collision(
    entity: &mut ItemEntityState,
    world: &dyn CollisionWorld,
    movement: [f64; 3],
) {
    let (mut min, mut max) = item_aabb(entity.pos);
    let mut movement = movement;

    let attempted_y = movement[1];
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

    // Entity.move only applies the position when the collided movement is
    // longer than sqrt(1e-7) — a gliding item's last sub-millimetre drifts
    // never land, which is what finally parks it. Collision flags and the
    // velocity zeroing below still run either way.
    let sqr = movement[0] * movement[0] + movement[1] * movement[1] + movement[2] * movement[2];
    if sqr > 1.0e-7 {
        entity.pos[0] += movement[0];
        entity.pos[1] += movement[1];
        entity.pos[2] += movement[2];
    }

    // Vanilla zeroes collided components and derives onGround from a downward
    // vertical collision of the attempted movement.
    entity.on_ground = clipped_y && attempted_y < 0.0;
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

/// `Entity.collideBoundingBox` for any box: clip `movement` (Y first, then
/// the larger horizontal axis) and report which axes hit. Shared by items and
/// minecarts.
/// The same sweep, plus a list of other entities' boxes to clip against.
///
/// `Entity.collide` feeds `level.getEntityCollisions(this, box.expandTowards(v))`
/// into the same sweep the blocks go through, so an entity that collides with
/// entities is stopped by them exactly as it is stopped by a wall. Minecarts do,
/// and it turns out to be the whole story behind chains of touching carts: a
/// cart shoved into a neighbour it is already flush against does not move at
/// all, and has that axis of its velocity zeroed.
pub(crate) fn collide_move_among(
    world: &dyn CollisionWorld,
    mut min: [f64; 3],
    mut max: [f64; 3],
    movement: [f64; 3],
    obstacles: &[([f64; 3], [f64; 3])],
) -> ([f64; 3], [bool; 3]) {
    let mut movement = movement;
    let mut hit = [false; 3];
    let clipped = clip_boxes(min, max, 1, clip_axis(world, min, max, 1, movement[1]), obstacles);
    hit[1] = clipped != movement[1];
    min[1] += clipped;
    max[1] += clipped;
    movement[1] = clipped;
    let x_first = movement[0].abs() > movement[2].abs();
    let order: [usize; 2] = if x_first { [0, 2] } else { [2, 0] };
    for &axis in &order {
        let clipped = clip_boxes(
            min,
            max,
            axis,
            clip_axis(world, min, max, axis, movement[axis]),
            obstacles,
        );
        hit[axis] = hit[axis] || clipped != movement[axis];
        min[axis] += clipped;
        max[axis] += clipped;
        movement[axis] = clipped;
    }
    (movement, hit)
}

/// Clip a single-axis movement against other entities' boxes — `VoxelShape.collide`.
///
/// The `1e-7` slack is vanilla's, and it is load-bearing rather than cosmetic:
/// two carts parked at exactly 0.98 have boxes that miss touching by about
/// 1.8e-15 one way or 1.9e-8 the other, purely from the float arithmetic in
/// `0.98F`, and only a tolerance this size makes them reliably block each other.
fn clip_boxes(
    min: [f64; 3],
    max: [f64; 3],
    axis: usize,
    mut delta: f64,
    obstacles: &[([f64; 3], [f64; 3])],
) -> f64 {
    const EPSILON: f64 = 1.0e-7;
    for (omin, omax) in obstacles {
        if delta.abs() < EPSILON {
            return 0.0;
        }
        // A box only blocks this axis if it overlaps on the other two.
        let clear = (0..3)
            .filter(|other| *other != axis)
            .any(|other| min[other] >= omax[other] - EPSILON || max[other] <= omin[other] + EPSILON);
        if clear {
            continue;
        }
        // Written so NaN takes neither branch, as it does in Java: `f64::min`
        // and `max` discard NaN where this must propagate it.
        if delta > 0.0 {
            let room = omin[axis] - max[axis];
            if room >= -EPSILON && room < delta {
                delta = room;
            }
        } else if delta < 0.0 {
            let room = omax[axis] - min[axis];
            if room <= EPSILON && room > delta {
                delta = room;
            }
        }
    }
    delta
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
                let cell = Pos::new(x, y, z);
                if !world.is_solid(cell) {
                    continue;
                }
                let block_min = [f64::from(x), f64::from(y), f64::from(z)];
                let block_max = [
                    block_min[0] + 1.0,
                    // Partial-height solids (soul sand: 14/16) top out early.
                    block_min[1] + world.solid_height(cell),
                    block_min[2] + 1.0,
                ];
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
    // An item carrying container contents (a dropped shulker box) never merges:
    // vanilla's `areMergable` fails on the component mismatch, and shulker
    // boxes stack to 1 anyway.
    if entities.contents.contains_key(&entities.items[index].id) {
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
        if entities.contents.contains_key(&other.id) {
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
            stuck: None,
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
