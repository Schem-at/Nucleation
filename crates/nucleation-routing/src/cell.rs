//! Placeable cell fragments and verified cell templates (port of
//! `cells.py`'s `Fragment`, plus the design doc's `CellTemplate` contract).

use crate::workspace::{Collision, Workspace};
use pnr_core::{Aabb, Pos};
use std::collections::{BTreeMap, BTreeSet};

/// A placeable piece: cells + labels + named ports, in fragment-local
/// coordinates.
#[derive(Clone, Debug, Default)]
pub struct Fragment {
    /// Block per cell.
    pub cells: BTreeMap<Pos, String>,
    /// Net label per cell.
    pub labels: BTreeMap<Pos, String>,
    /// Named port positions.
    pub ports: BTreeMap<String, Pos>,
}

impl Fragment {
    /// An empty fragment.
    pub fn new() -> Self {
        Self::default()
    }

    /// Tight bounding box of the fragment's cells.
    pub fn bounds(&self) -> Option<Aabb> {
        let mut it = self.cells.keys();
        let first = *it.next()?;
        let mut bb = Aabb::new(first, first);
        for p in it {
            bb.min = Pos::new(bb.min.x.min(p.x), bb.min.y.min(p.y), bb.min.z.min(p.z));
            bb.max = Pos::new(bb.max.x.max(p.x), bb.max.y.max(p.y), bb.max.z.max(p.z));
        }
        Some(bb)
    }

    /// Stamp the fragment into a workspace at `(dx, dy, dz)`, renaming net
    /// labels through `rename` (identity for signals it returns unchanged).
    /// Returns the translated port map. Exact stamps copy state strings
    /// verbatim — stored cell state (locked flags, comparator outputs) must
    /// survive placement.
    pub fn stamp(
        &self,
        ws: &mut Workspace,
        (dx, dy, dz): (i32, i32, i32),
        rename: &dyn Fn(&str) -> String,
    ) -> Result<BTreeMap<String, Pos>, Collision> {
        for (p, blk) in &self.cells {
            ws.put(p.offset(dx, dy, dz), blk)?;
        }
        for (p, lab) in &self.labels {
            ws.set_label(p.offset(dx, dy, dz), &rename(lab));
        }
        Ok(self
            .ports
            .iter()
            .map(|(name, p)| (name.clone(), p.offset(dx, dy, dz)))
            .collect())
    }
}

/// Which boundary cells of a cell's footprint may carry nets.
///
/// Provenance: seam adjacency — PITCH must leave a clearance row; only the
/// port column may touch the boundary (two cells' port approaches once
/// formed a repeater ring across the seam).
#[derive(Clone, Debug, Default)]
pub struct EdgeContract {
    /// Fragment-local boundary cells allowed to hold blocks/nets.
    pub allowed: BTreeSet<Pos>,
}

impl EdgeContract {
    /// Check a fragment against the contract along `axis_z` seams: every
    /// occupied cell on the min/max z rows of `footprint` must be allowed.
    /// Returns offenders.
    pub fn check_z_seams(&self, frag: &Fragment, footprint: Aabb) -> Vec<Pos> {
        frag.cells
            .keys()
            .filter(|p| {
                (p.z == footprint.min.z || p.z == footprint.max.z) && !self.allowed.contains(p)
            })
            .copied()
            .collect()
    }
}

/// A verified placeable template: fragment + physical contract.
#[derive(Clone, Debug)]
pub struct CellTemplate {
    /// Template name.
    pub name: String,
    /// The geometry.
    pub fragment: Fragment,
    /// The cell's claimed footprint (no foreign routing inside).
    pub keepout: Aabb,
    /// Which boundary cells may carry nets.
    pub edge_contract: EdgeContract,
    /// Port-to-port delays in redstone ticks, from characterization.
    pub delay_rt: BTreeMap<(String, String), u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{DUST, STONE};

    fn tiny() -> Fragment {
        let mut f = Fragment::new();
        f.cells.insert(Pos::new(0, 0, 0), STONE.to_string());
        f.cells.insert(Pos::new(0, 1, 0), DUST.to_string());
        f.labels.insert(Pos::new(0, 1, 0), "a".to_string());
        f.ports.insert("a".to_string(), Pos::new(0, 1, 0));
        f
    }

    #[test]
    fn stamp_translates_and_renames() {
        let f = tiny();
        let mut ws = Workspace::new();
        let ports = f
            .stamp(&mut ws, (10, 0, 4), &|s| {
                if s == "a" {
                    "h1x".to_string()
                } else {
                    format!("h2{s}")
                }
            })
            .unwrap();
        assert_eq!(ports["a"], Pos::new(10, 1, 4));
        assert_eq!(ws.get(Pos::new(10, 1, 4)), Some(DUST));
        assert_eq!(ws.label(Pos::new(10, 1, 4)), Some("h1x"));
    }

    #[test]
    fn stamp_collision_is_an_error() {
        let f = tiny();
        let mut ws = Workspace::new();
        ws.put(Pos::new(0, 1, 0), STONE).unwrap();
        assert!(f.stamp(&mut ws, (0, 0, 0), &|s| s.to_string()).is_err());
    }

    #[test]
    fn edge_contract_flags_non_port_seam_cells() {
        // RULE (bug provenance): only the port column may touch the cell
        // boundary — cross-cell seam shorts / the repeater ring both came
        // from stray seam occupancy.
        let mut f = Fragment::new();
        f.cells.insert(Pos::new(3, 1, 0), DUST.to_string()); // on min-z seam, allowed (port)
        f.cells.insert(Pos::new(7, 1, 0), DUST.to_string()); // on min-z seam, NOT allowed
        f.cells.insert(Pos::new(5, 1, 5), DUST.to_string()); // interior
        let footprint = Aabb::new(Pos::new(0, 0, 0), Pos::new(10, 3, 12));
        let mut contract = EdgeContract::default();
        contract.allowed.insert(Pos::new(3, 1, 0));
        let offenders = contract.check_z_seams(&f, footprint);
        assert_eq!(offenders, vec![Pos::new(7, 1, 0)]);
    }
}
