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
/// Derived from the tile plus the cluster's *canonical anchor* — the
/// lexicographically smallest occupied cell it contains. The anchor is a
/// property of the cluster's contents, so it does not depend on the order in
/// which cells were visited.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct ClusterId(pub ContentId);

impl ClusterId {
    pub fn new(tile: TileId, anchor_cell: (i32, i32, i32)) -> Self {
        ClusterId(ContentId::of(&[
            b"cluster.v1",
            &tile.x.to_le_bytes(),
            &tile.z.to_le_bytes(),
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
    fn cluster_id_derives_from_tile_and_anchor_only() {
        let t = TileId { x: 3, z: -7 };
        assert_eq!(ClusterId::new(t, (1, 2, 3)), ClusterId::new(t, (1, 2, 3)));
        assert_ne!(ClusterId::new(t, (1, 2, 3)), ClusterId::new(t, (1, 2, 4)));
        assert_ne!(
            ClusterId::new(t, (1, 2, 3)),
            ClusterId::new(TileId { x: 4, z: -7 }, (1, 2, 3))
        );
    }
}
