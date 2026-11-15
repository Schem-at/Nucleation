#!/usr/bin/env python3
"""Register a verified WorldRecordPlot extraction in the local schematic archive.

No downloading, simulation, inventory repair, source mutation or deletion is
performed. Repeated runs accept only byte-identical existing destinations.
Requires the Python blake3 package or a b3sum executable to check the Rust
extractor's hashes. --check-only performs every input check without writing.
"""
import argparse
from collections import Counter
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess


SOURCE_ID = "world-record-parts"
ORIGINAL_SOURCE_ID = "user-world-record-plot"
ORIGINAL_FILE = "WorldRecordPlot.litematic"
ORIGINAL_SHA256 = "5476ac349cd4529d7a237828633b9e7a995d70b8937f70e4836d2f9f8db66c1e"
EXPECTED_COUNTS = {"non_air_blocks": 458409, "preserved_block_states": 458569,
                   "explicit_air_states": 160, "block_entities": 12541, "entities": 318}
SUCCESS_FLAGS = (
    "all_exported_files_read_back", "source_block_states_exactly_once",
    "source_block_entities_exactly_once", "source_entities_exactly_once",
    "metadata_preserved",
)
PART_CHECKS = {
    "blocks": "exact_source_partition",
    "block_entities": "exact_nbt_and_positions",
    "block_entity_coordinate_normalization": "authoritative_id_and_absolute_position;relative_Id_id_Pos_x_y_z_wrappers_normalized;all_other_NBT_bit_exact",
    "entities": "exact_nbt_and_positions_including_passengers",
    "metadata": "equal_to_source",
    "readback": "passed",
    "float_comparison": "IEEE_bits_including_NaN",
    "functional_status": "unverified",
    "simulation_status": "not_run",
}


def json_bytes(value):
    return (json.dumps(value, indent=2, ensure_ascii=False, sort_keys=True, allow_nan=False) + "\n").encode()


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def fail(message):
    raise ValueError(message)


def hex_hash(value, label):
    if not isinstance(value, str) or not re.fullmatch(r"[0-9a-f]{64}", value):
        fail(f"invalid {label}")
    return value


def regular_file(path):
    if path.is_symlink() or not path.is_file():
        fail(f"expected a regular non-symlink file: {path}")
    return path.read_bytes()


def identical_destination(path, data):
    """Refuse replacement, including a dangling symlink or corrupted object."""
    if path.is_symlink():
        fail(f"refusing symlink destination: {path}")
    if path.exists() and (not path.is_file() or path.read_bytes() != data):
        fail(f"existing destination differs; nothing overwritten: {path}")


def immutable_write(path, data):
    identical_destination(path, data)
    if path.exists():
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with path.open("xb") as output:
            output.write(data)
            output.flush()
            os.fsync(output.fileno())
    except FileExistsError:
        # A concurrent identical registration is safe; never replace its file.
        identical_destination(path, data)


def hardlink_object(blob, target, data):
    immutable_write(blob, data)
    blob.chmod(0o444)
    identical_destination(target, data)
    if not target.exists():
        target.parent.mkdir(parents=True, exist_ok=True)
        try:
            os.link(blob, target)
        except FileExistsError:
            identical_destination(target, data)
    if target.read_bytes() != data:
        fail(f"post-link verification failed: {target}")


def blake3_checker(command=None):
    try:
        import blake3
    except ImportError:
        blake3 = None
    if command is None and blake3 is not None:
        return lambda path, data: blake3.blake3(data).hexdigest()
    executable = str(command) if command else shutil.which("b3sum")
    if not executable:
        fail("BLAKE3 verification requires Python blake3 or --b3sum /path/to/b3sum")

    def check(path, data):
        # No shell, and paths are absolute. The bytes will also be SHA-256
        # checked after registration, so a copied input cannot bypass checks.
        result = subprocess.run([executable, "--no-names", str(path)],
                                check=True, capture_output=True, text=True)
        if regular_file(path) != data:
            fail(f"file changed during external BLAKE3 verification: {path}")
        digest = result.stdout.strip()
        return hex_hash(digest, "b3sum result")

    return check


