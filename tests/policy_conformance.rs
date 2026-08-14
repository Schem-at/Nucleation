//! Cross-cutting conformance and deterministic fuzz regressions for untrusted
//! registry ingestion. These intentionally exercise public APIs only.

use nucleation::formats::limits::DecodeLimits;
use nucleation::formats::manager::get_manager;
use nucleation::formats::snapshot::{from_snapshot_bounded, to_snapshot};
use nucleation::{Entity, NbtValue, TransformPlan, UniversalSchematic};
use std::collections::HashMap;

fn hostile_nested_fixture() -> UniversalSchematic {
    let mut schematic = UniversalSchematic::new("policy-conformance".into());
    let mut owner = Entity::new("minecraft:armor_stand".into(), (0.5, 1.0, 0.5));
    owner
        .nbt
        .insert("UUID".into(), NbtValue::IntArray(vec![1, 2, 3, 4]));
    owner.nbt.insert(
        "Passengers".into(),
        NbtValue::List(vec![NbtValue::Compound(HashMap::from([
            (
                "id".into(),
                NbtValue::String("example:modded_passenger".into()),
            ),
            ("UUID".into(), NbtValue::IntArray(vec![5, 6, 7, 8])),
            ("Owner".into(), NbtValue::IntArray(vec![1, 2, 3, 4])),
            (
                "Command".into(),
                NbtValue::String("private-command-payload".into()),
            ),
            (
                "Profile".into(),
                NbtValue::String("private-profile-payload".into()),
            ),
            (
                "CustomName".into(),
                NbtValue::String("private-name-payload".into()),
            ),
        ]))]),
    );
    schematic.add_entity(owner);
    schematic
}

#[test]
fn inspect_apply_idempotence_history_and_snapshot_round_trip_conform() {
    let plan = TransformPlan::registry_safe();
    let source = hostile_nested_fixture();
    let preview = plan.inspect(&source).unwrap();
    let mut applied = source.clone();
    let report = plan.apply(&mut applied).unwrap();
    assert_eq!(preview.summary, report.summary);
    assert_eq!(preview.rejected, report.rejected);
    let report_json = report.to_json();
    for secret in [
        "private-command-payload",
        "private-profile-payload",
        "private-name-payload",
    ] {
        assert!(
            !report_json.contains(secret),
            "audit report leaked transformed content"
        );
    }

    let once = serde_json::to_value(&applied).unwrap();
    plan.apply(&mut applied).unwrap();
    assert_eq!(once, serde_json::to_value(&applied).unwrap());
    assert_eq!(applied.metadata.transformation_history.len(), 1);
    assert_eq!(
        applied.metadata.transformation_history[0]
            .verification
            .get("idempotence")
            .map(String::as_str),
        Some("passed")
    );

    let encoded = to_snapshot(&applied).unwrap();
    let decoded = from_snapshot_bounded(&encoded, &DecodeLimits::default()).unwrap();
    assert_eq!(
        decoded.metadata.transformation_history,
        applied.metadata.transformation_history
    );
}

#[test]
fn bounded_decoders_do_not_panic_on_deterministically_mutated_corpus() {
    let seed = to_snapshot(&hostile_nested_fixture()).unwrap();
    let manager = get_manager();
    let manager = manager
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let limits = DecodeLimits {
        max_input_bytes: seed.len() + 64,
        max_decompressed_bytes: 1024 * 1024,
        max_volume: 1024 * 1024,
        max_nbt_nodes: 100_000,
        ..DecodeLimits::default()
    };
    let cases = seed.len().min(512);
    for index in 0..cases {
        let mut mutated = seed.clone();
        mutated[index] ^= ((index as u8).wrapping_mul(73)).wrapping_add(1);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = manager.read_bounded(&mutated, &limits);
        }));
        assert!(
            result.is_ok(),
            "bounded decoder panicked for mutation {index}"
        );
    }
}

#[test]
fn malformed_policy_json_never_panics_or_silently_defaults() {
    for length in 0..512usize {
        let malformed = format!(
            "{{\"schema_version\":1,\"name\":\"x\",\"passes\":[{}]",
            "[".repeat(length)
        );
        let result = std::panic::catch_unwind(|| TransformPlan::from_json(&malformed));
        assert!(result.is_ok());
        assert!(result.unwrap().is_err());
    }
}
