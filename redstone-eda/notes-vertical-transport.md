# VERTICAL TRANSPORT — how a bus changes level

Status: probed 2026-08-10, mc-tick. Every row below cites the probe that fixed
it. Executable templates: **`vforms.py`**. Probes:

* **`probe_vertical_forms.py` — 33/33.** The four forms, their densities, their
  latencies, why DOWN is asymmetric, and the slope-transparency rule.
* **`probe_spiral_tiling.py` — 26/26.** The user's "spiral staircases tile if
  they are offset 180 degrees", generalised to an integer law and checked
  against 17 simulated configurations.
* **`showcase/showcase_vertical.py` — 2/2.** `vriser_ladder8.schem` (8 bits,
  256 patterns) and `vriser_ring53.schem` (4 bits, 16 patterns), saved only
  after passing.

Physics this file builds on, all in `TRANSPORT_MODEL.md`: the mechanism rows
(torch = row 7, dust = 1/2, strong block = 3, weak block = 4,
transparent support = 11), the CUT-cell/DIODE-cell split, the POINTING LAW, and
the cap law. Materials: `notes-material-model.md`.

Read this with `crosswire_tiles.md`: that file is the 90°-crossing half of the
fabric, this one is the level-change half.

## Why this file exists

`router.py` has one vertical gadget (a torch ladder) with a fixed clearance
rule, and `src/design_corridor.rs` cannot represent a level change at all — its
node is a *column*, claimed by one bus for its entire height. Every question a
level-changing bus actually asks was unanswered: how many bits fit in a shaft,
whether the shaft can go down as well as up, what a shaft costs per y, and what
its neighbour pitch is. Those are the rows below.

The single most useful result is that **up and down are different mechanisms
with a 3× density gap**, so a router must plan them separately:

* **UP** has an *active* carrier (the torch) that needs one 1×1 column per bit
  and refreshes to 15, so it neither decays nor needs a repeater. Torch towers
  may stand at **pitch 1**.
* **DOWN** has no active carrier at all. Every descending form is dust on a
  staircase, which costs 2 blocks per y per bit, decays 1 ss per y, and needs a
  **≥ 3×3 footprint** to fold into a shaft.

---

## THE FORM TABLE

Five forms (`vforms.py`). "blocks/y/bit" is the marginal cost of one more y of
rise — measured on a mid-height level of a real 8-bit rig, not counted by hand
(`probe_vertical_forms` section M). "xz-claim/bit" is the footprint a router
must reserve, *including* the form's required neighbour pitch.

### A. rate, latency, polarity, reach

| # | form | direction | rate | delay | inverting | signal strength | blocks/y/bit |
|---|---|---|---|---|---|---|---|
| 1 | `torch_ladder` | **UP only** | **2 y per torch** | 2 gt per torch = **1 gt per y** | **yes, once per torch** (even count = non-inverting) | **refresh to 15 at every cap — no reach limit** | **1.000** |
| 2 | `glass_tower` (transparent tower) | **UP only** (diode) | 1 y per cell | **0 gt** | no | −1 per y; 14 y per source | 2.000 |
| 3 | `ring_riser` (spiral staircase) | **UP and DOWN** | 1 y per cell | **0 gt** | no | −1 per y; 14 y per source, then a repeater | 2.000 |
| 4 | `half_slope` | UP and DOWN | 1 y per **2** x cells | 0 gt | no | −1 per cell = **−2 per y** | 2.000 |
| 5 | `stair_1to1` | UP and DOWN | 1 y per 1 x cell | 0 gt | no | −1 per y | 2.000 |
| 5b | `stair_1to1` on all-transparent supports | **UP only** (diode) | 1 y per 1 x cell | 0 gt | no | −1 per y | 2.000 |
| 6 | `repeater_drop` (row 4 + row 5 of the mechanism table) | **DOWN** | 1 y per **2** x cells (est.) | **2 gt per y** | no | **refresh to 15** | ~3 (est.) |

### B. footprint, neighbour pitch, support, preconditions

