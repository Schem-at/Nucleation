//! Weisfeiler-Lehman fingerprint for [`RedstoneGraph`] (Phase 3 of the
//! redstone-graph effort).
//!
//! This mirrors the voxel [`crate::fingerprint::Fingerprint`] API shape: a
//! 128-bit value with `to_hex()`, and a spec resolved `from_preset(name)`. It
//! reuses the SAME stable hashing primitive as the voxel fingerprint —
//! `blake3::hash`, folded to a `u128` via the low 16 bytes — so results are
//! stable across runs and machines (no `DefaultHasher`/`RandomState`).
//!
//! The algorithm is a directed Weisfeiler-Lehman colour refinement. Node
//! positions (`pos`) and node ids/indices are NEVER hashed, which is what makes
//! the fingerprint invariant to translation, rotation, layout, and node
//! ordering. Two graphs that are the "same shape of logic" (or same logic, or
//! same compiled circuit, depending on the preset) share a fingerprint.
//!
//! VF2 exact-isomorphism confirmation is intentionally deferred; the WL hash is
//! the deliverable here. As with any WL scheme, a shared hash is strong
//! (collision-resistant) evidence of isomorphism but not a proof.

use super::graph::{ComparatorMode, LinkKind, RedstoneGraph, RedstoneNode, RedstoneNodeKind};

/// 128-bit Weisfeiler-Lehman fingerprint of a redstone graph.
///
/// Mirrors [`crate::fingerprint::Fingerprint`]: `to_hex()` yields a lowercase
/// 32-char hex string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RedstoneFingerprint(pub u128);

impl std::fmt::Display for RedstoneFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl RedstoneFingerprint {
    /// Lowercase 32-char hex of the 128-bit fingerprint.
    pub fn to_hex(&self) -> String {
        format!("{:032x}", self.0)
    }
}

/// Which node/edge features are folded into the WL colours, and how many
/// refinement rounds to run.
///
/// `pos` is never included under any spec — that omission is the entire point
/// (translation / rotation / layout invariance).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphFingerprintSpec {
    /// Number of WL refinement rounds. Default 3.
    pub iterations: u32,
    /// Fold repeater `delay` into the node colour.
    pub include_delay: bool,
    /// Fold comparator `mode` + `far_input` into the node colour.
    pub include_comparator_mode: bool,
    /// Fold `facing_diode` into the node colour.
    pub include_facing: bool,
    /// Fold link `strength` into each edge colour.
    pub include_link_strength: bool,
    /// Fold dynamic state (`powered`, `repeater_locked`, `output_strength`)
    /// into the node colour.
    pub include_state: bool,
}

impl GraphFingerprintSpec {
    /// "Same shape of logic": node-kind discriminant + link-kind only.
    pub fn structural() -> Self {
        Self {
            iterations: 3,
            include_delay: false,
            include_comparator_mode: false,
            include_facing: false,
            include_link_strength: false,
            include_state: false,
        }
    }

    /// "Same logic & timing": structural + repeater delay + comparator mode.
    pub fn functional() -> Self {
        Self {
            iterations: 3,
            include_delay: true,
            include_comparator_mode: true,
            include_facing: false,
            include_link_strength: false,
            include_state: false,
        }
    }

    /// "Same compiled circuit": everything ON except positions.
    pub fn exact() -> Self {
        Self {
            iterations: 3,
            include_delay: true,
            include_comparator_mode: true,
            include_facing: true,
            include_link_strength: true,
            include_state: true,
        }
    }

    /// Canonical preset names.
    pub const PRESETS: &'static [&'static str] = &["structural", "functional", "exact"];

    /// Resolve a preset name to a spec. Returns `None` for unknown names.
    pub fn from_preset(name: &str) -> Option<Self> {
        Some(match name {
            "structural" => Self::structural(),
            "functional" => Self::functional(),
            "exact" => Self::exact(),
            _ => return None,
        })
    }
}

/// Stable 128-bit fold of an arbitrary byte buffer, identical in spirit to the
/// voxel fingerprint: blake3 digest, low 16 bytes as a little-endian `u128`.
fn hash128(bytes: &[u8]) -> u128 {
    let h = blake3::hash(bytes);
    let mut buf = [0u8; 16];
    buf.copy_from_slice(&h.as_bytes()[..16]);
    u128::from_le_bytes(buf)
}

