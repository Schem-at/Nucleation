//! Wiring Minecraft block descriptors to behaviour.
//!
//! Everything else in this crate is deliberately ignorant of Minecraft's block
//! list: [`StateRegistry`] interns opaque strings, and behaviours are registered
//! one state at a time. That keeps the engine free of a data dependency and a
//! version dependency, and it is the right shape for a core.
//!
//! It is the wrong shape for *using* the thing. Running a real schematic means
//! turning `minecraft:sticky_piston[extended=false,facing=east]` into a
//! [`Piston`] with the right facing, and doing that by hand for the hundreds of
//! states in a build is not work anybody should do twice.
//!
//! This module is that translation, and only that. It parses a descriptor into a
//! name and properties, decides which behaviour the name implies, and registers
//! it. Nothing here knows how the tick loop works.
//!
//! # What it deliberately does not do
//!
//! It does not guess. A block it does not recognise is left **unregistered**, so
//! [`BehaviourTable::unknown_report`] names it and a conformance run fails loudly
//! rather than simulating a contraption with a hole in it. Growing the coverage is
//! a matter of adding arms below, each backed by a captured trace.

use crate::behaviour::{BehaviourTable, Inert};
use crate::components::{
    Button, Comparator, ComparatorMode, CopperBulb, Door, Dropper, Hopper, Lamp, NoteBlock,
    PowerSource, PressurePlate, Repeater, StatePair, Torch, Trapdoor,
};
use crate::observer::Observer;
use crate::piston::{Movability, Piston, Sticky};
use crate::pos::{Dir, Pos};
use crate::state::{StateId, StateRegistry};
use crate::world::World;
use std::collections::HashMap;

/// A block descriptor split into its name and properties.
///
/// `minecraft:repeater[delay=2,facing=north]` becomes
/// `("minecraft:repeater", {delay: "2", facing: "north"})`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Descriptor {
    /// The block's identifier.
    pub name: String,
    /// Its properties, in the order they appeared.
    pub properties: Vec<(String, String)>,
}

impl Descriptor {
    /// Split a descriptor string.
    pub fn parse(text: &str) -> Self {
        match text.split_once('[') {
            None => Descriptor { name: text.to_string(), properties: Vec::new() },
            Some((name, rest)) => {
                let rest = rest.strip_suffix(']').unwrap_or(rest);
                let properties = rest
                    .split(',')
                    .filter(|part| !part.is_empty())
                    .filter_map(|part| part.split_once('='))
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                Descriptor { name: name.to_string(), properties }
            }
        }
    }

    /// A property's value.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// A boolean property, defaulting to false.
    pub fn flag(&self, key: &str) -> bool {
        self.get(key) == Some("true")
    }

    /// The `facing` property as a direction.
    pub fn facing(&self) -> Option<Dir> {
        match self.get("facing")? {
            "down" => Some(Dir::Down),
            "up" => Some(Dir::Up),
            "north" => Some(Dir::North),
            "south" => Some(Dir::South),
            "west" => Some(Dir::West),
            "east" => Some(Dir::East),
            _ => None,
        }
    }

    /// Rebuild a descriptor with one property replaced.
    ///
    /// Used to find a block's opposite state — the powered twin of an unpowered
    /// repeater, say — without the caller assembling strings by hand.
    pub fn with(&self, key: &str, value: &str) -> String {
        let mut properties: Vec<(String, String)> = self
            .properties
            .iter()
            .map(|(k, v)| {
                if k == key {
                    (k.clone(), value.to_string())
                } else {
                    (k.clone(), v.clone())
                }
            })
            .collect();
        if !properties.iter().any(|(k, _)| k == key) {
            properties.push((key.to_string(), value.to_string()));
        }
        properties.sort();
        if properties.is_empty() {
            return self.name.clone();
        }
        let rendered: Vec<String> = properties
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!("{}[{}]", self.name, rendered.join(","))
    }
}

/// Minecraft's power and movability rules, over whatever states have been seen.
///
/// Built alongside the behaviour registration so both agree on which blocks emit
/// power and which can be shoved.
#[derive(Debug, Clone, Default)]
pub struct VanillaRules {
    /// Where this world's `(0, 0, 0)` sits in the game's coordinates — see
    /// [`crate::wire::WireWorld::hash_origin`]. Zero unless a capture says
    /// otherwise.
    hash_origin: Pos,
    powered: Vec<StateId>,
    /// States that emit a *level* rather than a flat 15.
    ///
    /// A weighted pressure plate's signal is the number of entities on it
    /// (`WeightedPressurePlateBlock.getSignalStrength`), carried in its `power`
    /// property. Everything else in `powered` answers 15, so without this a
    /// plate holding one item would drive a full-strength line.
    analog_emission: HashMap<StateId, u8>,
    /// States that emit in **one** direction only, and which one.
    ///
    /// An observer powers only out of its back — `ObserverBlock.getSignal`
    /// checks the queried direction against `FACING`. Reading it as an
    /// omnidirectional source made a pulsing observer power the very note block
    /// it was watching, which re-triggered it forever. States absent from this
    /// map emit every way, like a redstone block.
    emit_only: HashMap<StateId, Dir>,
    /// States that **strongly** power the block in the given direction.
    ///
    /// A strongly powered conductor re-emits weak power on every face —
    /// `Level.getSignal` falls back to `getDirectSignal` when the queried block
    /// `isRedstoneConductor`. This is how an observer drives a piston through a
    /// slime block (captured: `flying_machine.json`, tick 1).
    strong_into: HashMap<StateId, Dir>,
    /// States that conduct strong power.
    ///
    /// Growing this list is capture-driven: slime is on it because the flying
    /// machine's trace proves the signal crossed it. Glass famously is not.
    conductors: Vec<StateId>,
    /// Container states and their slot counts, for the comparator's analog read.
    containers: HashMap<StateId, u32>,
    /// Hopper states, for the destination-cooldown rule.
    hoppers: Vec<StateId>,
    /// Full-cube states, for hopper-suction blocking and item collision.
    full_cubes: Vec<StateId>,
    /// Wire states: power level and horizontal connections (true = side or up).
    wires: HashMap<StateId, (u8, [crate::wire::WireSide; 4])>,
    /// `(power, connections)` -> the wire state with that shape, for the
    /// connection recompute in `RedStoneWireBlock.updateShape`.
    wire_shapes: HashMap<(u8, [crate::wire::WireSide; 4]), StateId>,
    /// `isSignalSource`: whether the block *can* emit, powered or not — which
    /// is what decides whether dust turns to face it.
    signal_sources: Vec<StateId>,
    /// Blocks dust can climb: `canSurviveOn`, i.e. a sturdy upward face.
    sturdy_up: Vec<StateId>,
    /// `PushReaction.DESTROY`: broken by a push rather than carried.
    destroyed_by_push: Vec<StateId>,
    /// A leaf state's `distance`, and the log states that count as distance 0.
    leaf_distance: HashMap<StateId, u8>,
    logs: Vec<StateId>,
    /// Repeater states, which dust faces only along their axis.
    repeaters: Vec<StateId>,
    /// Observer states and the direction they look, which is the only face
    /// dust turns toward.
    observer_facing: HashMap<StateId, Dir>,
    /// `(wire state, power)` -> the same shape at that power.
    wire_siblings: HashMap<(StateId, u8), StateId>,
    immovable: Vec<StateId>,
    slime: Vec<StateId>,
    honey: Vec<StateId>,
    diodes: HashMap<StateId, Dir>,
    /// Water per state: plain water blocks, waterlogged states and bubble
    /// columns (`getFluidState`).
    waters: HashMap<StateId, crate::fluid::WaterKind>,
    /// The plain `minecraft:water` state for each legacy `level` value.
    water_levels: HashMap<u8, StateId>,
    /// Bubble columns: `Some(drag_down)`.
    bubbles: HashMap<StateId, bool>,
    /// Comparator states, whose emission is their stored strength.
    comparators: Vec<StateId>,
    /// States that emit in every direction **except** one — a lit redstone
    /// torch, which powers everything but the block beneath it
    /// (`RedstoneTorchBlock.getSignal` returns 0 for a query from below).
    emit_except: HashMap<StateId, Dir>,
    /// States whose comparator read comes from the block state itself —
    /// a composter's `level` (0-8).
    state_analog: HashMap<StateId, u8>,
}

impl VanillaRules {
    /// What the block at `pos` emits toward `toward` on its own — vanilla's
    /// `BlockState.getSignal`, plus dust's own emission rules.
    ///
    /// `with_wires` is `shouldSignal`: the dust evaluator turns wire
    /// contributions off while computing a wire's own target, which is what
    /// stops a wire feeding on the floor block it powers.
    fn emitted(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
        toward: Dir,
        with_wires: bool,
    ) -> u8 {
        let state = world.get(pos);
        if self.powered.contains(&state)
            && self.emit_only.get(&state).is_none_or(|only| *only == toward)
            && self.emit_except.get(&state).is_none_or(|except| *except != toward)
        {
            // A comparator emits its **stored block-entity strength**, not a
            // flat 15 — and a freshly placed one holds 0 even while its block
            // state says `powered=true`.
            return if self.comparators.contains(&state) {
                outs.get(&pos).copied().unwrap_or(0)
            } else if let Some(&level) = self.analog_emission.get(&state) {
                level
            } else {
                15
            };
        }
        if !with_wires {
            return 0;
        }
        // Dust powers the block beneath it and the sides it connects to,
        // never the block above.
        //
        // The connections are **recomputed here**, not read from the state.
        // `RedStoneWireBlock.getSignal` calls `getConnectionState(level, state,
        // pos)` on every query, so what a wire powers follows the world as it
        // is now — not the shape the schematic was saved with. It matters
        // during placement: a wire saved climbing a block only powers that
        // block once the dust it climbs toward exists, and until then the
        // block is dark. A torch beside it reads that darkness and books the
        // tick that vanilla books.
        if let Some((power, _)) = self.wires.get(&state) {
            if *power > 0 {
                let connections = {
                    let stored = self
                        .wires
                        .get(&state)
                        .map(|(_, c)| *c)
                        .unwrap_or([crate::wire::WireSide::None; 4]);
                    crate::wire::connection_state(self, world, pos, stored)
                };
                match toward {
                    Dir::Down => return *power,
                    Dir::Up => {}
                    side => {
                        let index = match side {
                            Dir::North => 0,
                            Dir::South => 1,
                            Dir::West => 2,
                            _ => 3,
                        };
                        if connections[index] != crate::wire::WireSide::None {
                            return *power;
                        }
                    }
                }
            }
        }
        0
    }

    /// `Level.getDirectSignalTo`: the strongest *strong* signal into `pos`.
    ///
    /// Dust is the subtle one. `RedStoneWireBlock.getDirectSignal` delegates
    /// straight to `getSignal`, so dust strongly powers **everything it powers
    /// weakly** — the block below it *and* every side it connects into. A
    /// community door hangs on exactly that: dust running beside a solid block
    /// makes that block a source for anything touching it.
    fn direct_signal_to(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
        with_wires: bool,
    ) -> u8 {
        let mut best = 0u8;
        for dir in crate::pos::ALL_DIRS {
            let neighbour = pos.offset(dir);
            let toward = dir.opposite();
            let state = world.get(neighbour);
            let strong = if self.wires.contains_key(&state) {
                self.emitted(world, outs, neighbour, toward, with_wires)
            } else if self.strong_into.get(&state) == Some(&toward) {
                self.emitted(world, outs, neighbour, toward, with_wires)
            } else {
                0
            };
            best = best.max(strong);
        }
        best
    }

    /// `signal_strength` with `shouldSignal` off — every wire contribution
    /// muted, including dust strongly powering the conductor it sits on. This
    /// is what the dust evaluator's `getBlockSignal` reads.
    fn signal_no_wire(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
        toward: Dir,
    ) -> u8 {
        let mut strength = self.emitted(world, outs, pos, toward, false);
        if self.conductors.contains(&world.get(pos)) {
            strength = strength.max(self.direct_signal_to(world, outs, pos, false));
        }
        strength
    }

    /// Why `pos` does or does not power `toward` — a diagnostic for tracing a
    /// dust level back to its source.
    pub fn explain_power(&self, world: &World, pos: Pos, toward: Dir) -> String {
        let outs = crate::behaviour::ComparatorOutputs::new();
        let state = world.get(pos);
        format!(
            "emitted={} direct_to={} conductor={} => {}",
            self.emitted(world, &outs, pos, toward, true),
            self.direct_signal_to(world, &outs, pos, true),
            self.conductors.contains(&state),
            self.signal_strength(world, &outs, pos, toward)
        )
    }
}

