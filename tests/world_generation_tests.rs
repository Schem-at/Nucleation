use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use nucleation::building::{BrushEnum, SolidBrush};
use nucleation::sdf::SdfNode;
use nucleation::world_generation::{
    CellularSdfChunkSource, CellularSdfConfig, ChunkBounds, ChunkCoverage, ChunkOverlayMode,
    ChunkRequest, ChunkResult, ChunkSource, CompositeChunkSource, GeneratedChunkStream,
    ProjectedFootprintChunkSource, SdfChunkSource, SourceProvenance, WorldGenerationError,
};
use nucleation::BlockState;

fn stone_brush() -> BrushEnum {
    BrushEnum::Solid(SolidBrush::new(BlockState::new("minecraft:stone")))
}

#[test]
fn sdf_source_generates_requested_chunk_at_voxel_centers() {
    let source = SdfChunkSource::new(
        SdfNode::Sphere { radius: 2.0 },
        stone_brush(),
        -4,
        4,
        SourceProvenance::new("example:terrain", "v1").unwrap(),
    )
    .unwrap();

    let result = source.generate(ChunkRequest::new(0, 0)).unwrap();

    assert_eq!(result.coverage(), ChunkCoverage::Complete);
    assert_eq!(result.provenance().source_id(), "example:terrain");
    assert_eq!(result.provenance().version(), "v1");
    assert_eq!(result.chunk().cx(), 0);
    assert_eq!(result.chunk().cz(), 0);
    assert_eq!(
        result.chunk().get_block(0, 0, 0).unwrap().get_name(),
        "minecraft:stone"
    );
    assert_eq!(
        result.chunk().get_block(2, 0, 0).unwrap().get_name(),
        "minecraft:air"
    );
}

#[test]
fn sdf_source_rejects_reversed_y_bounds() {
    let result = SdfChunkSource::new(
        SdfNode::Sphere { radius: 2.0 },
        stone_brush(),
        5,
        4,
        SourceProvenance::new("example:terrain", "v1").unwrap(),
    );

    assert!(matches!(result, Err(WorldGenerationError::InvalidYBounds)));
}

#[test]
fn sdf_source_rejects_unrepresentable_section_bounds() {
    let result = SdfChunkSource::new(
        SdfNode::Sphere { radius: 2.0 },
        stone_brush(),
        (i8::MIN as i32) * 16 - 1,
        4,
        SourceProvenance::new("example:terrain", "v1").unwrap(),
    );

    assert!(matches!(
        result,
        Err(WorldGenerationError::YBoundsOutOfRange)
    ));
}

#[test]
fn sdf_source_rejects_invalid_sdf_before_generation() {
    let result = SdfChunkSource::new(
        SdfNode::Sphere { radius: f32::NAN },
        stone_brush(),
        -4,
        4,
        SourceProvenance::new("example:terrain", "v1").unwrap(),
    );

    assert!(matches!(result, Err(WorldGenerationError::InvalidSdf(_))));
}

#[test]
fn provenance_rejects_empty_identifiers() {
    assert!(matches!(
        SourceProvenance::new("", "v1"),
        Err(WorldGenerationError::InvalidProvenance)
    ));
    assert!(matches!(
        SourceProvenance::new("example:terrain", ""),
        Err(WorldGenerationError::InvalidProvenance)
    ));
}

#[test]
fn provenance_rejects_unbounded_metadata() {
    assert!(matches!(
        SourceProvenance::new("x".repeat(257), "v1"),
        Err(WorldGenerationError::InvalidProvenance)
    ));
}

#[test]
fn generation_rejects_chunk_coordinates_that_overflow_world_blocks() {
    let source = SdfChunkSource::new(
        SdfNode::Sphere { radius: 2.0 },
        stone_brush(),
        -4,
        4,
        SourceProvenance::new("example:terrain", "v1").unwrap(),
    )
    .unwrap();

    assert!(matches!(
        source.generate(ChunkRequest::new(i32::MAX, 0)),
        Err(WorldGenerationError::CoordinateOverflow)
    ));
}

