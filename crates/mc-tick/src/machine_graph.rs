//! Static structural analysis of piston contraptions: what a machine is *made
//! of*, and — sometimes — proof that it cannot possibly fly.
//!
//! # Why this is not a simulation
//!
//! A genetic search over flying machines spends nearly all of its time ticking
//! genomes that were never going to move. Simulation is the only way to prove a
//! machine *does* fly, but it is a very expensive way to discover that a machine
//! has no observer, or that every piston in it is pointed at bedrock.
//!
//! The asymmetry this module trades on: **static analysis cannot prove a machine
//! flies, but it can prove one can't.** Every verdict here is a *sound negative* —
//! a rejection is a claim that no sequence of ticks can produce motion, and it
//! must never be wrong. An over-eager filter that discards a working machine is
//! strictly worse than no filter at all, because the search never learns what it
//! threw away.
//!
//! # Nothing here re-implements a push
//!
//! Every question of the form "what would this piston move?" is answered by
//! [`crate::piston::resolve_push`] and [`crate::piston::resolve_pull`], the same
//! oracle-verified code the tick loop runs. The push rules are subtle — the
//! twelve-block limit, `PushReaction.DESTROY`, an extended base not being a full
//! cube, slime and honey refusing to stick to *each other* — and a second copy of
//! them written for the analyser would drift from the first one silently. So
//! there is no second copy.
//!
//! # The graph
//!
//! * **groups** — maximal sets of blocks joined by adhesion ([`crate::piston::adheres`]).
//!   A lone solid is its own group. These are a *view* construct: they say which
//!   blocks visibly hang together. They are deliberately **not** used for any
//!   soundness claim, because adhesion's transitive closure over-approximates
//!   rigid motion (vanilla branches only *from* sticky blocks, so slime-stone-slime
//!   is one group here but does not travel as one).
//! * **devices** — one node per piston, observer and power source.
//! * **edges** — `sticks_to` (device to the group it rides), `pushes` (piston to
//!   each group its resolved plan would move), `powers` (a source's signal can
//!   reach a piston's power region, quasi-connectivity included) and `observes`
//!   (an observer's watched cell).
//!
//! # Phase
//!
//! A contraption's push sets depend on where it is in its stroke, so "at rest" is
//! not enough. Each piston is resolved in **two phases**: the world as given
//! (the extend stroke), and the world with that piston's own [`crate::piston::PushPlan`]
//! applied (the retract stroke, via `resolve_pull`). The second phase is built by
//! applying the verified plan, not by guessing at the geometry. A piston's
//! *influence* is the union over both phases of what it moves, where those blocks
//! land, and what it destroys.
//!
//! This is a bounded unrolling, not a fixed point, and that is stated plainly
//! wherever a rejection depends on it — see [`MachineGraph::rejections`].

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};

use crate::piston::{adheres, resolve_pull, resolve_push, Movability, Sticky};
use crate::pos::{Dir, Pos, ALL_DIRS};
use crate::state::{StateId, StateRegistry};
use crate::world::World;

/// How many device subsets the engine search will consider before giving up.
///
/// Engine classification enumerates simple cycles of the drive graph; a
/// pathological graph has exponentially many. The cap keeps a per-genome call
/// bounded, and hitting it is reported rather than hidden.
pub const MAX_ENGINE_CANDIDATES: usize = 512;

/// The longest simple drive cycle the engine search will follow.
pub const MAX_CYCLE_LEN: usize = 10;

/* ------------------------------------------------------------ part table */

/// What one block state *is*, structurally.
///
/// Mirrors the registry-row shape used by [`crate::entity_kind`]: one row per
/// state, built once from the [`StateRegistry`]'s descriptors, and every lookup
/// goes through the table rather than re-parsing a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartKind {
    /// A piston base.
    Piston {
        /// Whether it pulls on retraction.
        sticky: bool,
        /// The direction its head travels.
        facing: Dir,
        /// Whether it is already extended in the given world.
        extended: bool,
    },
    /// An observer, with the direction it *watches*. It emits from the opposite
    /// face.
    Observer {
        /// The watched direction.
        facing: Dir,
    },
    /// Anything that can emit redstone power on its own — a redstone block, a
    /// lever, a torch, dust, a repeater.
    Source,
    /// A slime block.
    Slime,
    /// A honey block.
    Honey,
    /// Ordinary build material.
    Solid,
}

/// Block names that emit or carry a redstone signal of their own.
///
/// Deliberately generous: a name missed here becomes [`PartKind::Solid`], and a
/// solid still conducts in the power over-approximation, so a miss costs
/// precision and never soundness.
const SOURCE_NAMES: &[&str] = &[
    "minecraft:redstone_block",
    "minecraft:redstone_wire",
    "minecraft:redstone_torch",
    "minecraft:redstone_wall_torch",
    "minecraft:repeater",
    "minecraft:comparator",
    "minecraft:lever",
    "minecraft:daylight_detector",
    "minecraft:target",
    "minecraft:trapped_chest",
    "minecraft:detector_rail",
    "minecraft:sculk_sensor",
    "minecraft:calibrated_sculk_sensor",
    "minecraft:lightning_rod",
];

