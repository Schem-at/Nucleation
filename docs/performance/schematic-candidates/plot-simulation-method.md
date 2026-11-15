# WorldRecordPlot native simulation assessment

This protocol assesses whether each isolated candidate loads, steps, responds to real controls, and reproduces its own observed trace after restoration. **It does not certify a functioning door or Minecraft parity.** Extraction preservation and textured rendering also do not provide those proofs.

## Inputs and preservation

Select `library.json` entries from `world-record-parts` and `world-record-floor-parts`, classified `door_candidate`, without `infrastructure_review`. At implementation time this is 803 paths: 746 original components and 57 floor-aware children. Results use the exact input SHA-256; identical files share one isolated subprocess and retain every alias.

The runner verifies SHA-256 before launching. The Rust example reads the original `.litematic` using Nucleation's production format manager, exports through `to_gametest_snbt`, and loads the actual `mc-tick` engine via `mc_test::try_build_sim`. No blocks are replaced or declared inert, and no containers are filled. Original inputs are never rewritten.

The input must explicitly carry native save DataVersion **4671**. It is passed through unchanged to structure parsing and entity motion semantics. Block cells and entities are normalized by the production export's bounding-box minimum; that same original minimum supplies Java coordinate hash ordering. Output positions add the minimum back. Engine snapshots of entity vectors use local coordinates and explicitly retain the origin. Source/parsed entity and block-entity counts must agree. Unsupported entity types, unknown blocks, unsupported commands, invalid imports and panics are reported rather than silently omitted.

The original NBT remains in the archived file; simulation consumes only the fields implemented by the engine. Model limitations still apply, including frozen-body entity models without general mob AI, incomplete item-component semantics, assumed item stack limits and incomplete block-entity behaviours. These observations are not a claim that every arbitrary NBT property participates in simulation. Missing saved inventories and cut-off geometry remain unmodified. Existing curation warnings are attached to each result.

## Protocol

1. Load with `InWorld`: no placement callbacks or forced neighbour-update settling. RNG seed is 0; random tick speed is explicitly 0.
2. Run 80 idle observation ticks from the raw saved state. Record activity and net cell differences. A cell becoming air may be normal movement; it is a damage warning to investigate, not an automatic corruption verdict.
3. Save the resulting checkpoint.
4. Independently restore that checkpoint and reseed to 0 for a 320-tick idle control and **every** discovered lever/button. Control trials perform actual `use_block` interactions at elapsed ticks 0, 80, 160 and 240, immediately before the next simulation step. Missing/replaced controls are reported. Contact devices (pressure plates and tripwire hooks) are inventoried as untrialled: clicking them would not be a valid stimulus.
5. Repeat each completed trial from the same checkpoint with RNG reseeded to 0. Compare ordered-record/queue/entity trace fingerprints and final observable state fingerprints. Restoration intentionally restarts the RNG for trial reproducibility.

320 ticks provide four interactions: two on/off pairs for levers, four pulses for buttons. A button's control architecture may need a different protocol. A door taking more than 80 ticks to settle may also need a longer individually curated protocol. `--ticks 640` extends to eight interactions. Returning to the original layout is recorded, not used to assert that a doorway opened. Controls can address internal subcircuits; none is automatically declared the door's intended master input.

The initial idle run does not establish update stability under placement; `InWorld` intentionally preserves the saved state. Scheduler quiescence only describes the engine's scheduled queues: entities, hopper tickers or other block-entity work may remain active even when this flag is true.

## Recorded evidence

Each trial records block, inventory and entity change counts; activity ticks; activity away from the clicked control; piston-related changes; completed ticks; pending tick/event and moving-block counts; final entity and inventory counts; net changed/added/removed cells; and snapshots every 80 ticks. It separately records block/inventory equality at elapsed 0, 160 and 320, which is useful evidence of repeatable cycling but not a doorway/aperture assertion.

Fingerprints use BLAKE3 over deterministic, versioned harness serialization: ordered block descriptors, inventory and entity records; every-tick pending queues/entity state; and canonical full visible-world/inventory/comparator snapshots at endpoints. Debug-formatted entity/movement data retains NaN values; these fingerprints are comparable only using the same engine/harness serialization. This pass does **not** record every ordered neighbour delivery or prove hidden state/vanilla event equality. Full per-tick block-state vanilla captures remain a separate validation stage.

