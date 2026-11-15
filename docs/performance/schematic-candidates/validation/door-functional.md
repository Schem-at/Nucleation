# Retained interlocking-door capture checks

[The generated report](door-functional.json) passes for the pinned Minecraft 1.21.11 idle and cycle captures. Each contains an initial snapshot and 320 consecutive tick-end snapshots, all with zero recorded players.

- Idle: block, inventory, and comparator-output maps remain identical throughout.
- Cycle: all three maps restore their initial values at ticks 160 and 320.
- Panel: all nine cells at x=3..5, y=5..7, z=1 contain exactly `minecraft:waxed_oxidized_copper` at ticks 80 and 240; those cells are air initially and at ticks 160 and 320.
- Positive activity: 122 block, 22 inventory, and 16 comparator-output tick boundaries change. These are differences between captured boundaries, not counts of intra-tick writes.

Reproduce without running Minecraft or Rust:

```sh
python3 tools/validate_tick_community.py \
  --captures /path/to/Nucleation-schematic-archive/validation/candidate-run \
  --output docs/performance/schematic-candidates/validation/door-functional.json
```

The validator checks the original capture SHA-256 values before reading their records; the report retains both hashes, source identity, server/runtime metadata, inputs, watch bounds, and phase hashes. This validates functional properties of retained vanilla captures. Nucleation equivalence is checked separately. It does not establish runtime entity, item-component, moving block-entity, intra-tick, or outside-watch equality, and does not execute a fresh server capture.
