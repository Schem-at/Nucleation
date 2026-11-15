# Flag-2 observer regression provenance

Captured 15 November 2026 on harrison-work, using the verified Minecraft Java
1.21.11 GameTestServer and Temurin 21.0.13+9. Each capture used a fresh world,
zero connected players, raw InWorld placement, seed zero and native DataVersion
4671. The harness verifies the server/mappings/library hashes.

The three-block lever/lamp/observer fixture isolates a missing observer pulse
found in extracted WorldRecordPlot part0465. The four-block
lever/conductor/iron-trapdoor/observer fixture isolates the corresponding pulse
missing in part0117. These fixtures are explicitly synthetic; no archived door
was edited. Both positive runs pull their real lever before ticks 21, 41, 61 and 81;
both idle runs leave it unchanged, for 100 ticks each.

Before the fix, the lamp and trapdoor states themselves agreed with vanilla,
but their watching observers first differed at tick 22. Both idle controls
matched. `Lamp` and `Trapdoor` used `set_quiet` despite the vanilla flag-2
shape-notification semantics. The fix uses `set_shape_only`, retaining the
absence of ordinary neighbour-change delivery.

The JSON golden records all initial nonair states and every changed tick-end
block state, including both pulse edges and both complete input cycles. Its
expected values come entirely from vanilla captures. Each run includes its
raw capture/source/scenario SHA-256. Flat inventory/comparator arrays were empty
and no authored entities were supplied. This is not an entity or intra-tick
notification oracle.

Raw artifacts:

```text
/home/harrison/nucleation-perf/plot-validation/oracles/components-before/
Nucleation-schematic-archive/validation/plot-oracles-v1/components-before/
```

Committed input JSONs live in
`docs/performance/schematic-candidates/plot-validation/component-oracles/`.
For each `lamp` / `iron_trapdoor` and `cycle` / `idle` pair, run the pinned
`tools/gametest/run_community_capture12111.py` with its scenario, fixture JSON as
`--source-file`, existing runtime/JDK paths from the plot-validation report, and
a new output directory. Then compare with `check_community_capture`.

Focused regression and established golden checks:

```sh
cargo test --locked -p mc-tick --test flag2_observer --test hipster_door
cargo test --locked -p mc-tick --lib observer::tests
```

Both minimized regressions, both hipster tests (including 640 ticks), and all 8
observer unit tests passed on harrison-work after the change. The full plot
recheck also restored all 661 cycle frames of part0465, advanced part0117 from
its observer mismatch at 29 to a separate wall-state mismatch at 91, and preserved
the passing lamp 2×2 and 961-frame binary comparator reference.
