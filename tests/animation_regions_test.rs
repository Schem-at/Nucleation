use nucleation::animation::{
    AnimationEffect, BuildAnimation, GizmoKind, OperationKind, OperationScope,
};

use nucleation::BoundingBox;
use nucleation::{block_entity::BlockEntity, block_position::BlockPosition, Entity};

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < 1e-4)
}

#[test]
fn region_rotation_records_exact_discrete_transform() {
    let mut animation = BuildAnimation::new("region");
    animation.set_default_effect(AnimationEffect::new(0.0));
    animation
        .schematic_mut()
        .create_region("wing".to_string(), (10, 0, 10), (11, 0, 10));
    animation.begin_group(None).unwrap();
    animation
        .set_block_in_region("wing", 10, 0, 10, "minecraft:stone")
        .unwrap();
    animation
        .set_block_in_region("wing", 11, 0, 10, "minecraft:oak_stairs[facing=east]")
        .unwrap();
    let group = animation.end_group().unwrap();

    animation.rotate_region_y("wing", 90, 1_000.0).unwrap();

    assert_eq!(
        animation.groups()[group as usize].blocks,
        vec![(10, 0, 10), (11, 0, 10)]
    );
    let final_group = 1;
    assert_eq!(
        animation.groups()[final_group as usize].blocks,
        vec![(10, 0, 10), (10, 0, 11)]
    );
    let start = animation.frame_at(0.0);
    let end = animation.frame_at(1_000.0);
    assert!(close(
        start.pose(group).unwrap().apply([10.0, 0.0, 10.0]),
        [10.0, 0.0, 10.0]
    ));
    assert!(close(
        start.pose(group).unwrap().apply([11.0, 0.0, 10.0]),
        [11.0, 0.0, 10.0]
    ));
    assert!(close(
        end.pose(group).unwrap().apply([11.0, 0.0, 10.0]),
        [10.0, 0.0, 11.0]
    ));
    assert_eq!(end.pose(group).unwrap().opacity, 0.0);
    assert_eq!(end.pose(final_group).unwrap().opacity, 1.0);
    assert_eq!(animation.operation_count(), 1);
    let receipts = animation.operation_receipts();
    assert_eq!(receipts[0].scope, OperationScope::Region("wing".into()));
    assert_eq!(
        receipts[0].kind,
        OperationKind::Rotate {
            axis: nucleation::animation::TransformAxis::Y,
            quarter_turns: 1,
        }
    );
    assert_eq!(receipts[0].pivot2, Some([20, 0, 20]));
    assert_eq!(receipts[0].final_pivot2, Some([20, 0, 20]));
    assert!(receipts[0].cells.iter().any(|cell| {
        cell.before_block.contains("facing=east") && cell.final_block.contains("facing=south")
    }));
    let receipt_json = animation.operations_json().unwrap();
    assert!(receipt_json.contains("\"scope\":\"region\""));
    let middle = animation.frame_at(500.0);
    assert_eq!(
        middle
            .gizmos
            .iter()
            .filter(|line| line.kind == GizmoKind::RegionBounds)
            .count(),
        12
    );
    assert!(middle
        .gizmos
        .iter()
        .any(|line| line.kind == GizmoKind::Pivot));
    assert_eq!(
        middle
            .gizmos
            .iter()
            .filter(|line| line.kind == GizmoKind::RotationArc)
            .count(),
        24
    );
}

#[test]
fn sequential_translation_and_whole_rotation_compose() {
    let mut animation = BuildAnimation::new("sequence");
    animation.set_default_effect(AnimationEffect::new(0.0));
    animation.set_block(1, 0, 0, "minecraft:stone").unwrap();
    animation.translate(4, 0, 2, 500.0).unwrap();
    animation.rotate_all_y(90, 500.0).unwrap();

    let final_group = 2;
    let final_position = animation.groups()[final_group].blocks[0];
    assert_eq!(animation.operation_count(), 2);
    assert_eq!(final_position, (5, 0, 2));
    assert!(close(
        animation
            .frame_at(0.0)
            .pose(0)
            .unwrap()
            .apply([1.0, 0.0, 0.0]),
        [1.0, 0.0, 0.0],
    ));
    let end = animation.frame_at(1_000.0);
    assert_eq!(end.pose(0).unwrap().opacity, 0.0);
    assert_eq!(end.pose(1).unwrap().opacity, 0.0);
    assert_eq!(end.pose(final_group as u32).unwrap().opacity, 1.0);
}