#[test]
fn sdf_source_rejects_coordinates_without_exact_voxel_centers() {
    let source = SdfChunkSource::new(
        SdfNode::Sphere { radius: 2.0 },
        stone_brush(),
        -4,
        4,
        SourceProvenance::new("example:terrain", "v1").unwrap(),
    )
    .unwrap();

    assert!(matches!(
        source.generate(ChunkRequest::new(524_288, 0)),
        Err(WorldGenerationError::CoordinatePrecision)
    ));
}

#[test]
fn sdf_source_uses_absolute_coordinates_across_negative_chunk_boundaries() {
    let volume = SdfNode::Translate {
        child: Box::new(SdfNode::Sphere { radius: 2.0 }),
        offset: [-0.5, 0.0, 0.0],
    };
    let source = SdfChunkSource::new(
        volume,
        stone_brush(),
        -4,
        4,
        SourceProvenance::new("example:terrain", "v1").unwrap(),
    )
    .unwrap();

    let west = source.generate(ChunkRequest::new(-1, 0)).unwrap();
    let east = source.generate(ChunkRequest::new(0, 0)).unwrap();

    assert_eq!(
        west.chunk().get_block(-1, 0, 0).unwrap().get_name(),
        "minecraft:stone"
    );
    assert_eq!(
        east.chunk().get_block(0, 0, 0).unwrap().get_name(),
        "minecraft:stone"
    );
}

struct PartialMarkerSource;

impl ChunkSource for PartialMarkerSource {
    fn generate(&self, request: ChunkRequest) -> Result<ChunkResult, WorldGenerationError> {
        let mut chunk = nucleation::world_stream::WorldChunkView::new(request.cx(), request.cz());
        chunk.set_block(
            request.cx() * 16,
            5,
            request.cz() * 16,
            &BlockState::new("minecraft:gold_block"),
        );
        Ok(ChunkResult::new(
            chunk,
            ChunkCoverage::Partial,
            SourceProvenance::new("example:osm", "2026-07-25")?,
        ))
    }
}

#[test]
fn custom_source_can_return_partial_coverage() {
    let result = PartialMarkerSource
        .generate(ChunkRequest::new(-1, 2))
        .unwrap();

    assert_eq!(result.coverage(), ChunkCoverage::Partial);
    assert_eq!(result.chunk().cx(), -1);
    assert_eq!(result.provenance().source_id(), "example:osm");
}

struct MarkerSource {
    block: &'static str,
    coverage: ChunkCoverage,
}

impl ChunkSource for MarkerSource {
    fn generate(&self, request: ChunkRequest) -> Result<ChunkResult, WorldGenerationError> {
        let mut chunk = nucleation::world_stream::WorldChunkView::new(request.cx(), request.cz());
        chunk.set_block(
            request.cx() * 16,
            5,
            request.cz() * 16,
            &BlockState::new(self.block),
        );
        Ok(ChunkResult::new(
            chunk,
            self.coverage,
            SourceProvenance::new(self.block, "v1")?,
        ))
    }
}

#[test]
fn composite_source_applies_ordered_overlays_and_combines_coverage() {
    let mut source =
        CompositeChunkSource::new(SourceProvenance::new("example:composite", "v1").unwrap());
    source
        .add_layer(
            Arc::new(MarkerSource {
                block: "minecraft:stone",
                coverage: ChunkCoverage::Complete,
            }),
            ChunkOverlayMode::Replace,
        )
        .unwrap();
    source
        .add_layer(
            Arc::new(MarkerSource {
                block: "minecraft:gold_block",
                coverage: ChunkCoverage::Partial,
            }),
            ChunkOverlayMode::Replace,
        )
        .unwrap();

    let result = source.generate(ChunkRequest::new(-1, 2)).unwrap();

    assert_eq!(result.coverage(), ChunkCoverage::Complete);
    assert_eq!(result.provenance().source_id(), "example:composite");
    assert_eq!(
        result.chunk().get_block(-16, 5, 32).unwrap().get_name(),
        "minecraft:gold_block"
    );
}

struct CountingSource {
    calls: Arc<AtomicUsize>,
}

impl ChunkSource for CountingSource {
    fn generate(&self, request: ChunkRequest) -> Result<ChunkResult, WorldGenerationError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ChunkResult::new(
            nucleation::world_stream::WorldChunkView::new(request.cx(), request.cz()),
            ChunkCoverage::Complete,
            SourceProvenance::new("example:counting", "v1")?,
        ))
    }
}

