use nucleation::block_position::BlockPosition;
use nucleation::formats::anvil::is_mca;
use nucleation::formats::manager::get_manager;
use nucleation::formats::structure_snbt::{
    from_structure_snbt, is_structure_snbt, to_structure_snbt,
};
use nucleation::utils::NbtValue;
use nucleation::{BlockState, Region, UniversalSchematic};

const MINIMAL: &str = r#"{
    DataVersion: 4325,
    size: [2, 1, 1],
    data: [
        {pos: [0, 0, 0], state: "minecraft:stone"},
        {pos: [1, 0, 0], state: "minecraft:repeater{delay:4,facing:north,locked:false,powered:false}"}
    ],
    entities: [],
    palette: [
        "minecraft:stone",
        "minecraft:repeater{delay:4,facing:north,locked:false,powered:false}"
    ]
}"#;

#[test]
fn imports_structure_snbt_blocks_and_metadata() {
    assert!(is_structure_snbt(MINIMAL.as_bytes()));

    let schematic = from_structure_snbt(MINIMAL.as_bytes()).expect("import structure SNBT");
    assert_eq!(schematic.metadata.source_data_version, Some(4325));
    assert_eq!(schematic.metadata.mc_version, Some(4325));
    assert_eq!(schematic.default_region.position, (0, 0, 0));
    assert_eq!(schematic.default_region.size, (2, 1, 1));
    assert_eq!(
        schematic.get_block(0, 0, 0).unwrap().to_string(),
        "minecraft:stone"
    );
    assert_eq!(
        schematic.get_block(1, 0, 0).unwrap().to_string(),
        "minecraft:repeater[delay=4,facing=north,locked=false,powered=false]"
    );
}

#[test]
fn imports_block_entity_nbt_without_losing_tag_types() {
    let input = r#"{
        DataVersion: 4325,
        size: [1, 1, 1],
        data: [{
            pos: [0, 0, 0],
            state: "minecraft:test_block{mode:start}",
            nbt: {id: "minecraft:test_block", message: "begin", mode: "start", powered: 0b}
        }],
        entities: [],
        palette: ["minecraft:test_block{mode:start}"]
    }"#;

    let schematic = from_structure_snbt(input.as_bytes()).expect("import block entity");
    let block_entity = schematic
        .get_block_entity(BlockPosition { x: 0, y: 0, z: 0 })
        .expect("test block entity");
    assert_eq!(block_entity.id, "minecraft:test_block");
    assert_eq!(
        block_entity.nbt.get("message"),
        Some(&NbtValue::String("begin".to_string()))
    );
    assert_eq!(block_entity.nbt.get("powered"), Some(&NbtValue::Byte(0)));
}

#[test]
fn imports_entities_using_structure_relative_position() {
    let input = r#"{
        DataVersion: 4325,
        size: [3, 3, 3],
        data: [{pos: [0, 0, 0], state: "minecraft:air"}],
        entities: [{
            blockPos: [1, 1, 1],
            pos: [1.5d, 1.0d, 1.5d],
            nbt: {id: "minecraft:armor_stand", Pos: [31.5d, -58.0d, 28.5d], Health: 20.0f}
        }],
        palette: ["minecraft:air"]
    }"#;

    let schematic = from_structure_snbt(input.as_bytes()).expect("import entity");
    let entities = schematic.get_entities_as_list();
    assert_eq!(entities.len(), 1);
    assert_eq!(entities[0].id, "minecraft:armor_stand");
    assert_eq!(entities[0].position, (1.5, 1.0, 1.5));
    assert!(entities[0].nbt.contains_key("Health"));
}