| # | form | footprint per bit | legal neighbour pitch | support requirement | precondition that bites |
|---|---|---|---|---|---|
| 1 | `torch_ladder` | **1×1 column**; ports add one z lane on each side, at the entry level and at the exit level only | **1** (!) with ports on **alternating** sides; 2 with same-side ports | the column is its own support; only the two port dusts need a floor | **the port lanes, not the towers, set the pitch.** Two pitch-1 ladders with ports on the same side merge their entry dusts into one net, and that net is a T — a T does not point into the bases (POINTING LAW), so the array goes *dead*, not leaky (L5) |
| 2 | `glass_tower` | 2×1 | **z-pitch 2** (pitch 1 leaks at both dy=0 and dy=1) | **every support must be a NON-conductor** (glass, stained glass, top slab) | all-solid supports kill it after one cell: in this form every support *is* the CUT cell of the dust below it (G2). Bottom slabs are vanilla-illegal — `audit.py` |
| 3 | `ring_riser` | 4.50 (3×3, 2 bits) / **4.00** (4×3, 3 bits) / **3.75** (5×3, 4 bits) / 4.125 (11×3, 8 bits) / 4.00 (2×2, 1 bit) | **ring pitch = sx + 1**: one empty column between rings. Flush rings always leak | **every support SOLID**, and the cell directly **above every dust must be AIR** | footprint **≥ 3 in both axes** or the perimeter has a chord and the bits merge (T1); bits at path offset **≥ 3** (T2) |
| 4 | `half_slope` | 1×1 per line, and lines **stack at 2 y pitch** in the same 1×1 | 2 y between stacked lines; lane pitch 2 sideways | support[i] **transparent iff dust i is the lower end of a step** (⇒ ascending: glass at odd i, solid at even i) | all-transparent supports **merge** the stacked lines (H3); all-solid cuts every step |
| 5 | `stair_1to1` | 1×1 per line, **cannot stack** | — | all-solid (bidirectional) or all-transparent (diode) | **a 1:1 slope cannot carry a second line 2 y above it at all** — every dust is the lower end of a step, so the alternation rule degenerates to all-transparent, which merges (H5) |
| 6 | `repeater_drop` | ~2×1 per y | repeater rules (`probe_station`) | solid floor (static legality), and the weakly powered block the repeater's back reads | dust cannot read weak power, so only a *device* may stand at the bottom of a drop (D2/D3) |

### Probe citations, per row

| # | probe |
|---|---|
| 1 | `probe_vert.py` (the original primitive); `probe_vertical_forms.py` L1 (rise), L2 (parity), L3 (8 gt for 4 torches), **L4 (8 towers at pitch 1, 256/256 patterns, zero leak at every exit *and* every entry)**, L5 (the same-side failure), L6 (pitch 2) |
| 2 | `probe_vertical_forms.py` G1 (glass and top slab), G2 (all-solid negative control), G3 (diode), G4 (z-pitch 1 leaks / 2 is clean) |
| 3 | `probe_vertical_forms.py` S0 (2×2, one bit, both directions), S1 (up), S2 (down), S3 (−1 ss both ways), S4 (repeater refresh); `probe_spiral_tiling.py` T1–T8 |
| 4 | `probe_vertical_forms.py` H3 (all-transparent merges), H4 (alternating isolates, 4/4); `materials._verify_slope`; `notes-material-model.md` §2 |
| 5 | `probe_vertical_forms.py` H1 (solid, both ways), H2 (glass, diode), H5 (cannot stack); `probe_pivot.py` E (14-step descent) |
| 6 | `probe_vertical_forms.py` D2/D3; `TRANSPORT_MODEL.md` rows 4 and 5 |

---

## WHY DOWN IS ASYMMETRIC

Three measurements, and then the geometric consequence.

1. **D1 — a torch powers the block ABOVE it and nothing beneath it.** Probed
   directly: dust on the block above a lit torch reads 15, dust on the block
   under the torch's attachment reads 0. A torch is the only compact active
   carrier in the game, and it only ever hands the signal *up*. There is no
   descending torch form to find.
