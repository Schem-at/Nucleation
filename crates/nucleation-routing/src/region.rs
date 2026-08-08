//! DEF-style wiring-area constraints: include/exclude zones for routing.
//!
//! Cell keepouts, the FA edge contract, and the prototype's ad-hoc `bounds`
//! are all special cases of this mechanism.

use pnr_core::{Aabb, Pos};

/// A routing region: a cell is routable when it is inside at least one
/// include box (or the include list is empty) and inside no exclude box.
#[derive(Clone, Debug, Default)]
pub struct RoutingRegion {
    /// Boxes routing is confined to (empty = everywhere).
    pub include: Vec<Aabb>,
    /// Boxes routing must stay out of (keepouts).
    pub exclude: Vec<Aabb>,
}

impl RoutingRegion {
    /// Whether `p` is routable under this region.
    pub fn contains(&self, p: Pos) -> bool {
        (self.include.is_empty() || self.include.iter().any(|b| b.contains(p)))
            && !self.exclude.iter().any(|b| b.contains(p))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_and_exclude_compose() {
        let r = RoutingRegion {
            include: vec![Aabb::new(Pos::new(0, 0, 0), Pos::new(10, 5, 10))],
            exclude: vec![Aabb::new(Pos::new(4, 0, 4), Pos::new(6, 5, 6))],
        };
        assert!(r.contains(Pos::new(1, 1, 1)));
        assert!(!r.contains(Pos::new(5, 1, 5)), "keepout");
        assert!(!r.contains(Pos::new(11, 0, 0)), "outside include");
        let open = RoutingRegion::default();
        assert!(open.contains(Pos::new(-100, 0, 100)));
    }
}
