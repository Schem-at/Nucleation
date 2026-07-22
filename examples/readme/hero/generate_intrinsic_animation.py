"""Animate the enlarged intrinsic-surface 3x7 knot with one schematic per frame."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
from concurrent.futures import ProcessPoolExecutor, as_completed
from math import cos, pi, sin
from pathlib import Path

import numpy as np
from nucleation import Renderer, Schematic, Shape

import generate_intrinsic_approval_frame as base

FRAME_COUNT = 120
FPS = 20
SURFACE_U_RADIUS = 1.8
SURFACE_V_RADIUS = 0.9
_WORKER_PACK: bytes | None = None


def phase_for(frame_index: int) -> float:
    # Canonical modulo sampling makes the excluded endpoint use the exact same
    # floating inputs as frame zero, rather than approximate sin/cos values at 2π.
    return (frame_index % FRAME_COUNT) / FRAME_COUNT


def surface_offset(phase: float) -> tuple[float, float]:
    angle = 2.0 * pi * phase
    # Starts at zero so frame 0 is exactly the approved still. The closed ellipse
    # advances through longitudinal and circumferential curve coordinates.
    return (
        SURFACE_U_RADIUS * sin(angle),
        SURFACE_V_RADIUS * (1.0 - cos(angle)),
    )


def canonical_digest(
    inner_positions: np.ndarray,
    inner_indices: np.ndarray,
    plate_positions: np.ndarray,
    plate_indices: np.ndarray,
) -> str:
    mapping: dict[tuple[int, int, int], str] = {}
    for position, block_index in zip(inner_positions, inner_indices, strict=True):
        mapping[tuple(int(value) for value in position)] = base.LAVA_BLOCKS[int(block_index)]
    for position, block_index in zip(plate_positions, plate_indices, strict=True):
        mapping[tuple(int(value) for value in position)] = base.PLATE_BLOCKS[int(block_index)]
    digest = hashlib.sha256()
    for (x, y, z), block in sorted(mapping.items()):
        digest.update(f"{x},{y},{z}:{block}\n".encode())
    return digest.hexdigest()


def build_frame(frame_index: int) -> tuple[Schematic, dict[str, object]]:
    phase = phase_for(frame_index)
    geometry_phase = 2.0 * pi * phase
    centers = base.knot_points(geometry_phase=geometry_phase)
    curve = base.curve_from(centers)
    inner_positions = base.solid_positions(Shape.tube_along(curve, base.INNER_RADIUS))
    outer_positions = base.solid_positions(Shape.tube_along(curve, base.OUTER_RADIUS))

    inner_u, inner_v, _ = base.curve_coordinates(inner_positions, centers)
    outer_u, outer_v, outer_radial = base.curve_coordinates(outer_positions, centers)
    offset_u, offset_v = surface_offset(phase)
    inner_u = (inner_u + offset_u) % base.U_CELLS
    inner_v = (inner_v + offset_v) % base.V_CELLS
    outer_u = (outer_u + offset_u) % base.U_CELLS
    outer_v = (outer_v + offset_v) % base.V_CELLS

    inner_f1, inner_f2 = base.periodic_worley(inner_u, inner_v)
    outer_f1, outer_f2 = base.periodic_worley(outer_u, outer_v)
    shell = outer_radial > base.INNER_RADIUS - 0.35
    plate_mask = shell & ((outer_f2 - outer_f1) >= base.CRACK_THRESHOLD)
    plate_positions = outer_positions[plate_mask]
    plate_f1 = outer_f1[plate_mask]

    lava_value = np.clip((inner_f2 - inner_f1) / 0.75, 0.0, 0.999)
    lava_indices = np.minimum(
        (lava_value * len(base.LAVA_BLOCKS)).astype(np.int32), len(base.LAVA_BLOCKS) - 1
    )
    plate_value = np.clip(plate_f1 / 0.82, 0.0, 0.999)
    plate_indices = np.minimum(
        (plate_value * len(base.PLATE_BLOCKS)).astype(np.int32), len(base.PLATE_BLOCKS) - 1
    )

    schematic = Schematic.create(f"scorched-3x7-intrinsic-frame-{frame_index:03d}")
    base.assign_batches(schematic, inner_positions, lava_indices, base.LAVA_BLOCKS)
    base.assign_batches(schematic, plate_positions, plate_indices, base.PLATE_BLOCKS)
    record: dict[str, object] = {
        "frame": frame_index,
        "phase": phase,
        "geometry_phase_radians": geometry_phase,
        "surface_offset": [offset_u, offset_v],
        "inner_blocks": int(len(inner_positions)),
        "plate_blocks": int(len(plate_positions)),
        "canonical_sha256": canonical_digest(
            inner_positions, lava_indices, plate_positions, plate_indices
        ),
    }
    return schematic, record


def init_worker(pack_path: str) -> None:
    global _WORKER_PACK
    _WORKER_PACK = Path(pack_path).read_bytes()


def render_one(frame_index: int, output_text: str, pack_path: str) -> dict[str, object]:
    global _WORKER_PACK
    if _WORKER_PACK is None:
        _WORKER_PACK = Path(pack_path).read_bytes()
    output = Path(output_text)
    schematic, record = build_frame(frame_index)
    schematic.save_to_file(str(output / "schematics" / f"frame-{frame_index:03d}.schem"))
    Renderer.render_to_file(
        schematic,
        _WORKER_PACK,
        base.config(),
        str(output / "frames" / f"frame-{frame_index:03d}.png"),
    )
    return record


def prepare(output: Path) -> None:
    if output.exists():
        shutil.rmtree(output)
    (output / "schematics").mkdir(parents=True)
    (output / "frames").mkdir(parents=True)


def render_indices(indices: list[int], output: Path, workers: int) -> list[dict[str, object]]:
    prepare(output)
    pack_path = os.environ.get(
        "NUCLEATION_PACK", "/Users/harrison/code/schemati/renderer/harness/pack.zip"
    )
    records: list[dict[str, object]] = []
    if workers == 1:
        init_worker(pack_path)
        for frame_index in indices:
            record = render_one(frame_index, str(output), pack_path)
            records.append(record)
            print(json.dumps(record), flush=True)
    else:
        with ProcessPoolExecutor(
            max_workers=workers, initializer=init_worker, initargs=(pack_path,)
        ) as executor:
            futures = {
                executor.submit(render_one, index, str(output), pack_path): index for index in indices
            }
            for future in as_completed(futures):
                record = future.result()
                records.append(record)
                print(json.dumps(record), flush=True)
    records.sort(key=lambda item: int(item["frame"]))
    (output / "manifest.json").write_text(json.dumps(records, indent=2) + "\n")
    return records


def assemble(output: Path) -> None:
    ffmpeg = shutil.which("ffmpeg") or "/opt/homebrew/bin/ffmpeg"
    subprocess.run(
        [
            ffmpeg,
            "-y",
            "-v",
            "error",
            "-framerate",
            str(FPS),
            "-start_number",
            "0",
            "-i",
            str(output / "frames/frame-%03d.png"),
            "-filter_complex",
            "[0:v]split[a][b];[a]palettegen=reserve_transparent=1[p];"
            "[b][p]paletteuse=dither=sierra2_4a:alpha_threshold=128",
            "-loop",
            "0",
            str(output / "scorched-3x7-intrinsic.gif"),
        ],
        check=True,
    )
    shutil.copy2(output / "frames/frame-000.png", output / "scorched-3x7-intrinsic.png")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--prototype", action="store_true")
    parser.add_argument("--workers", type=int, default=3)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[3]
    output_root = root / "render_work/scorched-3x7-intrinsic"
    if args.prototype:
        output = output_root / "motion-prototype"
        records = render_indices([0, 1, 2, 30, 60, 90, 120], output, workers=1)
        if records[0]["canonical_sha256"] != records[-1]["canonical_sha256"]:
            raise RuntimeError("excluded frame 120 does not exactly equal frame 0")
        print("prototype loop proof: frame 0 == frame 120")
    else:
        output = output_root / "animation"
        records = render_indices(list(range(FRAME_COUNT)), output, workers=args.workers)
        if len({record["canonical_sha256"] for record in records}) != FRAME_COUNT:
            raise RuntimeError("included frames are not all unique")
        assemble(output)
        proof = output_root / "loop-proof"
        proof_records = render_indices([0, FRAME_COUNT], proof, workers=1)
        if proof_records[0]["canonical_sha256"] != proof_records[1]["canonical_sha256"]:
            raise RuntimeError("excluded frame 120 does not exactly equal frame 0")
        print(output / "scorched-3x7-intrinsic.gif")


if __name__ == "__main__":
    main()
