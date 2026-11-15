# ADD003: 8-bit carry cancel adder

This profile exposes `a: u8`, `b: u8`, `cin: bool` → `sum: u8`,
`cout: bool`. The oracle computes `total = a + b + cin`, returns the low
eight bits, and tests `total >= 256` for carry. It does not read simulation
results when generating expectations.

## Source and static port evidence

- [Logical Redstone Archive source](https://github.com/mattbatwings/Logical-Redstone-Archive/tree/6d054aa0a87d8f58f672625b3e6d0e727bd97016/Archive/ADD_adders-subtractors/ADD003_Fast_8-bit_Carry_Cancel_Adder_CCA)
- Local archive file: `sources/logical-redstone-archive/files/Archive/ADD_adders-subtractors/ADD003_Fast_8-bit_Carry_Cancel_Adder_CCA/attachments/ADD003_8bit_cca_3t.schem`
- SHA-256: `9046ececdca2d7fd481d2f70a679af6e2c4fe1c13bc87ad66cdbdf950dd4faa8`
- Original DataVersion: **4189**; dimensions **10 × 20 × 6**.

Coordinates below use the production export's bounding-box-relative frame;
`i = 0..7` means least-significant bit first. The original NBT was inspected
before running the suite; no ports were fitted to simulator responses.

| Port | Position | Static evidence |
|---|---|---|
| `a[i]` | `(0, 2+2i, 2)` | Actual west-facing wall lever; sign at `(0, 2+2i, 3)` labels 1, 2, 4, 8, 16, 32, 64, 128. |
| `b[i]` | `(0, 2+2i, 5)` | Parallel bank of actual wall levers at the same sign-labelled heights. Naming the two symmetric addends A/B does not affect the arithmetic. |
| `cin` | `(6, 1, 4)` | The seventeenth lever is attached to `(6,1,3)`, below dust `(6,2,3)` leading to the lowest carry comparator `(7,2,3)`. It is separate from both eight-bit banks. |
| `sum[i]` | `(8, 1+2i, 1)` | Output lamps have adjacent signs `(9,1+2i,1)` labelled 1 through 128. |
| `cout` | `(8, 18, 1)` | Upper lamp below the sign `(8,19,1)` reading `COUT` / `(4 ticks)`. |

The 16 lamps immediately behind the input levers are input indicators and
are not observed as outputs. No inventory population or schematic edit is
needed.

## Protocol, coverage and result

All levers are driven in order `a`, `b`, `cin`, with bits in increasing
significance. Changed levers are toggled before the next game tick. The
adapter settles for 128 game ticks initially and samples after 32 game
ticks per operation. It requires a quiescent scheduler at each endpoint.
RNG seed and hash origin are zero; random ticking and ordinary recording
are disabled. These generous endpoint budgets are not minimum latency or
source speed claims.

`oracle.py` generates **372 vectors**: arithmetic boundaries, walking bits,
carry chains, zero transitions, and 128 deterministic pseudorandom operand
pairs, with both carry-input values. This is not exhaustive over all
131,072 possible inputs.

`result.json` records a **functional pass on harrison-work**: 744 named
output assertions per run and exact output/final-checkpoint agreement after
reset and full replay. Two runs therefore checked 1,488 named outputs. The
receipt contains source, profile, suite and runner hashes. It contains no
timing samples. This establishes Nucleation endpoint functionality for the
documented protocol; it is not a fresh vanilla comparison or a claim about
intermediate glitches.

Reproduce from the repository root, substituting the archive location:

```sh
target/release/examples/run_tick_adapter circuit-adapters/add003/adapter.json "$ARCHIVE/sources/logical-redstone-archive/files/Archive/ADD_adders-subtractors/ADD003_Fast_8-bit_Carry_Cancel_Adder_CCA/attachments/ADD003_8bit_cca_3t.schem" circuit-adapters/add003/suite.json
```
