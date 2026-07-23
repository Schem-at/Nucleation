//! World-level constants, derived once and then pinned.
//!
//! Anything that needs world-wide statistics belongs here and nowhere else:
//! a heuristic that needed global knowledge at segment time would silently
//! break shardability. Derive it once, pin it, commit it.
//!
//! Derivation from real world data is Plan 2. This module defines the pinned
//! artifact and its hash.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::world_segment::ids::ContentId;

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct WorldProfile {
    /// Block names considered natural terrain. `BTreeSet` so iteration — and
    /// therefore the profile hash — is order-independent.
    pub substrate_palette: BTreeSet<String>,
    /// Inclusive `(min_y, max_y)` band within which natural blocks are ground.
    pub substrate_y_band: (i32, i32),
}

impl WorldProfile {
    pub fn new(substrate_palette: BTreeSet<String>, substrate_y_band: (i32, i32)) -> Self {
        WorldProfile { substrate_palette, substrate_y_band }
    }

    /// Stable hash of the pinned profile. Recorded on every build so a run can
    /// prove which constants produced it.
    pub fn profile_hash(&self) -> ContentId {
        let mut parts: Vec<Vec<u8>> = vec![b"profile.v1".to_vec()];
        for name in &self.substrate_palette {
            parts.push(name.as_bytes().to_vec());
        }
        parts.push(self.substrate_y_band.0.to_le_bytes().to_vec());
        parts.push(self.substrate_y_band.1.to_le_bytes().to_vec());
        let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
        ContentId::of(&refs)
    }
}
