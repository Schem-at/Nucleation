//! Transactional voxel occupancy + net labels.
//!
//! All routing is transactional: claim cells, then commit or roll back —
//! rip-up-and-reroute needs cheap undo. The workspace is a plain map of
//! block-state strings, so it can be filled from any schematic source (the
//! nucleation bridge converts `UniversalSchematic` both ways).

use crate::blocks;
use pnr_core::{Aabb, Pos};
use std::collections::BTreeMap;

/// A cell collision: `put` refused to overwrite an existing different block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Collision {
    /// Where.
    pub at: Pos,
    /// What was there.
    pub existing: String,
    /// What was being placed.
    pub placing: String,
}

/// Occupancy + labels over a build region, with journaled transactions.
#[derive(Clone, Debug, Default)]
pub struct Workspace {
    cells: BTreeMap<Pos, String>,
    labels: BTreeMap<Pos, String>,
    /// Undo journal: prior (cell, label) per touched position, oldest first.
    journal: Vec<(Pos, Option<String>, Option<String>)>,
    journaling: bool,
}

impl Workspace {
    /// An empty workspace.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a workspace from `(pos, block_state_string)` pairs.
    pub fn from_blocks(iter: impl IntoIterator<Item = (Pos, String)>) -> Self {
        let mut ws = Self::new();
        for (p, b) in iter {
            ws.cells.insert(p, b);
        }
        ws
    }

    /// The block at `p`.
    pub fn get(&self, p: Pos) -> Option<&str> {
        self.cells.get(&p).map(|s| s.as_str())
    }

    /// The net label at `p`.
    pub fn label(&self, p: Pos) -> Option<&str> {
        self.labels.get(&p).map(|s| s.as_str())
    }

    /// All cells, in deterministic order.
    pub fn cells(&self) -> &BTreeMap<Pos, String> {
        &self.cells
    }

    /// All labels, in deterministic order.
    pub fn labels(&self) -> &BTreeMap<Pos, String> {
        &self.labels
    }

    fn journal_touch(&mut self, p: Pos) {
        if self.journaling {
            self.journal
                .push((p, self.cells.get(&p).cloned(), self.labels.get(&p).cloned()));
        }
    }

    /// Place a block; refuses to overwrite a different existing block (the
    /// prototype's cell-collision assertion, as a `Result`).
    pub fn put(&mut self, p: Pos, block: &str) -> Result<(), Collision> {
        if let Some(prev) = self.cells.get(&p) {
            if prev != block {
                return Err(Collision {
                    at: p,
                    existing: prev.clone(),
                    placing: block.to_string(),
                });
            }
            return Ok(());
        }
        self.journal_touch(p);
        self.cells.insert(p, block.to_string());
        Ok(())
    }

    /// Deliberately overwrite whatever is in this cell.
    pub fn force(&mut self, p: Pos, block: &str) {
        self.journal_touch(p);
        self.cells.insert(p, block.to_string());
    }

    /// Place a structural block with a role colour, unless the cell is
    /// already solid (structural blocks legitimately coincide; first role
    /// keeps its colour — the block is solid either way).
    pub fn stone(&mut self, p: Pos, role: &str) -> Result<(), Collision> {
        if self.solid_at(p) {
            return Ok(());
        }
        self.put(p, blocks::role_block(role))
    }

    /// Whether the cell holds a solid conductor.
    pub fn solid_at(&self, p: Pos) -> bool {
        self.get(p).is_some_and(blocks::is_solid_block)
    }

    /// Label a cell with a net name.
    pub fn set_label(&mut self, p: Pos, label: &str) {
        self.journal_touch(p);
        self.labels.insert(p, label.to_string());
    }

    /// Dust with support: structural block below + dust + label.
    pub fn dust(&mut self, p: Pos, label: &str) -> Result<(), Collision> {
        self.stone(p.offset(0, -1, 0), "plain")?;
        self.put(p, blocks::DUST)?;
        self.set_label(p, label);
        Ok(())
    }

    /// Begin a transaction. Nested transactions are not supported; a second
    /// `begin` while journaling is a no-op (the outer journal keeps going).
    pub fn begin(&mut self) {
        if !self.journaling {
            self.journal.clear();
            self.journaling = true;
        }
    }

    /// Commit the open transaction (keep changes, drop the journal).
    pub fn commit(&mut self) {
        self.journal.clear();
        self.journaling = false;
    }

    /// Roll the open transaction back, restoring every touched cell/label.
    pub fn rollback(&mut self) {
        let journal = std::mem::take(&mut self.journal);
        self.journaling = false;
        for (p, cell, label) in journal.into_iter().rev() {
            match cell {
                Some(b) => {
                    self.cells.insert(p, b);
                }
                None => {
                    self.cells.remove(&p);
                }
            }
            match label {
                Some(l) => {
                    self.labels.insert(p, l);
                }
                None => {
                    self.labels.remove(&p);
                }
            }
        }
    }

    /// Tight bounding box of the occupied cells.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collision_is_refused_and_force_overrides() {
        let mut ws = Workspace::new();
        ws.put(Pos::new(0, 0, 0), blocks::STONE).unwrap();
        // Same block again: fine (idempotent).
        ws.put(Pos::new(0, 0, 0), blocks::STONE).unwrap();
        let err = ws.put(Pos::new(0, 0, 0), blocks::DUST).unwrap_err();
        assert_eq!(err.at, Pos::new(0, 0, 0));
        ws.force(Pos::new(0, 0, 0), blocks::DUST);
        assert_eq!(ws.get(Pos::new(0, 0, 0)), Some(blocks::DUST));
    }

    #[test]
    fn rollback_restores_cells_and_labels() {
        let mut ws = Workspace::new();
        ws.dust(Pos::new(1, 1, 0), "keep").unwrap();
        let before_cells = ws.cells.clone();
        let before_labels = ws.labels.clone();
        ws.begin();
        ws.dust(Pos::new(2, 1, 0), "tmp").unwrap();
        ws.force(Pos::new(1, 1, 0), blocks::STONE); // clobber inside txn
        ws.set_label(Pos::new(1, 1, 0), "clobbered");
        ws.rollback();
        assert_eq!(ws.cells, before_cells);
        assert_eq!(ws.labels, before_labels);
    }

    #[test]
    fn commit_keeps_changes() {
        let mut ws = Workspace::new();
        ws.begin();
        ws.dust(Pos::new(0, 1, 0), "n").unwrap();
        ws.commit();
        ws.rollback(); // nothing journaled: no-op
        assert_eq!(ws.get(Pos::new(0, 1, 0)), Some(blocks::DUST));
        assert_eq!(ws.label(Pos::new(0, 1, 0)), Some("n"));
    }

    #[test]
    fn stone_respects_existing_solids() {
        let mut ws = Workspace::new();
        ws.put(Pos::new(0, 0, 0), "minecraft:gray_concrete")
            .unwrap();
        // A route support over an existing rail floor keeps its colour.
        ws.stone(Pos::new(0, 0, 0), "route").unwrap();
        assert_eq!(ws.get(Pos::new(0, 0, 0)), Some("minecraft:gray_concrete"));
    }
}
