//! Shared scene for the pivot planner tests: an 8-bit vertical lever bank
//! (bus8 2y form) that must reach an 8-bit flat rail — the exact form
//! mismatch the v2h tile exists for.

use nucleation_routing::blocks::LEVER_OFF;
use nucleation_routing::bus::{Axis, BusPort, BusSpec, Encoding, Face, InOut, Pitch};
use nucleation_routing::{Pos, Workspace};

/// Tile-local origin the pivot is stamped at.
pub const PIVOT_AT: Pos = Pos { x: 6, y: 0, z: 0 };

/// Levers, their connection dusts, and the target rail cells (bit order).
pub struct Scene {
    pub ws: Workspace,
    pub levers: Vec<Pos>,
    pub from: BusPort,
    pub to: BusPort,
}

/// Build the scene: lever bank at x=4 driving connection dusts at x=5
/// (vertical 2y stack with separator supports, bus8 v2 form), and a flat
/// target rail of single dust cells at x=40, z=2n.
pub fn build_scene() -> Scene {
    let mut ws = Workspace::new();
    let mut levers = Vec::new();
    for n in 0..8 {
        let y = 1 + 2 * n;
        ws.stone(Pos::new(4, y - 1, 0), "plain").unwrap();
        ws.force(Pos::new(4, y, 0), LEVER_OFF);
        levers.push(Pos::new(4, y, 0));
        ws.dust(Pos::new(5, y, 0), &format!("bus{n}")).unwrap();
    }
    for n in 0..8 {
        ws.dust(Pos::new(40, 1, 2 * n), &format!("bus{n}")).unwrap();
    }
    let from = BusPort {
        spec: BusSpec {
            width: 8,
            pitch: Pitch {
                axis: Axis::Y,
                spacing: 2,
            },
            face: Face::East,
            encoding: Encoding::Binary1PerWire,
        },
        bit0: Pos::new(5, 1, 0),
        dir: InOut::Out,
    };
    let to = BusPort {
        spec: BusSpec {
            width: 8,
            pitch: Pitch {
                axis: Axis::Z,
                spacing: 2,
            },
            face: Face::West,
            encoding: Encoding::Binary1PerWire,
        },
        bit0: Pos::new(40, 1, 0),
        dir: InOut::In,
    };
    Scene {
        ws,
        levers,
        from,
        to,
    }
}
