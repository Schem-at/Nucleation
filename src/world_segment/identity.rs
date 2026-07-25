//! Cross-snapshot identity: match this snapshot's builds against the
//! previous snapshot's records via bbox-IoU, so a build keeps a stable
//! identity across re-extractions (edit -> same id + new version; new
//! build -> new id; split/merge handled explicitly).
//!
//! Deterministic and order-independent: all grouping goes through
//! `BTreeMap`s keyed by id, and the final output is sorted by `build_id`
//! with ties broken on `ClusterId`/`StableBuildId` ordering.

use std::collections::BTreeMap;

use crate::world_segment::ids::ClusterId;
use crate::world_segment::provenance::StableBuildId;
use crate::world_segment::stitch::Build;

/// A record of a build from the prior snapshot, carried forward only as the
/// slice of state needed to re-match it: its stable identity, its bbox and
/// its size.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PriorBuild {
    pub stable_id: StableBuildId,
    pub bbox: ((i32, i32, i32), (i32, i32, i32)),
    pub block_count: u64,
}

/// What happened to a current build relative to the prior snapshot.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// No prior build overlapped this one enough: a fresh identity.
    New,
    /// Exactly one prior build matched, and it matched only this current
    /// build: the identity carries over unchanged.
    Same(StableBuildId),
    /// One prior build's footprint now overlaps several current builds
    /// (it broke apart). This current build is not the one that inherited
    /// the prior identity; it gets a fresh, seeded id.
    Split { inherits: StableBuildId },
    /// Several prior builds now overlap this one current build (they were
    /// joined). `from` lists all matched priors' stable ids, sorted.
    Merge { from: Vec<StableBuildId> },
}

/// The match result for one current build.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SnapshotMatch {
    pub build_id: ClusterId,
    pub stable_id: StableBuildId,
    pub outcome: Outcome,
}

/// Intersection-over-union of two inclusive world-coordinate bboxes.
///
/// Bounds are inclusive, so the length of a box along an axis is
/// `max - min + 1`, and the intersection along an axis is
/// `max(0, min(a.max, b.max) - max(a.min, b.min) + 1)`. IoU is the
/// intersection volume over the union volume (0.0 if the union is empty,
/// which cannot actually happen for two well-formed boxes but is guarded
/// against divide-by-zero regardless).
pub fn bbox_iou(a: ((i32, i32, i32), (i32, i32, i32)), b: ((i32, i32, i32), (i32, i32, i32))) -> f32 {
    fn axis_len(min: i32, max: i32) -> i64 {
        (max - min + 1) as i64
    }
    fn axis_inter(amin: i32, amax: i32, bmin: i32, bmax: i32) -> i64 {
        let lo = amin.max(bmin);
        let hi = amax.min(bmax);
        (hi - lo + 1).max(0) as i64
    }

    let ((amin_x, amin_y, amin_z), (amax_x, amax_y, amax_z)) = a;
    let ((bmin_x, bmin_y, bmin_z), (bmax_x, bmax_y, bmax_z)) = b;

    let ix = axis_inter(amin_x, amax_x, bmin_x, bmax_x);
    let iy = axis_inter(amin_y, amax_y, bmin_y, bmax_y);
    let iz = axis_inter(amin_z, amax_z, bmin_z, bmax_z);
    let inter = ix * iy * iz;

    let vol_a = axis_len(amin_x, amax_x) * axis_len(amin_y, amax_y) * axis_len(amin_z, amax_z);
    let vol_b = axis_len(bmin_x, bmax_x) * axis_len(bmin_y, bmax_y) * axis_len(bmin_z, bmax_z);
    let union = vol_a + vol_b - inter;

    if union <= 0 {
        0.0
    } else {
        inter as f32 / union as f32
    }
}

