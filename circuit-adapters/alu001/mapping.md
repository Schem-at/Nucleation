# ALU001: eight-bit arithmetic/logic unit

The typed interface is `a: u8`, `b: u8`, `operation: u11` → `out: u8`.
`operation` is a **one-hot mask**, not an integer opcode: select one of the
11 actual levers with `1 << index`. This v1 profile does not enforce
one-hot selection itself; zero or multiple selected operations are outside
the functional contract tested here.

## Source and static port evidence

- [Logical Redstone Archive source](https://github.com/mattbatwings/Logical-Redstone-Archive/tree/6d054aa0a87d8f58f672625b3e6d0e727bd97016/Archive/ALU_alus/ALU001_8_bit_ALU_CCA-based)
- Local archive file: `sources/logical-redstone-archive/files/Archive/ALU_alus/ALU001_8_bit_ALU_CCA-based/attachments/ALU001_alu_cca.schem`
- SHA-256: `4fa213c46c4759a761f8473e21079bfcd487b1decb0e4df53ec6b96d05fa7d96`
- Original DataVersion: **4189**; dimensions **33 × 21 × 22**.

The source README says two eight-bit inputs, eleven operation options,
and one eight-bit output. Actual signs and control wiring were inspected
before the suite was run. Mappings were not selected by trying simulator
output permutations.

| Port | Positions, least-significant first | Static evidence |
|---|---|---|
| `a[i]` | `(0,4+2i,21)`, i=0..7 | Wall levers feed the x=0 bank and its input-inversion comparator pair x=1..2, z=16. The sign-labelled **Invert A** bus at z=13 runs along y=1 to x=1 and z=19, then into this bank. |
| `b[i]` | `(8,4+2i,21)`, i=0..7 | Wall levers feed the x=8 bank and its inversion pair x=6..7, z=16. **Invert B** at z=15 routes along y=1 to x=7 and z=19 into this bank. |
| `out[i]` | `(9,4+2i,0)`, i=0..7 | Eight lamps on the opposite side, connected to the stacked arithmetic/output slices. The carry-input control injects below the lowest slice through `(5,2,9)`; successive slices rise by two blocks. |
| `operation[i]` | `(12+2i,3,18)`, i=0..10 | Actual selector levers; signs above their indicator lamps at `(12+2i,4,17)` name each operation. |

The input-indicator lamps at z=20 and selector-indicator lamps at z=17 are
not used as outputs. Original saved A=5, B=3 and selected AND is consistent
with the saved output bit 0; this is a secondary file-consistency check,
not the independent arithmetic oracle.

## Operation contract

All formulas below return their low eight bits. Carry/borrow output is not
exposed by this source profile.

| Index | Mask | Exact source sign | Independent oracle |
|---|---:|---|---|
| 0 | 1 | Add | `(a+b) & 255` |
| 1 | 2 | Sub | `(a-b) & 255` |
| 2 | 4 | AND | `a & b` |
| 3 | 8 | NAND | `~(a & b) & 255` |
| 4 | 16 | OR | `a | b` |
| 5 | 32 | NOR | `~(a | b) & 255` |
| 6 | 64 | XOR | `a ^ b` |
| 7 | 128 | XNOR | `~(a ^ b) & 255` |
| 8 | 256 | A IMPLY B | `(~a | b) & 255` |
| 9 | 512 | A NIMPLY B | `(a & ~b) & 255` |
| 10 | 1024 | Right-Shift | `a >> 1`, **requires b=0** |

The subtraction selector's torch ROM connects to **Carry In** (z=9) and
**Invert B** (z=15), independently confirming two's-complement `a-b`.
The implication selector connects to **OR** (z=11) and **Invert A**
(z=13), independently confirming operand direction. The right-shift
selector only activates the output shift control at z=5, so this adapter
restricts the other operand to zero and does not claim arbitrary two-input
shift semantics.

## Protocol, coverage and result

Inputs are driven in order `a`, `b`, `operation`, each bank from low to high
bit, before the next game tick. Selector changes can temporarily leave
zero/two selectors set during the synchronous lever calls; the suite
checks the endpoint after all input calls and 64 game ticks. Initial settle
is 128 game ticks. Every endpoint must be quiescent. RNG seed/hash origin
are zero, random ticks and recording disabled. No minimum latency or
transient-glitch behavior is claimed.

`oracle.py` generates **1,017 vectors**:

- 75 operand pairs × ten binary operations, interleaved to exercise mode
  changes: boundary cases, walking bits/carry boundaries, and 32
  deterministic pseudorandom pairs.
- All 256 values of `a` for right-shift, with `b=0`.
- All eleven modes at zero inputs in reverse operation order.

`result.json` records a **functional pass on harrison-work**, with 1,017
output assertions per run. The whole suite is repeated after checkpoint
reset and must match every output and the exact final checkpoint: 2,034
output assertions total. The receipt pins source/profile/suite/runner
hashes and has no timing samples. This is engine endpoint functionality,
not a fresh vanilla comparison. Binary operand coverage is not exhaustive.

```sh
target/release/examples/run_tick_adapter circuit-adapters/alu001/adapter.json "$ARCHIVE/sources/logical-redstone-archive/files/Archive/ALU_alus/ALU001_8_bit_ALU_CCA-based/attachments/ALU001_alu_cca.schem" circuit-adapters/alu001/suite.json
```
