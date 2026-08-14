#!/usr/bin/env python3
"""Benchmark realistic Nucleation workflows through the public Python API.

This complements ``bench_python.py`` rather than comparing against
mcschematic: named regions, content shorthands, simulated placement, and the
tick engine have no equivalent API there.

Run from an environment containing a freshly built Nucleation wheel::

    python benches/bench_realistic_python.py
    python benches/bench_realistic_python.py --filter redstone --json results.json
"""

from __future__ import annotations

import argparse
import gc
import importlib.metadata
import json
import platform
import statistics
import time
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Callable

import nucleation


ResultValue = int | tuple[int, int, str]
Workload = Callable[[], ResultValue]
Validator = Callable[[ResultValue], bool]


@dataclass(frozen=True)
class Timing:
    median_seconds: float
    min_seconds: float
    max_seconds: float


@dataclass(frozen=True)
class Scenario:
    name: str
    size: int
    unit: str
    workload: Workload
    valid: Validator


@dataclass(frozen=True)
class Result:
    workload: str
    size: int
    unit: str
    timing: Timing
    operations_per_second: float


def sparse_positions(count: int) -> tuple[tuple[int, int, int], ...]:
    """Return a deterministic, unique permutation inside a 64-cubed volume."""
    if not 0 <= count <= 64**3:
        raise ValueError("count must fit in a 64-cubed volume")
    positions = []
    for i in range(count):
        shuffled = (i * 73 + 19) & (64**3 - 1)
        positions.append(
            (shuffled & 63, (shuffled >> 12) & 63, (shuffled >> 6) & 63)
        )
    return tuple(positions)


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


def sparse_mixed_workload(count: int) -> Workload:
    positions = sparse_positions(count)
    palette = (
        "minecraft:stone",
        "minecraft:oak_planks",
        "minecraft:redstone_wire[power=0,north=none,east=none,south=none,west=none]",
        "minecraft:repeater[delay=2,facing=east,locked=false,powered=false]",
        "minecraft:comparator[facing=north,mode=compare,powered=false]",
    )

    def workload() -> int:
        schematic = nucleation.Schematic.create("sparse-mixed")
        for i, (x, y, z) in enumerate(positions):
            if not schematic.set_block(x, y, z, palette[i % len(palette)]):
                raise RuntimeError("sparse block placement failed")
        return schematic.block_count()

    return workload


def named_regions_workload(count: int, region_count: int) -> Workload:
    names = tuple(f"build_{i:02}" for i in range(region_count))
    palette = (
        "minecraft:stone",
        "minecraft:oak_planks",
        "minecraft:redstone_wire[power=0,north=none,east=none,south=none,west=none]",
        "minecraft:repeater[delay=2,facing=east,locked=false,powered=false]",
        "minecraft:comparator[facing=north,mode=compare,powered=false]",
    )
    positions = []
    for i in range(count):
        local_i = i // region_count
        shuffled = (local_i * 73 + 19) & (16**3 - 1)
        positions.append(
            (shuffled & 15, (shuffled >> 8) & 15, (shuffled >> 4) & 15)
        )

    def workload() -> int:
        schematic = nucleation.Schematic.create("named-regions")
        for name in names:
            schematic.create_region(name)
        for i, (x, y, z) in enumerate(positions):
            if not schematic.set_block_in_region(
                names[i % region_count], x, y, z, palette[i % len(palette)]
            ):
                raise RuntimeError("named-region block placement failed")
        return schematic.block_count()

    return workload


def barrel_signal_workload(count: int) -> Workload:
    positions = sparse_positions(count)
    descriptors = tuple(
        f"minecraft:barrel[facing=up]{{signal={signal}}}"
        for signal in range(1, 16)
    )

    def workload() -> int:
        schematic = nucleation.Schematic.create("barrel-signals")
        for i, (x, y, z) in enumerate(positions):
            if not schematic.set_block(x, y, z, descriptors[i % len(descriptors)]):
                raise RuntimeError("barrel signal placement failed")
        return schematic.block_count()

    return workload