/// One row per block state: what it is, structurally.
#[derive(Debug, Clone, Default)]
pub struct PartTable {
    rows: HashMap<StateId, PartKind>,
}

impl PartTable {
    /// Build a table covering every non-air state present in `world`.
    ///
    /// Scoped to the world rather than the whole registry because the registry
    /// carries every state the sim ever interned, most of which the build does
    /// not contain.
    pub fn from_world(world: &World, registry: &StateRegistry) -> Self {
        let mut rows = HashMap::new();
        for (_, state) in world.iter_non_air() {
            if rows.contains_key(&state) {
                continue;
            }
            let kind = registry.descriptor(state).map_or(PartKind::Solid, classify);
            rows.insert(state, kind);
        }
        PartTable { rows }
    }

    /// The row for a state, or [`PartKind::Solid`] for anything unregistered.
    pub fn kind(&self, state: StateId) -> PartKind {
        self.rows.get(&state).copied().unwrap_or(PartKind::Solid)
    }

    /// The row for whatever stands at `pos`, or `None` for air.
    pub fn at(&self, world: &World, pos: Pos) -> Option<PartKind> {
        let state = world.get(pos);
        if state == StateId::AIR {
            None
        } else {
            Some(self.kind(state))
        }
    }
}

/// Read a blockstate property out of a descriptor's `[k=v,...]` tail.
fn prop<'a>(descriptor: &'a str, key: &str) -> Option<&'a str> {
    let tail = descriptor.split_once('[')?.1.trim_end_matches(']');
    tail.split(',').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k.trim() == key).then(|| v.trim())
    })
}

/// Parse a facing name. Defaults to north so a malformed descriptor cannot
/// panic mid-search; a wrong facing costs precision, never soundness, because
/// every rejection also requires a resolved [`crate::piston::PushPlan`].
fn parse_dir(name: Option<&str>) -> Dir {
    match name {
        Some("east") => Dir::East,
        Some("west") => Dir::West,
        Some("up") => Dir::Up,
        Some("down") => Dir::Down,
        Some("south") => Dir::South,
        _ => Dir::North,
    }
}

/// Classify one blockstate descriptor.
pub fn classify(descriptor: &str) -> PartKind {
    let name = descriptor.split_once('[').map_or(descriptor, |(n, _)| n);
    match name {
        "minecraft:sticky_piston" | "minecraft:piston" => PartKind::Piston {
            sticky: name == "minecraft:sticky_piston",
            facing: parse_dir(prop(descriptor, "facing")),
            extended: prop(descriptor, "extended") == Some("true"),
        },
        "minecraft:observer" => {
            PartKind::Observer { facing: parse_dir(prop(descriptor, "facing")) }
        }
        "minecraft:slime_block" => PartKind::Slime,
        "minecraft:honey_block" => PartKind::Honey,
        _ if SOURCE_NAMES.contains(&name) => PartKind::Source,
        _ if name.ends_with("_button") || name.ends_with("_pressure_plate") => PartKind::Source,
        _ => PartKind::Solid,
    }
}

/* ---------------------------------------------------------------- graph */

/// What a device node is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceKind {
    /// A piston base.
    Piston {
        /// Whether it pulls on retraction.
        sticky: bool,
    },
    /// An observer.
    Observer,
    /// A standalone power source.
    Source,
}

/// A piston, observer or power source, with everything static analysis knows
/// about it.
#[derive(Debug, Clone)]
pub struct Device {
    /// Index into [`MachineGraph::devices`].
    pub id: usize,
    /// Where it stands.
    pub pos: Pos,
    /// What it is.
    pub kind: DeviceKind,
    /// A piston's travel direction; an observer's watched direction.
    pub facing: Dir,
    /// The adhesion group it rides.
    pub group: usize,
    /// Already extended in the given world — the machine was captured mid-stroke.
    pub extended: bool,
    /// Whether [`crate::piston::resolve_push`] says the extend stroke is possible.
    pub can_extend: bool,
    /// Exactly what the extend stroke would move, from `resolve_push`.
    pub push: Vec<Pos>,
    /// Exactly what the following retract stroke would pull, from `resolve_pull`
    /// applied to the post-extension world.
    pub pull: Vec<Pos>,
    /// Every cell this device can change over both phases: what it moves, where
    /// those blocks land, what it destroys, and its own head cell.
    pub influence: BTreeSet<Pos>,
}

/// A maximal set of blocks joined by adhesion.
#[derive(Debug, Clone)]
pub struct Group {
    /// Index into [`MachineGraph::groups`].
    pub id: usize,
    /// Member cells, sorted.
    pub cells: Vec<Pos>,
}

