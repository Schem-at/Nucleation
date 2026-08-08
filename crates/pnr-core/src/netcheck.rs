//! Generic net-shorting checker: prove no two distinct labels share a
//! connected component.
//!
//! Framework port of the Python `nets.check`: the caller supplies the nodes,
//! an adjacency function (the technology's connectivity rules — e.g. redstone
//! dust adjacency with cut diagonals), a labelling, and alias pairs (labels
//! that are DELIBERATELY the same electrical net, e.g. a routed wire joining
//! a producer lane to a consumer rail). Both real bugs that mandated the
//! Python original were unintended adjacency: simulation only says "wrong
//! answer somewhere"; this says which two signals touch, and where.

use crate::unionfind::UnionFind;
use std::collections::BTreeMap;

/// A detected short: two distinct (unaliased) labels in one component, with
/// a witness node for each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Short<N, L> {
    /// First label (the smaller under `Ord`).
    pub label_a: L,
    /// Second label.
    pub label_b: L,
    /// A node carrying `label_a` in the shared component.
    pub at_a: N,
    /// A node carrying `label_b` in the shared component.
    pub at_b: N,
}

/// Find all label pairs that share a connected component.
///
/// `neighbours` must be symmetric in effect (if it is not, components are
/// still formed from the union of both directions encountered).
pub fn find_shorts<N, L>(
    nodes: &[N],
    mut neighbours: impl FnMut(&N) -> Vec<N>,
    mut label_of: impl FnMut(&N) -> Option<L>,
    aliases: &[(L, L)],
) -> Vec<Short<N, L>>
where
    N: Ord + Clone,
    L: Ord + Clone,
{
    let mut lalias: UnionFind<L> = UnionFind::new();
    for (a, b) in aliases {
        lalias.union(a, b);
    }
    let mut uf: UnionFind<N> = UnionFind::new();
    for n in nodes {
        uf.find(n);
        for q in neighbours(n) {
            uf.union(n, &q);
        }
    }
    let mut comps: BTreeMap<N, Vec<N>> = BTreeMap::new();
    for n in nodes {
        let r = uf.find(n);
        comps.entry(r).or_default().push(n.clone());
    }
    let mut shorts = Vec::new();
    for members in comps.values() {
        // First witness per alias-class, in deterministic node order.
        let mut seen: BTreeMap<L, N> = BTreeMap::new();
        for n in members {
            if let Some(lab) = label_of(n) {
                let root = lalias.find(&lab);
                seen.entry(root).or_insert_with(|| n.clone());
            }
        }
        if seen.len() > 1 {
            let items: Vec<(&L, &N)> = seen.iter().collect();
            shorts.push(Short {
                label_a: items[0].0.clone(),
                label_b: items[1].0.clone(),
                at_a: items[0].1.clone(),
                at_b: items[1].1.clone(),
            });
        }
    }
    shorts
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Line graph 0-1-2-3-4 with labels at the ends.
    fn line_neighbours(n: &i32) -> Vec<i32> {
        let mut out = Vec::new();
        if *n > 0 {
            out.push(n - 1);
        }
        if *n < 4 {
            out.push(n + 1);
        }
        out
    }

    #[test]
    fn detects_a_short() {
        let nodes: Vec<i32> = (0..5).collect();
        let labels = |n: &i32| match n {
            0 => Some("a"),
            4 => Some("b"),
            _ => None,
        };
        let shorts = find_shorts(&nodes, line_neighbours, labels, &[]);
        assert_eq!(shorts.len(), 1);
        assert_eq!((shorts[0].label_a, shorts[0].label_b), ("a", "b"));
    }

    #[test]
    fn aliases_silence_deliberate_joins() {
        let nodes: Vec<i32> = (0..5).collect();
        let labels = |n: &i32| match n {
            0 => Some("a"),
            4 => Some("b"),
            _ => None,
        };
        let shorts = find_shorts(&nodes, line_neighbours, labels, &[("a", "b")]);
        assert!(shorts.is_empty());
    }

    #[test]
    fn disconnected_labels_are_clean() {
        // Two components: {0,1} and {3,4} (2 removed from the node list and
        // adjacency clipped).
        let nodes = vec![0, 1, 3, 4];
        let neigh = |n: &i32| -> Vec<i32> {
            match n {
                0 => vec![1],
                1 => vec![0],
                3 => vec![4],
                4 => vec![3],
                _ => vec![],
            }
        };
        let labels = |n: &i32| match n {
            0 => Some("a"),
            4 => Some("b"),
            _ => None,
        };
        assert!(find_shorts(&nodes, neigh, labels, &[]).is_empty());
    }
}
