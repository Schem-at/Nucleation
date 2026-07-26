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
    Button, Comparator, ComparatorMode, Dropper, Hopper, Lamp, NoteBlock, PowerSource,
    PressurePlate, Repeater, StatePair, Torch,
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
    powered: Vec<StateId>,
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
    wires: HashMap<StateId, (u8, [bool; 4])>,
    /// `(wire state, power)` -> the same shape at that power.
    wire_siblings: HashMap<(StateId, u8), StateId>,
    immovable: Vec<StateId>,
    slime: Vec<StateId>,
    honey: Vec<StateId>,
    diodes: HashMap<StateId, Dir>,
}

impl VanillaRules {
    /// Whether any neighbour of `pos` strongly powers into it.
    fn strongly_powered(&self, world: &World, pos: Pos) -> bool {
        // Powered dust sitting on top of `pos` strongly powers it.
        if let Some((power, _)) = self.wires.get(&world.get(pos.offset(Dir::Up))) {
            if *power > 0 {
                return true;
            }
        }
        crate::pos::ALL_DIRS.iter().any(|dir| {
            let neighbour = world.get(pos.offset(*dir));
            self.strong_into.get(&neighbour) == Some(&dir.opposite())
        })
    }
}

impl VanillaRules {
    /// `is_powered` with `shouldSignal` off — every wire contribution muted,
    /// including dust strongly powering the conductor it sits on. This is what
    /// keeps a wire from feeding on its own floor block during evaluation.
    fn is_powered_no_wire(&self, world: &World, pos: Pos, toward: Dir) -> bool {
        let state = world.get(pos);
        if self.powered.contains(&state)
            && self.emit_only.get(&state).is_none_or(|only| *only == toward)
        {
            return true;
        }
        if !self.conductors.contains(&state) {
            return false;
        }
        crate::pos::ALL_DIRS.iter().any(|dir| {
            let neighbour = world.get(pos.offset(*dir));
            self.strong_into.get(&neighbour) == Some(&dir.opposite())
        })
    }
}

impl crate::wire::WireWorld for VanillaRules {
    /// `getBlockSignal`: the strongest non-wire signal into the wire —
    /// components, sources, and strongly powered conductors, with the stored
    /// comparator strength for exact analog levels.
    fn block_signal(&self, ctx: &crate::behaviour::TickCtx<'_>, pos: Pos) -> u8 {
        let mut best = 0u8;
        for dir in crate::pos::ALL_DIRS {
            let neighbour = pos.offset(dir);
            let state = ctx.world.get(neighbour);
            if self.wires.contains_key(&state) {
                continue; // shouldSignal is off while evaluating: wires never count
            }
            // A comparator's emitted strength is its stored block-entity value.
            let descriptor = ctx.states.descriptor(state).unwrap_or("");
            if descriptor.starts_with("minecraft:comparator") && descriptor.contains("powered=true")
            {
                let d = Descriptor::parse(descriptor);
                if d.facing().map(|f| f.opposite()) == Some(dir.opposite()) {
                    best = best.max(ctx.stored_comparator_output(neighbour));
                    continue;
                }
            }
            if self.is_powered_no_wire(ctx.world, neighbour, dir.opposite()) {
                best = best.max(15);
            }
        }
        best
    }

    fn conductor(&self, world: &World, pos: Pos) -> bool {
        self.full_cubes.contains(&world.get(pos))
    }

    fn wire_power(&self, world: &World, pos: Pos) -> Option<u8> {
        self.wires.get(&world.get(pos)).map(|(power, _)| *power)
    }

    fn wire_with_power(&self, world: &World, pos: Pos, power: u8) -> Option<StateId> {
        // The shape survives; only the power property changes. Resolved through
        // the registry via the sibling map built at registration.
        let state = world.get(pos);
        self.wire_siblings.get(&(state, power)).copied()
    }
}

