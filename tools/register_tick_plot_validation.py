#!/usr/bin/env python3
"""Prepare an evidence-backed plot validation overlay; only --apply changes status.json.

Native survival/activity is not functional success. An idle-only vanilla match
never promotes an entry. Independent adapter-profile evidence is left intact.
"""
from __future__ import annotations

import argparse
from collections import Counter
import copy
import hashlib
import json
from pathlib import Path


REJECTED = {"build_rejected", "structure_rejected", "unsupported_inventory_container"}
SOURCES = {"world-record-parts", "world-record-floor-parts"}
REPO = Path(__file__).resolve().parents[1]


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load(path: Path):
    return json.loads(path.read_bytes())


def save(path: Path, value) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_suffix(path.suffix + ".tmp")
    temp.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    temp.replace(path)


def relative(path: Path, archive: Path) -> str:
    return path.resolve().relative_to(archive.resolve()).as_posix()


def full_match(row: dict, source_sha: str, *, activated: bool) -> bool:
    frames = row.get("expected_frames", 0)
    return (row.get("status") == "matches_capture"
            and row.get("declared_source_sha256") == source_sha
            and isinstance(frames, int) and frames > 1
            and row.get("matching_frames_including_initial") == frames
            and row.get("first_difference") is None
            and row.get("compared_capture_state_blake3") is not None
            and row.get("compared_capture_state_blake3") == row.get("compared_engine_state_blake3")
            and row.get("negative_control") is (not activated)
            and ((row.get("actions", 0) > 0) if activated else row.get("actions") == 0))


