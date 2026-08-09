# Wire-crossing tiles — template cells for the Rust router port

Three verified 90° crossing families, from two user-supplied ground-truth
schematics (`crosswire/CROSSWIRE002_classic_crosswire.schem`,
`crosswire/CROSSWIRE001_instant_crosswire.schem`; copies — the originals in
`~/Downloads` are never touched). Port these exactly like `bus_cross8` v2 and
the `pivot_*` tiles were ported: every cell below is a formula in the tile's
level index, coordinates are tile-local.

Verification, 2026-08-09:

* `crosswire/verify_crosswire.py` — **881 output checks, zero crosstalk.** Every
  line is driven through a real lever with a 2-dust input stub and probed 2 dust
  cells PAST its output port, over the full 2ⁿ lever matrix where n ≤ 8 and a
  sampled matrix (all-off, walking-ones, all-on, 0xAA, 0x55) above that. A leak
  has to cross the tile *and* two fresh dust cells to score.
* `crosswire/test_crosswire_templates.py` — the formulas below describe those
  files EXACTLY, plus the physical invariants each family leans on.
* Physics: `probe_transport.py` (25/25) and `TRANSPORT_MODEL.md`.

Physics the templates lean on (all probed — see `TRANSPORT_MODEL.md` for the
mechanism rows and the probe per row):

* **strong power does not chain block→block** (`probe_transport` S1/S3): a
  hard-powered block's solid NEIGHBOUR stays dead and may carry a foreign
  dust line;
* **a repeater reads its BACK only** (S4): no quasi-connectivity, so a repeater
  may stand on a hard-powered block;
* **dust never powers the block above it** (W3): a solid lid over a live
  straight run carries an independent line;
* **CUT cell ≠ DIODE cell** (`probe_transport` group C, 8/8): a conductor
  directly above the LOWER dust of a step severs it both ways; the UPPER dust's
  support only gates the downhill read. `materials.step_reads()`;
* **cap law** (`materials.cap_is_harmful`, bus8 P1): a cap over dust matters
  only if that dust is the lower end of an in-use diagonal;
* **lane pitch ≥ 2** on one level (`probe_transport` P1/P2) and **no planar
  diagonal** (P3).

