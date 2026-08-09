# VERTICAL TRANSPORT — how a bus changes level

Status: probed 2026-08-10, mc-tick. Every row below cites the probe that fixed
it. Executable templates and the machine-readable form contract:
**`vforms.py`** (`vforms.FORMS`, `vforms.data_safe()`). Probes:

* **`probe_vertical_forms.py` — 35/35.** The forms, their densities, latencies,
  the best torch-free ascent, why DOWN is asymmetric, and the slope rule.
* **`probe_spiral_tiling.py` — 26/26.** The user's "spiral staircases tile if
  they are offset 180 degrees", generalised to an integer law and checked
  against 17 simulated configurations.
* **`probe_torch_burnout.py` — 16/16.** **Torch burnout — the failure mode
  every other probe in this directory is structurally blind to.** Read the
  methodology section below before trusting any torch-bearing result anywhere
  in this project.
* **`showcase/showcase_vertical.py` — 2/2.** Two verified `.schem` risers.

Physics this builds on, all in `TRANSPORT_MODEL.md`: the mechanism rows
(torch = row 7, dust = 1/2, strong block = 3, weak block = 4, transparent
support = 11), the CUT-cell/DIODE-cell split, the POINTING LAW, the cap law.
Materials: `notes-material-model.md`. Crossings: `crosswire_tiles.md`.

## Why this file exists

`router.py` has one vertical gadget (a torch ladder) with a fixed clearance
rule, and `src/design_corridor.rs` cannot represent a level change at all — its
node is a *column*, claimed by one bus for its whole height. Every question a
level-changing bus actually asks was unanswered: how many bits fit in a shaft,
whether the shaft can go down as well as up, what it costs per y, what its
neighbour pitch is — **and whether it survives real traffic**.

That last one turned out to be the decisive question, and it is the one this
file was originally wrong about. The first draft ranked the torch ladder first
on density. It is disqualified: it adds **1 gt per y to the critical path**, and
its torches **burn out** when the data toggles. The corrected headline:

> **For switching bus data, use the torch-free `ring_riser` in both
> directions.** 0 gt, no toggle limit, 3.75 xz cells per bit, and the same
> geometry climbs and descends. The torch ladder is confined to static and
> low-toggle control signals.

The up/down asymmetry is still real and still shapes the fabric:

* **UP** has an *active* carrier (the torch) that fits a 1×1 column and
  refreshes to 15 — but it is the slow, burnout-prone one, so the asymmetry is
  a trap rather than a gift.
* **DOWN** has no active carrier at all. Every descending form is dust on a
  staircase: 2 blocks per y per bit, −1 ss per y, and a **≥ 3×3 footprint** to
  fold into a shaft.

Since the torch is excluded for data, **up and down converge on the same
form**, which is a simplification for the router: one riser template, two
directions.

---

## THE FORM TABLE

Machine-readable in `vforms.FORMS`. **The first three columns outrank the
density columns.** A carrier that cannot survive the traffic is not a cheap
carrier, it is a broken one — so `data_safe` is a hard precondition, checked by
`vforms.assert_data_safe(form, hold_gt)`, not a cost to trade away.

### A. latency, toggle limit, then rate and reach

| # | form | **gt per y** | **max toggle rate** | **data-safe?** | direction | rate | inverting | signal strength | blocks/y/bit |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `torch_ladder` | **1.0** | **hold ≥ 4 gt per state** | **NO** — disqualifying | UP only | 2 y per torch | yes, per torch | refresh to 15, no reach limit | **1.000** |
| 2 | `glass_tower` | **0.0** | unlimited | yes | UP only (diode) | 1 y per cell | no | −1 per y; 14 y per source | 2.000 |
| 3 | **`ring_riser`** | **0.0** | **unlimited** | **yes** | **UP and DOWN** | 1 y per cell | no | −1 per y; 14 y per source | 2.000 |
| 4 | `half_slope` | 0.0 | unlimited | yes | both | 1 y per **2** x cells | no | −1 per cell = **−2 per y**; 7 y | 2.000 |
| 5 | `stair_1to1` | 0.0 | unlimited | yes | both | 1 y per x cell | no | −1 per y | 2.000 |
| 5b | `stair_1to1`, all-transparent supports | 0.0 | unlimited | yes | UP only (diode) | 1 y per x cell | no | −1 per y | 2.000 |
| 6 | `repeater_drop` | 2.0 | unlimited (**repeaters do not burn out**) | yes | DOWN | 1 y per station | no | refresh to 15 | ~3 (est.) |

