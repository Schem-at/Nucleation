//! Loading Java structure SNBT into a simulation.
//!
//! This is what lets the engine and the vanilla oracle run **the same input**.
//! Until now the two were compared through hand-written corpus cases; with this
//! they consume the identical `.snbt` file, which is the only way a trace diff
//! means anything.
//!
//! # Why a parser here rather than nucleation's
//!
//! nucleation already reads structure SNBT, and duplicating that is a real cost.
//! But depending on nucleation from this crate would drag in 188k lines and 159
//! dependencies, and the ~0.7s edit-test loop is the substrate the whole project
//! rests on — it is not worth trading for the removal of one small parser. The
//! parser here is deliberately narrow: it reads the structure format and nothing
//! else, and it is not a general SNBT implementation.
//!
//! If the duplication ever becomes a maintenance problem, the fix is a small
//! shared crate that both depend on — not a dependency edge from here to
//! nucleation.
//!
//! # Format
//!
//! ```text
//! {
//!   DataVersion: 4903,
//!   size: [3, 2, 1],
//!   palette: [{Name: "minecraft:stone"}, {Name: "minecraft:repeater", Properties: {delay: "1"}}],
//!   blocks: [{pos: [0, 0, 0], state: 0}, ...],
//!   entities: []
//! }
//! ```

use crate::pos::{Bounds, Pos};
use crate::state::{StateId, StateRegistry};
use crate::world::World;

/// A parsed structure, ready to place into a world.
#[derive(Debug, Clone, PartialEq)]
pub struct Structure {
    /// Extent along each axis.
    pub size: (i32, i32, i32),
    /// The save version the text states, from the top-level `DataVersion`.
    ///
    /// Not decoration, and not only for block ids: it is the authority on which
    /// `Entity.load` rules the authored `Motion` vectors were read under, which
    /// is the difference between a nan cart and an ordinary one. See
    /// [`crate::motion::MotionSemantics`]. `None` when the text omits the tag —
    /// hand-written fixtures do, and a caller that cannot tell "the file says
    /// 4903" from "the file says nothing" cannot decide whether to fall back to
    /// its own default.
    pub data_version: Option<i32>,
    /// Block state descriptors, indexed by palette entry.
    pub palette: Vec<String>,
    /// Positions with their palette index.
    pub blocks: Vec<(Pos, usize)>,
    /// Container contents, from block-entity `nbt` `Items` lists.
    ///
    /// Only `Items` is understood; other block-entity data is skipped. The
    /// caller turns these into engine inventories at load time — the parser
    /// does not know how many slots a barrel has.
    pub inventories: Vec<(Pos, Vec<crate::inventory::ItemStack>)>,
    /// Per-container insertion restrictions.
    ///
    /// This is the crafter block entity's `disabled_slots` int array, encoded
    /// as a bit mask so an empty disabled slot survives parsing without being
    /// forged into an item stack.
    pub inventory_blocked_slots: Vec<(Pos, u16)>,
    /// Comparator block-entity output strengths, from `nbt` `OutputSignal`.
    ///
    /// What a comparator *emits* until it next re-evaluates. A door saved with
    /// its comparators mid-cycle starts from these, not from zero.
    pub comparator_outputs: Vec<(Pos, u8)>,
    /// Positions that carried an `nbt` compound — i.e. that have a block
    /// entity. `placeInWorld` ends each of these with `BlockEntity.setChanged`,
    /// which pokes neighbouring comparators, so the list is load-bearing even
    /// when the compound holds nothing we model (an empty barrel still counts).
    pub block_entities: Vec<Pos>,
    /// Command-block `Command` strings, from block-entity `nbt`. Raw text:
    /// the loader decides what subset it can run (`vanilla::parse_command`).
    pub commands: Vec<(Pos, String)>,
    /// Every authored entity, in list order — the placement spawn order,
    /// which is also the server's id-assignment order.
    pub entities: Vec<SpawnedEntity>,
    /// Item entities authored in the structure's `entities` list.
    ///
    /// The RNG-free way to put an item into the world: authored positions and
    /// motion, no dispenser jitter. This is the `minecraft:item` subset of
    /// [`Self::entities`]; the other types are reachable there.
    pub item_entities: Vec<SpawnedItem>,
}

/// One authored entity, by type.
///
/// Parsing a type and being able to *simulate* it are deliberately different
/// questions. Everything here can be read out of a structure file faithfully;
/// whether the engine has a behaviour for it is decided at construction, by
/// whoever turns a `Structure` into a `Simulation`. Keeping the two apart means
/// adding a variant never quietly weakens anything — a type with no behaviour
/// still refuses, it just refuses one step later and with a better message.
///
/// The variants are the **motion classes** of
/// [`crate::entity_kind::EntityMotion`], not the entity types, and that is the
/// compile-time half of the gate. Which types exist is a registry question,
/// answered in one place by [`crate::vanilla::entity_table`]; what the engine
/// can *do* with one is a question with a fixed, small set of answers, and
/// adding a new answer to it — real projectile physics, mob AI — has to fail to
/// compile at every site that dispatches on this.
///
/// So a new entity type of an existing motion class is one registry row and no
/// change here. A type with no row is refused **by name** by
/// [`Reader::entity_entry`], which is the runtime half: a hand-kept list of
/// "kinds we support" would drift, but a lookup that returns `None` cannot.
#[derive(Debug, Clone, PartialEq)]
pub enum SpawnedEntity {
    /// An item entity.
    Item(SpawnedItem),
    /// A plain rideable minecart.
    Minecart(SpawnedMinecart),
    /// A furnace minecart — a cart that can drive itself.
    FurnaceMinecart(SpawnedFurnaceMinecart),
    /// An entity the engine models as a frozen hitbox and nothing more:
    /// fireballs, villagers, blazes, boats, armor stands.
    Body(SpawnedBody),
}

impl SpawnedEntity {
    /// The entity id this was parsed from, for messages.
    pub fn kind(&self) -> &str {
        match self {
            Self::Item(_) => "minecraft:item",
            Self::Minecart(cart) => &cart.kind,
            Self::FurnaceMinecart(_) => "minecraft:furnace_minecart",
            Self::Body(body) => &body.kind,
        }
    }
}

/// An authored minecart.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnedMinecart {
    /// e.g. `minecraft:minecart`.
    pub kind: String,
    /// Container contents for chest/hopper carts, empty for a plain cart.
    /// The loader mirrors these where its comparator rules can read them —
    /// a parked container cart on a detector rail is a comparator input.
    pub items: Vec<crate::inventory::ItemStack>,
    /// Spawn position.
    pub pos: [f64; 3],
    /// Spawn velocity.
    pub motion: [f64; 3],
    /// `Rotation[0]` — yaw in degrees, 0 when the tag is absent, as vanilla
    /// defaults it.
    ///
    /// Carried because cart-cart pushing gates on it: two carts only shove
    /// each other when the line between them lies within ~37° of the facing,
    /// so a cart parked on a stale heading is inert to its neighbour. The
    /// `cart_yaw` capture is two identical lanes that differ in nothing but
    /// this number, and only one of them moves.
    ///
    /// Passed through unconverted, and that is deliberate. `Rotation[0]` loads
    /// straight into `yRot`, and `AbstractMinecart` reads `yRot` as a plain
    /// polar angle in XZ — it writes it as `atan2(dz, dx)` and consumes it as
    /// `(cos yaw, 0, sin yaw)` — so 0 means +X, not south. That is 90° off the
    /// compass convention every other entity uses, and it is the game's quirk,
    /// not a transcription slip. `cart_yaw` settles which reading is right: its
    /// yaw-0 lane, separated along +Z, scores a dot of 0 and never moves, while
    /// its yaw-90 lane scores 1 and shoves itself apart. Under the compass
    /// reading the two lanes would swap.
    pub yaw: f64,
    /// `Passengers` — entities this cart carries.
    ///
    /// Nested in the vehicle's own compound rather than listed alongside it, so
    /// a reader that only walks the top level under-reports the world. The
    /// record 3x3 door is exactly that case: 22 top-level entities on disk, and
    /// two `minecraft:blaze` riding two of its four plain minecarts, which is
    /// how vanilla's own capture of the save counts 24.
    ///
    /// A passenger's `Pos` in the file is where it *was* when the world saved;
    /// the engine does not use it, because vanilla re-derives a rider's position
    /// from its vehicle on the first tick — see
    /// [`crate::entity::passenger_attachment`].
    pub passengers: Vec<SpawnedEntity>,
}

/// An authored furnace minecart.
///
/// Its own variant rather than a `kind` on [`SpawnedMinecart`] because the
/// engine models the plain cart and this one does strictly more: it carries
/// fuel and a push vector and can drive itself along a rail. Sharing a variant
/// would make "can the engine simulate this?" a string comparison instead of a
/// match arm.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnedFurnaceMinecart {
    /// Spawn position.
    pub pos: [f64; 3],
    /// Spawn velocity.
    pub motion: [f64; 3],
    /// `Fuel` — ticks of self-propulsion left.
    ///
    /// Measured zero on all fifteen furnace carts in the record 3x3 door, which
    /// makes those pure mass and hitbox rather than engines. Carried anyway: a
    /// fuelled cart drives itself, and silently dropping that would mis-simulate
    /// a different build in exactly the way this whole seam exists to prevent.
    pub fuel: u32,
    /// `PushX`/`PushZ` — the drive direction. Also zero throughout that door.
    pub push: [f64; 2],
    /// `Rotation[0]` — yaw in degrees, read exactly as [`SpawnedMinecart::yaw`]
    /// is, because a furnace cart *is* an `AbstractMinecart` and its push gate
    /// is the same code.
    ///
    /// Its absence here was a silent hole with a measurable cost: every one of
    /// the record 3x3 door's fifteen furnace carts carries `Rotation: [±90, 0]`,
    /// its top row is strung out along **x**, and yaw ±90 scores a dot of zero
    /// against that separation — so vanilla never pushes them and the row is
    /// motionless. Defaulted to 0 the row scores 1, shoves itself apart on
    /// tick 2, and walks the end cart off its ledge. `cart_furnace_yaw` is the
    /// capture: two identical x-separated furnace pairs that differ in nothing
    /// but this number, and only the yaw-0 one moves.
    pub yaw: f64,
}

