# Material-model generalization — user-specified techniques (to probe + implement)

## PROBED — empirical material table (probe_materials.py, mc-tick, 2026-08-08)

| material | dust sits | conducts weak | cuts diagonal | vertical step up | vertical step down |
|---|---|---|---|---|---|
| stone/solid | yes | yes | YES | conducts | conducts |
| glass | yes | no | no | conducts | **BLOCKED** |
| white stained glass | yes | no | no | conducts | **BLOCKED** |
| slab type=top | yes | no | no | conducts | **BLOCKED** |
| slab type=bottom | sim keeps it (vanilla-ILLEGAL) | no | no | conducts | BLOCKED |
| air | sim keeps floating dust (vanilla-ILLEGAL) | no | no | — | — |

Mechanism (mc-tick `vanilla.rs`/`wire.rs`, matches the probes exactly):

- **Support (`sturdy_up`)**: dust sits on full cubes, top-half slabs and
  top-half stairs. Glass IS a full cube, so it supports dust. Bottom slabs
  and air do NOT — but mc-tick never pops pre-placed dust (the sim happily
  ticks a floating wire), so support legality is a STATIC check (audit.py),
  not something the sim will catch.
- **Cut rule runs on CONDUCTIVITY, not solidity**: a block above the lower
  dust of a diagonal cuts the connection iff it `is_conductor`. Glass,
  stained glass and both slab halves are non-conductors → none of them cut,
  in either flow direction (probed CUT_UP and CUT_DOWN).
- **Weak power**: only conductors carry weak power into a repeater's back
  (dust → block → repeater). Glass/slabs conduct nothing.
- **The transparent diode**: a 1-y diagonal step whose UPPER dust sits on a
  non-conductor (glass/slab-top) passes signal UP only. Down-flow requires
  the lower dust to read the wire above its side block, and that read is
  gated on the side block (the upper dust's support) being a conductor.
  ⇒ **every climb that must conduct downhill needs its upper dust on a
  SOLID support**; the transparent support goes under the LOWER dust of the
  step, where it never gates anything (this is exactly the user's
  "transparent at (x,y), next step (+x,+y) on solid" alternation).
- **Pass-under (UNDER probe)**: a dust line runs under a top-slab that
  carries dust on top; dusts are 2 y apart (dust / slab / dust), both lines
  conduct, fully isolated in all 4 lever combos. Same vertical pitch as a
  solid lid, but the slab lid does not cut the under-dust's diagonals, so
  the under line may climb out directly at the crossing exit.
- **EXTRA_STATES**: glass, white_stained_glass and
  smooth_stone_slab[type=top|bottom,waterlogged=false] are interned in
  rs.EXTRA_STATES so late (router/template) placements never sit inert.


Captured 2026-08-08 from the user (domain expert input). ALL claims below must be
probed in mc-tick before use; then they become material-class rules + verified
tile/via templates, not special cases.

## 1. Material classes
The conduction model (nets.py, dust_ok, nucleation-routing rules) is binary
solid-vs-air today. Needed: per-class properties for at least {solid, air,
transparent (glass), slab-top, slab-bottom}: conducts weak/strong? supports
dust? cuts the up-diagonal when above dust? placeable as route support?
Probe each property empirically; build the table once.

## 2. Half-slope / interleaved levels (alternating transparent supports)
Two lines at 1-y separation on a slope CANNOT use all-glass supports — the
levels would mix. The pattern: a dust cell sits on TRANSPARENT at (x, y);
its next step (+x, +y) sits on SOLID. The transparent positions let each
line make its own diagonal climbs; the solid positions block reaching the
OTHER level's dust. Result: 1-y bus pitch on slopes and compact under-passes,
legal by alternation. Probe: glass-above-dust diagonal cut semantics, dust-on-
glass conduction, the exact alternating pattern both ascending and descending.

## 3. 90-degree bus crossings, two verified tile families
a) y-parity interpenetration: buses offset 1 in y (even-y vs odd-y); the
   block-sandwich repeater stations' solid blocks double as the isolation
   between perpendicular lines.
b) dip-under: the crossing bus steps DOWN 1 before the crossing, runs under,
   and steps back UP 1 after — built from the alternating transparent/solid
   slope pattern (technique 2), minimal footprint.
Both become registered crossing TILES (pair-wise templates the bus router
stamps on crossing demand), verified in-sim per bit incl. isolation.

## 4. Generalization principles (user's framing)
- Tricks must generalize: double-inversion elimination -> full logic
  optimization (polarity choice over the whole netlist, sharing, ABC+genlib).
- Bussing must be smart/dynamic, driven by an INFLUENCE MAP: every placed
  element declares the cells it electrically touches/could touch (conduction +
  cut + strong-power side effects), maintained incrementally in the Workspace —
  replaces the manual clearance-halo hack, makes coexistence a map query.
