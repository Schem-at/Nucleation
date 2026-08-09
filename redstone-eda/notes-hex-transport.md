# NOTES — the hex REPEATER transport bus (`TRANSMIT002_hex_transmit_flat`)

Status: probed 2026-08-10 against a real user-supplied build.
Corpus copy: `redstone-eda/corpus/TRANSMIT002_hex_transmit_flat.schem` (833 B,
copied, original untouched).
Probes: `probe_hex_transmit.py` (**66/66**), `probe_hex_vs_comparator.py`
(**22/22**). Every number below cites the group that produced it.

The hypothesis under test was the author's own description:

> "a typical hex bus that uses repeaters to transport the signal, it's wider and
> less easy to fit but way faster than alternating comparators dusts and blocks."

**Verdict: CONFIRMED, with numbers.** 4.00x faster, 3x wider, 2.8x the blocks
(C3.1), and — the part that matters most for us — it is **value-preserving for
all 16 analog levels** (H2.2, H3.2) *even though its carrier is repeaters*.
That last fact contradicts a hard rule currently written into
`BUS_CRATE_DESIGN.md`; see "What this breaks" at the end.

---

## 1. What the build actually is

3 wide (x) x 2 high (y, incl. floor) x 19 long (z); 98 non-air blocks (H0.1,
H0.3). Palette: 47 magenta wool (floor) + 31 redstone dust + **15 repeaters** +
2 comparators + 1 barrel + a lime and an orange wool marker (H0.4). Transport
runs along **-z**. The floor is solid under every powered cell (H0.5).

```
        x=0            x=1                    x=2
z=18   barrel  <- ANALOG SOURCE (orange wool marks the input)
z=17   CMP facing=south, compare        (reads the barrel, drives z=16)
z=16   dust  v         REP facing=west  dust
z=15   dust  v-1       REP facing=west  dust        <- 15 rungs, z=2..16
 ...   ...             ...              ...            all delay=1
z= 2   dust  v-15      REP facing=west  dust
z= 1                                    CMP facing=south, compare
z= 0                                    dust  <- READ (lime wool marks it)
```

* `facing` on both diode kinds names the **input** side (vanilla `DiodeBlock`
  reads the neighbour in the FACING direction). Verified against the author's
  own saved state, not assumed: the saved `powered=true` repeaters are exactly
  the rungs whose x=0 neighbour has ss >= 1 (H0.9).
* The barrel holds 3 stacks + 55 redstone and both comparators saved
  `OutputSignal:3` (H0.11); the author even named the barrel `"3"`.
* mc-tick reloads the file and reaches quiescence in **0 ticks** in `InWorld`
  mode and 2 in `Placement` mode, reproducing the saved lane strengths cell for
  cell (H1). The file is a self-consistent snapshot of a live circuit.

### Mechanism decomposition

| # | part | cells | what it does |
|---|---|---|---|
| a | container + comparator | (0,1,18) + (0,1,17) | analog SOURCE: barrel fill -> ss `v` in 0..15 (H2.1) |
| b | INPUT lane (x=0) | 15 dust, z=2..16 | carries `v` downward, **decaying 1 per cell**: ss(z) = v - (16-z) |
| c | the COMB (x=1) | 15 repeaters, facing=west, delay=1 | 15 *parallel* diodes. Rung z fires iff lane-b ss(z) >= 1, so **exactly `v` rungs fire** and the lowest firing rung sits at **z = 17 - v** (H2.3, H2.4) |
| d | OUTPUT lane (x=2) | 15 dust, z=2..16 | each firing rung strongly drives it to **15**; the lowest one dominates, so ss(z) = 15 - ((17-v) - z), i.e. the tap at z=2 reads **exactly `v`** |
| e | tap comparator | (2,1,1) -> (2,1,0) | re-emits `v` at full analog fidelity into the next stage |

