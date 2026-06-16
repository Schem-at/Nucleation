//! Redstone-graph extraction surface.
//!
//! This module exposes a Nucleation-owned, serde-serializable representation of
//! the redstone logic graph that the MCHPRS redpiler compiles from a schematic.
//! It mirrors `mchprs_redpiler::redpiler_graph::Node` but deliberately does NOT
//! leak mchprs types across the public API.
//!
//! The entry point is [`MchprsWorld::export_graph`], which runs the redpiler's
//! graph-compilation passes (without building a runnable backend) and lowers the
//! result into [`RedstoneGraph`].

use crate::simulation::MchprsWorld;
use mchprs_blocks::BlockPos;
use mchprs_redpiler::{
    redpiler_graph::{
        ComparatorMode as McComparatorMode, LinkType as McLinkType, Node as McNode,
        NodeType as McNodeType,
    },
    BackendVariant, Compiler, CompilerOptions,
};
use serde::{Deserialize, Serialize};

/// Comparator operating mode (mirror of `redpiler_graph::ComparatorMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparatorMode {
    Compare,
    Subtract,
}

/// The logical kind of a redstone node, with any kind-specific data inlined.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedstoneNodeKind {
    Repeater {
        delay: u8,
    },
    Comparator {
        mode: ComparatorMode,
        far_input: Option<u8>,
    },
    Torch,
    Lamp,
    Button,
    Lever,
    PressurePlate,
    Trapdoor,
    Wire,
    Constant,
    NoteBlock,
}

/// The kind of a link/edge between two nodes (mirror of `redpiler_graph::LinkType`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkKind {
    Default,
    Side,
}

/// An incoming edge: signal arrives at this node FROM node index `from`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedstoneLink {
    /// Index of the source node that feeds this node.
    pub from: usize,
    pub kind: LinkKind,
    /// Signal-strength loss along this link (redpiler "weight").
    pub strength: u8,
}

/// A single node in the extracted redstone graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedstoneNode {
    /// Index of this node within [`RedstoneGraph::nodes`].
    pub id: usize,
    pub kind: RedstoneNodeKind,
    /// World coordinates (de-normalized: mchprs pos + min_coords).
    /// `None` if the node has no backing block (e.g. synthesized constants).
    pub pos: Option<(i32, i32, i32)>,
    pub facing_diode: bool,
    pub powered: bool,
    pub repeater_locked: bool,
    pub output_strength: u8,
    /// World blocks coalesced into this node (de-normalized: mchprs pos +
    /// min_coords). Empty in the common case; non-empty when optimization
    /// passes merge multiple source blocks into one node.
    pub aliased_blocks: Vec<(i32, i32, i32)>,
    /// Incoming edges (sources feeding this node).
    pub inputs: Vec<RedstoneLink>,
}

/// The extracted redstone logic graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedstoneGraph {
    pub nodes: Vec<RedstoneNode>,
}

impl RedstoneGraph {
    /// Number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total number of edges (sum of every node's incoming-link count).
    pub fn edge_count(&self) -> usize {
        self.nodes.iter().map(|n| n.inputs.len()).sum()
    }

    /// Serialize the graph to JSON.
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    /// Deserialize a graph from JSON.
    pub fn from_json(s: &str) -> Result<Self, String> {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }

    /// The nodes as a JSON array of flat objects (kind-specific fields inlined).
    ///
    /// Shared by the WASM and FFI bindings so every language sees the same shape
    /// as the Python `nodes` property: each object has `id`, `kind`, `delay`
    /// (Repeater), `comparator_mode`/`comparator_far_input` (Comparator), `pos`,
    /// `aliased_blocks`, `powered`, `repeater_locked`, `output_strength`,
    /// `facing_diode`.
    pub fn nodes_json(&self) -> Result<String, String> {
        let views: Vec<NodeView<'_>> = self.nodes.iter().map(node_view).collect();
        serde_json::to_string(&views).map_err(|e| e.to_string())
    }

    /// The directed edges as a JSON array of `{source, target, kind, strength}`
    /// objects (each node's incoming links become `link.from -> node.id`).
    pub fn edges_json(&self) -> Result<String, String> {
        let mut views: Vec<EdgeView> = Vec::with_capacity(self.edge_count());
        for node in &self.nodes {
            for link in &node.inputs {
                views.push(EdgeView {
                    source: link.from,
                    target: node.id,
                    kind: link_kind_name(&link.kind),
                    strength: link.strength,
                });
            }
        }
        serde_json::to_string(&views).map_err(|e| e.to_string())
    }
}

