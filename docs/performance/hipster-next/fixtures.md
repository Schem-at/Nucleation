# Expanded simulation workloads

Prepared 15 November 2026. These complement the 640-tick Minecraft 1.21.11
hipster golden regression. A functional check or agreement with the previous
engine is not a new independent vanilla trace.

## Reproduce

On the benchmark machine (Rust compilation for this investigation used
`harrison-work`):

```sh
cargo build --locked --example export_tick_fixture
python3 tools/prepare_tick_fixtures.py --exporter target/debug/examples/export_tick_fixture
export MC_TICK_BENCH_FIXTURES="$PWD/tests/benchmark-fixtures"
CARGO_PROFILE_RELEASE_DEBUG=1 cargo build --locked --release -p mc-tick --example bench_circuits
target/release/examples/bench_circuits --verify --expanded
```

The preparation script verifies SHA-256 pins before importing, preserves the
schematic's save version, and writes a manifest with input, output, embedded
scenario and exporter hashes. Parsing and fixture file reads occur outside tick
timers. Downloaded source files are retained beside the generated fixtures.
No downloaded community schematic is redistributed in Git.

## Added passing workloads

| Case | Source | Timed ticks and controls | Functional assertion |
| --- | --- | --- | --- |
| `comparator_drain` | Existing vanilla-captured SNBT | 40, placement settling | Three items leave the barrel and arrive in the hopper |
| `item_into_hopper` | Existing vanilla-captured SNBT | 40, placement settling | Falling two-item stack disappears into the hopper intact |
| `item_sorter` | Lithium, existing attributed Litematic port | 400 after 10 setup ticks; start at (9,5,2) | Upstream accept at (11,2,10) fires; inventory transfers occur |
| `hopper_transfer_speed` | Lithium, existing attributed port | 400 after 10 setup ticks; start at (2,2,3) | Accept at (1,1,14) fires; inventory transfers occur |
| `hopper_storagecart_interaction` | Lithium, existing attributed port | 400 after 10 setup ticks; start at (6,2,5) | Accept at (5,1,25) fires; cart and inventory activity are fingerprinted |
| `comparator_update_collection` | Lithium, existing attributed port | 400 after 10 setup ticks; start at (1,3,12) | Accept at (8,1,26) fires; ordered notifications and inventory activity are fingerprinted |
| `trapdoor2x3` | Starry0Wolf's archived community schematic | 240; lever (0,3,7) at 10,70,130,190 | All six panel cells close and reopen twice; every changed block returns to its initial state |

Lithium workloads use placement settling, RNG seed 0 and random-tick speed 3.
Setup-fired accept blocks are rearmed before timing. RNG seed 0 is restored
after setup for both verification and every timed iteration. Start blocks become redstone
blocks, matching the existing embedded scenario controls. These assertions cover
the named accept and inventory outcomes; they do not newly implement all upstream
GameTest environment or fail-block semantics.

Lithium ports retain their existing LGPL-3.0 provenance at commit
`c42972b6e9d21c8ff45559df6b271802050a22e2`; see the repository's
[NOTICE](https://github.com/Schem-at/Nucleation/blob/v0.12.3/tests/corpus/lithium-litematic/NOTICE.md). Current upstream
was also inspected at `2f3cbe1b42c4cbe4813afee8ed837ae7f54b7ac2`:
[item sorter source](https://github.com/CaffeineMC/lithium/blob/2f3cbe1b42c4cbe4813afee8ed837ae7f54b7ac2/common/src/gametest/resources/data/lithium-gametest/gametest/structure/item_sorter.snbt).
The benchmark uses the pinned repository ports, not a silent upstream upgrade.

The trapdoor download is linked by its uploader in their
[original post](https://www.reddit.com/r/redstone/comments/1jmb2s4/need_help_with_a_redstone_build/)
and retained on [Internet Archive](https://archive.org/details/2x-3-flush-piston-trapdoor).
The archived file has 73 non-air blocks and save data version 3955. This is the
archive attachment, not the different MediaFire rebuild linked in a reply.
Its final six-cell panel is x=2..4, y=4, z=3..4. At ticks 60/180 these cells
must contain orange wool; at 120/240 they must be air. It has no new independent
vanilla capture, so it establishes functional cycle checks and differential
coverage, not complete vanilla parity.

## Quarantined community probe

[TheDarkLord715RS's fast 4x4 shutter](https://www.reddit.com/r/redstone/comments/1g2ixqc/fast_4x4_shutter_piston_door/)
was downloaded from the author's Google Drive link. It imports with 493 non-air
blocks, four inventories and save version 2975. The input is (0,1,7), and the
4x4 doorway is x=4, y=4..7, z=4..7.

It closes, but at tick 120 quartz remains at (4,5,5). The retained engine from
`7c52701d` reproduces this failure too. The cause is not established: the saved
build, placement/origin assumptions and version semantics need an independent
vanilla run. Activity alone would have accepted this broken cycle.

`--verify --expanded` excludes this case. The failure remains reproducible with:

```sh
target/release/examples/bench_circuits --verify-case shutter4
```

This command deliberately fails its doorway assertion. It is not a passing
performance workload, and the benchmark's strict validation rejects timing it.
