# TRANSPORT MODEL — how a redstone signal MOVES

Status: probed 2026-08-09. Every row cites the probe that fixed it; the model
lives in `materials.py` section 2b and is unit-tested against these verdicts by
`test_transport.py` (14/14). The physics comes from `probe_transport.py`
(25/25) plus the older probes, all in mc-tick, and from the two ground-truth
crossings in `crosswire/` (`verify_crosswire.py`, 881 checks, zero crosstalk).

## Why this file exists

The material model (`notes-material-model.md`) answered *what may sit where*.
It is **dust-centric**: a route is dust + supports + connect/cut/diode rules,
and everything else (repeater stations, crossings) is a pre-verified TILE the
router stamps rather than a mechanism it can reason about.

Redstone has several *distinct* transport mechanisms, and the good hand-built
solutions are good precisely because they switch mechanism at the tight spot:

* a **hard-powered block** carries the signal through a cell that needs no
  support and shares no power with the block beside it — so two of them may be
  neighbours, and a repeater may stand on one;
* a **weakly powered block** carries a signal that *dust cannot see at all* —
  so a live weak block may sit under a foreign dust line;
* a **transparent support** carries dust without conducting, which is a one-way
  diode, and which is a *different cell* from the block that cuts a diagonal;
* a **torch** is the only inverter and the only compact vertical carrier.

None of those were in the model as mechanisms. This file formalises them.

## Vocabulary — three block properties, four emission kinds

Three *independent* block properties (the first two were already split in
`materials.py`; the third was hiding inside the second):

| property | vanilla | means | glass | top slab | bottom slab | stone |
|---|---|---|---|---|---|---|
| `sturdy(b)` | `isFaceSturdy(UP)` | dust/repeater/torch may sit on it | ✔ | ✔ | ✘ | ✔ |
| `conducts(b)` | `isRedstoneConductor` | may become a *powered block* | ✘ | ✘ | ✘ | ✔ |
| (collision full cube) | `isCollisionShapeFullBlock` | cosmetic: wire renders `up` vs `side` | ✔ | ✘ | ✘ | ✔ |

Four emission kinds — *what* is being handed over, which decides *who can hear
it*:

| kind | source | dust hears it? | a device's input cell hears it? |
|---|---|---|---|
| `SOURCE` | lever, torch, redstone block, button… | **yes**, at 15 | yes |
| `STRONG` | a conductor facing a repeater/comparator output, or under a torch, or holding a lever | **yes**, at 15, on all six faces | yes |
| `WEAK` | a conductor a dust POINTS INTO, or the block under a dust | **NO — never** | yes |
| `WIRE` | dust | via the wire-connection rules only | yes |

`materials.READER_KINDS` is that table. The single most load-bearing line in
the model is **dust does not read WEAK** (`probe_transport.py` W1/W2).

## THE MECHANISM TABLE

11 rows (`materials.mechanism_table()`). Offsets are in a canonical frame whose
forward is `+X`; `materials._xform` rotates them.

### A. carrier / requirement / emission / strength / delay