fn node_kind_name(kind: &RedstoneNodeKind) -> &'static str {
    match kind {
        RedstoneNodeKind::Repeater { .. } => "Repeater",
        RedstoneNodeKind::Comparator { .. } => "Comparator",
        RedstoneNodeKind::Torch => "Torch",
        RedstoneNodeKind::Lamp => "Lamp",
        RedstoneNodeKind::Button => "Button",
        RedstoneNodeKind::Lever => "Lever",
        RedstoneNodeKind::PressurePlate => "PressurePlate",
        RedstoneNodeKind::Trapdoor => "Trapdoor",
        RedstoneNodeKind::Wire => "Wire",
        RedstoneNodeKind::Constant => "Constant",
        RedstoneNodeKind::NoteBlock => "NoteBlock",
    }
}

fn comparator_mode_name(mode: &ComparatorMode) -> &'static str {
    match mode {
        ComparatorMode::Compare => "Compare",
        ComparatorMode::Subtract => "Subtract",
    }
}

fn link_kind_name(kind: &LinkKind) -> &'static str {
    match kind {
        LinkKind::Default => "Default",
        LinkKind::Side => "Side",
    }
}

/// A flat, serde-serializable view of a node (kind-specific fields inlined).
/// Mirrors the dict shape produced by the Python `RedstoneGraph.nodes` getter.
#[derive(Serialize)]
struct NodeView<'a> {
    id: usize,
    kind: &'static str,
    delay: Option<u8>,
    comparator_mode: Option<&'static str>,
    comparator_far_input: Option<u8>,
    pos: Option<(i32, i32, i32)>,
    aliased_blocks: &'a [(i32, i32, i32)],
    powered: bool,
    repeater_locked: bool,
    output_strength: u8,
    facing_diode: bool,
}

/// A flat, serde-serializable view of a directed edge.
#[derive(Serialize)]
struct EdgeView {
    source: usize,
    target: usize,
    kind: &'static str,
    strength: u8,
}

fn node_view(node: &RedstoneNode) -> NodeView<'_> {
    let (delay, comparator_mode, comparator_far_input) = match &node.kind {
        RedstoneNodeKind::Repeater { delay } => (Some(*delay), None, None),
        RedstoneNodeKind::Comparator { mode, far_input } => {
            (None, Some(comparator_mode_name(mode)), *far_input)
        }
        _ => (None, None, None),
    };
    NodeView {
        id: node.id,
        kind: node_kind_name(&node.kind),
        delay,
        comparator_mode,
        comparator_far_input,
        pos: node.pos,
        aliased_blocks: &node.aliased_blocks,
        powered: node.powered,
        repeater_locked: node.repeater_locked,
        output_strength: node.output_strength,
        facing_diode: node.facing_diode,
    }
}

fn map_comparator_mode(mode: McComparatorMode) -> ComparatorMode {
    match mode {
        McComparatorMode::Compare => ComparatorMode::Compare,
        McComparatorMode::Subtract => ComparatorMode::Subtract,
    }
}

fn map_link_kind(ty: McLinkType) -> LinkKind {
    match ty {
        McLinkType::Default => LinkKind::Default,
        McLinkType::Side => LinkKind::Side,
    }
}

fn map_node_kind(node: &McNode) -> RedstoneNodeKind {
    match node.ty {
        McNodeType::Repeater(delay) => RedstoneNodeKind::Repeater { delay },
        McNodeType::Comparator(mode) => RedstoneNodeKind::Comparator {
            mode: map_comparator_mode(mode),
            far_input: node.comparator_far_input,
        },
        McNodeType::Torch => RedstoneNodeKind::Torch,
        McNodeType::Lamp => RedstoneNodeKind::Lamp,
        McNodeType::Button => RedstoneNodeKind::Button,
        McNodeType::Lever => RedstoneNodeKind::Lever,
        McNodeType::PressurePlate => RedstoneNodeKind::PressurePlate,
        McNodeType::Trapdoor => RedstoneNodeKind::Trapdoor,
        McNodeType::Wire => RedstoneNodeKind::Wire,
        McNodeType::Constant => RedstoneNodeKind::Constant,
        McNodeType::NoteBlock => RedstoneNodeKind::NoteBlock,
    }
}

