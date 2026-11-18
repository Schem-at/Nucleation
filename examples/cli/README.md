# Formula and query examples

Run these shell helpers from the repository root with the installed `nucleation`
and `jq` on PATH. The chest, block-entity and entity queries also use the `nbt` command
from the Python package `nbtlib` (for example, `uv tool install nbtlib`).

## Curves

```sh
examples/cli/curve helix.schem '[12*cos(t), 4*t, 12*sin(t)]'
examples/cli/curve knot.litematic \
  '[(18+7*cos(3*t))*cos(2*t), 7*sin(3*t), (18+7*cos(3*t))*sin(2*t)]' \
  minecraft:oxidized_copper 720 1.5
```

The expression returns `[x,y,z]` with Y up. The helper evaluates it as jq code.
Use `t` for radians from zero through 2π or `.` for a parameter from zero
through one. Optional arguments select the block, number of intervals and brush
radius in blocks. Samples snap to the nearest block, duplicates collapse, and
the curve's bounding box starts at the schematic origin. Increase the interval
count when a rapidly varying curve has gaps. Radius zero gives a single-block
stroke.

## Queries

```sh
examples/cli/query block-entities build.litematic | jq 'group_by(.id) | map({id: .[0].id, count: length})'
examples/cli/query block-entities build.litematic | jq '.[] | select(.nbt.Items != null) | {id, pos, items: .nbt.Items}'
examples/cli/query chests build.litematic | jq '.[] | {pos, items}'
examples/cli/query entities build.schem | jq '.[] | {id, pos}'
examples/cli/query parts build.schem | jq -r '.[] | [.count, .block] | @tsv'
```

Use `block-entities` (alias `tile-entities`) for every saved block-entity
record. Each row contains its `id`, schematic-local `pos`, and the Sponge v3
`Data` compound decoded as JSON under `nbt`. The inventory filter above
includes any block entity with a saved item list, including empty lists.
Regular entities such as minecarts use the same row format under `entities`.

Chest rows use structure-local coordinates and retain the raw item stacks,
including slot, item ID, count and saved components. A double chest has one
row per block. An empty list means an explicitly saved empty inventory. A null
means no item list was saved. The loot table is reported when present. Chest
minecarts appear in the entity query, where their stacks are in `.nbt.Items`.

The chest query and curve helper use Java structure SNBT, which supports up to
256 blocks along each axis and 262,144 cells in the bounding box in this branch.
The entity query uses Sponge v3 and returns its positions and full entity data.
The parts query sums placed blocks by block ID across facings and other state
variants. Counts include structure blocks such as extended piston heads. For
the individual state counts, use `nucleation info build.schem --json | jq .palette`.
