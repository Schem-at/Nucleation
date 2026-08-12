#!/usr/bin/env python3
"""Resumable control plane for arbitrary-partition world extraction.

Unlike ``distributed_world_extract.py`` (uniform grids), this scheduler keeps
the semantic partition geometry in a caller-supplied JSON file and schedules a
separate list of pairwise-disjoint, inclusive world rectangles.  Every compute
host receives the same partition file; each host may receive a different rect
file.  This is useful for merged plots, claims, campuses, or any other geometry
whose boundaries are not a regular lattice.

The Rust worker still performs all Anvil parsing, segmentation, provenance and
Store I/O.  Python only schedules bounded invocations and records completion.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class RectTask:
    name: str
    min_x: int
    min_z: int
    max_x: int
    max_z: int


SAFE_NAME = re.compile(r"^[A-Za-z0-9_.-]+$")


def parse_bool(value: str) -> bool:
    normalized = value.strip().lower()
    if normalized in {"1", "true", "yes", "on"}:
        return True
    if normalized in {"0", "false", "no", "off"}:
        return False
    raise argparse.ArgumentTypeError("expected true or false")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("world", help="local world directory/archive or input Store URL")
    parser.add_argument("store", help="output Store URL or local directory")
    parser.add_argument("state", type=Path, help="local resumability/log directory")
    parser.add_argument("--world-prefix", help="Store key of the Anvil region directory")
    parser.add_argument("--partition-hints", type=Path, required=True)
    parser.add_argument("--rects", type=Path, required=True)
    parser.add_argument("--workers", type=int, default=1)
    parser.add_argument("--substrate", required=True)
    parser.add_argument("--substrate-band", required=True)
    parser.add_argument("--partition-floor-share", type=float, default=0.30)
    parser.add_argument("--partition-dense-layer-coverage", type=float, default=0.80)
    parser.add_argument("--split-min-blocks", type=int, default=4096)
    parser.add_argument(
        "--component-attach-mode",
        choices=("exact", "nearby", "nearest"),
        default="nearby",
        help=(
            "final materialized-build split: exact emits every disconnected component; "
            "nearby keeps tiny nearby fixtures with a core; nearest is the legacy "
            "all-fragments-to-core policy"
        ),
    )
    parser.add_argument("--component-join-gap", type=int, default=3)
    parser.add_argument(
        "--component-min-blocks",
        type=int,
        default=16,
        help="nearby/nearest core threshold; ignored by exact mode (default: 16)",
    )
    parser.add_argument("--drop-unpartitioned", type=parse_bool, default=True)
    parser.add_argument("--source-id", required=True)
    parser.add_argument("--world-name", required=True)
    parser.add_argument("--map-name", required=True)
    parser.add_argument("--dimension", default="minecraft:overworld")
    parser.add_argument("--snapshot-id", required=True)
    parser.add_argument("--extracted-at", type=int, required=True)
    return parser.parse_args()


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_tasks(path: Path) -> list[RectTask]:
    raw = json.loads(path.read_text())
    if not isinstance(raw, list) or not raw:
        raise SystemExit("--rects must contain a non-empty JSON array")
    tasks = []
    names = set()
    for row in raw:
        task = RectTask(
            name=str(row["id"]),
            min_x=int(row["min_x"]),
            min_z=int(row["min_z"]),
            max_x=int(row["max_x"]),
            max_z=int(row["max_z"]),
        )
        if not SAFE_NAME.fullmatch(task.name):
            raise SystemExit(f"unsafe rectangle id {task.name!r}")
        if task.name in names:
            raise SystemExit(f"duplicate rectangle id {task.name!r}")
        if task.min_x > task.max_x or task.min_z > task.max_z:
            raise SystemExit(f"inverted rectangle {task.name!r}")
        names.add(task.name)
        tasks.append(task)
    return tasks


def main() -> None:
    args = parse_args()
    if args.workers < 1:
        raise SystemExit("--workers must be positive")
    if args.component_join_gap < 0 or args.component_min_blocks < 0 or args.split_min_blocks < 0:
        raise SystemExit("component thresholds must be non-negative")
    for name, value in (
        ("partition-floor-share", args.partition_floor_share),
        ("partition-dense-layer-coverage", args.partition_dense_layer_coverage),
    ):
        if not 0.0 <= value <= 1.0:
            raise SystemExit(f"--{name} must be in 0..=1")
    if args.world.startswith(("ssh://", "s3://", "redis://", "postgres://")) != bool(
        args.world_prefix
    ):
        raise SystemExit("Store world URLs require --world-prefix; local inputs must omit it")

    tasks = load_tasks(args.rects)
    args.state.mkdir(parents=True, exist_ok=True)
    (args.state / "logs").mkdir(exist_ok=True)

    job_config = {
        "schema": 1,
        "binary_sha256": sha256(args.binary),
        "partition_hints_sha256": sha256(args.partition_hints),
        "rects_sha256": sha256(args.rects),
        "world": args.world,
        "world_prefix": args.world_prefix,
        "store": args.store,
        "substrate": args.substrate,
        "substrate_band": args.substrate_band,
        "partition_floor_share": args.partition_floor_share,
        "partition_dense_layer_coverage": args.partition_dense_layer_coverage,
        "split_min_blocks": args.split_min_blocks,
        "component_attach_mode": args.component_attach_mode,
        "component_join_gap": args.component_join_gap,
        "component_min_blocks": args.component_min_blocks,
        "drop_unpartitioned": args.drop_unpartitioned,
        "source_id": args.source_id,
        "world_name": args.world_name,
        "map_name": args.map_name,
        "dimension": args.dimension,
        "snapshot_id": args.snapshot_id,
        "extracted_at": args.extracted_at,
    }
    job_path = args.state / "job.json"
    if job_path.exists():
        if json.loads(job_path.read_text()) != job_config:
            raise SystemExit(f"state directory {args.state} belongs to a different job")
    else:
        job_path.write_text(json.dumps(job_config, indent=2, sort_keys=True) + "\n")

    # Each host's rect file is already an explicit assignment.  Centre-out
    # ordering merely makes a partial run useful sooner.
    center_x2 = min(t.min_x for t in tasks) + max(t.max_x for t in tasks)
    center_z2 = min(t.min_z for t in tasks) + max(t.max_z for t in tasks)
    tasks.sort(
        key=lambda task: (
            max(abs(task.min_x + task.max_x - center_x2), abs(task.min_z + task.max_z - center_z2)),
            task.name,
        )
    )

    def run(task: RectTask) -> str:
        done = args.state / f"{task.name}.done.json"
        if done.exists():
            return f"skip {task.name}"
        command = [
            str(args.binary),
            args.world,
            args.store,
            str(task.min_x),
            str(task.min_z),
            str(task.max_x),
            str(task.max_z),
            "--partition-hints",
            str(args.partition_hints),
            "--drop-unpartitioned",
            str(args.drop_unpartitioned).lower(),
            "--partition-floor-share",
            str(args.partition_floor_share),
            "--partition-dense-layer-coverage",
            str(args.partition_dense_layer_coverage),
            "--split-min-blocks",
            str(args.split_min_blocks),
            "--component-attach-mode",
            args.component_attach_mode,
            "--component-join-gap",
            str(args.component_join_gap),
            "--component-min-blocks",
            str(args.component_min_blocks),
            "--source-id",
            args.source_id,
            "--world-name",
            args.world_name,
            "--map-name",
            args.map_name,
            "--dimension",
            args.dimension,
            "--snapshot-id",
            args.snapshot_id,
            "--extracted-at",
            str(args.extracted_at),
            "--substrate",
            args.substrate,
            "--substrate-band",
            args.substrate_band,
        ]
        if args.world_prefix:
            command.extend(["--world-prefix", args.world_prefix])
        log_path = args.state / "logs" / f"{task.name}.log"
        with log_path.open("wb") as log:
            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=True)
        done.write_text(
            json.dumps(
                {
                    "task": task.name,
                    "rect": [task.min_x, task.min_z, task.max_x, task.max_z],
                }
            )
            + "\n"
        )
        return f"done {task.name}"

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool:
        for result in pool.map(run, tasks):
            print(result, flush=True)


if __name__ == "__main__":
    main()