**The trick, stated once:** signal strength is converted to a *position* (which
rung fires) and the position is converted back to a strength (how far that rung's
15 has to decay to reach the tap). The two conversions are exact inverses, so a
**repeater** — the one component that destroys analog values — ends up in the
middle of an **analog-lossless** carrier. The decay budget spent is
`(v-1) + (15-v) = 14` cells for a 14-block traverse *for every value of v*; the
refresh always lands exactly where the budget runs out.

---

## 2. What "hex" means HERE — evidence, not assumption

**(a) one analog signal-strength value 0..15 on one logical wire**, transported
through a *unary/thermometer* intermediate. It is **not** (b) four binary lines
carrying a nibble, and **not** (c) a binary re-encoding.

Evidence:
* The source is a container comparator, whose only output is a signal strength
  (H0.11, H2.1). Nothing in the build reads or writes individual bits.
* Sweeping barrel fill over all 16 levels: `v_out == v_in` for every one of
  0..15 (H2.2). A 4-line binary bus could not deliver a *strength*.
* The intermediate is unary, not binary: `#rungs fired == v` exactly (H2.3), and
  the *active* rung is at `z = 17 - v` (H2.4). 15 rungs = 15 unary positions.
* There is no encoder and no decoder in the file. Compare
  `compositor/hexanalog.py`, which *does* encode/decode a nibble to and from a
  strength — this build has neither; it is pure transport.

---

## 3. Measured — head to head, matched spans

`span` = z-blocks from the cell where the analog value is injected to the cell
where it is read. Both carriers are driven by the **same** lever+attenuator
source, and the shared source comparator's 2 gt is subtracted from both, so each
carrier is charged only for itself (`probe_hex_vs_comparator.latency`).

| carrier | span (z) | latency (gt) | gt/block | width | height | devices | blocks | blocks/span | value-preserving |
|---|---|---|---|---|---|---|---|---|---|
| comparator+block chain (C1, C3) | 16 | 16 | 1.000 | 1 | 2 | 8 | 34 | 2.1 | **yes** (C0.2) |
| **hex repeater comb x1** | 16 | **4** | **0.250** | 3 | 2 | 17 | 94 | 5.9 | **yes** (C2.3) |
| comparator+block chain | 32 | 32 | 1.000 | 1 | 2 | 16 | 66 | 2.1 | yes |
| **hex repeater comb x2** | 32 | **8** | **0.250** | 3 | 2 | 34 | 186 | 5.8 | yes (C2.3) |
| comparator+block chain | 48 | 48 | 1.000 | 1 | 2 | 24 | 98 | 2.0 | yes |
| **hex repeater comb x3** | 48 | **12** | **0.250** | 3 | 2 | 51 | 278 | 5.8 | yes (C2.3) |
| binary dust + repeater/15 (C3b) — *reference* | 16/32/48 | 2/4/6 | **0.125** | 1 | 2 | 1/2/3 | 34/66/98 | 2.1 | **NO** — arrives normalised to 15 (C3b.3) |

**4.00x faster at every span probed, exactly** (C3.1). Both scale linearly, so
the ratio is a constant, not an artefact of one length.

Which comparator-chain link is legal (C0):

| link between comparators | pitch | lossless? |
|---|---|---|
| 1 solid block | 2 | **yes** (C0.2) |
| 1 dust | 2 | **yes** (C0.2) — this is our own `hexanalog` trunk, `build_trunk()` |
| 1 block + 1 dust | 3 | no, -1 ss/stage (C0.3) |
| 2 dust | 3 | no, -3..-4 ss/stage (C0.3) |

So the comparator trunk's 1.000 gt/block is not a strawman: pitch 2 is its best
legal packing, and the extra cell you would need to stretch it costs a level.

### Delay attribution for one hex stage (H4.6, H4.7)

| checkpoint | gt after the source lever flips |
|---|---|
| source dust | 0 (dust is free) |
| comb top = INJECT | 2 (source comparator) |
| OUTPUT lane, anywhere | 4 (the comb repeater; +2) |
| tap, 14 blocks away | 4 — **the whole 15-cell output lane is 0 gt** |
| READ dust | 6 (tap comparator; +2) |

