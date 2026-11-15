# World-record plot validation selection

All 803 geometric candidates were reread from hash-verified Litematic NBT. This is a selection and risk report, not a simulation pass list. The 746 first-pass and 57 floor-derived candidates remain intact. Three canonical infrastructure clusters are excluded from this population.

## Overlapping risk observations

| Observation | Candidates |
|---|---:|
| authored entities | 61 |
| empty or absent inventory observation | 624 |
| gravity blocks runtime entity watch | 72 |
| moving piston missing block entity | 2 |
| no lever or button | 183 |
| rail or entity dependency review | 194 |
| restored platform and neighbor dependency | 57 |
| small or single piston review | 76 |
| source boundary contact | 87 |
| source sign warns failure | 38 |

The categories overlap. Empty inventories do not establish lost items; boundary contacts do not establish a cut-off door. Single-piston and tiny mechanisms need inspection and are not automatically rejected. No-piston-base candidates are separately flagged for fragment review.

## Exclusive triage strata

| First applicable stratum | Candidates |
|---|---:|
| authored entities | 61 |
| empty or absent inventory observation | 401 |
| moving piston missing block entity | 2 |
| no lever or button | 117 |
| remaining manual control candidates | 67 |
| restored platform and neighbor dependency | 1 |
| small or single piston review | 70 |
| source boundary contact | 84 |

## Selected protocols

| Source part | Role | Proposed independent output |
|---|---|---|
| `part0465.litematic` | primary | Compact vertical 3x3 |
| `part0622.litematic` | primary | Vertical 5x5, three-block depth |
| `part0117.litematic` | primary | Horizontal 6x6 slime/honey hatch |
| `part0101.litematic` | primary | Sign-labelled vertical 7x7 |
| `part0004-floorpart0119.litematic` | primary with inventory risk | Floor-derived vertical 2x2 |
| `part0468.litematic` | reserve | Deep vertical 2x2, long piston chains |
| `part0074.litematic` | reserve | Horizontal 7x7 slime/honey hatch |
| `part0369.litematic` | button reserve | Button-driven compact 2x2 |
| `part0425.litematic` | diagnostic counterexample | Source-sign broken mechanism |
| `part0004-floorpart0210.litematic` | diagnostic inventory button | Floor-derived button/tripwire 3x3 |

Each selected JSON record contains the source SHA-256, exact control position and saved facing, full aperture cell list, reset requirements, timing, unit-test proposals and a disabled benchmark specification. Coordinates are source-global: use the fixture exporter origin when translating into a test world.

For lever candidates, the proposed adapter exposes `door_open: bool`. Restore a pristine checkpoint with RNG seed 0, settle 20 ticks, and verify the saved open aperture. Request closed/open/closed/open at ticks 21/181/341/501 and assert quartz/air/quartz/air at 180/340/500/660. Run an independent 660-tick idle capture. The 160-tick allowance is a test budget, not a measured latency. Measure actual settling time after successful captures.

Button candidates first require pulse characterization. Do not replace a button with a lever or directly alter power. The source-sign broken mechanism is deliberately diagnostic and has no functional-success oracle. No candidate enters timed benchmarks until its phase assertions and fresh vanilla comparison pass.

Each reset must restore inventories, block entities, entities, pending queues and RNG, not just visible blocks. Compare full watched tick-end states, comparator outputs and inventories, and report unsupported/runtime-entity limits. Checkpoint creation, restoration, rendering and assertions stay outside the benchmark timer.

### Compact vertical 3x3: `part0465.litematic`

Small clear quartz aperture; nine piston bases; observer ordering and delay-3 repeater. Sign warns about input timing.

Content: `1346e80046395ef3c4e47014d402cf2ee401967e287968022f40584cc71eab39`. Control: `[159, 59, 212]` / `minecraft:lever[face=wall,facing=north,powered=false]`.

Proposed plane: `[[156, 55, 213], [158, 57, 213]]`. Initial plane verified against saved NBT: 9 air cells. Runtime output remains unvalidated.

### Vertical 5x5, three-block depth: `part0622.litematic`

Clear 5x5 quartz perimeter; comparator and water-cauldron analog source; slime and concrete powder require runtime entity monitoring.

Content: `0579e496cd29babb04a56e2f33f44e693bfc4021b7fcb01a77e7ff6266e09406`. Control: `[180, 56, 128]` / `minecraft:lever[face=wall,facing=north,powered=false]`.

Proposed plane: `[[174, 56, 130], [178, 60, 130]]`. Initial plane verified against saved NBT: 25 air cells. Runtime output remains unvalidated.

### Horizontal 6x6 slime/honey hatch: `part0117.litematic`

Clear horizontal aperture and parked quartz panel; mixed slime/honey translation, observer chains, wall and trapdoor notifications.