| # | mechanism | carrier block | requires | emits (kind @ offset) | signal strength | delay |
|---|---|---|---|---|---|---|
| 1 | `dust` | `redstone_wire` | `sturdy` at `(0,-1,0)` | `WIRE`@self; `WEAK`@`(0,-1,0)`; `WEAK`@every *pointed-at* cell | **decays 1 per cell**, 15 max, 15 cells | 0 |
| 2 | `dust_step` | `redstone_wire` | `sturdy` at `(0,-1,0)`; see the step law | as `dust` | decays 1 per cell (a 1-y step costs the same as a flat cell) | 0 |
| 3 | `strong_block` | any conductor | **nothing** — it is not a placed carrier, it is a conductor an output face is pointed at | `STRONG`@self **and** @all 6 neighbours | re-emits **15 exactly**, on every face | 0 (inherits its driver's) |
| 4 | `weak_block` | any conductor | a dust pointing into it, or a dust on top of it | `WEAK`@self and @all 6 neighbours | carries "on/off", not a strength dust can read | 0 |
| 5 | `repeater` | `repeater` | `sturdy` floor (STATIC legality only) | `STRONG`@`(1,0,0)` | **refresh to 15** regardless of input; input floor ss ≥ 1 | 2/4/6/8 gt |
| 6 | `comparator` | `comparator` | `sturdy` floor | `STRONG`@`(1,0,0)` | **analog-exact**: preserves the value | 2 gt |
| 7 | `torch_floor` | `redstone_torch` | `sturdy` at its attachment `(0,-1,0)` | `STRONG`@`(0,1,0)`; `SOURCE`@self and @4 horizontals | refresh to 15, **inverted** | 2 gt |
| 8 | `redstone_block` | `redstone_block` | nothing | `SOURCE`@self and @all 6 | unconditional 15 | 0 |
| 9 | `lever` | `lever` | `sturdy` attachment | `STRONG`@its attachment; `SOURCE`@self and @all 6 | 15 | 0 |
| 10 | `solid_support` | any conductor | nothing | nothing on its own | — | — |
| 11 | `transparent_support` | glass / top slab | nothing | nothing | — | — |

### B. what it may share space with, and the legality predicate

| # | mechanism | may share space WITHOUT interference | legality predicate |
|---|---|---|---|
| 1 | `dust` | a solid **lid** directly above it (it never powers the block above — W3); a foreign dust ≥ 2 cells away on the same level (P2); a foreign dust with a conductor between them | `can_occupy("dust", c, grid)`; lane pitch ≥ 2 (P1); no planar diagonal exists (P3) |
| 2 | `dust_step` | a solid **cap over the UPPER dust** (the upper end of a step tolerates any cap) | `step_reads(cut_cell, diode_cell, downhill)` — see the split below |
| 3 | `strong_block` | **another strong block, immediately adjacent** (strong power never chains block→block — S1/S3); a **repeater standing on it** (S4); a solid block beside it that carries a foreign dust (S1) | `interferes()`: it energises only its own cell and its 6 faces |
| 4 | `weak_block` | **any foreign dust, on any face including on top** (W1); another weak block | it reaches device input cells only |
| 5 | `repeater` | a live rail beside it, a hard-powered block **under** it, a lid above it — it reads its BACK only (S4, `probe_station` S) | back must be entered IN LINE (the POINTING LAW, `probe_pivot` B) |
| 6 | `comparator` | same as the repeater on the floor/top; **not** the sides (it reads both) | `dust_drives_block` for the back; sides are inputs |
| 7 | `torch_floor` | its own attachment block stays unpowered by it (T2), so the attachment may carry another line's weak power | never place where its attachment block is a foreign net's weak/strong block |
| 8 | `redstone_block` | a solid block beside it stays dead, so that block may carry foreign dust (B2) | none — it is always on |
| 9 | `lever` | — | attachment becomes a strong block: treat as row 3 |
| 10 | `solid_support` | it may simultaneously be (a) a dust support, (b) a separator between stacked runs, (c) a lid over a live run, and (d) the CUT cell that severs a foreign diagonal — **all four at once**; that quadruple duty is what the crossings exploit | `sturdy ∧ conducts` |
| 11 | `transparent_support` | it supports dust and does **not** cut a diagonal, but it also does not carry weak power | `sturdy ∧ ¬conducts`; use ONLY where a diagonal must survive |

### Probe citations, per row

| # | probe |
|---|---|
| 1 | `probe_materials.py` (SIT/WEAK); `probe_transport.py` W1/W3; `probe_pivot.py` A/B/C (pointing law) |
| 2 | `probe_transport.py` group C (8/8 matrix); `probe_pivot.py` E (14-step descent); `materials._verify_slope` 9/9 |
| 3 | `probe_transport.py` S1 (no chaining), S2 (all six faces at 15), S3 (two adjacent strong blocks are independent) |
| 4 | `probe_transport.py` W1 (no dust anywhere), W2 (a repeater off the same block fires) |
| 5 | `probe_station.py` A/I/B/Z/F/S; `bus8_probe.py` P2 (slab floor still fires); `probe_transport.py` S4 |
| 6 | `probe_station.py` C1–C4 (analog through blocks both ways) |
| 7 | `probe_transport.py` T1/T2/T3 |
| 8 | `probe_transport.py` B1/B2 |
| 9 | `probe_pivot.py` A; every rig in this directory |
| 10 | `probe_materials.py` CUT_UP/CUT_DOWN; `bus8_probe.py` P1 (lids/separators) |
| 11 | `probe_materials.py` DIODE_UP/DIODE_DOWN; `probe_transport.py` group C |

## THE RULES WE HAD CONFLATED

Four separations. Each was one function or one unstated assumption before.

### 1. `cuts_diagonal` was the CUT rule *and* the WEAK-CARRY rule

`conducts(b)` documented itself as "carries weak power into a device back", and
`cuts_diagonal(b)` was defined as `== conducts(b)`. Same boolean, but they are
two different rules applied at two different cells, and a third one was inside
them. Now:

```
cuts_step(b)       CUT RULE      cell: directly ABOVE THE LOWER dust of a step
gates_downhill(b)  DIODE RULE    cell: the UPPER dust's SUPPORT
carries_weak(b)    WEAK-CARRY    cell: the block a dust POINTS INTO
```

All three still read `isRedstoneConductor`, so they are the same *predicate* —
that is exactly why conflating them was invisible. They are not the same
*rule*: they fire at different cells and have different effects, and a planner
that has one function cannot say "put a conductor HERE to sever a foreign
diagonal while a transparent block THERE keeps mine alive".

### 2. the CUT cell and the DIODE cell are different cells

`step_conducts(upper_support, downhill)` took only the support, silently
assuming the cut cell was clear. The truth (vanilla
`RedStoneWireBlock.calculateTargetStrength`, probed as a 2×2×2 matrix):

for a wire `self` and a horizontal neighbour `np`:

```
read the wire at np+UP    (self is the LOWER dust)   iff np is a CONDUCTOR
                                                     AND above(self) is NOT
read the wire at np+DOWN  (self is the UPPER dust)   iff np is NOT a conductor
```

so

| | cut cell = above the LOWER dust | diode cell = the UPPER dust's support |
|---|---|---|
| **uphill** | must be a non-conductor | *irrelevant* |
| **downhill** | must be a non-conductor | **must be a conductor** |

Probed (`probe_transport.py` C, ss in brackets):

| cut | diode | uphill | downhill |
|---|---|---|---|
| air | solid | conducts [14] | conducts [13] |
| air | glass | conducts [14] | **blocked** [0] |
| solid | solid | **blocked** | **blocked** |
| solid | glass | **blocked** | **blocked** |

New API: `step_reads(cut_block, upper_support, downhill)`. `step_conducts` is
kept for the existing patterns, which all place the cut cell themselves.

### 3. "a powered block" was one notion; it is two

There was no notion of a powered block at all — only `conducts` = "carries weak
power into a device back". But **strong** and **weak** are different states with
different audiences (`probe_transport.py` W1 vs S2): a strongly powered block
lights dust on all six faces at 15, a weakly powered block lights *no dust
anywhere* while still firing a repeater. Both are now mechanism rows with
different `emits` kinds and one shared `interferes()` rule.

### 4. dust's emission had three cases, modelled as one bool

`dust_drives_block(entry_on_pointing_axis)` was a single boolean. Dust actually
emits to three different places with three different rules:

* the block **below** it: always weak;
* the blocks it **points into** (its connection axes): weak — the pointing law;
* the block **above** it: **never** (`probe_transport.py` W3).

The third is what lets a solid lid sit on a live dust run and carry an
independent line, which is the whole `instant` crossing. `dust_emission(cell,
pointing)` now returns all three.

## THE TWO GROUND-TRUTH CROSSINGS, IN PRIMITIVE RULES

Full cell listings, ports, delays and footprints: `crosswire_tiles.md`.
Verification: `crosswire/verify_crosswire.py` (881 checks, 0 crosstalk).

### `CROSSWIRE002_classic` — the buffered crossing

The Z-line's repeater strong-powers the crossing cell; the X-line's repeater
stands on top of that same cell. In primitive rules:

1. `(2,y,2)` is a **strong_block** (row 3): the Z-repeater at `(2,y,3)` faces
   it. Row 3 says it emits `STRONG` at its own cell and its 6 faces — so the
   Z-line's readout dust at `(2,y,1)` reads 15 (refresh), and *nothing else*.
2. `(1,y,2)` and `(3,y,2)` are solid blocks **immediately adjacent** to it and
   they carry the X-line's dust. Row 3 also says strong power does not chain
   block→block (S1), so those two supports stay dead (S3 is the same fact with
   two drivers). **This is the "two hard-powered blocks can be neighbours"
   insight, in the form the tile actually uses it.**
3. `(2,y+1,2)` is a **repeater** whose floor is the strong block. Row 5: a
   repeater reads its back only (S4) — no quasi-connectivity — so the X-line
   is electrically blind to the cell it stands on.
4. The Z-line's dust at `(2,y,0..1)` is a straight run under a solid cap
   `(2,y+1,0..1)`; no diagonal is in use, so the cap is harmless (cap law) and
   doubles as the *next* stacked unit's support.

Cost: 1 redstone tick (2 gt) per line, both signals refreshed to 15, 2 y-levels
per crossing pair.

### `CROSSWIRE001_instant` — the pure-dust crossings

Two variants ship in one schematic (region A `z=0..8`, region B `z=10..16`);
the two glass cells at the bbox corners are markers, not circuitry.

**Region A, `xw_hop`** — lines sit at 1-y pitch with alternating axes; each
line **hops one block up over the line directly beneath it**. At the crossing
of the lower line L (along X at `y`, lane `z=zL`) and the upper line U (along Z
at `y+1`, lane `x=xU`):

1. L runs straight through `(xU, y, zL)` — a plain dust cell.
2. `(xU, y+1, zL)` is a **solid_support** (row 10) doing three jobs at once:
   it is U's bump support, it is a lid over L's dust — harmless, because L is a
   straight run there and dust never powers the block above it (W3) — and it is
   the **CUT cell** of the two would-be diagonals between L's dust and U's two
   legs at `(xU, y+1, zL±1)`. Being a conductor, it severs both.
3. `(xU, y+2, zL)` is U's bump-top dust. It is the UPPER end of both of its
   own steps, so its diode cell is the solid support (downhill conducts) and
   its cut cells are `(xU, y+2, zL±1)`, which must stay **air**.
4. Make that support glass instead and the crossing shorts (`test_transport.py`
   `test_instant_crossing_is_legal_in_the_model` asserts both negative
   controls). **The support has to be a conductor — the "transparent blocks"
   family is the wrong tool here; transparency is for keeping a diagonal
   alive, not for isolating two nets.**

Cost: **0 ticks**, +1 signal strength per hop, 1 y-level per line.

**Region B, `xw_updown`** — both lines enter and leave at the **same y**; one
dips one level down with a ±1 lane jog, the other bumps one level up with a ±1
lane jog, and **the intersection cell itself is left AIR**. Same three rules as
above, applied symmetrically: the bumping line's supports sit directly over the
dipping line's dust, where they are simultaneously lid and cut cell.

Cost: **0 ticks**, +2 signal strength per line, 3 y-levels of envelope for a
crossing pair (levels are shared between consecutive pairs, so still 1 y-level
per line) — and, unlike `xw_hop`, **both axes' ports are on the same levels**,
which is what a bus needs.

### Measured

| tile | delay/line | ss cost/line | envelope per tiling unit | lines per unit | crossing cell |
|---|---|---|---|---|---|
| `classic` | 2 gt | **none** (refreshed to 15) | 5 × 2 × 5 | 1 X + 1 Z | strong block + repeater above |
| `xw_hop` | 0 gt | +1 (X), +2 (Z, incl. its lane jog) | 7 × 4 × 9 | 2 X + 2 Z | plain dust; the hop is above it |
| `xw_updown` | 0 gt | +2 | 7 × 4 × 7 | 2 X + 2 Z | **air** |

## WHAT THE ROUTER WOULD NEED (spec — do not implement here)

### Where the fabric stands today

`router.py`'s search state is `(cell, consecutive-stair count, previous stair
direction)`; the Rust `BusFabric` (`src/design_corridor.rs`) is coarser still —
**one node per COLUMN on the bit-0 dust plane**, where a column is legal only
when the *entire* stack `y0-1 ..= y0+2*(width-1)` is free of hard occupancy and
of every foreign halo, plus an electrical-clearance rule ("no column cell
orthogonally adjacent to foreign dust or a foreign repeater") and `MIN_LEG = 3`.

That abstraction is why crossings must be stamped tiles: **a column is claimed
by one bus for its whole height**, so no other net can be *anywhere* in it. Both
ground-truth crossings work by putting two nets in the same column, one cell
apart in y, with one conductor between them. The fabric cannot represent that,
so it cannot find it.

### Mechanisms absent from the move set

| absent mechanism | what it would buy | why it is absent today |
|---|---|---|
| **hard-powered-block relay** (row 3) | a carrier cell that needs no support, refreshes to 15, and may be *adjacent to another net's identical cell* — the only way to pack two signals at 1-cell separation | the fabric models dust cells and repeater bodies; a strongly powered *block* is not a routable object at all, only an artefact inside the station tile |
| **1-y dust hop over a foreign line** (rows 2+10) | 0-tick crossings at 1 y-level per line, no detour, +1 ss | the column model forbids a foreign net in the column; the cut/diode cells were one predicate, so "solid here severs theirs, air there keeps mine" was inexpressible |
| **same-level dip/bump pair** (`xw_updown`) | two buses crossing while both stay on their own levels — removes the level-shift adapter that is listed as the single biggest remaining unlock | needs *both* nets' geometry in one decision; the router routes one net at a time |
| **weak-block corridor** (row 4) | a live conductor may pass *under a foreign dust line* with zero clearance, because dust cannot see weak power | `nets.py` is dust-only; weak power is not claimed or tracked |
| **torch tower** (row 7) | compact vertical transport (2 gt + one inversion per level) where a staircase needs 1 horizontal cell per level | ladders exist in `router.py` but as a fixed gadget, not as a mechanism with its own emission/inversion bookkeeping |
| **transparent diode as a one-way isolator** (row 11) | deliberately one-way segments — a cheap alternative to a diode repeater | glass is currently used only to *keep a diagonal alive*; its diode nature is never used on purpose |

### What the search state must become

The last audit concluded state must grow from *position* to *position + form +
bundle geometry* (bus form: vertical 2y stack, flat 2z lane, HexAnalog; and the
per-bit offsets within the bundle). Extend that reasoning by one axis:

```
state = ( position , form , bundle geometry , MECHANISM )
```

with `MECHANISM` ∈ the 11 rows, because:

1. **Legality is mechanism-dependent, not cell-dependent.** `dust` needs a
   sturdy support; `strong_block` needs nothing; `repeater` needs a floor but is
   immune to it electrically. A single `dust_ok(cell)` predicate cannot answer
   for all three. `materials.can_occupy(mech, cell, grid)` is the shape of the
   replacement.
2. **Clearance is a pair-of-mechanisms question.** "no foreign dust
   orthogonally adjacent" is right for `dust`↔`dust` and *wrong* for four of the
   eleven rows: `strong_block`↔`strong_block` at distance 1 is legal (S3),
   `weak_block`↔`dust` at distance 0 is legal (W1), `strong_block`↔`repeater`
   floor is legal (S4). `materials.interferes(a, b, grid)` replaces the scalar
   halo with the emission∩sensitivity test, and every one of those legal
   adjacencies falls out of it.
3. **Cost is mechanism-dependent.** `dust` costs 1 ss and 0 ticks; `repeater`
   costs 0 ss and 2 gt; a hop costs +1 ss and 0 ticks; a torch step costs 2 gt
   and an inversion (so *parity* joins the state, like the stair counter did).
4. **Transitions are gated.** A station entry must be reached in line (the
   pointing law); a hop's cut cells must be air; a step's direction decides
   which of the two cells is checked. These are edge predicates over
   (mechanism, mechanism, direction), i.e. exactly the shape `move_ok` already
   has — it just needs the mechanism pair as an argument.
5. **The column abstraction has to go, or gain a per-y mask.** The cheapest
   honest change: a column's legality becomes a *set of occupied y-slices plus
   the emission kinds present at each*, not a boolean.

### Ranked unlocks

1. **`interferes()`-based clearance instead of the scalar halo.** Pure win, no
   new geometry: it *permits* the four legal adjacencies above and keeps
   forbidding the illegal ones. Every other item below is blocked on it, and it
   alone should recover congestion failures caused by over-conservative halos.
2. **`xw_updown` as a stampable same-level crossing tile.** Verified hardware
   already in hand; removes the "two buses must occupy disjoint `y_band`s or
   match widths" constraint and directly attacks the level-adapter gap that the
   last audit called the single biggest remaining unlock. Delay-free.
3. **Per-y column masks (drop "a column is claimed for its whole height").**
   The enabling change for every dense trick; also the fix for the "3-high hole"
   pessimism. Cost: the memo and the halo bookkeeping get an extra dimension.
4. **The 1-y hop as a first-class MOVE** (rows 2+10 with the split cut/diode
   predicates). Lets the router cross a foreign line *anywhere* for +1 ss and 0
   ticks, instead of only where a tile was pre-stamped. Requires 1 and 3.
5. **`strong_block` as a routable carrier.** Unlocks 1-cell-separated parallel
   signals and support-free cells (useful over voids and inside cell keepouts).
   Requires 1; needs a new claim kind in `nets.py` (`strong` is already a
   sentinel there — promote it to a real kind).
6. **Torch towers as a mechanism** (vertical transport + parity in the state).
   Biggest win where vertical space is cheap and horizontal is not; the
   inversion bookkeeping is real work, so it ranks below the space-savers.
7. **Transparent diode as an intentional one-way** — small, cheap, mostly
   useful for suppressing back-feed at junctions; last because the same effect
   is already available from a repeater at a known cost.

## NOT MODELLED (recorded so it is not mistaken for coverage)

* **Quasi-connectivity.** Pistons/dispensers/droppers read the block *above*
  themselves. Not probed here, not a row — it is a consumer rule, not a
  carrier, but it is the next mechanism to add if pistons enter the fabric.
* **Observers, target blocks, tripwire, rails, string, trapdoors, fence
  gates.** All have transport-relevant quirks; none probed.
* **Locking** (a repeater/comparator powered from the side latches). Modelled
  as a note on rows 5–6, never probed here.
* **Signal-strength-dependent behaviour** other than dust decay — comparator
  arithmetic is row 6 but container fill (`probe_station` C, `TRICKS.md` T7)
  remains "verify per build".
* **Bottom slabs / air as supports.** mc-tick never pops pre-placed dust, so
  support legality is a *static* check (`audit.py`) that the sim will not catch;
  `can_occupy` is where it belongs.
