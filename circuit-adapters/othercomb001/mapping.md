# 4-bit decrementor

Source: `sources/logical-redstone-archive/files/Archive/OTHERCOMB_other-combinational/OTHERCOMB001_Decrementor/attachments/OTHERCOMB001_decrementor.schem`

SHA-256: `298c115dba3df03002a07a1c4e3e5a22e1237cdeca1d2057ac2ede95649ea9e8`. Sponge v3, DataVersion4189, size `[7, 9, 4]`, saved paste offset `[-3, -10, -1]`. Original bytes are retained outside Git. No entities; 8 block entities. No sign labels occur in this source. The adjacent archived README states the function; source-inspection.json pins that README hash and records actual controls and lamps.

Input and output ports are four-bit unsigned values. Input bit i is `[[0, 2, 2], [0, 4, 2], [0, 6, 2], [0, 8, 2]]` in listed LSB-first order; output bit i is `[[6, 0, 2], [6, 2, 2], [6, 4, 2], [6, 6, 2]]`. The source page explicitly describes outputs input minus one, modulo16. The lowest input layer is the start of the upward carry/borrow chain; each output is two blocks below its corresponding input. OTHERCOMB004 additionally saves input0000 with only the bottom output lamp lit, independently corroborating its bit-0 assignment. OTHERCOMB001 saves only its bottom input on with all outputs off. OTHERCOMB003 shares the same upward slice geometry as OTHERCOMB004 without the input-inversion torches.

All coordinates are schematic-local / production-export-relative. The Sponge offset is a paste offset, not a known original-world redstone origin; the adapter explicitly uses hash_origin `[0,0,0]`. It makes no source-world location or vanilla capture claim.

The adapter pulls existing levers, in declared bit order, and reads existing lamp states. It performs32 setup ticks and32 ticks after each complete input assignment, then requires quiescence. No blocks, items or source states are replaced. These are conservative settle windows, not minimal-latency measurements. Each suite contains513 operations covering all16 inputs and all256 ordered settled transitions. The runner resets and replays the suite and compares its full final checkpoint as well as outputs.

Expected values come from oracle.py's independent modulo-16 arithmetic, never from the engine. A native functional pass is not vanilla parity proof, does not prove transient hazards, and does not certify other timing or programming configurations. No benchmarks are run in this validation batch.