/// Which node an edge points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRef {
    /// An adhesion group.
    Group(usize),
    /// A piston, observer or source.
    Device(usize),
}

impl NodeRef {
    fn label(self) -> String {
        match self {
            NodeRef::Group(i) => format!("g{i}"),
            NodeRef::Device(i) => format!("d{i}"),
        }
    }
}

/// The three relations the graph carries, plus the observer's watched cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// The device rides this group.
    SticksTo,
    /// The piston's resolved plan moves blocks of this group.
    Pushes,
    /// This source's signal can reach that piston's power region.
    Powers,
    /// This device can change the cell that observer watches.
    Observes,
}

impl EdgeKind {
    fn name(self) -> &'static str {
        match self {
            EdgeKind::SticksTo => "sticks_to",
            EdgeKind::Pushes => "pushes",
            EdgeKind::Powers => "powers",
            EdgeKind::Observes => "observes",
        }
    }
}

/// One edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edge {
    /// Which relation.
    pub kind: EdgeKind,
    /// Source node.
    pub from: NodeRef,
    /// Target node.
    pub to: NodeRef,
}

/// A proof that the machine cannot move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    /// Stable machine-readable code.
    pub code: &'static str,
    /// What was shown, in words.
    pub reason: String,
    /// Whether the claim is "no block in this build ever moves" (`true`) or the
    /// weaker "this build cannot propel itself repeatedly" (`false`).
    ///
    /// The distinction is not pedantry, and it was found by measurement rather
    /// than assumed. A search that scores centre-of-mass displacement reads a
    /// *piston head appearing* as motion — a lone east-facing piston, kicked
    /// once from outside, scores two thirds of a block without the build going
    /// anywhere. Every rejection about drive topology is therefore conditional:
    /// it is true about flight and false about that score.
    ///
    /// See [`MachineGraph::rejected`] and [`MachineGraph::rejected_for_sustained`].
    pub unconditional: bool,
}

/// A minimal self-translating subgraph — the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    /// Device ids forming the drive cycle.
    pub devices: Vec<usize>,
    /// Every cell of the engine: its devices plus the connective material that
    /// links them.
    pub cells: Vec<Pos>,
}

/// The whole static picture of one machine.
#[derive(Debug, Clone, Default)]
pub struct MachineGraph {
    /// Adhesion groups.
    pub groups: Vec<Group>,
    /// Pistons, observers and sources.
    pub devices: Vec<Device>,
    /// `sticks_to` / `pushes` / `powers` / `observes`.
    pub edges: Vec<Edge>,
    /// Every minimal self-translating subgraph. More than one is a real design:
    /// a dual-engine machine has two.
    pub engines: Vec<Engine>,
    /// Carried, not driving: inside some engine's push closure, outside the
    /// engine.
    pub payload: Vec<Pos>,
    /// Devices in no drive cycle — they fire to start the machine and are then
    /// irrelevant.
    pub kickers: Vec<usize>,
    /// Neither driven nor driving.
    pub dead_weight: Vec<Pos>,
    /// Sound proofs that the machine cannot move. Empty means "not disproved",
    /// which is not the same as "flies".
    pub rejections: Vec<Rejection>,
    /// True when the engine search stopped at [`MAX_ENGINE_CANDIDATES`], so
    /// `engines` may be incomplete.
    pub engine_search_truncated: bool,
}

impl MachineGraph {
    /// Whether not one block of this build can ever move.
    ///
    /// Safe for any caller, whatever it scores. This is the tier a search should
    /// use by default.
    pub fn rejected(&self) -> bool {
        self.rejections.iter().any(|r| r.unconditional)
    }

    /// Whether the machine is provably incapable of *sustained* self-propulsion.
    ///
    /// Strictly stronger filtering, and only safe for a caller that requires
    /// sustained flight — the app's `requireSustained` constraint, or any run
    /// that would not accept a one-stroke lurch as a result.
    pub fn rejected_for_sustained(&self) -> bool {
        !self.rejections.is_empty()
    }
}

/* -------------------------------------------------------------- analysis */

/// Build the static graph of the contraption standing in `world`.
///
/// `movability` is the same trait object the tick loop uses, so `resolve_push`
/// answers here exactly what it answers there.
pub fn analyse(world: &World, registry: &StateRegistry, movability: &dyn Movability) -> MachineGraph {
    let table = PartTable::from_world(world, registry);
    let cells: Vec<Pos> = world.iter_non_air().map(|(p, _)| p).collect();
    if cells.is_empty() {
        let mut graph = MachineGraph::default();
        graph.rejections.push(Rejection {
            code: "empty",
            reason: "the build has no blocks".into(),
            unconditional: true,
        });
        return graph;
    }
    let occupied: HashSet<Pos> = cells.iter().copied().collect();

    let (groups, group_of) = adhesion_groups(world, movability, &cells);
    let blobs = touch_blobs(&cells, &occupied);

    let mut devices = build_devices(world, &table, &cells, &group_of);
    resolve_strokes(world, movability, &mut devices);

    let mut edges = Vec::new();
    structural_edges(&devices, &group_of, &mut edges);
    let drive = drive_edges(&devices, &blobs, &mut edges);

    let mut graph = MachineGraph {
        groups,
        devices,
        edges,
        ..MachineGraph::default()
    };
    classify_sections(&mut graph, &drive, &occupied);
    reject(&mut graph, &drive);
    graph
}