impl crate::wire::WireWorld for VanillaRules {
    fn hash_origin(&self) -> Pos {
        self.hash_origin
    }

    /// `getBlockSignal`: the strongest non-wire signal into the wire.
    ///
    /// Comparator strengths, strongly powered conductors and one-directional
    /// diodes all fall out of the shared power model — the wire path used to
    /// special-case comparators and answer a flat 15 for everything else,
    /// which lit a whole door's dust from a comparator that was emitting 0.
    fn block_signal(&self, ctx: &crate::behaviour::TickCtx<'_>, pos: Pos) -> u8 {
        let mut best = 0u8;
        for dir in crate::pos::ALL_DIRS {
            best = best.max(self.signal_no_wire(
                ctx.world,
                ctx.comparator_out,
                pos.offset(dir),
                dir.opposite(),
            ));
            if best == 15 {
                break;
            }
        }
        best
    }

    fn conductor(&self, world: &World, pos: Pos) -> bool {
        // isRedstoneConductor: the diagonal rules run on conductivity, which
        // is what makes glass a diode — full-cube-ness is not enough.
        self.conductors.contains(&world.get(pos))
    }

    fn wire_power(&self, world: &World, pos: Pos) -> Option<u8> {
        self.wires.get(&world.get(pos)).map(|(power, _)| *power)
    }

    fn wire_with_power(&self, world: &World, pos: Pos, power: u8) -> Option<StateId> {
        let state = world.get(pos);
        self.wire_siblings.get(&(state, power)).copied()
    }

    fn wire_shape(&self, world: &World, pos: Pos) -> Option<(u8, [crate::wire::WireSide; 4])> {
        self.wires.get(&world.get(pos)).copied()
    }

    fn wire_with_shape(
        &self,
        power: u8,
        sides: [crate::wire::WireSide; 4],
    ) -> Option<StateId> {
        self.wire_shapes.get(&(power, sides)).copied()
    }

    fn should_connect_to(&self, world: &World, pos: Pos, from: Option<Dir>) -> bool {
        let state = world.get(pos);
        if self.wires.contains_key(&state) {
            return true;
        }
        // A repeater takes a signal on its input face and gives one on its
        // output face, so dust faces it along its axis and not across it. An
        // observer only counts on the face it looks along. A comparator has no
        // special case in `shouldConnectTo` — it connects like any other
        // source, which is why dust wraps around its sides.
        if self.repeaters.contains(&state) {
            let facing = self.diodes.get(&state);
            return from.is_some_and(|dir| {
                facing.is_some_and(|f| dir == *f || dir == f.opposite())
            });
        }
        if let Some(facing) = self.observer_facing.get(&state) {
            return from == Some(*facing);
        }
        // Everything else that can emit connects, but only when asked about a
        // real direction: the diagonal checks pass `None` and want dust only.
        self.signal_sources.contains(&state) && from.is_some()
    }

    fn sturdy_up(&self, world: &World, pos: Pos) -> bool {
        self.sturdy_up.contains(&world.get(pos))
    }

    fn full_block(&self, world: &World, pos: Pos) -> bool {
        self.full_cubes.contains(&world.get(pos))
    }
}

impl crate::fluid::FluidWorld for VanillaRules {
    fn water(&self, world: &World, pos: Pos) -> Option<crate::fluid::WaterKind> {
        self.waters.get(&world.get(pos)).copied()
    }

    fn can_flow_into(&self, world: &World, pos: Pos) -> bool {
        // Air only; vanilla also floods replaceable plants, which this engine
        // does not model yet.
        world.get(pos) == StateId::AIR
    }

    fn is_solid(&self, world: &World, pos: Pos) -> bool {
        self.full_cubes.contains(&world.get(pos))
    }

    fn water_state(&self, level: u8) -> Option<StateId> {
        self.water_levels.get(&level).copied()
    }
}

impl PowerSource for VanillaRules {
    fn analog_signal(
        &self,
        world: &World,
        inventories: &crate::inventory::InventoryMap,
        pos: Pos,
    ) -> Option<u8> {
        // State-derived reads first: a composter's level is its signal.
        if let Some(level) = self.state_analog.get(&world.get(pos)) {
            return Some(*level);
        }
        let slots = *self.containers.get(&world.get(pos))?;
        // A container with no recorded contents is an empty container — its
        // analog output is a real 0, not an absence.
        Some(
            inventories
                .get(&pos)
                .map_or(0, crate::inventory::Inventory::analog_signal)
                .min(if slots == 0 { 0 } else { 15 }),
        )
    }

    fn is_conductor(&self, world: &World, pos: Pos) -> bool {
        self.conductors.contains(&world.get(pos))
    }

    fn leaf_distance(&self, world: &World, pos: Pos) -> u8 {
        let state = world.get(pos);
        if self.logs.contains(&state) {
            return 0;
        }
        self.leaf_distance.get(&state).copied().unwrap_or(7)
    }

    fn container_slots_at(&self, world: &World, pos: Pos) -> Option<u32> {
        self.containers.get(&world.get(pos)).copied()
    }

    fn hopper_at(&self, world: &World, pos: Pos) -> bool {
        self.hoppers.contains(&world.get(pos))
    }

    fn is_solid_at(&self, world: &World, pos: Pos) -> bool {
        self.full_cubes.contains(&world.get(pos))
    }

    fn is_powered(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
        toward: Dir,
    ) -> bool {
        self.signal_strength(world, outs, pos, toward) > 0
    }

    /// `Level.getSignal`: the block's own emission, and — when it conducts —
    /// the strongest strong signal into it.
    fn signal_strength(
        &self,
        world: &World,
        outs: &crate::behaviour::ComparatorOutputs,
        pos: Pos,
        toward: Dir,
    ) -> u8 {
        let mut strength = self.emitted(world, outs, pos, toward, true);
        if self.conductors.contains(&world.get(pos)) {
            strength = strength.max(self.direct_signal_to(world, outs, pos, true));
        }
        strength
    }
    fn is_diode(&self, world: &World, pos: Pos) -> bool {
        self.diodes.contains_key(&world.get(pos))
    }
    fn diode_facing(&self, world: &World, pos: Pos) -> Option<Dir> {
        self.diodes.get(&world.get(pos)).copied()
    }
}

impl Movability for VanillaRules {
    fn destroys(&self, world: &World, pos: Pos) -> bool {
        self.destroyed_by_push.contains(&world.get(pos))
    }

    fn is_movable(&self, world: &World, pos: Pos) -> bool {
        let state = world.get(pos);
        state != StateId::AIR && !self.immovable.contains(&state)
    }
    fn sticky(&self, world: &World, pos: Pos) -> Option<Sticky> {
        let state = world.get(pos);
        if self.slime.contains(&state) {
            Some(Sticky::Slime)
        } else if self.honey.contains(&state) {
            Some(Sticky::Honey)
        } else {
            None
        }
    }
}

/// Blocks that emit a constant full-strength signal.
const CONSTANT_SOURCES: &[&str] = &["minecraft:redstone_block"];

/// Blocks nothing can push.
const IMMOVABLE: &[&str] = &[
    // Blocks with block entities have PushReaction.BLOCK.
    "minecraft:barrel",
    "minecraft:chest",
    "minecraft:trapped_chest",
    "minecraft:hopper",
    "minecraft:dropper",
    "minecraft:dispenser",
    "minecraft:obsidian",
    "minecraft:crying_obsidian",
    "minecraft:bedrock",
    "minecraft:barrier",
    "minecraft:moving_piston",
    "minecraft:piston_head",
    "minecraft:jukebox",
    "minecraft:furnace",
    "minecraft:blast_furnace",
    "minecraft:smoker",
    // Shulker boxes of every colour are also immovable; they match by name
    // pattern in `register_all_at` rather than sixteen entries here.
];

/// Blocks with no behaviour at all, but which are legitimate build material.
///
/// Listing one here asserts it is inert. It is not a way to silence a block that
/// simply has not been implemented — that is what leaving it unregistered is for.
const INERT: &[&str] = &[
    "minecraft:stone",
    "minecraft:smooth_stone",
    "minecraft:cobblestone",
    "minecraft:dirt",
    "minecraft:grass_block",
    "minecraft:obsidian",
    "minecraft:bedrock",
    "minecraft:barrier",
    "minecraft:quartz_block",
    "minecraft:netherite_block",
    "minecraft:iron_block",
    "minecraft:gold_block",
    "minecraft:glass",
    "minecraft:white_stained_glass",
    "minecraft:barrel",
    "minecraft:chest",
    "minecraft:trapped_chest",
    "minecraft:sea_lantern",
    "minecraft:slime_block",
    "minecraft:honey_block",
    "minecraft:moving_piston",
    "minecraft:redstone_block",
    "minecraft:soul_sand",
    "minecraft:magma_block",
    "minecraft:ice",
    "minecraft:packed_ice",
    "minecraft:blue_ice",
    "minecraft:cobweb",
    // Door-build material, asserted inert (the census of the first five
    // community doors).
    "minecraft:chiseled_quartz_block",
    "minecraft:quartz_block",
    "minecraft:white_concrete",
    "minecraft:cyan_concrete",
    "minecraft:lime_concrete",
    // The floor the vault door stands on, which only an in-world recording
    // includes — a pasted build brings the machine and none of its setting.
    "minecraft:gray_concrete",
    "minecraft:cyan_wool",
    "minecraft:lime_wool",
    "minecraft:orange_wool",
    "minecraft:pink_wool",
    "minecraft:red_wool",
    "minecraft:oak_wood",
    "minecraft:smooth_stone_slab",
    "minecraft:composter",
    "minecraft:target",
    "minecraft:jukebox",
    // Shulker boxes (all sixteen colours plus undyed) are inert too; they
    // match by name pattern in the second registration pass.
    "minecraft:birch_wall_sign",
    "minecraft:player_wall_head",
    "minecraft:lightning_rod",
    "minecraft:tripwire_hook",
    // Smelting is not modelled, but a furnace is comparator-readable and its
    // slot count is carried in `container_slots`, so a stocked one still
    // reads correctly.
    "minecraft:furnace",
    "minecraft:blast_furnace",
    "minecraft:smoker",
];

/// Register vanilla behaviour for every state currently in `registry`.
///
/// Returns the rules, which callers may keep to interrogate power or movability.
/// Any state whose block is not recognised is left unregistered on purpose; see the
/// module docs.
pub fn register_all(registry: &mut StateRegistry, table: &mut BehaviourTable) -> VanillaRules {
    register_all_at(registry, table, Pos::new(0, 0, 0))
}