#[test]
fn region_flip_uses_its_exact_symmetry_plane() {
    let mut animation = BuildAnimation::new("flip");
    animation.set_default_effect(AnimationEffect::new(0.0));
    animation
        .schematic_mut()
        .create_region("wing".into(), (2, 0, 0), (4, 0, 0));
    animation.begin_group(None).unwrap();
    animation
        .set_block_in_region("wing", 2, 0, 0, "minecraft:stone")
        .unwrap();
    animation
        .set_block_in_region("wing", 4, 0, 0, "minecraft:dirt")
        .unwrap();
    animation.end_group().unwrap();
    animation.flip_region_x("wing", 400.0).unwrap();
    let start = animation
        .frame_at(0.0)
        .pose(0)
        .unwrap()
        .apply([4.0, 0.0, 0.0]);
    let end = animation
        .frame_at(400.0)
        .pose(0)
        .unwrap()
        .apply([4.0, 0.0, 0.0]);
    // Named-region transforms use the allocated region bounds, not tight
    // occupied bounds. Small dynamic regions expand in 64-block chunks, so
    // this region's X plane is (2 + 68) / 2 = 35.
    assert!(close(start, [4.0, 0.0, 0.0]));
    assert!(close(end, [66.0, 0.0, 0.0]));
    assert_eq!(animation.frame_at(400.0).pose(0).unwrap().opacity, 0.0);
    assert_eq!(animation.frame_at(400.0).pose(1).unwrap().opacity, 1.0);
}

#[test]
fn stamp_region_records_only_written_destinations_and_appears_at_anchor() {
    let mut source = nucleation::UniversalSchematic::new("source".into());
    source.create_region("module".into(), (5, 0, 5), (6, 0, 5));
    source
        .try_set_block_in_region_str("module", 5, 0, 5, "minecraft:stone")
        .unwrap();
    source
        .try_set_block_in_region_str("module", 6, 0, 5, "minecraft:dirt")
        .unwrap();

    let mut animation = BuildAnimation::new("stamp");
    animation.set_default_effect(AnimationEffect::new(0.0));
    animation
        .set_block(21, 0, 20, "minecraft:gold_block")
        .unwrap();
    let exclusions = vec!["minecraft:dirt".to_string()];
    animation
        .stamp_region(&source, "module", (20, 0, 20), &exclusions, 600.0)
        .unwrap();

    assert_eq!(animation.groups().last().unwrap().blocks, vec![(20, 0, 20)]);
    assert_eq!(
        animation
            .schematic()
            .get_block_string_in_region("Main", 21, 0, 20),
        Some("minecraft:gold_block".to_string())
    );
    let moving_group = animation.groups().len() as u32 - 2;
    let final_group = animation.groups().len() as u32 - 1;
    let receipt = &animation.operation_receipts()[0];
    let start_ms = receipt.start_ms;
    let end_ms = start_ms + receipt.duration_ms;
    assert_eq!(
        animation
            .frame_at(start_ms)
            .pose(moving_group)
            .unwrap()
            .opacity,
        1.0
    );
    assert_eq!(
        animation
            .frame_at(end_ms)
            .pose(moving_group)
            .unwrap()
            .opacity,
        0.0
    );
    assert_eq!(
        animation
            .frame_at(end_ms)
            .pose(final_group)
            .unwrap()
            .opacity,
        1.0
    );
    let midpoint_ms = receipt.start_ms + receipt.duration_ms * 0.5;
    assert!(animation
        .frame_at(midpoint_ms)
        .gizmos
        .iter()
        .any(|line| line.kind == GizmoKind::DestinationAnchor));
    assert!(animation
        .frame_at(midpoint_ms)
        .gizmos
        .iter()
        .any(|line| line.kind == GizmoKind::StampSourceBounds));
    assert!(animation
        .frame_at(midpoint_ms)
        .gizmos
        .iter()
        .any(|line| line.kind == GizmoKind::StampDestinationBounds));
    assert!(animation
        .frame_at(midpoint_ms)
        .gizmos
        .iter()
        .any(|line| line.kind == GizmoKind::ExcludedCell));
}

