#!/usr/bin/env python3
"""Check the retained interlocking-door vanilla captures, without running Minecraft.

This is deliberately specific to the two pinned captures. It verifies functional
properties of their tick-end records, not engine parity, timing performance,
runtime entities, item components, moving block entities, or intra-tick ordering.

python3 tools/validate_tick_community.py \
  --captures /path/to/archive/validation/candidate-run \
  --output docs/performance/schematic-candidates/validation/door-functional.json
"""

import argparse
import hashlib
import json
from pathlib import Path
import sys


CAPTURES = {
    "idle": ("vanilla-idle/capture.json",
             "d137b334f7ccac221f1ce6098bd2a8f99df6d12f9c2f01ca01cc3554388d9e15"),
    "cycle": ("vanilla-cycle/capture.json",
              "39602a9fd99caac0d1582600ac6f49c262a4a2c5a3bc789992ceeb26739a092c"),
}
SOURCE_SHA256 = "e7b964e92109edebd0860f1ec95c158eebd8f66436e4c6c5b7c36862b7675586"
FIELDS = ("blocks", "inventories", "comparator_outputs")
PANEL = [(x, y, 1) for x in range(3, 6) for y in range(5, 8)]
PANEL_STATE = "minecraft:waxed_oxidized_copper"
SCOPE = {
    "included": [
        "Retained Minecraft 1.21.11 tick-end block descriptors within the inclusive watch box",
        "Comparator output values and simple occupied inventory slot IDs/counts",
        "Recorded zero-player count at every captured boundary",
        "Door panel occupancy, repeated return to the initial maps, and idle stability",
    ],
    "excluded": [
        "Nucleation-versus-vanilla equivalence (requires the separate Rust capture checker)",
        "Runtime entity census/equality; entity_count describes the authored fixture only",
        "Item components, recursive item inventories, and moving block-entity payload equality",
        "Intra-tick notification order, intermediate states between captures, and cells outside watch",
        "Performance measurements or a newly executed Minecraft capture",
    ],
}


class ValidationError(Exception):
    def __init__(self, check, expected, actual, **context):
        self.detail = dict(check=check, expected=expected, actual=actual, **context)
        super().__init__(check)


def require(condition, check, expected, actual, **context):
    if not condition:
        raise ValidationError(check, expected, actual, **context)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, "unique JSON keys", "unique key", key)
        result[key] = value
    return result


def load_capture(root, mode):
    relative, expected_hash = CAPTURES[mode]
    path = root / relative
    require(path.stat().st_size <= 16 * 1024 * 1024, "capture size", "at most 16 MiB",
            path.stat().st_size, case=mode)
    raw = path.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    require(digest == expected_hash, "retained capture SHA-256", expected_hash, digest, case=mode)
    data = json.loads(raw, object_pairs_hook=unique_object,
                      parse_constant=lambda value: (_ for _ in ()).throw(ValueError(value)))
    return data


def in_watch(pos, watch):
    return (len(pos) == 3 and all(type(v) is int for v in pos)
            and all(watch["min"][i] <= pos[i] <= watch["max"][i] for i in range(3)))


