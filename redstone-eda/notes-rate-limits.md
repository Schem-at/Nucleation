# RATE LIMITS — what our verification never measured

Status: probed 2026-08-10, mc-tick. Companion to `notes-vertical-transport.md`,
which found the hole this file walks through.

New code: **`ratedrive.py`** (the at-speed driver + burst/damage protocol),
**`probe_rate_limits.py`** (**5/7** as of 2026-08-10 — the per-circuit sweep;
R0 now passes and the two FAILs are R5/R6, both genuine at-speed findings about
`mult4`, see §5), **`probe_lock_prime.py`** (21/21 — repeater
locking, comparator priming, the hex comb under sustained traffic), and a
burnout term in **`timing.py`** (`burnout`, `min_hold_gt`, `min_period`,
`peak_turnoffs`; `python timing.py` prints the binding constraint per design).

---

## 1. The finding in one paragraph

`probe_torch_burnout.py` showed that a redstone torch driven faster than one
input change per 4 gt goes stuck-until-update, and that the fault is invisible to
a settle-based check. Every NOT gate is a torch and the PLA puts a tap torch and
a gate torch in every column, so **every circuit this directory compiles is made
of the hazard**. The question was therefore never "are we exposed" but "does the
toggle budget bind before propagation delay does". Measured: **it does not, for
any datapath we have built** — every one of them is an order of magnitude slower
than its own torches, so respecting its propagation delay keeps its torches
inside budget automatically. The blind spot is real but its blast radius is
smaller than feared: **what was wrong was never the circuits, it was the
claims** — every exhaustive result was stated without the rate it holds at, and
above that rate two of the six circuits go silently wrong rather than merely
slow.

---

## 2. The per-circuit rate table

`probe_rate_limits.py`. All figures in **game ticks** (2 gt = 1 redstone tick).

* **latency** — worst quasi-static settle observed over the driven sequence
  (a sample, not a bound: see the escalation note below).
* **max rate** — the shortest input hold at which the circuit's **whole case
  list**, driven **word-parallel and back-to-back with no settling**, is correct
  when sampled at the end of each hold. This is the number a consumer needs.
* **failure mode** — what happens when it is over-driven.