/// An authored entity the engine carries as a frozen hitbox and nothing more.
///
/// One struct for fireballs, villagers, blazes, boats and armor stands, because
/// what the engine does with each of them is identical — hold its box where the
/// file put it, refuse it if it was moving — and the *differences* between them
/// are entirely in [`crate::vanilla::entity_table`]. Three separate structs used
/// to spell out that sameness three times.
///
/// The `kind` is kept rather than collapsed, because the *type* is mechanism.
/// The record doors freeze fireballs mid-flight with piston-and-cobweb timing
/// and use them as pressure-plate triggers, where the size decides everything: a
/// small fireball barely clips a plate, while a dragon fireball is a full block
/// tall and can reach a plate and the piston above it at once. In the record 3x3
/// door the villager is a wall stopping a cart and a floor it lands on, and two
/// blazes ride two of its four nan carts — the builders' published account said
/// villagers did that job; the save says blazes.
///
/// Hitbox dimensions deliberately do **not** live here. They are a property of
/// the entity type, like a container's slot count, and belong to the registry —
/// a structure file carries no such field, and accepting one would let a
/// hand-edited file claim a small fireball is a block tall.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnedBody {
    /// The registry name, e.g. `minecraft:dragon_fireball`.
    pub kind: String,
    /// Spawn position. Ignored while the entity is a rider, whose position
    /// vanilla re-derives from its vehicle every tick.
    pub pos: [f64; 3],
    /// Spawn velocity.
    ///
    /// A riding blaze in 26.2 reads exactly `(0, -0.0784000015258789, 0)` every
    /// tick, forever, and never moves: `Entity.rideTick` zeroes the passenger's
    /// delta, runs its tick — one step of living-entity gravity, 0.08 × 0.98 —
    /// and then `positionRider` overwrites the position anyway. That is why the
    /// door's saved riders carry a finite gravity velocity beside vehicles whose
    /// velocity is NaN, and why the number is not evidence that they fall.
    pub motion: [f64; 3],
    /// Whether the entity compound carried a vanilla leash attachment.
    ///
    /// Both the current lowercase `leash` tag and the legacy `Leash` compound
    /// count. The target is deliberately not interpreted here: a litematic
    /// preserves a fence knot's source-world coordinates even though it stores
    /// the entity position relative to its region. The simulation only needs
    /// the attachment's presence for the narrowly supported parked-boat case.
    pub leashed: bool,
    /// Entities riding this body.
    ///
    /// A boat passenger is just as load-bearing as a minecart passenger, so it
    /// cannot be dropped merely because this motion class is represented as a
    /// frozen body. The rider's authored position is retained for diagnostics;
    /// simulation derives its live position from the measured seat.
    pub passengers: Vec<SpawnedEntity>,
}

/// One authored item entity.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnedItem {
    /// Feet-centre position, structure-relative.
    pub pos: [f64; 3],
    /// Initial velocity.
    pub motion: [f64; 3],
    /// `(item id, count)`.
    pub item: (String, u8),
    /// `PickupDelay`, default 0.
    pub pickup_delay: u32,
}

/// Why a structure could not be read.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StructureError {
    /// The text was not the expected shape.
    #[error("malformed structure SNBT at byte {offset}: {reason}")]
    Malformed {
        /// Where the parser gave up.
        offset: usize,
        /// What it expected.
        reason: String,
    },
    /// A required key was absent.
    #[error("structure SNBT is missing `{0}`")]
    Missing(&'static str),
    /// A block referenced a palette entry that does not exist.
    #[error("block at index {index} references palette entry {entry}, which does not exist")]
    BadPaletteRef {
        /// Which block.
        index: usize,
        /// The out-of-range entry.
        entry: usize,
    },
    /// The structure named an entity the engine has no behaviour for.
    ///
    /// Deliberately *not* a `Malformed`, because the two demand opposite
    /// answers. Malformed means whatever wrote the text got the format wrong —
    /// our bug. This one means the text is perfectly well-formed and describes
    /// a build we cannot honestly simulate — the user's build. Callers have to
    /// be able to tell them apart to say whose problem it is, and to name the
    /// offending type instead of printing a byte offset at someone holding a
    /// door that will not load.
    #[error("unsupported entity type `{entity_type}` at byte {offset}")]
    UnsupportedEntity {
        /// The `id` the entity carried, e.g. `minecraft:furnace_minecart`.
        entity_type: String,
        /// Where in the text it appeared.
        offset: usize,
    },
}

impl Structure {
    /// Parse structure SNBT.
    pub fn parse(text: &str) -> Result<Self, StructureError> {
        let mut parser = Parser::new(text);
        let structure = parser.structure()?;

        for (index, (_, entry)) in structure.blocks.iter().enumerate() {
            if *entry >= structure.palette.len() {
                return Err(StructureError::BadPaletteRef { index, entry: *entry });
            }
        }
        Ok(structure)
    }

    /// The region this structure occupies, with `margin` blocks of padding.
    ///
    /// Padding matters: out-of-bounds neighbours read as air, so a contraption
    /// flush against the edge simulates differently than it would in game. Loading
    /// with a margin is the documented way to avoid that.
    pub fn bounds(&self, margin: i32) -> Bounds {
        Bounds::new(
            Pos::new(-margin, -margin, -margin),
            Pos::new(
                self.size.0 - 1 + margin,
                self.size.1 - 1 + margin,
                self.size.2 - 1 + margin,
            ),
        )
    }

    /// The order `StructureTemplate.placeInWorld` walks its blocks.
    ///
    /// Vanilla does **not** use the file's block order. `addToLists` splits
    /// the blocks three ways — full collision cubes without block-entity NBT,
    /// everything else without NBT, and everything with NBT — sorts each and
    /// concatenates them *solid, other, block-entities*
    /// (`buildInfoList`). The update pass then walks that list, so a build's
    /// solid frame settles before a single redstone component is touched.
    ///
    /// The order is observable: it decides which transient each repeater and
    /// torch latches, and running a community door in file order instead
    /// started clocks the game never starts.
    ///
    /// # The key order
    ///
    /// Within a group this walks **ascending y, then x, then z**, exactly as
    /// `buildInfoList`'s comparator reads:
    /// `comparingInt(getY).thenComparingInt(getX).thenComparingInt(getZ)`, with
    /// no `reversed()` and no negation in any of the three lambdas. The setBlock
    /// loop appends each placed block to an accumulator and the update pass
    /// iterates that accumulator forward.
    /// An earlier version of this engine descended y to satisfy the `piston_race`
    /// capture; that was an artifact of two bugs since fixed — placement wrote
    /// the whole structure before running any `onPlace` (so notifications saw
    /// blocks the game had not written yet), and the loud path skipped
    /// `updateShapeAtEdge`, which is what actually pulses observers during a
    /// placement. With both corrected, ascending order reproduces every capture.
    ///
    /// Two fixtures hold the order down:
    ///
    /// - `piston_race_quiet` — two opposed pistons, each already powered when
    ///   placed, one gap between them. `knownShape` runs no update pass, so the
    ///   only tiebreak is which `onPlace` queued its block event first. Vanilla
    ///   picks the **bottom** piston: the walk ascends y.
    /// - `gap_race` — the same race horizontally, with the high-x observer
    ///   written first in the file. Vanilla picks the **low-x** side, which
    ///   rules out file order and pins the x tie-break as ascending.
    pub fn placement_order(
        &self,
        is_full_cube: impl Fn(&str) -> bool,
        has_dynamic_shape: impl Fn(&str) -> bool,
    ) -> Vec<Pos> {
        let mut solid: Vec<Pos> = Vec::new();
        let mut other: Vec<Pos> = Vec::new();
        let mut entities: Vec<Pos> = Vec::new();
        for (pos, entry) in &self.blocks {
            let descriptor = &self.palette[*entry];
            //  splits on `info.nbt != null` — *any* block entity,
            // not just one holding items. A comparator carrying its saved
            // OutputSignal belongs in this group too, and putting it in the
            // "other" group instead places it earlier than the game does.
            if self.block_entities.contains(pos) {
                entities.push(*pos);
            } else if !has_dynamic_shape(descriptor) && is_full_cube(descriptor) {
                solid.push(*pos);
            } else {
                other.push(*pos);
            }
        }
        for group in [&mut solid, &mut other, &mut entities] {
            group.sort_by_key(|p| (p.y, p.x, p.z));
        }
        solid.into_iter().chain(other).chain(entities).collect()
    }

    /// Place this structure into `world`, interning states as it goes.
    ///
    /// Writes directly rather than through a tick context: loading is not a
    /// simulated event, and firing neighbour updates for every block of a build
    /// would run the contraption before the caller asked for it.
    pub fn place(&self, world: &mut World, states: &mut StateRegistry, origin: Pos) -> usize {
        let mut ids: Vec<StateId> = Vec::with_capacity(self.palette.len());
        for descriptor in &self.palette {
            ids.push(states.intern(descriptor).unwrap_or(StateId::AIR));
        }

        let mut placed = 0;
        for (pos, entry) in &self.blocks {
            let target = Pos::new(origin.x + pos.x, origin.y + pos.y, origin.z + pos.z);
            if world.set(target, ids[*entry]).is_some() {
                placed += 1;
            }
        }
        placed
    }