def frame_maps(frame, watch, mode, tick):
    require(type(frame.get("tick")) is int and frame["tick"] == tick,
            "consecutive tick boundaries", tick, frame.get("tick"), case=mode)
    require(type(frame.get("players")) is int and frame["players"] == 0,
            "zero recorded players", 0, frame.get("players"), case=mode, tick=tick)
    maps = {}
    for field, width in (("blocks", 4), ("inventories", 6), ("comparator_outputs", 4)):
        rows = frame.get(field)
        require(isinstance(rows, list), "snapshot rows", "array", type(rows).__name__,
                case=mode, tick=tick, field=field)
        result = {}
        for row in rows:
            require(isinstance(row, list) and len(row) == width, "snapshot row width", width,
                    row, case=mode, tick=tick, field=field)
            pos = tuple(row[:3])
            require(in_watch(pos, watch), "position inside watch", watch, row[:3],
                    case=mode, tick=tick, field=field)
            if field == "blocks":
                key, value = pos, row[3]
                require(isinstance(value, str) and value.startswith("minecraft:")
                        and value != "minecraft:air", "non-air block descriptor",
                        "non-air namespaced descriptor", value, case=mode, tick=tick, position=pos)
            elif field == "inventories":
                require(type(row[3]) is int and 0 <= row[3] <= 255
                        and isinstance(row[4], str) and row[4].startswith("minecraft:")
                        and type(row[5]) is int and 0 < row[5] <= 255,
                        "occupied inventory slot", "slot,item,count", row,
                        case=mode, tick=tick, position=pos)
                key, value = (*pos, row[3]), (row[4], row[5])
            else:
                key, value = pos, row[3]
                require(type(value) is int and 0 <= value <= 15, "vanilla comparator strength",
                        "0..15", value, case=mode, tick=tick, position=pos)
            require(key not in result, "unique snapshot key", "unique position/slot", key,
                    case=mode, tick=tick, field=field)
            result[key] = value
        maps[field] = result
    for field in ("inventories", "comparator_outputs"):
        for key in maps[field]:
            require(key[:3] in maps["blocks"], "block for inventory/comparator", "non-air block",
                    None, case=mode, tick=tick, field=field, position=key[:3])
    return maps


def same_maps(expected, actual, mode, tick, check):
    for field in FIELDS:
        for key in sorted(expected[field].keys() | actual[field].keys()):
            before, after = expected[field].get(key), actual[field].get(key)
            require(before == after, check, before, after,
                    case=mode, tick=tick, field=field, position=key[:3],
                    **({"slot": key[3]} if field == "inventories" else {}))


def canonical_hash(maps):
    rows = {field: [[*key, value] for key, value in sorted(maps[field].items())]
            for field in FIELDS}
    return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def activity(frames):
    result = {}
    for field in FIELDS:
        ticks, entries = [], 0
        for tick in range(1, len(frames)):
            before, after = frames[tick - 1][field], frames[tick][field]
            changes = sum(before.get(key) != after.get(key) for key in before.keys() | after.keys())
            if changes:
                ticks.append(tick)
                entries += changes
        result[field] = {
            "changed_tick_boundaries": len(ticks), "changed_map_entries": entries,
            "first_changed_tick": ticks[0] if ticks else None,
            "last_changed_tick": ticks[-1] if ticks else None,
        }
    return result