/// As [`register_all`], for a build that sits at `origin` in the game's own
/// coordinates.
///
/// The only thing this changes is the iteration order of the `HashSet<BlockPos>`
/// in `updatePowerStrength`, which is a function of absolute position — so a
/// trace recorded where a build stands can only be reproduced by hashing the
/// positions it was recorded at.
pub fn register_all_at(
    registry: &mut StateRegistry,
    table: &mut BehaviourTable,
    origin: Pos,
) -> VanillaRules {
    // Two passes. The first classifies every state, because a piston's behaviour
    // needs to know which blocks are sticky and which emit power — facts about
    // *other* states that may not have been seen yet when it is reached.
    let descriptors: Vec<(StateId, Descriptor)> = (0..registry.len())
        .map(|i| StateId(i as u16))
        .filter_map(|id| registry.descriptor(id).map(|d| (id, Descriptor::parse(d))))
        .collect();

    let mut rules = VanillaRules { hash_origin: origin, ..VanillaRules::default() };
    for (id, descriptor) in &descriptors {
        match descriptor.name.as_str() {
            n if CONSTANT_SOURCES.contains(&n) => rules.powered.push(*id),
            "minecraft:slime_block" => {
                rules.slime.push(*id);
            }
            "minecraft:honey_block" => rules.honey.push(*id),
            n if container_slots(n).is_some() => {
                rules.containers.insert(*id, container_slots(n).unwrap());
                if n == "minecraft:hopper" {
                    rules.hoppers.push(*id);
                }
            }
            _ => {}
        }
        if IMMOVABLE.contains(&descriptor.name.as_str()) || is_shulker_box(&descriptor.name) {
            rules.immovable.push(*id);
        }
        // `PistonBaseBlock.getPistonPushReaction` answers BLOCK while
        // EXTENDED: an extended piston base cannot be pushed, though a
        // retracted one can. The immovable list keys on block *name*, so this
        // state-dependent case needs saying separately — without it a slime
        // structure happily dragged an extended piston along, and a flying
        // machine performed a push the game refuses.
        if matches!(
            descriptor.name.as_str(),
            "minecraft:piston" | "minecraft:sticky_piston"
        ) && descriptor.flag("extended")
        {
            rules.immovable.push(*id);
        }
        if is_full_cube(descriptor) {
            rules.full_cubes.push(*id);
        }
        if is_conductor(descriptor) {
            rules.conductors.push(*id);
        }
        match descriptor.name.as_str() {
            "minecraft:water" => {
                let level = descriptor.get("level").and_then(|l| l.parse().ok()).unwrap_or(0);
                rules
                    .waters
                    .insert(*id, crate::fluid::WaterKind::from_level(level));
                rules.water_levels.insert(level, *id);
            }
            "minecraft:bubble_column" => {
                // A bubble column's fluid state is a full water source.
                rules.waters.insert(*id, crate::fluid::WaterKind::Source);
                rules.bubbles.insert(*id, descriptor.get("drag") == Some("true"));
            }
            // A comparator reads a cauldron's fill level directly: 0 when
            // empty, 1-3 for the layers of a water or powder-snow cauldron,
            // and a full 3 for lava, which has no partial states.
            n if n.ends_with("cauldron") => {
                let level = if n == "minecraft:lava_cauldron" {
                    3
                } else {
                    descriptor.get("level").and_then(|l| l.parse().ok()).unwrap_or(0)
                };
                rules.state_analog.insert(*id, level);
            }
            "minecraft:composter" => {
                let level = descriptor.get("level").and_then(|l| l.parse().ok()).unwrap_or(0);
                rules.state_analog.insert(*id, level);
            }
            _ => {
                if descriptor.get("waterlogged") == Some("true") {
                    rules.waters.insert(*id, crate::fluid::WaterKind::Source);
                }
            }
        }
        if descriptor.name == "minecraft:redstone_wire" {
            let power = descriptor
                .get("power")
                .and_then(|p| p.parse().ok())
                .unwrap_or(0);
            // Connection order matches Dir: North, South, West, East.
            let side = |key: &str| match descriptor.get(key) {
                Some("up") => crate::wire::WireSide::Up,
                Some("side") => crate::wire::WireSide::Side,
                _ => crate::wire::WireSide::None,
            };
            let sides = [side("north"), side("south"), side("west"), side("east")];
            rules.wires.insert(*id, (power, sides));
            rules.wire_shapes.insert((power, sides), *id);
        }
        // A powered diode, torch or observer is itself a source.
        let emits = match descriptor.name.as_str() {
            "minecraft:repeater" | "minecraft:comparator" | "minecraft:observer" => {
                descriptor.flag("powered")
            }
            // A weighted plate carries `power`, not `powered`, and emits that
            // number rather than a flat 15 — see `analog_emission`.
            n if n.ends_with("_weighted_pressure_plate") => {
                weighted_plate_power(descriptor).is_some_and(|power| power > 0)
            }
            n if n.ends_with("_button") || n.ends_with("_pressure_plate") => {
                descriptor.flag("powered")
            }
            // `DetectorRailBlock.getSignal` answers 15 whenever POWERED.
            "minecraft:detector_rail" => descriptor.flag("powered"),
            "minecraft:redstone_torch" | "minecraft:redstone_wall_torch" => descriptor.flag("lit"),
            "minecraft:lever" => descriptor.flag("powered"),
            _ => false,
        };
        // `isSignalSource` is a property of the *block*, not of this state:
        // an unlit torch and an unpowered lever are still signal sources, and
        // dust connects to them either way.
        if matches!(
            descriptor.name.as_str(),
            "minecraft:repeater"
                | "minecraft:comparator"
                | "minecraft:observer"
                | "minecraft:redstone_torch"
                | "minecraft:redstone_wall_torch"
                | "minecraft:lever"
                | "minecraft:redstone_block"
                | "minecraft:daylight_detector"
                | "minecraft:trapped_chest"
                | "minecraft:target"
                // Emitters that never emit in these builds and still matter,
                // because `shouldConnectTo` asks the block whether it is a
                // source and not whether it is emitting. Missing the hook cost
                // the 6x6 door its first divergence: dust that should have
                // faced north found nothing there, and the symmetry rule then
                // ran it east-west instead of north-south. The set is the
                // game's own answer for every block in the corpus.
                | "minecraft:tripwire_hook"
                | "minecraft:lightning_rod"
                | "minecraft:jukebox"
                // `DetectorRailBlock.isSignalSource` is true whatever POWERED
                // says, so dust turns to face one even while it is dark.
                | "minecraft:detector_rail"
        ) || is_button_or_plate(&descriptor.name)
        {
            rules.signal_sources.push(*id);
        }
        if descriptor.name == "minecraft:repeater" {
            rules.repeaters.push(*id);
        }
        if descriptor.name.ends_with("_leaves") {
            let distance = descriptor
                .get("distance")
                .and_then(|d| d.parse().ok())
                .unwrap_or(7);
            rules.leaf_distance.insert(*id, distance);
        }
        // `BlockTags.LOGS` members answer 0, which is what pulls a nearby
        // leaf's distance down to 1.
        if descriptor.name.ends_with("_log")
            || descriptor.name.ends_with("_wood")
            || descriptor.name.ends_with("_stem")
            || descriptor.name.ends_with("_hyphae")
        {
            rules.logs.push(*id);
        }
        // `PushReaction.DESTROY`. Dust and torches are the ones that matter for
        // redstone: a piston pushing into them breaks them, and the break is a
        // power change the circuit has to hear about.
        if matches!(
            descriptor.name.as_str(),
            "minecraft:redstone_wire"
                | "minecraft:redstone_torch"
                | "minecraft:redstone_wall_torch"
                | "minecraft:lever"
                | "minecraft:torch"
                | "minecraft:wall_torch"
                // `DiodeBlock` is `PushReaction.DESTROY`: a repeater or
                // comparator in a piston's way breaks, it does not ride along.
                // Carrying them inflates the push line — a honey slab reaching
                // down to a repeater collected two extra blocks, and a line of
                // thirteen is refused where vanilla's eleven goes through.
                | "minecraft:repeater"
                | "minecraft:comparator"
        )
            // Doors are `PushReaction.DESTROY` — captured, not assumed. A
            // piston reaching either half breaks it, and the other half then
            // breaks for want of a partner (`Door::on_shape_update`). Oak and
            // iron alike, and a slime array carrying both halves at once fares
            // no better. `door_push.json` pins all five arrangements.
            //
            // `_trapdoor` does not end with `_door`, so trapdoors keep riding
            // pistons intact as `trapdoor_push.json` requires.
            || descriptor.name.ends_with("_door")
            // Shulker boxes register `PushReaction.DESTROY` in their block
            // properties (`Blocks` bytecode) — a piston breaks one, and the
            // break drops the box as an item that keeps its slots.
            || is_shulker_box(&descriptor.name)
            || is_button_or_plate(&descriptor.name)
        {
            rules.destroyed_by_push.push(*id);
        }
        if descriptor.name == "minecraft:observer" {
            if let Some(facing) = descriptor.facing() {
                rules.observer_facing.insert(*id, facing);
            }
        }
        // `canSurviveOn`: dust sits on, and climbs, anything with a sturdy top
        // face — full cubes, plus the top halves of slabs and stairs.
        //
        // A `moving_piston` counts. `MovingPistonBlock.getShape` asks the block
        // entity, which answers with the shape of the block it is carrying, so
        // dust rests on a block in motion as it did on the block itself. It is
        // deliberately not a full cube: `getConnectingSide` gates on the top
        // face and then picks UP or SIDE by the face pointing back at the wire,
        // and a carried block that has begun to move no longer fills that one.
        // Without this a wire beside a piston that starts moving drops the
        // connection entirely instead of lowering it from `up` to `side`.
        // An *extended* piston base is a 12/16 box pushed to the far side of
        // its facing, so its top face is a full square only when it faces down —
        // then the body fills the upper twelve sixteenths and the block's own
        // top is the box's top. Facing up the box stops at 12/16; facing
        // sideways the top is 1x0.75. Dust beside a piston that has just
        // extended downward keeps its connection because of this, dropping from
        // `up` to `side` rather than to `none`, since the box is still not a
        // full cube.
        let extended_base_facing_down = matches!(
            descriptor.name.as_str(),
            "minecraft:piston" | "minecraft:sticky_piston"
        ) && descriptor.flag("extended")
            && descriptor.facing() == Some(Dir::Down);
        if is_full_cube(descriptor)
            || extended_base_facing_down
            || descriptor.name == "minecraft:moving_piston"
            || (descriptor.name.ends_with("_slab")
                && matches!(descriptor.get("type"), Some("top") | Some("double")))
            || (descriptor.name.ends_with("_stairs") && descriptor.get("half") == Some("top"))
        {
            rules.sturdy_up.push(*id);
        }
        if emits {
            rules.powered.push(*id);
            // An observer's pulse leaves through its back face only — and it
            // powers that block *strongly*, which a conductor then re-emits.
            if descriptor.name == "minecraft:observer" {
                if let Some(facing) = descriptor.facing() {
                    rules.emit_only.insert(*id, facing.opposite());
                    rules.strong_into.insert(*id, facing.opposite());
                }
            }
            // A diode emits in exactly one direction too. `DiodeBlock.getSignal`
            // answers only when the querying step equals `FACING`, which puts
            // the output on the *opposite* side (the input comes from
            // `pos.relative(FACING)`). Without this a powered repeater or
            // comparator lit dust on all six sides — invisible in every small
            // golden, and the reason the community doors' settle came out
            // fully powered.
            if matches!(
                descriptor.name.as_str(),
                "minecraft:repeater" | "minecraft:comparator"
            ) {
                if let Some(facing) = descriptor.facing() {
                    rules.emit_only.insert(*id, facing.opposite());
                    rules.strong_into.insert(*id, facing.opposite());
                }
                if descriptor.name == "minecraft:comparator" {
                    rules.comparators.push(*id);
                }
            }
            // Plates strongly power their floor. A detector rail does the same:
            // `DetectorRailBlock.getDirectSignal` answers 15 only for
            // `Direction.UP`, i.e. only to the block beneath it. Captured in
            // `detector_strong.json`, where dust that touches the block under
            // the rail — and nothing else — reads 15. Neither block carries a
            // `face`, so Down is their only attachment.
            if descriptor.name.ends_with("_pressure_plate")
                || descriptor.name == "minecraft:detector_rail"
            {
                rules.strong_into.insert(*id, Dir::Down);
            }
            // A button strongly powers whatever it hangs on, in every one of
            // its orientations — `ButtonBlock.getDirectSignal` answers 15 for
            // `getConnectedDirection(state)`, the same `FaceAttachedHorizontal
            // DirectionalBlock` helper the lever uses. Captured across all six
            // in `button_strong.snbt`: a pressed button lights exactly one
            // lamp, the one beyond its support, and a glass support (not a
            // redstone conductor) relays nothing.
            //
            // This used to read `face == "floor"` only, so a wall or ceiling
            // button was not a strong source at all and no conductor ever
            // relayed it — `button_wall.json` / `button_ceiling.json`.
            if descriptor.name.ends_with("_button") {
                if let Some(attached) = lever_attachment(descriptor) {
                    rules.strong_into.insert(*id, attached);
                }
            }
            // A weighted plate emits its `power`, not 15.
            if let Some(power) = weighted_plate_power(descriptor) {
                rules.analog_emission.insert(*id, power);
            }
            // A lit torch powers every face but the one below it, and
            // strongly powers the block **above** — which is how a torch
            // under a solid block feeds dust sitting on that block, the
            // signal path a whole community door was missing.
            if matches!(
                descriptor.name.as_str(),
                "minecraft:redstone_torch" | "minecraft:redstone_wall_torch"
            ) {
                // A torch powers everything around it *except the block it is
                // attached to* — `RedstoneTorchBlock.getSignal` excludes `UP`
                // for a standing torch, and `RedstoneWallTorchBlock`'s excludes
                // `FACING`, which is the wall behind it. Reading both as "not
                // downward" gave a wall torch the wrong exclusion twice over:
                // it powered its own support, and refused to power the block
                // below it.
                //
                // Powering its own support is self-defeating in the exact
                // sense — the torch then reads its support as lit and books a
                // tick to turn itself off, one vanilla never books.
                let support = if descriptor.name == "minecraft:redstone_wall_torch" {
                    descriptor.facing().map(Dir::opposite).unwrap_or(Dir::Down)
                } else {
                    Dir::Down
                };
                rules.emit_except.insert(*id, support);
                rules.strong_into.insert(*id, Dir::Up);
            }
            // A lever strongly powers the block it hangs on.
            if descriptor.name == "minecraft:lever" {
                if let Some(attached) = lever_attachment(descriptor) {
                    rules.strong_into.insert(*id, attached);
                }
            }
        }
        if matches!(
            descriptor.name.as_str(),
            "minecraft:repeater" | "minecraft:comparator"
        ) {
            if let Some(facing) = descriptor.facing() {
                rules.diodes.insert(*id, facing);
            }
        }
    }

    // Wire sibling map: every wire state's 16 power variants (same shape).
    {
        let wire_ids: Vec<StateId> = rules.wires.keys().copied().collect();
        for id in wire_ids {
            let Some(text) = registry.descriptor(id).map(str::to_string) else { continue };
            let descriptor = Descriptor::parse(&text);
            for power in 0u8..16 {
                if let Some(sibling) = registry.get(&descriptor.with("power", &power.to_string()))
                {
                    rules.wire_siblings.insert((id, power), sibling);
                }
            }
        }
    }

    // Second pass: register behaviour, resolving paired states through the
    // registry so a block can find its own opposite.
    for (id, descriptor) in &descriptors {
        let name = descriptor.name.as_str();

        if INERT.contains(&name) || is_shulker_box(name) {
            table.register(*id, Box::new(Inert::new("vanilla")));
            continue;
        }

        match name {
            "minecraft:repeater" => {
                let (Some(facing), Some(delay)) =
                    (descriptor.facing(), descriptor.get("delay").and_then(|d| d.parse().ok()))
                else {
                    continue;
                };
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Repeater {
                        facing,
                        delay,
                        powered: descriptor.flag("powered"),
                        states,
                        locked: descriptor.flag("locked"),
                        locked_twin: registry.get(&descriptor.with(
                            "locked",
                            if descriptor.flag("locked") { "false" } else { "true" },
                        )),
                        power: rules.clone(),
                    }),
                );
            }
            "minecraft:comparator" => {
                let Some(facing) = descriptor.facing() else { continue };
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Comparator {
                        facing,
                        powered: descriptor.flag("powered"),
                        mode: match descriptor.get("mode") {
                            Some("subtract") => ComparatorMode::Subtract,
                            _ => ComparatorMode::Compare,
                        },
                        states,
                        power: rules.clone(),
                    }),
                );
            }
            "minecraft:observer" => {
                let Some(facing) = descriptor.facing() else { continue };
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Observer { facing, powered: descriptor.flag("powered"), states }),
                );
            }
            n if n.ends_with("_button") => {
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                let Some(attached) = lever_attachment(descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Button {
                        powered: descriptor.flag("powered"),
                        states,
                        // BlockSetType: the stone family presses for 20 game
                        // ticks, every wooden one for 30.
                        duration: if is_stone_button(n) { 20 } else { 30 },
                        attached,
                        power: rules.clone(),
                    }),
                );
            }
            "minecraft:redstone_lamp" => {
                let Some(states) = lit_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Lamp {
                        lit: descriptor.flag("lit"),
                        states,
                        power: rules.clone(),
                    }),
                );
            }
            n if n.contains("copper_bulb") => {
                let Some(states) = bulb_states(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(CopperBulb {
                        lit: descriptor.flag("lit"),
                        powered: descriptor.flag("powered"),
                        states,
                        power: rules.clone(),
                    }),
                );
            }
            n if n.ends_with("_door") => {
                let Some(states) = trapdoor_pair(registry, descriptor) else { continue };
                // `half` says which way the other half lies. Anything else is
                // not a door state we can reason about, so leave it unregistered.
                let (other_half, partner_half) = match descriptor.get("half") {
                    Some("lower") => (Dir::Up, "upper"),
                    Some("upper") => (Dir::Down, "lower"),
                    _ => continue,
                };
                // Every state of *this* door block in the opposite half. Keyed
                // on the block name, so an oak door stacked on an iron one is
                // two broken halves rather than one tall door.
                let partner: std::sync::Arc<[StateId]> = descriptors
                    .iter()
                    .filter(|(_, d)| d.name == descriptor.name && d.get("half") == Some(partner_half))
                    .map(|(sid, _)| *sid)
                    .collect();
                table.register(
                    *id,
                    Box::new(Door {
                        powered: descriptor.flag("powered"),
                        other_half,
                        states,
                        power: rules.clone(),
                        partner,
                    }),
                );
            }
            // A fence gate's power response is a trapdoor's exactly: `open` and
            // `powered` written together, quietly, off the plain neighbour signal.
            n if n.ends_with("_fence_gate") => {
                let Some(states) = trapdoor_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Trapdoor {
                        powered: descriptor.flag("powered"),
                        states,
                        power: rules.clone(),
                    }),
                );
            }
            n if n.ends_with("_trapdoor") => {
                let Some(states) = trapdoor_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Trapdoor {
                        powered: descriptor.flag("powered"),
                        states,
                        power: rules.clone(),
                    }),
                );
            }
            // Weighted plates carry `power` rather than `powered`, so they must
            // be turned away *before* this arm — matching `_pressure_plate` and
            // then bailing out of `powered_pair` would consume them here and
            // starve the arm further down that actually handles them.
            n if n.ends_with("_pressure_plate") && !n.ends_with("_weighted_pressure_plate") => {
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(PressurePlate {
                        powered: descriptor.flag("powered"),
                        states,
                        // Only the stone family ignores items; every wooden
                        // plate is triggered by anything, dropped items too.
                        senses_items: !is_stone_button(n),
                        power: rules.clone(),
                    }),
                );
            }
            "minecraft:redstone_wire" => {
                let power = descriptor
                    .get("power")
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(0);
                table.register(
                    *id,
                    Box::new(crate::wire::Wire { power_level: power, rules: rules.clone() }),
                );
            }
            "minecraft:note_block" => {
                let Some(note) = descriptor.get("note").and_then(|n| n.parse::<u8>().ok())
                else {
                    continue;
                };
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                // The click target: same powered flag, next pitch, wrapping at 24.
                let next = (note + 1) % crate::components::NOTE_VALUES;
                let Some(cycled) = registry.get(&descriptor.with("note", &next.to_string()))
                else {
                    continue;
                };
                let instrument_states: Vec<(&'static str, StateId)> = ["harp", "basedrum"]
                    .into_iter()
                    .filter_map(|name| {
                        registry
                            .get(&descriptor.with("instrument", name))
                            .map(|state| (name, state))
                    })
                    .collect();
                let instrument = match descriptor.get("instrument") {
                    Some("basedrum") => "basedrum",
                    _ => "harp",
                };
                table.register(
                    *id,
                    Box::new(NoteBlock {
                        powered: descriptor.flag("powered"),
                        states,
                        cycled,
                        instrument,
                        instrument_states,
                        power: rules.clone(),
                    }),
                );
            }
            "minecraft:redstone_torch" => {
                let Some(states) = lit_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Torch {
                        attached: Dir::Down,
                        lit: descriptor.flag("lit"),
                        states,
                        power: rules.clone(),
                    }),
                );
            }
            "minecraft:redstone_wall_torch" => {
                // A wall torch hangs off the block behind it, which is the opposite
                // of the way it faces.
                let Some(facing) = descriptor.facing() else { continue };
                let Some(states) = lit_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Torch {
                        attached: facing.opposite(),
                        lit: descriptor.flag("lit"),
                        states,
                        power: rules.clone(),
                    }),
                );
            }
            n if n.ends_with("_leaves") => {
                let distance = descriptor
                    .get("distance")
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(7u8);
                let mut family = [None; 8];
                // Interned, not looked up: a schematic holds only the
                // distances it was saved with, and the tick needs the whole
                // family to write into.
                for want in 1u8..=7 {
                    family[want as usize] = registry
                        .intern(&descriptor.with("distance", &want.to_string()))
                        .ok();
                }
                table.register(
                    *id,
                    Box::new(crate::components::Leaves {
                        distance,
                        family,
                        rules: rules.clone(),
                    }),
                );
            }
            "minecraft:piston_head" => {
                let Some(facing) = descriptor.facing() else { continue };
                // `canSurvive` is `isFittingBase(base) || (base is MOVING_PISTON
                // with the same FACING)`. The second half is the whole point: a
                // base mid-retract *is* a moving_piston, and that is exactly
                // when the head still has to forward to it. Allowing only the
                // extended piston made the head go silent for the one dispatch
                // that decides whether the retract is heard.
                let bases: Vec<StateId> = descriptors
                    .iter()
                    .filter(|(_, d)| d.facing() == Some(facing))
                    .filter(|(_, d)| match d.name.as_str() {
                        "minecraft:piston" | "minecraft:sticky_piston" => d.flag("extended"),
                        "minecraft:moving_piston" => true,
                        _ => false,
                    })
                    .map(|(id, _)| *id)
                    .collect();
                table.register(*id, Box::new(crate::piston::PistonHead { bases, facing }));
            }
            "minecraft:piston" | "minecraft:sticky_piston" => {
                let Some(facing) = descriptor.facing() else { continue };
                let extended = descriptor.flag("extended");
                let Some(states) = extended_pair(registry, descriptor) else { continue };
                let head = registry
                    .get(&format!(
                        "minecraft:piston_head[facing={},short=false,type={}]",
                        face_name(facing),
                        if name == "minecraft:sticky_piston" { "sticky" } else { "normal" }
                    ))
                    .unwrap_or(StateId::AIR);
                let moving = registry
                    .get(&format!(
                        "minecraft:moving_piston[facing={},type={}]",
                        face_name(facing),
                        if name == "minecraft:sticky_piston" { "sticky" } else { "normal" }
                    ))
                    .unwrap_or(StateId::AIR);
                // Pushed and pulled blocks always ride a type=normal placeholder,
                // whatever the piston's own type; see Piston::moving_block.
                let moving_block = registry
                    .get(&format!(
                        "minecraft:moving_piston[facing={},type=normal]",
                        face_name(facing)
                    ))
                    .unwrap_or(StateId::AIR);
                table.register(
                    *id,
                    Box::new(Piston {
                        facing,
                        extended,
                        sticky: name == "minecraft:sticky_piston",
                        states,
                        head,
                        moving,
                        moving_block,
                        power: rules.clone(),
                        movability: rules.clone(),
                    }),
                );
            }
            "minecraft:hopper" => {
                let Some(facing) = descriptor.facing() else { continue };
                let Some(states) = enabled_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Hopper {
                        facing,
                        enabled: descriptor.get("enabled") != Some("false"),
                        states,
                        power: rules.clone(),
                    }),
                );
            }
            "minecraft:dropper" | "minecraft:dispenser" => {
                let Some(facing) = descriptor.facing() else { continue };
                let Some(states) = triggered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Dropper {
                        facing,
                        triggered: descriptor.flag("triggered"),
                        states,
                        dispenser: name == "minecraft:dispenser",
                        power: rules.clone(),
                    }),
                );
            }
            "minecraft:lever" => {
                let Some(attached) = lever_attachment(descriptor) else { continue };
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(crate::components::Lever {
                        powered: descriptor.flag("powered"),
                        states,
                        attached,
                    }),
                );
            }
            "minecraft:rail" => {
                // Cart physics reads rails through the rail tables; a plain
                // rail has no redstone behaviour of its own.
                table.register(*id, Box::new(Inert::new("rail")));
            }
            "minecraft:detector_rail" => {
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(crate::components::DetectorRail {
                        powered: descriptor.flag("powered"),
                        states,
                    }),
                );
            }
            "minecraft:powered_rail" | "minecraft:activator_rail" => {
                let Some(shape) = descriptor
                    .get("shape")
                    .and_then(crate::minecart::RailShape::from_name)
                else {
                    continue;
                };
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(crate::minecart::PoweredRail {
                        block: if descriptor.name == "minecraft:activator_rail" {
                            "minecraft:activator_rail"
                        } else {
                            "minecraft:powered_rail"
                        },
                        shape,
                        powered: descriptor.flag("powered"),
                        states,
                        power: rules.clone(),
                    }),
                );
            }
            "minecraft:water" => {
                let level = descriptor.get("level").and_then(|l| l.parse().ok()).unwrap_or(0);
                table.register(
                    *id,
                    Box::new(crate::fluid::Water {
                        kind: crate::fluid::WaterKind::from_level(level),
                        rules: rules.clone(),
                    }),
                );
            }
            // Bubble columns are inert as *blocks* here; their column-integrity
            // ticks are not modelled. Their water and their entity effects live
            // in the rules and the item physics.
            "minecraft:bubble_column" => {
                table.register(*id, Box::new(Inert::new("bubble_column")));
            }
            // Building material, checked only once every implemented block has
            // had its say. Order matters: `minecraft:piston_head` ends in
            // `_head` and a decoration family that ran first would claim it,
            // replacing a piston's arm with a brick and quietly breaking every
            // door in the corpus.
            // The blocks below have real vanilla behaviour, but every input
            // that could drive it is a player or a mob — and a headless door
            // simulation has neither. Each is registered only in the state
            // that is a *fixed point* under that absence; any other state is
            // left unregistered so the build fails loudly rather than quietly
            // simulating something the engine cannot actually reproduce.
            //
            // This covers the cauldron, the campfire, the lectern and unpressed
            // tripwire. It used to cover the weighted pressure plates too, on
            // the reasoning that nothing could stand on one. The record 3x3
            // door disproved that: its plates are pressed by *entities the
            // pistons move* — frozen fireballs — with no player anywhere. They
            // are now fully simulated, below.

            // A cauldron only fills or empties by hand or by weather. Its
            // `level` is read by a comparator and otherwise never moves.
            n if n.ends_with("cauldron") => {
                table.register(*id, Box::new(Inert::new("cauldron")));
            }
            // A campfire lights, smokes and cooks; none of that is redstone,
            // and nothing in a door can extinguish it.
            "minecraft:campfire" | "minecraft:soul_campfire" => {
                table.register(*id, Box::new(Inert::new("campfire")));
            }
            // A lectern pulses and feeds a comparator when a *player* turns a
            // page. With no book there is no page and no signal, ever.
            "minecraft:lectern" if !descriptor.flag("has_book") => {
                table.register(*id, Box::new(Inert::new("lectern")));
            }
            // Tripwire is pressed by entities intersecting it. An unpressed
            // string in an entity-free world stays unpressed.
            "minecraft:tripwire" if !descriptor.flag("powered") => {
                table.register(*id, Box::new(Inert::new("tripwire")));
            }
            // A weighted plate's `power` is the number of entities standing on
            // it — every entity type, items included
            // (`getEntitiesOfClass(Entity.class, ...)`). `maxWeight` is 15 for
            // the light plate and 150 for the heavy one, which is the whole
            // difference between them: light reads one per entity, heavy one
            // per ten. Captured in `weighted_plates.json`.
            "minecraft:light_weighted_pressure_plate"
            | "minecraft:heavy_weighted_pressure_plate" => {
                let Some(power) = weighted_plate_power(descriptor) else { continue };
                let mut states = Vec::with_capacity(16);
                for level in 0u8..16 {
                    let Some(state) = registry.get(&descriptor.with("power", &level.to_string()))
                    else {
                        break;
                    };
                    states.push(state);
                }
                if states.len() != 16 {
                    continue;
                }
                table.register(
                    *id,
                    Box::new(crate::components::WeightedPlate {
                        power,
                        max_weight: if descriptor.name == "minecraft:heavy_weighted_pressure_plate"
                        {
                            150
                        } else {
                            15
                        },
                        states,
                    }),
                );
            }
            n if decor_kind(n).is_some() => {
                table.register(*id, Box::new(Inert::new("material")));
            }
            // Anything else stays unregistered, and will be named in the report.
            _ => {}
        }
    }

    rules
}