#[test]
fn exports_parseable_snbt_and_semantically_round_trips() {
    let input = r#"{
        DataVersion: 4325,
        size: [2, 2, 1],
        data: [
            {pos: [0, 0, 0], state: "minecraft:stone"},
            {pos: [1, 0, 0], state: "minecraft:air"},
            {pos: [0, 1, 0], state: "minecraft:test_block{mode:start}", nbt: {id: "minecraft:test_block", message: "go", powered: 0b}},
            {pos: [1, 1, 0], state: "minecraft:repeater{delay:2,facing:east,locked:false,powered:true}"}
        ],
        entities: [{blockPos: [0, 1, 0], pos: [0.5d, 1.0d, 0.5d], nbt: {id: "minecraft:armor_stand", Health: 20.0f}}],
        palette: ["minecraft:air", "minecraft:stone", "minecraft:test_block{mode:start}", "minecraft:repeater{delay:2,facing:east,locked:false,powered:true}"]
    }"#;

    let first = from_structure_snbt(input.as_bytes()).expect("initial import");
    let exported = to_structure_snbt(&first).expect("export structure SNBT");
    assert!(is_structure_snbt(&exported));
    quartz_nbt::snbt::parse(std::str::from_utf8(&exported).unwrap()).expect("valid SNBT output");

    let second = from_structure_snbt(&exported).expect("re-import exported SNBT");
    assert_eq!(second.metadata.source_data_version, Some(4325));
    assert_eq!(second.default_region.size, (2, 2, 1));
    for y in 0..2 {
        for x in 0..2 {
            assert_eq!(
                first.get_block(x, y, 0).map(ToString::to_string),
                second.get_block(x, y, 0).map(ToString::to_string)
            );
        }
    }
    let first_be = first
        .get_block_entity_owned(BlockPosition { x: 0, y: 1, z: 0 })
        .expect("source block entity");
    let second_be = second
        .get_block_entity_owned(BlockPosition { x: 0, y: 1, z: 0 })
        .expect("round-tripped block entity");
    assert_eq!(first_be, second_be);
    let first_entities = first.get_entities_as_list();
    let second_entities = second.get_entities_as_list();
    assert_eq!(first_entities.len(), 1);
    assert_eq!(second_entities.len(), 1);
    assert_eq!(first_entities[0].id, second_entities[0].id);
    assert_eq!(first_entities[0].position, second_entities[0].position);
}

#[test]
fn export_normalizes_translated_schematic_to_structure_local_coordinates() {
    let mut schematic = from_structure_snbt(MINIMAL.as_bytes()).unwrap();
    schematic.translate(10, 5, -3).unwrap();

    let exported = to_structure_snbt(&schematic).unwrap();
    let imported = from_structure_snbt(&exported).unwrap();
    assert_eq!(imported.default_region.position, (0, 0, 0));
    assert_eq!(imported.default_region.size, (2, 1, 1));
    assert_eq!(
        imported.get_block(0, 0, 0).unwrap().to_string(),
        "minecraft:stone"
    );
    assert_eq!(
        imported.get_block(1, 0, 0).unwrap().to_string(),
        "minecraft:repeater[delay=4,facing=north,locked=false,powered=false]"
    );
}

#[test]
fn format_manager_detects_reads_and_writes_structure_snbt() {
    let manager = get_manager();
    let manager = manager.lock().unwrap();
    assert_eq!(
        manager.detect_format(MINIMAL.as_bytes()).as_deref(),
        Some("structure_snbt")
    );
    let schematic = manager.read(MINIMAL.as_bytes()).expect("manager import");
    let explicit = manager
        .write("structure_snbt", &schematic, None)
        .expect("manager named export");
    assert!(is_structure_snbt(&explicit));
    let output = manager
        .write_auto("fixture.snbt", &schematic, None)
        .expect("manager extension export");
    assert!(is_structure_snbt(&output));
    assert!(manager
        .write("structure_snbt", &schematic, Some("v2"))
        .is_err());
    assert!(manager.write("gametest_snbt", &schematic, None).is_err());
}

#[test]
fn rejects_oversized_imports_before_allocating_a_region() {
    let oversized = r#"{
        DataVersion: 4325,
        size: [257, 1, 1],
        data: [],
        entities: [],
        palette: []
    }"#;
    let error = from_structure_snbt(oversized.as_bytes()).unwrap_err();
    assert!(error.to_string().contains("exceeds"));
}