#[test]
fn invalid_rotation_does_not_mutate_or_record() {
    let mut animation = BuildAnimation::new("transactional");
    animation
        .schematic_mut()
        .create_region("wing".to_string(), (10, 0, 10), (11, 0, 10));
    animation
        .set_block_in_region("wing", 10, 0, 10, "minecraft:stone")
        .unwrap();
    let before = animation.schematic().clone();

    assert!(animation.rotate_region_y("wing", 45, 500.0).is_err());
    assert_eq!(animation.operation_count(), 0);
    assert_eq!(
        animation
            .schematic()
            .get_block_string_in_region("wing", 10, 0, 10),
        before.get_block_string_in_region("wing", 10, 0, 10)
    );
}

#[test]
fn stamp_box_records_merged_writes_exclusions_and_is_transactional() {
    let mut source = nucleation::UniversalSchematic::new("source".into());
    source
        .set_block_from_string(0, 0, 0, "minecraft:stone")
        .unwrap();
    source
        .try_set_block_in_region_str("module", 1, 0, 0, "minecraft:gold_block")
        .unwrap();
    source
        .try_set_block_in_region_str("module", 2, 0, 0, "minecraft:chest")
        .unwrap();
    source.get_region_mut("module").unwrap().set_block_entity(
        BlockPosition { x: 2, y: 0, z: 0 },
        BlockEntity::new("minecraft:chest".into(), (2, 0, 0)),
    );
    source.add_entity(Entity::new(
        "minecraft:armor_stand".into(),
        (0.25, 0.0, 0.25),
    ));

    let mut animation = BuildAnimation::new("destination");
    animation.set_block(100, 0, 0, "minecraft:barrel").unwrap();
    animation
        .set_block(101, 0, 0, "minecraft:diamond_block")
        .unwrap();
    animation.set_block(102, 0, 0, "minecraft:barrel").unwrap();
    animation.schematic_mut().set_block_entity(
        BlockPosition { x: 100, y: 0, z: 0 },
        BlockEntity::new("minecraft:barrel".into(), (100, 0, 0)),
    );
    animation.schematic_mut().set_block_entity(
        BlockPosition { x: 102, y: 0, z: 0 },
        BlockEntity::new("minecraft:barrel".into(), (102, 0, 0)),
    );
    let bounds = BoundingBox::new((0, 0, 0), (2, 0, 0));
    animation
        .stamp_box(
            &source,
            bounds.clone(),
            (100, 0, 0),
            &["minecraft:gold_block".to_string()],
            500.0,
        )
        .unwrap();

    assert_eq!(
        animation
            .schematic()
            .get_block_string_in_region("Main", 100, 0, 0),
        Some("minecraft:stone".to_string())
    );
    assert_eq!(
        animation
            .schematic()
            .get_block_string_in_region("Main", 101, 0, 0),
        Some("minecraft:diamond_block".to_string())
    );
    let receipts = animation.operation_receipts();
    assert_eq!(receipts[0].scope, OperationScope::StampBox);
    assert_eq!(receipts[0].cells.len(), 2);
    assert!(receipts[0]
        .cells
        .iter()
        .any(|cell| cell.final_position == [100, 0, 0]));
    assert!(receipts[0]
        .cells
        .iter()
        .any(|cell| cell.final_position == [102, 0, 0]));
    assert_eq!(receipts[0].excluded_cells, vec![[101, 0, 0]]);
    assert_eq!(receipts[0].block_entities.len(), 2);
    let cleared = receipts[0]
        .block_entities
        .iter()
        .find(|delta| delta.final_position == [100, 0, 0])
        .unwrap();
    assert!(cleared.source.is_none());
    assert_eq!(
        cleared.replaced_destination.as_ref().unwrap().id,
        "minecraft:barrel"
    );
    assert!(cleared.final_state.is_none());
    let replaced = receipts[0]
        .block_entities
        .iter()
        .find(|delta| delta.final_position == [102, 0, 0])
        .unwrap();
    assert_eq!(replaced.source.as_ref().unwrap().id, "minecraft:chest");
    assert_eq!(
        replaced.replaced_destination.as_ref().unwrap().id,
        "minecraft:barrel"
    );
    assert_eq!(replaced.final_state.as_ref().unwrap().id, "minecraft:chest");
    assert_eq!(receipts[0].entities.len(), 1);
    assert_eq!(receipts[0].entities[0].before.position, (0.25, 0.0, 0.25));
    assert_eq!(
        receipts[0].entities[0].final_state.position,
        (100.25, 0.0, 0.25)
    );

    let before_100 = animation
        .schematic()
        .get_block_string_in_region("Main", 100, 0, 0);
    let before_101 = animation
        .schematic()
        .get_block_string_in_region("Main", 101, 0, 0);
    let operation_count = animation.operation_count();
    assert!(animation
        .stamp_box(&source, bounds, (i32::MAX, 0, 0), &[], 500.0,)
        .is_err());
    assert_eq!(
        animation
            .schematic()
            .get_block_string_in_region("Main", 100, 0, 0),
        before_100
    );
    assert_eq!(
        animation
            .schematic()
            .get_block_string_in_region("Main", 101, 0, 0),
        before_101
    );
    assert_eq!(animation.operation_count(), operation_count);
}

