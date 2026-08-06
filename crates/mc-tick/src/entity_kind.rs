//! What one *kind* of entity is, as a registry row.
//!
//! The block half of this engine has had a shape since the beginning: a
//! [`crate::behaviour::BlockBehaviour`] trait, a
//! [`crate::behaviour::BehaviourTable`] that dispatches to it, and every
//! registration in one place in [`crate::vanilla`]. Adding a block is one
//! registration.
//!
//! Entities grew the other way — one type at a time, each chasing a specific
//! build — and ended up as four parallel collections keyed by matched-on
//! strings. This module is the same shape as the block side, deliberately: a
//! trait, a table, and one row per type registered beside the blocks.
//!
//! # What a row has to say
//!
//! Five things, and they are five because each one was a separate scattered
//! per-type lookup before:
//!
//! * **dimensions** — `EntityType.sized(width, height)`, read out of the game's
//!   own registry rather than remembered (see [`EntityBehaviour::dimensions`]).
//! * **the vehicle predicate** — whether a minecart driving into this body is
//!   stopped by it. Measured, not derived; see
//!   [`EntityBehaviour::obstructs_a_cart`].
//! * **passenger attachment** — where a rider of a given kind sits on this one,
//!   for the pairs that have been measured.
//! * **motion semantics** — [`EntityMotion`], which also answers "does it tick
//!   physics".
//!
//! # Refusing by name
//!
//! A kind with no row has no hitbox, no physics and no seat, and every path
//! that meets one refuses it *by name* rather than inventing a default. That is
//! the same contract the old hand-written tables had — each of them ended in a
//! `_ => return None` — except that now there is one place to add the row
//! instead of four places to forget.

use std::collections::HashMap;

/// How an entity of some kind moves — and whether it moves at all.
///
/// This is a property of the *type*, not of an instance: whether a particular
/// blaze happens to be riding a minecart is instance state, and lives with the
/// instance. What the type decides is which physics, if any, exist for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityMotion {
    /// `ItemEntity.tick` — gravity, drag, collision and merging.
    /// See [`crate::entity::tick_item`].
    Item,
    /// `AbstractMinecart.tick` — rails, slopes, pushes and cart-on-cart
    /// collision. See [`crate::minecart`].
    Minecart,
    /// No physics at all: the entity is a hitbox that holds its position.
    ///
    /// Not a simplification for its own sake. The record doors' fireballs are
    /// caught mid-flight by a piston-and-cobweb trick and have zero motion, so
    /// standing still *is* what the game does with them; the same is true of a
    /// `noai` blaze or villager standing on a plate. What this engine has no
    /// answer for is one of these with real velocity, and every spawn path
    /// refuses that case rather than freezing something that should be moving.
    ///
    /// A frozen body is still displaced by a piston arm — that is the one force
    /// in the engine that does not go through the entity's own physics.
    Frozen,
}

impl EntityMotion {
    /// Whether this kind has physics of its own that run every tick.
    pub fn ticks_physics(self) -> bool {
        !matches!(self, Self::Frozen)
    }
}

/// How one kind of entity behaves.
///
/// The entity-side mirror of [`crate::behaviour::BlockBehaviour`]. Dispatch is
/// by registry name rather than by `StateId` because that is the identity an
/// entity actually has — a `Passengers` compound in a save names its rider with
/// a string, and so does every capture.
pub trait EntityBehaviour: Send + Sync {
    /// The registry name this behaviour is registered under, e.g.
    /// `minecraft:furnace_minecart`.
    fn kind(&self) -> &'static str;

    /// The type's hitbox, as `EntityType.sized(width, height)` registered it.
    ///
    /// Read out of the game's own registry (`tools/gametest/src/EntityDims.java`
    /// prints `EntityType.getDimensions()` after `Bootstrap.bootStrap`), so
    /// these cannot disagree with the game the way a remembered number can.
    ///
    /// These are mechanism, not trivia. The record 3x3 door uses a **dragon**
    /// fireball where a small one will not do, and the registry says why: a
    /// dragon fireball is exactly one block tall, so resting at the bottom of a
    /// cell it spans the whole cell — reaching a pressure plate at the floor
    /// *and* the piston above. A small fireball is 5/16 and reaches neither.
    fn dimensions(&self) -> (f64, f64);