2. **D2 — a dust weak-powers the block it sits on, and dust cannot read weak
   power.** So a dust cannot hand its signal down through its own floor either:
   the dust one level under a driven dust's support block reads 0.
3. **D3 — but a repeater whose BACK is that weakly powered block fires**
   (ss 15). That is the *only* active descent: one y per repeater station, 2 gt
   and about two horizontal cells each. It refreshes to 15, so it is the
   unbounded option, and it is what a very long drop should use.

So every *passive* descent is dust on a staircase, and that is where the
footprint asymmetry comes from. Going **up**, a step's upper dust may sit on a
non-conductor, so a riser can fold into a 2-wide zigzag (form 2): the support
above each dust doubles as the next dust's floor. Going **down**, the diode rule
demands the upper dust's support be a **conductor** while the cut rule demands
the cell directly above each dust be a **non-conductor** — and those are the
same cell in a 2-wide fold. The fold is over-constrained, so a descending riser
must leave the cell above every dust empty, which forces the path to walk a
closed loop of length ≥ 8 — i.e. a **spiral staircase on a ≥ 3×3 footprint**.

That is the whole content of the user's "going down is harder and usually ends
up with spiral staircases", and the probes agree with it exactly: form 2
descends nowhere (G3), form 3 descends in both a 2×2 (one bit, S0) and a 3×3
(two bits, T6).

---

## THE RING LAW — what "offset 180 degrees" actually is

The user's packing claim was: *"for horizontal spiral staircases can tile if
they are offset 180 degrees."* It is true, and the reason generalises into one
integer.

A ring riser is dust on the **perimeter of an sx × sz footprint, one cell per
y**, so **y == path index**. Two cells of that path are planar-adjacent iff
they are *consecutive on it* — provided the cycle is **chordless**, which the
perimeter of a rectangle is exactly when `sx >= 3 && sz >= 3`. Put a second bit
on the same ring at path offset `sep`, and in **every column the two bits
share** they sit exactly `sep` apart in y. So the whole legality question is
one integer:

| `sep` | what happens | measured |
|---|---|---|
| 1 | the two bits are one step apart in every shared column: **one net** | T2 (leak 15) |
| 2 | the upper bit's SOLID support lands in the lower bit's **CUT cell** and severs it: one bit is **DEAD**, and foreign power sits inside its cells | **T3** |
| **≥ 3** | **legal.** No pair of bits is ever 0 or 1 apart in y in any shared column, so neither a merge nor a step exists | T2, T4, T5 |

A period-8 ring (3×3) therefore holds `floor(8/3) = 2` bits, and their offset
must be 3 or 4 — **offset 4 of 8 is "180 degrees"**, verified 4/4 with zero
crosstalk in one 3×3 footprint. In general a ring holds **`floor(perimeter/3)`
bits**, and `vforms.ring_bits` spreads them evenly rather than bunching them at
exactly 3, because the spare separation is what buys isolation across the seam
to the *next* ring (T7: offsets `[0,4]` tile at pitch 4, `[0,3]` do not).

Measured capacities:

| ring | perimeter | bits | offsets | xz-claim/bit | patterns |
|---|---|---|---|---|---|
| 2×2 | 4 | 1 | `[0]` | 4.00 | S0 (up **and** down) |
| 3×3 | 8 | **2** | `[0, 4]` = **180°** | 4.50 | 4/4 |
| 4×3 | 10 | 3 | `[0, 3, 7]` | 4.00 | 8/8 |
| **5×3** | 12 | **4** | `[0, 3, 6, 9]` | **3.75** ← densest | 16/16 |
| 6×3 | 14 | 4 | `[0, 4, 7, 10]` | 4.50 | 16/16 |
| **11×3** | 24 | **8 (a byte)** | `[0, 3, …, 21]` | 4.125 | 256 (full matrix ≤ 8 bits) |

`probe_spiral_tiling.legal(sx, sz, offsets)` is the closed form — footprint ≥
3×3 and every pair of offsets ≥ 3 apart around the perimeter — and **T8 checks
it against all 17 simulated configurations with zero mismatches**, so a router
can use the predicate instead of a table.

