//! Assemble a build's blocks into a local-origin schematic plus its provenance.
//! Pure: the caller supplies the exact blocks; this never reads the world.

use std::collections::BTreeMap;

use crate::block_entity::BlockEntity;
use crate::block_position::BlockPosition;
use crate::universal_schematic::UniversalSchematic;
use crate::world_segment::ids::ContentId;
use crate::world_segment::provenance::{Provenance, StableBuildId};
use crate::world_segment::score::Tier;
use crate::world_segment::stitch::Build;
use crate::BlockState;
use crate::{ProvenanceBounds, SchematicProvenance};

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
    materialize_with_block_entities(build, blocks, &BTreeMap::new(), tier, stable_id, ctx)
}

/// Materialize a build while retaining NBT-bearing blocks. Block-entity keys
/// and their embedded positions are translated from world space to the
/// schematic's local origin alongside the block states.
pub fn materialize_with_block_entities(
    build: &Build,
    blocks: &BTreeMap<(i32, i32, i32), BlockState>,
    block_entities: &BTreeMap<(i32, i32, i32), BlockEntity>,
    tier: Tier,
    stable_id: StableBuildId,
    ctx: &MaterializeCtx,
) -> (UniversalSchematic, Provenance) {
    debug_assert!(
        blocks.len() as u64 == build.block_count,
        "materialize: {} blocks vs build.block_count {}",
        blocks.len(),
        build.block_count
    );
    let min = build.bbox.0;
    let mut schem = UniversalSchematic::new(stable_id.to_string());
    // Allocate the final dense region once. Growing it for each placed block
    // repeatedly copies the volume, which is especially costly on large builds.
    schem.set_default_region(crate::region::Region::new(
        "Main".to_string(),
        (0, 0, 0),
        (
            build.bbox.1 .0 - min.0 + 1,
            build.bbox.1 .1 - min.1 + 1,
            build.bbox.1 .2 - min.2 + 1,
        ),
    ));
    // BTreeMap iteration is sorted → deterministic placement.
    for (&(x, y, z), state) in blocks.iter() {
        schem.set_block(x - min.0, y - min.1, z - min.2, state);
    }
    for (&(x, y, z), block_entity) in block_entities {
        if !blocks.contains_key(&(x, y, z)) {
            continue;
        }
        let local = (x - min.0, y - min.1, z - min.2);
        let mut translated = block_entity.clone();
        translated.position = local;
        schem.set_block_entity(
            BlockPosition {
                x: local.0,
                y: local.1,
                z: local.2,
            },
            translated,
        );
    }
    let fp = crate::fingerprint::fingerprint(&schem, &fingerprint_spec());
    let prov = Provenance {
        stable_build_id: stable_id,
        snapshot_build_id: build.id,
        source_id: ctx.source_id.to_string(),
        snapshot_id: ctx.snapshot_id.to_string(),
        world_bbox: build.bbox,
        origin_offset: min,
        partition_id: build.partition_id.clone(),
        // Provenance describes the schematic actually materialized from `blocks`,
        // not the build's nominal count; the debug_assert above is the dev-time
        // signal that a caller assembled `blocks` incorrectly for `build`.
        block_count: blocks.len() as u64,
        // Counts the build's member clusters; a cluster that contributed zero
        // blocks (e.g. fully subtracted) is still counted here. Benign.
        cluster_count: build.cluster_ids.len() as u32,
        fingerprint: fp.0,
        tier,
        config_hash: ctx.config_hash,
        profile_hash: ctx.profile_hash,
        extracted_at: ctx.extracted_at,
    };
    let mut embedded =
        SchematicProvenance::new(ctx.source_id).expect("world-segment source_id must be non-empty");
    embedded.snapshot_id = Some(ctx.snapshot_id.to_string());
    embedded.world_bbox = Some(
        ProvenanceBounds::new(
            [build.bbox.0 .0, build.bbox.0 .1, build.bbox.0 .2],
            [build.bbox.1 .0, build.bbox.1 .1, build.bbox.1 .2],
        )
        .expect("build bbox is ordered"),
    );
    embedded.origin = Some([min.0, min.1, min.2]);
    embedded.partition_id = build.partition_id.clone();
    embedded.stable_build_id = Some(stable_id.to_string());
    embedded.extracted_at = Some(ctx.extracted_at);
    embedded.config_hash = Some(ctx.config_hash.to_string());
    embedded.profile_hash = Some(ctx.profile_hash.to_string());
    schem.metadata.provenance = Some(embedded);
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
    use crate::world_segment::ids::{ClusterId, ContentId, TileId};
    use crate::world_segment::provenance::StableBuildId;
    use crate::world_segment::score::Tier;
    use crate::world_segment::stitch::Build;
    use crate::BlockState;
    use std::collections::BTreeMap;

    fn build() -> Build {
        let id = ClusterId::new(
            ContentId::of(&[b"b"]),
            TileId { x: 0, z: 0 },
            None,
            (0, 0, 0),
        );
        Build {
            id,
            cluster_ids: vec![id],
            bbox: ((10, -60, 10), (11, -60, 10)),
            block_count: 2,
            cell_count: 1,
            partition_id: None,
        }
    }

    fn ctx() -> MaterializeCtx<'static> {
        MaterializeCtx {
            source_id: "w",
            snapshot_id: "s",
            config_hash: ContentId::of(&[b"c"]),
            profile_hash: ContentId::of(&[b"p"]),
            extracted_at: 100,
        }
    }

    #[test]
    fn schematic_is_local_origin_normalized() {
        let mut blocks = BTreeMap::new();
        blocks.insert((10, -60, 10), BlockState::new("minecraft:redstone_wire"));
        blocks.insert((11, -60, 10), BlockState::new("minecraft:repeater"));
        let sid = StableBuildId::seed("w", build().id);
        let (schem, prov) = materialize(&build(), &blocks, Tier::Confident, sid, &ctx());
        // Block at world (10,-60,10) lands at local (0,0,0).
        assert_eq!(
            schem.get_block(0, 0, 0).map(|b| b.get_name().to_string()),
            Some("minecraft:redstone_wire".to_string())
        );
        assert_eq!(
            schem.get_block(1, 0, 0).map(|b| b.get_name().to_string()),
            Some("minecraft:repeater".to_string())
        );
        assert_eq!(prov.origin_offset, (10, -60, 10));
        assert_eq!(prov.world_bbox, ((10, -60, 10), (11, -60, 10)));
        assert_eq!(prov.block_count, 2);
        assert_eq!(prov.tier, Tier::Confident);
    }

    #[test]
    fn materialize_is_deterministic() {
        let mut blocks = BTreeMap::new();
        blocks.insert((10, -60, 10), BlockState::new("minecraft:redstone_wire"));
        blocks.insert((11, -60, 10), BlockState::new("minecraft:repeater"));
        let sid = StableBuildId::seed("w", build().id);
        let (_, p1) = materialize(&build(), &blocks, Tier::Confident, sid, &ctx());
        let (_, p2) = materialize(&build(), &blocks, Tier::Confident, sid, &ctx());
        assert_eq!(
            p1, p2,
            "same inputs → identical provenance (incl. fingerprint)"
        );
    }

    #[test]
    fn materialize_translates_and_preserves_block_entities() {
        let mut blocks = BTreeMap::new();
        blocks.insert((10, -60, 10), BlockState::new("minecraft:chest"));
        blocks.insert((11, -60, 10), BlockState::new("minecraft:repeater"));
        let mut block_entities = BTreeMap::new();
        block_entities.insert(
            (10, -60, 10),
            BlockEntity::new("minecraft:chest".to_string(), (10, -60, 10)),
        );
        let sid = StableBuildId::seed("w", build().id);
        let (schem, _) = materialize_with_block_entities(
            &build(),
            &blocks,
            &block_entities,
            Tier::Confident,
            sid,
            &ctx(),
        );
        let entities = schem.get_block_entities_as_list();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].position, (0, 0, 0));
        assert_eq!(entities[0].id, "minecraft:chest");
    }

    /// provenance.block_count tracks the materialized `blocks` map, not
    /// `build.block_count` — they happen to agree here (the debug_assert
    /// requires it), but the assertion is written against `blocks.len()` to
    /// document the actual source of truth. In release builds (where the
    /// debug_assert compiles out), a caller-supplied `blocks` map of a
    /// different length would still produce a correct `provenance.block_count`
    /// because it's derived from `blocks`, not from `build`.
    #[test]
    fn provenance_block_count_tracks_materialized_blocks_not_build() {
        let mut blocks = BTreeMap::new();
        blocks.insert((10, -60, 10), BlockState::new("minecraft:redstone_wire"));
        blocks.insert((11, -60, 10), BlockState::new("minecraft:repeater"));
        let sid = StableBuildId::seed("w", build().id);
        let (_, prov) = materialize(&build(), &blocks, Tier::Confident, sid, &ctx());
        assert_eq!(prov.block_count, blocks.len() as u64);
    }
}
