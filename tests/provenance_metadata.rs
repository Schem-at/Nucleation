use std::collections::BTreeMap;

use nucleation::{litematic, schematic, ProvenanceBounds, SchematicProvenance, UniversalSchematic};

fn provenance() -> SchematicProvenance {
    SchematicProvenance {
        schema_version: 1,
        source_id: "world:creative".into(),
        world_name: Some("Creative".into()),
        map_name: Some("Creative-MC26.1.2-DH".into()),
        dimension: Some("minecraft:overworld".into()),
        snapshot_id: Some("2026-08-11".into()),
        world_bbox: Some(ProvenanceBounds {
            min: [5, 62, 1105],
            max: [17, 85, 1114],
        }),
        origin: Some([5, 62, 1105]),
        partition_id: Some("plot:1:11".into()),
        stable_build_id: Some("abc123".into()),
        extracted_at: Some(1_786_400_000),
        config_hash: Some("config-hash".into()),
        profile_hash: Some("profile-hash".into()),
        attributes: BTreeMap::from([
            ("nucleation:partition_owner".into(), "ExamplePlayer".into()),
            (
                "nucleation:partition_trusted".into(),
                "BuilderOne,BuilderTwo".into(),
            ),
            (
                "nucleation:partition_catalog_hash".into(),
                "aa116952f726d718823dadb209fad6eb6a7130a0c8e6f04007db5c45f9b66c7b".into(),
            ),
        ]),
    }
}

fn schematic_with_provenance() -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new("provenance-roundtrip".into());
    schematic.set_block_str(0, 0, 0, "minecraft:stone");
    schematic.metadata.provenance = Some(provenance());
    schematic
}

#[test]
fn sponge_schematic_round_trips_standard_provenance() {
    let original = schematic_with_provenance();
    let bytes = schematic::to_schematic(&original).unwrap();
    let decoded = schematic::from_schematic(&bytes).unwrap();
    assert_eq!(decoded.metadata.provenance, original.metadata.provenance);
}

#[test]
fn litematic_round_trips_standard_provenance() {
    let original = schematic_with_provenance();
    let bytes = litematic::to_litematic(&original).unwrap();
    let decoded = litematic::from_litematic(&bytes).unwrap();
    assert_eq!(decoded.metadata.provenance, original.metadata.provenance);
}

#[test]
fn provenance_json_rejects_unversioned_and_unnamespaced_data() {
    let mut bad_version = provenance();
    bad_version.schema_version = 99;
    assert!(bad_version.to_json().is_err());

    let mut bad_attribute = provenance();
    bad_attribute
        .attributes
        .insert("claim_id".into(), "42".into());
    assert!(bad_attribute.to_json().is_err());
}
