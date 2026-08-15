"""Executable source for every Python snippet in docs/features/basics.md."""

from pathlib import Path

from nucleation import Schematic


# --8<-- [start:beacon]
from nucleation import Schematic

beacon = Schematic.create("beacon")
for x in range(-1, 2):
    for z in range(-1, 2):
        beacon.set_block(x, 0, z, "minecraft:gold_block")
beacon.set_block(0, 1, 0, "minecraft:beacon")
beacon.save_to_file("beacon.schem")
# --8<-- [end:beacon]

assert beacon.block_count() == 10
assert beacon.tight_dimensions().x == 3


# --8<-- [start:crafting-nook]
nook = Schematic.create("crafting_nook")
for x in range(5):
    for z in range(5):
        nook.set_block(x, 0, z, "minecraft:spruce_planks")

def wall_block(i, y, end_posts):
    if i == 2 and y == 2:
        return "minecraft:light_blue_stained_glass"
    if i in end_posts:
        return "minecraft:stripped_spruce_log[axis=y]"
    return "minecraft:oak_planks"

for y in (1, 2, 3):
    for x in range(5):
        nook.set_block(x, y, 0, wall_block(x, y, (0, 4)))
    for z in range(1, 5):
        nook.set_block(0, y, z, wall_block(z, y, (4,)))

nook.set_block(1, 1, 1, "minecraft:crafting_table")
nook.set_block(3, 1, 1, "minecraft:chest[facing=south]")
nook.set_block(4, 2, 1, "minecraft:wall_torch[facing=south]")
nook.set_block(1, 2, 4, "minecraft:wall_torch[facing=east]")
nook.save_to_file("crafting-nook.schem")
# --8<-- [end:crafting-nook]

assert nook.block_count() == 56


# --8<-- [start:coordinates]
build = Schematic.create("signed_coordinates")
build.set_block(-8, 64, 12, "minecraft:stone")
build.set_block(24, 80, -3, "minecraft:glass")

minimum = build.tight_bounds_min()
maximum = build.tight_bounds_max()
size = build.tight_dimensions()
print((minimum.x, minimum.y, minimum.z))  # (-8, 64, -3)
print((maximum.x, maximum.y, maximum.z))  # (24, 80, 12)
print((size.x, size.y, size.z))           # (33, 17, 16)
# --8<-- [end:coordinates]

assert (minimum.x, minimum.y, minimum.z) == (-8, 64, -3)
assert (maximum.x, maximum.y, maximum.z) == (24, 80, 12)
assert (size.x, size.y, size.z) == (33, 17, 16)


# --8<-- [start:block-states]
build = Schematic.create("inspect")
build.set_block(1, 1, 1, "minecraft:oak_log[axis=x]")
state = build.get_block(1, 1, 1)
print(state.name())                         # minecraft:oak_log
print(build.get_block_string(1, 1, 1))      # minecraft:oak_log[axis=x]

build.set_block(1, 1, 1, "minecraft:air")  # remove it
# --8<-- [end:block-states]

assert build.block_count() == 0


# --8<-- [start:contents]
contents = Schematic.create("contents")
contents.set_block(0, 0, 0, "minecraft:barrel{signal=13,item=diamond}")
contents.set_block(1, 0, 0, "minecraft:chest{items=[diamond*64,emerald*12]}")
contents.set_block(2, 0, 0, "minecraft:jukebox{record=pigstep}")
contents.set_block(3, 0, 0, "minecraft:jukebox{signal=13}")
# --8<-- [end:contents]

assert contents.block_count() == 4


# --8<-- [start:simulation]
circuit = Schematic.create("placed_by_engine")
circuit.set_block(4, 0, 0, "minecraft:redstone_block")
circuit.set_block(5, 0, 0, "minecraft:redstone_wire{simulate=true}")
print(circuit.get_block_string(5, 0, 0))
# minecraft:redstone_wire[east=side,north=none,power=15,south=none,west=side]
# --8<-- [end:simulation]

assert circuit.get_block_string(5, 0, 0) == (
    "minecraft:redstone_wire[east=side,north=none,power=15,south=none,west=side]"
)


# --8<-- [start:io]
copy = Schematic.load_from_file("beacon.schem")
copy.set_block(0, 2, 0, "minecraft:glass")
copy.save_to_file("beacon-edited.litematic")
# --8<-- [end:io]

assert Path("beacon.schem").stat().st_size > 0
assert Path("beacon-edited.litematic").stat().st_size > 0
assert copy.block_count() == 11
print("Basics Python examples: OK")
