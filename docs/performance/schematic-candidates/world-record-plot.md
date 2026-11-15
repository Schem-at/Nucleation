# WorldRecordPlot: static survey and candidate doors

The user-provided plot is a useful source library, but no build from it is yet a validated benchmark. The selection was approximate and may cut mechanisms at its edges. The user also reported possibly broken designs, missing controls, missing container contents and old mechanics. This survey preserves those uncertainties; it does not simulate, repair or export the schematic.

The original remains unchanged in the local archive under source `user-world-record-plot`, path `WorldRecordPlot.litematic`. Its SHA-256 is `5476ac349cd4529d7a237828633b9e7a995d70b8937f70e4836d2f9f8db66c1e` (1,269,492 compressed bytes; 27,255,981 decompressed bytes). The binary is not included in this report. [The compressed metadata catalog](catalog/world-record-plot.json.gz) contains all counts, boundary positions, saved signs, inventories, components and candidate controls. [The map](world-record-plot-map.svg) shows their approximate placement.

## Saved scope

| Field | Observed value |
|---|---:|
| Native data version | 4671, Minecraft 1.21.11 |
| Litematic version/subversion | 7 / 1 |
| Regions | One, `Unnamed`, position `(0,0,0)` |
| Dimensions | 284 × 208 × 295 |
| Inclusive bounds | `(0,0,0)` through `(283,207,294)` |
| Cells / non-air blocks | 17,426,240 / 458,409 |
| Palette states | 1,278 |
| Pistons / sticky pistons | 8,067 / 23,781 |
| Piston heads / moving piston states | 699 / 5 |
| Observers | 41,380 |
| Levers / buttons | 414 / 274 |
| Container blocks / all block entities | 7,770 / 12,541 |
| Saved top-level entities | 318 |
| Saved pending block/fluid ticks | 0 / 0 |

The five `moving_piston` block states have no saved piston block entities anywhere in the file. This is a source-state concern requiring inspection before selecting their mechanisms; the cause is unknown. No assertion is made that saved pending ticks are sufficient to resume every build.

The 318 top-level entities include 150 furnace minecarts, 51 chest minecarts, 7 hopper minecarts, 13 other minecarts, 57 item frames, 38 armor stands, one oak boat and one small fireball. One nested cherry-boat passenger is also saved. They are not all relevant to a particular candidate, but they preclude treating the whole plot as an entity-free circuit.

Native save version does not establish the design's intended version. A saved sign at `(150,10,3)` explicitly says 1.21.10 and 1.21.11 broke many doors. Other signs describe cooldowns, mob/cart dependencies, source-specific fixes and broken builds. The catalog retains these source observations without treating them as current functional verdicts.

## Inventory evidence

Of 7,770 container block entities, 7,749 have explicit empty `Items` lists, 20 have no `Items` tag, and one has occupied slots. The empty lists include every saved hopper (4,331), dropper (2,276), dispenser (370), furnace (349), chest (102), trapped chest (143), barrel (36) and crafter (113). The 19 shulker boxes and one decorated pot have no `Items` tag. Empty and absent are recorded separately.

Only the shelf at `(150,1,11)` retains positive-count block-entity item stacks: one diamond, one gold ingot and one totem. A recursive scan of all block-entity tags found those same three stacks and no other retained positive-count stacks. No nested `minecraft:container` entries were found in the container records. All 51 chest minecarts and 7 hopper minecarts also have explicit empty `Items` lists.

Entity display/equipment data is different: 75 item units are retained recursively, including named signs, armor, heads and a fire charge. Their component keys include custom names, lore, profiles, equipment colors, writable-book content and block-entity data. Those observations must not be interpreted as working mechanism inventory.

These facts describe this file. They do **not** establish that its original-world inventories were nonempty or that Litematica removed anything. A working source, build instructions or controlled reference must establish the required contents before any repair is proposed. Restored contents would be a separately documented variant, not a silent change to this source.

## Approximate-selection boundary audit

The only region is also the whole selection. The catalog records every qualifying block position and state on each exact face, as well as each physical component's six face distances.

| Face | All non-air blocks | Mechanism/control/container blocks |
|---|---:|---:|
| x = 0 | 900 | 1 |
| x = 283 | 1,443 | 360 |
| y = 0 | 77 | 16 |
| y = 207 | 0 | 0 |
| z = 0 | 1,017 | 1 |
| z = 294 | 879 | 10 |

The x-max face is particularly concerning: it contains 192 observers, 72 piston bases, 31 repeaters, 26 dust blocks, 8 hoppers, 7 droppers and additional mechanism blocks. The x-min contact is a lever; z-min is a redstone torch. Counts are per face, so a corner block can contribute to more than one face. The mechanism filter includes controls, containers, piston-related states, observers, redstone blocks/dust/torches/lamps, repeaters/comparators, rails, slime/honey, targets, sculk and copper bulbs. Ordinary support/wiring blocks are covered by the all-non-air count.

Any component touching a face is marked `possibly_truncated`. An activation failure there is not evidence of an engine defect. The 20 shortlisted components do not touch a face, but proximity alone does not prove completeness: diagonal power, quasi-connectivity, movable structures or nearby controls may cross their component bounds.

## Geometry-derived shortlist

The scanner found 1,101 six-neighbour physical components containing at least eight non-air blocks. It includes saved floors and signs: shared floors can join unrelated mechanisms, while diagonal dependencies can split one mechanism. Each box below is an exact connected-component box, **not an approved extraction box**. Sign associations are hypotheses. Candidate 01's 218 saved blocks, for example, should not be equated with its nearby sign's claimed 216-block record.

