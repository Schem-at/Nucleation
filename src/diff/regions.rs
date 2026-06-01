//! Cluster change cells into connected regions (face-connectivity).

use std::collections::{HashMap, HashSet, VecDeque};

use crate::diff::{Diff, IVec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RegionKind {
    Added,
    Removed,
    Changed,
    Mixed,
}

#[derive(Clone, Debug)]
pub struct Region {
    pub min: IVec3,
    pub max: IVec3,
    pub kind: RegionKind,
    pub count: u32,
}

pub fn regions(diff: &Diff) -> Vec<Region> {
    let mut kind_of: HashMap<IVec3, RegionKind> = HashMap::new();
    for (p, _) in &diff.added {
        kind_of.insert(*p, RegionKind::Added);
    }
    for (p, _) in &diff.removed {
        kind_of.insert(*p, RegionKind::Removed);
    }
    for (p, _, _) in &diff.changed {
        kind_of.insert(*p, RegionKind::Changed);
    }

    let all: HashSet<IVec3> = kind_of.keys().copied().collect();
    let mut seen: HashSet<IVec3> = HashSet::new();
    let mut out = Vec::new();
    let neighbours = |(x, y, z): IVec3| {
        [
            (x + 1, y, z),
            (x - 1, y, z),
            (x, y + 1, z),
            (x, y - 1, z),
            (x, y, z + 1),
            (x, y, z - 1),
        ]
    };
    for &start in &all {
        if seen.contains(&start) {
            continue;
        }
        let mut q = VecDeque::from([start]);
        seen.insert(start);
        let (mut mn, mut mx) = (start, start);
        let mut count = 0u32;
        let mut kinds: HashSet<RegionKind> = HashSet::new();
        while let Some(p) = q.pop_front() {
            count += 1;
            kinds.insert(kind_of[&p]);
            mn = (mn.0.min(p.0), mn.1.min(p.1), mn.2.min(p.2));
            mx = (mx.0.max(p.0), mx.1.max(p.1), mx.2.max(p.2));
            for n in neighbours(p) {
                if all.contains(&n) && !seen.contains(&n) {
                    seen.insert(n);
                    q.push_back(n);
                }
            }
        }
        let kind = if kinds.len() == 1 {
            *kinds.iter().next().unwrap()
        } else {
            RegionKind::Mixed
        };
        out.push(Region {
            min: mn,
            max: mx,
            kind,
            count,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_state::BlockState;
    use crate::diff::{diff, DiffSpec};
    use crate::fingerprint::testgen::filled_box;
    use crate::fingerprint::FingerprintSpec;

    #[test]
    fn two_separate_added_blobs_make_two_regions() {
        let a = filled_box((0, 0, 0), (0, 0, 0), "minecraft:stone");
        let mut b = filled_box((0, 0, 0), (0, 0, 0), "minecraft:stone");
        b.set_block(10, 0, 0, &BlockState::new("minecraft:stone"));
        b.set_block(20, 0, 0, &BlockState::new("minecraft:stone"));
        let spec = DiffSpec::from_preset(FingerprintSpec::structural());
        let d = diff(&a, &b, &spec);
        let regs = regions(&d);
        assert_eq!(regs.len(), 2, "two separated added cells → two regions");
        assert!(regs.iter().all(|r| matches!(r.kind, RegionKind::Added)));
    }
}
