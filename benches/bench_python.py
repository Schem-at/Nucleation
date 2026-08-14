#!/usr/bin/env python3
"""Compare the public Python APIs of nucleation and mcschematic.

Run from the repository root after installing both packages::

    python benches/bench_python.py

The benchmark reports medians and a paired speed ratio. It is intentionally a
Python end-to-end benchmark: object creation and Python/extension crossings are
part of the measured work. It is not a substitute for the Rust Criterion
benchmarks when profiling Nucleation internals.
"""

from __future__ import annotations

import argparse
import gc
import importlib.metadata
import json
import platform
import statistics
import tempfile
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Callable

import mcschematic
import nucleation


Workload = Callable[[], object]


@dataclass(frozen=True)
class Timing:
    median_seconds: float
    min_seconds: float
    max_seconds: float


@dataclass(frozen=True)
class Result:
    workload: str
    size: int
    unit: str
    nucleation: Timing
    mcschematic: Timing
    nucleation_speedup: float


def measure(fn: Workload, *, warmups: int, iterations: int) -> Timing:
    for _ in range(warmups):
        fn()

    samples: list[float] = []
    for _ in range(iterations):
        gc.collect()
        start = time.perf_counter_ns()
        fn()
        samples.append((time.perf_counter_ns() - start) / 1_000_000_000)

    return Timing(
        median_seconds=statistics.median(samples),
        min_seconds=min(samples),
        max_seconds=max(samples),
    )


def format_duration(seconds: float) -> str:
    if seconds < 1e-3:
        return f"{seconds * 1e6:.1f} us"
    if seconds < 1:
        return f"{seconds * 1e3:.2f} ms"
    return f"{seconds:.3f} s"


def nucleation_set_blocks(count: int) -> Workload:
    def workload() -> int:
        schematic = nucleation.Schematic.create("benchmark")
        for x in range(count):
            schematic.set_block(x, 0, 0, "minecraft:stone")
        return schematic.block_count()

    return workload


def mcschematic_set_blocks(count: int) -> Workload:
    def workload() -> object:
        schematic = mcschematic.MCSchematic()
        for x in range(count):
            schematic.setBlock((x, 0, 0), "minecraft:stone")
        return schematic

    return workload


def nucleation_fill(edge: int) -> Workload:
    def workload() -> int:
        schematic = nucleation.Schematic.create("benchmark")
        schematic.fill_cuboid(
            0, 0, 0, edge - 1, edge - 1, edge - 1, "minecraft:stone"
        )
        return schematic.block_count()

    return workload


def mcschematic_fill(edge: int) -> Workload:
    def workload() -> object:
        schematic = mcschematic.MCSchematic()
        schematic.getStructure().cuboidFilled(
            "minecraft:stone", (0, 0, 0), (edge - 1, edge - 1, edge - 1)
        )
        return schematic

    return workload


def nucleation_fill_and_export(edge: int) -> Workload:
    def workload() -> int:
        schematic = nucleation.Schematic.create("benchmark")
        schematic.fill_cuboid(
            0, 0, 0, edge - 1, edge - 1, edge - 1, "minecraft:stone"
        )
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "benchmark.schem"
            schematic.save(output, format="schematic")
            return output.stat().st_size

    return workload


def mcschematic_fill_and_export(edge: int) -> Workload:
    def workload() -> int:
        schematic = mcschematic.MCSchematic()
        schematic.getStructure().cuboidFilled(
            "minecraft:stone", (0, 0, 0), (edge - 1, edge - 1, edge - 1)
        )
        with tempfile.TemporaryDirectory() as directory:
            schematic.save(
                directory,
                "benchmark",
                mcschematic.Version.JE_1_21_5,
            )
            return (Path(directory) / "benchmark.schem").stat().st_size

    return workload


def compare(
    workload: str,
    size: int,
    unit: str,
    nucleation_fn: Workload,
    mcschematic_fn: Workload,
    *,
    warmups: int,
    iterations: int,
) -> Result:
    # Smoke each workload before timing it. This catches API drift and prevents
    # a fast exception path from ever being reported as a benchmark result.
    assert nucleation_fn()
    assert mcschematic_fn()

    nucleation_timing = measure(
        nucleation_fn, warmups=warmups, iterations=iterations
    )
    mcschematic_timing = measure(
        mcschematic_fn, warmups=warmups, iterations=iterations
    )
    speedup = (
        mcschematic_timing.median_seconds / nucleation_timing.median_seconds
    )
    result = Result(
        workload=workload,
        size=size,
        unit=unit,
        nucleation=nucleation_timing,
        mcschematic=mcschematic_timing,
        nucleation_speedup=speedup,
    )

    label = f"{workload} ({size:,} {unit})"
    print(f"\n{label}")
    print(
        f"  nucleation:  {format_duration(nucleation_timing.median_seconds):>12} "
        f"[{format_duration(nucleation_timing.min_seconds)} .. "
        f"{format_duration(nucleation_timing.max_seconds)}]"
    )
    print(
        f"  mcschematic: {format_duration(mcschematic_timing.median_seconds):>12} "
        f"[{format_duration(mcschematic_timing.min_seconds)} .. "
        f"{format_duration(mcschematic_timing.max_seconds)}]"
    )
    print(f"  nucleation speedup: {speedup:.2f}x")
    return result


def package_version(name: str) -> str:
    return importlib.metadata.version(name)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--warmups", type=int, default=2)
    parser.add_argument("--iterations", type=int, default=7)
    parser.add_argument(
        "--json",
        type=Path,
        help="also write machine-readable environment and timing results",
    )
    args = parser.parse_args()
    if args.warmups < 0 or args.iterations < 1:
        parser.error("--warmups must be >= 0 and --iterations must be >= 1")
    return args


def main() -> None:
    args = parse_args()
    environment = {
        "python": platform.python_version(),
        "platform": platform.platform(),
        "nucleation": package_version("nucleation"),
        "mcschematic": package_version("mcschematic"),
        "warmups": args.warmups,
        "iterations": args.iterations,
    }
    print("Nucleation vs mcschematic Python benchmark")
    print(
        f"Python {environment['python']} | nucleation {environment['nucleation']} | "
        f"mcschematic {environment['mcschematic']}"
    )
    print(f"{args.warmups} warmups, {args.iterations} measured iterations")
    print("Speedup is mcschematic median / nucleation median; higher favors Nucleation.")

    results: list[Result] = []
    for count in (100, 1_000, 10_000):
        results.append(
            compare(
                "set_blocks",
                count,
                "blocks",
                nucleation_set_blocks(count),
                mcschematic_set_blocks(count),
                warmups=args.warmups,
                iterations=args.iterations,
            )
        )
    for edge in (10, 32, 64):
        results.append(
            compare(
                "fill_cuboid",
                edge,
                "edge length",
                nucleation_fill(edge),
                mcschematic_fill(edge),
                warmups=args.warmups,
                iterations=args.iterations,
            )
        )
    for edge in (10, 32):
        results.append(
            compare(
                "fill_and_export",
                edge,
                "edge length",
                nucleation_fill_and_export(edge),
                mcschematic_fill_and_export(edge),
                warmups=args.warmups,
                iterations=args.iterations,
            )
        )

    if args.json:
        payload = {
            "environment": environment,
            "results": [asdict(result) for result in results],
        }
        args.json.parent.mkdir(parents=True, exist_ok=True)
        args.json.write_text(json.dumps(payload, indent=2) + "\n")
        print(f"\nWrote {args.json}")


if __name__ == "__main__":
    main()
