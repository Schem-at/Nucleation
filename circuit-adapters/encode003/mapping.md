# Horizontal programmable encoder: original all-enabled matrix

Source: `sources/logical-redstone-archive/files/Archive/ENCODE_encoders/ENCODE003_Encoder_horizontal_to_horizontal/attachments/ENCODE003_encode_h2h.schem`

SHA-256: `5cd4e0fe2b8eab183a5d5d925ffc4d2703f6796d8c055799577e9093f4b7505f`. Sponge v3, DataVersion4189, size `[11, 4, 10]`, saved paste offset `[-5, -5, -5]`. Original bytes are retained outside Git. No entities; 0 block entities. No sign labels occur in this source. The adjacent archived README states the function; source-inspection.json pins that README hash and records actual controls and lamps.

`selected` is a four-bit mask over real levers, in listed order `[[10, 3, 3], [10, 3, 5], [10, 3, 7], [10, 3, 9]]`. `code` reads the four far-side output lamps, in listed order `[[0, 1, 0], [2, 1, 0], [4, 1, 0], [6, 1, 0]]`; input-adjacent indicator lamps are excluded. All16 original programming crosspoints are populated (explicit positions/descriptors in source-inspection.json). Thus each selected input contributes programmed word1111, and the independent expected output is the bitwise OR of those words: zero for no selected rows,15 otherwise. This deliberately does not pretend that the unmodified template encodes row indices or applies priority. Its original one-hot use is covered; multi-hot OR behavior follows the diode-isolated programming matrix, without fitting to simulated outputs.

All coordinates are schematic-local / production-export-relative. The Sponge offset is a paste offset, not a known original-world redstone origin; the adapter explicitly uses hash_origin `[0,0,0]`. It makes no source-world location or vanilla capture claim.

The adapter pulls existing levers, in declared bit order, and reads existing lamp states. It performs32 setup ticks and32 ticks after each complete input assignment, then requires quiescence. No blocks, items or source states are replaced. These are conservative settle windows, not minimal-latency measurements. Each suite contains513 operations covering all16 inputs and all256 ordered settled transitions. The runner resets and replays the suite and compares its full final checkpoint as well as outputs.

Expected values come from oracle.py's independent OR of the original programmed masks, never from the engine. A native functional pass is not vanilla parity proof, does not prove transient hazards, and does not certify other timing or programming configurations. No benchmarks are run in this validation batch.