### B. footprint, neighbour pitch, support, preconditions

| # | form | footprint per bit | legal neighbour pitch | support requirement | precondition that bites |
|---|---|---|---|---|---|
| 1 | `torch_ladder` | 1×1 column; ports add a z lane each side at the entry and exit levels | **1** with ports on **alternating** sides; 2 with same-side ports | the column is its own support | **BURNOUT is the binding precondition** — see below. Then: two pitch-1 ladders with same-side ports merge their entry dusts into one net, and that net is a T, which does not point into the bases (POINTING LAW), so the array goes **dead**, not leaky (L5) |
| 2 | `glass_tower` | 2×1 | **z-pitch 2** (pitch 1 leaks at dy 0 and 1) | **every support a NON-conductor** (glass, stained glass, top slab) | all-solid supports kill it after one cell — in this form every support *is* the CUT cell of the dust two levels down (G2). Bottom slabs are vanilla-illegal (`audit.py`) |
| 3 | `ring_riser` | 4.50 (3×3, 2 bits) / 4.00 (4×3, 3) / **3.75** (5×3, 4) / 4.125 (11×3, 8) / 4.00 (2×2, 1) | **ring pitch `sx + 1`**: one empty column. Flush rings always leak | **every support SOLID**, and the cell directly **above every dust must be AIR** | footprint **≥ 3 in both axes** or the perimeter has a chord and the bits merge (T1); bit offsets **≥ 3** apart (T2) |
| 4 | `half_slope` | **a SLOPE, not a shaft**: the lane grows with the rise (2 x cells per y) but is **shared by every bit**, which stack at 2 y pitch in the same lane | 2 y between stacked lines; lane pitch 2 sideways | support[i] **transparent iff dust i is the lower end of a step** ⇒ ascending: glass at odd i, solid at even i | all-transparent supports **merge** the stacked lines (H3); all-solid cuts every step |
| 5 | `stair_1to1` | 1×1 per line, **cannot stack** | — | all-solid (both ways) or all-transparent (diode) | **a 1:1 slope cannot carry a second line 2 y above it at all** — every dust is the lower end of a step, so the rule degenerates to all-transparent, which merges (H5) |
| 6 | `repeater_drop` | ~2×1 per y (est.) | repeater rules (`probe_station`) | solid floor (static legality) + the weakly powered block the back reads | dust cannot read weak power, so only a *device* may stand at the bottom of a drop (D2/D3) |

### Probe citations, per row

| # | probe |
|---|---|
| 1 | `probe_vert.py`; `probe_vertical_forms.py` L1–L6; **`probe_torch_burnout.py` B1/B2/B3/B5** |
| 2 | `probe_vertical_forms.py` G1–G4; `probe_torch_burnout.py` B4 |
| 3 | `probe_vertical_forms.py` S0–S4; `probe_spiral_tiling.py` T1–T8; `probe_torch_burnout.py` B4 |
| 4 | `probe_vertical_forms.py` **A1** (4 bits stacked, 16/16), H3, H4; `materials._verify_slope` |
| 5 | `probe_vertical_forms.py` H1, H2, H5; `probe_pivot.py` E |
| 6 | `probe_vertical_forms.py` D2/D3; `TRANSPORT_MODEL.md` rows 4 and 5 |

---

## TORCH BURNOUT — the disqualifier, and the methodological hole

### mc-tick models it, faithfully

**Yes** — so this is measurable here and does not have to be deferred to the
gametest oracle. `crates/mc-tick/src/components.rs`:

```
TORCH_BURNOUT_WINDOW = 60     game ticks the engine looks back over
MAX_RECENT_TOGGLES   = 8      turn-offs in that window before it goes dead
```

and `RedstoneTorch::on_scheduled_tick` records a toggle **only when the torch
is about to go dark** (`if powered { record_toggle(pos) }`), so the budget is
**8 turn-offs per 60 gt, not 8 state changes** — the engine's comment cites
`RedstoneTorchBlock.RECENT_TOGGLE_TIMER` / `MAX_RECENT_TOGGLES` from a decompile
and notes that 15 state changes make only 8 burnouts. On burnout the scheduled
tick simply `return`s, so the torch keeps its stale state until something
updates it again. mc-tick also unit-tests both sides of the threshold. **Fidelity
is not the problem here; our probes were.**

