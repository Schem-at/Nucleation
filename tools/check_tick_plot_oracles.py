#!/usr/bin/env python3
"""Generate, run and independently audit paired plot-door vanilla captures.

The source files remain untouched. All coordinates in a spec are original plot
coordinates; the production export is rebased and the vanilla origin restores
that translation. A parity match alone never earns a working-door status.
"""
from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import pathlib
import shutil
import subprocess
import sys
import time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent / "gametest"))
from run_community_capture12111 import validate_capture, validate_scenario

AIR = {"minecraft:air", "minecraft:cave_air", "minecraft:void_air"}
SCOPE = ("Every captured tick-end block, comparator output and flat occupied inventory slot; "
         "no vanilla entity census, nested item-component, moving-BE payload or intra-tick parity.")


def load(path):
    return json.loads(pathlib.Path(path).read_text())


def digest(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def save(path, value):
    path = pathlib.Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")


def selected(specs, names):
    rows = load(specs)["cases"]
    if names:
        unknown = set(names) - {row["case"] for row in rows}
        if unknown:
            raise ValueError(f"Unknown cases: {sorted(unknown)}")
        rows = [row for row in rows if row["case"] in names]
    return rows


def box_cells(box):
    return list(itertools.product(*(range(a, b + 1) for a, b in zip(box["min"], box["max"]))))


def relative(position, origin):
    return [p - o for p, o in zip(position, origin)]


def generate(spec, fixture_path, source_path, out):
    fixture = load(fixture_path)
    if digest(source_path) != spec["sha256"]:
        raise ValueError("Original source hash changed")
    if fixture["data_version"] != 4671 or fixture["entity_count"]:
        raise ValueError("Only native 4671 zero-authored-entity fixtures supported")
    origin = spec["bounds"]["min"]
    size = [b - a + 1 for a, b in zip(origin, spec["bounds"]["max"])]
    if fixture["size"] != size:
        raise ValueError("Production export bounds disagree with independently curated source")
    blocks = {tuple(b["pos"]): b["state"] for b in fixture["fixture"]}
    control = relative(spec["lever"], origin)
    if not blocks.get(tuple(control), "").startswith("minecraft:lever["):
        raise ValueError("Curated control is not an actual exported lever")
    aperture = [relative(p, origin) for p in box_cells(spec["aperture"])]
    if not aperture or any(blocks.get(tuple(p), "minecraft:air") not in AIR for p in aperture):
        raise ValueError("Curated initially-open aperture is not all air")
    for mode in ["cycle", "idle"]:
        scenario = {
            "format_version": 1, "data_version": 4671, "case": spec["case"] + "_" + mode,
            "negative_control": mode == "idle", "size": size, "origin": origin,
            "watch": {"min": [-3] * 3, "max": [s + 2 for s in size]},
            "rng_seed": 0, "capture_ticks": spec.get("ticks", 660),
            "source": {"sha256": spec["sha256"], "data_version": 4671,
                       "archive_path": spec["source"], "plot_source_sha256": spec["plot_sha256"]},
            "fixture": fixture["fixture"],
            "actions": ([{"tick": t, "type": "pull_lever", "pos": control}
                         for t in spec.get("pull_ticks", [21, 181, 341, 501])]
                        if mode == "cycle" else []),
        }
        validate_scenario(scenario)
        save(pathlib.Path(out) / f"{mode}-scenario.json", scenario)
    save(pathlib.Path(out) / "checks.json", {
        "spec": spec, "source_sha256": digest(source_path), "fixture_json_sha256": digest(fixture_path),
        "aperture_local": aperture, "lever_local": control,
        "origin_preserves_source_coordinates": True,
        "criteria": "Four actual lever pulls, two repeatable open/closed cycles, idle unchanged, "
                    "full captured-state return, stable endpoints, complete declared panel material at closed endpoint.",
    })


def frame_state(frame):
    return {name: frame[name] for name in ["blocks", "inventories", "comparator_outputs"]}


def audit(run, spec_override=None, output=None):
    run = pathlib.Path(run)
    checks = load(run / "checks.json")
    spec = spec_override or checks["spec"]
    if spec_override:
        for key in ["case", "source", "sha256", "bounds", "lever"]:
            if spec[key] != checks["spec"][key]:
                raise ValueError(f"Revised aperture oracle may not change {key}")
    captures, structural = {}, {}
    for mode in ["cycle", "idle"]:
        scenario_path = run / f"{mode}-scenario.json"
        capture_path = run / mode / "capture.json"
        structural[mode] = validate_capture(capture_path, load(scenario_path), digest(scenario_path))
        captures[mode] = load(capture_path)
    cycle, idle = captures["cycle"], captures["idle"]
    frames = [cycle["initial"], *cycle["snapshots"]]
    idle_frames = [idle["initial"], *idle["snapshots"]]
    pulls = [a["tick"] for a in cycle["actions"]]
    if len(pulls) != 4 or any(a["type"] != "pull_lever" for a in cycle["actions"]):
        raise ValueError("Functional oracle expects four real lever pulls")
    endpoints = [pulls[1] - 1, pulls[2] - 1, pulls[3] - 1, cycle["capture_ticks"]]
    initial = frame_state(frames[0])
    aperture = [tuple(relative(p, cycle["origin"])) for p in box_cells(spec["aperture"])]
    projection = spec.get("closure_projection_axis")
    if projection is not None and projection not in [0, 1, 2]:
        raise ValueError("Projection must be an axis index")
    closed_names = set(spec.get("closed_blocks", ["minecraft:quartz_block"]))
    assertions = []

    def require(name, passed, details=None):
        assertions.append({"name": name, "passed": bool(passed), "details": details})

    require("source_and_initial_match_between_runs", cycle["fixture"] == idle["fixture"]
            and initial == frame_state(idle_frames[0]))
    changed_idle = [f["tick"] for f in idle_frames if frame_state(f) != initial]
    require("idle_remains_unchanged", not changed_idle, {"changed_ticks": len(changed_idle),
            "first_changed_tick": changed_idle[0] if changed_idle else None})
    require("pre_input_remains_unchanged", all(frame_state(f) == initial for f in frames[:pulls[0]]))
    for number, tick in enumerate(endpoints):
        frame = frames[tick]
        blocks = {tuple(r[:3]): r[3] for r in frame["blocks"]}
        states = [blocks.get(p, "minecraft:air") for p in aperture]
        is_closed = number % 2 == 0
        rays = {}
        for p, state in zip(aperture, states):
            key = tuple(v for axis, v in enumerate(p) if axis != projection)
            rays[key] = rays.get(key, False) or state.split("[")[0] in closed_names
        closed = all(rays.values()) if projection is not None else all(s.split("[")[0] in closed_names for s in states)
        require(f"endpoint_{tick}_{'closed' if is_closed else 'open'}_aperture",
                closed if is_closed else all(s in AIR for s in states),
                {"cells": len(aperture), "air_cells": sum(s in AIR for s in states),
                 "allowed_closed_cells": sum(s.split("[")[0] in closed_names for s in states),
                 "projection_axis": projection, "rays": len(rays),
                 "blocked_rays": sum(rays.values())})
        require(f"endpoint_{tick}_stable_last_16_ticks",
                all(frame_state(f) == frame_state(frame) for f in frames[max(0, tick - 15):tick + 1]))
        require(f"endpoint_{tick}_no_moving_piston", not any("minecraft:moving_piston[" in r[3] for r in frame["blocks"]))
        if not is_closed:
            require(f"endpoint_{tick}_full_state_returns", frame_state(frame) == initial)
    require("closed_endpoints_repeat", frame_state(frames[endpoints[0]]) == frame_state(frames[endpoints[2]]))
    lever = tuple(checks["lever_local"])
    original = {tuple(r[:3]): r[3] for r in frames[0]["blocks"]}
    active_positions = set()
    piston_ticks = []
    for frame in frames[1:]:
        blocks = {tuple(r[:3]): r[3] for r in frame["blocks"]}
        active_positions.update(p for p in original.keys() | blocks.keys()
                                if p != lever and original.get(p) != blocks.get(p))
        if any("minecraft:moving_piston[" in r[3] for r in frame["blocks"]):
            piston_ticks.append(frame["tick"])
    require("non_control_activity", bool(active_positions), len(active_positions))
    require("piston_motion_observed", bool(piston_ticks), {"ticks_with_moving_piston": len(piston_ticks)})
    passed = all(a["passed"] for a in assertions)
    result = {"case": spec["case"], "source_sha256": spec["sha256"],
              "status": "vanilla_functional_pass" if passed else "vanilla_functional_failed",
              "assertions": assertions, "captures": structural, "scope": SCOPE,
              "aperture": spec["aperture"], "closure_projection_axis": projection,
              "oracle_spec": spec, "revised_after_capture": bool(spec_override),
              "original_oracle_spec": checks["spec"] if spec_override else None,
              "origin": cycle["origin"],
              "pull_ticks": pulls, "endpoint_ticks": endpoints,
              "source_unchanged": True, "repairs": []}
    save(output or run / "functional.json", result)
    return result


def run_all(args):
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=True)
    # Builds elsewhere in this worktree must not change a running comparison.
    binaries = out / "binaries"
    binaries.mkdir(exist_ok=False)
    for name in ["exporter", "checker"]:
        destination = binaries / name
        shutil.copy2(getattr(args, name), destination)
        setattr(args, name, destination)
    results = []
    metadata = {"runner_sha256": digest(__file__), "specs_sha256": digest(args.specs),
                "exporter_sha256": digest(args.exporter), "checker_sha256": digest(args.checker),
                "scope": SCOPE, "cases": results}
    save(out / "run.json", metadata)
    for spec in selected(args.specs, args.case):
        case_dir = out / spec["case"]
        case_dir.mkdir(exist_ok=False)
        source = args.archive / spec["source"]
        result = {"case": spec["case"], "source_sha256": spec["sha256"], "status": "started"}
        results.append(result)
        started = time.monotonic()

        def execute(argv, label, timeout):
            with (case_dir / (label + ".stdout.log")).open("w") as stdout, (case_dir / (label + ".stderr.log")).open("w") as stderr:
                process = subprocess.run([str(x) for x in argv], stdout=stdout, stderr=stderr, timeout=timeout)
            if process.returncode:
                raise RuntimeError(f"{label} exited {process.returncode}")

        try:
            execute([args.exporter, source, case_dir / "fixture.json", case_dir / "fixture.snbt"], "export", 120)
            generate(spec, case_dir / "fixture.json", source, case_dir)
            comparisons = {}
            for mode in ["cycle", "idle"]:
                execute([sys.executable, pathlib.Path(__file__).parent / "gametest/run_community_capture12111.py",
                         "--runtime", args.runtime, "--jdk", args.jdk,
                         "--scenario", case_dir / f"{mode}-scenario.json", "--source-file", source,
                         "--output", case_dir / mode, "--timeout", str(args.timeout)], mode + "-capture", args.timeout + 60)
                capture = case_dir / mode / "capture.json"
                try:
                    execute([args.checker, "--capture", capture], mode + "-native", 180)
                except RuntimeError:
                    pass  # A complete mismatch JSON is evidence, not a reason to skip the idle control.
                comparisons[mode] = load(case_dir / (mode + "-native.stdout.log"))
            functional = audit(case_dir)
            result.update({"functional": functional, "native": comparisons,
                           "status": ("functional_and_parity_pass" if functional["status"] == "vanilla_functional_pass"
                                      and all(r["status"] == "matches_capture" for r in comparisons.values())
                                      else "not_admitted")})
        except Exception as error:
            result.update({"status": "run_failed", "error": repr(error)})
        result["elapsed_seconds"] = round(time.monotonic() - started, 3)
        save(case_dir / "result.json", result)
        save(out / "run.json", metadata)
        print(json.dumps({"case": spec["case"], "status": result["status"], "elapsed_seconds": result["elapsed_seconds"]}), flush=True)
    return metadata


