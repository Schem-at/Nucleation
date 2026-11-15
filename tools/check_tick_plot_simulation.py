#!/usr/bin/env python3
"""Run one timeout-isolated native process per distinct plot candidate SHA-256.

This is simulation triage, not a functional-door or vanilla-parity certification.
Compile on harrison-work; this script does not invoke Rust tooling.
"""
from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import platform
import signal
import subprocess
import sys
import time
from collections import Counter


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def save(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_suffix(path.suffix + ".tmp")
    temp.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    temp.replace(path)


def select(library: dict) -> list[dict]:
    return sorted(
        (row for row in library["entries"]
         if row.get("source") in {"world-record-parts", "world-record-floor-parts"}
         and row.get("classification") == "door_candidate"
         and "infrastructure_review" not in row.get("review_tags", [])),
        key=lambda row: (row["source"], row["path"]),
    )


def events_at(path: Path) -> list[dict]:
    events = []
    if path.exists():
        for line in path.read_text(errors="replace").splitlines():
            try:
                event = json.loads(line)
                if isinstance(event, dict):
                    events.append(event)
            except json.JSONDecodeError:
                pass  # retain non-JSON diagnostics in the original transcript
    return events


def evaluate(row: dict, aliases: list[dict], args: argparse.Namespace, run_config: dict) -> dict:
    key = row["sha256"]
    result_path = args.output / "results" / f"{key}.json"
    if args.resume and result_path.exists():
        saved = json.loads(result_path.read_text())
        if saved.get("run_config") == run_config and saved.get("source_sha256") == key:
            return saved
    source = args.inputs / row["local_path"]
    started = dt.datetime.now(dt.timezone.utc).isoformat()
    result = {
        "schema_version": 1, "source_sha256": key,
        "aliases": [{k: r.get(k) for k in ("source", "path", "local_path", "review_tags", "bounds",
                                              "derived_from", "parent_part", "inventory", "cutoff_face_contacts")}
                    for r in aliases],
        "started_utc": started, "run_config": run_config,
        "functional_status": "unproven", "vanilla_status": "not_checked",
    }
    try:
        actual = sha256(source)
    except OSError as exc:
        result.update(status="missing_input", detail=str(exc))
        save(result_path, result)
        return result
    if actual != key:
        result.update(status="source_hash_mismatch", actual_sha256=actual)
        save(result_path, result)
        return result
    transcript = args.output / "transcripts" / f"{key}.jsonl"
    stderr = args.output / "transcripts" / f"{key}.stderr.txt"
    transcript.parent.mkdir(parents=True, exist_ok=True)
    command = [str(args.binary), str(source), key, str(args.ticks), str(args.setup_ticks)]
    # Environment trace switches are process-cached by the engine; inherited
    # switches would alter observability and can generate unbounded output.
    env = {k: v for k, v in os.environ.items() if not k.startswith("MC_TICK_TRACE_") and k != "MC_TICK_DEBUG_COMPARATOR"}
    # prlimit applies limits without a thread-unsafe preexec_fn. Wall timeout
    # remains external and kills the complete process group on expiration.
    if sys.platform.startswith("linux") and args.memory_mb:
        command = ["prlimit", f"--as={args.memory_mb * 1024 * 1024}", "--", *command]
    begin = time.monotonic()
    status = None
    try:
        with transcript.open("wb") as out, stderr.open("wb") as err:
            child = subprocess.Popen(command, stdout=out, stderr=err, env=env, start_new_session=True)
            try:
                returncode = child.wait(timeout=args.timeout)
            except subprocess.TimeoutExpired:
                os.killpg(child.pid, signal.SIGKILL)
                returncode = child.wait()
                status = "process_timeout"
    except OSError as exc:
        status, returncode = "launch_error", None
        result["detail"] = str(exc)
    events = events_at(transcript)
    final = next((event for event in reversed(events) if event.get("type") == "result"), None)
    stages = [event for event in events if event.get("type") == "stage"]
    metadata = next((event["metadata"] for event in events if event.get("type") == "metadata"), None)
    partial = [event["trial"] for event in events if event.get("type") == "trial"]
    if status is None:
        status = ("process_failed" if returncode else final["assessment"]["status"] if final else "missing_native_result")
    result.update(status=status, returncode=returncode, elapsed_seconds=time.monotonic() - begin,
                  command=command, transcript=str(transcript.relative_to(args.output)),
                  stderr=str(stderr.relative_to(args.output)), last_stage=stages[-1] if stages else None,
                  metadata=metadata, native=final)
    if final is None:
        result["completed_partial_trials"] = partial
    save(result_path, result)
    return result


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--library", type=Path, required=True)
    parser.add_argument("--inputs", type=Path, required=True, help="Root containing archive-relative sources/ paths")
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--ticks", type=int, choices=(320, 480, 640), default=320)
    parser.add_argument("--setup-ticks", type=int, default=80)
    parser.add_argument("--timeout", type=float, default=120)
    parser.add_argument("--memory-mb", type=int, default=4096)
    parser.add_argument("--jobs", type=int, default=2, help="Independent candidate processes, never intra-tick parallelism")
    parser.add_argument("--only", action="append", default=[], help="Exact SHA or source/path (repeatable)")
    parser.add_argument("--resume", action="store_true", help="Reuse only results with identical binary, harness and settings")
    parser.add_argument("--revision", default="unknown", help="Git revision used to build the binary")
    args = parser.parse_args()
    if not 0 <= args.setup_ticks <= 320 or args.jobs < 1 or args.timeout <= 0:
        parser.error("Invalid setup, jobs or timeout budget")
    args.binary, args.inputs, args.output = args.binary.resolve(), args.inputs.resolve(), args.output.resolve()
    library = json.loads(args.library.read_text())
    candidates = select(library)
    if args.only:
        candidates = [r for r in candidates if r["sha256"] in args.only or f'{r["source"]}/{r["path"]}' in args.only]
        if not candidates:
            parser.error("--only matched no plot candidates")
    aliases: dict[str, list[dict]] = {}
    for row in candidates:
        aliases.setdefault(row["sha256"], []).append(row)
    config = {"schema_version": 1, "binary_sha256": sha256(args.binary), "harness_sha256": sha256(Path(__file__)),
              "revision": args.revision, "ticks": args.ticks, "setup_ticks": args.setup_ticks,
              "timeout_seconds": args.timeout, "memory_mb": args.memory_mb,
              "rng_seed": 0, "random_tick_speed": 0, "settle": "in_world", "jobs": args.jobs}
    save(args.output / "manifest.json", {"run_config": config, "library_sha256": sha256(args.library),
         "host": platform.uname()._asdict(), "candidate_paths": len(candidates), "distinct_sha256": len(aliases),
         "candidates": [{k: r.get(k) for k in ("source", "path", "local_path", "sha256")} for r in candidates],
         "qualification": "Native smoke/activity/replay assessment; functional status and vanilla parity remain unproven."})
    results = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        futures = {pool.submit(evaluate, rows[0], rows, args, config): key for key, rows in aliases.items()}
        for future in concurrent.futures.as_completed(futures):
            key = futures[future]
            try:
                result = future.result()
            except Exception as exc:
                result = {"source_sha256": key, "status": "runner_error", "detail": repr(exc),
                          "functional_status": "unproven", "vanilla_status": "not_checked"}
                save(args.output / "results" / f"{key}.json", result)
            results[key] = result
            print(f'{len(results)}/{len(aliases)} {aliases[key][0]["path"]}: {result["status"]}', flush=True)
    status_counts = Counter(r["status"] for r in results.values())
    save(args.output / "index.json", {"schema_version": 1, "run_config": config, "candidate_paths": len(candidates),
         "distinct_sha256": len(aliases), "status_counts": dict(sorted(status_counts.items())),
         "functional_status": "unproven", "vanilla_status": "not_checked",
         "results": {key: {"status": result["status"], "result": f"results/{key}.json",
                           "aliases": [f'{r["source"]}/{r["path"]}' for r in aliases[key]]}
                     for key, result in sorted(results.items())}})
    print(json.dumps(dict(sorted(status_counts.items())), indent=2))


if __name__ == "__main__":
    main()