### Measured failure rates

Metric: **output transitions per input transition while the input keeps
changing** — a healthy carrier scores 1.0 at every rate. `hold` is how long
each input state is held, in game ticks.

| hold per state | 1 gt | 2 gt | 3 gt | 4 gt | 5 gt | 6 gt | 8 gt |
|---|---|---|---|---|---|---|---|
| single torch (**== a NOT gate**) | **0.00** | 0.62 | 0.75 | 1.00 | 1.00 | 1.00 | 1.00 |
| 4-torch ladder (8 y) | **0.00** | 0.58 | 0.58 | 0.96 | — | 0.96 | 1.00 |
| `ring_riser` 3×3 | 1.00 | 1.00 | 1.00 | 1.00 | — | — | — |
| `glass_tower` | 1.00 | 1.00 | 1.00 | 1.00 | — | — | — |

* **Threshold: hold ≥ 4 gt is safe, ≤ 3 gt burns out.** Arrived at
  independently and then checked against the rule: a full cycle is 2 holds and
  only the off-going half counts, so burnout needs 8 turn-offs in 60 gt, i.e.
  hold ≤ 3.75 gt. Measured boundary 3 / 4. (B1)
* At **hold 1 gt the tower is completely dead** — zero output transitions.
* **A taller tower is not safer, only slower.** Burnout is per *torch*, and
  every torch in the chain sees every input change, so a 4-torch ladder fails
  at exactly the single torch's threshold. (B2)
* **Parity does not help**: a 2-torch non-inverting ladder burns out the same.
  Parity is about polarity, not the toggle budget. (B5)
* **⇒ the fastest data rate a torch will carry is one change per 4 gt (2
  redstone ticks). A bus that updates every redstone tick is over the limit.**
* The dust forms have **no toggle budget at all** — no scheduled component to
  burn — and track 1:1 down to one input change **per game tick**. (B4)

### Why every other probe here was blind to it — and how the fault hides

This is the important part, worth more than any single form.

Every other probe drives levers with `rs.Levers.set()`, which flips one lever
and then **settles to quiescence** before doing anything else. That is a toggle
rate of at most one change per settle — orders of magnitude below the burnout
threshold. So *"8 torch ladders verified over all 256 patterns"*
(`probe_vertical_forms` L4) measured precisely the regime in which burnout
**cannot appear**. The circuit was never wrong; the rig could not ask the
question.

Worse, the fault actively covers its tracks (B3):

1. **Silent internal corruption.** Right after a burst at hold 2 gt, the
   4-torch tower's internal state is `[dark, dark, lit, dark]` where a healthy
   lever-off tower must be `[lit, dark, lit, dark]` — **and the exit reads 0,
   which is the correct output for lever-off.** A port-only check sees a healthy
   riser.
2. **Settling cannot repair it.** The world is already quiescent, so
   `run_until_quiescent` advances **0 ticks** and re-evaluates nothing; 180
   further game ticks (3× the window) change nothing either. Burnout is
   stuck-until-update, exactly as vanilla.
3. **The first input change after the fault produces the WRONG OUTPUT**: the
   lever goes on, the tower is non-inverting, the exit must read 15 — it reads
   **0**. That is lost data at the port, not a delay.
4. **Then it heals**, over the next few updates — just in time for the next
   settle-based probe to call it healthy.

So the failure signature is: *correct when idle, wrong under load, correct again
by the time you look.* This is the same class of blind spot as the two already
recorded here — the ss-starved rig, where a leak one cell longer than the signal
read 0, and `pitch 1 leaks ss 14 while v=0 looks dark`. In all three the **rig,
not the circuit, decided the answer**.

**Rule adopted:** a form is not verified for bus data until it has been driven
by a *rate*, not a *sequence*. `probe_torch_burnout.Traffic` is that harness —
it toggles on a fixed period, never settles mid-run, and samples the output
every game tick so a transition cannot be missed.

### Every torch-bearing mechanism inherits this

Burnout lives on the torch, so the limit is not a riser property. Flagging the
whole model — **each of these carries `max hold ≥ 4 gt per state`**:

