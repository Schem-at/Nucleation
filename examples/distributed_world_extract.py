#!/usr/bin/env python3
"""Resumable Python control plane for Nucleation's compiled world extractor.

Python schedules deterministic grid shards; the Rust `segment_world` binary
does all Anvil decoding, segmentation, metadata embedding, serialization, and
Store I/O. The storage URL can point at any backend compiled into the worker.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import subprocess
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Shard:
    gx0: int
    gz0: int
    gx1: int
    gz1: int

    @property
    def name(self) -> str:
        return f"gx{self.gx0}_{self.gx1}-gz{self.gz0}_{self.gz1}"


def inclusive_chunks(lo: int, hi: int, size: int):
    start = lo
    while start <= hi:
        end = min(start + size - 1, hi)
        yield start, end
        start = end + 1


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path, help="compiled segment_world executable")
    parser.add_argument("world", help="local world directory/archive or input Store URL")
    parser.add_argument(
        "--world-prefix",
        help="Store key of the Anvil region directory when world is a Store URL",
    )
    parser.add_argument("store", help="output Store URL or local directory")
    parser.add_argument("state", type=Path, help="local resumability/log directory")
    parser.add_argument("--grid-bounds", required=True, help="gx0,gz0,gx1,gz1")
    parser.add_argument(
        "--work-bounds",
        help="optional gx0,gz0,gx1,gz1 subset assigned to this worker",
    )
    parser.add_argument("--pitch", type=int, required=True)
    parser.add_argument("--size", type=int, required=True)
    parser.add_argument("--offset-x", type=int, required=True)
    parser.add_argument("--offset-z", type=int, required=True)
    parser.add_argument("--shard-size", type=int, default=16)
    parser.add_argument("--workers", type=int, default=1)
    parser.add_argument(
        "--component-join-gap",
        type=int,
        default=3,
        help="empty-block gap used to reunite nearby disconnected parts",
    )
    parser.add_argument(
        "--component-min-blocks",
        type=int,
        default=16,
        help="connected components at least this large always remain independent",
    )
    parser.add_argument("--substrate", required=True)
    parser.add_argument("--substrate-band", required=True)
    parser.add_argument("--source-id", required=True)
    parser.add_argument("--world-name", required=True)
    parser.add_argument("--map-name", required=True)
    parser.add_argument("--dimension", default="minecraft:overworld")
    parser.add_argument("--snapshot-id", required=True)
    parser.add_argument("--extracted-at", type=int, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    bounds = tuple(int(value) for value in args.grid_bounds.split(","))
    if len(bounds) != 4:
        raise SystemExit("--grid-bounds must be gx0,gz0,gx1,gz1")
    work_bounds = tuple(int(value) for value in (args.work_bounds or args.grid_bounds).split(","))
    if len(work_bounds) != 4:
        raise SystemExit("--work-bounds must be gx0,gz0,gx1,gz1")
    gx0, gz0, gx1, gz1 = work_bounds
    global_gx0, global_gz0, global_gx1, global_gz1 = bounds
    if (
        gx0 > gx1
        or gz0 > gz1
        or args.shard_size < 1
        or args.workers < 1
        or args.component_join_gap < 0
        or args.component_min_blocks < 0
    ):
        raise SystemExit("invalid grid bounds, shard size, or worker count")
    if not (
        global_gx0 <= gx0 <= gx1 <= global_gx1
        and global_gz0 <= gz0 <= gz1 <= global_gz1
    ):
        raise SystemExit("--work-bounds must lie inside --grid-bounds")
    args.state.mkdir(parents=True, exist_ok=True)
    (args.state / "logs").mkdir(exist_ok=True)

    # Completion markers are only valid for the exact same semantic job and
    # worker binary. Refuse an accidental mixed-config resume instead of
    # silently leaving stale shards in the output catalog.
    with args.binary.open("rb") as binary_file:
        binary_sha256 = hashlib.file_digest(binary_file, "sha256").hexdigest()
    job_config = {
        "schema": 1,
        "binary_sha256": binary_sha256,
        "world": args.world,
        "world_prefix": args.world_prefix,
        "store": args.store,
        "grid_bounds": args.grid_bounds,
        "work_bounds": args.work_bounds,
        "pitch": args.pitch,
        "size": args.size,
        "offset_x": args.offset_x,
        "offset_z": args.offset_z,
        "shard_size": args.shard_size,
        "substrate": args.substrate,
        "substrate_band": args.substrate_band,
        "source_id": args.source_id,
        "world_name": args.world_name,
        "map_name": args.map_name,
        "dimension": args.dimension,
        "snapshot_id": args.snapshot_id,
        "extracted_at": args.extracted_at,
        "component_join_gap": args.component_join_gap,
        "component_min_blocks": args.component_min_blocks,
    }
    job_path = args.state / "job.json"
    if job_path.exists():
        previous = json.loads(job_path.read_text())
        if previous != job_config:
            raise SystemExit(
                f"state directory {args.state} belongs to a different job; "
                "use a new state directory"
            )
    else:
        job_path.write_text(json.dumps(job_config, indent=2, sort_keys=True) + "\n")

    shards = [
        Shard(x0, z0, x1, z1)
        for x0, x1 in inclusive_chunks(gx0, gx1, args.shard_size)
        for z0, z1 in inclusive_chunks(gz0, gz1, args.shard_size)
    ]
    # Useful output first: visit shard rectangles from the assigned area's
    # centre outward. The order is deterministic and does not affect identity
    # or resumability; completed shards are still skipped by their names.
    center_x2 = gx0 + gx1
    center_z2 = gz0 + gz1
    shards.sort(
        key=lambda shard: (
            max(
                abs(shard.gx0 + shard.gx1 - center_x2),
                abs(shard.gz0 + shard.gz1 - center_z2),
            ),
            abs(shard.gx0 + shard.gx1 - center_x2)
            + abs(shard.gz0 + shard.gz1 - center_z2),
            shard.name,
        )
    )

    def run(shard: Shard) -> str:
        done = args.state / f"{shard.name}.done.json"
        if done.exists():
            return f"skip {shard.name}"
        min_x = args.offset_x + shard.gx0 * args.pitch
        min_z = args.offset_z + shard.gz0 * args.pitch
        max_x = args.offset_x + shard.gx1 * args.pitch + args.size - 1
        max_z = args.offset_z + shard.gz1 * args.pitch + args.size - 1
        command = [
            str(args.binary), args.world, args.store,
            str(min_x), str(min_z), str(max_x), str(max_z),
            "--source-id", args.source_id,
            "--world-name", args.world_name,
            "--map-name", args.map_name,
            "--dimension", args.dimension,
            "--snapshot-id", args.snapshot_id,
            "--extracted-at", str(args.extracted_at),
            "--substrate", args.substrate,
            "--substrate-band", args.substrate_band,
            "--grid-pitch", str(args.pitch),
            "--grid-size", str(args.size),
            "--grid-offset-x", str(args.offset_x),
            "--grid-offset-z", str(args.offset_z),
            "--grid-index-bounds", args.grid_bounds,
            "--component-join-gap", str(args.component_join_gap),
            "--component-min-blocks", str(args.component_min_blocks),
        ]
        if args.world_prefix:
            command.extend(["--world-prefix", args.world_prefix])
        log_path = args.state / "logs" / f"{shard.name}.log"
        with log_path.open("wb") as log:
            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=True)
        done.write_text(json.dumps({"shard": shard.name, "rect": [min_x, min_z, max_x, max_z]}) + "\n")
        return f"done {shard.name}"

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as pool:
        for result in pool.map(run, shards):
            print(result, flush=True)


if __name__ == "__main__":
    main()