    /// Whether a **minecart** that moves into this body is stopped by it.
    ///
    /// This is not "living entities are solid". Ten bodies were measured, each
    /// twice — dropped on from above and driven into sideways — across
    /// `cart_body`..`cart_body4` and `blaze_ride_ai`:
    ///
    /// | body | cart |
    /// |---|---|
    /// | `blaze`, `villager`, `zombie`, `oak_boat`, `minecart` | **stopped** |
    /// | `armor_stand`, `small_fireball`, `dragon_fireball`, `fireball`, `item` | passes through |
    ///
    /// An **armor stand is a `LivingEntity` a cart falls through**; a **boat is
    /// not living and holds one up**. So "living" is refuted as the rule by both
    /// of its edges at once. What fits all ten is vanilla's *vehicle* predicate
    /// — a cart's collision set is `canBeCollidedWith() || isPushable()`, and an
    /// armor stand answers false to both (`ArmorStand.isPushable` is overridden
    /// to `false`). That reading is why the table is not ten coincidences; the
    /// measurement is still what this method returns.
    ///
    /// Two asymmetries are measured and are **not** here, because this only
    /// answers "what stops a cart":
    ///
    /// * A living body's *own* movement is not stopped by any of this. A blaze
    ///   dropped onto a minecart, a NaN minecart and a furnace minecart lands on
    ///   the floor at y = 1.0 on tick 19 in all three lanes — the same tick as
    ///   the empty control (`cart_body2`). Carts are transparent to a falling
    ///   mob. Nothing in this engine moves a mob, so there is nothing to
    ///   implement, but a future mob-physics pass must not reuse this.
    /// * A **rideable** cart that comes within `inflate(0.2, 0, 0.2)` of a free
    ///   living entity *mounts* it rather than being stopped by it: in
    ///   `cart_body` a plain cart rolling east picks the blaze up at t18, from
    ///   0.2 away, and carries on with the ridden `0.997` slowdown instead of
    ///   the empty `0.96`. That is not modelled. It cannot fire on the record
    ///   door — its rolling stock is furnace carts, which are not rideable, and
    ///   both of its blazes are already passengers, which vanilla's gate
    ///   excludes.
    fn obstructs_a_cart(&self) -> bool;

    /// This kind's motion semantics.
    fn motion(&self) -> EntityMotion;

    /// Where a passenger of kind `rider` sits on *this* entity, when the pair
    /// has been measured.
    ///
    /// A rider is not a body with a position of its own. Vanilla's
    /// `Entity.positionRider` hard-sets the passenger every tick to
    /// `vehicle.position() + vehiclePassengerAttachment - riderVehicleAttachment`,
    /// with no collision check, so this offset *is* the rider's position — see
    /// [`crate::sim::Simulation::spawn_authored_rider`].
    ///
    /// The two attachment points are properties of the two entity *types*, so
    /// the offset is neither a constant nor derivable from the hitboxes, and
    /// this refuses every pair it has not seen. `blaze_ride.entities.log`
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
    /// height" or "derived from the rider's box". Horizontal offset is zero in
    /// all three lanes, over twenty ticks and including a cart rolling east.
    ///
    /// The blaze row is confirmed twice over: `55_3x3.zip` itself holds a blaze
    /// at y = 2.2500 on a cart at 2.062 and another at 2.1875 on a cart at
    /// 2.000 — 0.1875 both times.
    fn seat_for(&self, _rider: &str) -> Option<[f64; 3]> {
        None
    }

    /// Whether *any* rider's seat on this vehicle has been measured.
    ///
    /// Derived from the same seat list [`EntityBehaviour::seat_for`] reads, so a
    /// vehicle cannot start accepting passengers without a measurement. Today
    /// the plain minecart is the only kind that answers `true`.
    fn carries_passengers(&self) -> bool {
        false
    }