| where | mechanism | consequence |
|---|---|---|
| `TRANSPORT_MODEL.md` row 7 | `torch_floor` — described there as "the only inverter and the only compact vertical carrier" | **both halves of that sentence need the limit attached.** The row's `delay 2 gt` should be joined by `max toggle: hold ≥ 4 gt` |
| `TRANSPORT_MODEL.md` router-unlock list, item 6 | "torch tower — compact vertical transport" | should be re-scoped to static/low-toggle signals, or dropped in favour of the ring riser |
| `genlib_map.py`, `rca_cells.py`, `seq_cells.py`, `cells.py` | **every NOT gate is a standing torch** | the limit bounds the whole design's **clock rate**, not just its buses: a 50%-duty clock needs a half-period ≥ 4 gt, so **period ≥ 8 gt (4 rt)**. Anything faster degrades combinational logic, not merely transport |
| `timing.py` | static timing analysis | has no burnout term; a design can pass STA and still burn out |
| repeaters, comparators, dust | — | **no burnout.** Repeaters are the safe way to refresh a fast signal |

I own none of those files, so this is a flag, not an edit. The two concrete
asks: add a `max_toggle` field beside `delay` on `TRANSPORT_MODEL.md`'s row 7,
and give `timing.py` a burnout check against each torch's fan-in toggle rate.

---

## WHY DOWN IS ASYMMETRIC

1. **D1 — a torch powers the block ABOVE it and nothing beneath.** Dust on the
   block above a lit torch reads 15; dust on the block under its attachment
   reads 0. The only compact active carrier only ever hands the signal *up*, so
   there is no descending torch form to find. (And, per the section above, the
   ascending one is unusable for data anyway.)
2. **D2 — a dust weak-powers the block it sits on, and dust cannot read weak
   power**, so a dust cannot hand its signal down through its own floor either.
3. **D3 — but a repeater whose BACK is that weakly powered block fires** (15).
   That is the only active descent: 1 y per station, 2 gt, ~2 horizontal cells —
   and **repeaters do not burn out**, so it is the safe unbounded option.

The footprint asymmetry then follows geometrically. Going **up**, a step's upper
dust may sit on a non-conductor, so a riser can fold into a 2-wide zigzag: the
support above each dust doubles as the next dust's floor. Going **down**, the
diode rule demands the upper dust's support be a **conductor** while the cut
rule demands the cell directly above each dust be a **non-conductor** — and in a
2-wide fold those are the same cell. Over-constrained, so a descending riser
must leave the cell above every dust empty, which forces a closed loop of length
≥ 8: a **spiral staircase on a ≥ 3×3 footprint**.

Exactly the user's "going down is harder and usually ends up with spiral
staircases", and the probes agree: form 2 descends nowhere (G3), form 3 descends
in a 2×2 (one bit, S0) and a 3×3 (two bits, T6).

---

## THE RING LAW — what "offset 180 degrees" actually is

The user's claim — *"for horizontal spiral staircases can tile if they are
offset 180 degrees"* — is true, and the reason generalises to one integer.

A ring riser is dust on the **perimeter of an sx × sz footprint, one cell per
y**, so **y == path index**. Two cells of that path are planar-adjacent iff they
are *consecutive on it* — provided the cycle is **chordless**, which the
perimeter of a rectangle is exactly when `sx >= 3 && sz >= 3`. Put a second bit
on the same ring at path offset `sep`, and in **every column they share** they
sit exactly `sep` apart in y. The whole legality question is that integer:

| `sep` | what happens | measured |
|---|---|---|
| 1 | one step apart in every shared column: **one net** | T2 (leak 15) |
| 2 | the upper bit's SOLID support lands in the lower bit's **CUT cell** and severs it: one bit **DEAD**, foreign power inside its cells | **T3** |
| **≥ 3** | **legal** — no pair is ever 0 or 1 apart in y anywhere | T2, T4, T5 |

A period-8 ring (3×3) holds `floor(8/3) = 2` bits at offset 3 or 4, and
**offset 4 of 8 is "180 degrees"** — 4/4 patterns, zero crosstalk, one
footprint. In general a ring holds **`floor(perimeter/3)` bits**, and
`vforms.ring_bits` spreads them evenly, because the spare separation is what
buys isolation across the seam to the next ring (T7: `[0,4]` tiles at pitch 4,
`[0,3]` does not).