/* ------------------------------------------------------------- grouping */

/// Partition the build into maximal adhesion-connected sets.
fn adhesion_groups(
    world: &World,
    movability: &dyn Movability,
    cells: &[Pos],
) -> (Vec<Group>, HashMap<Pos, usize>) {
    let sticky: HashMap<Pos, Option<Sticky>> =
        cells.iter().map(|&p| (p, movability.sticky(world, p))).collect();
    let mut group_of: HashMap<Pos, usize> = HashMap::new();
    let mut groups: Vec<Group> = Vec::new();
    for &seed in cells {
        if group_of.contains_key(&seed) {
            continue;
        }
        let id = groups.len();
        let mut members = Vec::new();
        let mut queue = VecDeque::from([seed]);
        group_of.insert(seed, id);
        while let Some(pos) = queue.pop_front() {
            members.push(pos);
            for dir in ALL_DIRS {
                let next = pos.offset(dir);
                if group_of.contains_key(&next) {
                    continue;
                }
                let Some(&other) = sticky.get(&next) else { continue };
                if adheres(sticky[&pos], other) {
                    group_of.insert(next, id);
                    queue.push_back(next);
                }
            }
        }
        members.sort_by_key(|p| (p.x, p.y, p.z));
        groups.push(Group { id, cells: members });
    }
    (groups, group_of)
}

/// Connected components of *touching* blocks, ignoring adhesion.
///
/// The power over-approximation travels along these: a redstone signal cannot
/// cross a gap of air, and treating every non-air block as a conductor is the
/// safe direction — it can only invent power paths, never miss one.
fn touch_blobs(cells: &[Pos], occupied: &HashSet<Pos>) -> HashMap<Pos, usize> {
    let mut blob: HashMap<Pos, usize> = HashMap::new();
    let mut next = 0usize;
    for &seed in cells {
        if blob.contains_key(&seed) {
            continue;
        }
        let id = next;
        next += 1;
        let mut queue = VecDeque::from([seed]);
        blob.insert(seed, id);
        while let Some(pos) = queue.pop_front() {
            for dir in ALL_DIRS {
                let n = pos.offset(dir);
                if occupied.contains(&n) && !blob.contains_key(&n) {
                    blob.insert(n, id);
                    queue.push_back(n);
                }
            }
        }
    }
    blob
}

/* -------------------------------------------------------------- devices */

fn build_devices(
    world: &World,
    table: &PartTable,
    cells: &[Pos],
    group_of: &HashMap<Pos, usize>,
) -> Vec<Device> {
    let mut devices = Vec::new();
    for &pos in cells {
        let Some(kind) = table.at(world, pos) else { continue };
        let (dk, facing, extended) = match kind {
            PartKind::Piston { sticky, facing, extended } => {
                (DeviceKind::Piston { sticky }, facing, extended)
            }
            PartKind::Observer { facing } => (DeviceKind::Observer, facing, false),
            PartKind::Source => (DeviceKind::Source, Dir::Up, false),
            _ => continue,
        };
        devices.push(Device {
            id: devices.len(),
            pos,
            kind: dk,
            facing,
            group: group_of[&pos],
            extended,
            can_extend: false,
            push: Vec::new(),
            pull: Vec::new(),
            influence: BTreeSet::new(),
        });
    }
    devices
}

/// Resolve both phases of every piston's stroke.
///
/// Phase 1 is the world as given. Phase 2 is that world with the piston's own
/// resolved plan applied — the geometry a retraction actually starts from. Both
/// come from the engine's own resolver; nothing here decides what moves.
fn resolve_strokes(world: &World, movability: &dyn Movability, devices: &mut [Device]) {
    for device in devices.iter_mut() {
        let DeviceKind::Piston { sticky } = device.kind else {
            // An observer changes only its own state; its influence is its own
            // cell, which is what lets a *moved* observer re-trigger a watcher.
            device.influence.insert(device.pos);
            continue;
        };
        let dir = device.facing;
        let plan = resolve_push(world, movability, device.pos, dir);
        device.can_extend = plan.possible;
        device.influence.insert(device.pos);
        device.influence.insert(device.pos.offset(dir));
        if plan.possible {
            device.push = plan.to_push.clone();
            for &p in &plan.to_push {
                device.influence.insert(p);
                device.influence.insert(p.offset(dir));
            }
            for &p in &plan.to_destroy {
                device.influence.insert(p);
            }

            // Phase 2: apply the plan, then ask the same resolver what the
            // retraction pulls back.
            if sticky {
                if let Some(extended) = apply_push(world, &plan, dir) {
                    let back = resolve_pull(&extended, movability, device.pos, dir);
                    if back.possible {
                        device.pull = back.to_push.clone();
                        let rev = dir.opposite();
                        for &p in &back.to_push {
                            device.influence.insert(p);
                            device.influence.insert(p.offset(rev));
                        }
                        for &p in &back.to_destroy {
                            device.influence.insert(p);
                        }
                    }
                }
            }
        }
    }
}

