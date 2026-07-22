"""Generate one enlarged approval frame for curve-local 3x7 surface motion.

This intentionally renders only phase zero. The eventual animation will vary
`geometry_phase` and `surface_phase` while rebuilding a new schematic per frame.
"""

from __future__ import annotations

import json
import os
from math import cos, pi, sin
from pathlib import Path

import numpy as np
from nucleation import Brush, BuildingTool, Curve3D, RenderConfig, Renderer, Schematic, Shape

STEPS = 80
P = 3
Q = 7
MAJOR_RADIUS = 38.0
MINOR_RADIUS = 17.0
INNER_RADIUS = 4.0
OUTER_RADIUS = 6.0
WIDTH = 1024
HEIGHT = 640
U_CELLS = 40
V_CELLS = 4
TWIST_TURNS = 3
CRACK_THRESHOLD = 0.12
SEED = 17

LAVA_BLOCKS = [
    "minecraft:magma_block",
    "minecraft:red_stained_glass",
    "minecraft:orange_stained_glass",
    "minecraft:orange_stained_glass",
    "minecraft:shroomlight",
]
PLATE_BLOCKS = [
    "minecraft:black_concrete",
    "minecraft:coal_block",
    "minecraft:blackstone",
    "minecraft:polished_blackstone",
    "minecraft:deepslate",
    "minecraft:polished_deepslate",
    "minecraft:gray_concrete",
]


def knot_points(geometry_phase: float = 0.0) -> np.ndarray:
    points = []
    for index in range(STEPS):
        angle = 2.0 * pi * index / STEPS
        corrugation = Q * angle + geometry_phase
        ring = MAJOR_RADIUS + MINOR_RADIUS * cos(corrugation)
        points.append(
            (
                ring * cos(P * angle),
                ring * sin(P * angle),
                MINOR_RADIUS * sin(corrugation),
            )
        )
    return np.asarray(points, dtype=np.float64)


def curve_from(points: np.ndarray) -> Curve3D:
    return Curve3D.from_points(points.reshape(-1).tolist(), True)


def solid_positions(shape: Shape) -> np.ndarray:
    temporary = Schematic.create("position-source")
    BuildingTool.fill(temporary, shape, Brush.solid("minecraft:stone"))
    blocks = json.loads(temporary.get_all_blocks_json())
    return np.asarray(
        [(block["x"], block["y"], block["z"]) for block in blocks if block["name"] != "minecraft:air"],
        dtype=np.int32,
    )


def curve_coordinates(positions: np.ndarray, centers: np.ndarray) -> tuple[np.ndarray, np.ndarray, np.ndarray]:
    previous = np.roll(centers, 1, axis=0)
    following = np.roll(centers, -1, axis=0)
    tangents = following - previous
    tangents /= np.linalg.norm(tangents, axis=1, keepdims=True)
    spacing = 0.5 * (
        np.linalg.norm(centers - previous, axis=1) + np.linalg.norm(following - centers, axis=1)
    )

    reference = np.tile(np.asarray([0.0, 0.0, 1.0]), (STEPS, 1))
    parallel = np.abs(np.sum(reference * tangents, axis=1)) > 0.90
    reference[parallel] = np.asarray([0.0, 1.0, 0.0])
    normals = np.cross(tangents, reference)
    normals /= np.linalg.norm(normals, axis=1, keepdims=True)
    binormals = np.cross(tangents, normals)

    count = len(positions)
    nearest = np.empty(count, dtype=np.int32)
    for start in range(0, count, 4_000):
        stop = min(count, start + 4_000)
        delta = positions[start:stop, None, :].astype(np.float64) - centers[None, :, :]
        nearest[start:stop] = np.argmin(np.einsum("ijk,ijk->ij", delta, delta), axis=1)

    relative = positions.astype(np.float64) - centers[nearest]
    along = np.einsum("ij,ij->i", relative, tangents[nearest]) / spacing[nearest]
    u = ((nearest.astype(np.float64) + along) / STEPS * U_CELLS) % U_CELLS
    nx = np.einsum("ij,ij->i", relative, normals[nearest])
    bx = np.einsum("ij,ij->i", relative, binormals[nearest])
    radial = np.hypot(nx, bx)
    theta = np.arctan2(bx, nx)
    v = ((theta + pi) / (2.0 * pi) * V_CELLS) % V_CELLS
    # Integer twist turns preserve the longitudinal seam exactly.
    v = (v + TWIST_TURNS * V_CELLS * u / U_CELLS) % V_CELLS
    return u, v, radial