| ring | perimeter | bits | offsets | xz-claim/bit | patterns |
|---|---|---|---|---|---|
| 2×2 | 4 | 1 | `[0]` | 4.00 | S0 (up **and** down) |
| 3×3 | 8 | **2** | `[0, 4]` = **180°** | 4.50 | 4/4 |
| 4×3 | 10 | 3 | `[0, 3, 7]` | 4.00 | 8/8 |
| **5×3** | 12 | **4** | `[0, 3, 6, 9]` | **3.75** ← densest | 16/16 |
| 6×3 | 14 | 4 | `[0, 4, 7, 10]` | 4.50 | 16/16 |
| **11×3** | 24 | **8 (a byte)** | `[0, 3, …, 21]` | 4.125 | 256 |

3.75 is the family optimum: with `sz = 3`, `cells/bit = 3·sx / floor((2·sx+2)/3)`
bottoms out at `sx = 5`, and `sz ≥ 4` is strictly worse.
`probe_spiral_tiling.legal(sx, sz, offsets)` is the closed form, and **T8 checks
it against all 17 simulated configurations with zero mismatches**.

Two properties that make this a real *bus* form:

* **all bits enter on one level and leave on one level.** Bit `p` occupies
  `ring[(k+p) mod m]` at `y0+k`, so at any y every bit has exactly one dust, in
  its own column. No level-shift adapter at either end.
* **direction-agnostic.** The same geometry carries every bit downward (T6) and
  upward (S1, and every T2/T4/T5 rig is driven from the bottom) with identical
  numbers. This is why demoting the torch ladder *simplifies* the fabric: one
  template serves both directions.

---

## THE SLOPE RULE — "every second block is transparent"

* **H1** — a 1:1 staircase on **all-solid** supports conducts both ways.
* **H2** — on **all-transparent** supports it conducts **up only**: the
  transparent diode, a free one-way isolator.
* **H3** — two lines 2 y apart on a half slope with **all-transparent**
  supports **MERGE**: the foreign diagonal from the lower line's dust up to the
  upper line's dust one x back survives, its cut cell being the upper line's
  glass support.
* **H4** — the same pair with **alternating** supports is isolated in all four
  patterns. The evaluable rule:

  > `support[i]` is **transparent** iff dust `i` is the **lower end of an in-use
  > 1-y step**; otherwise **solid**.

  On an ascending half slope that is glass at odd `i`, solid at even `i` — the
  user's "every second block is transparent". It is *exactly* every second block
  because a support does two jobs: it is this line's floor **and** the cut cell
  of the line 2 y beneath it (`support[i]` sits at `y_i − 1 == (y_i − 2) + 1`).
  Transparent where the lower line's own step must survive; solid where the
  *foreign* diagonal must be severed.
* **H5** — therefore **the slope must be a HALF slope.** On a 1:1 slope every
  dust is the lower end of a step, so the rule degenerates to all-transparent,
  which H3 measures as a merge. A half slope is the steepest slope that can
  carry a stacked bus, because half its dusts are flat cells whose cap may be a
  conductor. Derived *and* measured.
* **A1** — and it scales: **4 bits stacked at 2 y pitch on one half slope,
  16/16 patterns, zero crosstalk**, the whole bundle climbing 4 y inside a
  single 10-cell, 1-wide lane.

---

## CELL TEMPLATES

Closed forms in `crosswire_tiles.md` style: every cell is a formula in a level
or path index, tile-local coordinates, so these port to Rust mechanically.
Reference implementations in `vforms.py`, executed by the probes — so the
formulas below are the ones actually verified.

Any solid block works where a material is "solid"; any **sturdy
non-conductor** works where one is "transparent" (`glass`,
`white_stained_glass`, `smooth_stone_slab[type=top]` — G1 covers glass and the
top slab). Bottom slabs are vanilla-illegal.

---

### `LADDER_TOWER` — 1×1, UP, 2 y per torch, inverting

> ⚠ **NOT FOR SWITCHING DATA.** 1 gt per y on the critical path, and its
> torches burn out below a 4-gt hold per input state
> (`probe_torch_burnout.py`). Use for **static or low-toggle control signals**
> only — configuration lines, mode selects, one-shot enables — and call
> `vforms.assert_data_safe("torch_ladder", hold_gt)` first. For anything a bus
> carries, use `RING_RISER`.