Two properties that make this a real *bus* form, not just a wire trick:

* **all bits enter on one level and leave on one level.** Bit `p` occupies
  `ring[(k+p) mod m]` at `y0+k`, so at any single y every bit has exactly one
  dust, in a different column. No level-shift adapter is needed at either end.
* **the packing is direction-agnostic.** The same geometry carries all bits
  downward (T6), because nothing in the argument mentions which end is driven.

---

## THE SLOPE RULE — "every second block is transparent"

Verified, with the precondition the original statement did not carry.

* **H1** — a 1:1 staircase on **all-solid** supports conducts both ways
  (1 y per x cell, −1 ss per y). Nothing special is needed for a *single* line:
  the cell above each dust is empty anyway.
* **H2** — the same staircase on **all-transparent** supports conducts **up
  only**. That is the transparent diode, and it is a free one-way isolator.
* **H3** — two lines 2 y apart on a half slope with **all-transparent**
  supports **MERGE**: the foreign diagonal from the lower line's dust up to the
  upper line's dust one x back survives, because its cut cell is the upper
  line's glass support.
* **H4** — the same pair with **alternating** supports is isolated in all four
  patterns. The rule, in the form a planner can evaluate:

  > `support[i]` must be **transparent** iff dust `i` is the **lower end of an
  > in-use 1-y step**; otherwise **solid**.

  On an ascending half slope that is glass at odd `i`, solid at even `i` — the
  user's "every second block is transparent". The reason it is *exactly* every
  second block is that a support cell does two jobs at once: it is this line's
  floor **and** the cut cell of the line 2 y beneath it (`support[i]` sits at
  `y_i − 1 == (y_i − 2) + 1`). Transparent where the lower line needs its own
  step to survive; solid where the *foreign* diagonal must be severed.
* **H5** — and therefore **the slope must be a half slope.** On a 1:1 slope
  every dust is the lower end of a step, so the rule degenerates to
  all-transparent, which H3 measures as a merge. A half slope is the steepest
  slope that can carry a stacked bus, because half its dusts are flat cells
  whose cap is allowed to be a conductor. Measured as well as derived.

---

## CELL TEMPLATES

Closed forms, in the style of `crosswire_tiles.md`: every cell is a formula in
a level index or a path index, tile-local coordinates, so these port to Rust
mechanically. Reference implementations: `vforms.py` (executed by both probes,
so the formulas below are the ones actually verified).

Any solid block works wherever a material is "solid". Any **non-conductor that
is sturdy** works wherever one is "transparent": `glass`,
`white_stained_glass`, `smooth_stone_slab[type=top]` — all three probed
(G1 covers glass and the top slab). Bottom slabs are vanilla-illegal.

---

### `LADDER_TOWER` — 1×1, UP, 2 y per torch, inverting

Unit-local frame: column `(0, 0)`, base level `y0`, `T` torches, port side
`s ∈ {−1, +1}` along z.

| role | cells | material |
|---|---|---|
| torch attachment | `(0, y0 + 2t, 0)`, `t = 0 .. T−1` | solid |
| torch | `(0, y0 + 2t + 1, 0)`, `t = 0 .. T−1` | `redstone_torch[lit=true]` |
| **cap (STRONG)** | `(0, y0 + 2T, 0)` | solid; strongly powered by the last torch |
| entry dust | `(0, y0, s)` | dust — **dead-ends into the base block** |
| entry floor | `(0, y0 − 1, s)` | solid |
| exit dust | `(0, y0 + 2T, s)` | dust — reads the cap's STRONG power on its side face |
| exit floor | `(0, y0 + 2T − 1, s)` | solid |

Ports:

| line | in | enters from | out | leaves toward | delay | out ss |
|---|---|---|---|---|---|---|
| the bit | `(0, y0, s)` | along `s` | `(0, y0 + 2T, s)` | along `s` | `2T` gt | **15** |

