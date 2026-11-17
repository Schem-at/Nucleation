# Note-triggered extender: wall shape signals

Verified 17 November 2026. The wall update gap is fixed in the shared Rust
engine and the local Atlas WASM. The original 9,821-block schematic now matches
all **1,101 frames** (initial state plus 1,100 ticks) of the retained playerless
Minecraft Java 1.21.11 capture. The production WASM schematic importer separately
matches the original file over the 160-tick capture.

This supersedes the remaining tick-30 mismatch in
[note-trigger-scaffolding.md](note-trigger-scaffolding.md). It does **not** establish
a finished extension/retraction cycle: the vanilla reference is still active at
tick 1,100, with its honey payload at the saved positions. The source is stamped
26.3 / data version 5023; the explicitly selected reference remains 1.21.11 /
4671, using exact states without datafixing. Source metadata is preserved.

## Failure and fix

At tick 30, an observer finishes moving above the wall at local `(0,0,19)`.
Vanilla changes its east and south arms from `low` to `tall`. The observer
watching that wall sees the change and propagates the signal. Previously the
wall stayed inert, so later piston motion diverged. This was a simulation
failure, independent of Atlas's mesh-update coalescing.

The wall behavior now handles directional shape messages, connection exceptions,
arm heights, central-post rules, stacked posts, and waterlogged fluid scheduling.
Shape writes preserve observer notifications without adding ordinary neighbor
notifications. All 324 combinations per wall family are available, with shared
transition tables. Walls remain partial collision geometry and now participate
in active component isolation.

Static collision-face coverage is precomputed only when a registry contains
walls. Moving-piston coverage reads the current movement progress and head/base
union at the query, including mutations within the tick. It does not cache a
result across world mutations. Raw in-world loading preserves saved wall states.
The pinned server's WallBlock contracts were inspected with its bundled JDK;
the implementation and captures are maintained in this repository.

## Validation

Rust compilation and tests ran on **harrison-work**, Rust 1.98.0
`88d9e12ae` (2026-08-18), with locked dependencies and four build/test workers.
The WASM release uses `bridge,mc-tick,meshing` on `wasm32-unknown-unknown`.

| Check | Result |
|---|---|
| `cargo test --locked -p mc-tick -p mc-test` | 436 passed, 0 failed, 7 ignored |
| `cargo test --locked --features bridge,mc-tick --lib bridge::mc_tick` | 30 passed |
| Original native capture | All 1,101 frames match; 2,221,936 ordered block changes |
| Original through production WASM importer | All 161 frames match; 70,144 ordered block changes |
| Four new small vanilla captures, native and WASM | All 25 frames per case match |
| Existing three scaffolding captures through new WASM | All 25 frames per case match |

The four new hand-authored fixtures are wall heights/posts with observing
observers, connection exceptions, enclosed waterlogging, and a sticky piston
moving an observer over a wall through extension/retraction. The connection
fixture supplies already-shaped bars/gates; it tests wall connectivity, not those
neighbors' own shape derivation. Additional tests cover all wall companions,
idle loading, checkpoint replay of pending observer pulses, and exact face-union
coverage with a narrow gap.

A fresh browser loaded the installed engine and the original litematic. Over
160 ticks it processed 70,144 block records, wrote the render mirror 6,651 times,
and rebuilt 346 chunks in eight-tick batches. Up to 826 piston flight objects
were rendered; the final render mirror exactly matched the simulation after
excluding moving placeholders (drawn separately). These counts confirm the
new motion reaches the renderer; they are not FPS measurements. The mesh
coalescing from the preceding change remains enabled.

The original native trace fingerprint is
`4281e43b9d544cebd7503c089f686c90496bce0f5b7d07e318f9d85ff3c765be`
for both engines under `check_community_capture` serialization. This is a
comparison with retained vanilla evidence, not a newly captured full mechanism.
The four small captures were recorded afresh.

Native comparisons cover tick-end blocks, comparator outputs and occupied flat
inventory slots in the watch bounds. WASM comparisons cover tick-end blocks.
Neither claims vanilla intra-tick notification, moving-block-entity or runtime
entity equality. Ordered block-record counts above are not the vanilla capture's
sum of tick-end changed cells and are not performance measurements.

## Scope and remaining gaps

The collision table covers full cubes, piston bases/heads, slabs, trapdoors,
walls, soul sand and diodes; moving shapes use those supported static shapes.
Other partial collision shapes, including stairs and fences, still need broader
coverage. This is not a claim that every wall interaction in Minecraft is now
supported.

The exploratory connection matrix also exposed pre-existing missing melon
behavior and iron-bar shape updates. Its first capture attempt used an
incomplete barrier descriptor and was rejected by the oracle. Corrected/failing
artifacts are retained outside Git; the committed matrix uses canonical states
and pre-shaped neighbors. None of those unrelated behaviors was declared fixed.

## Reproduction and artifacts

The original SHA-256 is
`e3f1466374535f9eda361b0745dd6a81aa090aba5a961e3b93944b0d320e457b`.
Trigger its note block at `(0,3,0)` before tick 21. Native comparison:

```sh
cargo build --locked -p mc-tick --example check_community_capture
cargo test --locked -p mc-tick --test walls
# CAPTURE points at an external original capture.json.
target/debug/examples/check_community_capture --capture "$CAPTURE"
# ENGINE is the Atlas engine directory. SOURCE is the original litematic.
node --max-old-space-size=4096 tools/check_tick_block_capture_wasm.mjs \
  "$ENGINE" "$CAPTURE" "$SOURCE"
```

To regenerate a small oracle, select `wall_heights`, `wall_connections`,
`wall_waterlogged` or `wall_piston` and run on harrison-work with its pinned
runtime/JDK and a fresh output directory:

```sh
python3 tools/gametest/run_community_capture12111.py \
  --runtime "$RUNTIME" --jdk "$JDK" \
  --scenario "tools/gametest/scenarios/$CASE.json" \
  --source-file "tools/gametest/samples/$CASE.snbt" --output "$RUN"
```

Source checkpoint: `6e1bd6714129948e51344dec781b0e1ac295e8c9`.
Installed WASM SHA-256:
`f3469a884704684e445770d814e8b9ac98450d7c9658ff26b4e5cbd904519d74`.
The 797 tracked Rust source/manifest files were SHA-256 checked against the
remote build checkout, with no differences. The previous `c3361e60…` WASM and build metadata are retained under
`Nucleation-schematic-archive/validation/wall-engine/rollback/`.
Restore that pair to `apps/sim-lab-wasm/public/engine/` and reload Atlas to roll
back. The source change is isolated on `codex/extender-wall-updates`.

The original schematic, large captures, runtimes, WASM and raw logs remain outside
Git. Small authored fixtures/goldens, replay tools and this report are version
controlled. Compact results are in `validation/wall-shared-engine.json`; local
installation and browser evidence are in the external archive's
`validation/wall-engine/` directory. No package release was made.
