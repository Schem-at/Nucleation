#!/usr/bin/env python3
"""Register verified floor-mask derivatives without replacing the canonical plot partition.

Uses the same immutable SHA-256 object storage and BLAKE3 verifier as first-pass
registration. Every nonplatform state/entity remains uniquely owned; bounded
original platform context may overlap and is counted separately.
"""
import argparse
from collections import Counter
import json
from pathlib import Path
import re

import register_tick_plot_parts as base

SOURCE_ID = "world-record-floor-parts"
PARENT_SOURCE = "world-record-parts"
CONFIG = {"part0004": (50, 83774, 16460), "part0005": (101, 83775, 724), "part0006": (152, 83780, 582)}
MATERIALS = ["minecraft:black_stained_glass", "minecraft:white_stained_glass", "minecraft:birch_planks"]
BE_NORMALIZATION = "authoritative_id_and_absolute_position;relative_Id_id_Pos_x_y_z_wrappers_normalized;all_other_NBT_bit_exact"
TOP_FLAGS = ["all_exported_files_read_back", "all_nonplatform_states_exactly_once",
             "all_parent_block_entities_exactly_once", "all_parent_entities_exactly_once",
             "all_context_cells_exact_parent_states", "metadata_preserved"]
PARENT_FLAGS = ["nonplatform_states_exactly_once", "block_entities_exactly_once",
                "entities_exactly_once", "context_is_exact_source_subset", "all_exported_files_read_back"]
PART_CHECKS = {
    "readback": "passed", "all_states_exact_subset_of_parent": True,
    "nonplatform_ownership": "exactly_once_across_children",
    "platform_context": "exact_source_cells;overlap_allowed;not_a_partition",
    "block_entities": "exact_nbt_and_positions", "block_entity_coordinate_normalization": BE_NORMALIZATION,
    "entities": "exact_nbt_and_positions_including_passengers", "metadata": "equal_to_parent",
    "float_comparison": "IEEE_bits_including_NaN", "functional_status": "unverified", "simulation_status": "not_run",
}


def require(condition, message):
    if not condition:
        base.fail(message)


def integer(value, label):
    require(type(value) is int and value >= 0, f"invalid {label}")
    return value