Unit-local frame: column `(0, 0)`, base level `y0`, `T` torches, port side
`s ∈ {−1, +1}` along z.

| role | cells | material |
|---|---|---|
| torch attachment | `(0, y0 + 2t, 0)`, `t = 0 .. T−1` | solid |
| torch | `(0, y0 + 2t + 1, 0)` | `redstone_torch[lit=true]` |
| **cap (STRONG)** | `(0, y0 + 2T, 0)` | solid; strongly powered by the last torch |
| entry dust | `(0, y0, s)` | dust — **dead-ends into the base block** |
| entry floor | `(0, y0 − 1, s)` | solid |
| exit dust | `(0, y0 + 2T, s)` | dust — reads the cap's STRONG power on its side face |
| exit floor | `(0, y0 + 2T − 1, s)` | solid |

Ports: in `(0, y0, s)` from direction `s`; out `(0, y0 + 2T, s)` leaving along
`s`. Budgets: rise `2T`, delay `2T` gt (**1 gt per y**), **inverting iff `T` is
odd**, output refreshed to 15 (no reach limit), **max toggle: hold ≥ 4 gt**.

**`LADDER_BUS`** — bit `i` at `x = x0 + i` (**pitch 1**) with
`s_i = −1 if i even else +1`. 8 bits verified over all 256 *settled* patterns
with zero leak at every exit and entry (L4) — which, per the burnout section, is
a statement about isolation only, **not** about traffic. Same-side ports at
pitch 1 make the array read a constant (L5).

---

### `GLASS_TOWER` — 2×1, UP only, 1 y per cell, 0 gt

| role | cells | material |
|---|---|---|
| dust | `(k mod 2, y0 + k, 0)`, `k = 0 .. n−1` | dust |
| support | `(k mod 2, y0 + k − 1, 0)` | **transparent — mandatory** |

The support of dust `k` sits at `(k mod 2, y0 + k − 1, 0)`, and dust `k−2` sits
at `((k−2) mod 2, y0 + k − 2, 0)` — **same column, one level down**. So each
support *is* the CUT cell of the step `k−2 → k−1`: a conductor there severs it
and the tower dies after one cell (G2); a non-conductor keeps it alive but also
puts every upper dust on a non-conductor, which is the transparent-diode law and
is why the tower is one-way. Only `k = 0, 1` are unconstrained.

Ports: in `(0, y0, 0)` from −x; out `((n−1) mod 2, y0 + n − 1, 0)`. Budgets:
rise `n − 1`, **0 gt**, out ss `14 − (n − 1)` ⇒ **14 y per source**,
non-inverting, **one-way UP** (G3), **no toggle limit**. Neighbour towers at
**z-pitch ≥ 2** (G4).

---

### `RING_RISER` — the spiral staircase; UP **and** DOWN; multi-bit; **the recommended riser**

Footprint `sx × sz`, `sx, sz ≥ 3`, origin `(ox, oz)`, first level `y0`, `n`
cells. `ring(sx, sz)` is the clockwise perimeter from `(0,0)`;
`m = 2(sx + sz) − 4`; offsets `P = ring_bits(m)` — `floor(m/3)` values spread
evenly. For bit `p ∈ P`, `k = 0 .. n−1`, `c = ring[(k + p) mod m]`:

| role | cells | material |
|---|---|---|
| dust | `(ox + c.x, y0 + k, oz + c.z)` | dust |
| support | `(ox + c.x, y0 + k − 1, oz + c.z)` | **solid** |
| **must stay AIR** | `(ox + c.x, y0 + k + 1, oz + c.z)` | air — the CUT cell, and the reason the fold needs a loop |
| port stub | outward along `ring_outward(cell)`, both ends | dust on solid |

Ports: bit `p` enters at `ring[p]`, `y0`; leaves at `ring[(n−1+p) mod m]`,
`y0 + n − 1`. **Every bit enters on level `y0` and leaves on `y0 + n − 1`**, each
in its own column.

Budgets: rise `n − 1`, **0 gt**, **−1 ss per y** ⇒ 14 y per source,
non-inverting, **bidirectional**, **no toggle limit — safe for any traffic**. A
repeater on any flat ring cell refreshes to 15 for an unbounded climb or
descent, costing **one cell of rise and 2 gt** (S4: ss 5 → 15).

Legality (`probe_spiral_tiling.legal`):