/// How many inventory slots a container block has.
///
/// Sizes are the block entities' declared container sizes (`BarrelBlockEntity`
/// and `ChestBlockEntity` are 27, `HopperBlockEntity` 5, `DispenserBlockEntity`
/// 9). Hoppers, droppers and dispensers appear here so their inventories load,
/// even though their behaviours are not implemented yet — they stay
/// unregistered and loud until they are.
pub fn container_slots(name: &str) -> Option<u32> {
    match name {
        "minecraft:barrel" | "minecraft:chest" | "minecraft:trapped_chest" => Some(27),
        n if is_shulker_box(n) => Some(27),
        "minecraft:hopper" => Some(5),
        "minecraft:dropper" | "minecraft:dispenser" => Some(9),
        // A furnace is comparator-readable, so it needs its slot count even
        // though its smelting is not modelled — registering it inert without
        // this would silently read every furnace as empty.
        "minecraft:furnace" | "minecraft:blast_furnace" | "minecraft:smoker" => Some(3),
        _ => None,
    }
}

/// Whether a block state conducts redstone (`isRedstoneConductor`).
///
/// `BlockBehaviour.Properties` defaults the predicate to
/// `state.blocksMotion() && state.isCollisionShapeFullBlock(...)` — so a solid
/// block conducts unless its registration overrides it. The exclusions below
/// are exactly the blocks whose registration calls `isRedstoneConductor`,
/// read off `Blocks`' static initialiser in the 26.2 server jar. Slime and
/// honey call `noOcclusion` and no more: occlusion is a lighting concern, and
/// slime's collision box is a full cube, so slime conducts (which the flying
/// machine capture independently shows) while honey's inset box does not.
///
/// Two blocks used to be listed here on the assumption that emitting a signal
/// implies not conducting one. It does not — `TARGET` and `REDSTONE_LAMP`
/// register with no override at all, and the vault door turns a torch off by
/// strongly powering the target block the torch stands on.
fn is_conductor(descriptor: &Descriptor) -> bool {
    if !is_full_cube(descriptor) {
        return false;
    }
    !matches!(
        descriptor.name.as_str(),
        "minecraft:glass"
            | "minecraft:white_stained_glass"
            | "minecraft:sea_lantern"
            | "minecraft:redstone_block"
            | "minecraft:observer"
            // Honey's collision shape is inset, so it fails the default
            // predicate rather than overriding it.
            | "minecraft:honey_block"
            // `leavesProperties` and `pistonProperties` both call
            // `isRedstoneConductor`. A piston base is a full *collision* cube
            // when retracted, but declares itself no conductor — which is why
            // an observer's strong power once leaked through one into a door's
            // dust line.
            | "minecraft:oak_leaves"
            | "minecraft:piston"
            | "minecraft:sticky_piston"
    )
    // The rest of the same families: every stained glass, tinted glass, the
    // ices and every leaf. Generalising the entries above rather than adding
    // new judgement — `glass`, `white_stained_glass`, `sea_lantern` and
    // `oak_leaves` were each read off a capture, and their siblings share the
    // registration that produced them.
    && decor_kind(&descriptor.name) != Some(Decor::Glassy)
}

