# ADD008: one-bit ripple carry full adder

The typed interface is `a: bool`, `b: bool`, `cin: bool` → `sum: bool`,
`cout: bool`. The independent oracle counts the three asserted inputs:
`sum = bool(total & 1)`, `cout = total >= 2`.

## Source and static port evidence

- [Logical Redstone Archive source](https://github.com/mattbatwings/Logical-Redstone-Archive/tree/6d054aa0a87d8f58f672625b3e6d0e727bd97016/Archive/ADD_adders-subtractors/ADD008_Basic_Ripple_Carry_Adder_RCA)
- Local archive file: `sources/logical-redstone-archive/files/Archive/ADD_adders-subtractors/ADD008_Basic_Ripple_Carry_Adder_RCA/attachments/ADD008_rca.schem`
- SHA-256: `c18b62e0010cf03f5a5c0bd81d79197ba90a0666bf8411af91a12f02249ef0e9`
- Original DataVersion: **4189**; dimensions **13 × 4 × 7**.

The source README specifies `A, B, Cin` → `Sum, Cout` and describes a
stackable full-adder slice. The save has no signs or block entities. Ports
were traced from the actual wool, dust and torch layout before simulation.

| Port | Position | Static evidence |
|---|---|---|
| `a` | `(0,2,2)` | Floor lever over an input lamp; lime-supported dust enters the first XOR stage. |
| `b` | `(0,2,4)` | Parallel floor lever; light-blue-supported dust enters the same stage. A/B are symmetric. |
| `cin` | `(6,2,6)` | Third floor lever; red-supported dust at `(6,2,5)` and `(6,2,4)` enters the second XOR stage. |
| `sum` | `(12,1,3)` | Far lamp reached by magenta-supported dust `(10,1,3)` → `(11,1,3)`, following the second XOR's wall torches `(10,1,2)` and `(10,1,4)`. |
| `cout` | `(6,1,0)` | The other non-input lamp is reached through `(6,2,1)` from the carry branch above the first XOR and the second-stage carry connection. |

The first XOR is the orange torch network around x=3..5; its output feeds
the second network around x=8..10 together with `cin`. The north carry
branch exits separately. Lamps at `(0,1,2)`, `(0,1,4)` and `(6,1,6)` are
under the input levers and are excluded from observations. The source is
used without edits or added components.

## Protocol, coverage and result

Drive `a`, then `b`, then `cin`, toggling changed levers before the next
tick. Settle for 128 game ticks initially and 32 game ticks after each
operation. Require scheduler quiescence at every sampled endpoint. RNG
seed/hash origin are zero; random ticking and recording are disabled.
The response budget is conservative, not a latency measurement.

`oracle.py` produces **136 vectors**: the eight input combinations, then
each of the 64 directed source-to-target transitions (including unchanged
inputs), represented by two consecutive endpoints. This exhausts the
Boolean truth table and directed transitions under this fixed input order;
it does not exhaust arbitrary within-tick interleavings or glitch timing.

`result.json` records a **functional pass on harrison-work**, with 272 named
output assertions per run and exact final-checkpoint/output agreement on
reset replay. Both runs together check 544 outputs. Source/profile/suite/
runner hashes are saved. No benchmark timing was requested, and no fresh
vanilla comparison is claimed.

```sh
target/release/examples/run_tick_adapter circuit-adapters/add008/adapter.json "$ARCHIVE/sources/logical-redstone-archive/files/Archive/ADD_adders-subtractors/ADD008_Basic_Ripple_Carry_Adder_RCA/attachments/ADD008_rca.schem" circuit-adapters/add008/suite.json
```