/// Discriminant byte for a node kind. Stable, never derived from `pos`/`id`.
fn kind_discriminant(kind: &RedstoneNodeKind) -> u8 {
    match kind {
        RedstoneNodeKind::Repeater { .. } => 0,
        RedstoneNodeKind::Comparator { .. } => 1,
        RedstoneNodeKind::Torch => 2,
        RedstoneNodeKind::Lamp => 3,
        RedstoneNodeKind::Button => 4,
        RedstoneNodeKind::Lever => 5,
        RedstoneNodeKind::PressurePlate => 6,
        RedstoneNodeKind::Trapdoor => 7,
        RedstoneNodeKind::Wire => 8,
        RedstoneNodeKind::Constant => 9,
        RedstoneNodeKind::NoteBlock => 10,
    }
}

/// Initial WL colour for a node from its masked features.
fn initial_label(node: &RedstoneNode, spec: &GraphFingerprintSpec) -> u128 {
    let mut buf: Vec<u8> = Vec::with_capacity(16);
    buf.push(0xA0); // domain tag: "initial node label"
    buf.push(kind_discriminant(&node.kind));

    match &node.kind {
        RedstoneNodeKind::Repeater { delay } => {
            if spec.include_delay {
                buf.push(0xD0);
                buf.push(*delay);
            }
        }
        RedstoneNodeKind::Comparator { mode, far_input } => {
            if spec.include_comparator_mode {
                buf.push(0xC0);
                buf.push(match mode {
                    ComparatorMode::Compare => 0,
                    ComparatorMode::Subtract => 1,
                });
                // far_input: presence flag + value (0 when absent).
                buf.push(far_input.is_some() as u8);
                buf.push(far_input.unwrap_or(0));
            }
        }
        _ => {}
    }

    if spec.include_facing {
        buf.push(0xF0);
        buf.push(node.facing_diode as u8);
    }

    if spec.include_state {
        buf.push(0x50);
        buf.push(node.powered as u8);
        buf.push(node.repeater_locked as u8);
        buf.push(node.output_strength);
    }

    hash128(&buf)
}

/// Colour for a single edge (link kind, and strength if enabled).
fn link_label(kind: LinkKind, strength: u8, spec: &GraphFingerprintSpec) -> u8 {
    // Pack into a single discriminant byte: 2 bits kind, optional strength tag.
    // We keep it a byte for compact, deterministic serialization; strength is
    // appended separately by the caller when enabled.
    let _ = (strength, spec);
    match kind {
        LinkKind::Default => 0,
        LinkKind::Side => 1,
    }
}

/// Direction tags for the WL neighbour multiset.
const DIR_IN: u8 = 0x01;
const DIR_OUT: u8 = 0x02;

impl RedstoneGraph {
    /// Weisfeiler-Lehman fingerprint of this graph under `spec`.
    ///
    /// Order-independent and position-independent: shuffling `nodes` (with
    /// consistent re-indexing) or translating the build yields the same value.
    pub fn fingerprint(&self, spec: &GraphFingerprintSpec) -> RedstoneFingerprint {
        let n = self.nodes.len();

        // Precompute out-adjacency once: out_edges[from] = Vec<(to, kind, strength)>.
        // `inputs` holds incoming edges (from -> this), so we invert them.
        let mut out_edges: Vec<Vec<(usize, LinkKind, u8)>> = vec![Vec::new(); n];
        for node in &self.nodes {
            let to = node.id;
            for link in &node.inputs {
                if link.from < n {
                    out_edges[link.from].push((to, link.kind, link.strength));
                }
            }
        }

        // Round 0 colours.
        let mut labels: Vec<u128> = self
            .nodes
            .iter()
            .map(|node| initial_label(node, spec))
            .collect();

        // WL refinement rounds.
        for _ in 0..spec.iterations {
            let mut next: Vec<u128> = Vec::with_capacity(n);
            for node in &self.nodes {
                let idx = node.id;

                // Multiset of (direction, link_label[, strength], neighbour_label).
                let mut neigh: Vec<Vec<u8>> = Vec::new();

                // Incoming edges: (IN, link, neighbour=link.from).
                for link in &node.inputs {
                    if link.from >= n {
                        continue;
                    }
                    neigh.push(encode_edge(
                        DIR_IN,
                        link.kind,
                        link.strength,
                        labels[link.from],
                        spec,
                    ));
                }
                // Outgoing edges: (OUT, link, neighbour=to).
                for &(to, kind, strength) in &out_edges[idx] {
                    neigh.push(encode_edge(DIR_OUT, kind, strength, labels[to], spec));
                }

                // Sort the multiset → canonical / node-ordering independent.
                neigh.sort_unstable();

                let mut buf: Vec<u8> = Vec::with_capacity(17 + neigh.len() * 18);
                buf.push(0xB0); // domain tag: "refined node label"
                buf.extend_from_slice(&labels[idx].to_le_bytes());
                buf.extend_from_slice(&(neigh.len() as u32).to_le_bytes());
                for e in &neigh {
                    buf.extend_from_slice(&(e.len() as u32).to_le_bytes());
                    buf.extend_from_slice(e);
                }
                next.push(hash128(&buf));
            }
            labels = next;
        }

        // Graph fingerprint = hash(node_count, edge_count, sorted final labels).
        let edge_count: usize = self.nodes.iter().map(|nd| nd.inputs.len()).sum();
        let mut final_labels = labels;
        final_labels.sort_unstable();

        let mut buf: Vec<u8> = Vec::with_capacity(16 + final_labels.len() * 16);
        buf.push(0xC1); // domain tag: "graph fingerprint"
        buf.extend_from_slice(&(n as u64).to_le_bytes());
        buf.extend_from_slice(&(edge_count as u64).to_le_bytes());
        for l in &final_labels {
            buf.extend_from_slice(&l.to_le_bytes());
        }

        RedstoneFingerprint(hash128(&buf))
    }

