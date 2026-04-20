//! Tests for the bug: adding an entity outside the schematic's block
//! bounding box does not expand the tight bounds used by exporters, so
//! Litematica filters these entities out on load.
//!
//! Litematic export uses `Region::to_compact()` which drops any entity whose
//! position falls outside `tight_bounds`. If `add_entity` doesn't update
//! `tight_bounds`, the entity is silently lost in the exported file.

use nucleation::block_entity::BlockEntity;
use nucleation::{litematic, schematic, BlockState, Entity, UniversalSchematic};

fn covers(bb_min: (i32, i32, i32), bb_max: (i32, i32, i32), p: (i32, i32, i32)) -> bool {
    p.0 >= bb_min.0
        && p.0 <= bb_max.0
        && p.1 >= bb_min.1
        && p.1 <= bb_max.1
        && p.2 >= bb_min.2
        && p.2 <= bb_max.2
}

/// Baseline: adding a block expands tight_bounds.
#[test]
fn block_outside_bounds_expands_tight_bounds() {
    let mut schem = UniversalSchematic::new("Test".to_string());
    schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));
    schem.set_block(10, 5, 8, &BlockState::new("minecraft:stone".to_string()));

    let tight = schem.get_content_bounds().expect("has blocks");
    assert_eq!(tight.max, (10, 5, 8));
}

/// Adding an entity outside block bounds must expand tight_bounds so export
/// keeps the entity.
#[test]
fn entity_outside_bounds_expands_tight_bounds() {
    let mut schem = UniversalSchematic::new("Test".to_string());
    schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));

    let creeper = Entity::new("minecraft:creeper".to_string(), (50.5, 40.0, 30.5));
    assert!(schem.add_entity(creeper));

    let tight = schem.get_content_bounds().expect("tight bounds present");
    assert!(
        covers(tight.min, tight.max, (50, 40, 30)),
        "entity at (50,40,30) outside tight_bounds {:?}",
        tight
    );
}

/// Entity at negative coords must also expand tight_bounds.
#[test]
fn entity_at_negative_coords_expands_tight_bounds() {
    let mut schem = UniversalSchematic::new("Test".to_string());
    schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));

    let zombie = Entity::new("minecraft:zombie".to_string(), (-20.5, -5.0, -15.5));
    assert!(schem.add_entity(zombie));

    let tight = schem.get_content_bounds().expect("tight bounds present");
    assert!(
        covers(tight.min, tight.max, (-20, -5, -15)),
        "entity at (-20,-5,-15) outside tight_bounds {:?}",
        tight
    );
}

/// An entity-only schematic (no blocks) still needs tight_bounds covering the entity.
#[test]
fn entity_only_schematic_has_tight_bounds_covering_entity() {
    let mut schem = UniversalSchematic::new("Test".to_string());

    let villager = Entity::new("minecraft:villager".to_string(), (7.5, 3.0, 2.5));
    assert!(schem.add_entity(villager));

    let tight = schem
        .get_content_bounds()
        .expect("entity-only schematic should have tight bounds");
    assert!(
        covers(tight.min, tight.max, (7, 3, 2)),
        "entity at (7,3,2) outside tight_bounds {:?}",
        tight
    );
}

/// End-to-end: an entity outside the block bounds must survive a litematic
/// export/import round-trip. This is the user-facing manifestation of the bug
/// (Litematica filters such entities on load).
#[test]
fn entity_outside_bounds_survives_litematic_roundtrip() {
    let mut schem = UniversalSchematic::new("Test".to_string());
    schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));

    let creeper = Entity::new("minecraft:creeper".to_string(), (50.5, 40.0, 30.5));
    assert!(schem.add_entity(creeper));

    let bytes = litematic::to_litematic(&schem).expect("export");
    let reloaded = litematic::from_litematic(&bytes).expect("import");

    let entities: Vec<_> = reloaded
        .get_all_regions()
        .into_iter()
        .flat_map(|(_, r)| r.entities.clone())
        .collect();

    assert!(
        entities.iter().any(|e| e.id == "minecraft:creeper"),
        "creeper was dropped during litematic round-trip; entities found: {:?}",
        entities.iter().map(|e| &e.id).collect::<Vec<_>>()
    );
}

/// Parallel bug check: a block entity outside block tight_bounds used to be
/// filtered out in `to_compact`. get_content_bounds should include its
/// position so the exported region is large enough to keep it.
#[test]
fn block_entity_outside_bounds_expands_content_bounds() {
    let mut schem = UniversalSchematic::new("Test".to_string());
    schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));

    // Block entity position outside the 1x1x1 block region.
    let be = BlockEntity::new("minecraft:chest".to_string(), (25, 10, 33));
    assert!(schem.add_block_entity(be));

    let content = schem.get_content_bounds().expect("content bounds present");
    assert!(
        covers(content.min, content.max, (25, 10, 33)),
        "block entity at (25,10,33) outside content_bounds {:?}",
        content
    );
}

/// Entity outside block bounds must survive .schem round-trip too. Exercises
/// both the bbox fix AND the Sponge v3 entity wrapper (`{Id, Pos, Data}`).
#[test]
fn entity_outside_bounds_survives_schem_roundtrip() {
    let mut schem = UniversalSchematic::new("Test".to_string());
    schem.set_block(0, 0, 0, &BlockState::new("minecraft:stone".to_string()));

    let skeleton = Entity::new("minecraft:skeleton".to_string(), (40.5, 20.0, 15.5));
    assert!(schem.add_entity(skeleton));

    let bytes = schematic::to_schematic(&schem).expect("export");
    let reloaded = schematic::from_schematic(&bytes).expect("import");

    let entities: Vec<_> = reloaded
        .get_all_regions()
        .into_iter()
        .flat_map(|(_, r)| r.entities.clone())
        .collect();

    assert!(
        entities.iter().any(|e| e.id == "minecraft:skeleton"),
        "skeleton was dropped during .schem round-trip; entities found: {:?}",
        entities.iter().map(|e| &e.id).collect::<Vec<_>>()
    );
}