#[test]
fn generated_stream_is_lazy_and_uses_canonical_region_major_order() {
    let calls = Arc::new(AtomicUsize::new(0));
    let source: Arc<dyn ChunkSource> = Arc::new(CountingSource {
        calls: calls.clone(),
    });
    let bounds = ChunkBounds::new(31, 31, 32, 32).unwrap();
    let mut stream = GeneratedChunkStream::new(source, bounds);

    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let mut coordinates = Vec::new();
    while let Some(result) = stream.next() {
        let result = result.unwrap();
        coordinates.push((result.chunk().cx(), result.chunk().cz()));
        assert_eq!(calls.load(Ordering::SeqCst), coordinates.len());
    }

    let mut expected = vec![(31, 31), (31, 32), (32, 31), (32, 32)];
    expected.sort_by_key(|&(cx, cz)| nucleation::world_stream::chunk_order_key(cx, cz));
    assert_eq!(coordinates, expected);
    assert_eq!(stream.remaining(), 0);
}

#[test]
fn composite_source_rejects_unbounded_layer_graphs() {
    let mut source =
        CompositeChunkSource::new(SourceProvenance::new("example:composite", "v1").unwrap());
    for _ in 0..64 {
        source
            .add_layer(
                Arc::new(MarkerSource {
                    block: "minecraft:stone",
                    coverage: ChunkCoverage::Partial,
                }),
                ChunkOverlayMode::Replace,
            )
            .unwrap();
    }

    assert!(matches!(
        source.add_layer(
            Arc::new(MarkerSource {
                block: "minecraft:gold_block",
                coverage: ChunkCoverage::Partial,
            }),
            ChunkOverlayMode::Replace,
        ),
        Err(WorldGenerationError::TooManySourceLayers)
    ));
}

#[test]
fn projected_footprint_source_is_random_access_and_reports_sparse_coverage() {
    let source = ProjectedFootprintChunkSource::new(
        vec![nucleation::geo::Footprint {
            polygon: vec![(15.0, 0.0), (18.0, 0.0), (18.0, 3.0), (15.0, 3.0)],
            y_min: 1,
            y_max: 4,
            block: "minecraft:bricks".to_string(),
        }],
        None,
        SourceProvenance::new("example:osm", "2026-07-25").unwrap(),
    )
    .unwrap();

    let west = source.generate(ChunkRequest::new(0, 0)).unwrap();
    let east = source.generate(ChunkRequest::new(1, 0)).unwrap();
    let outside = source.generate(ChunkRequest::new(2, 0)).unwrap();

    assert_eq!(west.coverage(), ChunkCoverage::Partial);
    assert_eq!(east.coverage(), ChunkCoverage::Partial);
    assert_eq!(outside.coverage(), ChunkCoverage::Outside);
    assert_eq!(
        west.chunk().get_block(15, 2, 1).unwrap().get_name(),
        "minecraft:bricks"
    );
    assert_eq!(
        east.chunk().get_block(16, 2, 1).unwrap().get_name(),
        "minecraft:bricks"
    );
}

#[test]
fn projected_footprint_index_handles_region_boundaries_and_giant_fallbacks() {
    let source = ProjectedFootprintChunkSource::new(
        vec![
            nucleation::geo::Footprint {
                polygon: vec![(511.0, 0.0), (514.0, 0.0), (514.0, 3.0), (511.0, 3.0)],
                y_min: 1,
                y_max: 2,
                block: "minecraft:bricks".to_string(),
            },
            nucleation::geo::Footprint {
                polygon: vec![
                    (0.0, 100.0),
                    (2_100_000.0, 100.0),
                    (2_100_000.0, 102.0),
                    (0.0, 102.0),
                ],
                y_min: 1,
                y_max: 2,
                block: "minecraft:gold_block".to_string(),
            },
        ],
        None,
        SourceProvenance::new("example:projected", "v1").unwrap(),
    )
    .unwrap();

    assert_eq!(
        source
            .generate(ChunkRequest::new(31, 0))
            .unwrap()
            .chunk()
            .get_block(511, 1, 1)
            .unwrap()
            .get_name(),
        "minecraft:bricks"
    );
    assert_eq!(
        source
            .generate(ChunkRequest::new(32, 0))
            .unwrap()
            .chunk()
            .get_block(512, 1, 1)
            .unwrap()
            .get_name(),
        "minecraft:bricks"
    );
    assert_eq!(
        source
            .generate(ChunkRequest::new(100_000, 6))
            .unwrap()
            .chunk()
            .get_block(1_600_000, 1, 101)
            .unwrap()
            .get_name(),
        "minecraft:gold_block"
    );
}