#[test]
fn mixed_tracked_and_untracked_content_fails_closed() {
    let mut animation = BuildAnimation::new("mixed");
    animation.set_default_effect(AnimationEffect::new(0.0));
    animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
    animation
        .schematic_mut()
        .try_set_block_str(10, 0, 0, "minecraft:dirt")
        .unwrap();
    let groups = animation.groups();
    let operation_count = animation.operation_count();

    let error = animation.translate(1, 0, 0, 250.0).unwrap_err();
    assert!(error.contains("exact active-group coverage"));
    assert_eq!(
        animation
            .schematic()
            .get_block(0, 0, 0)
            .map(|block| block.get_name()),
        Some("minecraft:stone")
    );
    assert_eq!(
        animation
            .schematic()
            .get_block(10, 0, 0)
            .map(|block| block.get_name()),
        Some("minecraft:dirt")
    );
    assert_eq!(animation.groups(), groups);
    assert_eq!(animation.operation_count(), operation_count);
}

#[test]
fn large_coordinates_keep_exact_source_history_and_receipts() {
    const LARGE: i32 = 16_777_216;
    let mut translated = BuildAnimation::new("large-translate");
    translated.set_default_effect(AnimationEffect::new(0.0));
    translated
        .set_block(LARGE, 0, LARGE, "minecraft:stone")
        .unwrap();
    translated.translate(1, 0, 0, 250.0).unwrap();
    let receipt = &translated.operation_receipts()[0];
    assert_eq!(receipt.cells[0].before_position, [LARGE, 0, LARGE]);
    assert_eq!(receipt.cells[0].final_position, [LARGE + 1, 0, LARGE]);
    assert_eq!(translated.groups()[0].blocks, vec![(LARGE, 0, LARGE)]);

    let mut rotated = BuildAnimation::new("large-rotate");
    rotated.set_default_effect(AnimationEffect::new(0.0));
    rotated
        .set_block(LARGE, 0, LARGE, "minecraft:stone")
        .unwrap();
    rotated
        .set_block(LARGE + 1, 0, LARGE, "minecraft:dirt")
        .unwrap();
    rotated.rotate_y(90, 250.0).unwrap();
    let source_positions: std::collections::BTreeSet<_> = rotated.operation_receipts()[0]
        .cells
        .iter()
        .map(|cell| cell.before_position)
        .collect();
    assert_eq!(
        source_positions,
        [[LARGE, 0, LARGE], [LARGE + 1, 0, LARGE]].into()
    );
    assert_eq!(rotated.groups()[0].blocks, vec![(LARGE, 0, LARGE)]);
    assert_eq!(rotated.groups()[1].blocks, vec![(LARGE + 1, 0, LARGE)]);
}

#[test]
fn later_operations_preserve_retired_temporal_coordinates() {
    let mut animation = BuildAnimation::new("history");
    animation.set_default_effect(AnimationEffect::new(0.0));
    animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
    animation.translate(1, 0, 0, 300.0).unwrap();
    let retired = animation.groups()[0].blocks.clone();
    assert_eq!(retired, vec![(0, 0, 0)]);

    animation.rotate_all_y(90, 300.0).unwrap();

    assert_eq!(animation.groups()[0].blocks, retired);
    assert_eq!(
        animation.operation_receipts()[0].cells[0].before_position,
        [0, 0, 0]
    );
}

#[test]
fn non_loop_frames_include_authoritative_operation_endpoint() {
    let mut animation = BuildAnimation::new("endpoint");
    animation.set_default_effect(AnimationEffect::new(0.0));
    animation.set_block(0, 0, 0, "minecraft:stone").unwrap();
    animation.translate(1, 0, 0, 1_000.0).unwrap();
    let receipt = &animation.operation_receipts()[0];
    let endpoint = receipt.start_ms + receipt.duration_ms;

    let frames = animation.frames(30.0, 0.0);
    let final_frame = frames.last().unwrap();
    assert!((final_frame.time_ms - endpoint).abs() < 0.001);
    assert_eq!(final_frame.pose(0).unwrap().opacity, 0.0);
    assert_eq!(final_frame.pose(1).unwrap().opacity, 1.0);
}

