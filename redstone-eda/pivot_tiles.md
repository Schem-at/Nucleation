# Form-pivot adapter tiles — template cells for the Rust router port

Verified 2026-08-09 by `pivot_tiles.py` (96/96 output checks per tile,
zero crosstalk; baked `.schem`s in `showcase/`).  Port these exactly like
`bus_cross8` v2 was ported: every cell below is a formula in the bit index
`n` (0..7), coordinates are tile-local, `y_v(n) = 1+2n`, lane `z_n = 2n`.

Physics the templates lean on (all probed):

* block-sandwich station: `[entry block][repeater][exit block]`
  (`probe_station.py`: fires from entry ss>=1, exit re-emits 15, 15-dust
  max pitch);
* **POINTING LAW** (`probe_pivot.py`, `materials.dust_drives_block`): dust
  weak-powers only blocks on its connection axes — every station ENTRY
  block is preceded by >= 1 dust cell on the station's axis; a station
  EXIT block (strong) lights dust on any side, so corners directly after
  exits are free;
* diode law (`materials.step_conducts`): solid supports under step-UPPER
  dusts make a 1y/1x staircase conduct BOTH ways (descent probed at
  14 steps, probe E; ascent = h2v's 96/96);
* cap law: straight runs tolerate solid caps -> the fan column's support
  layers double as separators (`materials.separator()`, bus8 P1).

Any solid block works wherever a material is "solid" (colours below are
`rs.PALETTE` cosmetics).  NO transparent blocks in this family.

## pivot_v2h / pivot_h2v — vertical(2y stack) <-> horizontal(2z flat)

One geometry, two tiles: the repeater facings (and entry/exit block roles)
flip.  v2h flows V->H (+X); h2v flows H->V (-X).  Footprint x=0..19,
y=0..16, z=0..14 (20 x 17 x 15 incl. supports); 9 repeaters.

Ports:
* VERTICAL port: bit n dust at `(0, 1+2n, 0)` — dense vertical form
  (`bus8_probe.py`), attach along -z or -x.
* HORIZONTAL port: bit n dust at `(19, 1, 2n)` — flat form, attach +x.
* Port freshness: driver must deliver ss>=15 at the port (a station exit
  or lever adjacent).  Delivered output: worst-case ss1 (v2h H-port, h2v
  V-port bit<=6 ss>=2, bit 7 ss10) — feed a station within its budget.

Per bit n (all supports = the cell directly under a dust cell):

| role | cells | material |
|---|---|---|
| fan dust | `(0, 1+2n, z)` for z=0..2n (minus bit-7 station cells) | dust |
| fan support/separator | `(0, 2n, z)` for z=0..2n | solid (cap law) |
| bit-7 fan station (n=7 only) | `(0,15,6),(0,15,7),(0,15,8)` | v2h: entry block / repeater flows +Z (`facing=north`) / exit block; h2v: exit / repeater flows -Z (`facing=south`) / entry |
| lane approach dust | `(1, 1+2n, 2n)` + support `(1, 2n, 2n)` | dust / solid |
| lane station | `(2, 1+2n, 2n)`, `(3, 1+2n, 2n)`, `(4, 1+2n, 2n)`; repeater floor `(3, 2n, 2n)` | v2h: entry / repeater flows +X (`facing=west`) / exit; h2v: exit / repeater flows -X (`facing=east`) / entry; floor solid |
| staircase+flat dust | `(x, max(1, 1+2n-(x-5)), 2n)` for x=5..19 | dust |
| staircase+flat support | cell under each, `need_conductor=True` where the dust is a step UPPER (max(...) > 1) | solid (diode law) |

Derived invariants the port must keep:
* uniform decay: every bit has exactly 15 dust cells after its lane
  station exit (x-advance is uniform through the staircase);
* fan budget: port->lane-approach is `2n+2` cells; only bit 7 exceeds 15,
  hence its single fan station.  A wider tile (more bits) needs a fan
  station for every bit with `2n+2 > 15`, placed inline in the fan run;
* lanes are 2 apart in z and never interact; the only vertical stacking
  is the fan column, which is exactly the probed bus8 separator stack.

## pivot_flat90 — flat form corner (+X run -> +Z run)

Concentric lanes; order preserved in the travel frame (bit 0 stays the
leftmost lane).  Coordinate map: in-lane `z=2n` -> out-lane `x = 14-2n`
(reading the out ports by increasing x reverses the index — that is the
plane geometry, not a wiring choice; the mirrored/rotated variants are
coordinate transforms of this template).  Footprint x=0..14, y=0..1,
z=0..19 (15 x 2 x 20); 14 repeaters.  `x_c(n) = 14-2n`.

| role | cells | material |
|---|---|---|
| in port | `(0, 1, 2n)` | dust |
| in-leg dust | `(x, 1, 2n)` x=0..x_c (minus S1 cells when present) + supports `(x, 0, 2n)` | dust / solid |
| S1 station (only bits with x_c >= 4, i.e. n <= 5) | `(1,1,2n),(2,1,2n),(3,1,2n)`; floor `(2,0,2n)` | entry / repeater flows +X (`facing=west`) / exit |
| corner | `(x_c, 1, 2n)` — plain dust (in-leg's last cell) | dust |
| post-corner dust | `(x_c, 1, 2n+1)` + support | dust (acquires the Z axis -> points into S2) |
| S2 station (every bit) | `(x_c, 1, 2n+2..2n+4)`; floor `(x_c, 0, 2n+3)` | entry / repeater flows +Z (`facing=north`) / exit |
| out-leg dust | `(x_c, 1, z)` z=2n+5..19 + supports | dust |
| out port | `(x_c, 1, 19)` | dust |

Budgets: worst run is 15 dusts (bit 0's out-leg, arrives ss1 — probed
legal).  Bits 6,7 skip S1 because the corner leaves no room AND their
in-leg is short enough (port -> S2 entry <= 15 cells) — the general rule
for other widths: S1 iff `x_c >= 4`; S2 always.

## Verification (each tile, through real levers)

all-off + walking-ones(8) + all-on + 0xAA + 0x55 = 12 patterns x 8 output
bits = 96 checks, zero crosstalk tolerated; saved baked at rest only on
green.  Regenerate: `~/eda-venv/bin/python pivot_tiles.py`.
Physics probes: `~/eda-venv/bin/python probe_pivot.py` (5/5).
