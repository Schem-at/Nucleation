//! Dust electrical adjacency + the static net-shorting checker.
//!
//! Exact port of `nets.py`. Redstone dust connects to dust at the 4
//! horizontal neighbours, and diagonally one step up/down, with the
//! up-diagonal blocked when the cell above the LOWER dust is solid (exactly
//! what rail lids exploit). Both of the real bugs that mandated this were
//! unintended adjacency: a tap output touching its own input rail, and a
//! lamp adjacent to a wire pointing the wrong way. Simulation only says
//! "wrong answer somewhere"; this says which two signals touch, and where.

use crate::blocks::{is_dust, is_solid_block};
use pnr_core::netcheck::{self, Short};
use pnr_core::Pos;
use std::collections::BTreeMap;

/// Whether the cell at `pos` holds a solid conductor.
pub fn is_solid(cells: &BTreeMap<Pos, String>, pos: Pos) -> bool {
    cells.get(&pos).is_some_and(|b| is_solid_block(b))
}

/// Cells whose dust is electrically connected to dust at `pos`.
///
/// (Pure in `pos`: only surrounding cells are consulted, so callers may ask
/// about a hypothetical dust cell without inserting it — the clearance check
/// depends on this.)
pub fn neighbours(cells: &BTreeMap<Pos, String>, pos: Pos) -> Vec<Pos> {
    let Pos { x, y, z } = pos;
    let mut out = Vec::new();
    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let side = Pos::new(x + dx, y, z + dz);
        if cells.get(&side).is_some_and(|b| is_dust(b)) {
            out.push(side);
        }
        // Dust one step DOWN in that direction: blocked if the cell above
        // the lower dust is solid.
        let low = Pos::new(x + dx, y - 1, z + dz);
        if cells.get(&low).is_some_and(|b| is_dust(b))
            && !is_solid(cells, Pos::new(x + dx, y, z + dz))
        {
            out.push(low);
        }
        // Dust one step UP: blocked if the cell above THIS dust is solid.
        let high = Pos::new(x + dx, y + 1, z + dz);
        if cells.get(&high).is_some_and(|b| is_dust(b)) && !is_solid(cells, Pos::new(x, y + 1, z))
        {
            out.push(high);
        }
    }
    out
}

/// Prove no two distinct labels share an electrical net.
///
/// `aliases` are label pairs that are DELIBERATELY the same electrical net
/// (a routed wire joining a producer lane to a consumer rail). Returns one
/// short per offending component.
pub fn check(
    cells: &BTreeMap<Pos, String>,
    labels: &BTreeMap<Pos, String>,
    aliases: &[(String, String)],
) -> Vec<Short<Pos, String>> {
    let dust: Vec<Pos> = cells
        .iter()
        .filter(|(_, b)| is_dust(b))
        .map(|(p, _)| *p)
        .collect();
    netcheck::find_shorts(
        &dust,
        |p| neighbours(cells, *p),
        |p| labels.get(p).cloned(),
        aliases,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::{DUST, STONE};

    fn dust_at(cells: &mut BTreeMap<Pos, String>, p: Pos) {
        cells.insert(p, DUST.to_string());
    }

    #[test]
    fn horizontal_and_diagonal_adjacency() {
        let mut cells = BTreeMap::new();
        dust_at(&mut cells, Pos::new(0, 1, 0));
        dust_at(&mut cells, Pos::new(1, 1, 0)); // side
        dust_at(&mut cells, Pos::new(-1, 2, 0)); // up-diagonal
        dust_at(&mut cells, Pos::new(0, 0, 1)); // down-diagonal
        let n = neighbours(&cells, Pos::new(0, 1, 0));
        assert!(n.contains(&Pos::new(1, 1, 0)));
        assert!(n.contains(&Pos::new(-1, 2, 0)));
        assert!(n.contains(&Pos::new(0, 0, 1)));
    }

    #[test]
    fn lid_cuts_the_up_diagonal() {
        // The rail-lid trick: a solid above the LOWER dust cuts the
        // up-diagonal connection.
        let mut cells = BTreeMap::new();
        dust_at(&mut cells, Pos::new(0, 1, 0));
        dust_at(&mut cells, Pos::new(1, 2, 0));
        assert!(neighbours(&cells, Pos::new(0, 1, 0)).contains(&Pos::new(1, 2, 0)));
        cells.insert(Pos::new(0, 2, 0), STONE.to_string()); // lid above lower
        assert!(!neighbours(&cells, Pos::new(0, 1, 0)).contains(&Pos::new(1, 2, 0)));
        // Symmetric view from the upper dust: its DOWN-diagonal is cut by
        // the solid above the lower dust.
        assert!(!neighbours(&cells, Pos::new(1, 2, 0)).contains(&Pos::new(0, 1, 0)));
    }

    #[test]
    fn braid_short_is_reported_with_witnesses() {
        // Two labelled runs joined by a stray dust cell — every braid short
        // in the adder sessions looked exactly like this.
        let mut cells = BTreeMap::new();
        let mut labels = BTreeMap::new();
        for x in 0..3 {
            dust_at(&mut cells, Pos::new(x, 1, 0));
            labels.insert(Pos::new(x, 1, 0), "a".to_string());
            dust_at(&mut cells, Pos::new(x, 1, 2));
            labels.insert(Pos::new(x, 1, 2), "b".to_string());
        }
        assert!(check(&cells, &labels, &[]).is_empty());
        dust_at(&mut cells, Pos::new(1, 1, 1)); // the stray
        let shorts = check(&cells, &labels, &[]);
        assert_eq!(shorts.len(), 1);
        assert_eq!(
            (shorts[0].label_a.as_str(), shorts[0].label_b.as_str()),
            ("a", "b")
        );
    }

    #[test]
    fn exact_label_distinction_and_aliases() {
        // "sig#13" (a pre-gate collector) is a DIFFERENT electrical net from
        // "sig": touching it is a short — unless deliberately aliased.
        let mut cells = BTreeMap::new();
        let mut labels = BTreeMap::new();
        dust_at(&mut cells, Pos::new(0, 1, 0));
        labels.insert(Pos::new(0, 1, 0), "sig".to_string());
        dust_at(&mut cells, Pos::new(1, 1, 0));
        labels.insert(Pos::new(1, 1, 0), "sig#13".to_string());
        assert_eq!(check(&cells, &labels, &[]).len(), 1);
        let aliases = vec![("sig".to_string(), "sig#13".to_string())];
        assert!(check(&cells, &labels, &aliases).is_empty());
    }
}