def validate(captures):
    frames = {}
    expected_watch = {"min": [-2, -2, -2], "max": [10, 14, 6]}
    expected_actions = [{"tick": t, "type": "pull_lever", "pos": [6, 1, 4]}
                        for t in (11, 91, 171, 251)]
    for mode, capture in captures.items():
        expected = {
            "format_version": 1, "mc_version": "1.21.11", "data_version": 4671,
            "case": "interlocking_" + mode, "negative_control": mode == "idle",
            "size": [9, 13, 5], "origin": [0, 100, 0], "rng_seed": 0,
            "capture_ticks": 320, "watch": expected_watch, "settle": "in_world",
            "action_boundary": "immediately before the numbered tick; initial snapshot is tick zero",
            "actions": [] if mode == "idle" else expected_actions,
            "entity_count": 0,
        }
        for field, value in expected.items():
            require(capture.get(field) == value, "retained capture metadata", value,
                    capture.get(field), case=mode, field=field)
        source = capture.get("source", {})
        require(source.get("sha256") == SOURCE_SHA256 and source.get("data_version") == 4671,
                "native source identity", {"sha256": SOURCE_SHA256, "data_version": 4671},
                source, case=mode)
        snapshots = capture.get("snapshots", [])
        require(isinstance(snapshots, list) and len(snapshots) == 320,
                "initial plus 320 tick-end frames", 320, len(snapshots), case=mode)
        frames[mode] = [frame_maps(frame, expected_watch, mode, tick)
                        for tick, frame in enumerate([capture["initial"], *snapshots])]
    require(captures["idle"]["fixture"] == captures["cycle"]["fixture"],
            "same authored fixture", "identical fixtures", "different fixture arrays")
    same_maps(frames["idle"][0], frames["cycle"][0], "cycle", 0, "same initial maps")
    for tick, maps in enumerate(frames["idle"]):
        same_maps(frames["idle"][0], maps, "idle", tick, "idle stability")
    for tick in (160, 320):
        same_maps(frames["cycle"][0], frames["cycle"][tick], "cycle", tick,
                  "complete initial-map restoration")
    for tick in (0, 80, 160, 240, 320):
        expected = PANEL_STATE if tick in (80, 240) else None
        for pos in PANEL:
            actual = frames["cycle"][tick]["blocks"].get(pos)
            require(actual == expected, "exact nine-cell panel phase", expected, actual,
                    case="cycle", tick=tick, position=pos)
    changes = {mode: activity(values) for mode, values in frames.items()}
    for field in FIELDS:
        require(changes["cycle"][field]["changed_tick_boundaries"] > 0,
                "positive activity", ">0 changed boundaries", 0, case="cycle", field=field)
    for start, end in ((1, 160), (161, 320)):
        require(any(frames["cycle"][tick]["blocks"] != frames["cycle"][tick - 1]["blocks"]
                    for tick in range(start, end + 1)), "activity in each cycle", True, False,
                case="cycle", ticks=[start, end])
    capture_reports = {}
    for mode, capture in captures.items():
        relative, digest = CAPTURES[mode]
        capture_reports[mode] = {
            "path_relative_to_capture_root": relative, "sha256": digest,
            "case": capture["case"], "frames_including_initial": len(frames[mode]),
            "capture_ticks": capture["capture_ticks"], "actions": capture["actions"],
            "scenario_sha256": capture["scenario_sha256"], "warmup_ticks": capture["warmup_ticks"],
            "initial_map_counts": {field: len(frames[mode][0][field]) for field in FIELDS},
            "phase_map_sha256": {str(tick): canonical_hash(frames[mode][tick])
                                 for tick in (0, 80, 160, 240, 320)},
            "activity": changes[mode],
        }
    cycle = captures["cycle"]
    return {
        "schema_version": 1, "validator": "tools/validate_tick_community.py", "result": "pass",
        "validation_kind": "retained vanilla capture functional assertions",
        "source": cycle["source"],
        "reference": {key: cycle[key] for key in
                      ("mc_version", "data_version", "java_runtime_version", "mappings_sha1",
                       "server_inner_sha256", "server_kind", "origin", "watch", "rng_seed")},
        "captures": capture_reports,
        "activity_counting": "Differences between consecutive tick-end maps; not a count of every intra-tick write",
        "phase_map_hash_encoding": "SHA-256 of UTF-8 compact sorted-key JSON mapping each field to sorted [key coordinates..., value] rows",
        "checks": {
            "initial_plus_320_consecutive_frames_each": True,
            "zero_recorded_players_all_642_boundaries": True,
            "same_authored_fixture_and_initial_maps": True,
            "idle_all_three_maps_unchanged": True,
            "cycle_all_three_maps_restored_at_ticks": [160, 320],
            "panel": {"positions": PANEL, "state": PANEL_STATE,
                      "all_nine_present_at_ticks": [80, 240], "all_nine_air_at_ticks": [0, 160, 320]},
            "positive_activity_in_both_cycles_and_all_three_map_types": True,
        },
        "scope": SCOPE,
        "reproduce": "python3 tools/validate_tick_community.py --captures <archive>/validation/candidate-run "
                     "--output docs/performance/schematic-candidates/validation/door-functional.json",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--captures", type=Path, required=True, help="Directory containing vanilla-idle and vanilla-cycle")
    parser.add_argument("--output", type=Path, help="Write deterministic full JSON report here")
    args = parser.parse_args()
    try:
        report = validate({mode: load_capture(args.captures, mode) for mode in CAPTURES})
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
        print(json.dumps(report, indent=2, sort_keys=True))
        return 0
    except (ValidationError, OSError, ValueError, KeyError, TypeError) as exc:
        failure = exc.detail if isinstance(exc, ValidationError) else {"check": "input", "error": str(exc)}
        print(json.dumps({"result": "fail", "failure": failure}, indent=2), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