    /// Whether this is an `AbstractMinecart`.
    ///
    /// Detector rails select on exactly this class
    /// (`DetectorRailBlock.checkPressed` passes `AbstractMinecart.class` to
    /// `getInteractingMinecartOfType`), so a fireball or a villager standing on
    /// one must *not* power it.
    fn is_minecart(&self) -> bool {
        matches!(self.motion(), EntityMotion::Minecart)
    }
}

/// One registry row: an entity type described entirely by data.
///
/// The entity-side counterpart of [`crate::behaviour::Inert`] — a behaviour
/// whose whole content is what it was constructed with. Every kind this engine
/// models is one of these, which is what makes adding a type a single
/// registration rather than a new impl.
#[derive(Debug, Clone, Copy)]
pub struct EntityKind {
    /// Registry name.
    pub name: &'static str,
    /// Hitbox width, `EntityType.sized`'s first argument.
    pub width: f64,
    /// Hitbox height, `EntityType.sized`'s second argument.
    pub height: f64,
    /// Whether a minecart is stopped by this body — see
    /// [`EntityBehaviour::obstructs_a_cart`].
    pub obstructs_a_cart: bool,
    /// Motion semantics.
    pub motion: EntityMotion,
    /// Measured passenger seats, as `(rider kind, offset)`. Empty means no pair
    /// has been measured, and every one refuses.
    pub seats: &'static [(&'static str, [f64; 3])],
}

impl EntityBehaviour for EntityKind {
    fn kind(&self) -> &'static str {
        self.name
    }

    fn dimensions(&self) -> (f64, f64) {
        (self.width, self.height)
    }

    fn obstructs_a_cart(&self) -> bool {
        self.obstructs_a_cart
    }

    fn motion(&self) -> EntityMotion {
        self.motion
    }

    fn seat_for(&self, rider: &str) -> Option<[f64; 3]> {
        self.seats
            .iter()
            .find(|(kind, _)| *kind == rider)
            .map(|(_, seat)| *seat)
    }

    fn carries_passengers(&self) -> bool {
        !self.seats.is_empty()
    }
}

/// Dispatch from a registry name to its behaviour.
///
/// The entity-side [`crate::behaviour::BehaviourTable`]. A `HashMap` rather
/// than a flat `Vec` because the key is a name and not a dense index; the
/// lookups are per-entity-per-tick rather than per-block-event, so this is not
/// the hot path the block table is.
#[derive(Default)]
pub struct EntityTable {
    entries: HashMap<&'static str, Box<dyn EntityBehaviour>>,
}

impl std::fmt::Debug for EntityTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut names: Vec<&str> = self.entries.keys().copied().collect();
        names.sort_unstable();
        f.debug_struct("EntityTable")
            .field("kinds", &names)
            .finish()
    }
}

impl EntityTable {
    /// An empty table. Nothing is known until something is registered, and a
    /// kind that is not registered is refused by name.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register `behaviour`, replacing any previous entry for its name.
    pub fn register(&mut self, behaviour: Box<dyn EntityBehaviour>) {
        self.entries.insert(behaviour.kind(), behaviour);
    }

    /// Register a plain data row. The common case, and the one that makes a new
    /// entity type a single line.
    pub fn add(&mut self, kind: EntityKind) {
        self.register(Box::new(kind));
    }

    /// The behaviour for `kind`, if registered.
    pub fn get(&self, kind: &str) -> Option<&dyn EntityBehaviour> {
        self.entries.get(kind).map(std::convert::AsRef::as_ref)
    }

    /// Whether `kind` has a behaviour.
    pub fn is_registered(&self, kind: &str) -> bool {
        self.entries.contains_key(kind)
    }

    /// Every registered kind, in a deterministic order.
    pub fn kinds(&self) -> Vec<&'static str> {
        let mut names: Vec<&'static str> = self.entries.keys().copied().collect();
        names.sort_unstable();
        names
    }
}
