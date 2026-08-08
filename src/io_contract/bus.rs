//! Bus as a first-class concept.
//!
//! A bus is not just N ports — it is a geometry + ordering + timing contract.
//! Two [`BusPort`]s mate iff their [`BusSpec`]s are compatible
//! (width/pitch/face/encoding), which makes abutment compatibility checkable
//! at placement time instead of living in someone's head.

use super::io_type::IoType;
use super::layout_function::LayoutFunction;
use super::physical::{Face, PortDirection};
use crate::transforms::Axis;
use serde::{Deserialize, Serialize};

/// How a logical word is encoded onto the bus wires.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusEncoding {
    /// One binary bit per wire (signal 0 or 15).
    Binary1PerWire,
    /// One hex digit (4 bits) per wire as analog signal strength 0-15.
    HexAnalog,
}

impl BusEncoding {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "binary" | "binary1perwire" | "binary_1_per_wire" | "1perwire" => {
                Some(BusEncoding::Binary1PerWire)
            }
            "hex" | "hexanalog" | "hex_analog" | "analog" => Some(BusEncoding::HexAnalog),
            _ => None,
        }
    }

    /// Bits carried per wire.
    pub fn bits_per_wire(&self) -> usize {
        match self {
            BusEncoding::Binary1PerWire => 1,
            BusEncoding::HexAnalog => 4,
        }
    }

    /// The [`LayoutFunction`] that realizes this encoding on the executor.
    pub fn layout_function(&self) -> LayoutFunction {
        match self {
            BusEncoding::Binary1PerWire => LayoutFunction::OneToOne,
            BusEncoding::HexAnalog => LayoutFunction::Packed4,
        }
    }
}

/// Wire-to-wire geometry: which axis successive bits advance along and by
/// how many blocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pitch {
    pub axis: Axis,
    /// Blocks between successive wires (signed: bits may run in -axis).
    pub spacing: i32,
}

impl Pitch {
    /// Offset of wire `i` relative to wire 0.
    pub fn offset(&self, i: i32) -> (i32, i32, i32) {
        let d = self.spacing * i;
        match self.axis {
            Axis::X => (d, 0, 0),
            Axis::Y => (0, d, 0),
            Axis::Z => (0, 0, d),
        }
    }
}

/// Geometry + ordering + timing contract of a bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusSpec {
    /// Logical width in bits.
    pub width: u8,
    /// Semantic type of the word carried (reuses the executor's type system).
    pub ty: IoType,
    /// Axis + spacing between successive wires.
    pub pitch: Pitch,
    /// Which cell face the bus presents on.
    pub face: Face,
    /// Wire encoding.
    pub encoding: BusEncoding,
}

impl BusSpec {
    /// Number of physical wires this bus occupies.
    pub fn wire_count(&self) -> usize {
        let bpw = self.encoding.bits_per_wire();
        (self.width as usize).div_ceil(bpw)
    }

    /// Wire positions given the bit-0 wire position (bit order follows pitch).
    pub fn wire_positions(&self, bit0: (i32, i32, i32)) -> Vec<(i32, i32, i32)> {
        (0..self.wire_count() as i32)
            .map(|i| {
                let (dx, dy, dz) = self.pitch.offset(i);
                (bit0.0 + dx, bit0.1 + dy, bit0.2 + dz)
            })
            .collect()
    }

    /// Consistency: the semantic type must fit the declared width.
    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 {
            return Err("Bus width must be at least 1".to_string());
        }
        let bits = self.ty.bit_count();
        if bits != self.width as usize {
            return Err(format!(
                "Bus width {} does not match type bit count {}",
                self.width, bits
            ));
        }
        if self.pitch.spacing == 0 {
            return Err("Bus pitch spacing must be non-zero".to_string());
        }
        Ok(())
    }

    /// Two bus specs mate across an abutment seam iff width, pitch magnitude,
    /// encoding match and the faces are opposite.
    pub fn mates_with(&self, other: &BusSpec) -> bool {
        self.width == other.width
            && self.encoding == other.encoding
            && self.pitch.axis == other.pitch.axis
            && self.pitch.spacing.abs() == other.pitch.spacing.abs()
            && self.face == other.face.opposite()
    }
}

/// A concrete bus attachment point on a cell: the spec plus where wire 0
/// sits and which way the word flows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusPort {
    pub spec: BusSpec,
    /// Position of the bit-0 wire (cell-local coordinates).
    pub bit0: (i32, i32, i32),
    pub direction: PortDirection,
}

impl BusPort {
    /// All wire positions of this bus port.
    pub fn wire_positions(&self) -> Vec<(i32, i32, i32)> {
        self.spec.wire_positions(self.bit0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(width: u8, encoding: BusEncoding) -> BusSpec {
        BusSpec {
            width,
            ty: IoType::UnsignedInt {
                bits: width as usize,
            },
            pitch: Pitch {
                axis: Axis::Z,
                spacing: 2,
            },
            face: Face::East,
            encoding,
        }
    }

    #[test]
    fn wire_counts_per_encoding() {
        assert_eq!(spec(8, BusEncoding::Binary1PerWire).wire_count(), 8);
        assert_eq!(spec(8, BusEncoding::HexAnalog).wire_count(), 2);
        assert_eq!(spec(6, BusEncoding::HexAnalog).wire_count(), 2); // ceil
    }

    #[test]
    fn wire_positions_follow_pitch() {
        let s = spec(4, BusEncoding::Binary1PerWire);
        assert_eq!(
            s.wire_positions((10, 0, 0)),
            vec![(10, 0, 0), (10, 0, 2), (10, 0, 4), (10, 0, 6)]
        );
    }

    #[test]
    fn validation_catches_width_mismatch() {
        let mut s = spec(8, BusEncoding::Binary1PerWire);
        assert!(s.validate().is_ok());
        s.ty = IoType::UnsignedInt { bits: 4 };
        assert!(s.validate().is_err());
        s = spec(8, BusEncoding::Binary1PerWire);
        s.pitch.spacing = 0;
        assert!(s.validate().is_err());
    }

    #[test]
    fn abutment_mating() {
        let a = spec(8, BusEncoding::Binary1PerWire);
        let mut b = spec(8, BusEncoding::Binary1PerWire);
        b.face = Face::West; // opposite of East
        assert!(a.mates_with(&b));
        b.width = 4;
        assert!(!a.mates_with(&b));
        b.width = 8;
        b.encoding = BusEncoding::HexAnalog;
        assert!(!a.mates_with(&b));
        b.encoding = BusEncoding::Binary1PerWire;
        b.face = Face::East; // same face: no mate
        assert!(!a.mates_with(&b));
    }

    #[test]
    fn bus_spec_json_round_trip() {
        let s = spec(8, BusEncoding::HexAnalog);
        let json = serde_json::to_string(&s).unwrap();
        let back: BusSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
        let port = BusPort {
            spec: s,
            bit0: (3, 1, 0),
            direction: PortDirection::Input,
        };
        let json = serde_json::to_string(&port).unwrap();
        let back: BusPort = serde_json::from_str(&json).unwrap();
        assert_eq!(back, port);
    }
}