```
sx >= 3 and sz >= 3                        # the perimeter must be chordless
min(|a-b|, m-|a-b|) >= 3   for all a,b in P
```

Neighbour rings: **x-pitch `sx + 1`** — one empty column. Flush rings leak at
every rotation. At pitch `sx + 1`, T7 measures **4/4** clean rotations for 2×2,
**4/8** for 3×3, **8/12** for 5×3 — so a tiling router must pick a clean
rotation or verify the seam.

---

### `HALF_SLOPE` — level change that stacks the whole bundle in one lane

Line at `y0`, cells `i = 0 .. n−1`, lane `z`, ascending in +x:

| role | cells | material |
|---|---|---|
| dust | `(x0 + i, y0 + i//2, z)` | dust |
| support | `(x0 + i, y0 + i//2 − 1, z)` | **transparent if `lower_end[i]` else solid** |

`lower_end[i]` is true iff `y[i±1] == y[i] + 1` — on an ascending half slope,
exactly the odd `i`. Stacked lines repeat the pattern at `y0 + 2j` with the
**same** material sequence, because each line's support is the cut cell of the
line beneath it.

Budgets: 1 y per **2** x cells, **0 gt**, **−2 ss per y** ⇒ **7 y per source**,
bidirectional, **no toggle limit**, **stacks at 2 y pitch** (H4 4/4, A1 16/16
at 4 bits). A 1:1 slope cannot stack at all (H5).

---

## RANKED RECOMMENDATIONS

Ranked on **(latency, burnout safety, density)** in that order, for **switching
bus data**. Density is the tiebreak, never the lead.

### Vertical UP — for data

| rank | form | gt/y | data-safe | blocks/y/bit | xz-claim/bit | why it ranks here |
|---|---|---|---|---|---|---|
| **1** | **`RING_RISER` 5×3, 4 bits** | **0** | **yes** | 2.000 | **3.75** | 0 gt, no toggle limit, non-inverting, and the *same* template descends. 16/16 patterns |
| 2 | `RING_RISER` 11×3, 8 bits | 0 | yes | 2.000 | 4.125 | a whole byte in one shaft, all bits on one entry and one exit level; 256 patterns |
| 3 | `HALF_SLOPE`, stacked | 0 | yes | 2.000 | lane shared by all bits, 2 x cells per y | **best area of all** when the route has horizontal room: the entire bundle rides one 1-wide lane (A1). Costs reach — 7 y per source |
| 4 | `GLASS_TOWER` | 0 | yes | 2.000 | 4.00 | narrowest *dust* shaft at 2×1, but z-pitch 2 wastes the lane between towers. Take it for a single bit or when the diode is wanted |
| — | ~~`LADDER_BUS`~~ | **1** | **NO** | 1.000 | 3.00 | **DEMOTED — disqualified for data.** 1 gt/y on the critical path (a 30 y rise = 30 gt = 1.5 s) *and* it burns out above one change per 4 gt |

### Vertical DOWN — for data

| rank | form | gt/y | data-safe | xz-claim/bit | why it ranks here |
|---|---|---|---|---|---|
| **1** | **`RING_RISER`, sized to the bit count** — 5×3/4 bits, 11×3/byte, 3×3/2 bits at 180°, 2×2/1 bit | **0** | yes | **3.75** / 4.125 / 4.50 / 4.00 | densest descending form; 16/16 up and 16/16 driven from the top |
| 2 | `repeater_drop` | 2 | yes | ~2 (est.) | the only **unbounded** descent, refreshing to 15 every level, and repeaters do not burn out. For very deep drops. Only the single stage is probed (D3) |
| 3 | `HALF_SLOPE` / `stair_1to1` | 0 | yes | lane + horizontal travel | combined horizontal + vertical moves |
| — | any torch form | — | — | — | **does not exist going down** (D1). A descent must never be planned as a climb reversed |

### Where the torch ladder is still the right answer

Not nowhere — just not on a data path. It is 1 block per y per bit, needs no
repeaters at any height, and refreshes to 15, so it wins for signals that
change rarely: **configuration and mode lines, one-shot enables, static
selects, reset lines**. The gate is one call:
`vforms.assert_data_safe("torch_ladder", hold_gt)` with the hold time the caller
can actually guarantee.

### Best TORCH-FREE dense ascent — honestly against 1.000 block/y/bit