    /// Insertion-restriction mask authored for the container at `pos`.
    pub fn blocked_slots_at(&self, pos: Pos) -> u16 {
        self.inventory_blocked_slots
            .iter()
            .find_map(|(candidate, mask)| (*candidate == pos).then_some(*mask))
            .unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

struct Parser<'a> {
    text: &'a [u8],
    at: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text: text.as_bytes(), at: 0 }
    }

    fn err<T>(&self, reason: impl Into<String>) -> Result<T, StructureError> {
        Err(StructureError::Malformed { offset: self.at, reason: reason.into() })
    }

    fn skip_ws(&mut self) {
        while self.at < self.text.len() && (self.text[self.at] as char).is_whitespace() {
            self.at += 1;
        }
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_ws();
        self.text.get(self.at).copied()
    }

    fn eat(&mut self, byte: u8) -> Result<(), StructureError> {
        if self.peek() == Some(byte) {
            self.at += 1;
            Ok(())
        } else {
            self.err(format!("expected `{}`", byte as char))
        }
    }

    /// A bare or quoted key.
    fn key(&mut self) -> Result<String, StructureError> {
        self.skip_ws();
        if matches!(self.peek(), Some(b'"' | b'\'')) {
            return self.string();
        }
        let start = self.at;
        while self.at < self.text.len() {
            let c = self.text[self.at] as char;
            if c.is_alphanumeric() || c == '_' || c == '.' || c == '-' || c == '+' {
                self.at += 1;
            } else {
                break;
            }
        }
        if start == self.at {
            return self.err("expected a key");
        }
        Ok(String::from_utf8_lossy(&self.text[start..self.at]).into_owned())
    }

    /// A quoted string. SNBT permits either quote character, and writers pick
    /// whichever avoids escaping — a sign's `Text1` holds JSON full of `"`, so
    /// vanilla and quartz alike emit it single-quoted. Accepting only `"` made
    /// every signed build unparseable.
    fn string(&mut self) -> Result<String, StructureError> {
        let quote = match self.peek() {
            Some(q @ (b'"' | b'\'')) => q,
            _ => return self.err("expected a quoted string"),
        };
        self.at += 1;
        let mut out = String::new();
        while self.at < self.text.len() {
            match self.text[self.at] {
                b if b == quote => {
                    self.at += 1;
                    return Ok(out);
                }
                b'\\' if self.at + 1 < self.text.len() => {
                    out.push(self.text[self.at + 1] as char);
                    self.at += 2;
                }
                byte => {
                    out.push(byte as char);
                    self.at += 1;
                }
            }
        }
        self.err("unterminated string")
    }

    /// An integer, tolerating NBT's type suffixes (`1b`, `2s`, `3L`).
    fn int(&mut self) -> Result<i64, StructureError> {
        self.skip_ws();
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        while self.at < self.text.len() && (self.text[self.at] as char).is_ascii_digit() {
            self.at += 1;
        }
        if start == self.at {
            return self.err("expected an integer");
        }
        let digits = String::from_utf8_lossy(&self.text[start..self.at]).into_owned();
        // Skip a trailing type suffix if present.
        if self.at < self.text.len() && (self.text[self.at] as char).is_alphabetic() {
            self.at += 1;
        }
        digits.parse().map_or_else(|_| self.err("integer out of range"), Ok)
    }

    /// Skip any value without interpreting it — used for keys we ignore.
    fn skip_value(&mut self) -> Result<(), StructureError> {
        match self.peek() {
            Some(b'{') | Some(b'[') => {
                let open = self.text[self.at];
                let close = if open == b'{' { b'}' } else { b']' };
                let mut depth = 0;
                while self.at < self.text.len() {
                    match self.text[self.at] {
                        b'"' | b'\'' => {
                            self.string()?;
                            continue;
                        }
                        b if b == open => depth += 1,
                        b if b == close => {
                            depth -= 1;
                            if depth == 0 {
                                self.at += 1;
                                return Ok(());
                            }
                        }
                        _ => {}
                    }
                    self.at += 1;
                }
                self.err("unterminated value")
            }
            Some(b'"' | b'\'') => self.string().map(|_| ()),
            Some(_) => {
                while self.at < self.text.len() {
                    let c = self.text[self.at] as char;
                    if c == ',' || c == '}' || c == ']' {
                        break;
                    }
                    self.at += 1;
                }
                Ok(())
            }
            None => self.err("expected a value"),
        }
    }

    /// `[a, b, c]` of integers.
    fn int_list(&mut self) -> Result<Vec<i64>, StructureError> {
        self.eat(b'[')?;
        let mut out = Vec::new();
        loop {
            if self.peek() == Some(b']') {
                self.at += 1;
                return Ok(out);
            }
            out.push(self.int()?);
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {}
                _ => return self.err("expected `,` or `]`"),
            }
        }
    }

    /// An NBT int array (`[I; 0, 2]`), also accepting a plain integer list for
    /// hand-written fixtures.
    fn int_array(&mut self) -> Result<Vec<i64>, StructureError> {
        self.eat(b'[')?;
        if self.peek() == Some(b'I') {
            self.at += 1;
            self.eat(b';')?;
        }
        let mut out = Vec::new();
        loop {
            if self.peek() == Some(b']') {
                self.at += 1;
                return Ok(out);
            }
            out.push(self.int()?);
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => {}
                _ => return self.err("expected `,` or `]`"),
            }
        }
    }

    /// Skip a trailing NBT type letter (`d`, `f`, `b`, …) if one is present.
    fn eat_type_suffix(&mut self) {
        if self.at < self.text.len() && (self.text[self.at] as char).is_alphabetic() {
            self.at += 1;
        }
    }

    /// `NaN` or `Infinity`, the spellings Java's `Double.toString` produces.
    /// The sign has already been consumed by the caller.
    fn non_finite(&mut self, negative: bool) -> Option<f64> {
        for (word, value) in [("NaN", f64::NAN), ("Infinity", f64::INFINITY)] {
            if self.text[self.at..].starts_with(word.as_bytes()) {
                self.at += word.len();
                self.eat_type_suffix();
                // Negating NaN leaves NaN; only infinity carries the sign.
                return Some(if negative { -value } else { value });
            }
        }
        None
    }

    /// A floating-point number, tolerating NBT's `d`/`f` suffixes.
    ///
    /// Two things here go beyond plain digits, and both are load bearing.
    ///
    /// **`NaN` / `Infinity` / `-Infinity`.** SNBT has no grammar for these —
    /// vanilla's own number pattern demands a digit — while binary NBT stores
    /// them without fuss, as raw IEEE-754 doubles. Vanilla's *writer* then
    /// emits exactly these words via Java's `Double.toString`, so the text
    /// format can produce values it cannot read back. Accepting the words the
    /// vanilla writer already emits is the narrowest way to close that gap; it
    /// invents no new notation.
    ///
    /// This must never be "cleaned up" to 0.0. The record 3x3 door is held
    /// together by *nan carts*: minecarts whose velocity was deliberately
    /// overflowed to ±Infinity on sloped rails and then collided, so that
    /// `+Inf + -Inf` = NaN. A NaN velocity is dead physics — the cart does not
    /// fall when unsupported and nothing but a piston can move it — and the
    /// builders use them as glue to pin villagers and other carts in place.
    /// Round one to zero and it becomes an ordinary cart that moves, falls and
    /// is shoved by its neighbours, and the machine comes apart silently.
    /// See `docs/history/entity-abuse-in-record-doors.md`.
    ///
    /// **Exponents.** `4.27987680632209e-59` is a real motion component in that
    /// same world. Without handling it here, `e` would be eaten as a type
    /// suffix and `-59` read as a *second* list element — quietly turning a
    /// three-element `Motion` into four, which the caller then discards.
    fn float(&mut self) -> Result<f64, StructureError> {
        self.skip_ws();
        let start = self.at;
        let negative = match self.peek() {
            Some(b'-') => {
                self.at += 1;
                true
            }
            Some(b'+') => {
                self.at += 1;
                false
            }
            _ => false,
        };
        if let Some(value) = self.non_finite(negative) {
            return Ok(value);
        }
        let mantissa = self.at;
        while self.at < self.text.len() {
            let c = self.text[self.at] as char;
            if c.is_ascii_digit() || c == '.' {
                self.at += 1;
            } else {
                break;
            }
        }
        if self.at == mantissa {
            return self.err("expected a number");
        }
        // Only an exponent if digits actually follow; otherwise `e` was a type
        // suffix and the position is given back.
        if matches!(self.peek(), Some(b'e' | b'E')) {
            let mark = self.at;
            self.at += 1;
            if matches!(self.peek(), Some(b'-' | b'+')) {
                self.at += 1;
            }
            let digits = self.at;
            while self.at < self.text.len() && (self.text[self.at] as char).is_ascii_digit() {
                self.at += 1;
            }
            if self.at == digits {
                self.at = mark;
            }
        }
        let text = String::from_utf8_lossy(&self.text[start..self.at]).into_owned();
        self.eat_type_suffix();
        text.parse()
            .map_or_else(|_| self.err("number out of range"), Ok)
    }

    /// `[a, b, c]` of floats.
    fn float_list(&mut self) -> Result<Vec<f64>, StructureError> {
        self.eat(b'[')?;
        let mut out = Vec::new();
        loop {
            if self.peek() == Some(b']') {
                self.at += 1;
                return Ok(out);
            }
            out.push(self.float()?);
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }
    }

    /// A `Passengers` list — entity compounds nested inside a vehicle's own.
    ///
    /// The shape differs from an `entities` entry: there is no wrapping
    /// `{pos, blockPos, nbt}`, the compound *is* the entity, and its position
    /// tag is `Pos` rather than `pos`. That `Pos` is read only so an
    /// unparseable rider still fails loudly; the engine seats a rider from its
    /// vehicle, not from the file — see [`crate::entity::passenger_attachment`].
    fn passenger_list(&mut self) -> Result<Vec<SpawnedEntity>, StructureError> {
        self.eat(b'[')?;
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(b']') {
                self.at += 1;
                return Ok(out);
            }
            out.push(self.passenger_entry()?);
            self.skip_ws();
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }
    }

    /// One passenger compound: `{id: "...", Pos: [x, y, z], Motion: [..]}`.
    fn passenger_entry(&mut self) -> Result<SpawnedEntity, StructureError> {
        self.eat(b'{')?;
        let mut id = String::new();
        let mut pos: Option<[f64; 3]> = None;
        let mut motion = [0.0f64; 3];
        loop {
            self.skip_ws();
            if self.peek() == Some(b'}') {
                self.at += 1;
                break;
            }
            let field = self.key()?;
            self.eat(b':')?;
            match field.as_str() {
                "id" => id = self.string()?,
                "Pos" => {
                    let values = self.float_list()?;
                    if values.len() != 3 {
                        return self.err("passenger `Pos` must have three elements");
                    }
                    pos = Some([values[0], values[1], values[2]]);
                }
                "Motion" => {
                    let values = self.float_list()?;
                    if values.len() == 3 {
                        motion = [values[0], values[1], values[2]];
                    }
                }
                _ => self.skip_value()?,
            }
            self.skip_ws();
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }
        let Some(pos) = pos else {
            return self.err("passenger entity needs `Pos`");
        };
        // A passenger is a frozen body and nothing else: `positionRider`
        // overwrites its position every tick, so nothing with physics of its own
        // can meaningfully ride. A registered kind with any other motion class
        // refuses here rather than losing its physics silently.
        match crate::entity::entity_behaviour(&id).map(crate::entity_kind::EntityBehaviour::motion)
        {
            Some(crate::entity_kind::EntityMotion::Frozen) => {
                Ok(SpawnedEntity::Body(SpawnedBody {
                    kind: id,
                    pos,
                    motion,
                    leashed: false,
                    passengers: Vec::new(),
                }))
            }
            _ => Err(StructureError::UnsupportedEntity {
                entity_type: if id.is_empty() { "<no id>".to_string() } else { id },
                offset: self.at,
            }),
        }
    }

    /// One `entities` entry: an authored item entity or minecart.
    fn entity_entry(&mut self) -> Result<SpawnedEntity, StructureError> {
        self.eat(b'{')?;
        let mut pos: Option<[f64; 3]> = None;
        let mut motion = [0.0f64; 3];
        let mut item: Option<(String, u8)> = None;
        let mut pickup_delay = 0u32;
        let mut entity_items: Vec<crate::inventory::ItemStack> = Vec::new();
        let mut fuel = 0u32;
        let mut push = [0.0f64; 2];
        let mut yaw = 0.0f64;
        let mut entity_id = String::new();
        let mut passengers: Vec<SpawnedEntity> = Vec::new();
        let mut leashed = false;
        loop {
            if self.peek() == Some(b'}') {
                self.at += 1;
                break;
            }
            let key = self.key()?;
            self.eat(b':')?;
            match key.as_str() {
                "pos" => {
                    let values = self.float_list()?;
                    if values.len() != 3 {
                        return self.err("entity pos must have three elements");
                    }
                    pos = Some([values[0], values[1], values[2]]);
                }
                "nbt" => {
                    // The entity compound: id, Item, Motion, PickupDelay,
                    // Rotation, Fuel/Push, Passengers.
                    self.eat(b'{')?;
                    loop {
                        if self.peek() == Some(b'}') {
                            self.at += 1;
                            break;
                        }
                        let field = self.key()?;
                        self.eat(b':')?;
                        match field.as_str() {
                            "id" => entity_id = self.string()?,
                            "Motion" => {
                                let values = self.float_list()?;
                                if values.len() == 3 {
                                    motion = [values[0], values[1], values[2]];
                                }
                            }
                            "PickupDelay" => pickup_delay = self.int()? as u32,
                            // `Rotation: [yaw, pitch]`. Only yaw matters here:
                            // it is the gate on cart-cart pushing.
                            "Rotation" => {
                                let values = self.float_list()?;
                                if !values.is_empty() {
                                    yaw = values[0];
                                }
                            }
                            // A furnace cart's self-drive. Zero on every cart
                            // in the record door, but a fuelled one propels
                            // itself, so it is read rather than assumed.
                            "Items" => {
                                self.eat(b'[')?;
                                loop {
                                    if self.peek() == Some(b']') {
                                        self.at += 1;
                                        break;
                                    }
                                    entity_items.push(self.item_entry()?);
                                    if self.peek() == Some(b',') {
                                        self.at += 1;
                                    }
                                }
                            }
                            "Fuel" => fuel = self.int()? as u32,
                            "PushX" => push[0] = self.float()?,
                            "PushZ" => push[1] = self.float()?,
                            "Item" => {
                                let stack = self.item_entry()?;
                                item = Some((stack.id, stack.count));
                            }
                            // 26.2 writes `leash`; older saves used `Leash`.
                            // The payload can be a block position or an entity
                            // reference and is not portable across every
                            // structure carrier, but its presence distinguishes
                            // the measured tether-rest boat from a free-moving
                            // boat whose velocity must still refuse.
                            "leash" | "Leash" => {
                                leashed = true;
                                self.skip_value()?;
                            }
                            // Riders. A list of entity compounds nested inside
                            // the vehicle's own — the one place in a world file
                            // where an entity is not at the top level, and the
                            // reason a top-level count of `55_3x3.zip` reports
                            // 22 where vanilla's own capture of it reports 24.
                            "Passengers" => passengers = self.passenger_list()?,
                            _ => self.skip_value()?,
                        }
                        if self.peek() == Some(b',') {
                            self.at += 1;
                        }
                    }
                }
                _ => self.skip_value()?,
            }
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }
        // A rider changes what a vehicle *is* — its passenger's box is in the
        // world, and it moves when the vehicle moves. Only the plain minecart's
        // seat has been measured (`blaze_ride.entities.log`), so a `Passengers`
        // list on anything else refuses instead of being dropped: dropping it is
        // precisely the silent under-report this whole seam exists to stop.
        // Only the plain minecart's seats have been measured
        // (`blaze_ride.entities.log`), and the registry is what says so — a
        // `Passengers` list on a vehicle with no measured seat refuses instead of
        // being dropped, because dropping it is precisely the silent
        // under-report this whole seam exists to stop.
        let behaviour = crate::entity::entity_behaviour(&entity_id);
        if !passengers.is_empty() && !behaviour.is_some_and(|b| b.carries_passengers()) {
            return self.err(
                "`Passengers` on an entity with no measured seat: the offset cannot \
                 be derived from the hitboxes, and carrying the rider at a guessed \
                 one would move a hitbox the build depends on",
            );
        }
        // Dispatch on the *motion class*, not the name. A new entity type of an
        // existing class is a registry row and nothing here.
        match behaviour.map(crate::entity_kind::EntityBehaviour::motion) {
            Some(crate::entity_kind::EntityMotion::Item) => match (pos, item) {
                (Some(pos), Some(item)) => {
                    Ok(SpawnedEntity::Item(SpawnedItem { pos, motion, item, pickup_delay }))
                }
                _ => self.err("item entity needs `pos` and `Item`"),
            },
            Some(crate::entity_kind::EntityMotion::Frozen) => match pos {
                Some(pos) => Ok(SpawnedEntity::Body(SpawnedBody {
                    kind: entity_id,
                    pos,
                    motion,
                    leashed,
                    passengers,
                })),
                None => self.err("entity needs `pos`"),
            },
            // The cart classes are the one place a name still decides, and that
            // is not a gap in the registry: the reader has a *representation* for
            // a plain cart and for a furnace cart, and none for the container
            // variants. A chest, hopper or TNT cart shares the plain cart's
            // hitbox — which is why it has a row — but it also carries an
            // inventory or a fuse that this engine does not model, so loading one
            // as a plain cart would silently drop the thing that makes it that
            // cart.
            Some(crate::entity_kind::EntityMotion::Minecart) => match entity_id.as_str() {
                // Container carts ride the same hitbox as a plain cart, and
                // their inventory now rides with them — which is the thing
                // that made loading them as plain carts a silent lie.
                "minecraft:minecart" | "minecraft:chest_minecart" | "minecraft:hopper_minecart" => {
                    match pos {
                        Some(pos) => Ok(SpawnedEntity::Minecart(SpawnedMinecart {
                            kind: entity_id,
                            items: entity_items,
                            pos,
                            motion,
                            yaw,
                            passengers,
                        })),
                        None => self.err("minecart entity needs `pos`"),
                    }
                }
                "minecraft:furnace_minecart" => match pos {
                    Some(pos) => Ok(SpawnedEntity::FurnaceMinecart(SpawnedFurnaceMinecart {
                        pos,
                        motion,
                        fuel,
                        push,
                        yaw,
                    })),
                    None => self.err("furnace minecart entity needs `pos`"),
                },
                other => Err(StructureError::UnsupportedEntity {
                    entity_type: other.to_string(),
                    offset: self.at,
                }),
            },
            // A type this reader cannot even *represent*. Distinct from one it
            // can represent but the engine cannot yet simulate — that second
            // refusal belongs at construction, where the behaviour tables are.
            // Dropping either would let a build whose mechanism depends on the
            // entity load clean and run as though it were not there.
            None => Err(StructureError::UnsupportedEntity {
                // An entry with no `id` at all still has to say something
                // useful; `unsupported entity type ``` names nothing.
                entity_type: if entity_id.is_empty() {
                    "<entity with no id>".to_string()
                } else {
                    entity_id
                },
                offset: self.at,
            }),
        }
    }

    /// A block-entity `nbt` compound: `Items`, crafter `disabled_slots`, the
    /// comparator's `OutputSignal`, and a command block's `Command`, skipping
    /// the rest.
    ///
    /// `OutputSignal` is not decoration. A comparator emits its *stored*
    /// block-entity strength rather than anything its block state carries, so a
    /// build saved mid-cycle starts with whatever each comparator last emitted.
    /// Dropping it made every comparator in a loaded door start at zero, which
    /// is true of a freshly placed one and false of a saved one.
    fn nbt_items(
        &mut self,
    ) -> Result<
        (
            Vec<crate::inventory::ItemStack>,
            Option<u8>,
            Option<String>,
            Option<u16>,
        ),
        StructureError,
    >
    {
        self.eat(b'{')?;
        let mut items = Vec::new();
        let mut output_signal = None;
        let mut command = None;
        let mut blocked_slots = None;
        loop {
            if self.peek() == Some(b'}') {
                self.at += 1;
                return Ok((items, output_signal, command, blocked_slots));
            }
            let key = self.key()?;
            self.eat(b':')?;
            if key == "OutputSignal" {
                output_signal = Some(self.int()? as u8);
            } else if key == "Command" {
                command = Some(self.string()?).filter(|c| !c.is_empty());
            } else if key == "Items" {
                self.eat(b'[')?;
                loop {
                    if self.peek() == Some(b']') {
                        self.at += 1;
                        break;
                    }
                    items.push(self.item_entry()?);
                    if self.peek() == Some(b',') {
                        self.at += 1;
                    }
                }
            } else if key == "disabled_slots" {
                let mut mask = 0u16;
                for slot in self.int_array()? {
                    if !(0..=8).contains(&slot) {
                        return self.err("crafter disabled slot must be in 0..=8");
                    }
                    mask |= 1 << slot;
                }
                blocked_slots = Some(mask);
            } else {
                self.skip_value()?;
            }
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }
    }

    /// One `Items` entry: `{Slot: 0b, id: "minecraft:redstone", count: 3}`.
    fn item_entry(&mut self) -> Result<crate::inventory::ItemStack, StructureError> {
        self.eat(b'{')?;
        let mut slot = 0u8;
        let mut id = String::new();
        let mut count = 1u8;
        let mut contents: Option<Vec<crate::inventory::ItemStack>> = None;
        loop {
            if self.peek() == Some(b'}') {
                self.at += 1;
                break;
            }
            let key = self.key()?;
            self.eat(b':')?;
            match key.as_str() {
                "Slot" => slot = self.int()? as u8,
                "id" => id = self.string()?,
                // Both spellings: `Count` before the components rework, `count` after.
                "count" | "Count" => count = self.int()? as u8,
                "components" => contents = self.item_components()?,
                _ => self.skip_value()?,
            }
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }
        if id.is_empty() {
            return self.err("item entry needs an id");
        }
        Ok(crate::inventory::ItemStack { slot, id, count, contents })
    }

    /// An item's `components` compound. Only `minecraft:container` — a
    /// shulker box's slots — is understood; every other component is skipped.
    fn item_components(
        &mut self,
    ) -> Result<Option<Vec<crate::inventory::ItemStack>>, StructureError> {
        self.eat(b'{')?;
        let mut contents: Option<Vec<crate::inventory::ItemStack>> = None;
        loop {
            self.skip_ws();
            if self.peek() == Some(b'}') {
                self.at += 1;
                break;
            }
            let key = self.key()?;
            self.eat(b':')?;
            if key == "minecraft:container" {
                contents = Some(self.container_component()?);
            } else {
                self.skip_value()?;
            }
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }
        Ok(contents)
    }

    /// `minecraft:container`: `[{slot: 0, item: {id: "...", count: 2}}, ...]`.
    fn container_component(
        &mut self,
    ) -> Result<Vec<crate::inventory::ItemStack>, StructureError> {
        self.eat(b'[')?;
        let mut stacks = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(b']') {
                self.at += 1;
                break;
            }
            self.eat(b'{')?;
            let mut slot = 0u8;
            let mut id = String::new();
            let mut count = 1u8;
            loop {
                self.skip_ws();
                if self.peek() == Some(b'}') {
                    self.at += 1;
                    break;
                }
                let key = self.key()?;
                self.eat(b':')?;
                match key.as_str() {
                    "slot" | "Slot" => slot = self.int()? as u8,
                    "item" => {
                        self.eat(b'{')?;
                        loop {
                            self.skip_ws();
                            if self.peek() == Some(b'}') {
                                self.at += 1;
                                break;
                            }
                            let inner = self.key()?;
                            self.eat(b':')?;
                            match inner.as_str() {
                                "id" => id = self.string()?,
                                "count" | "Count" => count = self.int()? as u8,
                                _ => self.skip_value()?,
                            }
                            if self.peek() == Some(b',') {
                                self.at += 1;
                            }
                        }
                    }
                    _ => self.skip_value()?,
                }
                if self.peek() == Some(b',') {
                    self.at += 1;
                }
            }
            if id.is_empty() {
                return self.err("container component entry needs an item id");
            }
            stacks.push(crate::inventory::ItemStack { slot, id, count, contents: None });
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }
        Ok(stacks)
    }

    /// One palette entry, rendered as the descriptor the registry interns.
    fn palette_entry(&mut self) -> Result<String, StructureError> {
        self.eat(b'{')?;
        let mut name = String::new();
        let mut properties: Vec<(String, String)> = Vec::new();

        loop {
            if self.peek() == Some(b'}') {
                self.at += 1;
                break;
            }
            let key = self.key()?;
            self.eat(b':')?;
            match key.as_str() {
                "Name" => name = self.string()?,
                "Properties" => {
                    self.eat(b'{')?;
                    loop {
                        if self.peek() == Some(b'}') {
                            self.at += 1;
                            break;
                        }
                        let property = self.key()?;
                        self.eat(b':')?;
                        let value = self.string()?;
                        properties.push((property, value));
                        if self.peek() == Some(b',') {
                            self.at += 1;
                        }
                    }
                }
                _ => self.skip_value()?,
            }
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }

        if name.is_empty() {
            return Err(StructureError::Missing("palette entry Name"));
        }
        // Properties are sorted so the descriptor is canonical: the same state must
        // intern to the same id however the file happened to order them.
        properties.sort();
        if properties.is_empty() {
            return Ok(name);
        }
        let rendered: Vec<String> = properties
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        Ok(format!("{name}[{}]", rendered.join(",")))
    }

    fn structure(&mut self) -> Result<Structure, StructureError> {
        self.eat(b'{')?;
        let mut size = None;
        let mut palette = None;
        let mut blocks: Option<Vec<(Pos, usize)>> = None;
        let mut inventories: Vec<(Pos, Vec<crate::inventory::ItemStack>)> = Vec::new();
        let mut inventory_blocked_slots: Vec<(Pos, u16)> = Vec::new();
        let mut block_entities: Vec<Pos> = Vec::new();
        let mut comparator_outputs: Vec<(Pos, u8)> = Vec::new();
        let mut commands: Vec<(Pos, String)> = Vec::new();
        let mut entities: Vec<SpawnedEntity> = Vec::new();
        let mut data_version: Option<i32> = None;

        loop {
            if self.peek() == Some(b'}') {
                self.at += 1;
                break;
            }
            let key = self.key()?;
            self.eat(b':')?;
            match key.as_str() {
                // Read rather than skipped, because `Entity.load`'s treatment of
                // a non-finite `Motion` changed at 1.21.11 and the tag is the
                // only record of which side of that a file sits on. Skipping it
                // made every reader guess the modern rule, which silently
                // sanitises the NaN velocities an older nan-cart door is glued
                // together by.
                "DataVersion" => data_version = Some(self.int()? as i32),
                "size" => {
                    let values = self.int_list()?;
                    if values.len() != 3 {
                        return self.err("size must have three elements");
                    }
                    size = Some((values[0] as i32, values[1] as i32, values[2] as i32));
                }
                "palette" => {
                    self.eat(b'[')?;
                    let mut entries = Vec::new();
                    loop {
                        if self.peek() == Some(b']') {
                            self.at += 1;
                            break;
                        }
                        entries.push(self.palette_entry()?);
                        if self.peek() == Some(b',') {
                            self.at += 1;
                        }
                    }
                    palette = Some(entries);
                }
                "entities" => {
                    self.eat(b'[')?;
                    loop {
                        if self.peek() == Some(b']') {
                            self.at += 1;
                            break;
                        }
                        entities.push(self.entity_entry()?);
                        if self.peek() == Some(b',') {
                            self.at += 1;
                        }
                    }
                }
                "blocks" => {
                    self.eat(b'[')?;
                    let mut entries = Vec::new();
                    loop {
                        if self.peek() == Some(b']') {
                            self.at += 1;
                            break;
                        }
                        self.eat(b'{')?;
                        let mut pos = None;
                        let mut state = None;
                        let mut items: Vec<crate::inventory::ItemStack> = Vec::new();
                        let mut output_signal: Option<u8> = None;
                        let mut command: Option<String> = None;
                        let mut blocked_slots: Option<u16> = None;
                        let mut has_nbt = false;
                        loop {
                            if self.peek() == Some(b'}') {
                                self.at += 1;
                                break;
                            }
                            let field = self.key()?;
                            self.eat(b':')?;
                            match field.as_str() {
                                "pos" => {
                                    let values = self.int_list()?;
                                    if values.len() != 3 {
                                        return self.err("pos must have three elements");
                                    }
                                    pos = Some(Pos::new(
                                        values[0] as i32,
                                        values[1] as i32,
                                        values[2] as i32,
                                    ));
                                }
                                "state" => state = Some(self.int()? as usize),
                                "nbt" => {
                                    let parsed = self.nbt_items()?;
                                    items = parsed.0;
                                    output_signal = parsed.1;
                                    command = parsed.2;
                                    blocked_slots = parsed.3;
                                    has_nbt = true;
                                }
                                _ => self.skip_value()?,
                            }
                            if self.peek() == Some(b',') {
                                self.at += 1;
                            }
                        }
                        match (pos, state) {
                            (Some(p), Some(s)) => {
                                entries.push((p, s));
                                if !items.is_empty() || blocked_slots.is_some() {
                                    inventories.push((p, items));
                                }
                                if let Some(mask) = blocked_slots {
                                    inventory_blocked_slots.push((p, mask));
                                }
                                if has_nbt {
                                    block_entities.push(p);
                                }
                                if let Some(signal) = output_signal {
                                    comparator_outputs.push((p, signal));
                                }
                                if let Some(text) = command {
                                    commands.push((p, text));
                                }
                            }
                            _ => return self.err("block needs both `pos` and `state`"),
                        }
                        if self.peek() == Some(b',') {
                            self.at += 1;
                        }
                    }
                    blocks = Some(entries);
                }
                _ => self.skip_value()?,
            }
            if self.peek() == Some(b',') {
                self.at += 1;
            }
        }

        Ok(Structure {
            data_version,
            size: size.ok_or(StructureError::Missing("size"))?,
            palette: palette.ok_or(StructureError::Missing("palette"))?,
            blocks: blocks.ok_or(StructureError::Missing("blocks"))?,
            inventories,
            inventory_blocked_slots,
            comparator_outputs,
            commands,
            block_entities,
            item_entities: entities
                .iter()
                .filter_map(|e| match e {
                    SpawnedEntity::Item(item) => Some(item.clone()),
                    _ => None,
                })
                .collect(),
            entities,
        })
    }
}