Every input runs in its own process, with a default 120-second external wall timeout and 4 GiB address-space cap on Linux. The world is limited to 8,000,000 cells and each trial stops after exceeding 500,000 ordinary records. A hung intra-tick cascade cannot defeat the external process timeout. Resource exhaustion is a typed incomplete assessment, not an incompatibility finding. Completed phases survive in the JSONL transcript. The Python runner may execute two independent inputs concurrently; each engine simulation remains single-threaded. These diagnostic run durations are **not benchmarks**.

## Reproduction on harrison-work

Compile there, with the recorded locked dependency set and no experimental feature additions unless intentionally doing a separate comparison:

```sh
cargo build --locked --release --features mc-tick --example check_plot_simulation
python3 tools/check_tick_plot_simulation.py \
  --library /path/to/archive/library.json \
  --inputs /home/harrison/nucleation-perf/atlas-render/originals \
  --binary target/release/examples/check_plot_simulation \
  --output /home/harrison/nucleation-perf/plot-simulation \
  --revision "$(git rev-parse HEAD)" --jobs 2
```

Use `--only SOURCE/PATH` or `--only SHA256` for a pilot. `--resume` reuses a result only when the binary hash, Python harness hash and run configuration match. Changing the binary, protocol or timeout requires a fresh assessment. The manifest records input library hash, binary hash, revision, host and all selected source hashes. Result statuses distinguish import/build/structure rejection, runtime engine stop, nondeterministic replay, resource budgets, subprocess timeout, and completed observational assessment. `functional_status` remains `unproven` and `vanilla_status` remains `not_checked` for this tool, even for active repeatable candidates.

Next steps for promising candidates are manual input/aperture identification, explicit open/closed/reset assertions, and a native-version vanilla reference capture with identical source coordinates and inputs. Empty required inventories, missing controls and geometry cut at the plot edge must be resolved with source evidence rather than guessed repairs.

## Initial assessment: 15 November 2026, before observer-notification fix

The complete pass ran on `harrison-work` (Linux x86_64), with all 803 SHA-verified original files. The final release example compiled successfully there. The Python driver passed syntax/help checks and temporary mocked-process checks covering result capture, exact-configuration resume, external timeout with retained stage information, and source-hash rejection. No Rust compilation ran on the Mac.

| Outcome | Original parts | Floor-aware children | Total |
|---|---:|---:|---:|
| Completed native observational assessment | 635 | 53 | 688 |
| Unknown block state rejected at build | 100 | 4 | 104 |
| Unsupported entity rejected at parse | 10 | 0 | 10 |
| Unsupported inventory container rejected | 1 | 0 | 1 |
| Total | 746 | 57 | 803 |

There were no final-run process timeouts, crashes, runtime stops or nondeterministic replays. Of the 688 assessed candidates, 545 had clickable controls and 143 did not. The 596 independently trialled lever/button positions all replayed identically after restore/reseed; 520 trials across 481 distinct candidates produced piston-related changes. 179 trials across **160 distinct candidates** combined piston activity with equal block layout at elapsed 0, 160 and 320. This set includes tiny piston fragments as well as larger constructions; periodicity alone does not establish a usable door.

Four candidates changed blocks during the initial 80-tick idle observation: `part0578` (447 changes, 31 net changed cells), `part0640` (24, 10), `part0677` (23, 5), and `part0699` (10, 8). Sixteen candidates recorded initial entity activity. Twenty-five inputs report a frozen-body entity-model limitation. These are observations to examine with the original geometry and vanilla captures, not automatic failure labels.

The source inventory caveat is material: all 688 assessed candidates had **zero parsed saved block-inventory items**. The one input with nonzero parsed items was `part0441`, which contains three items in a `pale_oak_shelf` whose container slot semantics are unsupported by the engine. Its original contents were retained; the harness now returns `unsupported_inventory_container` before calling the shared builder. The first run exposed a builder panic at this point. The final change is an explicit harness preflight, not invented shelf mechanics. All 688 successful assessment payloads remained byte-equivalent JSON values between the first and final runs.