def validate(root, extraction, checker):
    require(not (extraction / "INCOMPLETE").exists(), "floor extraction is incomplete")
    raw = base.regular_file(extraction / "manifest.json")
    manifest = json.loads(raw)
    require(manifest.get("schema_version") == 1 and manifest.get("kind") == "world_record_floor_parts", "unexpected extraction schema")
    verify = manifest.get("verification", {})
    require(verify.get("status") == "passed", "floor extraction verification failed")
    for flag in TOP_FLAGS:
        require(verify.get(flag) is True, f"missing successful check {flag}")
    require(verify.get("block_entity_coordinate_normalization") == BE_NORMALIZATION, "unexpected BE coordinate normalization")
    require(manifest.get("canonical_partition_retained_unchanged") is True, "canonical partition preservation not asserted")
    original_path = root / "sources" / base.ORIGINAL_SOURCE_ID / "files" / base.ORIGINAL_FILE
    original = base.regular_file(original_path)
    require(base.sha256(original) == base.ORIGINAL_SHA256, "pinned original plot changed")
    require(checker(original_path, original) == manifest["original_plot"]["file_blake3"], "original plot BLAKE3 mismatch")
    canonical_path = root / "sources" / PARENT_SOURCE / "extraction-manifest.json"
    canonical_raw = base.regular_file(canonical_path)
    require(checker(canonical_path, canonical_raw) == manifest.get("canonical_extraction_manifest_blake3"), "canonical parent extraction manifest mismatch")
    canonical = json.loads(canonical_raw)
    require(manifest["original_plot"] == canonical["source"], "original plot provenance changed")
    parent_archive = json.loads(base.regular_file(root / "sources" / PARENT_SOURCE / "manifest.json"))
    archive_records = {r["path"]: r for r in parent_archive["files"]}
    canonical_records = {r["id"]: r for r in canonical["parts"]}
    isolation = manifest.get("isolation", {})
    for key, value in {"api": "UniversalSchematic::split_connected_attach_nearby", "connectivity": "Corner",
                       "min_standalone_blocks": 16, "max_air_gap": 3, "context_horizontal_halo": 1,
                       "coordinates": "original_source_coordinates_without_rebasing", "inventory_repairs": "none"}.items():
        require(isolation.get(key) == value, f"unexpected isolation setting {key}")
    parents = {}
    for parent in manifest["parents"]:
        pid = parent["id"]
        require(pid in CONFIG and pid not in parents, "unexpected or duplicate parent")
        floor_y, mask_count, non_air = CONFIG[pid]
        require(parent["mask"]["floor_y"] == floor_y and parent["mask"]["materials"] == MATERIALS,
                f"{pid}: platform mask changed")
        require(parent["mask"]["masked_platform_cells"] == mask_count and parent["nonplatform_non_air_blocks"] == non_air,
                f"{pid}: pinned parent counts changed")
        require(parent["data_version"] == 4671, f"{pid}: data version changed")
        require(parent["verification"].get("status") == "passed", f"{pid}: parent verification failed")
        for flag in PARENT_FLAGS:
            require(parent["verification"].get(flag) is True, f"{pid}: missing successful check {flag}")
        path = root / archive_records[parent["file"]]["local_path"]
        data = base.regular_file(path)
        sha = base.sha256(data)
        require(sha == archive_records[parent["file"]]["sha256"], f"{pid}: canonical archive SHA256 changed")
        require(checker(path, data) == parent["file_blake3"] == canonical_records[pid]["file_blake3"], f"{pid}: parent BLAKE3 changed")
        require(len(data) == parent["bytes"], f"{pid}: parent byte count changed")
        require(parent["entity_source_indices"] == canonical_records[pid]["entity_source_indices"], f"{pid}: source entity indices changed")
        parents[pid] = dict(parent, sha256=sha, local_path=str(path.relative_to(root)))
    require(set(parents) == set(CONFIG), "not all three parents represented")
    payloads, seen_ids, seen_files = [], set(), set()
    aggregates = {pid: Counter() for pid in CONFIG}
    entity_indices = {pid: set() for pid in CONFIG}
    context_union = {pid: set() for pid in CONFIG}
    child_ids = {pid: [] for pid in CONFIG}
    for part in manifest["parts"]:
        pid = part["parent_part"]
        require(pid in parents, "unknown child parent")
        parent = parents[pid]
        identifier, filename = part["id"], part["file"]
        require(re.fullmatch(re.escape(pid) + r"-floorpart[0-9]{4,}", identifier) is not None, "invalid child ID")
        require(filename == f"{identifier}.litematic" and identifier not in seen_ids and filename not in seen_files, "duplicate/unsafe child filename")
        seen_ids.add(identifier); seen_files.add(filename); child_ids[pid].append(identifier)
        require(part["parent_file_blake3"] == parent["file_blake3"], f"{identifier}: parent linkage changed")
        require(part["mask"] == {"floor_y": CONFIG[pid][0], "materials": MATERIALS, "applies_to_connectivity_only": True}, "child mask changed")
        for key, value in PART_CHECKS.items():
            require(part["verification"].get(key) == value, f"{identifier}: successful check missing: {key}")
        for key in ["non_air_blocks", "explicit_air_states", "preserved_block_states", "nonplatform_non_air_blocks",
                    "nonplatform_explicit_air_states", "nonplatform_preserved_block_states", "block_entities", "entities"]:
            integer(part.get(key), f"{identifier}.{key}")
            aggregates[pid][key] += part[key]
        context = part["restored_platform_context"]
        require(context.get("floor_y") == CONFIG[pid][0] and context.get("horizontal_halo") == 1, "context settings changed")
        for flag in ["exact_source_cells_only", "overlap_with_other_children_allowed", "remaining_platform_retained_in_canonical_parent"]:
            require(context.get(flag) is True, "missing context preservation flag")
        require(context.get("block_entities") == 0, "masked platform block entities require review")
        positions = context["positions"]
        require(len(positions) == context["count"], "context count mismatch")
        points = set()
        for pos in positions:
            require(isinstance(pos, list) and len(pos) == 3 and all(type(n) is int for n in pos), "invalid context coordinate")
            require(pos[1] == CONFIG[pid][0], "context escaped platform plane")
            bounds = part["nonplatform_bounds"]
            require(bounds is not None and bounds["min"][0] - 1 <= pos[0] <= bounds["max"][0] + 1
                    and bounds["min"][2] - 1 <= pos[2] <= bounds["max"][2] + 1, "context escaped bounded horizontal halo")
            require(all(a <= p <= b for a, p, b in zip(parent["bounds"]["min"], pos, parent["bounds"]["max"])), "context outside parent bounds")
            require(tuple(pos) not in points, "duplicate context position within child")
            points.add(tuple(pos))
        context_union[pid].update(points)
        aggregates[pid]["platform_context_instances"] += len(points)
        require(part["non_air_blocks"] == part["nonplatform_non_air_blocks"] + len(points), "non-air/context totals disagree")
        require(part["preserved_block_states"] == part["nonplatform_preserved_block_states"] + len(points), "preserved/context totals disagree")
        require(part["preserved_block_states"] == part["non_air_blocks"] + part["explicit_air_states"], "air state totals disagree")
        indices = part["parent_entity_indices"]
        require(len(indices) == part["entities"], "child entity count mismatch")
        for i in indices:
            require(type(i) is int and 0 <= i < parent["entities"] and i not in entity_indices[pid], "invalid/duplicate parent entity index")
            entity_indices[pid].add(i)
        require(part["entity_source_indices"] == [parent["entity_source_indices"][i] for i in indices], "global source entity identity mismatch")
        classification = part["classification"]
        pistons = part["pistons"]["total"]
        require(classification in ("door_candidate", "other_component") and (classification == "door_candidate") == (pistons > 0), "door classification lacks piston evidence")
        aggregates[pid]["files"] += 1
        aggregates[pid]["door_candidates"] += classification == "door_candidate"
        aggregates[pid]["entity_quarantine_files"] += bool(part.get("entity_quarantine"))
        path = extraction / filename
        data = base.regular_file(path)
        require(len(data) == part["bytes"] and checker(path, data) == part["file_blake3"], f"{identifier}: extracted bytes/hash changed")
        for key in ["content_blake3", "nonplatform_content_blake3", "block_entities_blake3", "entities_blake3"]:
            base.hex_hash(part[key], key)
        base.hex_hash(context["content_blake3"], "context content hash")
        payloads.append((part, data, base.sha256(data)))
    for pid, parent in parents.items():
        for key in ["nonplatform_non_air_blocks", "nonplatform_preserved_block_states", "nonplatform_explicit_air_states",
                    "block_entities", "entities", "files", "door_candidates", "entity_quarantine_files", "platform_context_instances"]:
            require(aggregates[pid][key] == parent[key], f"{pid}: child aggregate mismatch {key}")
        require(aggregates[pid]["files"] - aggregates[pid]["entity_quarantine_files"] == parent["physical_components"], "physical component count mismatch")
        require(entity_indices[pid] == set(range(parent["entities"])), "parent entity coverage incomplete")
        require(len(context_union[pid]) == parent["unique_platform_context_cells"], "unique context count mismatch")
        require(len(context_union[pid]) + parent["platform_cells_retained_only_in_canonical_parent"] == parent["mask"]["masked_platform_cells"], "platform context accounting mismatch")
        require(child_ids[pid] == parent["part_ids"], "parent child ordering mismatch")
    return manifest, raw, parents, payloads, parent_archive["source"]["url"]