Any solid block works wherever a material is "solid" (the schematics' red/blue
wool is the author's colour coding). **No transparent blocks in any of these
three families** — see the warning under `xw_hop`.

---

## `xw_buffered` — the classic crossing (repeater + hard-powered block)

Two orthogonal lines at 1-y pitch. The Z-line's repeater strong-powers the
crossing cell; the X-line's repeater stands on that same cell. Tiles in y at
**pitch 2**, one crossing per unit. Both signals come out **refreshed to 15**.

Unit-local frame: `y` = the Z-line's level, X-line at `y+1`; the `y-1` layer is
the previous unit's cap layer and doubles as this unit's Z supports.
Footprint **5 × 2 × 5** (x = 0..4, z = 0..4), 18 blocks per unit + a 4-block
support layer at the bottom of the stack.

| role | cells | material |
|---|---|---|
| Z in port | `(2, y, 4)` | dust |
| Z repeater | `(2, y, 3)` | `repeater[facing=south,delay=1]` — flows −Z |
| **Z crossing cell (STRONG)** | `(2, y, 2)` | solid; strongly powered by the Z repeater. **No support beneath it — `(2, y-1, 2)` is air.** |
| Z out dust | `(2, y, 1)`, `(2, y, 0)` | dust — `(2,y,1)` reads the strong cell at 15 |
| Z supports | `(2, y-1, z)`, z ∈ {0,1,3,4} | solid |
| X support row | `(x, y, 2)`, x = 0..4 | solid; **x = 2 IS the Z-line's strong cell**, x ∈ {1,3} are its dead neighbours |
| X in port | `(4, y+1, 2)`, `(3, y+1, 2)` | dust |
| X repeater | `(2, y+1, 2)` | `repeater[facing=east,delay=1]` — flows −X; **its floor is the strong cell** |
| X out dust | `(1, y+1, 2)`, `(0, y+1, 2)` | dust |
| Z caps / next unit's Z supports | `(2, y+1, z)`, z ∈ {0,1,3,4} | solid |

Ports (in → out):

| line | in | enters from | out | leaves toward | delay | out ss |
|---|---|---|---|---|---|---|
| Z | `(2, y, 4)` | +Z | `(2, y, 0)` | −Z | 2 gt | 14 (refreshed) |
| X | `(4, y+1, 2)` | +X | `(0, y+1, 2)` | −X | 2 gt | 14 (refreshed) |

Budgets: **no signal-strength cost** — each line is refreshed to 15 inside the
tile, so a crossing-heavy route pays ticks, not reach. `delay=2..4` raises the
delay to 4/6/8 gt and changes nothing else. Verified as a 5-high stack
(`classic_stack`, 140 checks) as well as a single unit.

Why it is isolated, cell by cell: `(2,y,2)` is a `strong_block` (mechanism row
3) so it energises only its own cell and its six faces; `(1,y,2)`/`(3,y,2)` are
solid neighbours, and strong power never chains block→block, so the X-line's
dust standing on them is dark; `(2,y+1,2)` is a repeater whose only input is its
back at `(3,y+1,2)`, so its floor is electrically invisible to it.

---

## `xw_hop` — the 0-tick staggered crossing (dust hops the line below)

Lines at **1-y pitch with alternating axes**; each line hops one block up over
the line directly beneath it. Pure dust: **0 ticks**, cost is +1 signal strength
per hop. Region A of `CROSSWIRE001`, tiling period 4 in y.

### The crossing primitive (this is what the router wants)

Lower line **L** along X at level `y`, lane `z = zL`. Upper line **U** along Z
at level `y+1`, lane `x = xU`.

| role | cells | material | why |
|---|---|---|---|
| L runs straight through | `(xU, y, zL)` | dust | the crossing cell is ordinary dust |
| **hop support** | `(xU, y+1, zL)` | **solid — mandatory** | three jobs at once: U's hop support; a lid over L (harmless — W3); the **CUT cell** of both would-be L↔U diagonals |
| hop top | `(xU, y+2, zL)` | dust | UPPER end of both its steps ⇒ its diode cell is the solid support, so it conducts downhill |
| U legs | `(xU, y+1, zL±1)` | dust + solid supports at `(xU, y, zL±1)` | |
| **must stay AIR** | `(xU, y+2, zL±1)` | air | the hop's own two CUT cells |

⚠ **The hop support must be a CONDUCTOR.** Making it glass (the
"transparent blocks" family) carries the hop but does not cut the diagonals, and
the two nets short — asserted as a negative control in
`test_transport.py::test_instant_crossing_is_legal_in_the_model`. Transparency
is for keeping a diagonal *alive*, never for isolating two nets.

### The shipped lattice (region A, x = 0..6, z = 0..8, period 4 in y)

`r = (y − 1) mod 4`; the line whose PORT level is `y` hops the line at `y−1`.
Every dust cell has a solid support directly beneath it.

| r | axis | lane | hop at | dust cells |
|---|---|---|---|---|
| 0 | X | z = 3 | x = 5 | `(0,y,4) (1,y,4) (1,y,3) (2,y,3) (3,y,3) (4,y,3) (5,y+1,3) (6,y,3)` |
| 1 | Z | x = 3 | z = 3 | `(4,y,0) (4,y,1) (3,y,1) (3,y,2) (3,y+1,3) (3,y,4) (3,y,5) (3,y,6) (3,y,7) (4,y,7) (4,y,8)` |
| 2 | X | z = 5 | x = 3 | `(0,y,4) (1,y,4) (1,y,5) (2,y,5) (3,y+1,5) (4,y,5) (5,y,5) (6,y,5)` |
| 3 | Z | x = 5 | z = 5 | `(4,y,0) (4,y,1) (5,y,1) (5,y,2) (5,y,3) (5,y,4) (5,y+1,5) (5,y,6) (4,y,7) (5,y,7) (4,y,8)` |

Ports:

| r | in | out | span | dust used | ss cost | delay |
|---|---|---|---|---|---|---|
| 0, 2 (X) | `(0, y, 4)` | `(6, y, 3)` / `(6, y, 5)` | 7 | 8 | **+1** | 0 gt |
| 1, 3 (Z) | `(4, y, 0)` | `(4, y, 8)` | 9 | 11 | **+2** (hop + lane jog) | 0 gt |

Footprint per 4-level unit: **7 × 4 × 9**, carrying 2 X-lines + 2 Z-lines ⇒
**1 y-level per line**. X-lines land on `y ≡ 1, 3 (mod 4)` and Z-lines on
`y ≡ 0, 2 (mod 4)`, i.e. the two axes live on OPPOSITE y-parities — the tile's
one real cost, since a bus arriving at level `y` can only be an X or a Z.

Measured (`verify_crosswire.py`, instant_A, 16 lines, 320 checks): X out-port
ss 6, Z out-port ss 3 after a 2-dust input stub, 0 gt on every line.

---

## `xw_updown` — the 0-tick coplanar crossing (one dips, one bumps, cell empty)

Both lines enter and leave at the **same y**. One dips one level down with a ±1
lane jog, the other bumps one level up with a ±1 lane jog, and **the
intersection cell is left AIR**. Region B of `CROSSWIRE001`, period 4 in y.
This is the bus-friendly member of the family: **both axes' ports sit on the
same levels.**

Tile-local x = 0..6, z = 10..16, port level `y` (= 3 + 4k or 5 + 4k). On
`y = 3 + 4k` the X-line dips and the Z-line bumps; on `y = 5 + 4k` they swap.
Every dust cell has a solid support directly beneath it.

**`y = 3 + 4k`**

| role | cells | material |
|---|---|---|
| X in / out legs | `(0,y,13) (1,y,13)` … `(5,y,13) (6,y,13)` | dust |
| **X dip** (−z jog, y−1) | `(x, y-1, 12)`, x = 1..5 | dust |
| Z in / out legs | `(3,y,10) (3,y,11) (4,y,11)` … `(4,y,15) (3,y,15) (3,y,16)` | dust |
| **Z bump** (+x jog, y+1) | `(4, y+1, z)`, z ∈ {12,13,14} | dust |
| bump supports = X-dip lids = CUT cells | `(4, y, z)`, z ∈ {12,13,14} | solid |
| **intersection** | `(3, y, 13)` | **AIR** |

**`y = 5 + 4k`** (the mirror; the dip/bump roles swap axes)

| role | cells | material |
|---|---|---|
| Z in / out legs | `(3,y,10) (3,y,11)` … `(3,y,15) (3,y,16)` | dust |
| **Z dip** (−x jog, y−1) | `(2, y-1, z)`, z = 11..15 | dust |
| X in / out legs | `(0,y,13) (1,y,13) (1,y,14)` … `(5,y,14) (5,y,13) (6,y,13)` | dust |
| **X bump** (+z jog, y+1) | `(x, y+1, 14)`, x ∈ {2,3,4} | dust |
| bump supports = Z-dip lids = CUT cells | `(x, y, 14)`, x ∈ {2,3,4} | solid |
| **intersection** | `(3, y, 13)` | **AIR** |

Ports:

| line | in | out | span | dust used | ss cost | delay |
|---|---|---|---|---|---|---|
| X | `(0, y, 13)` | `(6, y, 13)` | 7 | 9 | **+2** | 0 gt |
| Z | `(3, y, 10)` | `(3, y, 16)` | 7 | 9 | **+2** | 0 gt |

Footprint per 4-level unit: **7 × 4 × 7**, carrying 2 X-lines + 2 Z-lines with
ports on 2 levels ⇒ still **1 y-level per line**, and each axis is on a 2-y
pitch that the *other* axis shares. Consecutive port levels share their
intermediate level (`y+1` of one pair is `y−1` of the next), so the envelope of
a crossing pair is 3 levels but the tiling pitch is 2.

Measured (`verify_crosswire.py`, instant_B, 15 lines, 285 checks): out-port ss 5
on both axes after a 2-dust input stub, 0 gt on every line. (The 16th line is
the file's truncated top unit; the verifier and the template test both report
and skip it rather than fabricate a port for it.)

---

## Choosing between them

| want | pick |
|---|---|
| zero delay, both buses on the same levels | **`xw_updown`** — the default for bus crossings |
| zero delay, buses already interleaved on opposite y-parities | `xw_hop` — 1 y-level per line, +1 ss for the straight axis |
| the signal must come out at 15 (long route on the far side) | **`xw_buffered`** — it is the only one that refreshes |
| smallest plan-view footprint | `xw_buffered` — 5 × 5 vs 7 × 9 / 7 × 7 |
| longest reach without a station | `xw_buffered` (0 ss) ≫ `xw_hop` X (+1) > `xw_updown` / `xw_hop` Z (+2) |
| fewest ticks on a latency-critical bus | `xw_updown` / `xw_hop` (0 gt) vs 2 gt per crossing |

The existing `crossing_parity` and `crossing_dipunder` tiles
(`materials.py` §4) remain the *within-one-bus-form* crossings; these three are
the *between-two-lines* primitives, and `xw_hop`'s crossing primitive is the one
worth promoting to a router MOVE rather than a stamped tile
(`TRANSPORT_MODEL.md`, ranked unlocks).