The **transport section alone** (comb top -> tap, 14 blocks of z) is **2 gt**
(H4.8) = 0.143 gt/block; the chainable figure of 0.25 gt/block includes the tap
comparator that makes the value re-injectable. Falling edges cost the same 6 gt
(H4.5). Repeater `delay` is a usable knob: latency = `4 + 2*delay` gt and every
delay stays lossless (H9.4, H9.3).

### Throughput / pipelining (H5)

* **Pipelined, depth 4 gt.** During a 15 -> 6 transition the INPUT lane already
  carries 6 while the OUTPUT still reads 15, for game ticks 2..5 (H5.5, H5.6).
  Two values are genuinely in flight at once; a second value may be injected
  4 gt before the first arrives.
* **Minimum separation 3 gt.** A 1-gt or 2-gt gap between values is *swallowed*
  by the delay-1 repeater and never appears at the output (H5.8); >= 3 gt gets
  through intact (H5.9). The comparator chain filters the same way (C4.2), so
  this is not a disadvantage of the hex bus — but it does mean neither carrier
  is safe for sub-3-gt pulse protocols.

### Envelope, clearance, crosstalk (H8)

* A solid **lid** over all three lanes changes nothing (H8.1) — so the envelope
  really is 3x2, with no vertical halo above.
* A **foreign dust line on that lid** (y+2) is bidirectionally isolated: the bus
  does not power it (dust never powers the block above — W3) and driving it does
  not disturb the bus (H8.2, H8.3, H8.4).
* **Lane pitch 2.** A foreign dust lane 1 cell from the OUTPUT lane picks up
  ss 14 (H8.6) — the OUTPUT lane runs *hot at 15* along most of its length, so
  it is the worst possible neighbour. At gap 2 the leak is 0 (H8.7).
* **No locking.** The 15 rungs are side by side but all face the same way, so
  none points into another's side; `locked=true` never appears (H9.1).
* Trap found while building the rig, worth a model note: a floor **lever
  strongly powers its attachment block**, and dust reads a strong block at 15 on
  **all six faces** — parking a lever's attachment block directly above an
  analog lane injected 15 into it and silently corrupted the value (comment at
  H8b). A planner must treat a lever attachment as a 6-directional 15-source.

### Calibration — the stage is a fixed-length TILE (H6)

Only a **15-rung comb tapped at its last rung** is lossless. Otherwise:

```
out = min( 15 , v + (15 - comb_len) )        (H6.3, exact over 20 combinations)
out = min( 15 , v + (tap - 2)     )        (H6.5, exact)
```

A short comb **gains** signal strength. That is a feature — the comb is also a
free **+k level shifter** — but it means the carrier has no variable-length
form: you pick a length and you get a known shift, you do not pay a per-block
loss.

### Chaining (H7, C2)

Stages ping-pong across the same three columns: stage k's OUTPUT lane becomes
stage k+1's INPUT lane and the comb stays in the middle column x=1, flipping
`facing` west/east. Cross-section stays **3x2 for any number of stages**
(H7.5). Verified lossless through 3 stages, 48 z-blocks, 12 gt (C2.2, C2.3).

---

## 4. The carrier PROFILE (planner-facing)

Written to match the `TRANSPORT_MODEL.md` mechanism-row style. This is the row
that file does not have; it belongs there as **row 12**.

### A. carrier / requirement / emission / strength / delay

| # | mechanism | carrier block | requires | emits (kind @ offset) | signal strength | delay |
|---|---|---|---|---|---|---|
| 12 | `hex_comb_stage` | a 3x1x16 TILE: `redstone_wire` lane + `repeater` comb (all one `facing`, delay d) + `redstone_wire` lane + one `comparator` | `sturdy` at `(*,-1,*)` under all three lanes (45 cells/stage); an analog `v` presented on the INPUT lane's top cell | `STRONG`@the tap comparator's output cell, at strength `v` | **analog-exact: preserves the value** over the whole 16-block stage, 0 ss cost (H2.2/H3.2) | `4 + 2d` gt per stage; 2 gt for the transport section alone |

