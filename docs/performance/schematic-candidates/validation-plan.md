# Reproducing the community checks

The [results](README.md) describe what actually ran on 15 November 2026. This
recipe uses the same source files and modes. Run Rust/Java commands on
**harrison-work**, outside any native performance measurement session.

## Eight initial imports and four logic checks

From the repository root, retain originals and use the production importer:

```sh
python3 tools/prepare_tick_logic_archive.py
python3 tools/prepare_tick_dynamic_candidates.py
CARGO_BUILD_JOBS=4 cargo build --locked --features mc-tick --example export_tick_oracle_fixture
python3 tools/import_tick_candidates.py \
  --exporter target/debug/examples/export_tick_oracle_fixture \
  --sources tests/benchmark-fixtures/candidates \
  --output tests/benchmark-fixtures/candidates/imported-new
cargo build --locked -p mc-tick --example check_logic_candidates
 target/debug/examples/check_logic_candidates \
  --fixtures tests/benchmark-fixtures/candidates/imported-new
```

The full four-case command currently exits **1** because COUNT001 fails its first
increment after loading zero. Preserve its JSON/stderr and diagnose it; do not
turn that failure into a pass. For the accepted functional subset, append
`--case LFF006 --case DECODE004 --case ADD005`. Setup, exact controls and
assertions are in the example. DataVersion4189 stays4189.

Compare the reference build above with a second build using:

```sh
cargo build --locked -p mc-tick --example check_logic_candidates \
  --example check_community_capture \
  --features experiment-directional,experiment-update-pool,experiment-world-plan-cache,experiment-execution-view,experiment-lazy-journal,experiment-logic-plan-storage,experiment-moving-phase,experiment-transition-signatures
MC_TICK_WIRE_PLAN=shadow MC_TICK_TRANSITION_SIGNATURES=shadow \
  target/debug/examples/check_logic_candidates \
  --fixtures tests/benchmark-fixtures/candidates/imported-new
```

Save each binary/hash and redirect complete JSON records before rebuilding the
same target path. Compare all parsed record fields, not just final outputs.
These are debug correctness runs; shadow mode must be off for timing.

## Native 1.21.11 comparator reference

Use the original file under the local archive's
`sources/logical-redstone-archive/files/Archive/OTHERCOMB_other-combinational/OTHERCOMB017_8_bit_Binary_Comparator/attachments/OTHERCOMB017_8bit_binary_comparator.schem`.
Its SHA-256 is
`92d44580dc1539903b56ad08a8b1be89d70de602328d5fe36ef589743c3a3db3`.
Set `SOURCE` to that file on the laptop, and `RUN` to a fresh artifact directory:

```sh
mkdir -p "$RUN"
target/debug/examples/export_tick_oracle_fixture \
  "$SOURCE" "$RUN/fixture.json" "$RUN/fixture.snbt"
python3 tools/check_tick_binary_comparator.py generate \
  --fixture-json "$RUN/fixture.json" --source-file "$SOURCE" \
  --scenario-out "$RUN/scenario.json" --checks-out "$RUN/checks.json"
python3 tools/gametest/run_community_capture12111.py \
  --runtime /home/harrison/nucleation-perf/execution-next/vanilla/runtime \
  --jdk /home/harrison/nucleation-perf/execution-next/vanilla/jdk-21.0.13+9 \
  --scenario "$RUN/scenario.json" --source-file "$SOURCE" \
  --output "$RUN/vanilla"
python3 tools/check_tick_binary_comparator.py validate \
  --fixture-json "$RUN/fixture.json" --source-file "$SOURCE" \
  --scenario "$RUN/scenario.json" --capture "$RUN/vanilla/capture.json" \
  --output "$RUN/functional.json"
cargo build --locked -p mc-tick --example check_community_capture
target/debug/examples/check_community_capture \
  --capture "$RUN/vanilla/capture.json"
```

Repeat the last checker using the selected feature build and shadow environment.
Repeat the Java runner into another new directory, then compare initial state
and every snapshot plus the fixture/actions. Runtime timing metadata may differ.
The 40-stage endpoint oracle is independent of Nucleation's output; it checks
120 output lamps and640 input states. It is not an exhaustive input-pair test.

The runner verifies the existing server/mappings/libraries and Java21.0.13;
it downloads nothing. Actual runs were loopback-only, offline, zero-player,
fresh-world captures. Native4671 inputs only are accepted: no DFU is implemented
in this new generic path, and an older schematic must not be relabeled4671.
`capture.sh`, `run.sh` and the larger `TraceCapture.java` target26.2 and are not
substitutes for this pinned1.21.11 path.

## Door evidence

The archive retains `validation/candidate-run/interlocking_cycle.json` and
`interlocking_idle.json`, plus the original source file and both captures. The
scenarios preserve all exported typed BE SNBT, source hash and native version.
They use InWorld placement, origin `[0,100,0]`, watch `[-2,-2,-2]..[10,14,6]`,
seed0 and320 ticks. The positive scenario pulls the real lever at `[6,1,4]`
before ticks11,91,171,251; the idle control has no actions. Re-run either with
the generic Java runner and a fresh output directory.

To audit the exact retained captures:

```sh
python3 tools/validate_tick_community.py \
  --captures /path/to/Nucleation-schematic-archive/validation/candidate-run \
  --output /new/path/door-functional.json
```

This audit deliberately pins the retained capture hashes. A new capture with
different runtime metadata requires an explicit new evidence pin after checking
its full state sequence. Nucleation currently rejects
`minecraft:waxed_oxidized_copper` in this fixture before tick0. The vanilla
functional pass therefore does not admit the door as an engine benchmark.

## Admission of further library designs

1. Preserve original bytes, source/author/license, version, bounds and contents.
   Check format variants for lost entities and marker/control changes.
2. Identify real controls and independently defined outputs. Use finite positive
   and unchanged-input controls; require activity and quiescence/return/cycle
   properties appropriate to the mechanism. For flyers, include the travel
   corridor and expected translation, not merely nonzero writes.
3. Import with production code. Reject unknown active blocks, unexplained losses
   and unsupported version/entity/inventory behavior instead of substituting air.
4. Add functional assertions and, where the mechanism depends on update order,
   a matched Minecraft reference. Native older saves need a verified datafix
   path before using the generic native4671 oracle.
5. Compare reference and optimized engines, including ordered records/shadow
   assertions where applicable. Retain failures separately from accepted scores.
6. Only then add a timed workload: checkpoint/RNG reset, setup and assertions
   outside timers, input/stepping inside, identical recording mode, compiler,
   flags and harness across variants, randomized interleaved rounds.

The new generic capture observes tick-end blocks, comparator outputs and flat
occupied inventory slots inside the watch box. It does not independently census
vanilla entities, nested item components, moving BE payloads or intra-tick
notifications. Extend that observation model before claiming those mechanics
are covered. Storage/crafting shortlist notes identify those required extensions.