/// The world after `plan` has been carried out.
///
/// Sources are read before any write so the result does not depend on the order
/// `PistonStructureResolver` happened to collect them in — the analyser must not
/// smuggle in an ordering assumption the tick loop is entitled to change.
fn apply_push(world: &World, plan: &crate::piston::PushPlan, dir: Dir) -> Option<World> {
    let mut next = world.clone();
    let moved: Vec<(Pos, StateId)> = plan.to_push.iter().map(|&p| (p, world.get(p))).collect();
    for &p in &plan.to_destroy {
        if world.contains(p) {
            next.set(p, StateId::AIR);
        }
    }
    for &(p, _) in &moved {
        next.set(p, StateId::AIR);
    }
    for &(p, state) in &moved {
        let dest = p.offset(dir);
        if !next.contains(dest) {
            // The plan runs off the edge of the analysed region: the retract
            // phase would be built on a lie, so decline it.
            return None;
        }
        next.set(dest, state);
    }
    // The head now occupies the cell in front of the base. Leaving it air is the
    // conservative reading for a *pull*: `resolve_pull` walks back toward the
    // base and stops at the first empty cell, exactly where the head is.
    Some(next)
}

/* ---------------------------------------------------------------- edges */

fn structural_edges(devices: &[Device], group_of: &HashMap<Pos, usize>, edges: &mut Vec<Edge>) {
    for device in devices {
        edges.push(Edge {
            kind: EdgeKind::SticksTo,
            from: NodeRef::Device(device.id),
            to: NodeRef::Group(device.group),
        });
        let mut touched: BTreeSet<usize> = BTreeSet::new();
        for p in device.push.iter().chain(device.pull.iter()) {
            if let Some(&g) = group_of.get(p) {
                touched.insert(g);
            }
        }
        for g in touched {
            edges.push(Edge {
                kind: EdgeKind::Pushes,
                from: NodeRef::Device(device.id),
                to: NodeRef::Group(g),
            });
        }
    }
}

/// The causal graph: who can make whom fire.
///
/// Returns adjacency over device ids. Both edge kinds are deliberate
/// over-approximations, because a *missing* edge is what would make the
/// acyclic-drive rejection unsound.
fn drive_edges(
    devices: &[Device],
    blobs: &HashMap<Pos, usize>,
    edges: &mut Vec<Edge>,
) -> Vec<Vec<usize>> {
    let mut adj = vec![Vec::new(); devices.len()];

    // powers: a source's signal reaches a piston's power region. The signal is
    // allowed to travel anywhere within the touching blob it is emitted into —
    // strictly more than vanilla permits, so no real path is missed.
    for src in devices {
        let emit = match src.kind {
            DeviceKind::Observer => src.pos.offset(src.facing.opposite()),
            DeviceKind::Source => src.pos,
            DeviceKind::Piston { .. } => continue,
        };
        let mut reach: HashSet<usize> = HashSet::new();
        if let Some(&b) = blobs.get(&emit) {
            reach.insert(b);
        }
        for target in devices {
            if target.id == src.id || !matches!(target.kind, DeviceKind::Piston { .. }) {
                continue;
            }
            if power_region(target.pos).into_iter().any(|c| {
                c == emit || blobs.get(&c).is_some_and(|b| reach.contains(b))
            }) {
                adj[src.id].push(target.id);
                edges.push(Edge {
                    kind: EdgeKind::Powers,
                    from: NodeRef::Device(src.id),
                    to: NodeRef::Device(target.id),
                });
            }
        }
    }

    // observes: an observer fires when the block state in its watched cell
    // changes. Anything whose influence covers that cell can cause it — and if a
    // device stands there, its own state change counts too.
    for obs in devices {
        if !matches!(obs.kind, DeviceKind::Observer) {
            continue;
        }
        let watched = obs.pos.offset(obs.facing);
        edges.push(Edge {
            kind: EdgeKind::Observes,
            from: NodeRef::Device(obs.id),
            to: match devices.iter().find(|d| d.pos == watched) {
                Some(d) => NodeRef::Device(d.id),
                None => NodeRef::Device(obs.id),
            },
        });
        for other in devices {
            if other.id == obs.id {
                continue;
            }
            let moves_it = other.influence.contains(&watched);
            let is_it = other.pos == watched;
            if moves_it || is_it {
                adj[other.id].push(obs.id);
            }
        }
    }

    for row in adj.iter_mut() {
        row.sort_unstable();
        row.dedup();
    }
    adj
}