Budgets: rise `2T`, delay `2T` gt (**1 gt per y**), **inverting iff `T` is
odd**, output refreshed to 15 — a ladder has no reach limit and never needs a
repeater.

**`LADDER_BUS`** — bit `i` at `x = x0 + i` (**pitch 1**) with
`s_i = −1 if i even else +1`. 8 bits verified over all 256 patterns with zero
leak at every exit *and* every entry (L4). The alternation is not cosmetic: with
same-side ports the pitch-1 entry dusts merge into one T-shaped net that no
longer points into the bases, and every bit reads a constant (L5).

Reserve, for an `n`-bit tower bank: `n × 1` columns for the shafts, plus a z
lane on each side at the entry level and at the exit level — `n × 3` xz cells if
the router claims the whole prism, but only `n × 1` per y of rise.

---

### `GLASS_TOWER` — 2×1, UP only, 1 y per cell, 0 gt

Unit-local frame: columns `(0,0)` and `(1,0)`, first dust at `y0`, `n` cells.

| role | cells | material |
|---|---|---|
| dust | `(k mod 2, y0 + k, 0)`, `k = 0 .. n−1` | dust |
| support | `(k mod 2, y0 + k − 1, 0)` | **transparent — mandatory** |

Every support does two jobs, and that coincidence is the whole form. The
support of dust `k` sits at `(k mod 2, y0 + k − 1, 0)`, and dust `k−2` sits at
`((k−2) mod 2, y0 + k − 2, 0)` — **the same column, one level down**. So the
support of dust `k` *is* the CUT cell of the step `k−2 → k−1`. A conductor
there severs that step, and the tower is dead after one cell (G2); a
non-conductor keeps it alive but also makes every upper dust sit on a
non-conductor, which is the transparent-diode law and is why the tower is
one-way. Only `k = 0, 1` have unconstrained supports.

Ports: in at `(0, y0, 0)` from −x; out at `((n−1) mod 2, y0 + n − 1, 0)`.
Budgets: rise `n − 1`, **0 gt**, out ss `14 − (n − 1)` ⇒ **14 y per source**,
non-inverting, **one-way UP** (G3 — a free diode). Neighbour towers at
**z-pitch ≥ 2** (G4).

---

### `RING_RISER` — the spiral staircase; UP **and** DOWN; multi-bit

Footprint `sx × sz` with `sx, sz ≥ 3`, origin `(ox, oz)`, first level `y0`,
`n` cells of rise. `ring(sx, sz)` is the clockwise perimeter starting at
`(0,0)`; `m = 2(sx + sz) − 4`; bit offsets `P = ring_bits(m)`, i.e.
`floor(m/3)` values spread evenly around `m`.

For bit `p ∈ P` and `k = 0 .. n−1`, with `c = ring[(k + p) mod m]`:

| role | cells | material |
|---|---|---|
| dust | `(ox + c.x, y0 + k, oz + c.z)` | dust |
| support | `(ox + c.x, y0 + k − 1, oz + c.z)` | **solid** |
| **must stay AIR** | `(ox + c.x, y0 + k + 1, oz + c.z)` | air — this is the CUT cell, and it is why the fold needs a loop |
| port stub | outward from `ring_outward(cell)` at both ends | dust on solid |

Ports: bit `p` enters at `ring[p]`, `y0`, and leaves at `ring[(n−1+p) mod m]`,
`y0 + n − 1`. **Every bit enters on level `y0` and leaves on level
`y0 + n − 1`**, each in its own column — so the form is a bus, not a wire.

Budgets: rise `n − 1`, **0 gt**, **−1 ss per y** ⇒ 14 y per source;
non-inverting; **bidirectional** (drive either end). A repeater standing on any
flat cell of the ring refreshes it to 15 for an unbounded climb or descent, at
the cost of **one cell of rise and 2 gt** (S4: ss 5 → 15).

Legality (the predicate a router should call, `probe_spiral_tiling.legal`):

```
sx >= 3 and sz >= 3                       # the perimeter must be chordless
min(|a-b|, m-|a-b|) >= 3  for all a,b in P
```