def prepare(archive: Path, native: Path, report_path: Path, oracle_artifacts: Path, existing: dict):
    manifest = load(native / "manifest.json")
    index = load(native / "index.json")
    config = manifest["run_config"]
    assert index["run_config"] == config, "index and manifest configurations differ"
    expected = {row["sha256"] for row in manifest["candidates"]}
    assert expected == set(index["results"]), "native batch is incomplete"
    library = load(archive / "library.json")
    candidates = [r for r in library["entries"] if r.get("source") in SOURCES
                  and r.get("classification") == "door_candidate"
                  and "infrastructure_review" not in r.get("review_tags", [])]
    assert {r["sha256"] for r in candidates} == expected, "native batch does not cover current plot candidates"
    report_bytes = report_path.read_bytes()
    report = json.loads(report_bytes)
    assert report["format_version"] == 1, "unknown oracle report format"
    provenance = {"native_manifest": relative(native / "manifest.json", archive),
                  "native_manifest_sha256": sha((native / "manifest.json").read_bytes()),
                  "native_index_sha256": sha((native / "index.json").read_bytes()),
                  "native_run_config": config, "host": manifest["host"],
                  "oracle_report": str(report_path.resolve()), "oracle_report_sha256": sha(report_bytes),
                  "oracle_report_snapshot": f"validation/plot-oracle-reports/{sha(report_bytes)}.json",
                  "oracle_artifacts": relative(oracle_artifacts, archive)}
    proposed = copy.deepcopy(existing)
    native_counts, projected_counts, observations = Counter(), Counter(), Counter()
    failures, oracle_cases = [], []
    by_source: dict[str, Counter] = {}
    for key in sorted(expected):
        path = native / "results" / f"{key}.json"
        raw = load(path)
        assert raw["source_sha256"] == key and raw["run_config"] == config
        assert raw["status"] == index["results"][key]["status"]
        assessment = (raw.get("native") or {}).get("assessment", {})
        assert (raw.get("native") or {}).get("source_sha256", key) == key
        status = raw["status"]
        native_counts[status] += 1
        for alias in raw["aliases"]:
            by_source.setdefault(alias["source"], Counter())[status] += 1
        trials = [t for t in assessment.get("trials", []) if t.get("control") is not None]
        count = len(trials)
        piston_trials = sum(t["first"].get("piston_related_changes", 0) > 0 for t in trials)
        cycles = sum(t["first"].get("piston_related_changes", 0) > 0
                     and t["first"].get("block_layout_equal_at_0_160_320") is True for t in trials)
        repeats = sum(t.get("deterministic_after_restore_and_reseed") is True for t in trials)
        if status == "assessed":
            assert all(t.get("deterministic_after_restore_and_reseed") is True
                       for t in assessment["trials"]), "assessed row contains failed replay"
            mapped = "simulation_smoke_checked"
            detail = (f"Completed native idle and {count} independent lever/button trials; {piston_trials} show piston "
                      f"activity and {cycles} repeat layout at 0/160/320. Functional behavior and vanilla parity unproven.")
            observations.update(control_trials=count, repeatable_control_trials=repeats, piston_control_trials=piston_trials,
                                active_layout_cycle_trials=cycles, candidates_with_controls=int(count > 0),
                                candidates_without_clickable_controls=int(count == 0),
                                candidates_with_piston_activity=int(piston_trials > 0),
                                candidates_with_active_layout_cycle=int(cycles > 0),
                                candidates_with_initial_idle_block_activity=int(assessment["settling"]["block_changes"] > 0))
        elif status in REJECTED:
            mapped = "engine_unsupported"
            detail = assessment.get("detail", status)
            failures.append({"source_sha256": key, "aliases": index["results"][key]["aliases"],
                             "native_status": status, "reason": detail, "raw_result": relative(path, archive)})
        else:
            mapped = "simulation_incomplete"
            detail = assessment.get("detail", f"Native assessment did not complete: {status}")
        entry = proposed.setdefault(key, {})
        entry.update(validation_status=mapped, validation_detail=detail, validation_evidence=relative(path, archive),
                     simulation_status=status, functional_status="unproven", vanilla_status="not_checked",
                     benchmark_status="not_admitted")
        entry["plot_validation"] = {
            "schema_version": 1, "source_sha256": key,
            "native": {"status": status, "raw_result": relative(path, archive),
                       "controls_trialled": count, "controls_with_piston_activity": piston_trials,
                       "controls_with_active_layout_cycle": cycles, "repeatable_control_trials": repeats,
                       "binary_sha256": config["binary_sha256"], "revision": config["revision"],
                       "run_manifest": relative(native / "manifest.json", archive)},
            "scope": "Native simulation survival/activity/replay only; no functional or vanilla proof."}
    seen_oracles = set()
    for case in report["cases"]:
        key = case["source_sha256"]
        assert key in expected and key not in seen_oracles, "unknown/duplicate oracle source"
        seen_oracles.add(key)
        for comparison in case.get("native", {}).values():
            assert comparison.get("declared_source_sha256") == key, "oracle source mismatch"
        functional = case.get("functional_status", "not_completed")
        cycle = case.get("native", {}).get("cycle", {})
        idle = case.get("native", {}).get("idle", {})
        cycle_pass = full_match(cycle, key, activated=True)
        idle_pass = full_match(idle, key, activated=False)
        passed = functional == "vanilla_functional_pass" and not case.get("functional_failures") and cycle_pass and idle_pass
        artifact = oracle_artifacts / case["artifact_directory"]
        relative(artifact, oracle_artifacts)  # reject absolute/path traversal outside the evidence directory
        entry = proposed[key]
        if passed:
            status = "vanilla_tick_end_parity_pass"
            detail = (f"{case['case']}: explicit vanilla aperture/reset assertions passed; activated cycle and idle "
                      f"both match all {cycle['expected_frames']} captured frames. Tick-end scope only.")
        elif functional == "vanilla_functional_failed":
            status = "vanilla_functional_fail"
            detail = (f"{case['case']}: vanilla aperture/reset assertions failed; native cycle status "
                      f"{cycle.get('status', 'not_run')}, idle status {idle.get('status', 'not_run')}. Not admitted.")
        elif cycle.get("status") == "failed" or idle.get("status") == "failed":
            status = "vanilla_tick_end_parity_fail"
            detail = (f"{case['case']}: vanilla functional status {functional}; native cycle status "
                      f"{cycle.get('status', 'not_run')}, idle status {idle.get('status', 'not_run')}. "
                      "Idle-only agreement does not establish activated behavior.")
        else:
            status = entry["validation_status"]
            detail = entry["validation_detail"] + f" Oracle case {case['case']} incomplete; no promotion."
        compact = {k: case.get(k) for k in ("case", "source_sha256", "functional_status", "functional_failures",
                   "oracle_revised_after_capture", "aperture", "closure_projection_axis", "origin", "pull_ticks",
                   "endpoint_ticks", "binaries", "captures", "benchmark_admission", "run_status", "run_error",
                   "rechecked_checker_sha256", "rechecked_checker_revision", "recheck_build_provenance")}
        # Retain any additional recheck provenance fields introduced by the
        # oracle reporter without mistaking the original binary for the rerun.
        compact.update({k: v for k, v in case.items() if k.startswith("recheck")})
        if case.get("native_before_fix"):
            compact["native_before_fix"] = {mode: {k: row.get(k) for k in ("status", "first_difference")}
                                            for mode, row in case["native_before_fix"].items()}
        compact.update(artifact_directory=relative(artifact, archive), scope=report["scope"],
                       report_sha256=provenance["oracle_report_sha256"], promoted=passed,
                       native={mode: {k: row.get(k) for k in ("status", "actions", "expected_frames",
                               "matching_frames_including_initial", "first_difference", "compared_capture_state_blake3",
                               "compared_engine_state_blake3", "entity_guard", "scope")}
                               for mode, row in case.get("native", {}).items()})
        entry.update(validation_status=status, validation_detail=detail,
                     validation_evidence=provenance["oracle_report_snapshot"],
                     functional_status=functional,
                     vanilla_status="cycle_and_idle_match" if cycle_pass and idle_pass else "activated_parity_not_established",
                     benchmark_status="admitted" if passed else "not_admitted")
        entry["plot_validation"]["oracle"] = compact
        oracle_cases.append(compact)
    for key in expected:
        projected_counts[proposed[key]["validation_status"]] += 1
    summary = {"schema_version": 1, "scope": "Evidence-backed archive overlay; native smoke is not functional proof.",
               "candidate_paths": len(candidates), "distinct_source_sha256": len(expected),
               "native_counts": dict(native_counts), "native_counts_by_source": by_source,
               "observations": dict(observations), "proposed_validation_counts": dict(projected_counts),
               "oracle_counts": {"cases": len(oracle_cases), "promoted": sum(c["promoted"] for c in oracle_cases),
                                 "vanilla_functional_pass": sum(c["functional_status"] == "vanilla_functional_pass" for c in oracle_cases),
                                 "native_cycle_matches": sum(c["native"].get("cycle", {}).get("status") == "matches_capture" for c in oracle_cases),
                                 "native_idle_matches": sum(c["native"].get("idle", {}).get("status") == "matches_capture" for c in oracle_cases)},
               "provenance": provenance, "rejections": failures, "oracle_cases": oracle_cases}
    return proposed, summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--native", default="validation/plot-native-v3", help="Archive-relative native results directory")
    parser.add_argument("--oracle-artifacts", default="validation/plot-oracles-v1", help="Archive-relative raw oracle directory")
    parser.add_argument("--oracles", type=Path, default=REPO / "docs/performance/schematic-candidates/plot-validation/oracle-results.json")
    parser.add_argument("--summary", type=Path, default=REPO / "docs/performance/schematic-candidates/validation/plot-native-summary.json")
    parser.add_argument("--overlay", type=Path, help="Optional proposed status file for review")
    parser.add_argument("--apply", action="store_true", help="Explicitly merge into archive validation/status.json")
    args = parser.parse_args()
    archive = args.archive.resolve()
    status_path = archive / "validation/status.json"
    old_bytes = status_path.read_bytes() if status_path.exists() else b"{}"
    old = json.loads(old_bytes)
    proposed, summary = prepare(archive, archive / args.native, args.oracles,
                                archive / args.oracle_artifacts, old)
    summary["integration"] = {"script_sha256": sha(Path(__file__).read_bytes()),
                              "preserved_unrelated_entries": len(set(old) - {r["sha256"] for r in load(archive / args.native / "manifest.json")["candidates"]}),
                              "status_before_sha256": sha(old_bytes), "status_entries_before": len(old),
                              "status_entries_after": len(proposed)}
    save(args.summary, summary)
    if args.overlay:
        assert args.overlay.resolve() != status_path, "use --apply to write the live archive status"
        save(args.overlay, proposed)
    if args.apply:
        current = status_path.read_bytes() if status_path.exists() else b"{}"
        assert current == old_bytes, "status.json changed during preparation; retry without losing concurrent edits"
        # Link status rows to the precise reviewed report, including later
        # comparisons against fixed engine binaries, rather than a stale raw
        # case result from the original capture run.
        report_bytes = args.oracles.read_bytes()
        assert sha(report_bytes) == summary["provenance"]["oracle_report_sha256"], "oracle report changed during preparation"
        snapshot = archive / summary["provenance"]["oracle_report_snapshot"]
        snapshot.parent.mkdir(parents=True, exist_ok=True)
        if snapshot.exists():
            assert snapshot.read_bytes() == report_bytes, "report snapshot hash collision"
        else:
            snapshot.write_bytes(report_bytes)
        save(status_path, proposed)
    print(json.dumps({"applied": args.apply, "summary": str(args.summary),
                      "proposed_validation_counts": summary["proposed_validation_counts"],
                      "oracle_counts": summary["oracle_counts"]}, indent=2))


if __name__ == "__main__":
    main()