/// Every cell whose emission can power a piston at `pos`.
///
/// `PistonBaseBlock.getNeighborSignal`: the six neighbours of the base, plus the
/// six neighbours of the block above it — quasi-connectivity, which is not
/// optional in a contraption this small.
fn power_region(pos: Pos) -> Vec<Pos> {
    let up = pos.offset(Dir::Up);
    let mut cells = Vec::with_capacity(12);
    for dir in ALL_DIRS {
        cells.push(pos.offset(dir));
        cells.push(up.offset(dir));
    }
    cells
}

/* ------------------------------------------------------- classification */

/// Find every minimal self-translating subgraph, then label the rest.
fn classify_sections(graph: &mut MachineGraph, drive: &[Vec<usize>], occupied: &HashSet<Pos>) {
    let cycles = simple_cycles(drive, MAX_ENGINE_CANDIDATES, MAX_CYCLE_LEN);
    graph.engine_search_truncated = cycles.len() >= MAX_ENGINE_CANDIDATES;

    // A candidate is an engine when the push closure of its own pistons carries
    // every one of its own devices. That is what "self-translating" means: the
    // set shoves itself, rather than shoving something else.
    let mut candidates: Vec<BTreeSet<usize>> = Vec::new();
    for cycle in &cycles {
        let set: BTreeSet<usize> = cycle.iter().copied().collect();
        if self_translating(graph, &set) && !candidates.contains(&set) {
            candidates.push(set);
        }
    }
    // A single cycle is not always enough — two interlocking cycles can be
    // self-translating only together. Try pairwise unions, bounded.
    if candidates.is_empty() {
        'outer: for (i, a) in cycles.iter().enumerate() {
            for b in cycles.iter().skip(i + 1) {
                let set: BTreeSet<usize> = a.iter().chain(b.iter()).copied().collect();
                if self_translating(graph, &set) && !candidates.contains(&set) {
                    candidates.push(set);
                    if candidates.len() >= 8 {
                        break 'outer;
                    }
                }
            }
        }
    }
    // Minimal by inclusion.
    let minimal: Vec<BTreeSet<usize>> = candidates
        .iter()
        .filter(|c| !candidates.iter().any(|o| o != *c && o.is_subset(c)))
        .cloned()
        .collect();

    let mut engine_cells: BTreeSet<Pos> = BTreeSet::new();
    for set in &minimal {
        let devices: Vec<usize> = set.iter().copied().collect();
        let cells = engine_cells_of(graph, &devices, occupied);
        engine_cells.extend(cells.iter().copied());
        graph.engines.push(Engine { devices, cells });
    }

    // Payload: carried by an engine, not part of one.
    let mut payload: BTreeSet<Pos> = BTreeSet::new();
    for set in &minimal {
        for &id in set {
            for p in &graph.devices[id].influence {
                if occupied.contains(p) && !engine_cells.contains(p) {
                    payload.insert(*p);
                }
            }
        }
    }
    graph.payload = payload.iter().copied().collect();

    // Kickers: a device in no drive cycle at all.
    let in_a_cycle: HashSet<usize> = cycles.iter().flatten().copied().collect();
    graph.kickers = graph
        .devices
        .iter()
        .filter(|d| !in_a_cycle.contains(&d.id) && !engine_cells.contains(&d.pos))
        .map(|d| d.id)
        .collect();

    // Dead weight: neither driven nor driving.
    let kicker_cells: HashSet<Pos> = graph.kickers.iter().map(|&i| graph.devices[i].pos).collect();
    let mut dead: Vec<Pos> = occupied
        .iter()
        .copied()
        .filter(|p| {
            !engine_cells.contains(p) && !payload.contains(p) && !kicker_cells.contains(p)
        })
        .collect();
    dead.sort_by_key(|p| (p.x, p.y, p.z));
    graph.dead_weight = dead;
}

/// Does the push closure of this set's own pistons carry the set's own devices?
fn self_translating(graph: &MachineGraph, set: &BTreeSet<usize>) -> bool {
    let mut closure: HashSet<Pos> = HashSet::new();
    let mut has_piston = false;
    for &id in set {
        let d = &graph.devices[id];
        if matches!(d.kind, DeviceKind::Piston { .. }) {
            has_piston = true;
            closure.extend(d.push.iter().copied());
            closure.extend(d.pull.iter().copied());
        }
    }
    has_piston && set.iter().all(|&id| closure.contains(&graph.devices[id].pos))
}

