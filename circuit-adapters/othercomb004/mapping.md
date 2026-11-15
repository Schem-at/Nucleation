# 4-bit incrementor

Source: `sources/logical-redstone-archive/files/Archive/OTHERCOMB_other-combinational/OTHERCOMB004_Incrementor/attachments/OTHERCOMB004_incrementor.schem`

SHA-256: `5658936124734a8b7c6bfd144f81bd73ba9f860fe39d51134170dc36f0f3ff1e`. Sponge v3, DataVersion4189, size `[4, 11, 8]`, saved paste offset `[-1, -11, -3]`. Original bytes are retained outside Git. No entities; 8 block entities. No sign labels occur in this source. The adjacent archived README states the function; source-inspection.json pins that README hash and records actual controls and lamps.

Input and output ports are four-bit unsigned values. Input bit i is `[[1, 3, 0], [1, 5, 0], [1, 7, 0], [1, 9, 0]]` in listed LSB-first order; output bit i is `[[1, 1, 7], [1, 3, 7], [1, 5, 7], [1, 7, 7]]`. The source page explicitly describes outputs input plus one, modulo16. The lowest input layer is the start of the upward carry/borrow chain; each output is two blocks below its corresponding input. OTHERCOMB004 additionally saves input0000 with only the bottom output lamp lit, independently corroborating its bit-0 assignment. OTHERCOMB001 saves only its bottom input on with all outputs off. OTHERCOMB003 shares the same upward slice geometry as OTHERCOMB004 without the input-inversion torches.

All coordinates are schematic-local / production-export-relative. The Sponge offset is a paste offset, not a known original-world redstone origin; the adapter explicitly uses hash_origin `[0,0,0]`. It makes no source-world location or vanilla capture claim.

The adapter pulls existing levers, in declared bit order, and reads existing lamp states. It performs32 setup ticks and32 ticks after each complete input assignment, then requires quiescence. No blocks, items or source states are replaced. These are conservative settle windows, not minimal-latency measurements. Each suite contains513 operations covering all16 inputs and all256 ordered settled transitions. The runner resets and replays the suite and compares its full final checkpoint as well as outputs.

Expected values come from oracle.py's independent modulo-16 arithmetic, never from the engine. A native functional pass is not vanilla parity proof, does not prove transient hazards, and does not certify other timing or programming configurations. No benchmarks are run in this validation batch.