#[test]
fn stamp_region_uses_entity_aware_authoritative_bounds() {
    let mut source = nucleation::UniversalSchematic::new("source".into());
    source
        .try_set_block_in_region_str("module", 10, 0, 0, "minecraft:stone")
        .unwrap();
    source
        .get_region_mut("module")
        .unwrap()
        .entities
        .push(Entity::new(
            "minecraft:armor_stand".into(),
            (0.25, 0.0, 0.25),
        ));

    let mut animation = BuildAnimation::new("stamp");
    animation
        .stamp_region(&source, "module", (100, 0, 0), &[], 500.0)
        .unwrap();

    assert_eq!(
        animation
            .schematic()
            .get_block_string_in_region("Main", 110, 0, 0),
        Some("minecraft:stone".into())
    );
    let receipt = &animation.operation_receipts()[0];
    assert_eq!(receipt.cells[0].final_position, [110, 0, 0]);
    assert_eq!(receipt.final_bounds.unwrap().min, [100, 0, 0]);
    assert_eq!(receipt.final_bounds.unwrap().max, [110, 0, 0]);
    assert_eq!(
        receipt.entities[0].final_state.position,
        (100.25, 0.0, 0.25)
    );
}

#[test]
fn entity_only_and_all_excluded_stamps_still_record_and_advance_time() {
    let mut entity_source = nucleation::UniversalSchematic::new("entities".into());
    entity_source
        .try_set_block_in_region_str("module", 0, 0, 0, "minecraft:air")
        .unwrap();
    entity_source
        .get_region_mut("module")
        .unwrap()
        .entities
        .push(Entity::new("minecraft:pig".into(), (2.25, 0.0, 0.25)));
    let mut animation = BuildAnimation::new("destination");
    animation
        .stamp_region(&entity_source, "module", (50, 0, 0), &[], 250.0)
        .unwrap();
    assert_eq!(animation.operation_count(), 1);
    assert!(animation.operation_receipts()[0].cells.is_empty());
    assert_eq!(animation.operation_receipts()[0].entities.len(), 1);

    let mut excluded_source = nucleation::UniversalSchematic::new("excluded".into());
    excluded_source
        .try_set_block_in_region_str("module", 0, 0, 0, "minecraft:dirt")
        .unwrap();
    animation
        .stamp_region(
            &excluded_source,
            "module",
            (60, 0, 0),
            &["minecraft:dirt".into()],
            300.0,
        )
        .unwrap();
    assert_eq!(animation.operation_count(), 2);
    let receipt = &animation.operation_receipts()[1];
    assert!(receipt.cells.is_empty());
    assert_eq!(receipt.excluded_cells, vec![[60, 0, 0]]);
    assert_eq!(receipt.start_ms, 250.0);
    assert_eq!(animation.duration_ms(), 550.0);
}

#[test]
fn transform_receipts_include_block_entities_and_entities() {
    let mut animation = BuildAnimation::new("contents");
    animation.set_block(0, 0, 0, "minecraft:chest").unwrap();
    animation.schematic_mut().set_block_entity(
        BlockPosition { x: 0, y: 0, z: 0 },
        BlockEntity::new("minecraft:chest".into(), (0, 0, 0)),
    );
    animation.schematic_mut().add_entity(Entity::new(
        "minecraft:armor_stand".into(),
        (0.25, 0.0, 0.25),
    ));

    animation.translate(5, 0, 0, 400.0).unwrap();

    let receipt = &animation.operation_receipts()[0];
    assert_eq!(receipt.block_entities.len(), 1);
    assert_eq!(receipt.block_entities[0].source_position, [0, 0, 0]);
    assert_eq!(receipt.block_entities[0].final_position, [5, 0, 0]);
    assert_eq!(
        receipt.block_entities[0]
            .final_state
            .as_ref()
            .unwrap()
            .position,
        (5, 0, 0)
    );
    assert_eq!(receipt.entities.len(), 1);
    assert_eq!(receipt.entities[0].before.position, (0.25, 0.0, 0.25));
    assert_eq!(receipt.entities[0].final_state.position, (5.25, 0.0, 0.25));
}