/// The engine's blocks: its devices plus the material that connects them.
///
/// Deliberately *not* "the groups the devices belong to" — attach a slime cargo
/// to a machine and the group grows, but the engine has not changed. A spanning
/// connector over touching blocks, anchored at the lowest device id, keeps the
/// engine fixed while the payload varies.
fn engine_cells_of(graph: &MachineGraph, devices: &[usize], occupied: &HashSet<Pos>) -> Vec<Pos> {
    let mut cells: BTreeSet<Pos> = devices.iter().map(|&i| graph.devices[i].pos).collect();
    let Some(&anchor) = devices.first() else { return Vec::new() };
    let start = graph.devices[anchor].pos;

    let mut prev: HashMap<Pos, Pos> = HashMap::new();
    let mut seen: HashSet<Pos> = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(pos) = queue.pop_front() {
        for dir in ALL_DIRS {
            let n = pos.offset(dir);
            if occupied.contains(&n) && seen.insert(n) {
                prev.insert(n, pos);
                queue.push_back(n);
            }
        }
    }
    for &id in devices.iter().skip(1) {
        let mut at = graph.devices[id].pos;
        while at != start {
            cells.insert(at);
            match prev.get(&at) {
                Some(&p) => at = p,
                None => break,
            }
        }
    }
    let mut out: Vec<Pos> = cells.into_iter().collect();
    out.sort_by_key(|p| (p.x, p.y, p.z));
    out
}