def check_bounds(value, label, optional=False):
    if optional and value is None:
        return
    if not isinstance(value, dict):
        fail(f"invalid {label}")
    for edge in ("min", "max"):
        coordinates = value.get(edge)
        if (not isinstance(coordinates, list) or len(coordinates) != 3
                or any(type(n) is not int for n in coordinates)):
            fail(f"invalid {label}.{edge}")
    if any(a > b for a, b in zip(value["min"], value["max"])):
        fail(f"inverted {label}")


def dimensions(bounds):
    return [b - a + 1 for a, b in zip(bounds["min"], bounds["max"])] if bounds else None


def display_title(part):
    number = part["id"].removeprefix("part")
    if part["classification"] == "entity_quarantine":
        return "Unassigned plot entities"
    if part["classification"] == "door_candidate":
        size = dimensions(part.get("block_bounds") or part.get("bounds"))
        return f"Plot door {number} · {'×'.join(str(n) for n in size)}"
    return f"Plot fragment {number}"


def validate_inputs(root, extraction, source_path, check_blake3):
    if (extraction / "INCOMPLETE").exists():
        fail("extraction has an INCOMPLETE marker; all exported read-back checks must pass first")
    manifest_path = extraction / "manifest.json"
    manifest_bytes = regular_file(manifest_path)
    manifest = json.loads(manifest_bytes)
    if manifest.get("schema_version") != 1:
        fail("unsupported extraction manifest schema")
    verification = manifest.get("verification", {})
    if verification.get("status") != "passed":
        fail("extraction verification did not pass")
    for field in SUCCESS_FLAGS:
        if verification.get(field) is not True:
            fail(f"missing successful extraction check: {field}")
    source = manifest.get("source", {})
    original = regular_file(source_path)
    if sha256(original) != ORIGINAL_SHA256:
        fail("source file is not the pinned original WorldRecordPlot")
    archive_original = root / "sources" / ORIGINAL_SOURCE_ID / "files" / ORIGINAL_FILE
    archived = regular_file(archive_original)
    if archived != original:
        fail("archive original differs from the provided source file")
    if check_blake3(source_path, original) != hex_hash(source.get("file_blake3"), "source BLAKE3"):
        fail("extraction source BLAKE3 does not match the pinned original")
    if source.get("bytes") != len(original) or source.get("data_version") != 4671:
        fail("extraction source size/data version mismatch")
    if source.get("selection_bounds") != {"min": [0, 0, 0], "max": [283, 207, 294]}:
        fail("extraction source selection bounds mismatch")
    for field, count in EXPECTED_COUNTS.items():
        if source.get(field) != count or verification.get(field) != count:
            fail(f"source or verified aggregate {field} mismatch")
    original_manifest = json.loads(regular_file(root / "sources" / ORIGINAL_SOURCE_ID / "manifest.json"))
    original_records = [r for r in original_manifest.get("files", []) if r.get("path") == ORIGINAL_FILE]
    if (len(original_records) != 1 or original_records[0].get("sha256") != ORIGINAL_SHA256
            or original_records[0].get("local_path") != str(archive_original.relative_to(root))):
        fail("archive source manifest does not identify the pinned original")
    isolation = manifest.get("isolation", {})
    for field, value in {
        "api": "UniversalSchematic::split_connected_attach_nearby",
        "connectivity": "Corner", "min_standalone_blocks": 16, "max_air_gap": 3,
        "coordinates": "original_source_coordinates_without_rebasing", "inventory_repairs": "none",
    }.items():
        if isolation.get(field) != value:
            fail(f"unexpected isolation setting {field}: {isolation.get(field)!r}")
    parts = manifest.get("parts")
    if not isinstance(parts, list) or not parts:
        fail("extraction has no part records")
    ids, filenames, assigned_entities = set(), set(), set()
    payloads = []
    totals = Counter()
    for part in parts:
        part_id, filename = part.get("id"), part.get("file")
        if not isinstance(part_id, str) or not re.fullmatch(r"part[0-9]{4,}", part_id):
            fail("invalid part ID")
        if filename != f"{part_id}.litematic" or PurePosixPath(filename).name != filename:
            fail(f"unsafe or mismatched filename for {part_id}")
        if part_id in ids or filename in filenames:
            fail(f"duplicate part ID/filename: {part_id}")
        ids.add(part_id)
        filenames.add(filename)
        for field, value in PART_CHECKS.items():
            if part.get("verification", {}).get(field) != value:
                fail(f"{part_id}: missing successful read-back check {field}")
        for field in EXPECTED_COUNTS:
            value = part.get(field)
            if type(value) is not int or value < 0:
                fail(f"{part_id}: invalid {field}")
            totals[field] += value
        if part["preserved_block_states"] != part["non_air_blocks"] + part["explicit_air_states"]:
            fail(f"{part_id}: explicit air and non-air state counts do not sum")
        classification = part.get("classification")
        pistons = part.get("pistons", {}).get("total")
        if type(pistons) is not int or pistons < 0:
            fail(f"{part_id}: invalid piston count")
        if classification not in ("door_candidate", "other_component", "entity_quarantine"):
            fail(f"{part_id}: unknown classification")
        if (classification == "door_candidate") != (pistons > 0):
            fail(f"{part_id}: door classification does not match piston evidence")
        if classification == "entity_quarantine" and (part["non_air_blocks"] != 0 or part["entities"] == 0):
            fail(f"{part_id}: invalid entity quarantine")
        if classification != "entity_quarantine" and part["non_air_blocks"] == 0:
            fail(f"{part_id}: empty non-quarantine component")
        totals[classification] += 1
        check_bounds(part.get("bounds"), f"{part_id}.bounds")
        check_bounds(part.get("block_bounds"), f"{part_id}.block_bounds", optional=classification == "entity_quarantine")
        indices = part.get("entity_source_indices", [])
        if len(indices) != part["entities"]:
            fail(f"{part_id}: entity assignment count mismatch")
        for index in indices:
            if type(index) is not int or not 0 <= index < EXPECTED_COUNTS["entities"] or index in assigned_entities:
                fail(f"{part_id}: duplicate or invalid source entity index")
            assigned_entities.add(index)
        path = extraction / filename
        data = regular_file(path)
        if part.get("bytes") != len(data):
            fail(f"{part_id}: extracted file size changed")
        if check_blake3(path, data) != hex_hash(part.get("file_blake3"), f"{part_id} BLAKE3"):
            fail(f"{part_id}: extracted file BLAKE3 mismatch")
        for field in ("content_blake3", "block_entities_blake3", "entities_blake3"):
            hex_hash(part.get(field), f"{part_id} {field}")
        payloads.append((part, data, sha256(data)))
    for field, count in EXPECTED_COUNTS.items():
        if totals[field] != count:
            fail(f"part aggregate {field} mismatch")
    if assigned_entities != set(range(EXPECTED_COUNTS["entities"])):
        fail("source entity assignment coverage is incomplete")
    if totals["entity_quarantine"] != manifest.get("quarantine_count"):
        fail("quarantine count mismatch")
    if len(parts) - totals["entity_quarantine"] != manifest.get("component_count"):
        fail("component count mismatch")
    return manifest, manifest_bytes, original_manifest["source"], payloads


