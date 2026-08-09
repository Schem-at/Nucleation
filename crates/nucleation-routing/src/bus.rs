//! Buses as a first-class concept: geometry + ordering + encoding contract.
//!
//! Two `BusPort`s mate iff their specs are compatible — the contract that
//! previously lived in someone's head and failed twice (seam shorts, PITCH
//! mismatch).

use pnr_core::Pos;

/// A grid axis.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Axis {
    /// X.
    X,
    /// Y.
    Y,
    /// Z.
    Z,
}

impl Axis {
    /// Unit step along this axis.
    pub fn unit(self) -> (i32, i32, i32) {
        match self {
            Axis::X => (1, 0, 0),
            Axis::Y => (0, 1, 0),
            Axis::Z => (0, 0, 1),
        }
    }
}

/// A cell face a bus presents on.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Face {
    /// -Z.
    North,
    /// +Z.
    South,
    /// +X.
    East,
    /// -X.
    West,
    /// +Y.
    Up,
    /// -Y.
    Down,
}

impl Face {
    /// The face that mates with this one on an abutting cell.
    pub fn opposite(self) -> Face {
        match self {
            Face::North => Face::South,
            Face::South => Face::North,
            Face::East => Face::West,
            Face::West => Face::East,
            Face::Up => Face::Down,
            Face::Down => Face::Up,
        }
    }
}

/// Port direction.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InOut {
    /// Signal enters the cell here.
    In,
    /// Signal leaves the cell here.
    Out,
}

/// Wire encoding of a word.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Encoding {
    /// One binary bit per wire.
    Binary1PerWire,
    /// A hex digit (signal strength 0-15) per wire — redstone's native
    /// density advantage; reserved in the spec even while v1 implements
    /// Binary only.
    HexAnalog,
}

/// Spacing between bus bits.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Pitch {
    /// Axis the bits advance along.
    pub axis: Axis,
    /// Cells between consecutive bits.
    pub spacing: i32,
}

/// A bus: width + pitch + face + encoding.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BusSpec {
    /// Number of wires.
    pub width: u8,
    /// Bit spacing.
    pub pitch: Pitch,
    /// Which cell face the bus presents on.
    pub face: Face,
    /// Wire encoding.
    pub encoding: Encoding,
}

/// A bus endpoint on a placed cell.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BusPort {
    /// The geometry/encoding contract.
    pub spec: BusSpec,
    /// Position of bit 0.
    pub bit0: Pos,
    /// Direction.
    pub dir: InOut,
}

impl BusPort {
    /// Position of bit `i`.
    pub fn bit(&self, i: u8) -> Pos {
        let (dx, dy, dz) = self.spec.pitch.axis.unit();
        let k = i as i32 * self.spec.pitch.spacing;
        self.bit0.offset(dx * k, dy * k, dz * k)
    }

    /// Whether two ports mate by abutment: same width/pitch/encoding,
    /// opposite faces, opposite directions — checkable at placement time
    /// instead of failing in-sim.
    pub fn mates(&self, other: &BusPort) -> bool {
        self.spec.width == other.spec.width
            && self.spec.pitch == other.spec.pitch
            && self.spec.encoding == other.spec.encoding
            && self.spec.face == other.spec.face.opposite()
            && self.dir != other.dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(face: Face, spacing: i32) -> BusSpec {
        BusSpec {
            width: 4,
            pitch: Pitch {
                axis: Axis::Z,
                spacing,
            },
            face,
            encoding: Encoding::Binary1PerWire,
        }
    }

    #[test]
    fn bit_positions_follow_pitch() {
        let p = BusPort {
            spec: spec(Face::West, 13),
            bit0: Pos::new(0, 1, 0),
            dir: InOut::In,
        };
        assert_eq!(p.bit(0), Pos::new(0, 1, 0));
        assert_eq!(p.bit(2), Pos::new(0, 1, 26));
    }

    #[test]
    fn abutment_compatibility_is_checkable() {
        let out = BusPort {
            spec: spec(Face::East, 13),
            bit0: Pos::new(9, 1, 0),
            dir: InOut::Out,
        };
        let input = BusPort {
            spec: spec(Face::West, 13),
            bit0: Pos::new(10, 1, 0),
            dir: InOut::In,
        };
        assert!(out.mates(&input));
        // PITCH mismatch — the exact failure the FA/RCA seam hit.
        let wrong_pitch = BusPort {
            spec: spec(Face::West, 12),
            bit0: Pos::new(10, 1, 0),
            dir: InOut::In,
        };
        assert!(!out.mates(&wrong_pitch));
        // Same-direction ports never mate.
        let also_out = BusPort {
            spec: spec(Face::West, 13),
            bit0: Pos::new(10, 1, 0),
            dir: InOut::Out,
        };
        assert!(!out.mates(&also_out));
    }
}
