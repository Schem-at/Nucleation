# Basics

A `Schematic` is Nucleation's editable model: blocks, entities, block entities,
metadata, and one or more regions. Start empty or open an existing build, edit it
with ordinary Minecraft block-state strings, inspect the result, then save it in
the format you need.

## Build a beacon

```python
from nucleation import Schematic

beacon = Schematic.create("beacon")
for x in range(-1, 2):
    for z in range(-1, 2):
        beacon.set_block(x, 0, z, "minecraft:gold_block")
beacon.set_block(0, 1, 0, "minecraft:beacon")
beacon.save_to_file("beacon.schem")
```

<div align="center">
<img src="../media/readme/basics/beacon.gif" width="480" alt="A three-by-three gold-block beacon assembling at the origin of a five-by-five Cartesian grid">
</div>

[Download the beacon](../downloads/readme/basics/beacon.schem)

## Build a crafting nook

```python
from nucleation import Schematic

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

copy = Schematic.load_from_file("crafting-nook.schem")
block = copy.get_block(1, 1, 1)
print(block.name())  # minecraft:crafting_table
```

<div align="center">
<img src="../media/readme/basics/animation.gif" width="480" alt="A compact crafting nook assembling with two centered windows, a crafting table, chest, and two wall torches">
</div>

[Download the crafting nook](../downloads/readme/basics/crafting-nook.schem)

## Coordinates and automatic growth

Coordinates are signed integers in Minecraft order: **X, Y, Z**. Positive Y is
up. A newly created schematic does not need dimensions up front: placing a block
automatically grows the default region to contain it, including negative
coordinates.

```python
build = Schematic.create("negative_coordinates")
build.set_block(-8, 64, 12, "minecraft:stone")
build.set_block(24, 80, -3, "minecraft:glass")
```

Setting the same coordinate again replaces the previous state. Set
`minecraft:air` to remove a block.

## Block-state strings

Use the same namespaced block-state syntax used by commands and structure files:

```text
minecraft:stone
minecraft:oak_log[axis=x]
minecraft:oak_stairs[facing=east,half=bottom,shape=straight]
minecraft:water[level=0]
```

Properties are part of the state, so orientation and variants survive format
round-trips. Use `BlockState` directly when you need to construct or inspect
properties programmatically.

## Inspect blocks

`get_block(x, y, z)` returns the complete `BlockState`:

```python
build = Schematic.create("inspect")
build.set_block(1, 1, 1, "minecraft:oak_log[axis=x]")
state = build.get_block(1, 1, 1)
print(state.name())
print(build.get_block_string(1, 1, 1))
```

For lighter-weight queries, use `get_block_name` or `get_block_string`. A lookup
outside every region returns a `NotFound` error.

## Open and save

Nucleation infers formats from file extensions:

```python
from nucleation import Schematic

build = Schematic.load_from_file("castle.litematic")
build.set_block(0, 64, 0, "minecraft:gold_block")
build.save_to_file("castle-edited.schem")
```

The same model can also load from and save to bytes for servers, databases, and
object storage. See [Formats and I/O](formats-and-io.md) for format detection,
conversion, byte APIs, and round-trip guarantees.

## Next

- [Formats and I/O](formats-and-io.md)
- [Shapes, brushes, and masked fills](shapes-and-brushes.md)
- [Animating a build](animation.md)