#[test]
fn composite_keep_existing_does_not_replace_occupied_voxels() {
    let mut source =
        CompositeChunkSource::new(SourceProvenance::new("example:composite", "v1").unwrap());
    source
        .add_layer(
            Arc::new(MarkerSource {
                block: "minecraft:stone",
                coverage: ChunkCoverage::Complete,
            }),
            ChunkOverlayMode::Replace,
        )
        .unwrap();
    source
        .add_layer(
            Arc::new(MarkerSource {
                block: "minecraft:gold_block",
                coverage: ChunkCoverage::Partial,
            }),
            ChunkOverlayMode::KeepExisting,
        )
        .unwrap();

    let result = source.generate(ChunkRequest::new(0, 0)).unwrap();
    assert_eq!(
        result.chunk().get_block(0, 5, 0).unwrap().get_name(),
        "minecraft:stone"
    );
}

struct WrongCoordinateSource;

impl ChunkSource for WrongCoordinateSource {
    fn generate(&self, request: ChunkRequest) -> Result<ChunkResult, WorldGenerationError> {
        Ok(ChunkResult::new(
            nucleation::world_stream::WorldChunkView::new(request.cx() + 1, request.cz()),
            ChunkCoverage::Complete,
            SourceProvenance::new("example:wrong", "v1")?,
        ))
    }
}

#[test]
fn composite_rejects_mismatched_source_coordinates() {
    let mut source =
        CompositeChunkSource::new(SourceProvenance::new("example:composite", "v1").unwrap());
    source
        .add_layer(Arc::new(WrongCoordinateSource), ChunkOverlayMode::Replace)
        .unwrap();

    assert!(matches!(
        source.generate(ChunkRequest::new(0, 0)),
        Err(WorldGenerationError::MismatchedChunkCoordinates)
    ));
}

#[test]
fn chunk_bounds_reject_reversed_and_uncountable_rectangles() {
    assert!(matches!(
        ChunkBounds::new(1, 0, 0, 0),
        Err(WorldGenerationError::InvalidChunkBounds)
    ));
    assert!(matches!(
        ChunkBounds::new(i32::MIN, i32::MIN, i32::MAX, i32::MAX),
        Err(WorldGenerationError::TooManyChunks)
    ));
}

#[test]
fn region_major_cursor_matches_canonical_sort_for_negative_multi_region_bounds() {
    let calls = Arc::new(AtomicUsize::new(0));
    let source: Arc<dyn ChunkSource> = Arc::new(CountingSource { calls });
    let bounds = ChunkBounds::new(-33, -33, 33, 33).unwrap();
    let actual: Vec<_> = GeneratedChunkStream::new(source, bounds)
        .map(|result| {
            let result = result.unwrap();
            (result.chunk().cx(), result.chunk().cz())
        })
        .collect();
    let mut expected: Vec<_> = (-33..=33)
        .flat_map(|cx| (-33..=33).map(move |cz| (cx, cz)))
        .collect();
    expected.sort_by_key(|&(cx, cz)| nucleation::world_stream::chunk_order_key(cx, cz));

    assert_eq!(actual, expected);
}

#[test]
fn generated_stream_rejects_mismatched_source_coordinates() {
    let bounds = ChunkBounds::new(0, 0, 0, 0).unwrap();
    let mut stream = GeneratedChunkStream::new(Arc::new(WrongCoordinateSource), bounds);

    assert!(matches!(
        stream.next(),
        Some(Err(WorldGenerationError::MismatchedChunkCoordinates))
    ));
}

