//! Physical contract sidecar for a schematic + IoLayout pair.
//!
//! [`PhysicalContract`] carries everything the router/placer needs to treat a
//! verified circuit as a black box: keepouts, boundary (edge) contract,
//! measured port-pair delays, drive strength, paste-safety, and the declared
//! initial state. [`CellContract`] bundles it with the IO layout and a
//! name/version — the serializable half of the CellTemplate concept (the
//! other half is the schematic file itself).

use super::io_layout_builder::IoLayout;
use crate::bounding_box::BoundingBox;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A cell face (axis-aligned boundary plane), Minecraft direction names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Face {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl Face {
    /// Parse from a lowercase direction name.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "down" => Some(Face::Down),
            "up" => Some(Face::Up),
            "north" => Some(Face::North),
            "south" => Some(Face::South),
            "west" => Some(Face::West),
            "east" => Some(Face::East),
            _ => None,
        }
    }

    /// Unit outward normal of this face.
    pub fn normal(&self) -> (i32, i32, i32) {
        match self {
            Face::Down => (0, -1, 0),
            Face::Up => (0, 1, 0),
            Face::North => (0, 0, -1),
            Face::South => (0, 0, 1),
            Face::West => (-1, 0, 0),
            Face::East => (1, 0, 0),
        }
    }

    /// The face that mates with this one when two cells abut.
    pub fn opposite(&self) -> Face {
        match self {
            Face::Down => Face::Up,
            Face::Up => Face::Down,
            Face::North => Face::South,
            Face::South => Face::North,
            Face::West => Face::East,
            Face::East => Face::West,
        }
    }
}

/// Direction of a port as seen from the cell (signal flows in or out).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortDirection {
    Input,
    Output,
}

impl PortDirection {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "input" | "in" => Some(PortDirection::Input),
            "output" | "out" => Some(PortDirection::Output),
            _ => None,
        }
    }
}

/// A window on one cell face through which foreign nets may cross the
/// boundary. Boxes are in cell-local coordinates and should lie on the
/// named face's boundary plane.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeWindow {
    pub face: Face,
    pub boxes: Vec<BoundingBox>,
}

/// Which boundary cells may carry nets (the seam contract).
///
/// An empty window list means the boundary is closed: nets may only enter or
/// leave through declared ports. This is the rule that prevented cross-cell
/// seam shorts.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EdgeContract {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub windows: Vec<EdgeWindow>,
}

/// Measured delay from one port to another, in redstone ticks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortPairDelay {
    pub from: String,
    pub to: String,
    pub delay_rt: u32,
}

/// Declared initial state of a cell.
///
/// Cells are BAKED at a chosen state and deployed trusting the saved block
/// states (like an FPGA bitstream carrying initial register values); reset
/// is a per-design option, not a correctness need.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum InitialState {
    /// No declaration: combinational or don't-care.
    #[default]
    Unspecified,
    /// The schematic was baked quiescent with these output-port values
    /// (verified to wake holding them under in-world settle).
    Baked {
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        port_values: BTreeMap<String, u64>,
    },
}

/// Physical sidecar for a schematic + [`IoLayout`] pair.
///
/// All coordinates are cell-local (same frame as the IoLayout positions).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PhysicalContract {
    /// Regions no foreign routing may enter.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keepouts: Vec<BoundingBox>,

    /// Which boundary cells may carry nets.
    #[serde(default)]
    pub edge_contract: EdgeContract,

    /// Measured per-port-pair delays in redstone ticks (characterization).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delays_rt: Vec<PortPairDelay>,

    /// Output signal strength under load, per output port name (0-15).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub drive_strength: BTreeMap<String, u8>,

    /// True iff the cell's declared state survives placement-mode loading
    /// (locked-flag re-derivation, observer pulses). False = in-world only.
    #[serde(default)]
    pub paste_safe: bool,

    /// Declared initial state carried by the baked schematic.
    #[serde(default)]
    pub initial_state: InitialState,
}

impl PhysicalContract {
    /// Look up the measured delay between two ports, if characterized.
    pub fn delay_rt(&self, from: &str, to: &str) -> Option<u32> {
        self.delays_rt
            .iter()
            .find(|d| d.from == from && d.to == to)
            .map(|d| d.delay_rt)
    }