def jukebox_record_workload(count: int) -> Workload:
    positions = sparse_positions(count)
    descriptors = (
        "minecraft:jukebox{record=pigstep}",
        "minecraft:jukebox{record=cat}",
        "minecraft:jukebox{record=blocks}",
        "minecraft:jukebox{record=chirp}",
    )

    def workload() -> int:
        schematic = nucleation.Schematic.create("jukebox-records")
        for i, (x, y, z) in enumerate(positions):
            if not schematic.set_block(x, y, z, descriptors[i % len(descriptors)]):
                raise RuntimeError("jukebox record placement failed")
        return schematic.block_count()

    return workload


def replacement_workload(count: int) -> Workload:
    positions = sparse_positions(count)
    descriptors = tuple(
        f"minecraft:barrel{{signal={signal}}}" for signal in range(1, 16)
    )

    def workload() -> int:
        schematic = nucleation.Schematic.create("replacement-session")
        for i, (x, y, z) in enumerate(positions):
            schematic.set_block(x, y, z, descriptors[i % len(descriptors)])
        for x, y, z in positions:
            schematic.set_block(x, y, z, "minecraft:stone")
        return schematic.block_count()

    return workload


def simulated_wire_workload(length: int) -> Workload:
    def workload() -> int:
        schematic = nucleation.Schematic.create("simulated-wire")
        for x in range(length + 3):
            schematic.set_block(x, 0, 0, "minecraft:smooth_stone")
        schematic.set_block(0, 1, 0, "minecraft:redstone_block")
        for x in range(1, length + 1):
            if not schematic.set_block(
                x, 1, 0, "minecraft:redstone_wire{simulate=true}"
            ):
                raise RuntimeError("simulated redstone placement failed")
        return schematic.block_count()

    return workload


def simulated_wire_batch_workload(length: int) -> Workload:
    positions = [coordinate for x in range(1, length + 1) for coordinate in (x, 1, 0)]

    def workload() -> int:
        schematic = nucleation.Schematic.create("simulated-wire-batch")
        for x in range(length + 3):
            schematic.set_block(x, 0, 0, "minecraft:smooth_stone")
        schematic.set_block(0, 1, 0, "minecraft:redstone_block")
        schematic.set_blocks_simulated(positions, "minecraft:redstone_wire")
        return schematic.block_count()

    return workload


def piston_scene() -> object:
    scene = nucleation.Schematic.create("piston-demo")
    for x in range(6):
        scene.set_block(x, 0, 0, "minecraft:smooth_stone")
    scene.set_block(
        0,
        1,
        0,
        "minecraft:oak_button[face=floor,facing=east,powered=false]",
    )
    wire = "minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]"
    scene.set_block(1, 1, 0, wire)
    scene.set_block(2, 1, 0, wire)
    scene.set_block(
        3, 1, 0, "minecraft:sticky_piston[facing=east,extended=false]"
    )
    scene.set_block(4, 1, 0, "minecraft:stone")
    return scene


def tick_load_workload(scene: object) -> Workload:
    def workload() -> int:
        simulation = nucleation.TickSimulation.from_schematic(
            scene, nucleation.TickSettleMode.Placement, 0, 0, 0, ""
        )
        return simulation.non_air_count()

    return workload


def tick_run_workload(scene: object) -> Workload:
    def workload() -> tuple[int, int, str]:
        simulation = nucleation.TickSimulation.from_schematic(
            scene, nucleation.TickSettleMode.Placement, 0, 0, 0, ""
        )
        simulation.use_block(0, 1, 0)
        simulation.run_until_quiescent(200)
        return (
            simulation.changes_count(),
            simulation.tick_count(),
            simulation.get_block(4, 1, 0),
        )

    return workload


