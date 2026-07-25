//! Content-derived identifiers.
//!
//! Every id here is a function of content alone. This is what makes the
//! pipeline reorderable and shardable: two workers processing the same tile in
//! different orders produce byte-identical ids.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A tile's position in the tile lattice. The lattice origin and pitch live in
/// `SegConfig`, so a `TileId` is meaningful only alongside its config.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct TileId {
    pub x: i32,
    pub z: i32,
}

/// A 128-bit truncated BLAKE3 digest. Truncation is fine here: these are
/// content addresses, not security tokens.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ContentId([u8; 16]);

impl ContentId {
    /// Hash a positional sequence of byte parts.
    ///
    /// Each part is length-prefixed so that `["ab", "c"]` and `["a", "bc"]`
    /// cannot collide.
    pub fn of(parts: &[&[u8]]) -> Self {
        let mut hasher = blake3::Hasher::new();
        for part in parts {
            hasher.update(&(part.len() as u64).to_le_bytes());
            hasher.update(part);
        }
        let mut out = [0u8; 16];
        out.copy_from_slice(&hasher.finalize().as_bytes()[..16]);
        ContentId(out)
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}

impl fmt::Display for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.0 {
            write!(f, "{b:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for ContentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentId({self})")
    }
}

/// Identifies a cluster within a tile.
///
/// Derived from four things, all of them content:
///
/// - `config`, the [`SegConfig::config_hash`] of the configuration and world
///   profile that produced the cluster;
/// - `tile`, which tile it was found in;
/// - `partition`, the partition it fell in, if any;
/// - `anchor_cell`, the cluster's *canonical anchor* — the lexicographically
///   smallest occupied cell it contains.
///
/// The anchor is a property of the cluster's contents, so it does not depend
/// on the order in which cells were visited.
///
/// `config` is folded in because an anchor cell is expressed in *cell* units:
/// the same tile segmented at `cell_size` 4 and at `cell_size` 8 can produce
/// the same anchor coordinates for entirely different clusters, and a
/// different substrate profile can change which blocks survive without moving
/// any anchor at all. Without the config hash those cases collide, which
/// poisons any cache keyed on `ClusterId`.
///
/// `partition` is folded in because under
/// [`PartitionPolicy::HardCut`](crate::world_segment::partition::PartitionPolicy::HardCut)
/// blocks are partitioned individually, so one cell may be occupied in two
/// partitions' grids at once and yield the same anchor in each. Those are
/// genuinely different clusters and must not share an id.
///
/// [`SegConfig::config_hash`]: crate::world_segment::segment::SegConfig::config_hash
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterId(pub ContentId);

impl ClusterId {
    pub fn new(
        config: ContentId,
        tile: TileId,
        partition: Option<&str>,
        anchor_cell: (i32, i32, i32),
    ) -> Self {
        // `None` and `Some("")` must not collide, so presence is its own byte
        // rather than being inferred from an empty name.
        let present = [u8::from(partition.is_some())];
        let name = partition.unwrap_or("").as_bytes();
        ClusterId(ContentId::of(&[
            b"cluster.v2",
            config.as_bytes(),
            &tile.x.to_le_bytes(),
            &tile.z.to_le_bytes(),
            &present,
            name,
            &anchor_cell.0.to_le_bytes(),
            &anchor_cell.1.to_le_bytes(),
            &anchor_cell.2.to_le_bytes(),
        ]))
    }
}

impl fmt::Display for ClusterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_id_is_stable_and_order_sensitive() {
        let a = ContentId::of(&[b"alpha", b"beta"]);
        let b = ContentId::of(&[b"alpha", b"beta"]);
        let c = ContentId::of(&[b"beta", b"alpha"]);
        assert_eq!(a, b, "same parts must give the same id");
        assert_ne!(a, c, "part order must matter (fields are positional)");
    }

    #[test]
    fn content_id_resists_concatenation_collisions() {
        // Without length framing, ["ab","c"] and ["a","bc"] would collide.
        assert_ne!(ContentId::of(&[b"ab", b"c"]), ContentId::of(&[b"a", b"bc"]));
    }

    #[test]
    fn cluster_id_derives_from_config_tile_partition_and_anchor() {
        let t = TileId { x: 3, z: -7 };
        let cfg = ContentId::of(&[b"config-a"]);
        let base = ClusterId::new(cfg, t, None, (1, 2, 3));

        assert_eq!(base, ClusterId::new(cfg, t, None, (1, 2, 3)), "same inputs, same id");
        assert_ne!(base, ClusterId::new(cfg, t, None, (1, 2, 4)), "anchor matters");
        assert_ne!(
            base,
            ClusterId::new(cfg, TileId { x: 4, z: -7 }, None, (1, 2, 3)),
            "tile matters"
        );
        assert_ne!(
            base,
            ClusterId::new(ContentId::of(&[b"config-b"]), t, None, (1, 2, 3)),
            "config matters"
        );
        assert_ne!(base, ClusterId::new(cfg, t, Some("left"), (1, 2, 3)), "partition matters");
        assert_ne!(
            ClusterId::new(cfg, t, Some("left"), (1, 2, 3)),
            ClusterId::new(cfg, t, Some("right"), (1, 2, 3)),
            "the partition name, not just its presence, matters"
        );
    }

    #[test]
    fn absent_and_empty_partition_names_do_not_collide() {
        // `None` is framed with its own presence byte, so it cannot be
        // confused with a caller who genuinely named a partition "".
        let t = TileId { x: 0, z: 0 };
        let cfg = ContentId::of(&[b"c"]);
        assert_ne!(ClusterId::new(cfg, t, None, (0, 0, 0)), ClusterId::new(cfg, t, Some(""), (0, 0, 0)));
    }
}