def describe_registration(root, extraction, manifest, manifest_bytes, original_source, payloads):
    source = {
        "id": SOURCE_ID,
        "url": original_source["url"],
        "category": "isolated WorldRecordPlot candidates and retained fragments",
        "provenance": "Local derivatives of the unchanged user-provided WorldRecordPlot; extracted with Nucleation's part isolator, without coordinate rebasing or inventory repairs. Source author/license unverified.",
        "license": "unknown; local research copy",
        "local_only": True,
        "derived_from": {"source": ORIGINAL_SOURCE_ID, "path": ORIGINAL_FILE, "sha256": ORIGINAL_SHA256,
                         "blake3": manifest["source"]["file_blake3"], "data_version": 4671},
        "extraction_manifest_sha256": sha256(manifest_bytes),
        "extraction": manifest["isolation"],
    }
    records, curated = [], []
    tags_count = Counter()
    for part, data, sha in payloads:
        local_path = f"sources/{SOURCE_ID}/files/{part['file']}"
        object_path = f"objects/sha256/{sha[:2]}/{sha}"
        records.append({
            "path": part["file"], "kind": "schematic", "status": "downloaded",
            "origin": "local_part_isolator_derivative", "bytes": len(data), "sha256": sha,
            "blake3": part["file_blake3"], "source_url": original_source["url"],
            "local_path": local_path, "object_path": object_path,
            "validation": "source_partition_and_file_readback_verified; not_simulated",
            "local_only": True,
        })
        tags = ["world-record-parts", "user-provided", "part-isolator", "local-only", part["classification"]]
        if part["classification"] == "door_candidate":
            tags.extend(["door", "piston"])
        if part.get("observers", 0):
            tags.append("observer")
        if part.get("containers", {}).get("count", 0):
            tags.append("inventories")
        if part["entities"]:
            tags.append("entities")
        tags.extend(part.get("review_tags", []))
        tags_count.update(part.get("review_tags", []))
        row = dict(part)
        title = display_title(part)
        row.update({
            "source": SOURCE_ID, "path": part["file"], "local_path": local_path,
            "sha256": sha, "source_url": original_source["url"],
            "title": title, "name": title, "display_name": title,
            "local_only": True, "tags": sorted(set(tags)), "readiness": "unvalidated",
            "stage": "isolated_source", "evidence_basis": "Existing Corner/16/3 part isolator; exact source partition and exported file read-back checks. No functional or vanilla simulation performed.",
            "priority": "medium" if part["classification"] == "door_candidate" else "low",
            "native_data_version": 4671, "data_version": 4671,
            "dimensions": dimensions(part.get("bounds")),
            "isolation": manifest["isolation"], "derived_from": source["derived_from"],
            "functional_status": "unverified", "simulation_status": "not_run", "vanilla_status": "not_run",
            "import_status": "readback_verified", "benchmark_status": "not_admitted",
            "static": {
                "format": "litematic", "native_data_version": 4671,
                "dimensions": dimensions(part.get("bounds")),
                "non_air_blocks": part["non_air_blocks"], "block_entities": part["block_entities"],
                "preserved_block_states": part["preserved_block_states"], "explicit_air_states": part["explicit_air_states"],
                "entities": part["entities"], "bounds": part.get("bounds"),
                "block_counts": part.get("block_histogram", {}),
                "control_count": part.get("controls", {}).get("count", 0),
                "inspection": "Rust production import, part isolation and exact exported read-back",
                "sha256_matches_manifest": True,
            },
            "test_plan": "Inspect component dependencies and travel halo, identify controls and intended aperture, establish source inventory/entity requirements, then capture full open/close/reset cycles and idle controls in Minecraft 1.21.11 before benchmark admission.",
            "warnings": [
                "A connected component is not proof of a complete or working door.",
                "The approximate source selection may have cut mechanisms at the plot boundary.",
                "Saved empty/absent inventories, controls, entities and outdated mechanics require review; no repair was made.",
            ],
            "license_status": "Unknown; local research derivative, not redistributed in Git.",
        })
        curated.append(row)
    archive_manifest = {
        "source": source, "files": records, "selected_files": len(records),
        "downloaded": len(records), "schematics": len(records), "layouts": 0, "errors": 0,
        "downloaded_bytes": sum(r["bytes"] for r in records),
        "extraction_verification": manifest["verification"],
    }
    curation = {
        "schema_version": 1, "curator": SOURCE_ID,
        "scope": "All verified exported parts, including non-door fragments and entity quarantine; functionality remains unvalidated.",
        "source": source, "isolation": manifest["isolation"], "entries": curated,
    }
    summary = {
        "schema_version": 1, "source": SOURCE_ID, "original_sha256": ORIGINAL_SHA256,
        "extraction_manifest_sha256": sha256(manifest_bytes),
        "files": len(records), "component_count": manifest["component_count"],
        "quarantine_count": manifest["quarantine_count"],
        "classifications": dict(sorted(Counter(p["classification"] for p, _, _ in payloads).items())),
        "totals": EXPECTED_COUNTS, "review_tags": dict(sorted(tags_count.items())),
        "native_data_version": 4671, "isolation": manifest["isolation"],
        "extraction_verification": manifest["verification"],
        "registered_bytes": archive_manifest["downloaded_bytes"],
        "functional_status": "unverified", "vanilla_status": "not_run",
        "benchmark_status": "not_admitted", "local_only": True,
    }
    return archive_manifest, curation, summary


