//! The provenance envelope: where a build came from, stored outside the
//! schematic binary so an external resolver can later map it to ownership
//! without parsing blocks.
//!
//! `extracted_at` is an INPUT (unix seconds), never read from the clock —
//! reading the clock would make output non-deterministic and defeat the
//! "re-run a shard and compare hashes" verification.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::world_segment::ids::{ClusterId, ContentId};
use crate::world_segment::score::Tier;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct StableBuildId(pub ContentId);

impl StableBuildId {
    /// Deterministic seed for a build with no prior match: hash the opaque
    /// source id together with the snapshot ClusterId.
    pub fn seed(source_id: &str, snapshot_build: ClusterId) -> Self {
        StableBuildId(ContentId::of(&[
            b"stable.v1",
            source_id.as_bytes(),
            snapshot_build.0.as_bytes(),
        ]))
    }
}

impl fmt::Display for StableBuildId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub stable_build_id: StableBuildId,
    pub snapshot_build_id: ClusterId,
    pub source_id: String,
    pub snapshot_id: String,
    pub world_bbox: ((i32, i32, i32), (i32, i32, i32)),
    pub origin_offset: (i32, i32, i32),
    pub block_count: u64,
    pub cluster_count: u32,
    pub fingerprint: u128,
    pub tier: Tier,
    pub config_hash: ContentId,
    pub profile_hash: ContentId,
    /// Unix seconds, passed in by the caller — never the system clock.
    pub extracted_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_segment::ids::{ClusterId, ContentId, TileId};

    fn cid() -> ClusterId {
        ClusterId::new(ContentId::of(&[b"c"]), TileId { x: 0, z: 0 }, None, (0,0,0))
    }

    #[test]
    fn seed_is_deterministic_and_source_scoped() {
        let a = StableBuildId::seed("world_a", cid());
        let b = StableBuildId::seed("world_a", cid());
        let c = StableBuildId::seed("world_b", cid());
        assert_eq!(a, b, "same source + build → same id");
        assert_ne!(a, c, "different source → different id");
    }

    #[test]
    fn provenance_round_trips_through_serde() {
        let p = Provenance {
            stable_build_id: StableBuildId::seed("w", cid()),
            snapshot_build_id: cid(),
            source_id: "w".into(),
            snapshot_id: "2026-05-19".into(),
            world_bbox: ((0,0,0),(9,9,9)),
            origin_offset: (0,0,0),
            block_count: 100,
            cluster_count: 1,
            fingerprint: 42,
            tier: crate::world_segment::score::Tier::Confident,
            config_hash: ContentId::of(&[b"cfg"]),
            profile_hash: ContentId::of(&[b"prof"]),
            extracted_at: 1_700_000_000,
        };
        let bytes = serde_json::to_vec(&p).unwrap();
        let back: Provenance = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(p, back);
    }
}
