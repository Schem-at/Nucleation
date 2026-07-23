//! Assemble a build's blocks into a local-origin schematic plus its provenance.
//! Pure: the caller supplies the exact blocks; this never reads the world.

use std::collections::BTreeMap;

use crate::universal_schematic::UniversalSchematic;
use crate::world_segment::ids::ContentId;
use crate::world_segment::provenance::{Provenance, StableBuildId};
use crate::world_segment::score::Tier;
use crate::world_segment::stitch::Build;
use crate::BlockState;

pub struct MaterializeCtx<'a> {
    pub source_id: &'a str,
    pub snapshot_id: &'a str,
    pub config_hash: ContentId,
    pub profile_hash: ContentId,
    pub extracted_at: i64,
}

pub fn materialize(
    build: &Build,
    blocks: &BTreeMap<(i32, i32, i32), BlockState>,
    tier: Tier,
    stable_id: StableBuildId,
    ctx: &MaterializeCtx,
) -> (UniversalSchematic, Provenance) {
    let min = build.bbox.0;
    let mut schem = UniversalSchematic::new(stable_id.to_string());
    // BTreeMap iteration is sorted → deterministic placement.
    for (&(x, y, z), state) in blocks.iter() {
        schem.set_block(x - min.0, y - min.1, z - min.2, state);
    }
    let fp = crate::fingerprint::fingerprint(&schem, &fingerprint_spec());
    let prov = Provenance {
        stable_build_id: stable_id,
        snapshot_build_id: build.id,
        source_id: ctx.source_id.to_string(),
        snapshot_id: ctx.snapshot_id.to_string(),
        world_bbox: build.bbox,
        origin_offset: min,
        block_count: build.block_count,
        cluster_count: build.cluster_ids.len() as u32,
        fingerprint: fp.0,
        tier,
        config_hash: ctx.config_hash,
        profile_hash: ctx.profile_hash,
        extracted_at: ctx.extracted_at,
    };
    (schem, prov)
}

/// The most exact fingerprint preset, so a content change always shows.
fn fingerprint_spec() -> crate::fingerprint::FingerprintSpec {
    // Most exact preset (verified to exist): block-entities on, no rotation
    // tolerance, so any content change bumps the fingerprint.
    crate::fingerprint::FingerprintSpec::exact()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use crate::world_segment::ids::{ClusterId, ContentId, TileId};
    use crate::world_segment::score::Tier;
    use crate::world_segment::stitch::Build;
    use crate::world_segment::provenance::StableBuildId;
    use crate::BlockState;

    fn build() -> Build {
        let id = ClusterId::new(ContentId::of(&[b"b"]), TileId{x:0,z:0}, None, (0,0,0));
        Build { id, cluster_ids: vec![id], bbox: ((10,-60,10),(11,-60,10)),
                block_count: 2, cell_count: 1, partition_id: None }
    }

    fn ctx() -> MaterializeCtx<'static> {
        MaterializeCtx { source_id: "w", snapshot_id: "s", config_hash: ContentId::of(&[b"c"]),
                         profile_hash: ContentId::of(&[b"p"]), extracted_at: 100 }
    }

    #[test]
    fn schematic_is_local_origin_normalized() {
        let mut blocks = BTreeMap::new();
        blocks.insert((10,-60,10), BlockState::new("minecraft:redstone_wire"));
        blocks.insert((11,-60,10), BlockState::new("minecraft:repeater"));
        let sid = StableBuildId::seed("w", build().id);
        let (schem, prov) = materialize(&build(), &blocks, Tier::Confident, sid, &ctx());
        // Block at world (10,-60,10) lands at local (0,0,0).
        assert_eq!(schem.get_block(0,0,0).map(|b| b.get_name().to_string()),
                   Some("minecraft:redstone_wire".to_string()));
        assert_eq!(schem.get_block(1,0,0).map(|b| b.get_name().to_string()),
                   Some("minecraft:repeater".to_string()));
        assert_eq!(prov.origin_offset, (10,-60,10));
        assert_eq!(prov.world_bbox, ((10,-60,10),(11,-60,10)));
        assert_eq!(prov.block_count, 2);
        assert_eq!(prov.tier, Tier::Confident);
    }

    #[test]
    fn materialize_is_deterministic() {
        let mut blocks = BTreeMap::new();
        blocks.insert((10,-60,10), BlockState::new("minecraft:redstone_wire"));
        blocks.insert((11,-60,10), BlockState::new("minecraft:repeater"));
        let sid = StableBuildId::seed("w", build().id);
        let (_, p1) = materialize(&build(), &blocks, Tier::Confident, sid, &ctx());
        let (_, p2) = materialize(&build(), &blocks, Tier::Confident, sid, &ctx());
        assert_eq!(p1, p2, "same inputs → identical provenance (incl. fingerprint)");
    }
}
