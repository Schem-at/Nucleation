//! Union-find over arbitrary ordered keys (deterministic, allocation-light).

use std::collections::BTreeMap;

/// Disjoint-set forest keyed by any `Ord + Clone` type.
#[derive(Clone, Debug, Default)]
pub struct UnionFind<T: Ord + Clone> {
    parent: BTreeMap<T, T>,
}

impl<T: Ord + Clone> UnionFind<T> {
    /// An empty forest.
    pub fn new() -> Self {
        UnionFind {
            parent: BTreeMap::new(),
        }
    }

    /// Representative of `a`'s set (inserting `a` as a singleton if new).
    pub fn find(&mut self, a: &T) -> T {
        if !self.parent.contains_key(a) {
            self.parent.insert(a.clone(), a.clone());
            return a.clone();
        }
        // Path compression via two passes (borrow rules make halving clumsy).
        let mut root = a.clone();
        loop {
            let p = self.parent.get(&root).unwrap().clone();
            if p == root {
                break;
            }
            root = p;
        }
        let mut cur = a.clone();
        while cur != root {
            let p = self.parent.get(&cur).unwrap().clone();
            self.parent.insert(cur, root.clone());
            cur = p;
        }
        root
    }

    /// Merge the sets containing `a` and `b`.
    pub fn union(&mut self, a: &T, b: &T) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra != rb {
            self.parent.insert(ra, rb);
        }
    }

    /// Whether `a` and `b` share a set.
    pub fn same(&mut self, a: &T, b: &T) -> bool {
        self.find(a) == self.find(b)
    }

    /// Group all known keys by representative, deterministically ordered.
    pub fn components(&mut self) -> BTreeMap<T, Vec<T>> {
        let keys: Vec<T> = self.parent.keys().cloned().collect();
        let mut out: BTreeMap<T, Vec<T>> = BTreeMap::new();
        for k in keys {
            let r = self.find(&k);
            out.entry(r).or_default().push(k);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_union_find() {
        let mut uf = UnionFind::new();
        uf.union(&1, &2);
        uf.union(&3, &4);
        assert!(uf.same(&1, &2));
        assert!(!uf.same(&1, &3));
        uf.union(&2, &3);
        assert!(uf.same(&1, &4));
        assert_eq!(uf.components().len(), 1);
    }
}