def scenarios() -> list[Scenario]:
    scene = piston_scene()
    return [
        Scenario(
            "sparse_mixed_default_region",
            10_000,
            "placements",
            sparse_mixed_workload(10_000),
            lambda result: result == 10_000,
        ),
        Scenario(
            "sparse_32_named_regions",
            10_000,
            "placements",
            named_regions_workload(10_000, 32),
            lambda result: result == 10_000,
        ),
        Scenario(
            "barrel_signal",
            5_000,
            "placements",
            barrel_signal_workload(5_000),
            lambda result: result == 5_000,
        ),
        Scenario(
            "jukebox_record",
            5_000,
            "placements",
            jukebox_record_workload(5_000),
            lambda result: result == 5_000,
        ),
        Scenario(
            "barrel_then_plain_replacement",
            10_000,
            "writes",
            replacement_workload(5_000),
            lambda result: result == 5_000,
        ),
        Scenario(
            "redstone_simulated_wire_repeated_1",
            1,
            "simulated placement",
            simulated_wire_workload(1),
            lambda result: result == 6,
        ),
        Scenario(
            "redstone_simulated_wire_repeated_9",
            9,
            "simulated placements",
            simulated_wire_workload(9),
            lambda result: result == 22,
        ),
        Scenario(
            "redstone_simulated_wire_batch_1",
            1,
            "simulated placement",
            simulated_wire_batch_workload(1),
            lambda result: result == 6,
        ),
        Scenario(
            "redstone_simulated_wire_batch_9",
            9,
            "simulated placements",
            simulated_wire_batch_workload(9),
            lambda result: result == 22,
        ),
        Scenario(
            "tick_load_piston_scene",
            11,
            "blocks loaded",
            tick_load_workload(scene),
            lambda result: isinstance(result, int) and result >= 10,
        ),
        Scenario(
            "tick_load_press_and_settle_piston",
            1,
            "simulation run",
            tick_run_workload(scene),
            lambda result: (
                isinstance(result, tuple)
                and result[0] >= 20
                and 1 <= result[1] <= 200
                and result[2] == "minecraft:stone"
            ),
        ),
    ]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--warmups", type=int, default=2)
    parser.add_argument("--iterations", type=int, default=7)
    parser.add_argument(
        "--filter",
        default="",
        help="run only workloads whose names contain this text",
    )
    parser.add_argument("--json", type=Path, help="write machine-readable results")
    args = parser.parse_args()
    if args.warmups < 0 or args.iterations < 1:
        parser.error("--warmups must be >= 0 and --iterations must be >= 1")
    return args


def main() -> None:
    args = parse_args()
    selected = [case for case in scenarios() if args.filter in case.name]
    if not selected:
        raise SystemExit(f"no workload matched --filter={args.filter!r}")

    environment = {
        "python": platform.python_version(),
        "platform": platform.platform(),
        "nucleation": importlib.metadata.version("nucleation"),
        "warmups": args.warmups,
        "iterations": args.iterations,
        "filter": args.filter,
    }
    print("Nucleation realistic Python benchmark")
    print(
        f"Python {environment['python']} | nucleation {environment['nucleation']}"
    )
    print(f"{args.warmups} warmups, {args.iterations} measured iterations")

    results: list[Result] = []
    for case in selected:
        smoke = case.workload()
        if not case.valid(smoke):
            raise AssertionError(
                f"{case.name} did not perform the expected work: {smoke!r}"
            )
        timing = measure(
            case.workload, warmups=args.warmups, iterations=args.iterations
        )
        rate = case.size / timing.median_seconds
        result = Result(case.name, case.size, case.unit, timing, rate)
        results.append(result)
        print(f"\n{case.name} ({case.size:,} {case.unit})")
        print(
            f"  median: {format_duration(timing.median_seconds):>12} "
            f"[{format_duration(timing.min_seconds)} .. "
            f"{format_duration(timing.max_seconds)}]"
        )
        print(f"  throughput: {rate:,.0f} {case.unit}/s")

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