    /// Record (or overwrite) a measured port-pair delay.
    pub fn set_delay_rt(&mut self, from: impl Into<String>, to: impl Into<String>, delay_rt: u32) {
        let (from, to) = (from.into(), to.into());
        if let Some(d) = self
            .delays_rt
            .iter_mut()
            .find(|d| d.from == from && d.to == to)
        {
            d.delay_rt = delay_rt;
        } else {
            self.delays_rt.push(PortPairDelay { from, to, delay_rt });
        }
    }
}

/// The serializable contract file of a cell library entry.
///
/// CellTemplate = a schematic file + this contract file. The contract names
/// the cell, embeds the IO layout (ports with types, positions, optional
/// face/direction, bus ports) and the physical sidecar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellContract {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub io: IoLayout,
    #[serde(default)]
    pub physical: PhysicalContract,
}

impl CellContract {
    pub fn new(name: impl Into<String>, io: IoLayout) -> Self {
        Self {
            name: name.into(),
            version: None,
            io,
            physical: PhysicalContract::default(),
        }
    }

    /// Serialize to pretty JSON (the contract-file format).
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }

    /// Parse from JSON.
    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{IoLayoutBuilder, IoType, LayoutFunction};
    use super::*;

    fn layout() -> IoLayout {
        IoLayoutBuilder::new()
            .add_input(
                "a",
                IoType::UnsignedInt { bits: 4 },
                LayoutFunction::OneToOne,
                vec![(0, 0, 0), (0, 0, 2), (0, 0, 4), (0, 0, 6)],
            )
            .unwrap()
            .add_output(
                "sum",
                IoType::UnsignedInt { bits: 4 },
                LayoutFunction::OneToOne,
                vec![(9, 0, 0), (9, 0, 2), (9, 0, 4), (9, 0, 6)],
            )
            .unwrap()
            .build()
    }

    #[test]
    fn face_normals_and_opposites() {
        assert_eq!(Face::East.normal(), (1, 0, 0));
        assert_eq!(Face::East.opposite(), Face::West);
        assert_eq!(Face::parse("NORTH"), Some(Face::North));
        assert_eq!(Face::parse("bogus"), None);
    }

    #[test]
    fn delay_table_set_and_get() {
        let mut pc = PhysicalContract::default();
        pc.set_delay_rt("a", "sum", 3);
        pc.set_delay_rt("a", "sum", 4); // overwrite
        assert_eq!(pc.delay_rt("a", "sum"), Some(4));
        assert_eq!(pc.delay_rt("sum", "a"), None);
    }

    #[test]
    fn cell_contract_json_round_trip() {
        let mut contract = CellContract::new("full_adder", layout());
        contract.version = Some("1.0.0".to_string());
        contract.physical.keepouts.push(BoundingBox {
            min: (0, 0, 0),
            max: (9, 3, 6),
        });
        contract.physical.edge_contract.windows.push(EdgeWindow {
            face: Face::East,
            boxes: vec![BoundingBox {
                min: (9, 0, 0),
                max: (9, 0, 6),
            }],
        });
        contract.physical.set_delay_rt("a", "sum", 2);
        contract.physical.drive_strength.insert("sum".into(), 15);
        contract.physical.paste_safe = true;
        contract.physical.initial_state = InitialState::Baked {
            port_values: [("sum".to_string(), 0u64)].into_iter().collect(),
        };

        let json = contract.to_json().unwrap();
        let back = CellContract::from_json(&json).unwrap();
        assert_eq!(back, contract);
        // the sidecar concept: contract file references ports by name
        assert!(back.io.get_input("a").is_some());
        assert_eq!(back.physical.delay_rt("a", "sum"), Some(2));
    }

    #[test]
    fn default_physical_contract_is_minimal_json() {
        let pc = PhysicalContract::default();
        let json = serde_json::to_string(&pc).unwrap();
        // empty collections are skipped; only structural defaults remain
        assert!(!json.contains("keepouts"));
        assert!(!json.contains("delays_rt"));
        let back: PhysicalContract = serde_json::from_str(&json).unwrap();
        assert_eq!(back, pc);
    }
}
