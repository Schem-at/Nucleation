# Conservative derived strong-power cache experiment

The prototype caches only the structural part of `direct_signal_to`:

`max(non-comparator contribution, live outputs of eligible comparator neighbours)`.

A six-bit mask retains eligible comparator positions even when their current
output is absent or zero. The cache reads the caller's current public
`ComparatorOutputs` map on every hit; `insert`, `remove`, `entry`, and whole-map
replacement therefore do not require a new setter or invalidation hook. Powered
state, allowed/excluded direction, and strong-output direction gate eligibility.
Exact strength preserves the entire u8 domain and computes the full maximum.

Each `World` owns a bounded 1,024-slot direct-mapped table protected by a mutex.
Queries use `try_lock`; contention/poisoning falls back to uncached computation.
This preserves Send/Sync and adds no unsafe code or runtime dependencies. Full
positions, wire-enabled/muted mode, and generation must match; hash collisions
can only evict entries. A retained Arc identity binds the table to immutable
rules. Each rules clone gets a fresh identity, and an old identity remains alive
while cached, preventing allocator-address reuse from producing stale hits.

Every attempted `World::set` and explicit growth call increments the whole-world
generation immediately, including redundant/failed writes. Rollover clears all
slots. The cache does not depend on storage indices. World cloning clears it
while preserving mode; checkpoint restoration clears it and retains the receiving
simulation's selected mode. Cache/configuration are excluded from physical world
equality. Off, On, and Shadow modes are explicit; Shadow recomputes results and
asserts equality. No shadow timings are accepted by the runner.

The initial prototype leaves boolean power queries on their existing
any-positive path. The second prototype shares cached exact strong power with
boolean conductor queries, while preserving own-emission short circuiting and
the original uncached any-positive path when disabled. It also avoids inspecting
the comparator mask when empty and uses wrapping hash arithmetic on 32-bit hosts.

Validation includes comparator 0/positive/0, arbitrary u8 values, all public map
mutation styles, emission gates, distinct rules/worlds with overlapping state
IDs, collisions, generation rollover, growth/negative positions, wire power and
connection changes, warm checkpoint restoration, and paired circuit traces.
The latter exercise pistons and wires inside ticks, inventories, item pickup,
flying machines, and two hipster cycles (17,968 changes, 659,770 updates).

This is experimental code. The first nine-round pilot did not establish a
hipster improvement. Its source patch and unfiltered measurements are retained;
the final 21-round comparison rejects both prototypes from production.

Fresh prototype validation: 420 engine/scenario/doctests passed, seven ignored;
WebAssembly check passed. Both prototype variants matched the baseline's eight
verification rows. The seven dynamic differential scenarios include the hipster,
6×6 door, small and large flyers, adder, comparator drain, and item-into-hopper.