### B. what it may share space with, and the legality predicate

| # | mechanism | may share space WITHOUT interference | legality predicate |
|---|---|---|---|
| 12 | `hex_comb_stage` | a solid **lid** directly above all three lanes, and a **foreign dust line on that lid** (H8.1–H8.4); a foreign lane **>= 2** cells to either side (H8.7) | `sturdy` floor under 3x16; comb length **exactly 15** and tap at the last rung for losslessness, else apply the shift law; no foreign dust at pitch 1 (H8.6); no lever attachment block on any face of any lane cell |

### Profile numbers

| property | value | probe |
|---|---|---|
| footprint per lane (one analog nibble = one lane) | **3 x 2** cross-section, 16 z per stage | H0.13, H7.5 |
| ticks per block of distance | **0.25 gt/block** chained; 0.143 gt/block for the bare transport section | C2.2, H4.8 |
| refresh interval | **every 16 blocks, mandatory** — the stage *is* the refresh | H7.2, H7.3 |
| max span without a refresh stage | 16 blocks (14 + 2 comparator cells); the length is fixed, not a maximum | H6.2 |
| value-preserving | **YES**, all 16 levels, both barrel-driven and injector-driven | H2.2, H3.2 |
| support requirement | solid `sturdy` floor under all three lanes; repeaters need a floor (static legality only) | H0.5 |
| legal neighbour pitch | 2 cells lateral; 0 cells vertical if a solid lid is used | H8.6, H8.7 |
| pipelining | depth 4 gt; minimum value separation 3 gt | H5.5, H5.8 |
| block cost | ~5.8 blocks per block of span (vs 2.1 for a comparator trunk) | C3 |
| device cost | 17 devices per 16 blocks (15 repeaters + 2 comparators) | C3 |
| free extra | a `+k` level shift by shortening the comb to `15-k` rungs | H6.3 |

### Applies when

* the payload is an analog 0..15 strength (`Encoding::HexAnalog`), and
* latency dominates volume — it buys 4x over a comparator trunk, and
* a 3-wide x 2-high corridor is available for the whole run, and
* the span is a multiple of 16 (or a deliberate level shift is acceptable), and
* nothing needs to tap the value mid-run (a mid-run tap changes it — H6.5), and
* no other net needs to run at pitch 1 beside the OUTPUT lane.

### Declines when

* width < 3, or the corridor bends often — each stage is a rigid 16-long tile
  and turns are not characterised here.
* the span is not a multiple of 16 *and* an exact value is required: the
  mismatch shows up as a level shift, not as a graceful loss.
* pulses shorter than 3 gt must survive (H5.8) — but the comparator trunk fails
  this identically (C4.2).
* the signal is binary. A plain repeater line does the same job at 0.125
  gt/block in 1 wide and 1/17th the devices (C3b).
* block/volume budget is tight: 2.8x the blocks of a comparator trunk.

---

## 5. What this breaks in our model — the most valuable output

### 5.1 The hard blocker: `value_preserving` is attached to the wrong object

`BUS_CRATE_DESIGN.md` (section (c), ~line 780) states as a **hard rule**:

> A bus whose `Encoding` carries meaning in the signal *strength* — including
> `Encoding::HexAnalog` — **must refuse** any carrier with
> `value_preserving == false`. […] A **repeater normalizes its output to 15 and
> destroys the analog signal-strength value**; a **comparator preserves it**.

and mandates the test `hexanalog_bus_refuses_repeater_carrier`.