/// Simple cycles of the drive graph, bounded in count and length.
fn simple_cycles(adj: &[Vec<usize>], max_cycles: usize, max_len: usize) -> Vec<Vec<usize>> {
    let mut out: Vec<Vec<usize>> = Vec::new();
    let mut seen: HashSet<BTreeSet<usize>> = HashSet::new();
    for start in 0..adj.len() {
        let mut path = vec![start];
        let mut on_path: HashSet<usize> = HashSet::from([start]);
        walk(adj, start, start, &mut path, &mut on_path, &mut out, &mut seen, max_cycles, max_len);
        if out.len() >= max_cycles {
            break;
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn walk(
    adj: &[Vec<usize>],
    start: usize,
    at: usize,
    path: &mut Vec<usize>,
    on_path: &mut HashSet<usize>,
    out: &mut Vec<Vec<usize>>,
    seen: &mut HashSet<BTreeSet<usize>>,
    max_cycles: usize,
    max_len: usize,
) {
    if out.len() >= max_cycles || path.len() > max_len {
        return;
    }
    for &next in &adj[at] {
        if next == start {
            let key: BTreeSet<usize> = path.iter().copied().collect();
            if seen.insert(key) {
                out.push(path.clone());
                if out.len() >= max_cycles {
                    return;
                }
            }
        } else if next > start && !on_path.contains(&next) {
            // `next > start` keeps each cycle to one canonical rotation.
            path.push(next);
            on_path.insert(next);
            walk(adj, start, next, path, on_path, out, seen, max_cycles, max_len);
            on_path.remove(&next);
            path.pop();
        }
    }
}

/* ----------------------------------------------------------- rejections */

fn reject(graph: &mut MachineGraph, drive: &[Vec<usize>]) {
    let pistons: Vec<&Device> = graph
        .devices
        .iter()
        .filter(|d| matches!(d.kind, DeviceKind::Piston { .. }))
        .collect();

    // (1) Nothing can extend. With no piston able to move, no block moves; with
    // no block moving, no geometry changes; with no geometry change, no piston
    // that was blocked becomes free. The rest position is a fixed point.
    //
    // Withheld when a piston is captured already extended, because its *first*
    // stroke is a retraction, which this argument does not cover.
    let any_extended = pistons.iter().any(|d| d.extended);
    if !any_extended {
        if pistons.is_empty() {
            graph.rejections.push(Rejection {
                code: "no_piston",
                reason: "the build contains no piston, so no block can ever move".into(),
                unconditional: true,
            });
        } else if pistons.iter().all(|d| !d.can_extend) {
            graph.rejections.push(Rejection {
                code: "all_pistons_blocked",
                reason: format!(
                    "resolve_push refuses all {} piston(s) at rest; with nothing able to \
                     move, the geometry that blocks them can never change",
                    pistons.len()
                ),
                unconditional: true,
            });
        }
    }

    // (2) The build carries nothing that can re-fire a piston.
    //
    // NOT unconditional, and the reason is worth stating: a search that starts
    // its machines with an external kick has *lent* the build a power source for
    // a few ticks. A piston with no observer of its own still extends once under
    // that kick. What it cannot do is fire again once the kick is withdrawn, so
    // this disproves sustained motion and nothing stronger.
    let has_driver = graph
        .devices
        .iter()
        .any(|d| matches!(d.kind, DeviceKind::Observer | DeviceKind::Source));
    if !has_driver && !any_extended && !pistons.is_empty() {
        graph.rejections.push(Rejection {
            code: "no_driver",
            reason: "no observer and no power source: nothing inside the build can fire a \
                     piston twice"
                .into(),
            unconditional: false,
        });
    }

    // (3) The drive never moves itself. Every piston's resolved influence — both
    // strokes — is checked against the device positions. If no piston can shift
    // any piston or observer, the drive assembly is nailed to the world and
    // whatever it shoves is cargo. That is a cannon, not a flying machine.
    let device_cells: HashSet<Pos> = graph.devices.iter().map(|d| d.pos).collect();
    let any_can_extend = pistons.iter().any(|d| d.can_extend);
    if any_can_extend && !any_extended {
        let moves_a_device = pistons.iter().any(|p| {
            p.push.iter().chain(p.pull.iter()).any(|c| device_cells.contains(c))
        });
        if !moves_a_device {
            graph.rejections.push(Rejection {
                code: "drive_never_moves",
                reason: "no piston's push or pull set contains any piston or observer: the \
                         drive assembly cannot translate, only its cargo can"
                    .into(),
                // NOT unconditional. The drive staying put does not stop a
                // piston head from appearing, and a centre-of-mass score reads
                // that one new block as displacement. It does stop the machine
                // from ever going anywhere as a machine.
                unconditional: false,
            });
        }
    }

    // (4) No cycle in the drive graph. Nothing re-triggers anything, so the
    // machine fires a bounded number of times and stops.
    //
    // NOT unconditional. A finite burst still displaces a build by a block or
    // two, so this disproves *sustained* flight only. Callers whose success test
    // is a single lurch must ignore it.
    if !drive.is_empty() && simple_cycles(drive, 1, MAX_CYCLE_LEN).is_empty() {
        graph.rejections.push(Rejection {
            code: "acyclic_drive",
            reason: "the drive graph has no cycle: nothing re-triggers, so the machine fires \
                     a bounded number of times and stops"
                .into(),
            unconditional: false,
        });
    }
}

/* ---------------------------------------------------------------- json */

fn pos_json(p: Pos) -> String {
    format!("[{},{},{}]", p.x, p.y, p.z)
}

fn cells_json(cells: &[Pos]) -> String {
    let body: Vec<String> = cells.iter().map(|&p| pos_json(p)).collect();
    format!("[{}]", body.join(","))
}

fn dir_name(d: Dir) -> &'static str {
    match d {
        Dir::Down => "down",
        Dir::Up => "up",
        Dir::North => "north",
        Dir::South => "south",
        Dir::West => "west",
        Dir::East => "east",
    }
}

impl MachineGraph {
    /// The whole graph as JSON, which is how it crosses the bridge.
    pub fn to_json(&self) -> String {
        let mut out = String::from("{\"groups\":[");
        for (i, g) in self.groups.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!("{{\"id\":{},\"cells\":{}}}", g.id, cells_json(&g.cells)));
        }
        out.push_str("],\"devices\":[");
        for (i, d) in self.devices.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let kind = match d.kind {
                DeviceKind::Piston { sticky: true } => "sticky_piston",
                DeviceKind::Piston { sticky: false } => "piston",
                DeviceKind::Observer => "observer",
                DeviceKind::Source => "source",
            };
            let influence: Vec<Pos> = d.influence.iter().copied().collect();
            out.push_str(&format!(
                "{{\"id\":{},\"pos\":{},\"kind\":\"{}\",\"facing\":\"{}\",\"group\":{},\
                 \"extended\":{},\"can_extend\":{},\"push\":{},\"pull\":{},\"influence\":{}}}",
                d.id,
                pos_json(d.pos),
                kind,
                dir_name(d.facing),
                d.group,
                d.extended,
                d.can_extend,
                cells_json(&d.push),
                cells_json(&d.pull),
                cells_json(&influence),
            ));
        }
        out.push_str("],\"edges\":[");
        for (i, e) in self.edges.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"kind\":\"{}\",\"from\":\"{}\",\"to\":\"{}\"}}",
                e.kind.name(),
                e.from.label(),
                e.to.label()
            ));
        }
        out.push_str("],\"engines\":[");
        for (i, e) in self.engines.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let ids: Vec<String> = e.devices.iter().map(usize::to_string).collect();
            out.push_str(&format!(
                "{{\"devices\":[{}],\"cells\":{}}}",
                ids.join(","),
                cells_json(&e.cells)
            ));
        }
        out.push_str("],\"payload\":");
        out.push_str(&cells_json(&self.payload));
        out.push_str(",\"kickers\":[");
        let ks: Vec<String> = self.kickers.iter().map(usize::to_string).collect();
        out.push_str(&ks.join(","));
        out.push_str("],\"dead_weight\":");
        out.push_str(&cells_json(&self.dead_weight));
        out.push_str(",\"rejections\":[");
        for (i, r) in self.rejections.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"code\":\"{}\",\"unconditional\":{},\"reason\":\"{}\"}}",
                r.code,
                r.unconditional,
                r.reason.replace('"', "'")
            ));
        }
        out.push_str(&format!(
            "],\"rejected\":{},\"rejected_for_sustained\":{},\"engine_search_truncated\":{}}}",
            self.rejected(),
            self.rejected_for_sustained(),
            self.engine_search_truncated
        ));
        out
    }
}