def hash_random(i: np.ndarray, j: np.ndarray, salt: int) -> np.ndarray:
    ii = i.astype(np.uint64)
    jj = j.astype(np.uint64)
    value = (
        ii * np.uint64(0x9E3779B185EBCA87)
        ^ jj * np.uint64(0xC2B2AE3D27D4EB4F)
        ^ np.uint64((SEED + salt) * 0x165667B19E3779F9 & ((1 << 64) - 1))
    )
    value ^= value >> np.uint64(30)
    value *= np.uint64(0xBF58476D1CE4E5B9)
    value ^= value >> np.uint64(27)
    value *= np.uint64(0x94D049BB133111EB)
    value ^= value >> np.uint64(31)
    return (value >> np.uint64(11)).astype(np.float64) / float(1 << 53)


def periodic_worley(u: np.ndarray, v: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    base_u = np.floor(u).astype(np.int32)
    base_v = np.floor(v).astype(np.int32)
    first = np.full(len(u), np.inf)
    second = np.full(len(u), np.inf)
    for du in (-1, 0, 1):
        for dv in (-1, 0, 1):
            wrapped_u = (base_u + du) % U_CELLS
            wrapped_v = (base_v + dv) % V_CELLS
            seed_u = base_u + du + 0.12 + 0.76 * hash_random(wrapped_u, wrapped_v, 0)
            seed_v = base_v + dv + 0.12 + 0.76 * hash_random(wrapped_u, wrapped_v, 1)
            distance = np.hypot(u - seed_u, v - seed_v)
            replace_first = distance < first
            second = np.where(replace_first, first, np.minimum(second, distance))
            first = np.where(replace_first, distance, first)
    return first, second


def assign_batches(schematic: Schematic, positions: np.ndarray, indices: np.ndarray, blocks: list[str]) -> None:
    for block_index, block in enumerate(blocks):
        selected = positions[indices == block_index]
        if len(selected):
            schematic.set_blocks(selected.reshape(-1).tolist(), block)


def build_frame() -> tuple[Schematic, dict[str, int]]:
    centers = knot_points(geometry_phase=0.0)
    curve = curve_from(centers)
    inner_positions = solid_positions(Shape.tube_along(curve, INNER_RADIUS))
    outer_positions = solid_positions(Shape.tube_along(curve, OUTER_RADIUS))

    inner_u, inner_v, _ = curve_coordinates(inner_positions, centers)
    outer_u, outer_v, outer_radial = curve_coordinates(outer_positions, centers)
    inner_f1, inner_f2 = periodic_worley(inner_u, inner_v)
    outer_f1, outer_f2 = periodic_worley(outer_u, outer_v)

    shell = outer_radial > INNER_RADIUS - 0.35
    plate_mask = shell & ((outer_f2 - outer_f1) >= CRACK_THRESHOLD)
    plate_positions = outer_positions[plate_mask]
    plate_f1 = outer_f1[plate_mask]

    # Smooth deterministic material bands preserve the scorched palette without
    # spatial dithering, so later motion will read as translation rather than shimmer.
    lava_value = np.clip((inner_f2 - inner_f1) / 0.75, 0.0, 0.999)
    lava_indices = np.minimum((lava_value * len(LAVA_BLOCKS)).astype(np.int32), len(LAVA_BLOCKS) - 1)
    plate_value = np.clip(plate_f1 / 0.82, 0.0, 0.999)
    plate_indices = np.minimum((plate_value * len(PLATE_BLOCKS)).astype(np.int32), len(PLATE_BLOCKS) - 1)

    schematic = Schematic.create("scorched-3x7-intrinsic-approval-frame")
    assign_batches(schematic, inner_positions, lava_indices, LAVA_BLOCKS)
    assign_batches(schematic, plate_positions, plate_indices, PLATE_BLOCKS)
    return schematic, {
        "inner_blocks": len(inner_positions),
        "plate_blocks": len(plate_positions),
        "total_blocks": len(inner_positions) + len(plate_positions),
    }


def config() -> RenderConfig:
    result = RenderConfig.create(WIDTH, HEIGHT)
    result.set_isometric()
    result.set_yaw(0.0)
    result.set_pitch(14.0)
    result.set_zoom(1.50)
    result.set_sphere_fit(True)
    result.set_background(0.0, 0.0, 0.0, 0.0)
    return result


def main() -> None:
    root = Path(__file__).resolve().parents[3]
    output = root / "render_work/scorched-3x7-intrinsic"
    output.mkdir(parents=True, exist_ok=True)
    pack_path = Path(
        os.environ.get("NUCLEATION_PACK", "/Users/harrison/code/schemati/renderer/harness/pack.zip")
    )
    schematic, metrics = build_frame()
    schematic_path = output / "approval-frame.schem"
    image_path = output / "approval-frame.png"
    schematic.save_to_file(str(schematic_path))
    Renderer.render_to_file(schematic, pack_path.read_bytes(), config(), str(image_path))
    (output / "approval-frame.json").write_text(json.dumps(metrics, indent=2) + "\n")
    print(json.dumps(metrics))
    print(image_path)
    print(schematic_path)


if __name__ == "__main__":
    main()