#[test]
fn cellular_sdf_source_varies_cells_deterministically() {
    let config = CellularSdfConfig {
        cell_size_x: 32,
        cell_size_z: 32,
        seed: 0x6a09_e667,
        max_jitter_x: 5.0,
        max_jitter_z: 5.0,
        max_yaw_degrees: 18.0,
        min_scale: 0.85,
        max_scale: 1.15,
        min_y_offset: -2,
        max_y_offset: 3,
        presence_numerator: 1,
        presence_denominator: 1,
        feature_salt: 0,
    };
    let source = CellularSdfChunkSource::new(
        SdfNode::Box {
            half_extents: [3.0, 2.0, 1.0],
            rounding: 0.0,
        },
        stone_brush(),
        -8,
        8,
        config,
        SourceProvenance::new("example:cellular", "v1").unwrap(),
    )
    .unwrap();

    fn generated(source: &CellularSdfChunkSource) -> Vec<(i32, i32, i32)> {
        let mut blocks = Vec::new();
        for cx in -1..=2 {
            for cz in -1..=1 {
                blocks.extend(
                    source
                        .generate(ChunkRequest::new(cx, cz))
                        .unwrap()
                        .chunk()
                        .blocks()
                        .into_iter()
                        .map(|(x, y, z, _)| (x, y, z)),
                );
            }
        }
        blocks
    }
    let first = generated(&source);
    assert_eq!(first, generated(&source));

    let local_zero: Vec<_> = first
        .iter()
        .filter(|(x, _, _)| (-10..=10).contains(x))
        .copied()
        .collect();
    let local_one: Vec<_> = first
        .iter()
        .filter(|(x, _, _)| (22..=42).contains(x))
        .map(|(x, y, z)| (x - 32, *y, *z))
        .collect();
    assert!(!local_zero.is_empty());
    assert!(!local_one.is_empty());
    assert_ne!(local_zero, local_one);
}

#[test]
fn cellular_sdf_source_rejects_unbounded_motifs() {
    let result = CellularSdfChunkSource::new(
        SdfNode::Plane {
            normal: [0.0, 1.0, 0.0],
            offset: 0.0,
        },
        stone_brush(),
        -8,
        8,
        CellularSdfConfig::default(),
        SourceProvenance::new("example:cellular", "v1").unwrap(),
    );

    assert!(matches!(
        result,
        Err(WorldGenerationError::InvalidCellularSource(_))
    ));
}

#[test]
fn cellular_sdf_source_rejects_inverted_motif_bounds() {
    let motif = SdfNode::Intersect {
        children: vec![
            SdfNode::Sphere { radius: 2.0 },
            SdfNode::Translate {
                child: Box::new(SdfNode::Sphere { radius: 2.0 }),
                offset: [100.0, 0.0, 0.0],
            },
        ],
    };
    let result = CellularSdfChunkSource::new(
        motif,
        stone_brush(),
        -8,
        8,
        CellularSdfConfig::default(),
        SourceProvenance::new("example:cellular", "v1").unwrap(),
    );

    assert!(matches!(
        result,
        Err(WorldGenerationError::InvalidCellularSource(_))
    ));
}

#[test]
fn cellular_sdf_source_includes_neighbor_motifs_that_cross_a_chunk_seam() {
    let source = CellularSdfChunkSource::new(
        SdfNode::Translate {
            child: Box::new(SdfNode::Sphere { radius: 2.0 }),
            offset: [-17.0, 0.0, 0.0],
        },
        stone_brush(),
        -4,
        4,
        CellularSdfConfig {
            cell_size_x: 32,
            cell_size_z: 32,
            ..CellularSdfConfig::default()
        },
        SourceProvenance::new("example:cellular", "v1").unwrap(),
    )
    .unwrap();

    let result = source.generate(ChunkRequest::new(0, 0)).unwrap();
    assert!(result.chunk().blocks().any(|(x, _, _, _)| x == 15));
}

#[test]
fn cellular_sdf_source_uses_local_precision_at_extreme_world_coordinates() {
    let source = CellularSdfChunkSource::new(
        SdfNode::Sphere { radius: 2.0 },
        stone_brush(),
        -4,
        4,
        CellularSdfConfig {
            cell_size_x: 32,
            cell_size_z: 32,
            ..CellularSdfConfig::default()
        },
        SourceProvenance::new("example:cellular", "v1").unwrap(),
    )
    .unwrap();

    assert!(source
        .generate(ChunkRequest::new(i32::MAX.div_euclid(16), 0))
        .is_ok());
    assert!(source
        .generate(ChunkRequest::new(i32::MIN.div_euclid(16), 0))
        .is_ok());
}

