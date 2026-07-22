# Regions, transforms, and stamping

A `Schematic` always has one default region named `Main`. Additional named regions are independent Litematica-style sub-volumes with their own bounds, palettes, blocks, block entities, and entities.

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/regions.png" width="760" alt="Before and after: a quartz keep with copper and prismarine wings as three named regions, with one wing rotated in place">

## Region lifecycle

Create regions explicitly when their lifetime matters:

```python
from nucleation import Schematic

s = Schematic.create("castle")
s.create_region("keep")
s.create_region("gate")
assert s.has_region("keep")

s.rename_region("gate", "east_gate")
s.remove_region("east_gate")
assert not s.has_region("east_gate")
```

Names must be non-empty and unique. `Main` cannot be removed or renamed. Creating a duplicate, removing `Main`, or renaming onto an existing name raises a binding exception.

`set_block_in_region` also creates a missing named region lazily. An explicitly created empty region anchors its bounds at the first placed block, rather than retaining an artificial origin cell.

```python
s.set_block_in_region("west_wing", -20, 4, 8, "minecraft:copper_block")
```

`region_names_json()` returns all live region names in stable order (`Main`, then lexical named-region order). A region remains part of the in-memory schematic until it is removed, and `deep_clone()` preserves it. Region-aware formats such as Litematic and snapshots preserve named identity; merged formats export combined content rather than the region lifecycle itself.

## Placement and deterministic reads

`set_block_in_region` accepts the same full block-string grammar as `set_block_from_string`: IDs, properties, and optional SNBT. Parsed block entities belong to the selected region, and replacing the block removes obsolete block-entity data.

```python
import json

s.set_block_in_region(
    "west_wing",
    -19, 4, 8,
    "minecraft:chest[facing=east]{CustomName:'\"Supplies\"'}",
)

state = s.get_block_in_region("west_wing", -19, 4, 8)
assert state.name() == "minecraft:chest"
assert json.loads(state.properties_json())["facing"] == "east"
assert "Supplies" in s.get_block_entity_json_in_region("west_wing", -19, 4, 8)
```

Use region-specific reads whenever regions overlap:

- `get_block_in_region(name, x, y, z)`
- `get_block_string_in_region(name, x, y, z)`
- `get_block_entity_json_in_region(name, x, y, z)`
- `region_bounding_box_json(name)`
- `region_palette_json(name)`

Composite methods such as `get_block(x, y, z)` use stable precedence: the default region first, then named regions in lexical order. Region-specific methods avoid that ownership ambiguity.

## Transform scope

The short names stay optimized for the common single-region schematic:

| Scope | Rotate | Flip | Translate |
| --- | --- | --- | --- |
| Default `Main` region | `rotate_x/y/z(degrees)` | `flip_x/y/z()` | `translate(dx, dy, dz)` |
| One named region | `rotate_region_x/y/z(name, degrees)` | `flip_region_x/y/z(name)` | `translate_region(name, dx, dy, dz)` |
| Entire multi-region schematic | `rotate_schematic_x/y/z(degrees)` | `flip_schematic_x/y/z()` | `translate_schematic(dx, dy, dz)` |

```python
# Common case: transform Main only.
s.rotate_y(90)
s.translate(10, 0, 0)

# Multi-region case: transform exactly one named region.
s.rotate_region_y("west_wing", 90)
s.translate_region("west_wing", 0, 6, 0)

# Explicit edge case: transform every region as one rigid schematic.
s.rotate_schematic_y(90)
s.translate_schematic(100, 0, 20)
```

Only multiples of 90 degrees are valid. Invalid angles raise an exception and leave the schematic unchanged. Negative quarter turns wrap normally.

Positive rotations follow the schematic block-transform convention:

- `rotate_y(90)` is clockwise from above: east (`+X`) becomes south (`+Z`).
- `rotate_x(90)` maps south (`+Z`) to down (`-Y`).
- `rotate_z(90)` maps up (`+Y`) to west (`-X`).

Default and named-region rotation preserve that region's minimum corner. Whole-schematic rotation uses one shared overall bounding box and preserves relative spacing between regions; it does **not** rotate every region independently around its own corner. Blocks, block-state orientation, block entities, entities, and bounds move together.

Whole-schematic translation and rotation are transactional: coordinate overflow raises an exception without partially moving regions.

## Deep-cloned variants

`deep_clone()` returns a fully independent schematic. Use it to derive transformed variants without serializing or mutating the source:

```python
original = Schematic.create("module")
original.set_block(0, 0, 0, "minecraft:stone")

variant = original.deep_clone()
variant.rotate_y(90)
variant.translate(32, 0, 0)

assert original.get_block_name(0, 0, 0) == "minecraft:stone"
assert variant.get_block_name(32, 0, 0) == "minecraft:stone"
```

## Stamping

Use `stamp_box` for an explicit merged source box and `stamp_region` for one named source region. The target coordinate receives the explicit box minimum or the region's tight content minimum; internal storage padding never changes the anchor or clears unrelated destination cells.

```python
src = Schematic.create("module")
src.create_region("tower")
src.set_block_in_region("tower", 4, 0, 7, "minecraft:stone")
src.set_block_in_region("tower", 5, 0, 7, "minecraft:gold_block")

dst = Schematic.create("city")
dst.stamp_region(src, "tower", 100, 0, 200, "[]")
assert dst.get_block_name(100, 0, 200) == "minecraft:stone"
assert dst.get_block_name(101, 0, 200) == "minecraft:gold_block"
```

For an arbitrary source box:

```python
dst.stamp_box(
    src,
    4, 0, 7,       # source minimum
    5, 0, 7,       # source maximum, inclusive
    120, 0, 200,   # destination for the source minimum
    '["minecraft:air"]',
)
```

Stamping semantics are replacement-oriented and deterministic:

- Excluded source blocks are **skipped**; existing destination blocks and block entities remain unchanged.
- Non-excluded source air is written and therefore clears destination content.
- Every written cell removes a stale destination block entity before copying the source block entity, if present.
- Matching source entities are copied with the same offset.
- `stamp_region` selects exactly one named source region, even when source regions overlap.
- For merged `stamp_box` reads, `Main` wins overlaps, followed by named regions in lexical order.
- Destination-coordinate overflow is rejected before any block, block entity, or entity is copied.

`copy_region(...)` remains as a compatibility alias for `stamp_box(...)`, with the corrected skip semantics for exclusions.

## Complete multi-region example

```python
import json
from nucleation import Schematic

build = Schematic.create("build")
build.set_block(0, 0, 0, "minecraft:quartz_block")
build.create_region("gate")
build.set_block_in_region("gate", 10, 0, 0, "minecraft:oak_stairs[facing=east]")

assert json.loads(build.region_names_json()) == ["Main", "gate"]
build.rotate_region_y("gate", 90)
assert build.get_block_name(0, 0, 0) == "minecraft:quartz_block"
assert json.loads(build.get_block_in_region("gate", 10, 0, 0).properties_json())["facing"] == "south"

variant = build.deep_clone()
variant.rotate_schematic_y(90)
variant.translate_schematic(100, 0, 100)
```

The mandala below uses the same composition model: build one asymmetric petal, clone or flip it, then stamp the variants into a shared canvas.

<div align="center">
<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/transforms.png" width="520" alt="A four-fold symmetric mandala built by mirroring one petal into each quadrant">
</div>
