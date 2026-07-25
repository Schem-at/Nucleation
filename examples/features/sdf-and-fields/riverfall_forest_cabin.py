#!/usr/bin/env python3
"""Build a reusable-Field3 river, waterfall, cabin, and forest escarpment."""

import json
import random
import sys
from collections.abc import Callable
from functools import cache
from pathlib import Path
from nucleation import (
    Brush,
    BuildingTool,
    Field3,
    InterpolationSpace,
    Palette,
    Schematic,
    Sdf,
    SdfCellMode,
)

NAME = "riverfall-forest-cabin-escarpment"


def palette(block_ids: list[str]) -> Palette:
    """Create a compact palette without exposing the binding's JSON transport."""
    return Palette.from_block_ids(json.dumps(block_ids))


def tree_position(
    surface: Callable[[int, int], int], x: int, z: int, max_slope: int = 3
) -> tuple[int, int, int] | None:
    """Return a tree base above sufficiently level terrain."""
    ground_y = surface(x, z)
    slope = max(
        abs(ground_y - surface(x + 2, z)),
        abs(ground_y - surface(x, z + 2)),
    )
    return (x, ground_y + 1, z) if slope <= max_slope else None


def build() -> tuple[Schematic, int, int]:
    schematic = Schematic.create(NAME)

    # Broad lowland. One reusable field drives both its silhouette and rock strata.
    terrain_field = Field3.value_noise_fbm(frequency=0.027, seed=31, octaves=5)
    terrain = Sdf.plane(0, 1, 0, 0).offset_by_field(
        field=terrain_field, amplitude=6.5
    )
    ground = terrain.intersection_with(Sdf.box_shape(68, 12, 50, 6).translate(0, -5, 0))
    ground_brush = Brush.field3(
        field=terrain_field,
        stops=[0.0, 0.48, 1.0],
        colors=[42, 45, 52, 98, 96, 88, 148, 142, 126],
        lo=-1.0,
        hi=1.0,
        space=InterpolationSpace.Oklab,
    )
    ground_brush.set_palette(
        palette(
            [
                "minecraft:deepslate",
                "minecraft:tuff",
                "minecraft:stone",
                "minecraft:andesite",
            ]
        )
    )
    BuildingTool.fill(schematic, ground.to_shape(), ground_brush)

    @cache
    def surface(x, z):
        return next(y for y in range(11, -18, -1) if terrain.eval_at(x, y, z) <= 0)

    # Tall escarpment. Independent reusable fields displace its top and front.
    plateau_field = Field3.value_noise_fbm(frequency=0.024, seed=73, octaves=4)
    cliff_field = Field3.value_noise_fbm(frequency=0.032, seed=91, octaves=4)
    plateau_top = Sdf.plane(0, 1, 0, -35).offset_by_field(
        field=plateau_field, amplitude=5.5
    )
    cliff_front = Sdf.plane(-1, 0, 0, 15).offset_by_field(
        field=cliff_field, amplitude=5.2
    )
    # Keep a steep central face for the waterfall, then attach two diagonal
    # buttresses that descend into the lowland. Only the central mass is clipped
    # by the vertical front plane, so the shoulders can form a natural ramp.
    cliff_profile = plateau_top.smooth_intersection(cliff_front, radius=9.0)
    central_envelope = Sdf.box_shape(29, 27, 34, 9).translate(44, 9, 0)
    central_cliff = central_envelope.intersection_with(cliff_profile)
    north_shoulder = Sdf.round_cone(
        10, 1, -39,
        48, 28, -32,
        22, 15,
    ).intersection_with(plateau_top)
    south_shoulder = Sdf.round_cone(
        10, 1, 39,
        48, 28, 32,
        22, 15,
    ).intersection_with(plateau_top)
    cliff = central_cliff.smooth_union(north_shoulder, radius=7.0).smooth_union(
        south_shoulder, radius=7.0
    )
    cracks = Sdf.cells(
        frequency=0.085,
        seed=19,
        jitter=1.0,
        mode=SdfCellMode.F2MinusF1,
        threshold=0.46,
    ).rotate(18, 27, 9)
    fractured = cliff.subtract(cracks)
    BuildingTool.fill(schematic, cliff.to_shape(), Brush.solid("minecraft:cobbled_deepslate"))
    cliff_brush = Brush.field3(
        field=cliff_field,
        stops=[0.0, 0.42, 0.72, 1.0],
        colors=[50, 52, 57, 105, 101, 91, 155, 149, 132, 205, 198, 177],
        lo=-1.0,
        hi=1.0,
        space=InterpolationSpace.Oklab,
    )
    cliff_brush.set_palette(
        palette(
            [
                "minecraft:cobbled_deepslate",
                "minecraft:tuff",
                "minecraft:andesite",
                "minecraft:stone",
                "minecraft:calcite",
            ]
        )
    )
    BuildingTool.fill(schematic, fractured.to_shape(), cliff_brush)

    @cache
    def plateau_surface(x, z):
        return next((y for y in range(45, 14, -1) if cliff.eval_at(x, y, z) <= 0), None)

    @cache
    def face_x(y, z):
        return next((x for x in range(6, 31) if cliff.eval_at(x, y, z) <= 0), None)

    # Give visible front-face crack channels actual air depth before the dark substrate.
    for y in range(-8, 43):
        for z in range(-49, 50):
            fx = face_x(y, z)
            if fx is None:
                continue
            carved = False
            for depth in range(4):
                x = fx + depth
                if cracks.eval_at(x + 0.5, y + 0.5, z + 0.5) <= 0:
                    schematic.set_block(x, y, z, "minecraft:air")
                    carved = True
            if carved and cliff.eval_at(fx + 4.5, y + 0.5, z + 0.5) <= 0:
                schematic.set_block(fx + 4, y, z, "minecraft:cobbled_deepslate")

    # Restrained terrain caps before feature carving.
    for x in range(-68, 16):
        for z in range(-50, 51):
            y = surface(x, z)
            block = "minecraft:moss_block" if (x + 2 * z) % 13 == 0 else "minecraft:grass_block"
            schematic.set_block(x, y, z, block)
    for x in range(16, 74):
        for z in range(-50, 51):
            y = plateau_surface(x, z)
            if y is not None:
                schematic.set_block(x, y, z, "minecraft:moss_block")

    # Deterministic rasterized polylines.
    def polyline(control):
        out = []
        for (x0, z0), (x1, z1) in zip(control, control[1:]):
            steps = max(abs(x1 - x0), abs(z1 - z0)) * 2
            for i in range(steps + 1):
                t = i / max(1, steps)
                p = (round(x0 + (x1 - x0) * t), round(z0 + (z1 - z0) * t))
                if not out or p != out[-1]:
                    out.append(p)
        return out

    def stamp_water(path, surface_fn, radius=2, start_level=None):
        cells, level = set(), start_level
        for cx, cz in path:
            raw = surface_fn(cx, cz)
            if raw is None:
                continue
            level = raw if level is None else min(level, raw)
            for dx in range(-radius, radius + 1):
                for dz in range(-radius, radius + 1):
                    if dx * dx + dz * dz > radius * radius + 1:
                        continue
                    x, z = cx + dx, cz + dz
                    top = surface_fn(x, z)
                    if top is None:
                        continue
                    for y in range(level + 1, max(level + 2, top + 3)):
                        schematic.set_block(x, y, z, "minecraft:air")
                    schematic.set_block(x, level - 1, z, "minecraft:gravel")
                    schematic.set_block(x, level, z, "minecraft:water")
                    cells.add((x, z))
        return cells, level

    # Plateau river converges on the waterfall lip.
    fall_z = 4
    upper_path = polyline([(69, 20), (58, 17), (47, 10), (35, 12), (25, 6), (18, fall_z)])
    water_cells, fall_top = stamp_water(upper_path, plateau_surface, radius=2)

    # Find the exposed face and form a broad pool below it.
    fall_face = face_x(fall_top, fall_z)
    pool_center = (fall_face - 6, fall_z)
    pool_level = min(surface(pool_center[0], pool_center[1]), surface(pool_center[0] - 3, fall_z))
    for dx in range(-9, 10):
        for dz in range(-7, 8):
            if (dx / 9) ** 2 + (dz / 7) ** 2 > 1:
                continue
            x, z = pool_center[0] + dx, pool_center[1] + dz
            top = surface(x, z)
            for y in range(pool_level + 1, max(pool_level + 2, top + 3)):
                schematic.set_block(x, y, z, "minecraft:air")
            bed = "minecraft:clay" if (x + z) % 3 else "minecraft:gravel"
            schematic.set_block(x, pool_level - 1, z, bed)
            schematic.set_block(x, pool_level, z, "minecraft:water")
            water_cells.add((x, z))

    # Water sheet follows the displaced face and meets the pool.
    for y in range(pool_level + 1, fall_top + 1):
        fx = face_x(y, fall_z)
        if fx is None:
            continue
        for dz in range(-2, 3):
            edge = face_x(y, fall_z + dz)
            if edge is not None:
                schematic.set_block(edge - 1, y, fall_z + dz, "minecraft:water")
                if abs(dz) == 2 and y % 5 == 0:
                    schematic.set_block(edge - 2, y, fall_z + dz, "minecraft:water")

    # Lower outflow meanders away from the pool.
    lower_path = polyline([
        (pool_center[0] - 2, fall_z), (-8, 1), (-22, -7), (-39, -12), (-53, -19), (-61, -22)
    ])
    lower_cells, _ = stamp_water(lower_path, surface, radius=2, start_level=pool_level)
    water_cells |= lower_cells

    # Terrain-following footpath from the scene edge, past the cabin, to the pool.
    path_points = polyline(
        [
            (-66, 34),
            (-52, 30),
            (-39, 18),
            (-26, 12),
            (-12, 8),
            (pool_center[0] - 7, 9),
        ]
    )
    path_cells = set()
    for cx, cz in path_points:
        for dx, dz in ((0, 0), (1, 0), (-1, 0)):
            x, z = cx + dx, cz + dz
            if (x, z) in water_cells:
                continue
            y = surface(x, z)
            schematic.set_block(x, y, z, "minecraft:gravel" if dx == 0 else "minecraft:coarse_dirt")
            schematic.set_block(x, y + 1, z, "minecraft:air")
            schematic.set_block(x, y + 2, z, "minecraft:air")
            path_cells.add((x, z))

    # Compact timber cabin in a deliberately cleared lowland patch.
    cx, cz = -39, 22
    x0, x1, z0, z1 = cx - 4, cx + 4, cz - 3, cz + 3
    base = max(surface(x, z) for x in range(x0, x1 + 1) for z in range(z0, z1 + 1))
    for x in range(x0 - 2, x1 + 3):
        for z in range(z0 - 2, z1 + 3):
            for y in range(base, base + 12):
                schematic.set_block(x, y, z, "minecraft:air")
    for x in range(x0, x1 + 1):
        for z in range(z0, z1 + 1):
            for y in range(surface(x, z), base):
                schematic.set_block(x, y, z, "minecraft:cobblestone")
            schematic.set_block(x, base - 1, z, "minecraft:cobblestone")
            schematic.set_block(x, base, z, "minecraft:spruce_planks")

    # Walls, corner posts, windows, and open doorway facing the path.
    for y in range(base + 1, base + 6):
        for x in range(x0, x1 + 1):
            for z in (z0, z1):
                schematic.set_block(x, y, z, "minecraft:spruce_planks")
        for z in range(z0, z1 + 1):
            for x in (x0, x1):
                schematic.set_block(x, y, z, "minecraft:spruce_planks")
    for x in (x0, x1):
        for z in (z0, z1):
            for y in range(base + 1, base + 7):
                schematic.set_block(x, y, z, "minecraft:stripped_spruce_log")
    for x, z in ((cx - 2, z0), (cx + 2, z0), (cx - 2, z1), (cx + 2, z1)):
        for y in (base + 3, base + 4):
            schematic.set_block(x, y, z, "minecraft:glass_pane")
    for y in range(base + 1, base + 4):
        schematic.set_block(cx, y, z0, "minecraft:air")
    schematic.set_block(cx - 1, base + 3, z0 - 1, "minecraft:lantern")

    # Layered dark roof with strong cabin silhouette.
    for layer in range(4):
        y = base + 6 + layer
        za, zb = z0 - 1 + layer, z1 + 1 - layer
        for x in range(x0 - 1, x1 + 2):
            schematic.set_block(x, y, za, "minecraft:dark_oak_planks")
            schematic.set_block(x, y, zb, "minecraft:dark_oak_planks")
            if layer == 3:
                for z in range(za, zb + 1):
                    schematic.set_block(x, y, z, "minecraft:dark_oak_planks")
    for y in range(base + 5, base + 11):
        schematic.set_block(x1 - 1, y, z1 - 1, "minecraft:cobblestone")
    schematic.set_block(x1 - 1, base + 11, z1 - 1, "minecraft:campfire")

    # Porch and short cabin spur.
    for x in range(cx - 2, cx + 3):
        schematic.set_block(x, base, z0 - 1, "minecraft:spruce_planks")
    for x, z in polyline([(cx, z0 - 2), (-43, 20), (-47, 24)]):
        y = surface(x, z)
        schematic.set_block(x, y, z, "minecraft:gravel")
        path_cells.add((x, z))

    # Forest respects hydrology, trail, waterfall sightline, and cabin clearing.
    def near(cells, x, z, radius):
        r2 = radius * radius
        return any((x - px) ** 2 + (z - pz) ** 2 <= r2 for px, pz in cells)

    def reserved(x, z):
        if x0 - 8 <= x <= x1 + 8 and z0 - 8 <= z <= z1 + 8:
            return True
        if near(water_cells, x, z, 5) or near(path_cells, x, z, 3):
            return True
        return 4 <= x <= 24 and -5 <= z <= 13

    rng, points = random.Random(23), [[], []]
    for gx in range(-60, 14, 13):
        for gz in range(-44, 45, 13):
            x, z = gx + rng.randint(-4, 4), gz + rng.randint(-4, 4)
            if reserved(x, z):
                continue
            position = tree_position(surface, x, z)
            if position is not None:
                points[(gx // 13 + gz // 13) & 1] += position
    for x in (31, 47, 63):
        for z in (-37, -20, 21, 38):
            if near(water_cells, x, z, 6):
                continue
            y = plateau_surface(x, z)
            if y is not None:
                points[1] += [x, y + 1, z]

    trunk = Sdf.capped_cylinder(1.25, 4.8).translate(0, 4.8, 0)
    crown = Sdf.round_cone(0, 5.8, 0, 0, 15.5, 0, 4.2, 0.35)
    for scale, xyz in ((1.0, points[0]), (0.8, points[1])):
        trunks = trunk.scale(scale).repeat_points(xyz)
        crowns = crown.scale(scale).repeat_points(xyz)
        BuildingTool.fill(
            schematic, trunks.to_shape(), Brush.solid("minecraft:spruce_log")
        )
        BuildingTool.fill(
            schematic, crowns.to_shape(), Brush.solid("minecraft:spruce_leaves")
        )

    tree_count = sum(len(group) // 3 for group in points)
    return schematic, tree_count, base


def main() -> None:
    output = Path(sys.argv[1] if len(sys.argv) > 1 else f"{NAME}.litematic")
    output.parent.mkdir(parents=True, exist_ok=True)

    schematic, tree_count, base = build()
    schematic.save(str(output))
    print(
        f"wrote {output}: {schematic.block_count()} blocks, "
        f"{tree_count} trees, cabin base y={base}"
    )


if __name__ == "__main__":
    main()