The rule's *intent* is right (silent corruption is the failure class we keep
paying for) and its *premise* is right at block granularity — we re-measured it:
feed a repeater 8 and read 15 (C3b.3). But the corpus build is a **repeater
carrier that is `value_preserving == true`**, verified over all 16 levels
(H2.2), and it is the fastest analog carrier we have measured. As written, our
planner would refuse it by name.

The defect is granularity: **`value_preserving` is a property of a STAGE, not of
a block kind.** `normalizes(repeater) = true` does not imply
`normalizes(stage containing repeaters) = true`, because 15 repeaters in a unary
comb compose into an exact identity on the value. The flag must be a measured
property of a tile (`value_preserving_carriers_round_trip_all_16_levels` — which
already exists as a mandated test! — is the right *definition*, and the
block-kind inference is the wrong one). Recommended fix: derive
`value_preserving` **only** from that in-sim 16-level round trip and delete the
block-kind inference; keep the refusal, keep the fuzz test, and let this carrier
pass it. *(That file is being edited by another agent; this note only cites it.)*

### 5.2 Cost is not additive over cells

Every cost model in the fabric charges per cell: `dust` costs 1 ss and 0 ticks;
a hop costs +1 ss; a repeater costs 0 ss and 2 gt (`TRANSPORT_MODEL.md`, "What
the search state must become", point 3). This carrier's stage costs **0 ss for
16 blocks**, and with a short comb it costs **negative ss** (a +k gain, H6.3).
A negative edge weight breaks the monotonicity that A*/Dijkstra in `router.py`
relies on. The stage has to enter the move set as an **atomic tile with a
fixed span and a fixed (possibly negative) ss delta**, never as a sequence of
per-cell moves.

### 5.3 A route is a path; this carrier is a *field*

`router.py` finds a path (`find`/`emit` produce a cell sequence) and `nets.py`
claims those cells for one label. Here **all 45 lane cells + 15 rungs belong to
one net**, and the *electrical* path through them changes with the transported
value: for `v=3` the current flows through rung z=14, for `v=15` through rung
z=2 (H2.4). The geometry is value-independent; the path inside it is not. No
"sequence of cells + delay" representation can express that, and any DRC that
asserts "one driver per net cell" or "the net is a simple path" will flag a
correct build.

### 5.4 One-lane-per-signal, and the 3-wide bundle

`nets.py` is dust-only and one claim per cell; the bundle geometry enumerated in
the routing design is "vertical 2y stack, flat 2z lane, HexAnalog", where
`HexAnalog` means *one wire per nibble* (`compositor/hexanalog.py`: "a 4-bit
value on ONE wire"). This carrier is one nibble on **three x-adjacent columns of
which the middle is devices, not wire** — a lane form the bundle enumeration has
no name for. The 2-cell lateral pitch then applies to the *bundle*, not to a
wire, so two hex buses need 3+1 = 4 x-cells each.

### 5.5 What is NOT broken (checked, so it is not re-litigated)

* Support legality: ordinary `sturdy` floor, static check, `audit.py` covers it.
* Per-y column masks: the build is 2 high; the "column claimed for its whole
  height" pessimism costs nothing here, and the lid result (H8.1) means the
  cell above is genuinely free.
* `interferes()`: the measured clearances (lid OK, pitch 2 OK, pitch 1 leaks)
  fall straight out of the existing emission/sensitivity table — dust at 15
  reaching 1 cell away, and W3 for the lid. No new rule needed.

---

## Reproducing

```
cd redstone-eda
python3 probe_hex_transmit.py        # 66/66  — structure, replay, value sweep,
                                    #          latency, pipelining, calibration,
                                    #          chaining, envelope, locking
python3 probe_hex_vs_comparator.py   # 22/22  — the head-to-head table
```

Both probes drive **mc-tick** (`TickSimulation`) via `rs.py`, and both extend
`rs.EXTRA_STATES` with comparator / delay-2..4 repeater / wool / barrel states —
mc-tick binds behaviour to states interned at construction time, so an
un-interned state sits inert and the whole measurement silently reads zero.