| circuit | torches | latency | **max rate** | case list at that rate | failure mode when over-driven |
|---|---|---|---|---|---|
| `not_gate` (1 torch — a NOT gate, the PLA's atom) | 1 | 2 gt | **4 gt** | 16/16 | **silently wrong (persistent)** at 2 gt |
| `rca4_cells` (genlib: repeaters + comparators) | **0** | 60 gt | **96 gt** | 512/512 | slow only |
| `ripple_carry_adder_4bit` (`build_adder.py`, PLA) | 112 | 56 gt | **52 gt** | 512/512 | **silently wrong (persistent)** at 2–3 gt |
| `alu_4bit` (`build_alu.py`, PLA) | 224 | 98 gt | **98 gt** | 2048/2048 | slow only (no damage observed) |
| `kogge_stone_8bit` (`build_ppa.py`, PLA) | 358 | 108 gt | **108 gt** | 64/64 (sampled) | slow only (no damage observed) |
| `mult_4x4` (stacked PLA + routed interconnect) | 590 | 264 gt | **not established** | 251/256 at 262 gt | slow only (no damage observed) — **see §5** |
| `dff` (`seq_cells.py`) | 1 (on CLK) | — | **20 gt period** | 40/40 cycles | wrong bit captured, silently, per cycle |

Sequential rows come from `probe_lock_prime.py` group D; the rest from
`probe_rate_limits.py`.

### Reading the table

* **Two circuits fail SILENTLY WRONG**: the single NOT gate at a 2 gt hold, and
  the PLA ripple adder at 2–3 gt holds. In both cases the traffic leaves the
  structure in a state that a **later, fully settled** input reads incorrectly —
  lost data behind a port that still returns a legal-looking number. Everything
  else observed loses values *in flight* and recovers.
* **The damage lands on the FIRST torch stage.** `build_adder`'s head inverters
  sit directly on the lever rails, so they see every input change and burn. The
  ALU and the Kogge-Stone adder showed no persistent damage at any hold — not
  because they are immune (their input stages are the same torches) but because
  at 2–3 gt the input is filtered out before it reaches most of the tree. Read
  their "slow only" as *not observed with these vectors*, not as *cannot happen*.
* **`max rate` can sit just under `latency`** (adder: 52 vs 56 gt) because the
  output is correct a few ticks before the world is quiescent — `settle` counts
  trailing activity on nets nobody reads.
* **`rca4_cells` is the control group.** Zero torches, so no toggle budget: it
  fails "slow only" at every rate tried, which is what confirms the damage in the
  other rows is the torch and not the harness.

---

## 3. The burnout term in STA (`timing.py`)

Arrival time is half a timing closure. `timing.py` now computes both terms and
names the binding one; existing behaviour (`analyze`, `critical_path`,
`net_repeaters`, `measure`) is unchanged, and `report()` gained lines rather than
losing any.

```
TORCH_MIN_HOLD_GT   = ceil(BURNOUT_WINDOW / (2 * MAX_TURNOFFS)) = 4 gt
TORCH_MIN_PERIOD_GT = 2 * TORCH_MIN_HOLD_GT                     = 8 gt
min_period(ppa, sinks) -> (period, "arrival"|"burnout", detail)
```

The factor of 2 is the part that is easy to get wrong: a torch toggles twice per
cycle of its input and **only the off-going half is charged**, so a 60 gt window
with an 8-turn-off budget buys one input change per 3.75 gt, not per 7.5.

`python timing.py`:

| design | torches | arrival | torch floor | min period | binding |
|---|---|---|---|---|---|
| `kogge_stone_4bit` | 150 | 92 gt | 8 gt | 92 gt | ARRIVAL |
| `kogge_stone_8bit` | 358 | 128 gt | 8 gt | 128 gt | ARRIVAL |
| `kogge_stone_32bit` | 1926 | 280 gt | 8 gt | 280 gt | ARRIVAL |
| `alu_4bit` | 224 | 116 gt | 8 gt | 116 gt | ARRIVAL |
| `alu_8bit` | 492 | 154 gt | 8 gt | 154 gt | ARRIVAL |

**Arrival binds everywhere in this flow**, and it is not close: a PLA column
already costs two torches, and nothing we compile has a critical path near 8 gt.
The term still has to be in the tool, because it binds for exactly the designs we
keep saying we want — a hand-built fast gadget, a pipelined datapath whose
*stage* delay is short, or any clocked design whose period is chosen from the
clock generator rather than from the logic.

### The term is calibrated, not assumed

`timing.peak_turnoffs()` measures the worst per-torch turn-off count in the 60 gt
window following **one** input change. On `kogge_stone_8bit`, driving the
worst-case carry ripple (`255 + 0 + 1`): **max 1 turn-off, at every one of the
358 torches, zero torches above 1**. So a single input change cannot glitch a
torch past its budget, which is the reassuring half of this work: **the
quasi-static exhaustive runs were not secretly burning out**. Pass a measured
`toggles_per_change` to `min_hold_gt()` when a net is known to glitch; the 0.5
default is the clean-square-wave case that `probe_torch_burnout` B1 measured.

---

## 4. The two named suspects, and the hex comb

`probe_lock_prime.py`, 21/21. `notes-vertical-transport.md` asked whether
repeater locking and comparator priming hide the same fault, "both stateful and
both currently verified only by settle". Both were driven by rate. **Both are
clean.**

### Repeater LOCKING — not rate-blind (verdict: claims stand)

| probe | result |
|---|---|
| L1 | the `locked` flag tracks its locker 1:1 at every period **≥ 2 gt** — 2 gt being the locking repeater's own delay. At 1 gt half the lock changes are swallowed *at the input*. No toggle budget exists to exhaust. |
| L2 | after 24 **simultaneous D+LOCK** changes at periods down to 1 gt, releasing the lock always leaves `locked=false` and the latch always resumes tracking D, including on the next fresh change. The flag is re-derived from the locker's power on every evaluation, so **it cannot desync** — there is no stuck-locked state to get stuck in. |
| L3 | **no setup requirement to violate**: D and LOCK toggled inside the *same* game tick still store the new value (the data repeater's own 2 gt supplies the margin). This kills the "word-parallel input change is a zero-setup event" hypothesis. |

So the `seq_probe.py` verdicts hold at any rate the latch is fast enough to
follow, and the sequential library's rate limit is **latency, not state loss**.

### Comparator PRIMING — not rate-blind (verdict: claims stand)

| probe | result |
|---|---|
| C1 | tracks its back input 1:1 at every period **≥ 2 gt** in both `compare` and `subtract`, for 24 back-to-back changes. Only a 1 gt period is swallowed. **No toggle budget.** Note this is *not* the 3 gt figure `notes-hex-transport.md` publishes — 3 gt is a property of the 15-repeater comb, not of the comparator, and neither number generalises to the other. |
| C2 | a comparator used as a *locking driver* behaves exactly as the repeater locker does under the same traffic: no stuck lock, correct resumption at every period down to 1 gt. |
| C3 | after 24 **analog value** changes at periods down to 1 gt, all four settled strengths match the quasi-static truth table. **No stale primed strength survives the input that produced it**, so comparator state needs no recovery protocol. |

### The hex analog comb — separation 3 gt holds under sustained traffic

`notes-hex-transport.md` published *pipeline depth 4 gt, minimum value separation
3 gt* from a **single** transition (H5.8/H5.9). Re-measured as sustained traffic:

| probe | result |
|---|---|
| H1 | single-burst claim reproduced: 1 gt and 2 gt gaps swallowed, 3 gt gets through. |
| H2 | **sustained**: 32 back-to-back injector changes at 3 gt each track **1:1** at the output (0.97, i.e. 31/32 with the last still in flight); 4/6/8 gt likewise. Sub-3 gt fails identically sustained and once — swallowed at the first delay-1 repeater, deterministic input filtering, not device damage. The comb is 15 repeaters + 2 comparators and **contains no torch**. |
| H3 | after 40 injector changes at **1 gt** each — far inside the swallowing regime — the comb settles to the correct analog value again. Over-driving a torch-free carrier loses values in flight and leaves **no residue**. |

**Verdict: the hex bus's published pipeline numbers are rate-safe as stated.**
It is the recommended carrier for fast traffic precisely because it has nothing
that can burn out.

---

## 5. A regression this work uncovered — FIXED, and what it left behind

**Original finding.** `mult4.py` — the 4×4 multiplier whose result is recorded
as **256/256** — scored **64/256 quasi-statically**, under its own published
protocol. Output bit 3 (`m3s0`) was stuck at 1, so every `A=0` case read 8. It
was not a rate finding: `python mult4.py` unmodified reproduced it, so it had
regressed in committed code.

**Resolved 2026-08-10.** The cause was a ring latch, not a rate effect. Raising
the router's refresh pitch (`REFRESH` 5 → 14, commit `5076520c`) re-pathed net
`m1B2` along the rail it was supposed to DRIVE; a compiled rail is fed through a
repeater, so the route closed a directed cycle containing a diode, and a ring
holding a repeater latches at 15. `router.downstream_rail` refuses that contact
now. **`python mult4.py` scores 256/256 again**, and the general rule is checked
statically by `drc.repeater_cycles`, which every builder in this directory runs
(see `drc.py` and `test_diode_ring.py`).

**So R0 now PASSES** — the earlier "R0 fails by design until the multiplier is
fixed" remark is obsolete, and the rate sweep no longer skips `mult_4x4`.

**But the sweep now reports 5/7, not 6/7.** Admitting `mult4` to the rate table
moved the failures from R0 to **R5 and R6**, and both are real:

* `mult4` reproduces only **251/256** of its case list at the hold the sweep
  chose for it (**262 gt**), while every other circuit reproduces its full list.
  So R5's claim — that the exhaustive results survive with a rate attached —
  does not hold for this circuit.
* That hold is **below `mult4`'s own measured latency of 264 gt**, which is why
  it cannot pass: the sweep picks the first hold at which the sampled,
  worst-activity drive reads clean (262 gt), and for `mult4` the escalation to
  the real case list did not then push past its latency. R6 fails on exactly
  the effect R6 itself describes — the sample under-estimates.

**`mult4`'s maximum input rate is therefore still unestablished.** The circuit
is functionally correct (256/256 quasi-static) and its failure mode over-driven
is "slow only" — no persistent damage — but the 262 gt figure in the table
above must not be read as a certified rate. Escalating the sweep past a
circuit's measured latency is the fix, and is the first item below.

---

## 6. Methodology: what settle-based verification actually proves

This is the part worth more than any individual number.

### What the old protocol proves, exactly

`rs.Levers.set()` flips **one** lever, runs `run_until_quiescent`, and repeats.
An exhaustive pass under that protocol is a valid, complete proof of one thing:

> the circuit computes the right function **in the quasi-static regime** —
> unbounded time between input changes, and one input bit changing at a time.

That is a real result and it is not weakened by anything here. It is also not
what a datapath does, and it is silent about three separate things:

| hazard | what a settle protocol cannot see | who it bit |
|---|---|---|
| **LATENCY** | sampling before arrival | everything: max rate is 52–108 gt for our 4- to 8-bit datapaths |
| **SIMULTANEITY** | transients from a whole word changing in one tick | nothing, as measured — but only because it was finally measured |
| **BURNOUT** | a stuck torch: internally corrupt, port plausible, next answer wrong, settling repairs nothing | the NOT gate; `build_adder` at 2–3 gt holds; every torch ladder |

Burnout is the malign one because settling cannot detect it *even in principle*:
the world is already quiescent, so `run_until_quiescent` advances 0 ticks and
re-evaluates nothing.

### What a rate-aware harness must do

Eight requirements, each of which cost a wrong measurement to learn:

1. **Establish the quasi-static baseline first.** Reproduce the circuit's own
   published protocol on a sample before claiming anything about rate. Skipping
   this means happily reporting rate numbers for an already-broken circuit —
   which is exactly what happened with `mult_4x4` until the check was added.
2. **Drive word-parallel with an explicit hold, and never settle inside the
   sequence.** One lever at a time with a settle between *is* the old protocol.
3. **Sample at the end of each hold**, not after settling. The metric is
   "correct when the consumer looks", not "correct eventually".
4. **Separate the three failure classes.** "Wrong" is not one thing: late,
   swallowed-at-the-input, and damaged need different fixes and only the last is
   dangerous.
5. **Test damage with a cooled-down, spaced canary.** Four settled flips of a
   torch inverter take ~10 gt in total, which is over budget on its own — a naive
   canary phase reports "damaged" at *every* rate, including safe ones. Wait out
   a full burnout window first, and space the canaries. (First version of this
   harness got this wrong and produced a beautifully alarming table of noise.)
6. **Also test RESUME**: one settled operation immediately after the traffic
   stops, inside the window the traffic exhausted. That is the fault a user
   actually meets, and one change cannot exhaust a budget by itself.
7. **Take a fresh sim per rate.** Burnout is stuck-until-update, so a rig damaged
   at one hold carries the damage into every later measurement and the sweep
   reports the first rate it happened to be tested at.
8. **Escalate against the real case list.** Worst-case **activity** is not
   worst-case **latency**: for `rca4_cells` the all-ones/all-zeros pair is one of
   the *faster* transitions, and a sweep driven by extremes certified 84 gt while
   the full 512-case list still failed there (480/512). It needed 96 gt.

A ninth, implicit: a wrong output in a **quiescent** world is categorically worse
than a wrong output with updates pending. `ratedrive.burst` records
`drain_quiet`/`stuck_wrong` for that reason, and it is what promoted the PLA
adder from "slow only" to "silently wrong".

### Which claims need narrowing, and which survive

Precisely — an exhaustive settle-based pass is still valid *for its regime*, so
the damage should not be overstated in either direction.

**Survive unchanged, and are now stronger** — re-established at speed,
word-parallel and back-to-back, at the stated hold:

* `rca4_cells` 512/512 — now also 512/512 at a 96 gt hold.
* `build_adder` 4-bit 512/512 — now also 512/512 at a 52 gt hold.
* `build_alu` 4-bit 2048/2048 — now also 2048/2048 at a 98 gt hold.
* `kogge_stone_8bit` — sampled 64/64 at a 108 gt hold.
* `seq_probe.py` repeater-lock semantics — hold at any rate (L1–L3).
* `notes-hex-transport.md` separation 3 gt / depth 4 gt — hold under sustained
  traffic (H2, H3).
* `seq_README.md` DFF min period 20 gt — **confirmed sustained**: 40 consecutive
  random-D cycles, 800 gt of continuous clocking, 13× the burnout window, all
  captured. The 10-cycle sweep that produced the number was not measuring a
  transient.

**Need a rate attached — the unqualified reading was never established:** all of
the above. Every one of those results should be quoted as "*N/N* at a hold of
*H* gt or slower". Above *H*, the same circuits are wrong, and two of them are
wrong *silently*.

**Narrower than stated:**

* `probe_vertical_forms.py` L4, "8 torch ladders verified over all 256 patterns"
  — already flagged in `notes-vertical-transport.md`; measured only the regime in
  which burnout cannot appear. The form is disqualified for unbounded traffic.
* `alu_8bit` / `kogge_stone_32bit` — verified quasi-statically only; their
  arrival-based min periods (154 gt, 280 gt) are STA predictions, not measured
  max rates. Not re-verified at speed here (cost).
* `seq_counter.py` counter4 min period 100 gt — a 16-step sweep. The DFF's
  20 gt survived a sustained run; the counter's 100 gt was not re-tested.

**Re-established since:** `mult_4x4` is back to 256/256 quasi-statically (§5).
Its **max rate** is still unestablished, though — the sweep's chosen hold
(262 gt) sits below the circuit's own 264 gt latency and only reproduces
251/256.

**Explicitly *not* damaged by any of this:** the correctness of the compiler, the
PLA architecture, or the cell library. No circuit was found to be wrong at a rate
it was ever claimed to work at. The defect was in the shape of the claim.

---

## CARRIED OUT OF THIS WORK (open)

* ~~**Fix `mult_4x4`**~~ — DONE (§5): a ring latch from `5076520c`'s refresh
  pitch, fixed in `router.downstream_rail` and now caught statically for every
  builder by `drc.repeater_cycles`. 256/256 again.
* **Escalate the rate sweep past a circuit's measured latency** (§5) — the
  reason `probe_rate_limits.py` is 5/7 rather than 7/7. It selects `mult_4x4`'s
  hold at 262 gt, below that circuit's own 264 gt latency, so R5/R6 fail on a
  hold nothing could have met. Clamp the candidate holds to ≥ latency, or keep
  escalating while the full case list is short.
* **`max_toggle` on `TRANSPORT_MODEL.md` row 7** — still open from
  `notes-vertical-transport.md`; this file supplies the number (4 gt hold /
  8 gt period) and now `timing.py` supplies the API.
* **Re-verify `alu_8bit` and `kogge_stone_32bit` at speed.** The harness handles
  them (`rig_ppa(32)`); it is only runtime. Their STA min periods are untested
  predictions.
* **Re-measure `counter4`'s 100 gt min period sustained**, the way the DFF's
  20 gt was. A feedback loop is where a dropped bit becomes a wrong *count*.
* **A `max_rate` field on every emitted artifact.** A `.schem` that carries
  "512/512 correct" and not "at ≤ 52 gt per input" is a trap for its next
  consumer. The number now exists for every circuit in §2 and nothing records it.
* **Deeper-stage burnout.** The ALU and KS adder showed no persistent damage, but
  only because 2–3 gt traffic never reaches their inner torches. A rig that
  drives an *inner* net at rate (via a routed test point) would say whether depth
  protects or merely hides.