/// Whether a block state is a full collision cube.
///
/// Drives hopper-suction blocking and item-entity collision. Everything
/// registered defaults to a full cube except the shapes that plainly are not;
/// chests and hoppers are close-but-not-full and count as not-full, which
/// matches `isCollisionShapeFullBlock` for suction. An extended piston base is
/// not a full cube; a retracted one is.
fn is_full_cube(descriptor: &Descriptor) -> bool {
    match descriptor.name.as_str() {
        "minecraft:air" | "minecraft:water" | "minecraft:bubble_column" => false,
        // Soul sand's collision column tops at 14/16: not a full cube, so it
        // neither conducts nor blocks hopper suction (isCollisionShapeFullBlock).
        "minecraft:soul_sand" | "minecraft:cobweb" => false,
        "minecraft:rail"
        | "minecraft:powered_rail"
        | "minecraft:detector_rail"
        | "minecraft:activator_rail" => false,
        "minecraft:birch_wall_sign"
        | "minecraft:player_wall_head"
        | "minecraft:lightning_rod"
        | "minecraft:tripwire_hook"
        | "minecraft:composter" => false,
        // A slab is a full cube only when doubled.
        n if n.ends_with("_slab") => descriptor.get("type") == Some("double"),
        // A trapdoor is a 3/16 plate whatever its pose: never a full cube, so
        // it neither conducts nor blocks hopper suction. A door is the same
        // 3/16 slice stood upright, and getting this wrong would route redstone
        // through the door leaf as if it were a solid block.
        n if n.ends_with("_trapdoor") || n.ends_with("_door") => false,
        // Hollow, low or flat: a cauldron's basin, a campfire's logs, a
        // lectern's desk, a plate and a string. None is a full collision cube,
        // so none conducts and none blocks a hopper.
        n if n.ends_with("cauldron") => false,
        "minecraft:campfire"
        | "minecraft:soul_campfire"
        | "minecraft:lectern"
        | "minecraft:tripwire" => false,
        n if n.ends_with("_pressure_plate") || n.ends_with("_button") => false,
        "minecraft:redstone_wire"
        | "minecraft:redstone_torch"
        | "minecraft:redstone_wall_torch"
        | "minecraft:repeater"
        | "minecraft:comparator"
        | "minecraft:lever"
        | "minecraft:chest"
        | "minecraft:trapped_chest"
        | "minecraft:hopper"
        | "minecraft:piston_head"
        | "minecraft:moving_piston" => false,
        "minecraft:piston" | "minecraft:sticky_piston" => !descriptor.flag("extended"),
        // Building material whose shape is not a full cell — walls, fences,
        // panes, stairs, carpets. They must not conduct and must not block a
        // hopper, which both follow from answering false here.
        n if decor_kind(n) == Some(Decor::Partial) => false,
        _ => true,
    }
}

/// Collision and friction tables for item physics, indexed by `StateId`.
///
/// Friction is `Block.getFriction` from the `Blocks` static initialiser: 0.6
/// default, slime 0.8, ice/packed/frosted 0.98, blue ice 0.989. The third
/// table is each solid state's collision-box height (soul sand tops at 14/16;
/// everything else solid is a full cube here). The fourth marks cobwebs,
/// whose `entityInside` sets the stuck-speed multiplier.
pub fn physics_tables(registry: &StateRegistry) -> (Vec<bool>, Vec<f32>, Vec<f32>, Vec<bool>) {
    let mut solidity = Vec::with_capacity(registry.len());
    let mut frictions = Vec::with_capacity(registry.len());
    let mut heights = Vec::with_capacity(registry.len());
    let mut webs = Vec::with_capacity(registry.len());
    for index in 0..registry.len() {
        let descriptor = registry
            .descriptor(StateId(index as u16))
            .map(Descriptor::parse);
        let (solid, friction, height, web) = match &descriptor {
            None => (false, 0.6, 1.0, false),
            Some(d) => {
                let friction = match d.name.as_str() {
                    "minecraft:slime_block" => 0.8,
                    "minecraft:ice" | "minecraft:packed_ice" | "minecraft:frosted_ice" => 0.98,
                    "minecraft:blue_ice" => 0.989,
                    _ => 0.6,
                };
                match d.name.as_str() {
                    "minecraft:soul_sand" => (true, friction, 0.875, false),
                    "minecraft:cobweb" => (false, friction, 1.0, true),
                    _ => (is_full_cube(d), friction, 1.0, false),
                }
            }
        };
        solidity.push(solid);
        frictions.push(friction);
        heights.push(height);
        webs.push(web);
    }
    (solidity, frictions, heights, webs)
}

/// Fluid tables for item physics, indexed by `StateId`: the water in each
/// state and whether it is a bubble column (`Some(drag_down)`).
pub fn fluid_tables(
    registry: &StateRegistry,
) -> (Vec<Option<crate::fluid::WaterKind>>, Vec<Option<bool>>) {
    let mut water_kinds = Vec::with_capacity(registry.len());
    let mut bubble_kinds = Vec::with_capacity(registry.len());
    for index in 0..registry.len() {
        let descriptor = registry
            .descriptor(StateId(index as u16))
            .map(Descriptor::parse);
        let (water, bubble) = match &descriptor {
            None => (None, None),
            Some(d) => match d.name.as_str() {
                "minecraft:water" => {
                    let level = d.get("level").and_then(|l| l.parse().ok()).unwrap_or(0);
                    (Some(crate::fluid::WaterKind::from_level(level)), None)
                }
                "minecraft:bubble_column" => (
                    Some(crate::fluid::WaterKind::Source),
                    Some(d.get("drag") == Some("true")),
                ),
                _ if d.get("waterlogged") == Some("true") => {
                    (Some(crate::fluid::WaterKind::Source), None)
                }
                _ => (None, None),
            },
        };
        water_kinds.push(water);
        bubble_kinds.push(bubble);
    }
    (water_kinds, bubble_kinds)
}

/// Whether a block is a button or a pressure plate of any material. Both are
/// signal sources whatever their state, and both are `PushReaction.DESTROY`.
fn is_button_or_plate(name: &str) -> bool {
    name.ends_with("_button") || name.ends_with("_pressure_plate")
}