impl PowerSource for VanillaRules {
    fn analog_signal(
        &self,
        world: &World,
        inventories: &crate::inventory::InventoryMap,
        pos: Pos,
    ) -> Option<u8> {
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

    fn container_slots_at(&self, world: &World, pos: Pos) -> Option<u32> {
        self.containers.get(&world.get(pos)).copied()
    }

    fn hopper_at(&self, world: &World, pos: Pos) -> bool {
        self.hoppers.contains(&world.get(pos))
    }

    fn is_solid_at(&self, world: &World, pos: Pos) -> bool {
        self.full_cubes.contains(&world.get(pos))
    }

    fn is_powered(&self, world: &World, pos: Pos, toward: Dir) -> bool {
        let state = world.get(pos);
        if self.powered.contains(&state)
            && self.emit_only.get(&state).is_none_or(|only| *only == toward)
        {
            return true;
        }
        // Dust: powers the block beneath it and the sides it points into,
        // never the block above.
        if let Some((power, connections)) = self.wires.get(&state) {
            if *power > 0 {
                match toward {
                    Dir::Down => return true,
                    Dir::Up => {}
                    side => {
                        let index = match side {
                            Dir::North => 0,
                            Dir::South => 1,
                            Dir::West => 2,
                            _ => 3,
                        };
                        if connections[index] {
                            return true;
                        }
                    }
                }
            }
        }
        // A strongly powered conductor re-emits weak power on every face.
        self.conductors.contains(&state) && self.strongly_powered(world, pos)
    }
    fn is_diode(&self, world: &World, pos: Pos) -> bool {
        self.diodes.contains_key(&world.get(pos))
    }
    fn diode_facing(&self, world: &World, pos: Pos) -> Option<Dir> {
        self.diodes.get(&world.get(pos)).copied()
    }
}

impl Movability for VanillaRules {
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
    "minecraft:piston_head",
    "minecraft:moving_piston",
    "minecraft:redstone_block",
];

/// Register vanilla behaviour for every state currently in `registry`.
///
/// Returns the rules, which callers may keep to interrogate power or movability.
/// Any state whose block is not recognised is left unregistered on purpose; see the
/// module docs.
pub fn register_all(registry: &mut StateRegistry, table: &mut BehaviourTable) -> VanillaRules {
    // Two passes. The first classifies every state, because a piston's behaviour
    // needs to know which blocks are sticky and which emit power — facts about
    // *other* states that may not have been seen yet when it is reached.
    let descriptors: Vec<(StateId, Descriptor)> = (0..registry.len())
        .map(|i| StateId(i as u16))
        .filter_map(|id| registry.descriptor(id).map(|d| (id, Descriptor::parse(d))))
        .collect();

    let mut rules = VanillaRules::default();
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
        if IMMOVABLE.contains(&descriptor.name.as_str()) {
            rules.immovable.push(*id);
        }
        if is_full_cube(descriptor) {
            rules.full_cubes.push(*id);
        }
        if is_conductor(descriptor) {
            rules.conductors.push(*id);
        }
        if descriptor.name == "minecraft:redstone_wire" {
            let power = descriptor
                .get("power")
                .and_then(|p| p.parse().ok())
                .unwrap_or(0);
            // Connection order matches Dir: North, South, West, East.
            let connected = |key: &str| descriptor.get(key).is_some_and(|v| v != "none");
            rules.wires.insert(
                *id,
                (
                    power,
                    [
                        connected("north"),
                        connected("south"),
                        connected("west"),
                        connected("east"),
                    ],
                ),
            );
        }
        // A powered diode, torch or observer is itself a source.
        let emits = match descriptor.name.as_str() {
            "minecraft:repeater" | "minecraft:comparator" | "minecraft:observer" => {
                descriptor.flag("powered")
            }
            "minecraft:stone_button"
            | "minecraft:oak_button"
            | "minecraft:stone_pressure_plate"
            | "minecraft:oak_pressure_plate" => descriptor.flag("powered"),
            "minecraft:redstone_torch" | "minecraft:redstone_wall_torch" => descriptor.flag("lit"),
            "minecraft:lever" => descriptor.flag("powered"),
            _ => false,
        };
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
            // Plates strongly power their floor; floor buttons theirs.
            if descriptor.name.ends_with("_pressure_plate")
                || (descriptor.name.ends_with("_button")
                    && descriptor.get("face") == Some("floor"))
            {
                rules.strong_into.insert(*id, Dir::Down);
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

        if INERT.contains(&name) {
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
            "minecraft:stone_button" | "minecraft:oak_button" => {
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(Button {
                        powered: descriptor.flag("powered"),
                        states,
                        // BlockSetType: stone presses for 20 game ticks, wood 30.
                        duration: if name == "minecraft:stone_button" { 20 } else { 30 },
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
            "minecraft:stone_pressure_plate" | "minecraft:oak_pressure_plate" => {
                let Some(states) = powered_pair(registry, descriptor) else { continue };
                table.register(
                    *id,
                    Box::new(PressurePlate {
                        powered: descriptor.flag("powered"),
                        states,
                        senses_items: name == "minecraft:oak_pressure_plate",
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
                table.register(
                    *id,
                    Box::new(NoteBlock {
                        powered: descriptor.flag("powered"),
                        states,
                        cycled,
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
        "minecraft:hopper" => Some(5),
        "minecraft:dropper" | "minecraft:dispenser" => Some(9),
        _ => None,
    }
}

/// Whether a block state conducts redstone (`isRedstoneConductor`).
///
/// Full cubes conduct, with the redstone-transparent exceptions: the glass
/// family (the glass-diode captures prove it), sources like the redstone
/// block, and observers. Slime conducting is capture-verified (the flying
/// machine); stone conducting is capture-verified (dust soft-powering a
/// piston through its floor block).
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
            | "minecraft:redstone_lamp"
            | "minecraft:observer"
            | "minecraft:slime_block"
            | "minecraft:honey_block"
    ) || matches!(descriptor.name.as_str(), "minecraft:slime_block")
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
        "minecraft:air" => false,
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
        | "minecraft:moving_piston"
        | "minecraft:stone_button"
        | "minecraft:oak_button"
        | "minecraft:stone_pressure_plate"
        | "minecraft:oak_pressure_plate" => false,
        "minecraft:piston" | "minecraft:sticky_piston" => !descriptor.flag("extended"),
        _ => true,
    }
}

/// Collision and friction tables for item physics, indexed by `StateId`.
///
/// Friction is `Block.getFriction`: 0.6 for almost everything, 0.8 for slime.
pub fn physics_tables(registry: &StateRegistry) -> (Vec<bool>, Vec<f32>) {
    let mut solidity = Vec::with_capacity(registry.len());
    let mut frictions = Vec::with_capacity(registry.len());
    for index in 0..registry.len() {
        let descriptor = registry
            .descriptor(StateId(index as u16))
            .map(Descriptor::parse);
        let (solid, friction) = match &descriptor {
            None => (false, 0.6),
            Some(d) => (
                is_full_cube(d),
                if d.name == "minecraft:slime_block" { 0.8 } else { 0.6 },
            ),
        };
        solidity.push(solid);
        frictions.push(friction);
    }
    (solidity, frictions)
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
pub fn intern_companions(registry: &mut StateRegistry) {
    let existing: Vec<String> = (0..registry.len())
        .filter_map(|i| registry.descriptor(StateId(i as u16)).map(str::to_string))
        .collect();

    for text in existing {
        let descriptor = Descriptor::parse(&text);
        let companions: Vec<String> = match descriptor.name.as_str() {
            "minecraft:repeater" | "minecraft:comparator" | "minecraft:observer"
            | "minecraft:lever" => {
                vec![descriptor.with("powered", "false"), descriptor.with("powered", "true")]
            }
            "minecraft:redstone_torch" | "minecraft:redstone_wall_torch" => {
                vec![descriptor.with("lit", "false"), descriptor.with("lit", "true")]
            }
            "minecraft:redstone_wire" => {
                (0u8..16).map(|p| descriptor.with("power", &p.to_string())).collect()
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
                let mut all = Vec::new();
                for note in 0..crate::components::NOTE_VALUES {
                    let at_note = Descriptor::parse(&descriptor.with("note", &note.to_string()));
                    all.push(at_note.with("powered", "false"));
                    all.push(at_note.with("powered", "true"));
                }
                all
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
            rules.is_powered(&world, Pos::new(1, 0, 0), Dir::East),
            "the slime re-emits the strong power behind the observer"
        );

        world.set(Pos::new(1, 0, 0), glass);
        assert!(
            !rules.is_powered(&world, Pos::new(1, 0, 0), Dir::East),
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
}