def recheck(runs, checker, output, extras):
    runs, output = pathlib.Path(runs).resolve(), pathlib.Path(output).resolve()
    output.mkdir(parents=True, exist_ok=False)
    binary = output / "checker"
    shutil.copy2(checker, binary)
    result = {"checker_sha256": digest(binary), "scope": SCOPE, "captures": []}
    paths = [(str(p.relative_to(runs)), p) for p in sorted(runs.rglob("capture.json"))]
    paths.extend(("extra/" + str(i), p.resolve()) for i, p in enumerate(extras))
    for index, (relative_path, path) in enumerate(paths):
        process = subprocess.run([str(binary), "--capture", str(path)],
                                 capture_output=True, text=True, timeout=180)
        (output / f"{index:03}.stderr.log").write_text(process.stderr)
        (output / f"{index:03}.stdout.json").write_text(process.stdout)
        row = {"artifact_path": relative_path, "capture_sha256": digest(path),
               "returncode": process.returncode, "report": json.loads(process.stdout)}
        result["captures"].append(row)
        save(output / "results.json", result)
        print(relative_path, row["report"]["status"], flush=True)
    return result


def report(runs, output, native_results=None):
    runs = pathlib.Path(runs)
    results = []
    refreshed = load(native_results) if native_results else None
    refreshed_by_path = {r["artifact_path"]: r for r in refreshed["captures"]} if refreshed else {}
    for path in sorted(runs.glob("batch*/*/result.json")):
        raw = load(path)
        revised = path.parent / "functional-revised.json"
        functional = load(revised) if revised.exists() else raw.get("functional", {})
        metadata = load(path.parent.parent / "run.json")
        native = raw.get("native", {})
        original_native = native
        if refreshed:
            native = {mode: refreshed_by_path[str((path.parent / mode / "capture.json").relative_to(runs))]["report"]
                      for mode in ["cycle", "idle"]}
        results.append({
            "case": raw["case"], "source_sha256": raw["source_sha256"],
            "artifact_directory": str(path.parent.relative_to(runs)),
            "functional_status": functional.get("status", "not_completed"),
            "functional_failures": [a for a in functional.get("assertions", []) if not a["passed"]],
            "oracle_revised_after_capture": functional.get("revised_after_capture", False),
            "native": native, "captures": functional.get("captures", {}),
            "native_before_fix": original_native if refreshed else None,
            "aperture": functional.get("aperture"),
            "closure_projection_axis": functional.get("closure_projection_axis"),
            "origin": functional.get("origin"), "pull_ticks": functional.get("pull_ticks"),
            "endpoint_ticks": functional.get("endpoint_ticks"),
            "benchmark_admission": ("accepted" if functional.get("status") == "vanilla_functional_pass"
                                    and len(native) == 2 and all(r["status"] == "matches_capture" for r in native.values())
                                    else "not_admitted"),
            "run_status": raw["status"], "run_error": raw.get("error"),
            "binaries": {k: metadata[k] for k in ["exporter_sha256", "checker_sha256"]},
            "rechecked_checker_sha256": refreshed["checker_sha256"] if refreshed else None,
            "rechecked_checker_revision": refreshed.get("source_revision") if refreshed else None,
            "rechecked_checker_revision_note": refreshed.get("source_revision_note") if refreshed else None,
        })
    result = {"format_version": 1, "scope": SCOPE, "cases": results,
              "counts": {"cases": len(results),
                         "vanilla_functional_pass": sum(r["functional_status"] == "vanilla_functional_pass" for r in results),
                         "native_cycle_matches": sum(r["native"].get("cycle", {}).get("status") == "matches_capture" for r in results),
                         "native_idle_matches": sum(r["native"].get("idle", {}).get("status") == "matches_capture" for r in results),
                         "admitted": sum(r["benchmark_admission"] == "accepted" for r in results)}}
    save(output, result)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    p = commands.add_parser("run")
    p.add_argument("--specs", type=pathlib.Path, required=True)
    p.add_argument("--case", action="append")
    for name in ["archive", "exporter", "checker", "runtime", "jdk", "output"]:
        p.add_argument("--" + name, type=pathlib.Path, required=True)
    p.add_argument("--timeout", type=int, default=300)
    p = commands.add_parser("audit")
    p.add_argument("run", type=pathlib.Path)
    p.add_argument("--specs", type=pathlib.Path)
    p.add_argument("--case")
    p.add_argument("--output", type=pathlib.Path)
    p = commands.add_parser("report")
    p.add_argument("--runs", type=pathlib.Path, required=True)
    p.add_argument("--output", type=pathlib.Path, required=True)
    p.add_argument("--native-results", type=pathlib.Path)
    p = commands.add_parser("recheck")
    p.add_argument("--runs", type=pathlib.Path, required=True)
    p.add_argument("--checker", type=pathlib.Path, required=True)
    p.add_argument("--output", type=pathlib.Path, required=True)
    p.add_argument("--extra-capture", action="append", type=pathlib.Path, default=[])
    args = parser.parse_args()
    if args.command == "run":
        run_all(args)
    elif args.command == "audit":
        if bool(args.specs) != bool(args.case):
            parser.error("Revised audit requires both --specs and --case")
        override = selected(args.specs, [args.case])[0] if args.specs else None
        result = audit(args.run, override, args.output)
        print(json.dumps(result, indent=2))
        return int(result["status"] != "vanilla_functional_pass")
    elif args.command == "report":
        result = report(args.runs, args.output, args.native_results)
        print(json.dumps(result["counts"]))
    else:
        recheck(args.runs, args.checker, args.output, args.extra_capture)
    return 0


if __name__ == "__main__":
    sys.exit(main())