#[test]
fn cellular_sdf_source_rejects_candidate_budget_exhaustion() {
    let result = CellularSdfChunkSource::new(
        SdfNode::Box {
            half_extents: [100.0, 1.0, 1.0],
            rounding: 0.0,
        },
        stone_brush(),
        -4,
        4,
        CellularSdfConfig {
            cell_size_x: 1,
            cell_size_z: 100,
            ..CellularSdfConfig::default()
        },
        SourceProvenance::new("example:cellular", "v1").unwrap(),
    );

    assert!(matches!(
        result,
        Err(WorldGenerationError::InvalidCellularSource(_))
    ));
}

#[test]
fn cellular_config_validation_is_shared_with_source_construction() {
    // Every invariant `CellularSdfChunkSource::new` enforces must already be
    // rejected by `validate` alone, so the bindings' config constructor cannot
    // hand back a config that a later source call refuses.
    let cases = [
        CellularSdfConfig {
            min_scale: -1.0,
            ..CellularSdfConfig::default()
        },
        CellularSdfConfig {
            min_scale: 2.0,
            max_scale: 1.0,
            ..CellularSdfConfig::default()
        },
        CellularSdfConfig {
            max_scale: 9.0,
            ..CellularSdfConfig::default()
        },
        CellularSdfConfig {
            max_jitter_x: -1.0,
            ..CellularSdfConfig::default()
        },
        CellularSdfConfig {
            min_y_offset: 10,
            max_y_offset: -10,
            ..CellularSdfConfig::default()
        },
        CellularSdfConfig {
            max_yaw_degrees: 361.0,
            ..CellularSdfConfig::default()
        },
        CellularSdfConfig {
            cell_size_x: 0,
            ..CellularSdfConfig::default()
        },
        CellularSdfConfig {
            presence_denominator: 0,
            ..CellularSdfConfig::default()
        },
    ];

    for config in cases {
        assert!(
            config.validate().is_err(),
            "validate accepted a config the source rejects: {config:?}"
        );
        let source = CellularSdfChunkSource::new(
            SdfNode::Sphere { radius: 1.0 },
            stone_brush(),
            -4,
            4,
            config,
            SourceProvenance::new("example:cellular", "v1").unwrap(),
        );
        assert!(source.is_err(), "source accepted an invalid config");
    }

    assert!(CellularSdfConfig::default().validate().is_ok());
}

#[test]
fn composite_layers_added_after_a_stream_do_not_affect_it() {
    // The bridge hands composites to streams behind an `Arc` and mutates via
    // `Arc::make_mut`, so an in-flight traversal keeps the layers it started
    // with while the generator itself still accepts new ones.
    let mut composite =
        CompositeChunkSource::new(SourceProvenance::new("example:composite", "v1").unwrap());
    composite
        .add_layer(
            Arc::new(
                SdfChunkSource::new(
                    SdfNode::Sphere { radius: 2.0 },
                    stone_brush(),
                    -4,
                    4,
                    SourceProvenance::new("example:base", "v1").unwrap(),
                )
                .unwrap(),
            ),
            ChunkOverlayMode::Replace,
        )
        .unwrap();

    let snapshot = Arc::new(composite.clone());
    let mut stream = GeneratedChunkStream::new(
        snapshot.clone() as Arc<dyn ChunkSource>,
        ChunkBounds::new(0, 0, 0, 0).unwrap(),
    );

    // Mutating the generator after the snapshot must not disturb the stream.
    let mut live = snapshot;
    Arc::make_mut(&mut live)
        .add_layer(
            Arc::new(
                SdfChunkSource::new(
                    SdfNode::Sphere { radius: 4.0 },
                    stone_brush(),
                    -4,
                    4,
                    SourceProvenance::new("example:extra", "v1").unwrap(),
                )
                .unwrap(),
            ),
            ChunkOverlayMode::Replace,
        )
        .unwrap();

    let chunk = stream.next().expect("chunk").expect("generated");
    assert_eq!(chunk.chunk().cx(), 0);
    assert!(stream.next().is_none());
}