/// A weighted pressure plate's `power`, or `None` for any other block.
fn weighted_plate_power(descriptor: &Descriptor) -> Option<u8> {
    if !descriptor.name.ends_with("_weighted_pressure_plate") {
        return None;
    }
    descriptor.get("power").and_then(|p| p.parse().ok())
}

/// Whether a button or pressure plate belongs to the *stone* `BlockSetType`
/// rather than a wooden one. Stone presses for 20 game ticks and its plate
/// ignores dropped items; wood presses for 30 and senses anything.
fn is_stone_button(name: &str) -> bool {
    matches!(
        name,
        "minecraft:stone_button"
            | "minecraft:polished_blackstone_button"
            | "minecraft:stone_pressure_plate"
            | "minecraft:polished_blackstone_pressure_plate"
    )
}

/// The direction from a lever or button to its support block —
/// `FaceAttachedHorizontalDirectionalBlock.getConnectedDirection` inverted.
/// Both blocks strongly power exactly this neighbour, captured for the lever in
/// `lever_lamp.json` and for the button in all six orientations by
/// `button_strong.snbt` (`button_wall.json`, `button_ceiling.json`).
fn lever_attachment(descriptor: &Descriptor) -> Option<Dir> {
    match descriptor.get("face") {
        Some("floor") => Some(Dir::Down),
        Some("ceiling") => Some(Dir::Up),
        _ => descriptor.facing().map(Dir::opposite),
    }
}

/// Whether a block descriptor is a full collision cube — `isCollisionShapeFullBlock`
/// against an empty world, which is what `StructureTemplate.addToLists` uses to
/// decide a placed block's update-pass group.
pub fn is_collision_full_cube(descriptor: &str) -> bool {
    is_full_cube(&Descriptor::parse(descriptor))
}

/// Whether a block's shape depends on its surroundings (`hasDynamicShape`).
///
/// Only the shulker box in this corpus, and it carries a block entity anyway,
/// so the distinction never changes a group.
pub fn has_dynamic_shape(descriptor: &str) -> bool {
    let name = descriptor.split('[').next().unwrap_or(descriptor);
    is_shulker_box(name)
}

/// Whether a block is a shulker box — any of the sixteen dye colours or the
/// undyed `minecraft:shulker_box`.
fn is_shulker_box(name: &str) -> bool {
    name.ends_with("_shulker_box") || name == "minecraft:shulker_box"
}

/// The shape of an inert building material, for the two questions the engine
/// asks about a block that does nothing: does it fill its cell, and does it
/// carry redstone?
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Decor {
    /// An ordinary opaque cube: fills the cell, conducts.
    Cube,
    /// Fills the cell but its registration calls `isRedstoneConductor(never)`
    /// — glass and its kin. Conducts nothing.
    Glassy,
    /// Not a full cell at all: walls, fences, panes, stairs, carpets. Neither
    /// conducts nor blocks a hopper.
    Partial,
}

/// Classify a block as inert building material, or `None` if the engine has no
/// business guessing.
///
/// A door is mostly decoration, and refusing a whole build because one cell
/// holds yellow wool is useless — 20 of 28 real community doors were rejected
/// that way, several for a single colour. But the gate exists for a reason: a
/// block wrongly called a conductor silently changes where redstone goes, and
/// a wrong answer is worse than a refusal. So this names families whose vanilla
/// answer is not in doubt and leaves everything else unregistered.
///
/// Deliberately absent, and still failing loudly:
/// - **copper bulbs** — they latch on a pulse and a comparator reads them.
///   They are redstone components wearing a building block's shape.
/// - **doors, lecterns, cauldrons, campfires** — each has behaviour worth
///   modelling properly rather than asserting away.
///
/// Known simplification: gravity is not simulated, so `sand` and `gravel` are
/// inert here. A door whose pattern is sand (§5.5) holds its shape while
/// vanilla would drop an unsupported grain.
pub(crate) fn decor_kind(name: &str) -> Option<Decor> {
    let short = name.strip_prefix("minecraft:")?;

    // Blocks the engine implements, or means to, are never decoration however
    // their name reads. `piston_head` ends in `_head` and would otherwise be
    // claimed by the head-and-skull family — replacing a piston's arm with a
    // brick. The registration match already puts material last, but this is
    // also called straight from `is_full_cube` and `is_conductor`, where no
    // ordering protects anything.
    if matches!(
        short,
        "piston_head"
            | "moving_piston"
            | "piston"
            | "sticky_piston"
            | "observer"
            | "redstone_lamp"
            | "note_block"
            | "target"
            | "lectern"
    ) || short.ends_with("_door")
        || short.ends_with("_trapdoor")
        || short.contains("copper_bulb")
    {
        return None;
    }

    // Shapes that plainly do not fill their cell. Checked first: a
    // `stone_brick_wall` must never be mistaken for the stone family below.
    const PARTIAL_SUFFIX: &[&str] = &[
        "_stairs",
        "_wall",
        "_fence",
        "_fence_gate",
        "_pane",
        "_carpet",
        "_sign",
        "_head",
        "_skull",
        "_banner",
        "_candle",
        "_lantern",
        "_bars",
        "_ladder",
    ];
    if PARTIAL_SUFFIX.iter().any(|s| short.ends_with(s))
        || matches!(short, "chain" | "scaffolding" | "iron_bars" | "ladder" | "snow")
    {
        // A wall sign is a sign; a player head is a head. Both land here.
        return Some(Decor::Partial);
    }

    // Glass and its kin: full cells whose registration refuses to conduct.
    // The engine already carried `glass`, `white_stained_glass` and
    // `sea_lantern` from capture-verified builds; these are the rest of the
    // same families, and `*_leaves` generalises the verified `oak_leaves`.
    if short.ends_with("_stained_glass")
        || short.ends_with("_leaves")
        || matches!(
            short,
            "glass" | "tinted_glass" | "ice" | "packed_ice" | "blue_ice" | "sea_lantern"
        )
    {
        return Some(Decor::Glassy);
    }

    // Ordinary opaque cubes. Dye-colour and wood-species families are matched
    // by suffix so a new colour never has to be added by hand again — that is
    // the mistake this whole function exists to stop repeating.
    const CUBE_SUFFIX: &[&str] = &[
        "_wool",
        "_concrete",
        "_concrete_powder",
        "_terracotta", // covers `*_glazed_terracotta`
        "_planks",
        "_log",
        "_wood",
        "_bricks",
        "_ore",
        "_nylium",
        "_wart_block",
    ];
    if CUBE_SUFFIX.iter().any(|s| short.ends_with(s)) || short.starts_with("stripped_") {
        return Some(Decor::Cube);
    }
    if matches!(
        short,
        // Stone family.
        "stone" | "granite" | "diorite" | "andesite" | "calcite" | "tuff" | "basalt"
            | "smooth_basalt" | "deepslate" | "cobbled_deepslate" | "polished_deepslate"
            | "blackstone" | "polished_blackstone" | "netherrack" | "end_stone"
            | "dripstone_block" | "obsidian" | "crying_obsidian" | "bricks" | "prismarine"
            | "purpur_block" | "clay" | "mud" | "snow_block"
            // Quartz family.
            | "quartz_block" | "quartz_bricks" | "smooth_quartz" | "chiseled_quartz_block"
            | "quartz_pillar"
            // Mineral blocks.
            | "iron_block" | "gold_block" | "diamond_block" | "emerald_block" | "lapis_block"
            | "coal_block" | "netherite_block" | "copper_block" | "raw_iron_block"
            | "raw_gold_block" | "raw_copper_block"
            // Miscellany that turns up in builds.
            | "hay_block" | "bone_block" | "sponge" | "wet_sponge" | "dried_kelp_block"
            | "moss_block" | "glowstone" | "tnt" | "sand" | "red_sand" | "gravel"
            | "shroomlight" | "sculk" | "amethyst_block" | "budding_amethyst"
    ) {
        return Some(Decor::Cube);
    }
    None
}

/// The instrument a block gives a note block sitting on it.
///
/// Vanilla reads this from each block's `BlockBehaviour.Properties.instrument`,
/// a per-block data table. The entries here are **derived from placement
/// captures** (`--dump-placed`): a note block whose instrument vanilla rewrote
/// at placement told us what the block below it provides. Anything unlisted
/// answers `harp`, which is vanilla's own default, so a missing entry surfaces
/// as a loud trace divergence rather than a quiet wrong note.
///
/// Only instruments that do **not** work above a note block are modelled, so
/// the block above never wins — true for everything in the corpus (the
/// exceptions are mob heads).
pub fn instrument_below(name: &str) -> &'static str {
    let name = name.split('[').next().unwrap_or(name);
    match name {
        "minecraft:observer" => "basedrum",
        n if n.ends_with("_concrete") => "basedrum",
        "minecraft:stone"
        | "minecraft:smooth_stone"
        | "minecraft:cobblestone"
        | "minecraft:obsidian"
        | "minecraft:quartz_block"
        | "minecraft:chiseled_quartz_block" => "basedrum",
        _ => "harp",
    }
}

/// Rail and conductor tables for cart physics, indexed by `StateId`.
pub fn rail_tables(
    registry: &StateRegistry,
) -> (Vec<Option<crate::minecart::Rail>>, Vec<bool>) {
    let mut rails = Vec::with_capacity(registry.len());
    let mut conductors = Vec::with_capacity(registry.len());
    for index in 0..registry.len() {
        let descriptor = registry
            .descriptor(StateId(index as u16))
            .map(Descriptor::parse);
        let (rail, conductor) = match &descriptor {
            None => (None, false),
            Some(d) => {
                let rail = match d.name.as_str() {
                    "minecraft:rail"
                    | "minecraft:powered_rail"
                    | "minecraft:detector_rail"
                    | "minecraft:activator_rail" => d
                        .get("shape")
                        .and_then(crate::minecart::RailShape::from_name)
                        .map(|shape| crate::minecart::Rail {
                            shape,
                            powered_rail: d.name == "minecraft:powered_rail",
                            powered: d.flag("powered"),
                        }),
                    _ => None,
                };
                (rail, is_conductor(d))
            }
        };
        rails.push(rail);
        conductors.push(conductor);
    }
    (rails, conductors)
}

/// The unpowered/powered pair for a descriptor, if both states are interned.
fn powered_pair(registry: &StateRegistry, descriptor: &Descriptor) -> Option<StatePair> {
    Some(StatePair {
        off: registry.get(&descriptor.with("powered", "false"))?,
        on: registry.get(&descriptor.with("powered", "true"))?,
    })
}

fn lit_pair(registry: &StateRegistry, descriptor: &Descriptor) -> Option<StatePair> {
    Some(StatePair {
        off: registry.get(&descriptor.with("lit", "false"))?,
        on: registry.get(&descriptor.with("lit", "true"))?,
    })
}

fn enabled_pair(registry: &StateRegistry, descriptor: &Descriptor) -> Option<StatePair> {
    Some(StatePair {
        off: registry.get(&descriptor.with("enabled", "false"))?,
        on: registry.get(&descriptor.with("enabled", "true"))?,
    })
}

/// The four `(lit, powered)` states a copper bulb cycles through, indexed
/// `lit * 2 + powered` to match [`CopperBulb::states`].
fn bulb_states(registry: &StateRegistry, descriptor: &Descriptor) -> Option<[StateId; 4]> {
    let mut states = [StateId(0); 4];
    for lit in [false, true] {
        let with_lit = Descriptor::parse(&descriptor.with("lit", if lit { "true" } else { "false" }));
        for powered in [false, true] {
            let name = with_lit.with("powered", if powered { "true" } else { "false" });
            states[usize::from(lit) * 2 + usize::from(powered)] = registry.get(&name)?;
        }
    }
    Some(states)
}

/// A trapdoor's power response sets `open` and `powered` together (see
/// [`Trapdoor`]): off is both-false, on is both-true.
fn trapdoor_pair(registry: &StateRegistry, descriptor: &Descriptor) -> Option<StatePair> {
    let both = |value: &str| {
        let opened = Descriptor::parse(&descriptor.with("open", value));
        registry.get(&opened.with("powered", value))
    };
    Some(StatePair { off: both("false")?, on: both("true")? })
}

fn triggered_pair(registry: &StateRegistry, descriptor: &Descriptor) -> Option<StatePair> {
    Some(StatePair {
        off: registry.get(&descriptor.with("triggered", "false"))?,
        on: registry.get(&descriptor.with("triggered", "true"))?,
    })
}

fn extended_pair(registry: &StateRegistry, descriptor: &Descriptor) -> Option<StatePair> {
    Some(StatePair {
        off: registry.get(&descriptor.with("extended", "false"))?,
        on: registry.get(&descriptor.with("extended", "true"))?,
    })
}