Neighbour rings: **x-pitch `sx + 1`** — one empty column. Flush rings always
leak, at every relative rotation. At pitch `sx + 1`, T7 measures **4 of 4**
rotations clean for 2×2, **4 of 8** for 3×3 and **8 of 12** for 5×3 — so a
tiling router must either pick a clean rotation or, for the bigger rings,
verify the seam rather than assume it.

---

### `HALF_SLOPE` — horizontal level change that stacks at 2 y

Line at level `y0`, cells `i = 0 .. n−1`, lane `z`, ascending in +x:

| role | cells | material |
|---|---|---|
| dust | `(x0 + i, y0 + i//2, z)` | dust |
| support | `(x0 + i, y0 + i//2 − 1, z)` | **transparent if `lower_end[i]` else solid** |

where `lower_end[i]` is true iff `y[i±1] == y[i] + 1` — on an ascending half
slope, exactly the odd `i`. Stacked lines repeat the whole pattern at
`y0 + 2j`, using the **same** material pattern, because each line's support is
the cut cell of the line beneath it.

Budgets: 1 y per **2** x cells, 0 gt, **−1 ss per cell = −2 ss per y**,
bidirectional, **stacks at 2 y pitch** (H4, 4/4). A 1:1 slope cannot stack at
all (H5).

---

## RANKED RECOMMENDATIONS

### Dense vertical UP

| rank | form | blocks/y/bit | xz-claim/bit | delay | polarity | reach | why it ranks here |
|---|---|---|---|---|---|---|---|
| **1** | **`LADDER_BUS`, pitch 1, alternating ports** | **1.000** | **1.0 per y** (3.0 incl. both port lanes) | 1 gt/y | **inverts per torch** | **unlimited** (refresh 15) | **2× fewer blocks per y and 3.75× less reserved area than the best dust form, and no repeaters ever.** 256/256 patterns clean |
| 2 | `RING_RISER` 5×3, 4 bits | 2.000 | 3.75 | **0 gt** | non-inverting | 14 y per repeater | the pick when polarity must be preserved or latency must be zero; also the only one of these that can be reversed later |
| 3 | `RING_RISER` 11×3, 8 bits | 2.000 | 4.125 | 0 gt | non-inverting | 14 y | one structure for a whole byte, all 8 bits entering and leaving on one level |
| 4 | `GLASS_TOWER` | 2.000 | 4.00 | 0 gt | non-inverting | 14 y | 2 cells wide is the narrowest *dust* riser, but z-pitch 2 throws away the lane between towers; worth it for a single bit, or where the diode is wanted |
| 5 | `HALF_SLOPE` / `stair_1to1` | 2.000 | 1.0 per line + horizontal travel | 0 gt | non-inverting | 14 y (7 y for the half slope) | only when the route has to cover that horizontal distance anyway |

**Recommendation: `LADDER_BUS` at pitch 1.** The two things a caller must
handle are (a) the inversion parity — pick an even torch count, or fold the
inversion into the logic the way `anneal_genlib.py` already folds
double-inversions — and (b) the port lanes: the towers pitch at 1, but the
entry and exit dusts must alternate z sides, and each port lane is a normal
dust lane subject to lane-pitch 2. If the caller cannot alternate sides, the
form degrades to pitch 2 (L6) and 2.0 xz/bit — still the densest riser here.

### Vertical DOWN