Six sign-anchored candidates are followed by controlled interior components sampled across four height bands. All are `unvalidated`. “Edge” is the minimum distance in blocks from the component box to any selection face. `L` means lever, `B` button; coordinates are source-relative `(x,y,z)`. Detailed states and all six distances are in the catalog.

| ID | Candidate lead | Inclusive component bounds | Blocks | Controls | Containers | Edge |
|---|---|---|---:|---|---:|---:|
| 01 | LucasDaRedstone 3×3 glass hipster label | `(159,3,212)`–`(164,9,221)` | 218 | L `(163,3,212)` | 9 | 3 |
| 02 | 4×4 full-lamp record label | `(239,52,3)`–`(246,62,5)` | 123 | L `(240,52,3)`, `(240,54,5)` | 6 | 3 |
| 03 | Semi-seamless 3×3 label | `(172,2,275)`–`(179,17,277)` | 138 | B `(178,17,275)` | 6 | 2 |
| 04 | Seamless hipster category | `(154,2,190)`–`(161,6,201)` | 209 | B `(157,3,193)`, L `(158,2,193)` | 6 | 2 |
| 05 | SoulBanished repair note | `(112,53,148)`–`(127,58,160)` | 504 | None in component | 7 | 53 |
| 06 | SpaceWalker / g4me4u note | `(113,68,64)`–`(133,86,87)` | 181 | None in component | 1 | 64 |
| 07 | Controlled lower-band component | `(166,2,45)`–`(172,12,48)` | 102 | B `(166,2,47)` | 0 | 2 |
| 08 | Simple interior component | `(128,53,86)`–`(140,56,87)` | 41 | B `(136,54,86)` | 0 | 53 |
| 09 | Controlled upper-band component | `(154,119,2)`–`(160,126,3)` | 46 | L `(160,126,2)` | 0 | 2 |
| 10 | Simple high interior component | `(159,155,87)`–`(162,162,88)` | 44 | B `(159,155,88)` | 0 | 45 |
| 11 | Flying-movement sign nearby | `(85,2,2)`–`(94,4,17)` | 268 | L `(89,2,2)` | 0 | 2 |
| 12 | Non-locational-version sign | `(155,62,86)`–`(160,65,89)` | 41 | L `(157,63,89)` | 0 | 62 |
| 13 | Simple interior lever component | `(132,104,149)`–`(139,109,151)` | 59 | L `(133,108,149)` | 0 | 98 |
| 14 | Flat high-band component | `(91,155,276)`–`(108,155,280)` | 68 | B `(91,155,276)` | 0 | 14 |
| 15 | Lower-band lever component | `(71,2,3)`–`(82,4,19)` | 318 | L `(82,2,3)` | 0 | 2 |
| 16 | Interior lever component | `(162,53,86)`–`(169,57,88)` | 49 | L `(163,57,86)` | 0 | 53 |
| 17 | Two-button component | `(138,104,23)`–`(142,112,26)` | 70 | B `(141,104,23)`, `(142,104,24)` | 0 | 23 |
| 18 | Floor-dependence notes nearby | `(110,155,170)`–`(114,157,176)` | 82 | B `(114,155,172)` | 0 | 50 |
| 19 | Larger interior lever component | `(215,3,128)`–`(227,24,131)` | 529 | L `(219,3,128)` | 0 | 3 |
| 20 | Interior button component | `(186,61,234)`–`(189,71,237)` | 67 | B `(187,63,237)` | 0 | 57 |

For a first bounded reference attempt, 08, 10, 12, 13 and 16 have saved controls, no container blocks in their components and substantial face clearance. This is a scope advantage, not proof they are complete or functional. For richer doors, 01 and 02 have useful labels and controls but all their saved container inventories are empty. Candidate 01's nearby sign specifies roughly 40 seconds of cooldown; 04 has patience/12–20-second notes. Candidate 03 has an “epic carts” note. No saved entities occur within these candidate boxes plus two blocks, but a note may refer to a missing or more distant dependency. Candidates 05 and 06 lack saved controls in their components. The named authors do not establish that either is the existing SpaceWalker/SoulBanished hipster golden fixture.

Before admission, inspect adjacent components and a dependency/travel halo, identify the intended aperture and controls, establish required initial inventory/entity data, then capture real activation and idle controls on native Minecraft 1.21.11. Include sufficient full open/close/reset cycles and cooldowns. Assert intended function independently of engine matching. Compare fresh engine and vanilla tick-end blocks, live comparator outputs and inventories; moving-piston block-entity state and dynamic entities require broader capture than the current community oracle provides. Ordered intra-tick delivery parity remains a separate check. Quarantine a broken, incomplete, truncated or unsupported source without labeling it a simulator regression.

## Reproduction and scope

The read-only [survey tool](https://github.com/Schem-at/Nucleation/blob/v0.12.3/tools/inspect_tick_world_record_plot.py) uses the existing bounded NBT reader. It permits one region, up to 128 MiB decompressed NBT, 25 million cells and one million non-air blocks. It refuses to overwrite an output path and makes no source changes.

```bash
python3 -B tools/inspect_tick_world_record_plot.py \
  --input /path/to/WorldRecordPlot.litematic \
  --output /tmp/world-record-plot-survey.json
```

Static consistency checks confirmed source hash preservation, exact agreement between metadata and decoded non-air totals, projection totals, unique component-backed candidate boxes, face flags and inventory counts. No Rust/Java build, simulation, runtime performance measurement or whole-plot benchmark was performed for this survey.
