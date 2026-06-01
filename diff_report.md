# Diff demo — `4bit_adder.litematic`

B = A shifted by (7,0,4); `minecraft:redstone_wire` → `minecraft:white_concrete` (29 cells); ~1/40 blocks removed; 3 glass added.

## Preset `exact` (material-sensitive)

| metric | value |
|---|---|
| recovered translate | (7, 0, 4) |
| edit distance | 7 |
| alignment support | 0.48 |
| added | 3 |
| removed | 2 |
| changed | 0 |
| swapped cells | 28 |
| palette swaps | minecraft:redstone_wire|east=side|north=none|power=0|south=none|west=side→minecraft:white_concrete, minecraft:redstone_wire|east=none|north=side|power=15|south=side|west=none→minecraft:white_concrete |
| change regions | 3 |

| region | kind | bbox | cells |
|---|---|---|---|
| 1 | Added | (3,0,4)–(5,0,4) | 3 |
| 2 | Removed | (21,0,5)–(21,0,5) | 1 |
| 3 | Removed | (17,1,6)–(17,1,6) | 1 |

## Preset `redstone_computational` (functional)

| metric | value |
|---|---|
| recovered translate | (7, 0, 4) |
| edit distance | 6 |
| alignment support | 0.48 |
| added | 3 |
| removed | 2 |
| changed | 0 |
| swapped cells | 28 |
| palette swaps | redstone_wire→solid |
| change regions | 3 |

| region | kind | bbox | cells |
|---|---|---|---|
| 1 | Removed | (21,0,5)–(21,0,5) | 1 |
| 2 | Added | (3,0,4)–(5,0,4) | 3 |
| 3 | Removed | (17,1,6)–(17,1,6) | 1 |


## JSON (exact)

```json
{"added":[{"block":"minecraft:glass","pos":[5,0,4]},{"block":"minecraft:glass","pos":[3,0,4]},{"block":"minecraft:glass","pos":[4,0,4]}],"changed":[],"distance":7,"palette_swaps":[["minecraft:redstone_wire|east=side|north=none|power=0|south=none|west=side","minecraft:white_concrete"],["minecraft:redstone_wire|east=none|north=side|power=15|south=side|west=none","minecraft:white_concrete"]],"removed":[{"block":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[17,1,6]},{"block":"minecraft:gray_concrete","pos":[21,0,5]}],"schema":"nucleation.diff/1","support":0.4833333194255829,"swapped":[{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[7,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[11,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[9,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[12,0,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[10,0,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[16,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[15,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[14,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[18,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[10,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[8,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[11,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[8,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[16,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[21,1,5],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=none,north=side,power=15,south=side,west=none]","pos":[19,2,5],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[8,0,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[7,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[13,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[17,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[12,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[12,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[14,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[18,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[13,1,4],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[10,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[15,1,6],"to":"minecraft:white_concrete"},{"from":"minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]","pos":[9,1,4],"to":"minecraft:white_concrete"}],"transform":{"rotate":{"steps":[]},"translate":[7,0,4]}}
```