| rank | form | blocks/y/bit | xz-claim/bit | delay | reach | why it ranks here |
|---|---|---|---|---|---|---|
| **1** | **`RING_RISER` 5×3, 4 bits, offsets `[0,3,6,9]`** | **2.000** | **3.75** | **0 gt** | 14 y, then 1 cell + 2 gt per refresh | densest descending form measured; 16/16 patterns, and 16/16 driven from the top |
| 2 | `RING_RISER` 11×3, 8 bits | 2.000 | 4.125 | 0 gt | 14 y | a byte in one shaft; 256 patterns |
| 3 | `RING_RISER` 4×3, 3 bits | 2.000 | 4.00 | 0 gt | 14 y | best fit when the shaft may only be 4 wide |
| 4 | `RING_RISER` 3×3, 2 bits at **180°** | 2.000 | 4.50 | 0 gt | 14 y | the minimum *multi-bit* descender; this is the user's tiling claim in its original form |
| 5 | `RING_RISER` 2×2, **1 bit** | 2.000 | 4.00 | 0 gt | 14 y | the absolute minimum footprint that descends at all — but it holds one bit and needs x-pitch 3 to a neighbour (T7), so a *bus* of them costs 6.0 xz/bit |
| 6 | `repeater_drop` | ~3 (est.) | ~2 per y (est.) | **2 gt per y** | **unlimited** | the only unbounded descent; use for very deep drops or where 15 must be re-established every level. **Only the single stage is probed (D3)** — the per-y formula is an estimate |
| 7 | `stair_1to1` | 2.000 | 1.0 + horizontal travel | 0 gt | 14 y | combined horizontal + vertical moves |

**Recommendation: `RING_RISER`, sized to the bit count** — 5×3 for a nibble,
11×3 for a byte, 3×3 for two bits, 2×2 for one. Insert a repeater on a flat
ring cell every ≤ 14 y. There is **no torch option going down at all** (D1), so
a router must not plan a descent as "a climb, reversed".

---

## TRAPS (each one cost a probe iteration)

1. **An ss-starved rig reports "no crosstalk" for a shorted circuit.** The
   first tiling sweep drove each bit through a bare lever and read 12 ring
   cells later, so the exit sat at **ss 1** — and a leak arriving one cell
   longer than the intended path read **0**. Two-wide rings scored *clean* at
   offset 3 while they were in fact one merged net. Fixed by driving each bit
   through a **repeater** (the riser starts at a full 15) *and* by treating a
   leak **anywhere in an undriven bit's cells** as a failure, not just at its
   exit port. Both probes now do this.
2. **A form can fail by going DEAD rather than by leaking.** Ring offset 2
   severs one bit while its exit stays quiet in every pattern (T3), and
   same-side pitch-1 ladders read a constant with zero leakage (L5). A probe
   that only looks for crosstalk passes both. Always assert **conduction** and
   **isolation** together.
3. **The port lane, not the carrier, usually sets the pitch.** Torch towers are
   electrically happy at pitch 1; it is their entry dusts that are not — and
   the failure is the POINTING LAW, not a short.
4. **Transparency is directional.** A transparent support keeps a step alive
   *uphill* and blocks it *downhill*. So "use glass to keep the diagonal" turns
   any bidirectional form into a diode. Never place transparent supports on a
   path that must carry a signal down.
5. **The cell above a dust and the cell below the next dust are the same cell
   in a 2-wide fold.** That single coincidence is the whole reason DOWN needs a
   ≥ 3×3 spiral while UP fits in 2×1.

---

## CARRIED OUT OF THIS WORK (open, deliberately not built)

* **Ladder port adapters.** The pitch-1 bus needs a fan-in on two alternating z
  lanes at the entry level and a fan-out at the exit level. `vforms.ladder_bus`
  emits the stubs; the *routing* of eight lanes into eight alternating stubs is
  not templated.
* **The ring's repeater station is measured (S4), not templated.** Its cell
  formula — which flat ring cell, and how the path resumes one level late — is
  the missing piece for descents deeper than 14 y.
* **Crossing a riser.** Nothing here says how a horizontal bus passes a shaft.
  The `crosswire_tiles.md` families are all planar.
* **`legal(sx, sz, offsets)` should move into `materials.py`** (and then into
  Rust) beside `step_reads`; it is 6 lines and T8 shows it is exact.
* **2-D ladder arrays are port-limited, not electrically limited.** A pitch-1
  array in both x and z would be 1 block/y/bit, but its interior columns have
  no port access. Whether a staggered-height port scheme reaches them is
  unprobed.
* **`repeater_drop` is only proven as a single stage** (D3). Its per-y cell
  formula and its neighbour pitch are unmeasured.
