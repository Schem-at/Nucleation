//! Substrate vs. artificial.
//!
//! Two conditions, both required for substrate: the block is in the natural
//! palette, **and** it sits inside the substrate Y band. Palette alone would
//! misclassify a stone-brick house as ground; the band alone would misclassify
//! a buried redstone line.
//!
//! Deliberately decidable from a single block plus the pinned profile, with no
//! neighbour access — that locality is what lets tiles be segmented
//! independently on different machines.

use crate::block_state::BlockState;
use crate::world_segment::profile::WorldProfile;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockClass {
    Substrate,
    Artificial,
}

pub fn classify(state: &BlockState, y: i32, profile: &WorldProfile) -> BlockClass {
    let (lo, hi) = profile.substrate_y_band;
    let in_band = y >= lo && y <= hi;
    let natural = profile.substrate_palette.contains(state.get_name());
    if in_band && natural {
        BlockClass::Substrate
    } else {
        BlockClass::Artificial
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block_state::BlockState;
    use crate::world_segment::profile::WorldProfile;

    fn profile() -> WorldProfile {
        WorldProfile::new(
            ["minecraft:stone", "minecraft:dirt", "minecraft:bedrock"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            (-64, -50),
        )
    }

    #[test]
    fn natural_block_inside_the_band_is_substrate() {
        assert_eq!(
            classify(&BlockState::new("minecraft:stone"), -60, &profile()),
            BlockClass::Substrate
        );
    }

    #[test]
    fn natural_block_above_the_band_is_artificial() {
        // Someone placed stone at build height: that is a build, not ground.
        assert_eq!(
            classify(&BlockState::new("minecraft:stone"), 10, &profile()),
            BlockClass::Artificial
        );
    }

    #[test]
    fn non_natural_block_inside_the_band_is_artificial() {
        // A buried redstone line is still a build.
        assert_eq!(
            classify(&BlockState::new("minecraft:redstone_wire"), -60, &profile()),
            BlockClass::Artificial
        );
    }

    #[test]
    fn band_edges_are_inclusive() {
        let p = profile();
        assert_eq!(classify(&BlockState::new("minecraft:stone"), -64, &p), BlockClass::Substrate);
        assert_eq!(classify(&BlockState::new("minecraft:stone"), -50, &p), BlockClass::Substrate);
        assert_eq!(classify(&BlockState::new("minecraft:stone"), -49, &p), BlockClass::Artificial);
    }

    #[test]
    fn block_properties_do_not_affect_classification() {
        let p = profile();
        let plain = BlockState::new("minecraft:stone");
        let propped = BlockState::new("minecraft:stone").with_property("waterlogged", "true");
        assert_eq!(classify(&plain, -60, &p), classify(&propped, -60, &p));
    }
}
