# Vertical programmable encoder: original all-enabled matrix

Source: `sources/logical-redstone-archive/files/Archive/ENCODE_encoders/ENCODE001_Encoder_vertical_to_vertical/attachments/ENCODE001_encode_v2v.schem`

SHA-256: `43141915727b29d5b2bbb5b6f0c8f2afab6f3b0a069349397080e7deb0ce09e4`. Sponge v3, DataVersion4189, size `[6, 10, 13]`, saved paste offset `[-4, -15, -6]`. Original bytes are retained outside Git. No entities; 0 block entities. No sign labels occur in this source. The adjacent archived README states the function; source-inspection.json pins that README hash and records actual controls and lamps.

`selected` is a four-bit mask over real levers, in listed order `[[5, 2, 12], [5, 4, 12], [5, 6, 12], [5, 8, 12]]`. `code` reads the four far-side output lamps, in listed order `[[0, 2, 0], [0, 4, 0], [0, 6, 0], [0, 8, 0]]`; input-adjacent indicator lamps are excluded. All16 original programming crosspoints are populated (explicit positions/descriptors in source-inspection.json). Thus each selected input contributes programmed word1111, and the independent expected output is the bitwise OR of those words: zero for no selected rows,15 otherwise. This deliberately does not pretend that the unmodified template encodes row indices or applies priority. Its original one-hot use is covered; multi-hot OR behavior follows the diode-isolated programming matrix, without fitting to simulated outputs.

All coordinates are schematic-local / production-export-relative. The Sponge offset is a paste offset, not a known original-world redstone origin; the adapter explicitly uses hash_origin `[0,0,0]`. It makes no source-world location or vanilla capture claim.

The adapter pulls existing levers, in declared bit order, and reads existing lamp states. It performs32 setup ticks and32 ticks after each complete input assignment, then requires quiescence. No blocks, items or source states are replaced. These are conservative settle windows, not minimal-latency measurements. Each suite contains513 operations covering all16 inputs and all256 ordered settled transitions. The runner resets and replays the suite and compares its full final checkpoint as well as outputs.

Expected values come from oracle.py's independent OR of the original programmed masks, never from the engine. A native functional pass is not vanilla parity proof, does not prove transient hazards, and does not certify other timing or programming configurations. No benchmarks are run in this validation batch.