/// Match this snapshot's builds against the prior snapshot's records.
///
/// For each current build, the set of prior builds whose bbox-IoU with it
/// is `>= iou_threshold` is found. Priors are indexed by position in
/// `prior` (their index) rather than by `StableBuildId`, since two priors
/// could in principle carry equal stable ids only if the caller passed
/// duplicates; using the index keeps grouping well-defined regardless.
///
/// Ambiguity note: when a group of current and prior builds all mutually
/// overlap (a many-to-many tangle, not just one-to-many), this function
/// resolves it as a sequence of independent decisions per current build
/// (merge) and per prior build (split), not a single global assignment.
/// Concretely: a current build with multiple prior matches always takes
/// the merge branch (inheriting the largest-block-count prior), and a
/// prior with multiple current matches always contributes its identity to
/// the single largest-block-count current build among its matches
/// (ties broken by id ordering in both cases) while every other current
/// build it matches is treated as a split away from it. This is
/// deterministic and depends only on content (ClusterId/StableBuildId
/// ordering), never on input order, but it does mean a current build can
/// simultaneously be the "merge target" for some priors and the "split
/// survivor" for others; in that case its own match set alone decides
/// whether it is `Merge` (more than one prior matched it) or `Same`
/// (exactly one matched it and it is that prior's chosen survivor).
pub fn match_snapshots(
    current: &[Build],
    prior: &[PriorBuild],
    source_id: &str,
    iou_threshold: f32,
) -> Vec<SnapshotMatch> {
    // current index -> sorted list of prior indices that matched it.
    let mut current_matches: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    // prior index -> sorted list of current indices that matched it.
    let mut prior_matches: BTreeMap<usize, Vec<usize>> = BTreeMap::new();

    for (ci, c) in current.iter().enumerate() {
        for (pi, p) in prior.iter().enumerate() {
            if bbox_iou(c.bbox, p.bbox) >= iou_threshold {
                current_matches.entry(ci).or_default().push(pi);
                prior_matches.entry(pi).or_default().push(ci);
            }
        }
    }

    // For each prior with more than one current match (a split), determine
    // which current build is the survivor: the one with the largest
    // block_count, ties broken by ClusterId order.
    let mut split_survivor: BTreeMap<usize, usize> = BTreeMap::new();
    for (&pi, cis) in &prior_matches {
        if cis.len() > 1 {
            let survivor = *cis
                .iter()
                .max_by(|&&a, &&b| {
                    current[a]
                        .block_count
                        .cmp(&current[b].block_count)
                        .then(current[a].id.cmp(&current[b].id))
                })
                .unwrap();
            split_survivor.insert(pi, survivor);
        }
    }

    let mut results = Vec::with_capacity(current.len());

    for (ci, c) in current.iter().enumerate() {
        let matched_priors = current_matches.get(&ci).cloned().unwrap_or_default();

        let (stable_id, outcome) = if matched_priors.is_empty() {
            (StableBuildId::seed(source_id, c.id), Outcome::New)
        } else if matched_priors.len() > 1 {
            // Merge: several priors match this one current build. Inherit
            // the prior with the largest block_count, ties broken by
            // StableBuildId order.
            let inherited_pi = *matched_priors
                .iter()
                .max_by(|&&a, &&b| {
                    prior[a]
                        .block_count
                        .cmp(&prior[b].block_count)
                        .then(prior[a].stable_id.cmp(&prior[b].stable_id))
                })
                .unwrap();
            let mut from: Vec<StableBuildId> =
                matched_priors.iter().map(|&pi| prior[pi].stable_id).collect();
            from.sort();
            (prior[inherited_pi].stable_id, Outcome::Merge { from })
        } else {
            // Exactly one prior matched this current build.
            let pi = matched_priors[0];
            let is_split_of_this_prior = prior_matches.get(&pi).map_or(false, |cis| cis.len() > 1);
            if is_split_of_this_prior {
                if split_survivor.get(&pi) == Some(&ci) {
                    (prior[pi].stable_id, Outcome::Same(prior[pi].stable_id))
                } else {
                    (
                        StableBuildId::seed(source_id, c.id),
                        Outcome::Split { inherits: prior[pi].stable_id },
                    )
                }
            } else {
                (prior[pi].stable_id, Outcome::Same(prior[pi].stable_id))
            }
        };

        results.push(SnapshotMatch { build_id: c.id, stable_id, outcome });
    }

    results.sort_by_key(|m| m.build_id);
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_segment::ids::{ClusterId, ContentId, TileId};
    use crate::world_segment::stitch::Build;
    use crate::world_segment::provenance::StableBuildId;

    fn build(tag: &[u8], bbox: ((i32,i32,i32),(i32,i32,i32)), n: u64) -> Build {
        let id = ClusterId::new(ContentId::of(&[tag]), TileId{x:0,z:0}, None, (0,0,0));
        Build { id, cluster_ids: vec![id], bbox, block_count: n, cell_count: n, partition_id: None }
    }

    #[test]
    fn iou_of_identical_boxes_is_one() {
        assert!((bbox_iou(((0,0,0),(9,9,9)), ((0,0,0),(9,9,9))) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn iou_of_disjoint_boxes_is_zero() {
        assert_eq!(bbox_iou(((0,0,0),(1,1,1)), ((100,100,100),(101,101,101))), 0.0);
    }

    #[test]
    fn unmatched_current_is_new() {
        let cur = vec![build(b"a", ((0,0,0),(9,9,9)), 100)];
        let out = match_snapshots(&cur, &[], "w", 0.5);
        assert!(matches!(out[0].outcome, Outcome::New));
    }

    #[test]
    fn overlapping_current_inherits_prior_stable_id() {
        let prior = vec![PriorBuild { stable_id: StableBuildId::seed("w", build(b"old", ((0,0,0),(9,9,9)), 90).id),
                                       bbox: ((0,0,0),(9,9,9)), block_count: 90 }];
        let cur = vec![build(b"a", ((0,0,0),(9,9,9)), 100)]; // same box, edited
        let out = match_snapshots(&cur, &prior, "w", 0.5);
        assert_eq!(out[0].stable_id, prior[0].stable_id);
        assert!(matches!(out[0].outcome, Outcome::Same(_)));
    }

    /// One prior overlaps two currents that only partially cover it (a
    /// building split into two pieces). The larger current inherits the
    /// prior's stable id as `Same`; the smaller gets a fresh id and
    /// `Split { inherits }` pointing at the same prior.
    #[test]
    fn split_gives_the_largest_current_the_inherited_id() {
        let old = build(b"old", ((0,0,0),(9,0,9)), 200);
        let prior_id = StableBuildId::seed("w", old.id);
        let prior = vec![PriorBuild { stable_id: prior_id, bbox: old.bbox, block_count: 200 }];

        // Two current builds each overlapping the old footprint enough to
        // pass threshold (using generous IoU here isn't the point; both
        // must independently clear the threshold against the same prior).
        let big = build(b"big", ((0,0,0),(9,0,9)), 150);
        let small = build(b"small", ((0,0,0),(4,0,4)), 50);
        let cur = vec![big.clone(), small.clone()];

        let out = match_snapshots(&cur, &prior, "w", 0.1);
        assert_eq!(out.len(), 2);

        let big_match = out.iter().find(|m| m.build_id == big.id).unwrap();
        let small_match = out.iter().find(|m| m.build_id == small.id).unwrap();

        assert_eq!(big_match.stable_id, prior_id, "largest current inherits the prior id");
        assert!(matches!(big_match.outcome, Outcome::Same(sid) if sid == prior_id));

        assert_ne!(small_match.stable_id, prior_id, "split-off current gets a fresh id");
        assert!(matches!(&small_match.outcome, Outcome::Split { inherits } if *inherits == prior_id));

        // Output sorted by build_id.
        assert!(out[0].build_id <= out[1].build_id);
    }

    /// Two priors both overlap a single current build (two old buildings
    /// merged into one). The current inherits the larger prior's stable
    /// id and records both priors in `from`, sorted.
    #[test]
    fn merge_records_all_priors() {
        let old_a = build(b"old_a", ((0,0,0),(4,0,9)), 40);
        let old_b = build(b"old_b", ((5,0,0),(9,0,9)), 90);
        let stable_a = StableBuildId::seed("w", old_a.id);
        let stable_b = StableBuildId::seed("w", old_b.id);
        let prior = vec![
            PriorBuild { stable_id: stable_a, bbox: old_a.bbox, block_count: 40 },
            PriorBuild { stable_id: stable_b, bbox: old_b.bbox, block_count: 90 },
        ];

        let merged = build(b"merged", ((0,0,0),(9,0,9)), 130);
        let cur = vec![merged.clone()];

        let out = match_snapshots(&cur, &prior, "w", 0.1);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].build_id, merged.id);

        // Largest block_count prior (old_b, 90) is inherited.
        assert_eq!(out[0].stable_id, stable_b);

        let mut expected_from = vec![stable_a, stable_b];
        expected_from.sort();
        match &out[0].outcome {
            Outcome::Merge { from } => assert_eq!(from, &expected_from),
            other => panic!("expected Merge, got {other:?}"),
        }
    }
}
