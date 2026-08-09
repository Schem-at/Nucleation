# What computational redstone wants from Nucleation core

Evidence base: a 4-bit ripple-carry adder (512/512 exhaustive), a 32-bit
Kogge-Stone prefix adder (153k blocks, per-node verified), and an N-bit 4-op
ALU, all compiled from netlists to geometry by ~800 lines of Python driving
only `set_block_from_string` + `TickSimulation`. Every item below is something
that exercise needed and had to build or discover the hard way.

## 1. Structural validation (highest value, smallest API)

**`schematic.redstone_audit() -> json`** — two checks that caught every
show-stopper before simulation did:

- *support audit*: dust/repeater/torch/lever with no solid block where it
  attaches. The tick engine happily simulates floating wire, so a build can
  verify 512/512 and still drop 84 wires when pasted (that happened).
- *net shorts*: union-find over the dust connectivity rules (4 horizontal
  neighbours + up/down diagonals, up-diagonal cut by a solid block above the
  lower dust). Caller supplies per-position net labels; the check reports any
  component containing two labels. Both bugs the 32-bit build shipped with
  were unintended adjacency; simulation says "wrong somewhere", this says
  "these two wires touch at (x,y,z)".

Rust already has the connectivity rules inside mchprs/mc-tick; exposing them
as a static analysis costs little and is exactly what the sim cannot say.

## 2. Bake-settled-states (`{simulate=true}` in bulk)

`{simulate=true}` exists in root `src/` but not in the shipped wheel
(`bindings/python/rust` is a stale vendored copy — re-vendor). Per-placement
it also re-settles the whole world per block; for a 75k-wire build that is
unusable. Propose **`TickSimulation.bake_to(schematic)`**: after the world
settles, write every changed block state back. One pass turns "all wires are
disconnected dots at power 0" into a file whose palette carries real
connections and power, loads quiescent under `InWorld` in 0 ticks, and renders
correctly in any static consumer. (Implemented in Python as `bake()`; it is a
loop over `get_block` — core could do it without string round-trips.)

## 3. Diagnostics that surfaced only by luck

- `TickSimulation.from_schematic` errors are a bare `InvalidArgument`;
  `last_error_detail()` had the real story ("19.2M cells over the 8M cap —
  looks like a saved world"). Attach the detail to the exception message.
- The volume cap measures the **allocated region**, not content bounds. A
  1.9M-cell build refused to load because its region was padded to 19.2M.
  Either tighten regions on load or say "allocated" in the error.
- Bare `minecraft:redstone_wire` (no properties) interns a property-less state
  that never ticks. Either normalise to the default state on set, or warn.
  Symptom today: wire that reads back with no `power=` key, silently inert.

## 4. Lever/IO driving

`set_lever_power` sets the reported value but schedules nothing downstream —
every real test drives inputs by `use_block` toggle-to-target with tracked
state, settling between flips (multiple same-tick flips let ripple chains
latch transients). That idiom (`Levers` in rs.py, ~20 lines) belongs next to
`TypedCircuitExecutor`, or as a documented footgun on `set_lever_power`.

## 5. Static timing analysis

`timing.py`: torch = 1 redstone tick, repeater = its delay, dust free;
arrival = max over taps + per-net repeater counts from geometry. Gives a
critical path ("cin -> g0 -> G1_1 -> ... -> out31 @ 44 rt") without running
the sim, and a measured cross-check via `tick_count()` deltas. Cheap to port,
pairs naturally with `RedstoneGraph` (the compile graph already knows the
component types; it could do this exactly instead of as an upper bound).

## 6. Shorts are static, opens are not — and what would close the gap

The stacked-multiplier exercise sharpened the checker story.  Three broken
builds in a row passed the net-short check AND settled quiescent, because
every fault was an **open**, not a short: a torch base fed by dust that did
not point into it, and a 15-cell repeater-free staircase decaying to zero.  A
dead circuit is a perfectly quiescent circuit.

Statically provable today: support, shorts, torch-base *pointing* (dust shape
is derivable from neighbours), repeater-free span length (decay budget).  The
last two are one-page additions to the audit and caught-by-construction in
the router now (`rules: max 4 stairs, refresh every 5`).  What genuinely
needs the engine is end-to-end continuity — and there `RedstoneGraph`
(the mchprs compile graph) could answer "is there a conduction path from A to
B" without a tick of simulation.  **`schematic.redstone_connectivity(a, b)`**
would have turned three sim-sweep debugging rounds into three instant
static failures.

## 7. Verified building blocks worth shipping as data

The torch ladder (1x1 vertical link, 2 blocks/torch, inverts per torch, cap
exits a fresh strong 15, no crosstalk at 2-block spacing, 1 rt/torch) and the
PLA column are now sim-verified TEMPLATES with known delay and clearance
behaviour.  A `redstone_primitives` module of such stamps — verified once in
CI against mc-tick, with metadata (footprint, delay, entry/exit contracts) —
is the difference between "a library that can simulate redstone" and "a
library you can build computers with".

## 8. What did NOT need core support

Placement/routing (interval-coloured rail tracks, per-slice corridor
allocation, riser trunks) stayed comfortably in Python and is design-specific;
it does not belong in core. What it needed from core was only: fast block
writes, a truthful simulator, and the checkers above.

## Numbers that motivate the sim-side items

| build | blocks | wires | verification |
|---|---|---|---|
| RCA 4-bit | 3,362 | ~1,200 | 512/512 exhaustive + lamps |
| Kogge-Stone 32-bit | 153,512 | ~72,000 | 47 vectors x ~700 internal nodes |
| ALU (4-bit / 8-bit) | 12,764 / ~28k | — | exhaustive 2,048 / directed+random |

A verification sweep is thousands of `use_block`+`run_until_quiescent` cycles;
everything that reduces per-cycle Python/FFI chatter (batched probe reads —
e.g. `get_blocks_json` for a probe list — would be the next ask) multiplies
directly into test throughput.
