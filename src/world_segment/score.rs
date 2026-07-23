//! Per-build tiering. Pure: no cross-build comparison, so it shards trivially.
//! Every tier decision carries the signals that produced it, so a human can
//! see why. The machine orders the queue; it does not decide worth — `Debris`
//! is retained, never deleted.

use serde::{Deserialize, Serialize};

use crate::world_segment::ids::ClusterId;
use crate::world_segment::stitch::Build;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Tier { Confident, Probable, Debris }

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Signal { pub name: String, pub value: f64 }

#[derive(Clone, Debug)]
pub struct ScoreConfig {
    pub debris_max_blocks: u64,
    pub confident_min_blocks: u64,
    pub confident_min_density: f64,
}

impl Default for ScoreConfig {
    fn default() -> Self {
        ScoreConfig { debris_max_blocks: 100, confident_min_blocks: 1000, confident_min_density: 0.02 }
    }
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Scored { pub build_id: ClusterId, pub tier: Tier, pub signals: Vec<Signal> }

fn volume(bbox: ((i32,i32,i32),(i32,i32,i32))) -> u64 {
    let dx = (bbox.1.0 - bbox.0.0 + 1).max(1) as u64;
    let dy = (bbox.1.1 - bbox.0.1 + 1).max(1) as u64;
    let dz = (bbox.1.2 - bbox.0.2 + 1).max(1) as u64;
    dx * dy * dz
}

pub fn score(build: &Build, cfg: &ScoreConfig) -> Scored {
    let vol = volume(build.bbox);
    let density = build.block_count as f64 / vol as f64;
    let signals = vec![
        Signal { name: "block_count".into(), value: build.block_count as f64 },
        Signal { name: "cell_count".into(), value: build.cell_count as f64 },
        Signal { name: "bbox_volume".into(), value: vol as f64 },
        Signal { name: "density".into(), value: density },
        Signal { name: "cluster_count".into(), value: build.cluster_ids.len() as f64 },
    ];
    let tier = if build.block_count <= cfg.debris_max_blocks {
        Tier::Debris
    } else if build.block_count >= cfg.confident_min_blocks && density >= cfg.confident_min_density {
        Tier::Confident
    } else {
        Tier::Probable
    };
    Scored { build_id: build.id, tier, signals }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_segment::ids::{ClusterId, ContentId, TileId};
    use crate::world_segment::stitch::Build;

    fn build(blocks: u64, bbox: ((i32,i32,i32),(i32,i32,i32))) -> Build {
        let id = ClusterId::new(ContentId::of(&[b"x"]), TileId{x:0,z:0}, None, (0,0,0));
        Build { id, cluster_ids: vec![id], bbox, block_count: blocks, cell_count: blocks/4 + 1, partition_id: None }
    }

    fn cfg() -> ScoreConfig {
        ScoreConfig { debris_max_blocks: 20, confident_min_blocks: 500, confident_min_density: 0.05 }
    }

    #[test]
    fn tiny_clusters_are_debris() {
        let s = score(&build(5, ((0,0,0),(1,1,1))), &cfg());
        assert_eq!(s.tier, Tier::Debris);
    }

    #[test]
    fn large_dense_builds_are_confident() {
        // 1000 blocks in a 10x10x10 box (volume 1000) -> density 1.0.
        let s = score(&build(1000, ((0,0,0),(9,9,9))), &cfg());
        assert_eq!(s.tier, Tier::Confident);
    }

    #[test]
    fn large_but_sparse_builds_are_probable() {
        // 100 blocks spread across a 50x50x50 box -> density ~0.0008 < 0.05,
        // and 100 < confident_min_blocks -> Probable (above debris, below confident).
        let s = score(&build(100, ((0,0,0),(49,49,49))), &cfg());
        assert_eq!(s.tier, Tier::Probable);
    }

    #[test]
    fn signals_are_recorded_and_named() {
        let s = score(&build(1000, ((0,0,0),(9,9,9))), &cfg());
        assert!(s.signals.iter().any(|sg| sg.name == "block_count" && sg.value == 1000.0));
        assert!(s.signals.iter().any(|sg| sg.name == "density" && sg.value == 1.0));
    }
}
