# Active stationary logic workloads

The shared [benchmark harness](https://github.com/Schem-at/Nucleation/blob/v0.12.3/crates/mc-tick/examples/bench_circuits.rs)
adds `adder_active` and `logic_network`. Their inputs, generated fixture and
assertions live in [logic.rs](https://github.com/Schem-at/Nucleation/blob/v0.12.3/crates/mc-tick/examples/bench_circuits/logic.rs).
Existing workload loaders, settling, input schedules and verification case lists
remain unchanged. `--verify --logic` runs the two additions separately from
`--verify --expanded`.

Both cases passed the laptop checks against current master and the standard/PGO
configurations tested in this round. The saved [master verification](validation/master/reference-logic-verify.jsonl)
and [candidate audit](validation/candidate-trace-audit.json) retain the comparison
evidence. These are arithmetic/signal checks and engine differential traces;
they are **not fresh Minecraft vanilla captures** or exhaustive circuit proofs.

## Active 32-bit adder

`adder_active` uses the existing `adder32.snbt` with Placement settling. Preparation
sets both operands and carry-in to zero, runs 200 ticks and asserts quiescence.
The measured iteration then runs eight 96-tick phases, **768 ticks** in total.
Phase zero is an unchanged zero-input control; subsequent phases drive the adder.

For bit `i` from 0 to 31, A is at `(13,3+2*i,0)`, B at `(13,3+2*i,4)`, and the
sum output repeater at `(0,3+2*i,2)`. Carry-in is at `(13,1,0)`. At the beginning
of each phase, inputs are applied A then B for each bit, low bit first, followed
by carry-in. True places a redstone block; false places light-gray wool, retaining
the carry-in sign's support. An already-correct input is left alone.

| Phase | A | B | Carry-in | Expected sum modulo 2³² |
|---|---|---|---:|---|
| 0 | `00000000` | `00000000` | 0 | `00000000` |
| 1 | `00000001` | `00000000` | 0 | `00000001` |
| 2 | `00000000` | `00000001` | 0 | `00000001` |
| 3 | `ffffffff` | `00000000` | 1 | `00000000` |
| 4 | `55555555` | `aaaaaaaa` | 0 | `ffffffff` |
| 5 | `12345678` | `87654321` | 1 | `9999999a` |
| 6 | `80000000` | `80000000` | 0 | `00000000` |
| 7 | `ffffffff` | `00000001` | 0 | `00000000` |

After timing, the ordinary change log reconstructs all 32 output repeater states
at each phase end and checks the wrapping sum. Carry-out is not asserted. Every
sum bit must have changed during the iteration, and comparator state changes must
have occurred; toggling input blocks alone cannot satisfy the activity checks.

## Generated comparator/repeater network

`logic_network` generates an authored `8×2×36` SNBT fixture inside the helper,
with four separated lanes on a stone floor. No external fixture is needed. It
uses Placement settling, runs 40 preparation ticks and asserts quiescence.
The measured iteration contains sixteen 24-tick phases, **384 ticks** in total.

Lane `l` has center `z=4+10*l`, input A at `(0,1,z)` and input B at `(4,1,z-4)`.
At phase `p`, its inputs are entry `(p+l) mod 4` of the Gray-code sequence
`(0,0), (1,0), (1,1), (0,1)`, applied A then B with the same block-driving helper.
The input repeater has delay `l+1`; the output repeater has delay `4-l`.

Two wire cells after A's repeater supply comparator rear strength 14 when A is
true. Three wire cells after B supply side strength 13 when B is true. Lanes 0
and 2 compare; lanes 1 and 3 subtract. For rear `R` and side `S`, the expected
output is `R` when `R >= S`, otherwise zero, in compare mode; subtract mode uses
`max(R-S,0)`. This exercises exact strengths 0, 1 and 14.

At every phase end, validation reconstructs output wire `(5,1,z)` and checks its
exact strength, then checks that lamp `(7,1,z)` is lit exactly when the strength
is positive. Each lane's lamp must change at least six times over the iteration.
These checks include the downstream output repeater and lamp delay.

## Validation and measurement boundaries

Preparation and final validation assert no moving blocks, item entities,
minecarts or generic entity bodies. Entity and inventory event logs must be
empty. Current blocks and both sides of every recorded block transition must
contain no piston states. Unknown block reports must remain empty.

Both timed cases use ordinary block/inventory/entity recording. Timers include
input application, simulation stepping and recording. Parsing, construction,
settling, checkpoint restoration, RNG reset to zero, recording setup, validation,
fingerprinting and destruction are outside the timers. Each subprocess runs one
unreported warmup sample with the same iteration count as its measured samples.
Elapsed and thread CPU nanoseconds are accumulated across each sample's iterations.

Verification additionally records ordered update deliveries. After every tick it
hashes the tick number, non-air positions and canonical descriptors in storage
order, pending scheduled ticks/events, and the comparator output map sorted by
position. Live comparator outputs matter because their strength can change
without changing the powered block state. Ordered block, update, inventory and
entity records are added to the final fingerprint. The ordinary logs reconstruct
phase-end states using changes whose tick is strictly below the phase-end tick
count; validation performs no extra observation inside the timed loop.

From the repository root, using the built binary under comparison:

```sh
target/release/examples/bench_circuits --verify --logic
target/release/examples/bench_circuits --verify-case adder_active
target/release/examples/bench_circuits adder_active 1 3
target/release/examples/bench_circuits logic_network 1 3
```

For the isolated master checkout at `ee89008d`, copy the identical main harness
and both helper modules (`fuzz.rs`, `logic.rs`), add the `cpu-time = "1.0.0"` dev
dependency with its matching lock entries, and build with `--locked`. Add
`--cfg bench_master --check-cfg=cfg(bench_master)` to that build's Rust flags;
otherwise keep compiler, dependency versions and comparison flags identical.
This conditional excludes `hipster_unrecorded` from master verification and
rejects direct timing of it, because master has no API to disable ordinary
recording. All shared workloads keep the same harness path. No production engine
code or recording API is backported for the comparison.