#[test]
fn transforms_fail_closed_for_unrecorded_model_content() {
    let mut animation = BuildAnimation::new("raw");
    animation
        .schematic_mut()
        .set_block_from_string(0, 0, 0, "minecraft:stone")
        .unwrap();
    let before = animation.schematic().clone();

    assert!(animation.translate(1, 0, 0, 100.0).is_err());
    assert_eq!(animation.operation_count(), 0);
    assert_eq!(
        animation
            .schematic()
            .get_block_string_in_region("Main", 0, 0, 0),
        before.get_block_string_in_region("Main", 0, 0, 0)
    );
}

#[test]
fn rotated_flipped_region_stamp_rebuilds_tight_bounds_and_preserves_cells() {
    let mut source = nucleation::UniversalSchematic::new("asymmetric-source".into());
    source.add_region(nucleation::Region::new(
        "module".into(),
        (0, 0, 0),
        (5, 4, 3),
    ));
    for x in 0..5 {
        for y in 0..4 {
            for z in 0..3 {
                source
                    .try_set_block_in_region_str("module", x, y, z, "minecraft:stone")
                    .unwrap();
            }
        }
    }

    source.rotate_region_y("module", 90).unwrap();
    source.flip_region_x("module").unwrap();

    let tight = source
        .get_region("module")
        .unwrap()
        .get_tight_bounds()
        .unwrap();
    assert_eq!(tight.get_dimensions(), (3, 4, 5));

    let mut animation = BuildAnimation::new("stamp-target");
    animation
        .stamp_region(&source, "module", (10, 0, 10), &[], 500.0)
        .unwrap();
    let receipt = animation.operation_receipts().remove(0);
    assert_eq!(receipt.cells.len(), 60);
    assert_eq!(
        receipt.before_bounds.unwrap(),
        nucleation::animation::OperationBounds {
            min: [0, 0, 0],
            max: [2, 3, 4],
        }
    );
}

#[test]
fn rotation_gizmo_uses_the_min_anchor_block_center() {
    let mut animation = BuildAnimation::new("pivot");
    animation
        .create_region("module", (-8, 0, 0), (-6, 1, 0))
        .unwrap();
    animation.begin_group(None).unwrap();
    for x in -8..=-6 {
        animation
            .set_block_in_region("module", x, 0, 0, "minecraft:stone")
            .unwrap();
    }
    animation
        .set_block_in_region("module", -8, 1, 0, "minecraft:sea_lantern")
        .unwrap();
    animation.end_group().unwrap();
    animation.rotate_region_x("module", 90, 1_000.0).unwrap();

    let receipt = &animation.operation_receipts()[0];
    assert_eq!(receipt.pivot2, Some([-16, 0, 0]));
    let middle = animation.frame_at(receipt.start_ms + 500.0);
    let axis = middle
        .gizmos
        .iter()
        .find(|line| line.kind == GizmoKind::Pivot)
        .unwrap();
    assert_eq!(axis.start[1], 0.0);
    assert_eq!(axis.start[2], 0.0);
    assert_eq!(axis.end[1], 0.0);
    assert_eq!(axis.end[2], 0.0);
}

#[test]
fn animation_region_creation_rejects_duplicates_default_and_excessive_volume() {
    let mut animation = BuildAnimation::new("safe-region-creation");
    animation
        .set_block_in_region("Main", 0, 0, 0, "minecraft:diamond_block")
        .unwrap();

    assert!(animation
        .create_region("Main", (0, 0, 0), (1, 1, 1))
        .is_err());
    assert_eq!(
        animation
            .schematic()
            .get_block_string_in_region("Main", 0, 0, 0),
        Some("minecraft:diamond_block".to_string())
    );

    animation
        .create_region("module", (10, 0, 10), (11, 1, 11))
        .unwrap();
    let original_bounds = animation
        .schematic()
        .get_region("module")
        .unwrap()
        .get_bounding_box();
    assert!(animation
        .create_region("module", (20, 0, 20), (21, 1, 21))
        .is_err());
    assert_eq!(
        animation
            .schematic()
            .get_region("module")
            .unwrap()
            .get_bounding_box(),
        original_bounds
    );

    assert!(animation
        .create_region("huge", (0, 0, 0), (50_000, 50_000, 50_000))
        .is_err());
    assert!(!animation.schematic().has_region("huge"));
}
