# Hipster door reference

`hipster_door_1_21_11.json` contains observed tick-end block changes from an
actual Minecraft Java 1.21.11 dedicated server. It is not engine-generated.
The structure is `../corpus/structures/hipster_door.snbt`, converted from the
user-supplied **Fastest 3x3 Hipster — SpaceWalker and SoulBanished** Litematic.
The original Litematic's SHA-256 is
`a16c594465e5664c1a29c4b87b66745916dc99b377d68be62a960847674cd8a0`.

The server ran Fabric Loader 0.19.3, Fabric API 0.141.4+1.21.11, Java 21.0.13,
and a dedicated capture mod, with no redstone rule changes and zero connected
players. Official server/mapping artifacts were verified against the supplied
Mojang metadata chain, and Fabric dependencies against their profile hashes.

The probe loaded 990 block states directly into chunk sections, preserving the
saved machine without placement callbacks. It used Minecraft's datafixer on
all 48 block entities from data version 2730, restored them, and checked the
initial states, comparator outputs, 20 occupied inventory slots, empty initial
event/tick queues, and chunk ticking readiness. Origin and random seed were
both zero. The engine test uses the same saved-in-world loading.

The real lever behavior at `(12,7,13)` is invoked before ticks 10 and 170.
The capture has exactly 320 ticks, independent of chunk warmup. The nine quartz
blocks close into x=5, z=7..9, y=8..10, then return to their saved positions.
The last block change is tick 216; all final block states equal the initial
snapshot. A separate 640-tick vanilla run with actions at 10,170,330,490 repeats
every first-cycle block snapshot on the second cycle. The test reuses the
320-tick golden twice instead of duplicating its data.

Controls: 160 idle ticks have no block changes. A second 320-tick capture with
the passive event-queue recorder disabled has identical tick-end block states,
pending queue counts, and initial/final block entities. Every capture tick has
zero players.

State properties are sorted. Only the newer default properties absent from the
1.17.1 save are normalized: `barrier[waterlogged=false]` to `barrier`, and
`player_head[powered=false,rotation=4]` to `player_head[rotation=4]`. Waterlogged
barriers or powered heads are not equivalent. The trace includes all 2,918
observed tick-end changes; intra-tick round trips are invisible. This regression
asserts block states, not full within-tick event or inventory parity.

The test fails on the original engine at tick 12 (missing hopper observer pulse).
Fixing that and the short-pulse piston landing order makes closing match, but
reopening also requires door shape notifications, direct comparator side-input
semantics, and the removal callback of a moved powered observer. The latter
withdraws the observer's output at tick 178 so the upper pistons retract.

Run with:

```sh
cargo test --locked -p mc-tick --test hipster_door
```

The original capture tooling, raw snapshots, event traces, logs, and uploaded
server bundle are retained in the debugging environment under
`render_work/vanilla-oracle/` (see its README). Those runtime artifacts are not
dependencies of this committed regression test.