def reproducible_gzip(data):
    output = io.BytesIO()
    # Empty filename, fixed timestamp and GzipFile's portable header prevent
    # build-machine paths/timestamps from changing the committed report bytes.
    with gzip.GzipFile(filename="", mode="wb", fileobj=output, compresslevel=9, mtime=0) as gz:
        gz.write(data)
    return output.getvalue()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--extraction", type=Path, required=True)
    parser.add_argument("--source-file", type=Path, help="Default: the archive's pinned original file; byte identity permits relocation from the extraction host")
    parser.add_argument("--b3sum", type=Path, help="Optional BLAKE3 verifier executable")
    parser.add_argument("--check-only", action="store_true")
    parser.add_argument("--report-summary", type=Path, help="Optional reproducible metadata-only summary JSON; never overwrite different bytes")
    parser.add_argument("--report-catalog", type=Path, help="Optional reproducible metadata-only extraction/curation report .json.gz; no schematic binaries")
    args = parser.parse_args()
    root, extraction = args.archive.resolve(), args.extraction.resolve()
    if not root.is_dir() or not extraction.is_dir():
        fail("archive and extraction directories must exist")
    source_path = (args.source_file or root / "sources" / ORIGINAL_SOURCE_ID / "files" / ORIGINAL_FILE).absolute()
    manifest, raw_manifest, original_source, payloads = validate_inputs(root, extraction, source_path, blake3_checker(args.b3sum))
    archive_manifest, curation, summary = describe_registration(root, extraction, manifest, raw_manifest, original_source, payloads)
    source_dir = root / "sources" / SOURCE_ID
    writes = [
        (source_dir / "source.json", json_bytes(archive_manifest["source"])),
        (source_dir / "extraction-manifest.json", raw_manifest),
        (root / "curation" / f"{SOURCE_ID}.json", json_bytes(curation)),
    ]
    if args.report_summary:
        writes.append((args.report_summary.absolute(), json_bytes(summary)))
    if args.report_catalog:
        report = {"summary": summary, "extraction": manifest, "curation": curation}
        writes.append((args.report_catalog.absolute(), reproducible_gzip(json_bytes(report))))
    # The archive index sees only completed registration: manifest is last.
    writes.append((source_dir / "manifest.json", json_bytes(archive_manifest)))
    # Preflight every destination before writing any content, including reruns.
    for path, data in writes:
        identical_destination(path, data)
    for (_, data, _), record in zip(payloads, archive_manifest["files"]):
        identical_destination(root / record["object_path"], data)
        identical_destination(root / record["local_path"], data)
    if args.check_only:
        print(json.dumps(dict(summary, registration="check_only_no_files_written"), indent=2))
        return
    for (_, data, _), record in zip(payloads, archive_manifest["files"]):
        hardlink_object(root / record["object_path"], root / record["local_path"], data)
    for path, data in writes:
        immutable_write(path, data)
    print(json.dumps(dict(summary, registration="registered"), indent=2))


if __name__ == "__main__":
    main()