#[cfg(test)]
mod tests {

    /// A sign's `Text1` is JSON, so every writer emits it single-quoted rather
    /// than escaping each inner `"`. The parser rejected those outright, which
    /// meant no build containing a signed label could load at all.
    #[test]
    fn single_quoted_strings_parse() {
        let text = r#"{
            DataVersion: 4903,
            size: [1, 1, 1],
            palette: [{Name: "minecraft:oak_sign"}],
            blocks: [
                {pos: [0, 0, 0], state: 0, nbt: {Text1: '{"text":"a \"b\" c"}', Color: white}}
            ],
            entities: []
        }"#;
        let parsed = Structure::parse(text).expect("single-quoted nbt should parse");
        assert_eq!(parsed.blocks.len(), 1);
        assert_eq!(parsed.block_entities, vec![Pos::new(0, 0, 0)]);
    }
    use super::*;

    const SAMPLE: &str = r#"{
        DataVersion: 4903,
        size: [3, 2, 1],
        palette: [
            {Name: "minecraft:stone"},
            {Name: "minecraft:repeater", Properties: {facing: "east", delay: "1"}}
        ],
        blocks: [
            {pos: [0, 0, 0], state: 0},
            {pos: [1, 1, 0], state: 1}
        ],
        entities: []
    }"#;

    #[test]
    fn parses_the_structure_format() {
        let s = Structure::parse(SAMPLE).expect("parses");
        assert_eq!(s.size, (3, 2, 1));
        assert_eq!(s.blocks.len(), 2);
        assert_eq!(s.blocks[1], (Pos::new(1, 1, 0), 1));
    }

    #[test]
    fn a_crafter_keeps_empty_disabled_slots_as_inventory_state() {
        let text = r#"{
            DataVersion: 4903,
            size: [1, 1, 1],
            palette: [{
                Name: "minecraft:crafter",
                Properties: {crafting: "false", orientation: "west_up", triggered: "false"}
            }],
            blocks: [{
                pos: [0, 0, 0],
                state: 0,
                nbt: {Items: [], disabled_slots: [I; 0, 4]}
            }],
            entities: []
        }"#;
        let parsed = Structure::parse(text).expect("crafter block entity parses");
        let pos = Pos::new(0, 0, 0);
        assert_eq!(parsed.inventories, vec![(pos, Vec::new())]);
        assert_eq!(parsed.blocked_slots_at(pos), (1 << 0) | (1 << 4));
        assert_eq!(parsed.inventory_blocked_slots, vec![(pos, 17)]);
    }

    #[test]
    fn properties_render_sorted_so_descriptors_are_canonical() {
        // The same state must intern to the same id however the file ordered its
        // properties, or two identical blocks become two different ids.
        let s = Structure::parse(SAMPLE).unwrap();
        assert_eq!(s.palette[0], "minecraft:stone");
        assert_eq!(s.palette[1], "minecraft:repeater[delay=1,facing=east]");
    }

    #[test]
    fn unknown_keys_are_skipped_not_fatal() {
        // Real files carry DataVersion, entities, author fields and more. A reader
        // that choked on them would reject nearly every structure in the wild.
        let text = r#"{
            DataVersion: 4903, author: "someone", size: [1,1,1],
            palette: [{Name: "minecraft:stone"}],
            blocks: [{pos: [0,0,0], state: 0, nbt: {Items: [{id: "x", Count: 1b}]}}],
            entities: [{pos: [0.5, 0.0, 0.5], blockPos: [0,0,0], nbt: {id: "minecraft:item", Extra: 1b, Item: {id: "minecraft:stone", count: 2}}}]
        }"#;
        let s = Structure::parse(text).expect("must tolerate extra keys");
        assert_eq!(s.blocks.len(), 1);
        assert_eq!(s.item_entities.len(), 1);
        assert_eq!(s.item_entities[0].item, ("minecraft:stone".to_string(), 2));
        assert_eq!(s.item_entities[0].pos, [0.5, 0.0, 0.5]);
    }

    #[test]
    fn integer_type_suffixes_are_tolerated() {
        let text = r#"{size: [1b, 1b, 1b], palette: [{Name: "minecraft:stone"}],
                       blocks: [{pos: [0,0,0], state: 0}]}"#;
        assert_eq!(Structure::parse(text).unwrap().size, (1, 1, 1));
    }

    #[test]
    fn a_bad_palette_reference_is_rejected() {
        // Silently dropping the block would leave a hole in the build and a
        // simulation that quietly disagrees with the file.
        let text = r#"{size: [1,1,1], palette: [{Name: "minecraft:stone"}],
                       blocks: [{pos: [0,0,0], state: 7}]}"#;
        assert_eq!(
            Structure::parse(text),
            Err(StructureError::BadPaletteRef { index: 0, entry: 7 })
        );
    }

    #[test]
    fn a_missing_key_names_itself() {
        let text = r#"{size: [1,1,1], blocks: []}"#;
        assert_eq!(Structure::parse(text), Err(StructureError::Missing("palette")));
    }

    #[test]
    fn placing_interns_states_and_writes_blocks() {
        let s = Structure::parse(SAMPLE).unwrap();
        let mut states = StateRegistry::new();
        let mut world = World::new(s.bounds(2));
        let placed = s.place(&mut world, &mut states, Pos::new(0, 0, 0));

        assert_eq!(placed, 2);
        let stone = states.get("minecraft:stone").unwrap();
        assert_eq!(world.get(Pos::new(0, 0, 0)), stone);
        assert_eq!(
            states.descriptor(world.get(Pos::new(1, 1, 0))),
            Some("minecraft:repeater[delay=1,facing=east]")
        );
    }

    #[test]
    fn bounds_include_the_requested_margin() {
        // Padding is not cosmetic: out-of-bounds neighbours read as air, so a
        // contraption flush against the edge simulates differently than in game.
        let s = Structure::parse(SAMPLE).unwrap();
        let bounds = s.bounds(4);
        assert_eq!(bounds.min, Pos::new(-4, -4, -4));
        assert_eq!(bounds.max, Pos::new(6, 5, 4));
    }

    #[test]
    fn malformed_input_reports_where_it_failed() {
        let err = Structure::parse("{size: [1,1,1], palette: [").unwrap_err();
        assert!(matches!(err, StructureError::Malformed { .. }), "{err}");
    }

    /// A cart's `Passengers` are read, and an absent tag means none.
    ///
    /// The second cart is the control, and it is the point: without it a
    /// passenger-shaped assertion could pass on a reader that put *every* cart's
    /// rider list at one element. `55_3x3.zip` has both kinds — two of its four
    /// plain carts carry a blaze and two do not — and reading the tag is the
    /// difference between counting 22 entities and counting the 24 vanilla does.
    #[test]
    fn a_carts_passengers_are_read_and_default_to_none() {
        const TEXT: &str = r#"{
            size: [1, 1, 1],
            palette: [{Name: "minecraft:rail"}],
            blocks: [{pos: [0, 0, 0], state: 0}],
            entities: [
                {pos: [0.5d, 2.0625d, 0.5d], blockPos: [0, 2, 0], nbt: {id: "minecraft:minecart", Motion: [0.0d, 0.0d, NaN], Passengers: [{id: "minecraft:blaze", Pos: [0.5d, 2.25d, 0.5d], Motion: [-0.039d, -0.0784d, 0.0253d]}]}},
                {pos: [0.5d, 2.0625d, 1.5d], blockPos: [0, 2, 1], nbt: {id: "minecraft:minecart", Motion: [0.0d, 0.0d, 0.0d]}}
            ]
        }"#;
        let s = Structure::parse(TEXT).unwrap();
        match (&s.entities[0], &s.entities[1]) {
            (SpawnedEntity::Minecart(carrying), SpawnedEntity::Minecart(empty)) => {
                assert_eq!(carrying.passengers.len(), 1);
                match &carrying.passengers[0] {
                    SpawnedEntity::Body(blaze) => {
                        assert_eq!(blaze.kind, "minecraft:blaze");
                        assert_eq!(blaze.pos, [0.5, 2.25, 0.5]);
                        // The gravity a rider accrues and never uses.
                        assert_eq!(blaze.motion, [-0.039, -0.0784, 0.0253]);
                    }
                    other => panic!("expected a blaze rider, got {other:?}"),
                }
                assert!(
                    empty.passengers.is_empty(),
                    "no Passengers tag means no riders — if this cart has one, the \
                     reader is inventing them and the assertion above proves nothing"
                );
            }
            other => panic!("expected two minecarts, got {other:?}"),
        }
    }

    /// A rider on a vehicle whose seat nobody measured refuses.
    ///
    /// The plain-minecart case above is the control: the same tag on the type
    /// that *has* been measured parses fine, so this is a refusal about the
    /// vehicle and not a reader that cannot read `Passengers` at all.
    #[test]
    fn passengers_on_an_unmeasured_vehicle_refuse() {
        const TEXT: &str = r#"{
            size: [1, 1, 1],
            palette: [{Name: "minecraft:rail"}],
            blocks: [{pos: [0, 0, 0], state: 0}],
            entities: [
                {pos: [0.5d, 2.0d, 0.5d], blockPos: [0, 2, 0], nbt: {id: "minecraft:furnace_minecart", Passengers: [{id: "minecraft:blaze", Pos: [0.5d, 2.2d, 0.5d]}]}}
            ]
        }"#;
        let err = Structure::parse(TEXT).unwrap_err();
        assert!(
            format!("{err}").contains("Passengers"),
            "the refusal must name the tag it refused: {err}"
        );
    }

    /// A rider this reader cannot even represent is named, not dropped.
    #[test]
    fn an_unrepresentable_rider_is_refused_by_name() {
        const TEXT: &str = r#"{
            size: [1, 1, 1],
            palette: [{Name: "minecraft:rail"}],
            blocks: [{pos: [0, 0, 0], state: 0}],
            entities: [
                {pos: [0.5d, 2.0d, 0.5d], blockPos: [0, 2, 0], nbt: {id: "minecraft:minecart", Passengers: [{id: "minecraft:creeper", Pos: [0.5d, 2.2d, 0.5d]}]}}
            ]
        }"#;
        match Structure::parse(TEXT).unwrap_err() {
            StructureError::UnsupportedEntity { entity_type, .. } => {
                assert_eq!(entity_type, "minecraft:creeper");
            }
            other => panic!("expected UnsupportedEntity, got {other:?}"),
        }
    }

    /// A cart's `Rotation` is read, and defaults to 0 when absent.
    ///
    /// Not decoration: yaw gates cart-cart pushing, so dropping this tag turns
    /// a cart that vanilla leaves alone into one the engine shoves. The
    /// `cart_yaw` conformance golden is the end-to-end version of this.
    #[test]
    fn a_carts_rotation_is_read_and_defaults_to_zero() {
        const TEXT: &str = r#"{
            size: [1, 1, 1],
            palette: [{Name: "minecraft:rail"}],
            blocks: [{pos: [0, 0, 0], state: 0}],
            entities: [
                {pos: [0.5d, 0.0625d, 0.5d], blockPos: [0, 0, 0], nbt: {id: "minecraft:minecart", Motion: [0.0d, 0.0d, 0.0d], Rotation: [90.0f, 0.0f]}},
                {pos: [0.5d, 0.0625d, 1.5d], blockPos: [0, 0, 1], nbt: {id: "minecraft:minecart", Motion: [0.0d, 0.0d, 0.0d]}}
            ]
        }"#;
        let s = Structure::parse(TEXT).unwrap();
        match (&s.entities[0], &s.entities[1]) {
            (SpawnedEntity::Minecart(turned), SpawnedEntity::Minecart(plain)) => {
                assert_eq!(turned.yaw, 90.0);
                assert_eq!(plain.yaw, 0.0, "no Rotation tag means vanilla's default");
            }
            other => panic!("expected two minecarts, got {other:?}"),
        }
    }

    /// And a **furnace** cart's `Rotation` is read the same way.
    ///
    /// Its own test because it was its own hole: `SpawnedFurnaceMinecart` had
    /// no `yaw` field at all, so every furnace cart in every loaded build
    /// arrived facing +X no matter what its NBT said. The record 3x3 door is
    /// fifteen furnace carts, its top row is strung out along x, and all of
    /// them read `Rotation: [±90, 0]` — the difference between a row vanilla
    /// never touches and a row that shoves itself apart.
    #[test]
    fn a_furnace_carts_rotation_is_read_and_defaults_to_zero() {
        const TEXT: &str = r#"{
            size: [1, 1, 1],
            palette: [{Name: "minecraft:rail"}],
            blocks: [{pos: [0, 0, 0], state: 0}],
            entities: [
                {pos: [0.5d, 0.0625d, 0.5d], blockPos: [0, 0, 0], nbt: {id: "minecraft:furnace_minecart", Motion: [0.0d, 0.0d, 0.0d], Rotation: [-90.0f, 0.0f]}},
                {pos: [0.5d, 0.0625d, 1.5d], blockPos: [0, 0, 1], nbt: {id: "minecraft:furnace_minecart", Motion: [0.0d, 0.0d, 0.0d]}}
            ]
        }"#;
        let s = Structure::parse(TEXT).unwrap();
        match (&s.entities[0], &s.entities[1]) {
            (SpawnedEntity::FurnaceMinecart(turned), SpawnedEntity::FurnaceMinecart(plain)) => {
                assert_eq!(turned.yaw, -90.0);
                assert_eq!(plain.yaw, 0.0, "no Rotation tag means vanilla's default");
            }
            other => panic!("expected two furnace minecarts, got {other:?}"),
        }
    }

    /// Carts and items share the `entities` list and keep their order.
    ///
    /// Order is not cosmetic: it is the placement spawn order, which is also
    /// the server's id-assignment order, and ids break ties in update order.
    #[test]
    fn an_entities_list_carries_carts_and_items_in_order() {
        const TEXT: &str = r#"{
            size: [1, 1, 1],
            palette: [{Name: "minecraft:rail"}],
            blocks: [{pos: [0, 0, 0], state: 0}],
            entities: [
                {pos: [0.5d, 0.0625d, 0.5d], blockPos: [0, 0, 0], nbt: {id: "minecraft:minecart", Motion: [0.25d, 0.0d, -0.5d]}},
                {pos: [0.5d, 1.0d, 0.5d], blockPos: [0, 1, 0], nbt: {id: "minecraft:item", Motion: [0.0d, 0.0d, 0.0d], Item: {id: "minecraft:redstone", count: 7b}, PickupDelay: 40s}}
            ]
        }"#;
        let s = Structure::parse(TEXT).unwrap();
        assert_eq!(s.entities.len(), 2);
        match &s.entities[0] {
            SpawnedEntity::Minecart(cart) => {
                assert_eq!(cart.kind, "minecraft:minecart");
                assert_eq!(cart.pos, [0.5, 0.0625, 0.5]);
                assert_eq!(cart.motion, [0.25, 0.0, -0.5]);
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
        match &s.entities[1] {
            SpawnedEntity::Item(item) => {
                assert_eq!(item.item, ("minecraft:redstone".to_string(), 7));
                assert_eq!(item.pickup_delay, 40);
            }
            other => panic!("expected an item, got {other:?}"),
        }
        assert_eq!(s.item_entities.len(), 1);
    }

    /// Non-finite and exponent-bearing velocities parse, and keep their values.
    ///
    /// The two interact: `55_3x3.zip` carries `Motion: [4.27987680632209e-59,
    /// 0.0, NaN]` on a single cart. Read without exponent support that is four
    /// numbers, not three, and the motion is discarded; read with NaN
    /// sanitised the cart stops being a nan cart and the door breaks. Both
    /// halves have to work at once, so they are asserted together.
    #[test]
    fn nan_infinity_and_exponents_survive_the_parser() {
        const TEXT: &str = r#"{
            size: [1, 1, 1],
            palette: [{Name: "minecraft:rail"}],
            blocks: [{pos: [0, 0, 0], state: 0}],
            entities: [
                {pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:minecart", Motion: [4.27987680632209e-59d, 0.0d, NaN]}},
                {pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:minecart", Motion: [Infinity, -Infinity, 1.5E+2d]}}
            ]
        }"#;
        let s = Structure::parse(TEXT).unwrap();
        assert_eq!(s.entities.len(), 2);
        match &s.entities[0] {
            SpawnedEntity::Minecart(cart) => {
                // Three elements, not four — the exponent stayed one number.
                assert_eq!(cart.motion[0], 4.27987680632209e-59);
                assert_eq!(cart.motion[1], 0.0);
                assert!(cart.motion[2].is_nan(), "NaN was lost: {:?}", cart.motion);
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
        match &s.entities[1] {
            SpawnedEntity::Minecart(cart) => {
                assert_eq!(cart.motion[0], f64::INFINITY);
                assert_eq!(cart.motion[1], f64::NEG_INFINITY);
                assert_eq!(cart.motion[2], 150.0);
            }
            other => panic!("expected a minecart, got {other:?}"),
        }
    }

    /// The record door's entity cast parses, whether or not it can be run.
    ///
    /// Reading and simulating are separate questions now: everything here is
    /// carried faithfully out of the file, and the engine decides at
    /// construction what it can actually tick. `Fuel` and `PushX`/`PushZ` are
    /// read rather than assumed — they are zero on all fifteen furnace carts in
    /// `55_3x3`, which is what makes those pure mass and hitbox, but a fuelled
    /// cart drives itself and a build using one must not be quietly flattened.
    #[test]
    fn the_record_doors_entity_cast_parses() {
        const TEXT: &str = r#"{
            size: [1, 1, 1],
            palette: [{Name: "minecraft:rail"}],
            blocks: [{pos: [0, 0, 0], state: 0}],
            entities: [
                {pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:furnace_minecart", Fuel: 3s, PushX: 0.5d, PushZ: -1.0d}},
                {pos: [1.5d, 0.0d, 0.5d], nbt: {id: "minecraft:furnace_minecart"}},
                {pos: [2.5d, 0.0d, 0.5d], nbt: {id: "minecraft:small_fireball"}},
                {pos: [3.5d, 0.0d, 0.5d], nbt: {id: "minecraft:dragon_fireball"}},
                {pos: [4.5d, 0.0d, 0.5d], nbt: {id: "minecraft:villager"}},
                {pos: [5.5d, 0.0d, 0.5d], nbt: {id: "minecraft:oak_boat"}},
                {pos: [6.5d, 0.0d, 0.5d], nbt: {id: "minecraft:armor_stand"}}
            ]
        }"#;
        let s = Structure::parse(TEXT).unwrap();
        assert_eq!(s.entities.len(), 7);
        match &s.entities[0] {
            SpawnedEntity::FurnaceMinecart(cart) => {
                assert_eq!(cart.fuel, 3);
                assert_eq!(cart.push, [0.5, -1.0]);
            }
            other => panic!("expected a furnace minecart, got {other:?}"),
        }
        match &s.entities[1] {
            // Absent `Fuel`/`Push*` mean a cart with no drive, not a parse error.
            SpawnedEntity::FurnaceMinecart(cart) => {
                assert_eq!(cart.fuel, 0);
                assert_eq!(cart.push, [0.0, 0.0]);
            }
            other => panic!("expected a furnace minecart, got {other:?}"),
        }
        // Both fireball sizes keep their kind: the hitboxes differ, and that
        // difference is why the doors use one rather than the other.
        assert_eq!(s.entities[2].kind(), "minecraft:small_fireball");
        assert_eq!(s.entities[3].kind(), "minecraft:dragon_fireball");
        assert!(matches!(s.entities[2], SpawnedEntity::Body(_)));
        assert!(matches!(s.entities[3], SpawnedEntity::Body(_)));
        assert!(matches!(s.entities[4], SpawnedEntity::Body(_)));
        assert_eq!(s.entities[4].kind(), "minecraft:villager");
        // A boat and an armor stand read as the same frozen body, because the
        // reader dispatches on the *motion class* and both rows say `Frozen`.
        // Nothing in this file was edited to let them through — that is the
        // whole claim of the registry, asserted rather than asserted-to.
        assert!(matches!(s.entities[5], SpawnedEntity::Body(_)));
        assert_eq!(s.entities[5].kind(), "minecraft:oak_boat");
        assert!(matches!(s.entities[6], SpawnedEntity::Body(_)));
        assert_eq!(s.entities[6].kind(), "minecraft:armor_stand");
        // None of them are items, so none reach the item-entity view.
        assert!(s.item_entities.is_empty());
    }

    /// The exact entity state from `boat_fence.litematic`.
    ///
    /// The lowercase `leash` array is the 26.2 spelling. Its source-world
    /// coordinates are not useful after the litematic is moved, but the tag's
    /// presence is: it distinguishes a tether-rest correction from an
    /// unsupported free boat with the same nonzero Motion.
    #[test]
    fn the_boat_fence_entity_keeps_its_leash_and_motion() {
        const TEXT: &str = r#"{
            DataVersion: 4903,
            size: [3, 7, 5],
            palette: [{Name: "minecraft:oak_fence"}],
            blocks: [{pos: [1, 6, 3], state: 0}],
            entities: [{
                pos: [1.488202109234411d, 0.04504328578133254d, 2.8126012571156025d],
                blockPos: [1, 0, 2],
                nbt: {
                    id: "minecraft:oak_boat",
                    Motion: [
                        -0.00000000000000043238883150846266d,
                        0.011681267954871115d,
                        -0.00000000000000000000000000000000000000000000000000000000000000000000000000000000000036110308073897784d
                    ],
                    leash: [I; -23, -52, -46]
                }
            }]
        }"#;

        let structure = Structure::parse(TEXT).expect("the supplied boat fixture parses");
        let SpawnedEntity::Body(boat) = &structure.entities[0] else {
            panic!("expected the oak boat to remain a body");
        };
        assert_eq!(boat.kind, "minecraft:oak_boat");
        assert!(boat.leashed, "the 26.2 lowercase leash tag was dropped");
        assert_eq!(
            boat.motion,
            [
                -4.323_888_315_084_626_6e-16,
                0.011_681_267_954_871_115,
                -3.611_030_807_389_778_4e-85,
            ]
        );
        assert!(boat.passengers.is_empty());
    }

    /// A representative boat/rider pair from `Elevator Decorated.litematic`.
    ///
    /// Litematica keeps the nested rider's source-world `Pos` while the
    /// top-level boat is region-relative. That mismatch is harmless: vanilla
    /// and this engine both derive the live passenger position from the
    /// vehicle seat. What must survive parsing is the relationship itself.
    #[test]
    fn the_elevator_boat_keeps_its_silverfish_passenger() {
        const TEXT: &str = r#"{
            DataVersion: 4903,
            size: [23, 146, 31],
            palette: [{Name: "minecraft:stone"}],
            blocks: [],
            entities: [{
                pos: [5.6875d, 61.0625d, 16.748073537581092d],
                nbt: {
                    id: "minecraft:pale_oak_boat",
                    UUID: [I; -2028611585, -1291040436, -1882018199, 1996411432],
                    Motion: [-5e-324d, 0.0d, -0.000005059650645427016d],
                    leash: {UUID: [I; -1279546162, 300368672, -1901651432, 933286764]},
                    Passengers: [{
                        id: "minecraft:silverfish",
                        UUID: [I; 1031472237, 1891585571, -1719794062, 603555933],
                        Pos: [-103.3125d, 4.25d, 128.7480735375811d],
                        Motion: [-0.00455000002942979d, -0.0784000015258789d, 0.0000004362258015156135d]
                    }]
                }
            }]
        }"#;

        let structure = Structure::parse(TEXT).expect("the elevator pair parses");
        let SpawnedEntity::Body(boat) = &structure.entities[0] else {
            panic!("expected a pale-oak boat body");
        };
        assert_eq!(boat.kind, "minecraft:pale_oak_boat");
        assert!(boat.leashed);
        assert_eq!(boat.passengers.len(), 1);
        let SpawnedEntity::Body(rider) = &boat.passengers[0] else {
            panic!("expected a frozen silverfish rider");
        };
        assert_eq!(rider.kind, "minecraft:silverfish");
        assert_eq!(rider.pos, [-103.3125, 4.25, 128.748_073_537_581_1]);
        assert_eq!(
            rider.motion,
            [
                -0.004_550_000_029_429_79,
                -0.078_400_001_525_878_9,
                4.362_258_015_156_135e-7,
            ]
        );
    }

    /// A type with no representation at all is refused by name, in its own
    /// error variant.
    ///
    /// Note what this is *not*: it is not "the engine has no behaviour for
    /// this" — that question is asked at construction, and a furnace minecart
    /// parses fine here. This is a type the reader cannot carry at all, so
    /// there is nothing to hand on.
    ///
    /// The variant matters as much as the message: the bridge tells the user
    /// "your build has a creeper in it" only if it can distinguish this from
    /// "our converter emitted garbage", and those read identically once both
    /// collapse into `Malformed`.
    #[test]
    fn an_unrepresentable_entity_type_is_refused_by_name() {
        const TEXT: &str = r#"{
            size: [1, 1, 1],
            palette: [{Name: "minecraft:rail"}],
            blocks: [{pos: [0, 0, 0], state: 0}],
            entities: [{pos: [0.5d, 0.0d, 0.5d], nbt: {id: "minecraft:creeper"}}]
        }"#;
        let err = Structure::parse(TEXT).unwrap_err();
        match &err {
            StructureError::UnsupportedEntity { entity_type, .. } => {
                assert_eq!(entity_type, "minecraft:creeper");
            }
            other => panic!("expected UnsupportedEntity, got {other:?}"),
        }
        assert!(err.to_string().contains("minecraft:creeper"), "{err}");
    }
}