impl MchprsWorld {
    /// Builds the (bounds, options) pair used to drive the redpiler, replicating
    /// the inputs that [`MchprsWorld::initialize_compiler`] constructs.
    pub(crate) fn build_compile_inputs(&self) -> ((BlockPos, BlockPos), CompilerOptions) {
        let bounding_box = self.schematic.get_bounding_box();
        let bounds = (
            BlockPos::new(0, 0, 0),
            BlockPos::new(
                bounding_box.max.0 - self.min_coords.0,
                bounding_box.max.1 - self.min_coords.1,
                bounding_box.max.2 - self.min_coords.2,
            ),
        );

        let normalized_custom_io: Vec<BlockPos> = self
            .options
            .custom_io
            .iter()
            .map(|&pos| self.normalize_pos(pos))
            .collect();

        let compiler_options = CompilerOptions {
            optimize: self.options.optimize,
            io_only: self.options.io_only,
            wire_dot_out: true,
            backend_variant: BackendVariant::Direct,
            custom_io: normalized_custom_io,
            ..Default::default()
        };

        (bounds, compiler_options)
    }

    /// Extracts the redstone logic graph for this world.
    ///
    /// Runs the redpiler's compilation passes against the current world state and
    /// lowers the result into a Nucleation-owned [`RedstoneGraph`]. Node positions
    /// are de-normalized back into schematic/world coordinates.
    ///
    /// # Errors
    /// Returns an error string if graph compilation panics.
    pub fn export_graph(&self) -> Result<RedstoneGraph, String> {
        let (bounds, compiler_options) = self.build_compile_inputs();
        let monitor = Default::default();

        let nodes: Vec<McNode> =
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Compiler::compile_graph(self, bounds, compiler_options, monitor)
            })) {
                Ok(nodes) => nodes,
                Err(e) => return Err(downcast_panic(e)),
            };

        Ok(self.lower_nodes(nodes))
    }

    /// Extracts the *structural* (as-built) redstone graph for this world.
    ///
    /// Unlike [`MchprsWorld::export_graph`], this runs only the redpiler's
    /// pre-fold passes (IdentifyNodes → InputSearch → ClampWeights → DedupLinks);
    /// it skips ConstantFold and Coalesce. Combined with `optimize: false`, this
    /// keeps redstone *wires* as individual nodes and preserves repeaters/torches
    /// that the optimizing pipeline would otherwise fold away — recovering a graph
    /// that matches the physically-placed components much more closely.
    ///
    /// # Errors
    /// Returns an error string if graph compilation panics.
    pub fn export_graph_structural(&self) -> Result<RedstoneGraph, String> {
        let (bounds, mut compiler_options) = self.build_compile_inputs();
        // Force optimization off: keeps wires as nodes and avoids wire-stripping.
        compiler_options.optimize = false;
        let monitor = Default::default();

        let nodes: Vec<McNode> =
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Compiler::compile_graph_structural(self, bounds, compiler_options, monitor)
            })) {
                Ok(nodes) => nodes,
                Err(e) => return Err(downcast_panic(e)),
            };

        Ok(self.lower_nodes(nodes))
    }

    /// Lowers a list of redpiler nodes into a Nucleation-owned [`RedstoneGraph`],
    /// de-normalizing positions and aliased blocks back into world coordinates.
    fn lower_nodes(&self, nodes: Vec<McNode>) -> RedstoneGraph {
        let (min_x, min_y, min_z) = self.min_coords;
        let nodes = nodes
            .into_iter()
            .enumerate()
            .map(|(id, node)| {
                let kind = map_node_kind(&node);
                let pos = node
                    .block
                    .map(|(p, _protocol_id)| (p.x + min_x, p.y + min_y, p.z + min_z));
                let aliased_blocks = node
                    .aliased_blocks
                    .iter()
                    .map(|(p, _protocol_id)| (p.x + min_x, p.y + min_y, p.z + min_z))
                    .collect();
                let inputs = node
                    .inputs
                    .iter()
                    .map(|link| RedstoneLink {
                        from: link.to,
                        kind: map_link_kind(link.ty),
                        strength: link.weight,
                    })
                    .collect();
                RedstoneNode {
                    id,
                    kind,
                    pos,
                    facing_diode: node.facing_diode,
                    powered: node.state.powered,
                    repeater_locked: node.state.repeater_locked,
                    output_strength: node.state.output_strength,
                    aliased_blocks,
                    inputs,
                }
            })
            .collect();

        RedstoneGraph { nodes }
    }
}

