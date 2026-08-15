"""Executable Python source for docs/features/fast-generation.md."""

import os
from pathlib import Path

from nucleation import Schematic


WIDTH = 48


def light_positions():
    positions = set()
    for p in range(0, WIDTH, 4):
        positions.update(
            {
                (p, 2, 0),
                (p, 2, WIDTH - 1),
                (0, 2, p),
                (WIDTH - 1, 2, p),
                (p, 2, WIDTH // 2),
                (WIDTH // 2, 2, p),
            }
        )
    return sorted(positions)


def towers():
    for gx in range(4, 44, 8):
        for gz in range(4, 44, 8):
            yield gx, gz, 6 + ((gx // 8 + gz // 8) % 5) * 2


# --8<-- [start:build]
from nucleation import Schematic

campus = Schematic.create("bulk_campus")

# A dense rectangular run belongs in the cuboid fast path.
campus.fill_cuboid(
    0, 0, 0,
    WIDTH - 1, 1, WIDTH - 1,
    "minecraft:polished_deepslate",
)

# Sparse coordinates with one descriptor cross the binding once.
lights = [coordinate for pos in light_positions() for coordinate in pos]
assert campus.set_blocks(lights, "minecraft:sea_lantern") == 68

# Resolve the three tower materials once before the mixed-material hot loop.
brick = campus.prepare_block("minecraft:deepslate_bricks")
glass = campus.prepare_block("minecraft:light_blue_stained_glass")
cap = campus.prepare_block("minecraft:oxidized_cut_copper")

for gx, gz, height in towers():
    for y in range(2, height + 2):
        material = cap if y == height + 1 else glass if y % 3 == 0 else brick
        for dx in range(3):
            for dz in range(3):
                campus.place(gx + dx, y, gz + dz, material)
# --8<-- [end:build]


# --8<-- [start:inspect]
size = campus.tight_dimensions()
print(campus.block_count())                 # 6926
print((size.x, size.y, size.z))             # (48, 16, 48)
print(campus.get_block_string(36, 15, 4))   # minecraft:oxidized_cut_copper
# --8<-- [end:inspect]

assert campus.block_count() == 6_926
assert (size.x, size.y, size.z) == (48, 16, 48)
assert campus.get_block_string(36, 15, 4) == "minecraft:oxidized_cut_copper"

output = Path(os.environ.get("FAST_GENERATION_OUT", "bulk-campus.schem"))
output.parent.mkdir(parents=True, exist_ok=True)
campus.save_to_file(str(output))
print(f"Fast generation Python example: OK ({output})")