Content: `0fe86bdc4514fd616f83eaa78c492c76f835b9bdde894ffe3c61911a1055296d`. Control: `[93, 54, 191]` / `minecraft:lever[face=wall,facing=north,powered=false]`.

Proposed plane: `[[92, 55, 198], [97, 55, 203]]`. Initial plane verified against saved NBT: 36 air cells. Runtime output remains unvalidated.

### Sign-labelled vertical 7x7: `part0101.litematic`

Source sign says Smallest 7x7; quartz panel parked below the opening, mixed slime/honey, note blocks and trapdoors.

Content: `bfa34da16e8e609dd9cdc718d8268c9d6887e2e27049f173619df55e310eee07`. Control: `[92, 53, 179]` / `minecraft:lever[face=wall,facing=south,powered=false]`.

Proposed plane: `[[87, 60, 176], [93, 66, 176]]`. Initial plane verified against saved NBT: 49 air cells. Runtime output remains unvalidated.

### Floor-derived vertical 2x2: `part0004-floorpart0119.litematic`

Clear four-cell aperture, four sticky pistons, actual lever. Two opposing empty droppers are retained as saved; do not infer missing contents or repair. Exercises restored platform context.

Content: `db88c79d2d2e31362af0c280e29a6a2f04441901283c0f47d4c610f39b086cb7`. Control: `[144, 55, 170]` / `minecraft:lever[face=wall,facing=north,powered=false]`.

Proposed plane: `[[143, 53, 171], [144, 54, 171]]`. Initial plane verified against saved NBT: 4 air cells. Runtime output remains unvalidated.

### Deep vertical 2x2, long piston chains: `part0468.litematic`

Small opening with 25 piston bases and long horizontal chains; quartz stairs, lamps and multiple repeater delays. Geometric inference needs vanilla confirmation.

Content: `76f26e2e5a63e647b73c65364bb683b003853af1683a58cf648508caddb4b55e`. Control: `[153, 106, 214]` / `minecraft:lever[face=wall,facing=west,powered=false]`.

Proposed plane: `[[161, 104, 213], [162, 105, 213]]`. Initial plane verified against saved NBT: 4 air cells. Runtime output remains unvalidated.

### Horizontal 7x7 slime/honey hatch: `part0074.litematic`

Larger counterpart to the 6x6 horizontal mechanism; tests scaling of moving panels with a separate source and exact aperture.

Content: `14b99acabae4db6e36b726452499cf1882a6400bdc0ceb86e0cbf132a0382e6e`. Control: `[78, 53, 191]` / `minecraft:lever[face=wall,facing=north,powered=false]`.

Proposed plane: `[[75, 55, 199], [81, 55, 205]]`. Initial plane verified against saved NBT: 49 air cells. Runtime output remains unvalidated.

### Button-driven compact 2x2: `part0369.litematic`

Actual ceiling stone button; quartz aperture with six piston bases. Button pulse versus persistent lever timing must be established, not emulated with an injected power block.

Content: `391fb5907b699f81112a21250f7dcb95d7816bc825540d554a99ccc515afe76a`. Control: `[141, 154, 255]` / `minecraft:stone_button[face=ceiling,facing=east,powered=false]`.

Proposed plane: `[[139, 155, 255], [140, 156, 255]]`. Initial plane verified against saved NBT: 4 air cells. Runtime output remains unvalidated.

### Source-sign broken mechanism: `part0425.litematic`

Source sign explicitly says broken now. Detector rail powered=true without an authored entity and update-order warning. Preserve as a counterexample; matching engines must not certify functionality.

Content: `ac23048e837cf1570c865a8a80395cc89e7e07831f9b53a3a5974dd7e99704a3`. Control: `[145, 17, 234]` / `minecraft:lever[face=floor,facing=south,powered=false]`.

Proposed plane: `None`. Initial plane verified against saved NBT: 0 air cells. Runtime output remains unvalidated.

### Floor-derived button/tripwire 3x3: `part0004-floorpart0210.litematic`

Clear aperture, button and tripwire controls, two empty droppers, one triggered=true. Exercises inventory and saved-phase risks; no invented items or lever replacement.

Content: `6130b77d5b51cb1cdfe0b86528c441d58a93d3c9db91178fe4e706ce2a1c92b1`. Control: `[172, 56, 278]` / `minecraft:stone_button[face=floor,facing=south,powered=false]`.

Proposed plane: `[[166, 53, 277], [168, 55, 277]]`. Initial plane verified against saved NBT: 9 air cells. Runtime output remains unvalidated.

## Reproduce

```sh
python3 tools/curate_tick_plot_validation.py --archive /path/to/Nucleation-schematic-archive
```

The script changes only this selection report. Archive validation status and the original sources remain unchanged. Fresh simulation evidence is maintained by the separate batch and vanilla validation runners.