The 104 block rejections include overlapping groups: 53 inputs with unregistered button/pressure-plate variants, 12 with beds, 10 with copper grates, and seven with `cave_air`, `void_air` or `structure_void`. Frequent individual names are `polished_blackstone_button` (18 inputs), `cherry_pressure_plate` (14), `waxed_oxidized_copper_grate` (9), `creaking_heart` (9), `birch_pressure_plate` (6), and `lectern` (5). Other rejected names include newer lightning-rod variants, iron chains, crafters, ender chests and some decorative/environment blocks. These are actual missing registrations/model coverage in this build; no inert replacement was used to make the run pass. Entity parsing rejected nine inputs containing `item_frame` and one containing `cherry_boat`.

Full per-input results, transcripts and the manifest are kept outside Git:

- Laptop: `/home/harrison/nucleation-perf/plot-validation/native-all803-final/`
- Local archive: `Nucleation-schematic-archive/validation/plot-native-v2/`
- Initial diagnostic run retained separately in `validation/plot-native-v1/`.

The final run used the settings above, no added experimental features, and base revision `ecc1a6a3` plus the assessment/example and contemporaneous adapter work. The exact binary and harness hashes pin the run more precisely than that uncommitted-base label:

```text
Executable SHA-256:
72f618ac07cc8fb1a9fba381ec12960e0cf723fd4c692828ba06f810a70ad52f
Python runner SHA-256:
77bd6106b44197941f0e0cbaf7df9d3928695bf4f5f1961ee2f52e87f124812f
Rust example SHA-256:
a955f847fe7b4af40929f57200f69a84bb2d55988ed9ba72790adb02b60d69d8
```

These counts describe native load/run/activity/replay evidence only. Every row from this tool still carries `functional_status: unproven` and `vanilla_status: not_checked`; independent aperture assertions and vanilla comparisons may add stronger evidence separately.

## Final rerun after the observer-notification fix

After core commit `9a51b44000c48a2261afe8b9f12ba7d6850c28dc` restored observer shape notifications from lamp/trapdoor state changes, all 803 inputs were rerun on `harrison-work` with the same protocol. The executable was copied to an immutable run path before launch. The new binary SHA-256 is `81c9bd2c2be9cd1d5fedc3a6ed9b3b35e07d64aa986886f272de1206258bb4d0`.

Support outcomes stayed at 688 completed assessments, 104 block rejections, ten entity rejections, and one unsupported inventory container. All 596 control trials replayed deterministically. **563 control trials across 520 candidates** now show piston activity; **180 trials across 163 candidates** combine activity with layout equality at 0/160/320. There are 212 changed assessment payloads compared with the pre-fix run, as expected for a real notification-behaviour change. Neither activity nor cycle equality is a standalone functional or vanilla certificate.

The final raw dataset is `/home/harrison/nucleation-perf/plot-validation/native-all803-flag2/` on the laptop and `Nucleation-schematic-archive/validation/plot-native-v3/` locally. Earlier v1/v2 evidence remains intact. [The compact integration report](validation/plot-native-summary.json) records exact run settings, binary/revision, support failures, current oracle outcomes and artifact paths.

`tools/register_tick_plot_validation.py` merges the native evidence with the six independent vanilla oracle cases. It defaults to a dry report and writes archive status only with `--apply`. It preserves unrelated status entries and additional fields, and checks matching source hashes, all captured frames, actual cycle actions and both cycle/idle comparisons before granting a parity pass. A hash-named snapshot of the exact oracle report is saved with applied status, retaining current and pre-fix comparator provenance.

```sh
python3 tools/register_tick_plot_validation.py \
  --archive /path/to/Nucleation-schematic-archive \
  --native validation/plot-native-v3 \
  --overlay /tmp/plot-validation-status-proposed.json
# After reviewing the proposed status:
python3 tools/register_tick_plot_validation.py \
  --archive /path/to/Nucleation-schematic-archive \
  --native validation/plot-native-v3 --apply
```

The applied plot overlay has 682 smoke-only entries, 115 unsupported entries, four vanilla tick-end mismatches, one failed vanilla functional oracle and one functional-plus-parity pass. Five of six candidates pass their vanilla functional oracle; two native activated traces match vanilla, but the hollow-centre 3×3 still fails the complete-door criterion, so only the lamp 2×2 is admitted. All six idle traces match; idle-only agreement never promotes an entry. The initial six unrelated validation entries are preserved. Separate typed-adapter evidence may be attached by its own workflow.