/// Extracts a human-readable message from a `catch_unwind` panic payload.
fn downcast_panic(e: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = e.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = e.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown graph extraction error".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlockState, UniversalSchematic};

    /// Lever -> redstone wire -> redstone lamp. This is the same known-good
    /// fixture used by `simulation::tests::create_simple_redstone_line`, which
    /// is already exercised by the existing simulation test-suite, so we know it
    /// compiles cleanly through the redpiler.
    fn create_simple_redstone_line() -> UniversalSchematic {
        let mut schematic = UniversalSchematic::new("Simple Redstone Line".to_string());

        for x in 0..16 {
            schematic.set_block(
                x,
                0,
                0,
                &BlockState::new("minecraft:gray_concrete".to_string()),
            );
        }

        for x in 1..15 {
            let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
            wire.set_property("power", "0");
            wire.set_property("east", "side");
            wire.set_property("west", "side");
            wire.set_property("north", "none");
            wire.set_property("south", "none");
            schematic.set_block(x, 1, 0, &wire);
        }

        let mut lever = BlockState::new("minecraft:lever".to_string());
        lever.set_property("facing", "east");
        lever.set_property("powered", "false");
        lever.set_property("face", "floor");
        schematic.set_block(0, 1, 0, &lever);

        let mut lamp = BlockState::new("minecraft:redstone_lamp".to_string());
        lamp.set_property("lit", "false");
        schematic.set_block(15, 1, 0, &lamp);

        schematic
    }

    #[test]
    fn test_export_graph_simple_line() {
        let schematic = create_simple_redstone_line();
        let world = MchprsWorld::new(schematic).expect("world creation should succeed");

        let graph = world
            .export_graph()
            .expect("graph extraction should succeed");

        // Topology may be coalesced by optimization passes; assert on the kinds
        // that must survive, not exact counts.
        assert!(
            graph.node_count() >= 2,
            "expected at least 2 nodes, got {}",
            graph.node_count()
        );

        let has_lever = graph
            .nodes
            .iter()
            .any(|n| matches!(n.kind, RedstoneNodeKind::Lever));
        let has_lamp = graph
            .nodes
            .iter()
            .any(|n| matches!(n.kind, RedstoneNodeKind::Lamp));
        assert!(has_lever, "expected a Lever node in the graph");
        assert!(has_lamp, "expected a Lamp node in the graph");

        // Adjacency must be captured: at least one node has an incoming edge.
        assert!(
            graph.edge_count() >= 1,
            "expected at least one edge in the graph"
        );

        // Every link's `from` must reference a valid node index.
        for node in &graph.nodes {
            for link in &node.inputs {
                assert!(
                    link.from < graph.node_count(),
                    "link.from {} out of range (node_count {})",
                    link.from,
                    graph.node_count()
                );
            }
        }
    }

    /// The shared `nodes_json`/`edges_json` helpers (used by the WASM and FFI
    /// bindings) must produce the flat object shape the Python `nodes`/`edges`
    /// getters expose: an array whose length matches `node_count`/`edge_count`,
    /// with `id`/`kind` on every node and `source`/`target`/`kind`/`strength`
    /// on every edge.
    #[test]
    fn test_nodes_edges_json_shape() {
        let schematic = create_simple_redstone_line();
        let world = MchprsWorld::new(schematic).expect("world creation should succeed");
        let graph = world
            .export_graph()
            .expect("graph extraction should succeed");

        let nodes: serde_json::Value =
            serde_json::from_str(&graph.nodes_json().expect("nodes_json")).unwrap();
        let edges: serde_json::Value =
            serde_json::from_str(&graph.edges_json().expect("edges_json")).unwrap();

        let nodes = nodes.as_array().expect("nodes is a JSON array");
        let edges = edges.as_array().expect("edges is a JSON array");
        assert_eq!(nodes.len(), graph.node_count(), "nodes_json length");
        assert_eq!(edges.len(), graph.edge_count(), "edges_json length");

        for n in nodes {
            assert!(n.get("id").is_some(), "node has id");
            assert!(
                n.get("kind").and_then(|k| k.as_str()).is_some(),
                "node has kind"
            );
        }
        for e in edges {
            for key in ["source", "target", "kind", "strength"] {
                assert!(e.get(key).is_some(), "edge has {key}");
            }
        }
    }

    #[test]
    fn test_export_graph_deterministic() {
        let schematic = create_simple_redstone_line();
        let world = MchprsWorld::new(schematic).expect("world creation should succeed");

        let g1 = world
            .export_graph()
            .expect("first extraction should succeed");
        let g2 = world
            .export_graph()
            .expect("second extraction should succeed");

        assert_eq!(g1, g2, "export_graph should be deterministic");
    }

    #[test]
    fn test_export_graph_json_round_trip() {
        let schematic = create_simple_redstone_line();
        let world = MchprsWorld::new(schematic).expect("world creation should succeed");

        let graph = world
            .export_graph()
            .expect("graph extraction should succeed");

        let json = graph.to_json().expect("serialization should succeed");
        let restored = RedstoneGraph::from_json(&json).expect("deserialization should succeed");

        assert_eq!(graph, restored, "JSON round-trip should preserve the graph");
    }
}
