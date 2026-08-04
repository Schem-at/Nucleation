# Lithium gametest descriptors

Structures come from CaffeineMC/lithium (LGPL-3.0), fetched — not vendored —
by `tools/fetch-lithium-gametests.sh` into the git-ignored
`tests/corpus/lithium/`. The descriptors here are this repository's own.

Pairing: `tests/corpus/lithium/<rel>.snbt` ↔ `tests/corpus/lithium-specs/<rel>.test.json`.

Run:

```sh
cargo run -p nucleation-cli -- \
    test tests/corpus/lithium --specs tests/corpus/lithium-specs
```

A row with no descriptor shows as `∅ unported` (work not yet done). A red `✗`
or `!` with a descriptor is a real engine gap — leave it red and list it below.

## How these ports work

At the pinned commit every lithium test is `minecraft:block_based`: the
assertions live *inside* the structure as vanilla `test_block`s (a `start`
block pulses when the test begins; the test passes when every `accept` block
is powered). The engine models the accept side natively: an accept-mode
test_block latches to an engine-internal `fired=true` variant on its first
neighbour signal (`TestAccept` in mc-tick), recorded as a block change. A
descriptor therefore only needs to:

- emulate the start pulse (place a `redstone_block` at the start block's
  cell, clear it two ticks later) — the one remaining approximation, and
- claim `events: [{pos: <accept>, to: "minecraft:test_block[fired=true]"}]`
  per accept — the vanilla pass condition, headless.

Most ports are **synthesized automatically** by `nucleation port`:

```sh
cargo run -p nucleation-cli -- \
    port --path tests/corpus/lithium --specs tests/corpus/lithium-specs \
         --out tests/corpus/lithium-litematic
cargo run -p nucleation-cli -- test --path tests/corpus/lithium-litematic
```

`port` writes one self-contained `.litematic` per structure into the tracked
`tests/corpus/lithium-litematic/` (see its `NOTICE.md` for the LGPL
attribution), carrying either a hand spec from this directory or a
synthesized one: 10 setup ticks (vanilla's `setup_ticks`, so placement
transients don't count), start pulses emulated, engine-unknown blocks
(command blocks, mob-test dressing) auto-asserted inert via a probe build —
visible in the embedded spec's `inert` list — and one accept-latch claim per
accept block. A machine gutted by the inert assertions goes honestly red on
its accept claim rather than lying green; a structure with no accepts at all
is skipped rather than given a spec that cannot fail.

Ported specs carry `origin` (the pre-compaction position of the build) and
pre-shifted coordinates: a litematic compacts to its non-air bounding box,
and same-tick update order hashes absolute positions, so an uncompensated
one-block shift phase-shifts observer chains — `hopper_dc_invalidation` was
red for exactly that reason.

## Status (pinned commit, 33 structures; 31 ported, 0 skipped, 0 broken)

Mob/AI-dependent ports (11) are parked in
`tests/corpus/lithium-litematic-unsupported/` — kept runnable, excluded from
the supported corpus. The supported corpus (20 files):

- ✓ passing (20): `itempickup`, `hopper_transfer_speed`, `item_sorter`,
  `hopper_dc_invalidation`, `hopper_dc_interaction_change`,
  `hopper_item_datacommand`, `ice_melt`, `cart_signalstrength`,
  `hopper_interaction_change` (support-lost rails pop, the chest cart falls
  onto the hopper, and both hoppers transfer through the cart entity),
  `hopper_interaction_change_v2` (hopper *minecarts* pull one item per game
  tick — from a container block above, or from a chest cart resting where
  the block was), `hopper_storagecart_interaction` (a chest cart riding the
  powered-rail loop is drained by the hopper under the track),
  `moving_block_collision`, `spawn_almost_all_entities`,
  `tnt_above_world`, `tnt_below_world` (summoned TNT falls out of the world
  and explodes in the void, touching nothing), `tnt_block_shield_entity`
  (the witch behind the shield reads a zero seen-percent),
  `tnt_knockback_entity` (real `ServerExplosion` knockback throws the witch
  onto the tripwire), `tnt_minecart_rail_shielding` (a primed TNT cart's
  blast treats rails — and blocks directly under rails — as indestructible,
  `MinecartTNT`'s damage calculator, read from the 26.2 bytecode and
  verified against a vanilla oracle trace), `lava_push_speed` (a dispensed
  armor stand rides the lava current onto the pressure plate; the
  fire-resistance splash potion is dressing an engine with no fire damage
  does not need).
  Finally `comparator_update_collection` — the hardest of the set, a
  gauntlet of same-tick piston races where the fail blocks fire unless
  vanilla's exact sub-tick order is reproduced. Settled against a vanilla
  oracle queue-log capture; it took four mechanisms: container writes poke
  `Direction.Plane.HORIZONTAL` (N,E,S,W — the order that decides which
  comparator schedules first), the poke extends through one conductor to a
  comparator beyond, a hopper's *failed* pull still fires the source
  container's `setChanged` every enabled tick (the churn lithium's update
  collection batches), and a chest with a solid block on its lid reads
  analog 0 (`ChestBlock.getContainer` answers null when blocked).

Nothing is failing. Every red between here and there was resolved by making
the engine more vanilla, never by softening a spec.