Two answers, because the forms are different kinds of thing:

* **For a pure shaft** (no horizontal budget): `RING_RISER` 5×3 at 4 bits —
  **2.000 blocks/y/bit and 3.75 xz cells/bit**, versus the ladder's **1.000 and
  3.00**. So going torch-free costs **2× the blocks per y and 1.25× the
  reserved area**. That is the correct trade and it should be stated as one:
  the ladder's 1.000 was never available for data at any price.
* **If the route has horizontal room**: the **stacked `HALF_SLOPE`** beats even
  the ladder on area — the entire bundle rides a single 1-wide lane, so per-bit
  footprint *falls* as the bus widens (10 cells for 4 bits at 4 y of climb;
  8 bits in the same lane would halve it again). It pays in reach (7 y per
  source) and horizontal distance (2 cells per y), not in area.

---

## TRAPS (each one cost a probe iteration)

1. **Settle-based verification cannot see burnout.** The whole methodology
   section above. A form is only verified for bus data once it has been driven
   by a *rate*, not a *sequence*.
2. **An ss-starved rig reports "no crosstalk" for a shorted circuit.** The first
   tiling sweep read 12 ring cells past a bare lever, so the exit sat at **ss
   1**, and a leak arriving one cell longer read **0**. Two-wide rings scored
   clean while being one merged net. Fixed by driving through a repeater and by
   treating a leak **anywhere in an undriven bit's cells** as a failure.
3. **A form can fail by going DEAD rather than by leaking.** Ring offset 2
   severs a bit whose exit stays quiet in every pattern (T3); same-side pitch-1
   ladders read a constant with zero leakage (L5). Assert **conduction** and
   **isolation** together, always.
4. **A fault that repairs itself before you look.** Post-burnout the tower is
   internally corrupt but its exit reads correctly, and a few updates later it
   is genuinely fine. Sample *during* the stimulus.
5. **The port lane, not the carrier, usually sets the pitch.** Torch towers are
   electrically happy at pitch 1; their entry dusts are not — and the failure is
   the POINTING LAW, not a short.
6. **Transparency is directional.** A transparent support keeps a step alive
   *uphill* and blocks it *downhill*, so "use glass to keep the diagonal" turns
   any bidirectional form into a diode.
7. **The cell above a dust and the cell below the next dust are the same cell in
   a 2-wide fold.** That coincidence is the whole reason DOWN needs a ≥ 3×3
   spiral while UP fits in 2×1.
8. **Optimising the wrong objective.** The first version of this file ranked on
   density and put a 1-gt-per-y, burnout-prone carrier first. For a bus,
   **latency and survivability outrank density**, which is why `vforms.FORMS`
   now lists `gt_per_y`, `max_toggle_gt` and `data_safe` *before* the density
   fields.

---

## CARRIED OUT OF THIS WORK (open, deliberately not built)

* **`TRANSPORT_MODEL.md` needs a `max_toggle` field** beside `delay` on row 7
  (`torch_floor`), and its router-unlock item 6 ("torch tower") should be
  re-scoped to static signals. Not my file.
* **`timing.py` has no burnout term.** A design can pass static timing and
  still burn its torches out. The check is per-torch: the toggle rate of its
  fan-in versus a 4-gt hold. Likewise the NOT-gate limit bounds a **clocked
  design's period to ≥ 8 gt**, which `seq_counter.py` / `seq_cells.py` do not
  know.
* **The ring's repeater station is measured (S4), not templated.** Which flat
  ring cell, and how the path resumes one level late, is the missing piece for
  descents deeper than 14 y.
* **`repeater_drop` is proven only as a single stage** (D3); its per-y formula
  and neighbour pitch are unmeasured. It matters more now that it is the
  recommended unbounded descent.
* **Ladder port adapters** (fan-in/fan-out across two alternating z lanes) —
  lower priority now that the form is off the data path.
* **Crossing a riser.** Nothing here says how a horizontal bus passes a shaft;
  the `crosswire_tiles.md` families are all planar.
* **`legal(sx, sz, offsets)` should move into `materials.py`** and then to Rust,
  beside `step_reads`. Six lines, and T8 shows it is exact.
* **Are there other rate-dependent mechanisms we have never driven by rate?**
  Repeater locking and comparator priming are both stateful and both currently
  verified only by settle. Same rig would answer it.
