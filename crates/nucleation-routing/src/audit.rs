//! Structural audit: every block that needs something to hold it up must
//! have it (port of `audit.py`).
//!
//! The simulator is happy to tick a floating wire, so this is checked
//! statically — otherwise the schematic only works until someone actually
//! pastes it.

use crate::blocks::{facing_of, is_solid_block};
use pnr_core::Pos;
use std::collections::BTreeMap;

/// Substring keys of blocks that need a solid floor below them.
/// (`lever[face=floor` and standing `redstone_torch`, not wall torches.)
pub const NEEDS_FLOOR: [&str; 5] = [
    "redstone_wire",
    "repeater",
    "comparator",
    "lever[face=floor",
    "redstone_torch[",
];

/// The audit findings.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuditReport {
    /// Blocks needing a floor whose support cell is not solid:
    /// `(pos, block name, what is below)`.
    pub floating: Vec<(Pos, String, Option<String>)>,
    /// Wall torches whose anchor block is not solid: `(pos, facing, anchor)`.
    pub unattached_wall_torch: Vec<(Pos, String, Pos)>,
}

impl AuditReport {
    /// True when nothing floats.
    pub fn is_clean(&self) -> bool {
        self.floating.is_empty() && self.unattached_wall_torch.is_empty()
    }
}

/// Audit every cell. Deterministic order (sorted by position).
pub fn audit(cells: &BTreeMap<Pos, String>) -> AuditReport {
    let mut report = AuditReport::default();
    let solid = |p: Pos| cells.get(&p).is_some_and(|b| is_solid_block(b));
    for (p, block) in cells {
        if block.contains("redstone_wall_torch") {
            let face = facing_of(block).unwrap_or("north");
            // The anchor is one step OPPOSITE the facing.
            let back = match face {
                "north" => (0, 0, 1),
                "south" => (0, 0, -1),
                "east" => (-1, 0, 0),
                _ => (1, 0, 0), // west
            };
            let anchor = p.offset(back.0, back.1, back.2);
            if !solid(anchor) {
                report
                    .unattached_wall_torch
                    .push((*p, face.to_string(), anchor));
            }
            continue;
        }
        if NEEDS_FLOOR.iter().any(|k| block.contains(k)) {
            let below = p.offset(0, -1, 0);
            // Support legality is STURDINESS, not conductivity: glass and
            // top-half slabs hold dust (probed material model — the bus
            // dip's transparent supports must not read as floating).
            let sturdy = cells
                .get(&below)
                .is_some_and(|b| crate::blocks::is_sturdy_support(b));
            if !sturdy {
                let name = block.split('[').next().unwrap_or(block).to_string();
                report.floating.push((*p, name, cells.get(&below).cloned()));
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{repeater, wall_torch, DUST, STONE, TORCH};

    #[test]
    fn floating_dust_and_repeater_are_reported() {
        let mut cells = BTreeMap::new();
        cells.insert(Pos::new(0, 1, 0), DUST.to_string()); // no floor
        cells.insert(Pos::new(1, 0, 0), STONE.to_string());
        cells.insert(Pos::new(1, 1, 0), repeater("west", 1)); // supported
        cells.insert(Pos::new(2, 1, 0), repeater("west", 1)); // floating
        let r = audit(&cells);
        assert_eq!(r.floating.len(), 2);
        assert_eq!(r.floating[0].0, Pos::new(0, 1, 0));
        assert_eq!(r.floating[0].1, "minecraft:redstone_wire");
        assert_eq!(r.floating[1].0, Pos::new(2, 1, 0));
    }

    #[test]
    fn glass_and_top_slabs_are_sturdy_supports() {
        // Provenance: the bus dip (bus8_cross v2) rests dust on glass where
        // a diagonal below must survive — sturdiness is not conductivity.
        let mut cells = BTreeMap::new();
        cells.insert(Pos::new(0, 0, 0), "minecraft:glass".to_string());
        cells.insert(Pos::new(0, 1, 0), DUST.to_string());
        cells.insert(
            Pos::new(1, 0, 0),
            "minecraft:smooth_stone_slab[type=top,waterlogged=false]".to_string(),
        );
        cells.insert(Pos::new(1, 1, 0), DUST.to_string());
        assert!(audit(&cells).is_clean());
    }

    #[test]
    fn standing_torch_needs_floor_but_wall_torch_needs_anchor() {
        let mut cells = BTreeMap::new();
        cells.insert(Pos::new(0, 1, 0), TORCH.to_string()); // floating standing torch
                                                            // Wall torch facing south hangs off the block one step north.
        cells.insert(Pos::new(5, 1, 5), wall_torch("south", true));
        let r = audit(&cells);
        assert_eq!(r.floating.len(), 1);
        assert_eq!(r.unattached_wall_torch.len(), 1);
        assert_eq!(r.unattached_wall_torch[0].2, Pos::new(5, 1, 4));
        // Anchor it: clean.
        cells.insert(Pos::new(5, 1, 4), STONE.to_string());
        cells.insert(Pos::new(0, 0, 0), STONE.to_string());
        assert!(audit(&cells).is_clean());
    }
}
