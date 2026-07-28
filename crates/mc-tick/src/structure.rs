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
    /// Every authored entity, in list order — the placement spawn order,
    /// which is also the server's id-assignment order.
    pub entities: Vec<SpawnedEntity>,
    /// Item entities authored in the structure's `entities` list.
    ///
    /// The RNG-free way to put an item into the world: authored positions and
    /// motion, no dispenser jitter. Only `minecraft:item` is understood; any
    /// other entity type is a loud parse error rather than a silent hole.
    pub item_entities: Vec<SpawnedItem>,
}

/// One authored entity, by type.
#[derive(Debug, Clone, PartialEq)]
pub enum SpawnedEntity {
    /// An item entity.
    Item(SpawnedItem),
    /// A minecart.
    Minecart(SpawnedMinecart),
}

/// An authored minecart.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnedMinecart {
    /// e.g. `minecraft:minecart`.
    pub kind: String,
    /// Spawn position.
    pub pos: [f64; 3],
    /// Spawn velocity.
    pub motion: [f64; 3],
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
        if self.peek() == Some(b'"') {
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

    fn string(&mut self) -> Result<String, StructureError> {
        self.eat(b'"')?;
        let mut out = String::new();
        while self.at < self.text.len() {
            match self.text[self.at] {
                b'"' => {
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
                        b'"' => {
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
            Some(b'"') => self.string().map(|_| ()),
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

    /// A floating-point number, tolerating NBT's `d`/`f` suffixes.
    fn float(&mut self) -> Result<f64, StructureError> {
        self.skip_ws();
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        while self.at < self.text.len() {
            let c = self.text[self.at] as char;
            if c.is_ascii_digit() || c == '.' {
                self.at += 1;
            } else {
                break;
            }
        }
        if start == self.at {
            return self.err("expected a number");
        }
        let digits = String::from_utf8_lossy(&self.text[start..self.at]).into_owned();
        if self.at < self.text.len() && (self.text[self.at] as char).is_alphabetic() {
            self.at += 1;
        }
        digits
            .parse()
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

    /// One `entities` entry: an authored item entity or minecart.
    fn entity_entry(&mut self) -> Result<SpawnedEntity, StructureError> {
        self.eat(b'{')?;
        let mut pos: Option<[f64; 3]> = None;
        let mut motion = [0.0f64; 3];
        let mut item: Option<(String, u8)> = None;
        let mut pickup_delay = 0u32;
        let mut entity_id = String::new();
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
                    // The entity compound: id, Item, Motion, PickupDelay.
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
                            "Item" => {
                                let stack = self.item_entry()?;
                                item = Some((stack.id, stack.count));
                            }
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
        match entity_id.as_str() {
            "minecraft:item" => match (pos, item) {
                (Some(pos), Some(item)) => {
                    Ok(SpawnedEntity::Item(SpawnedItem { pos, motion, item, pickup_delay }))
                }
                _ => self.err("item entity needs `pos` and `Item`"),
            },
            // Only the plain rideable cart is implemented; the container
            // variants stay loud until their behaviours exist.
            "minecraft:minecart" => match pos {
                Some(pos) => Ok(SpawnedEntity::Minecart(SpawnedMinecart {
                    kind: entity_id,
                    pos,
                    motion,
                })),
                None => self.err("minecart entity needs `pos`"),
            },
            _ => self.err(format!("unsupported entity type `{entity_id}`")),
        }
    }

    /// A block-entity `nbt` compound: the `Items` list and the comparator's
    /// `OutputSignal`, skipping the rest.
    ///
    /// `OutputSignal` is not decoration. A comparator emits its *stored*
    /// block-entity strength rather than anything its block state carries, so a
    /// build saved mid-cycle starts with whatever each comparator last emitted.
    /// Dropping it made every comparator in a loaded door start at zero, which
    /// is true of a freshly placed one and false of a saved one.
    fn nbt_items(
        &mut self,
    ) -> Result<(Vec<crate::inventory::ItemStack>, Option<u8>), StructureError> {
        self.eat(b'{')?;
        let mut items = Vec::new();
        let mut output_signal = None;
        loop {
            if self.peek() == Some(b'}') {
                self.at += 1;
                return Ok((items, output_signal));
            }
            let key = self.key()?;
            self.eat(b':')?;
            if key == "OutputSignal" {
                output_signal = Some(self.int()? as u8);
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
        let mut block_entities: Vec<Pos> = Vec::new();
        let mut comparator_outputs: Vec<(Pos, u8)> = Vec::new();
        let mut entities: Vec<SpawnedEntity> = Vec::new();

        loop {
            if self.peek() == Some(b'}') {
                self.at += 1;
                break;
            }
            let key = self.key()?;
            self.eat(b':')?;
            match key.as_str() {
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
                                if !items.is_empty() {
                                    inventories.push((p, items));
                                }
                                if has_nbt {
                                    block_entities.push(p);
                                }
                                if let Some(signal) = output_signal {
                                    comparator_outputs.push((p, signal));
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
            size: size.ok_or(StructureError::Missing("size"))?,
            palette: palette.ok_or(StructureError::Missing("palette"))?,
            blocks: blocks.ok_or(StructureError::Missing("blocks"))?,
            inventories,
            comparator_outputs,
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
}