def title(part):
    parent = part["parent_part"].removeprefix("part")
    number = part["id"].split("floorpart")[-1]
    if part.get("entity_quarantine"):
        return f"Unassigned floor entities · {parent}"
    if part["classification"] == "door_candidate":
        size = "×".join(str(n) for n in base.dimensions(part["nonplatform_bounds"]))
        return f"Floor door {parent}/{number} · {size}"
    return f"Floor fragment {parent}/{number}"


def registration(manifest, raw, parents, payloads, original_url):
    source = {"id": SOURCE_ID, "url": original_url, "category": "WorldRecordPlot floor-attached candidates",
              "local_only": True, "license": "unknown; local research derivative",
              "provenance": "Second pass over three retained infrastructure components. Exact platform mask for connectivity; original bounded platform context restored after isolation. Canonical first-pass partition unchanged.",
              "original_plot_sha256": base.ORIGINAL_SHA256, "parents": list(parents.values()),
              "isolation": manifest["isolation"], "extraction_manifest_sha256": base.sha256(raw)}
    records, rows = [], []
    for part, data, sha in payloads:
        path = f"sources/{SOURCE_ID}/files/{part['file']}"
        records.append({"path": part["file"], "kind": "schematic", "status": "downloaded",
                        "origin": "local_floor_mask_derivative", "bytes": len(data), "sha256": sha,
                        "blake3": part["file_blake3"], "source_url": original_url,
                        "local_path": path, "object_path": f"objects/sha256/{sha[:2]}/{sha}",
                        "validation": "source_subset_and_nonplatform_partition_readback_verified; not_simulated", "local_only": True})
        tags = [SOURCE_ID, "floor-mask-derivative", "part-isolator", "local-only", part["classification"]] + part.get("review_tags", [])
        if part["classification"] == "door_candidate": tags += ["door", "piston"]
        if part["observers"]: tags.append("observer")
        if part["containers"]["count"]: tags.append("inventories")
        if part["entities"]: tags.append("entities")
        parent = parents[part["parent_part"]]
        row = dict(part)
        row.update({"source": SOURCE_ID, "path": part["file"], "local_path": path, "sha256": sha,
                    "title": title(part), "name": title(part), "display_name": title(part), "source_url": original_url,
                    "native_data_version": 4671, "data_version": 4671, "local_only": True,
                    "dimensions": base.dimensions(part["bounds"]), "tags": sorted(set(tags)),
                    "isolation": manifest["isolation"], "readiness": "unvalidated", "stage": "isolated_floor_context_derivative",
                    "functional_status": "unverified", "simulation_status": "not_run", "vanilla_status": "not_run",
                    "import_status": "readback_verified", "benchmark_status": "not_admitted",
                    "priority": "medium" if part["classification"] == "door_candidate" else "low",
                    "derived_from": {"source": PARENT_SOURCE, "path": parent["file"], "parent_part": part["parent_part"],
                                     "sha256": parent["sha256"], "blake3": parent["file_blake3"], "original_plot_sha256": base.ORIGINAL_SHA256},
                    "evidence_basis": "Exact source subset with a disjoint nonplatform partition and bounded overlapping original platform context. Every file read back; no functionality test.",
                    "test_plan": "Inspect adjacent components and required floor/travel area, establish controls and saved inventory/entity requirements, then capture full activation/reset and idle controls in native Minecraft 1.21.11.",
                    "static": {"format": "litematic", "native_data_version": 4671, "dimensions": base.dimensions(part["bounds"]),
                               "non_air_blocks": part["non_air_blocks"], "nonplatform_non_air_blocks": part["nonplatform_non_air_blocks"],
                               "preserved_block_states": part["preserved_block_states"], "explicit_air_states": part["explicit_air_states"],
                               "block_entities": part["block_entities"], "entities": part["entities"], "block_counts": part["block_histogram"],
                               "control_count": part["controls"]["count"], "sha256_matches_manifest": True},
                    "warnings": manifest["warnings"], "license_status": "Unknown; local-only derivative, not redistributed in Git."})
        rows.append(row)
    archive_manifest = {"source": source, "files": records, "selected_files": len(records), "downloaded": len(records),
                        "schematics": len(records), "layouts": 0, "errors": 0, "downloaded_bytes": sum(r["bytes"] for r in records),
                        "extraction_verification": manifest["verification"]}
    curation = {"schema_version": 1, "curator": SOURCE_ID, "scope": "Every second-pass component and any entity quarantine; context overlap explicit, functionality unvalidated.", "entries": rows}
    totals = {key: sum(p[key] for p in parents.values()) for key in ["nonplatform_non_air_blocks", "nonplatform_preserved_block_states", "nonplatform_explicit_air_states", "block_entities", "entities", "physical_components", "entity_quarantine_files", "platform_context_instances", "unique_platform_context_cells"]}
    summary = {"schema_version": 1, "source": SOURCE_ID, "files": len(records), "classifications": dict(Counter(p["classification"] for p, _, _ in payloads)),
               "original_plot_sha256": base.ORIGINAL_SHA256, "extraction_manifest_sha256": base.sha256(raw), "totals": totals,
               "parents": list(parents.values()), "isolation": manifest["isolation"], "extraction_verification": manifest["verification"],
               "registered_bytes": archive_manifest["downloaded_bytes"], "canonical_partition_unchanged": True,
               "platform_context_overlap_allowed": True, "functional_status": "unverified", "benchmark_status": "not_admitted", "local_only": True}
    return archive_manifest, curation, summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--extraction", type=Path, required=True)
    parser.add_argument("--b3sum", type=Path)
    parser.add_argument("--check-only", action="store_true")
    parser.add_argument("--report-summary", type=Path)
    parser.add_argument("--report-catalog", type=Path)
    args = parser.parse_args()
    root, extraction = args.archive.resolve(), args.extraction.resolve()
    manifest, raw, parents, payloads, original_url = validate(root, extraction, base.blake3_checker(args.b3sum))
    archive_manifest, curation, summary = registration(manifest, raw, parents, payloads, original_url)
    source_dir = root / "sources" / SOURCE_ID
    writes = [(source_dir / "source.json", base.json_bytes(archive_manifest["source"])),
              (source_dir / "extraction-manifest.json", raw),
              (root / "curation" / f"{SOURCE_ID}.json", base.json_bytes(curation))]
    if args.report_summary: writes.append((args.report_summary.absolute(), base.json_bytes(summary)))
    if args.report_catalog: writes.append((args.report_catalog.absolute(), base.reproducible_gzip(base.json_bytes({"summary": summary, "extraction": manifest, "curation": curation}))))
    writes.append((source_dir / "manifest.json", base.json_bytes(archive_manifest)))
    for path, data in writes: base.identical_destination(path, data)
    for (_, data, _), record in zip(payloads, archive_manifest["files"]):
        base.identical_destination(root / record["object_path"], data)
        base.identical_destination(root / record["local_path"], data)
    if not args.check_only:
        for (_, data, _), record in zip(payloads, archive_manifest["files"]):
            base.hardlink_object(root / record["object_path"], root / record["local_path"], data)
        for path, data in writes: base.immutable_write(path, data)
    print(json.dumps({k: v for k, v in dict(summary, registration="check_only" if args.check_only else "registered").items() if k != "parents"}, indent=2))


if __name__ == "__main__":
    main()