#[test]
fn rejects_oversized_export_bounds_before_materializing_air() {
    let mut schematic = UniversalSchematic::new("oversized".to_string());
    let stone = BlockState::new("minecraft:stone");
    schematic.set_block(0, 0, 0, &stone);
    schematic.set_block(256, 0, 0, &stone);

    let error = to_structure_snbt(&schematic).unwrap_err();
    assert!(error.to_string().contains("exceeds"));
}

#[test]
fn rejects_extreme_export_bounds_without_dimension_overflow() {
    let mut schematic = UniversalSchematic::new("extreme".to_string());
    schematic.add_region(Region::new("left".to_string(), (i32::MIN, 0, 0), (1, 1, 1)));
    schematic.add_region(Region::new(
        "right".to_string(),
        (i32::MAX, 0, 0),
        (1, 1, 1),
    ));

    let stone = BlockState::new("minecraft:stone");
    assert!(schematic.set_block_in_region("left", i32::MIN, 0, 0, &stone));
    assert!(schematic.set_block_in_region("right", i32::MAX, 0, 0, &stone));

    let error = to_structure_snbt(&schematic).unwrap_err();
    assert!(error.to_string().contains("exceeds"));
}

#[test]
fn long_snbt_text_is_not_misdetected_as_mca() {
    let mut padded = MINIMAL.as_bytes().to_vec();
    padded.resize(9_000, b' ');
    assert!(is_structure_snbt(&padded));
    assert!(!is_mca(&padded));
}

#[test]
fn valid_mca_location_and_chunk_header_is_still_detected() {
    let mut mca = vec![0u8; 3 * 4096];
    mca[0..4].copy_from_slice(&[0, 0, 2, 1]);
    mca[8192..8196].copy_from_slice(&2u32.to_be_bytes());
    mca[8196] = 2; // zlib compression
    mca[8197] = 0; // one byte of payload; full parsing is outside detection's scope
    assert!(is_mca(&mca));
}

#[test]
#[ignore = "requires the pinned, gitignored Lithium fixture corpus"]
fn lithium_fixture_corpus_imports_and_round_trips() {
    let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/lithium-gametests/data/gametest/structure");
    assert!(
        fixtures.exists(),
        "local Lithium fixture corpus is missing at {}",
        fixtures.display()
    );

    let mut paths: Vec<_> = std::fs::read_dir(&fixtures)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("snbt"))
        .collect();
    paths.sort();
    assert_eq!(paths.len(), 33, "unexpected Lithium fixture count");

    for path in paths {
        let source = std::fs::read(&path).unwrap();
        assert!(
            is_structure_snbt(&source),
            "structure SNBT detector rejected {}",
            path.display()
        );
        assert!(
            !is_mca(&source),
            "MCA detector accepted SNBT file {}",
            path.display()
        );
        let first = from_structure_snbt(&source)
            .unwrap_or_else(|error| panic!("failed importing {}: {error}", path.display()));
        let exported = to_structure_snbt(&first)
            .unwrap_or_else(|error| panic!("failed exporting {}: {error}", path.display()));
        let second = from_structure_snbt(&exported)
            .unwrap_or_else(|error| panic!("failed re-importing {}: {error}", path.display()));

        assert_eq!(
            first.metadata.source_data_version,
            second.metadata.source_data_version
        );
        assert_eq!(first.default_region.size, second.default_region.size);
        let size = first.default_region.size;
        for y in 0..size.1 {
            for z in 0..size.2 {
                for x in 0..size.0 {
                    assert_eq!(
                        first.get_block(x, y, z).map(ToString::to_string),
                        second.get_block(x, y, z).map(ToString::to_string),
                        "block mismatch in {} at ({x}, {y}, {z})",
                        path.display()
                    );
                    assert_eq!(
                        first.get_block_entity_owned(BlockPosition { x, y, z }),
                        second.get_block_entity_owned(BlockPosition { x, y, z }),
                        "block entity mismatch in {} at ({x}, {y}, {z})",
                        path.display()
                    );
                }
            }
        }
        assert_eq!(
            first.get_entities_as_list(),
            second.get_entities_as_list(),
            "entity mismatch in {}",
            path.display()
        );
    }
}