fn face_name(dir: Dir) -> &'static str {
    match dir {
        Dir::Down => "down",
        Dir::Up => "up",
        Dir::North => "north",
        Dir::South => "south",
        Dir::West => "west",
        Dir::East => "east",
    }
}

/// Intern every state a block needs, including the ones it transitions into.
///
/// A structure only contains the states it was saved with: an unextended piston
/// has no reason to mention `extended=true`, and a build never mentions
/// `moving_piston` at all. Without their counterparts a block cannot change state,
/// so a simulation would silently freeze.
/// The values a wire's four side properties take — `RedstoneSide`.
const WIRE_SIDE_VALUES: [&str; 3] = ["none", "side", "up"];

pub fn intern_companions(registry: &mut StateRegistry) {
    let existing: Vec<String> = (0..registry.len())
        .filter_map(|i| registry.descriptor(StateId(i as u16)).map(str::to_string))
        .collect();

    for text in existing {
        let descriptor = Descriptor::parse(&text);
        let companions: Vec<String> = match descriptor.name.as_str() {
            "minecraft:repeater" => {
                // Both powered *and* both locked variants: `locked` is derived
                // by updateShape, so either value may be needed at runtime.
                let mut all = Vec::new();
                for locked in ["false", "true"] {
                    let at = Descriptor::parse(&descriptor.with("locked", locked));
                    all.push(at.with("powered", "false"));
                    all.push(at.with("powered", "true"));
                }
                all
            }
            "minecraft:comparator" | "minecraft:observer" | "minecraft:lever" => {
                vec![descriptor.with("powered", "false"), descriptor.with("powered", "true")]
            }
            "minecraft:redstone_torch" | "minecraft:redstone_wall_torch" => {
                vec![descriptor.with("lit", "false"), descriptor.with("lit", "true")]
            }
            // Every state a wire can take: sixteen powers across all eighty-one
            // combinations of the four sides. Interning only the powers was
            // enough for as long as a wire's *connections* never changed, and
            // they do — `getConnectionState` reshapes a wire whenever a
            // neighbour appears or leaves. A shape the schematic never happened
            // to contain simply did not exist, and the rewrite was dropped: the
            // wire kept a connection to a block a piston had pulled away, which
            // then suppressed the symmetry rule that would have run it straight.
            // Silent, because a missing state is indistinguishable from no
            // change at the call site.
            "minecraft:redstone_wire" => {
                let mut all = Vec::new();
                for power in 0u8..16 {
                    let at = Descriptor::parse(&descriptor.with("power", &power.to_string()));
                    for north in WIRE_SIDE_VALUES {
                        let at = Descriptor::parse(&at.with("north", north));
                        for south in WIRE_SIDE_VALUES {
                            let at = Descriptor::parse(&at.with("south", south));
                            for east in WIRE_SIDE_VALUES {
                                let at = Descriptor::parse(&at.with("east", east));
                                for west in WIRE_SIDE_VALUES {
                                    all.push(at.with("west", west));
                                }
                            }
                        }
                    }
                }
                all
            }
            // Every distance a leaf can take. `register_all` interns these too,
            // for the family it writes into — but it does so while iterating a
            // snapshot of the registry, so a distance the schematic never held
            // would exist as a state with no behaviour attached. A leaf pulled
            // to distance 1 by a log then went inert: it could no longer book
            // its own re-check, and the observer watching it never fired again.
            n if n.ends_with("_leaves") => {
                (1u8..=7).map(|d| descriptor.with("distance", &d.to_string())).collect()
            }
            "minecraft:stone_button"
            | "minecraft:oak_button"
            | "minecraft:stone_pressure_plate"
            | "minecraft:oak_pressure_plate" => {
                vec![descriptor.with("powered", "false"), descriptor.with("powered", "true")]
            }
            "minecraft:redstone_lamp" => {
                vec![descriptor.with("lit", "false"), descriptor.with("lit", "true")]
            }
            // A bulb's `lit` survives losing power, so any of the four
            // `(lit, powered)` pairings is reachable and all four must exist
            // before a behaviour can bind to them.
            n if n.contains("copper_bulb") => {
                let mut variants = Vec::new();
                for lit in ["false", "true"] {
                    let with_lit = Descriptor::parse(&descriptor.with("lit", lit));
                    for powered in ["false", "true"] {
                        variants.push(with_lit.with("powered", powered));
                    }
                }
                variants
            }
            // Doors and fence gates write `open` and `powered` together, as a
            // trapdoor does, but a structure may hold any of the four.
            n if n.ends_with("_door") || n.ends_with("_fence_gate") => {
                let mut variants = Vec::new();
                for open in ["false", "true"] {
                    let opened = Descriptor::parse(&descriptor.with("open", open));
                    for powered in ["false", "true"] {
                        variants.push(opened.with("powered", powered));
                    }
                }
                variants
            }
            // A trapdoor's power write flips `open` and `powered` together,
            // but a structure may hold any of the four combinations.
            n if n.ends_with("_trapdoor") => {
                let mut variants = Vec::new();
                for open in ["false", "true"] {
                    let opened = Descriptor::parse(&descriptor.with("open", open));
                    for powered in ["false", "true"] {
                        variants.push(opened.with("powered", powered));
                    }
                }
                variants
            }
            "minecraft:hopper" => {
                vec![descriptor.with("enabled", "false"), descriptor.with("enabled", "true")]
            }
            "minecraft:dropper" | "minecraft:dispenser" => {
                vec![
                    descriptor.with("triggered", "false"),
                    descriptor.with("triggered", "true"),
                ]
            }
            "minecraft:note_block" => {
                // Every pitch, powered and not: a click cycles `note` and wraps at
                // 24, and a product run may click any number of times, so the
                // whole cycle is interned rather than one step of it.
                //
                // The instrument is read off the block *underneath* — wool gives
                // `guitar`, glass `hat`, wood `bass` — so harp and basedrum alone
                // cover only note blocks sitting on dirt or stone. Interning just
                // those two left every other note block without its powered
                // partner, `powered_pair` returned nothing, and the block was
                // dropped as unimplemented: three doors in the corpus were
                // rejected over an instrument we simply never interned. The
                // instrument each note block actually has is added to the pair,
                // so the palette's own instruments are always covered without
                // interning all twenty-three.
                let mut all = Vec::new();
                let own = descriptor.get("instrument").unwrap_or("harp").to_string();
                for instrument in ["harp", "basedrum", own.as_str()] {
                    let at_inst = Descriptor::parse(&descriptor.with("instrument", instrument));
                    for note in 0..crate::components::NOTE_VALUES {
                        let at_note = Descriptor::parse(&at_inst.with("note", &note.to_string()));
                        all.push(at_note.with("powered", "false"));
                        all.push(at_note.with("powered", "true"));
                    }
                }
                all
            }
            "minecraft:powered_rail" | "minecraft:activator_rail" | "minecraft:detector_rail" => {
                vec![descriptor.with("powered", "false"), descriptor.with("powered", "true")]
            }
            // Every count a weighted plate can show.
            "minecraft:light_weighted_pressure_plate"
            | "minecraft:heavy_weighted_pressure_plate" => {
                (0u8..16).map(|power| descriptor.with("power", &power.to_string())).collect()
            }
            "minecraft:water" | "minecraft:bubble_column" => {
                // Every level a flow can take, and air to empty into. Falling
                // water beyond level 8 never appears from our spread rules.
                (0u8..=8).map(|l| format!("minecraft:water[level={l}]")).collect()
            }
            "minecraft:piston" | "minecraft:sticky_piston" => {
                let sticky = descriptor.name == "minecraft:sticky_piston";
                let kind = if sticky { "sticky" } else { "normal" };
                let Some(facing) = descriptor.facing() else { continue };
                vec![
                    descriptor.with("extended", "false"),
                    descriptor.with("extended", "true"),
                    format!(
                        "minecraft:piston_head[facing={},short=false,type={kind}]",
                        face_name(facing)
                    ),
                    format!("minecraft:moving_piston[facing={},type={kind}]", face_name(facing)),
                    // Moved blocks always ride a type=normal placeholder, even
                    // when a sticky piston does the moving.
                    format!("minecraft:moving_piston[facing={},type=normal]", face_name(facing)),
                ]
            }
            _ => Vec::new(),
        };
        for companion in companions {
            let _ = registry.intern(&companion);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptors_split_into_name_and_properties() {
        let d = Descriptor::parse("minecraft:repeater[delay=2,facing=north,powered=false]");
        assert_eq!(d.name, "minecraft:repeater");
        assert_eq!(d.get("delay"), Some("2"));
        assert_eq!(d.facing(), Some(Dir::North));
        assert!(!d.flag("powered"));
    }

    #[test]
    fn a_descriptor_without_properties_still_parses() {
        let d = Descriptor::parse("minecraft:stone");
        assert_eq!(d.name, "minecraft:stone");
        assert!(d.properties.is_empty());
        assert_eq!(d.facing(), None);
    }

    #[test]
    fn with_rebuilds_a_sorted_descriptor() {
        // Sorted, because the registry interns by string: an unsorted rebuild would
        // mint a second id for a state that already exists.
        let d = Descriptor::parse("minecraft:repeater[facing=north,delay=2,powered=false]");
        assert_eq!(
            d.with("powered", "true"),
            "minecraft:repeater[delay=2,facing=north,powered=true]"
        );
    }

    #[test]
    fn companions_are_interned_so_blocks_can_change_state() {
        // A structure holds only the states it was saved with. Without their
        // counterparts a block cannot transition and the simulation freezes.
        let mut registry = StateRegistry::new();
        registry
            .intern("minecraft:piston[extended=false,facing=east]")
            .unwrap();
        intern_companions(&mut registry);

        assert!(registry.get("minecraft:piston[extended=true,facing=east]").is_some());
        assert!(registry
            .get("minecraft:piston_head[facing=east,short=false,type=normal]")
            .is_some());
        assert!(registry
            .get("minecraft:moving_piston[facing=east,type=normal]")
            .is_some());
    }

    #[test]
    fn a_whole_contraption_registers_without_gaps() {
        let mut registry = StateRegistry::new();
        for descriptor in [
            "minecraft:stone",
            "minecraft:redstone_block",
            "minecraft:sticky_piston[extended=false,facing=east]",
            "minecraft:observer[facing=west,powered=false]",
            "minecraft:repeater[delay=1,facing=east,locked=false,powered=false]",
            "minecraft:comparator[facing=west,mode=subtract,powered=false]",
            "minecraft:redstone_torch[lit=true]",
            "minecraft:slime_block",
            "minecraft:honey_block",
        ] {
            registry.intern(descriptor).unwrap();
        }
        intern_companions(&mut registry);

        let mut table = BehaviourTable::new();
        let rules = register_all(&mut registry, &mut table);

        table.note_unknown_in(&World::new(crate::pos::Bounds::new(
            Pos::new(0, 0, 0),
            Pos::new(1, 1, 1),
        )));
        assert_eq!(
            table.unknown_report(&registry),
            None,
            "every block above must be recognised"
        );
        assert!(!rules.slime.is_empty(), "slime must be classified sticky");
        assert!(!rules.honey.is_empty(), "honey must be classified sticky");
    }

    #[test]
    fn dust_re_faces_itself_when_a_neighbour_leaves() {
        // `RedStoneWireBlock.updateShape`. Dust beside a solid block faces it
        // (a block that takes a signal is worth connecting to); take the block
        // away and the connection has to go with it — otherwise the wire keeps
        // powering a hole, which is what kept this engine's dust frozen in the
        // shape its schematic was saved in.
        use crate::wire::{WireSide, WireWorld};
        let mut registry = StateRegistry::new();
        for descriptor in [
            "minecraft:air",
            "minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=none]",
            "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]",
            "minecraft:redstone_wire[east=side,north=side,power=0,south=side,west=side]",
            "minecraft:lever[face=floor,facing=west,powered=false]",
            "minecraft:cyan_concrete",
        ] {
            registry.intern(descriptor).unwrap();
        }
        intern_companions(&mut registry);
        let mut table = BehaviourTable::new();
        let rules = register_all(&mut registry, &mut table);

        let mut world = World::new(crate::pos::Bounds::new(
            Pos::new(-1, -1, -1),
            Pos::new(2, 2, 2),
        ));
        let dust = Pos::new(0, 0, 0);
        let east = Pos::new(1, 0, 0);
        world.set(
            dust,
            registry
                .get("minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=none]")
                .unwrap(),
        );
        world.set(east, registry.get("minecraft:lever[face=floor,facing=west,powered=false]").unwrap());

        // A lever is a signal source, so the wire faces it.
        let sides = crate::wire::connection_state(&rules, &world, dust, [WireSide::None; 4]);
        assert_eq!(sides[3], WireSide::Side, "east faces the lever");
        // With only one connection, the opposite face is forced to `side` too —
        // the symmetry rule that renders a lone neighbour as a line.
        assert_eq!(sides[2], WireSide::Side, "west is filled in by symmetry");
        assert_eq!(sides[0], WireSide::None);

        // Take the lever away and the wire has nothing to face — so the
        // symmetry rule fills in all four and it renders as a cross, which is
        // what isolated dust looks like in vanilla. A dot only survives if the
        // wire was already a dot.
        world.set(east, StateId::AIR);
        let sides = crate::wire::connection_state(&rules, &world, dust, sides);
        assert_eq!(sides, [WireSide::Side; 4], "isolated dust is a cross");
        let dot = crate::wire::connection_state(&rules, &world, dust, [WireSide::None; 4]);
        assert_eq!(dot, [WireSide::None; 4], "a dot with no neighbours stays a dot");
        assert!(
            rules.wire_with_shape(0, sides).is_some(),
            "the recomputed shape must resolve back to a real state"
        );
    }

    #[test]
    fn dust_climbs_a_full_block_and_only_leans_on_a_slab() {
        // `getConnectingSide`: the wire connects *up* a block it can walk onto
        // when dust sits above it, but renders that connection as `up` only
        // when the block is a full cube. A top slab carries the signal just the
        // same and stays `side`, which is why a slab staircase looks flat.
        use crate::wire::{WireSide, WireWorld};
        let mut registry = StateRegistry::new();
        for descriptor in [
            "minecraft:air",
            "minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]",
            "minecraft:cyan_concrete",
            "minecraft:smooth_stone_slab[type=top,waterlogged=false]",
        ] {
            registry.intern(descriptor).unwrap();
        }
        intern_companions(&mut registry);
        let mut table = BehaviourTable::new();
        let rules = register_all(&mut registry, &mut table);

        let mut world = World::new(crate::pos::Bounds::new(
            Pos::new(-1, -1, -1),
            Pos::new(3, 3, 3),
        ));
        let wire = registry
            .get("minecraft:redstone_wire[east=none,north=none,power=0,south=none,west=none]")
            .unwrap();
        let dust = Pos::new(0, 0, 0);
        world.set(dust, wire);
        // A full cube to the east with dust on top of it.
        world.set(Pos::new(1, 0, 0), registry.get("minecraft:cyan_concrete").unwrap());
        world.set(Pos::new(1, 1, 0), wire);
        assert_eq!(
            crate::wire::connecting_side(&rules, &world, dust, Dir::East, true),
            WireSide::Up
        );
        // The same climb over a top slab is a `side` connection.
        world.set(
            Pos::new(1, 0, 0),
            registry.get("minecraft:smooth_stone_slab[type=top,waterlogged=false]").unwrap(),
        );
        assert_eq!(
            crate::wire::connecting_side(&rules, &world, dust, Dir::East, true),
            WireSide::Side
        );
        // Cover the wire and it cannot climb at all.
        assert_eq!(
            crate::wire::connecting_side(&rules, &world, dust, Dir::East, false),
            WireSide::None,
            "a covered wire ignores what is above its neighbour"
        );
    }

    #[test]
    fn an_unrecognised_block_is_left_unregistered_rather_than_guessed() {
        // The whole point: a block we do not implement must be *named*, not
        // silently simulated as something plausible.
        let mut registry = StateRegistry::new();
        let hopper = registry.intern("minecraft:hopper[facing=down,enabled=true]").unwrap();
        let mut table = BehaviourTable::new();
        register_all(&mut registry, &mut table);

        assert!(!table.is_registered(hopper), "hopper is not implemented");
        table.note_unknown(hopper);
        let report = table.unknown_report(&registry).expect("must be reported");
        assert!(report.contains("minecraft:hopper"), "{report}");
    }

    #[test]
    fn a_strongly_powered_conductor_re_emits_on_every_face() {
        // Captured with the flying machine: an observer's back strongly powers
        // a slime block, and the piston on the slime's far side reads that
        // signal. Glass-like blocks are not conductors and stay dark.
        let mut registry = StateRegistry::new();
        let observer_on = registry
            .intern("minecraft:observer[facing=west,powered=true]")
            .unwrap();
        let slime = registry.intern("minecraft:slime_block").unwrap();
        let glass = registry.intern("minecraft:white_stained_glass").unwrap();
        registry
            .intern("minecraft:observer[facing=west,powered=false]")
            .unwrap();
        let mut table = BehaviourTable::new();
        let rules = register_all(&mut registry, &mut table);

        let mut world = World::new(crate::pos::Bounds::new(
            Pos::new(0, 0, 0),
            Pos::new(7, 1, 1),
        ));
        // observer(facing west, so its back is east) | slime | reader at x=2
        world.set(Pos::new(0, 0, 0), observer_on);
        world.set(Pos::new(1, 0, 0), slime);
        assert!(
            rules.is_powered(&world, &Default::default(), Pos::new(1, 0, 0), Dir::East),
            "the slime re-emits the strong power behind the observer"
        );

        world.set(Pos::new(1, 0, 0), glass);
        assert!(
            !rules.is_powered(&world, &Default::default(), Pos::new(1, 0, 0), Dir::East),
            "glass does not conduct"
        );
    }

    #[test]
    fn powered_components_are_classified_as_sources() {
        let mut registry = StateRegistry::new();
        let off = registry
            .intern("minecraft:repeater[delay=1,facing=east,locked=false,powered=false]")
            .unwrap();
        let on = registry
            .intern("minecraft:repeater[delay=1,facing=east,locked=false,powered=true]")
            .unwrap();
        let mut table = BehaviourTable::new();
        let rules = register_all(&mut registry, &mut table);

        assert!(rules.powered.contains(&on), "a powered repeater emits");
        assert!(!rules.powered.contains(&off), "an unpowered one does not");
        assert!(rules.diodes.contains_key(&off), "both states are diodes");
    }

    /// The blocks that rejected twenty of twenty-eight community doors, each
    /// one an ordinary piece of decoration. Taken from the minimal breaking
    /// sets the batch run bisected out of the real files.
    #[test]
    fn ordinary_decoration_is_building_material() {
        for name in [
            "minecraft:yellow_wool",
            "minecraft:light_blue_wool",
            "minecraft:green_wool",
            "minecraft:white_wool",
            "minecraft:red_concrete",
            "minecraft:yellow_concrete",
            "minecraft:orange_concrete",
            "minecraft:purple_concrete",
            "minecraft:purple_concrete_powder",
            "minecraft:magenta_stained_glass",
            "minecraft:quartz_bricks",
            "minecraft:stone_brick_wall",
            "minecraft:polished_deepslate_wall",
            "minecraft:oak_log",
            "minecraft:sand",
            "minecraft:gray_glazed_terracotta",
            "minecraft:polished_deepslate_stairs",
            "minecraft:cyan_carpet",
            "minecraft:white_stained_glass_pane",
        ] {
            assert!(decor_kind(name).is_some(), "{name} should be building material");
        }
    }

    /// Shape drives conduction and hopper suction, so a wall must never be
    /// mistaken for the stone it is cut from.
    #[test]
    fn decoration_keeps_its_shape() {
        for name in [
            "minecraft:stone_brick_wall",
            "minecraft:oak_fence",
            "minecraft:quartz_stairs",
            "minecraft:cyan_carpet",
            "minecraft:glass_pane",
        ] {
            assert_eq!(decor_kind(name), Some(Decor::Partial), "{name} is not a full cell");
            assert!(!is_full_cube(&Descriptor::parse(name)), "{name} must not fill its cell");
            assert!(!is_conductor(&Descriptor::parse(name)), "{name} must not conduct");
        }
        for name in ["minecraft:blue_stained_glass", "minecraft:birch_leaves", "minecraft:ice"] {
            assert_eq!(decor_kind(name), Some(Decor::Glassy), "{name} is glass-like");
            assert!(is_full_cube(&Descriptor::parse(name)), "{name} still fills its cell");
            assert!(!is_conductor(&Descriptor::parse(name)), "{name} must not conduct");
        }
        for name in ["minecraft:yellow_wool", "minecraft:red_concrete", "minecraft:oak_planks"] {
            assert!(is_conductor(&Descriptor::parse(name)), "{name} is an ordinary solid");
        }
    }

    /// `minecraft:piston_head` ends in `_head`. A decoration family that ran
    /// before the implemented blocks claimed it and replaced a piston's arm
    /// with a brick, which broke every door in the corpus — the reason the
    /// material check is the *last* arm of the registration match.
    #[test]
    fn implemented_blocks_outrank_the_decoration_families() {
        let mut registry = StateRegistry::new();
        let head = registry.intern("minecraft:piston_head[facing=up,short=false,type=normal]").unwrap();
        let mut table = BehaviourTable::default();
        register_all(&mut registry, &mut table);
        assert_eq!(
            table.get(head).map(|b| b.name()),
            Some("piston_head"),
            "a piston head is an arm, not decoration"
        );
    }

    /// Blocks that carry redstone state are not decoration, however solid they
    /// look. Asserting one away would silently change what a door does.
    #[test]
    fn redstone_components_are_never_called_decoration() {
        for name in [
            "minecraft:waxed_copper_bulb",
            "minecraft:waxed_oxidized_copper_bulb",
            "minecraft:piston_head",
            "minecraft:observer",
            "minecraft:redstone_lamp",
            "minecraft:iron_door",
        ] {
            assert_eq!(decor_kind(name), None, "{name} has behaviour worth modelling");
        }
    }

    /// The whole point, end to end: a build dressed in the decoration that used
    /// to be fatal still loads *and* still works. Twenty of twenty-eight
    /// community doors were rejected outright because a wool block somewhere in
    /// the casing had no behaviour, so this asserts both halves — nothing is
    /// reported unknown, and the piston the decoration surrounds still extends.
    #[test]
    fn a_decorated_build_loads_and_its_piston_still_fires() {
        use crate::Bounds;
        let mut sim = crate::Simulation::new(Bounds::new(Pos::new(0, 0, 0), Pos::new(15, 15, 15)));

        // A note block sitting on wool plays `guitar`, not `harp` — the
        // instrument is read off the block underneath. Interning only harp and
        // basedrum left every other note block without its powered partner and
        // dropped it as unimplemented, which is three more doors in the corpus.
        let decoration = [
            "minecraft:yellow_wool",
            "minecraft:purple_concrete",
            "minecraft:magenta_stained_glass",
            "minecraft:stone_brick_wall",
            "minecraft:polished_deepslate_stairs",
            "minecraft:cyan_carpet",
            "minecraft:oak_log",
            "minecraft:quartz_bricks",
            "minecraft:shroomlight",
            "minecraft:note_block[instrument=guitar,note=0,powered=false]",
        ];
        let mut placed = Vec::new();
        for (i, name) in decoration.iter().enumerate() {
            let id = sim.registry_mut().intern(name).unwrap();
            let at = Pos::new(7, 1 + i as i32, 7);
            sim.world_mut().set(at, id);
            placed.push((at, id, *name));
        }

        let piston = sim
            .registry_mut()
            .intern("minecraft:piston[extended=false,facing=east]")
            .unwrap();
        let lever = sim
            .registry_mut()
            .intern("minecraft:lever[face=floor,facing=north,powered=false]")
            .unwrap();
        let base = sim.registry_mut().intern("minecraft:stone").unwrap();
        sim.world_mut().set(Pos::new(2, 1, 2), piston);
        sim.world_mut().set(Pos::new(2, 0, 3), base);
        sim.world_mut().set(Pos::new(2, 1, 3), lever);

        intern_companions(sim.registry_mut());
        let mut table = std::mem::take(sim.behaviours_mut());
        register_all(sim.registry_mut(), &mut table);
        *sim.behaviours_mut() = table;

        assert_eq!(
            sim.unknown_report(),
            None,
            "ordinary decoration must not reject the build"
        );
        for (at, id, name) in placed {
            assert_eq!(sim.world().get(at), id, "{name} must survive registration");
        }

        sim.use_block(Pos::new(2, 1, 3));
        sim.run_until_quiescent(50);
        assert!(
            Descriptor::parse(sim.registry().descriptor(sim.world().get(Pos::new(2, 1, 2))).unwrap())
                .flag("extended"),
            "the piston inside the decoration must still extend"
        );
    }
}