    /// Convenience: equal `structural` fingerprints.
    pub fn is_structurally_equal(&self, other: &RedstoneGraph) -> bool {
        let spec = GraphFingerprintSpec::structural();
        self.fingerprint(&spec) == other.fingerprint(&spec)
    }
}

/// Serialize one neighbour-edge tuple into a comparable, deterministic byte
/// vector. Includes strength bytes only when the spec enables them.
fn encode_edge(
    dir: u8,
    kind: LinkKind,
    strength: u8,
    neighbour_label: u128,
    spec: &GraphFingerprintSpec,
) -> Vec<u8> {
    let mut e = Vec::with_capacity(19);
    e.push(dir);
    e.push(link_label(kind, strength, spec));
    if spec.include_link_strength {
        e.push(strength);
    }
    e.extend_from_slice(&neighbour_label.to_le_bytes());
    e
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::graph::{RedstoneLink, RedstoneNode, RedstoneNodeKind};
    use crate::simulation::MchprsWorld;
    use crate::{BlockState, UniversalSchematic};

    // ---- fixture builders ------------------------------------------------

    /// lever -> N wires -> lamp, with a configurable concrete platform offset.
    /// Same family as `graph::tests::create_simple_redstone_line`.
    fn redstone_line(len: u32, origin: (i32, i32, i32)) -> UniversalSchematic {
        let (ox, oy, oz) = origin;
        let mut schematic = UniversalSchematic::new("line".to_string());
        for x in 0..(len as i32 + 1) {
            schematic.set_block(
                ox + x,
                oy,
                oz,
                &BlockState::new("minecraft:gray_concrete".to_string()),
            );
        }
        for x in 1..(len as i32) {
            let mut wire = BlockState::new("minecraft:redstone_wire".to_string());
            wire.set_property("power", "0");
            wire.set_property("east", "side");
            wire.set_property("west", "side");
            wire.set_property("north", "none");
            wire.set_property("south", "none");
            schematic.set_block(ox + x, oy + 1, oz, &wire);
        }
        let mut lever = BlockState::new("minecraft:lever".to_string());
        lever.set_property("facing", "east");
        lever.set_property("powered", "false");
        lever.set_property("face", "floor");
        schematic.set_block(ox, oy + 1, oz, &lever);

        let mut lamp = BlockState::new("minecraft:redstone_lamp".to_string());
        lamp.set_property("lit", "false");
        schematic.set_block(ox + len as i32, oy + 1, oz, &lamp);
        schematic
    }

    fn graph_of(schem: UniversalSchematic) -> RedstoneGraph {
        MchprsWorld::new(schem)
            .expect("world")
            .export_graph()
            .expect("graph")
    }

    // ---- hand-built graphs ----------------------------------------------

    fn node(id: usize, kind: RedstoneNodeKind, inputs: Vec<RedstoneLink>) -> RedstoneNode {
        RedstoneNode {
            id,
            kind,
            pos: Some((id as i32, 0, 0)),
            facing_diode: false,
            powered: false,
            repeater_locked: false,
            output_strength: 0,
            aliased_blocks: Vec::new(),
            inputs,
        }
    }

    fn link(from: usize) -> RedstoneLink {
        RedstoneLink {
            from,
            kind: LinkKind::Default,
            strength: 0,
        }
    }

    /// lever(0) -> repeater(1, delay) -> lamp(2).
    fn repeater_chain(delay: u8) -> RedstoneGraph {
        RedstoneGraph {
            nodes: vec![
                node(0, RedstoneNodeKind::Lever, vec![]),
                node(1, RedstoneNodeKind::Repeater { delay }, vec![link(0)]),
                node(2, RedstoneNodeKind::Lamp, vec![link(1)]),
            ],
        }
    }

    // ---- tests -----------------------------------------------------------

    #[test]
    fn layout_invariance_structural() {
        // Same logic (lever->wire->lamp coalesces regardless of wire length and
        // world position) → identical structural fingerprint.
        let a = graph_of(redstone_line(14, (0, 0, 0)));
        let b = graph_of(redstone_line(10, (40, 8, -12)));
        let spec = GraphFingerprintSpec::structural();
        let fa = a.fingerprint(&spec);
        let fb = b.fingerprint(&spec);
        assert_eq!(
            fa,
            fb,
            "layout-invariant structural dup: {} vs {}",
            fa.to_hex(),
            fb.to_hex()
        );
        assert!(a.is_structurally_equal(&b));
        // Surface the actual hex for the report.
        println!("LAYOUT_INVARIANCE_HEX a={} b={}", fa.to_hex(), fb.to_hex());
    }

    #[test]
    fn different_circuits_differ() {
        // A line (lever -> lamp) vs a bare lever with no load: genuinely
        // different topology → different structural fingerprints.
        let line = graph_of(redstone_line(12, (0, 0, 0)));
        let solo = RedstoneGraph {
            nodes: vec![
                node(0, RedstoneNodeKind::Lever, vec![]),
                node(1, RedstoneNodeKind::Torch, vec![link(0)]),
                node(2, RedstoneNodeKind::Torch, vec![link(1)]),
            ],
        };
        let spec = GraphFingerprintSpec::structural();
        assert_ne!(line.fingerprint(&spec), solo.fingerprint(&spec));
    }

    #[test]
    fn mask_sensitivity_delay() {
        // Two graphs differing ONLY in a repeater delay.
        let a = repeater_chain(1);
        let b = repeater_chain(4);

        let structural = GraphFingerprintSpec::structural();
        assert_eq!(
            a.fingerprint(&structural),
            b.fingerprint(&structural),
            "delay must be invisible to structural"
        );

        let functional = GraphFingerprintSpec::functional();
        assert_ne!(
            a.fingerprint(&functional),
            b.fingerprint(&functional),
            "delay must change functional"
        );
    }

    #[test]
    fn determinism_and_order_independence() {
        let g = repeater_chain(2);
        let spec = GraphFingerprintSpec::exact();
        assert_eq!(g.fingerprint(&spec), g.fingerprint(&spec));

        // Shuffle node order with consistent re-indexing: new order [2,0,1].
        // old id -> new id: 0->1, 1->2, 2->0.
        let remap = |old: usize| match old {
            0 => 1,
            1 => 2,
            2 => 0,
            _ => unreachable!(),
        };
        let mut shuffled_nodes: Vec<RedstoneNode> = g
            .nodes
            .iter()
            .map(|nd| {
                let mut nn = nd.clone();
                nn.id = remap(nd.id);
                for l in &mut nn.inputs {
                    l.from = remap(l.from);
                }
                nn
            })
            .collect();
        // Place in the new physical order so the Vec ordering actually differs.
        shuffled_nodes.sort_by_key(|nd| nd.id);
        let shuffled = RedstoneGraph {
            nodes: shuffled_nodes,
        };

        assert_eq!(
            g.fingerprint(&spec),
            shuffled.fingerprint(&spec),
            "fingerprint must be independent of node Vec order"
        );
    }

    #[test]
    fn to_hex_format() {
        let g = repeater_chain(1);
        let hx = g.fingerprint(&GraphFingerprintSpec::structural()).to_hex();
        assert_eq!(hx.len(), 32);
        assert!(hx
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn preset_names_resolve() {
        for name in GraphFingerprintSpec::PRESETS {
            assert!(GraphFingerprintSpec::from_preset(name).is_some(), "{name}");
        }
        assert!(GraphFingerprintSpec::from_preset("nope").is_none());
        assert_eq!(
            GraphFingerprintSpec::from_preset("structural"),
            Some(GraphFingerprintSpec::structural())
        );
    }
}
