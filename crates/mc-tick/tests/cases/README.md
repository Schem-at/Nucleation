# Dynamic cases

Black-box scenario tests, one per `*.test.json` in this folder. Adding a case
is adding files — nothing recompiles. Run them all with
`cargo test -p mc-tick --test cases`, or one while iterating with
`MC_TICK_CASE=<substring> cargo test -p mc-tick --test cases`.

## Two carriers, one vocabulary

The descriptor below has two homes, and the evaluator
(`crates/mc-tick/tests/support/scenario.rs`) is shared verbatim between them:

- **`*.test.json` beside an `.snbt`**, here — run by `crates/mc-tick/tests/cases.rs`.
- **inside a `.litematic`**, in nucleation's `tests/scenarios/` — run by
  `tests/litematic_cases.rs`. The descriptor is stored in a root-level
  `NucleationTest` compound (`{Format: 1, Spec: "<this JSON>"}`), beside
  `Metadata` rather than inside it: Litematica ignores unknown root tags, and
  our own writer rebuilds `Metadata` from scratch on every save, so a spec
  stored in there would vanish on the next re-save. `Spec` may be one descriptor
  object or an array of them — a door needs to prove both that it stays still
  untouched and that it opens when pressed, and those are two runs of one build,
  not two copies of it. The `structure` field is rejected there: the carrier
  *is* the structure. Authoring loop:
  `cargo run --example scenario_inspect -- <build> --dump-test | ... | --embed spec.json --write <build>`.

A `.litematic` also states the Minecraft data version it was captured at, and
the runner passes it to the engine as the `Entity.load` Motion semantics. That
is load-bearing rather than decorative: the record 3x3 door is 4082, which keeps
NaN velocities, and read at the wrong version its nan carts load as ordinary
carts and the door silently comes apart with no error anywhere.

These deliberately assert **end states only** — which blocks stand where at
named ticks — never traces or event order. That is the contract a door owes its
user: it opens, it closes, it lands back exactly where it started. A different
redstone backend that keeps that contract passes these unchanged; the
per-tick-exact engine tests live in `conformance.rs` and stay engine-specific.

## Format

```json
{
  "name": "what this proves, in one sentence",
  "structure": "../corpus/structures/door.snbt",
  "origin": [15, -64, 0],
  "settle": "in-world",
  "actions": [
    { "tick": 10, "use": [10, 4, 1] },
    { "tick": 20, "place": [0, 0, 0], "state": "minecraft:air" }
  ],
  "checks": [
    { "tick": 60, "expect": "changed" },
    { "tick": 95, "expect": "initial" },
    { "tick": 95, "expect": "same-as", "as_tick": 30 },
    { "tick": 40, "expect": "air", "region": [[2, 2, 2], [7, 7, 2]] }
  ]
}
```

- `structure` — relative to this file; defaults to `<stem>.snbt` beside the
  case. SNBT today; litematic/schem carriers arrive via the nucleation-side
  loader, which will embed this whole descriptor in the schematic's metadata.
- `origin` — where the capture's (0,0,0) sat in the game's world. Wire update
  order hashes absolute positions, so an in-world build simulated at the wrong
  origin is subtly a different machine. Omit (defaults to 0,0,0) for
  origin-agnostic builds.
- `settle` — `placement` (default; vanilla placement pass + settle, for builds
  saved at rest), `quiet` (`onPlace` only), or `in-world` (neither — the build
  was recorded mid-state in the world it stood in).
- `actions` — `use` right-clicks (levers, buttons); `place`+`state` writes a
  block (`minecraft:air` breaks). An action at tick T fires during tick T.
- `checks` — evaluated at tick T *before* any action at T fires, seeing the
  world after exactly T ticks:
  - `initial` — equals the settled pre-action world (a reset check: needs no
    authored expectations at all).
  - `changed` — differs from initial (the machine actually moved).
  - `same-as` + `as_tick` — equals the world at an earlier check's tick. This
    is the repeatability claim: a door's second close must land the same world
    as its first, or it is a door that works once.
  - `air` + `region` — every block in the box is air (a doorway is passable).
  - `blocks` + `blocks` map — exact states at positions:
    `"blocks": { "8,4,2": "minecraft:white_concrete", "3,4,1": "minecraft:redstone_wire[power=14]" }`.
    A descriptor without properties matches the block name alone; listed
    properties must each hold and unlisted ones are free, so
    `redstone_wire[power=15]` matches any dust at power 15 whatever its
    connections. `"minecraft:air"` asserts a position is empty.
  - `entities` + `entities` list — item entities and minecarts:
    `{ "item": "minecraft:redstone", "region": [[0,0,0],[0,2,0]], "count": 1 }`
    (`kind` instead of `item` for minecarts; omit `count` for "at least one").
    An item entity can also be required to carry container contents — a
    dropped shulker box keeping its slots:
    `"with_contents": [{ "id": "minecraft:diamond", "count": 2 }]`.
  - `quiescent` — nothing is pending: no scheduled tick, no queued update. The
    backend-agnostic spelling of "the run finished", and the only thing that
    distinguishes a door that came to rest from one still mid-cycle.
  - `entity-count` — how many entities the world holds, with `count`,
    `at_least` or `at_most`. A door glued together by nan carts is a door whose
    entity count is load-bearing.
  - `changes` — how many blocks changed over the run so far, same three bounds.
    `{"expect": "changes", "count": 0}` on an untouched build is the strongest
    claim in the file: vanilla ticking that same save in place changes nothing,
    so neither may we.
  - `fill` + `cells` — how many of a named cell set are non-air, same three
    bounds. A doorway is nine cells and *how many of them are filled* is the
    question; which nine is the authored part.
    `"cells": ["3,0,20", "4,0,20", ...]`.
  - `peak-fill` + `cells` — the most of those cells that were *ever* filled, up
    to this tick. How far the door got rather than where it stopped: a leaf
    sweeps through the doorway and settles somewhere, and the width of the sweep
    is the claim worth pinning — unlike a reading at one named tick, it does not
    care which tick the sweep peaked on.
  - `min-entity-y` + `y` — no entity sits below `y`. The cheap form of "it did
    not fall apart": a cart that lost its NaN velocity leaves through the floor.
  - `riders` + `kind` + `seats` — the passengers' exact seat heights, ascending.
    Seats are structure, not physics: a blaze 0.1875 above its cart is where the
    file put it, and a seat that moved means the save was reinterpreted.
- `seed` — seeds the vanilla random source (`java.util.Random`'s LCG,
  bit-for-bit). Behaviours that jitter — dispense trajectories, dispenser
  slot choice, piston-destroy drops — draw from it in a fixed order, so a
  seeded case is exactly reproducible. Omit it and the engine uses each
  distribution's mean instead (no jitter at all). Reproducibility is the
  contract; matching a live server draw-for-draw is not, because a real
  `ServerLevel.random` is shared with everything else in the world.
  - Any check takes an optional `region` (`[[min],[max]]`, inclusive) to
    restrict the comparison.

The door cases show the intended layering: `changed`/`initial`/`same-as` prove
the machine moves and resets without authoring anything, while a `blocks` check
on the seal (panel corners, core slime, a dust's power) proves the door is
actually *closed* — not merely different. Author those positions with
`MC_TICK_DIFF_LIMIT=5000` and a deliberate `initial` check at the closed tick:
the failure diff prints every block that appeared, which is the door leaf.

Failures print per-block diffs (`pos: expected X, got Y`), capped at 20 lines.

## Suites: more than one test per build

An embedded or sidecar descriptor may be any of three spellings:

- one case object — `{"name": ..., "checks": [...]}`,
- an array of them — a door proves it stays still untouched *and* opens when
  pressed, as separate runs of the same build,
- a versioned suite — `{"format": 1, "cases": [...]}`. An unknown `format`
  is refused, so a future container change cannot be silently misread.

An empty suite is refused too: a build whose test says nothing would pass by
having nothing to say.

## `events`: opt-in ordering claims

Checks are deliberately end-state only, so a faster backend that preserves
observable behaviour passes without golden churn. When a case's *point* is
ordering — a subtick race, an update that must fire exactly once — it opts in:

```json
"events": [
  { "kind": "block-changed", "pos": [0, 1, 0],
    "to": "minecraft:redstone_lamp[lit=true]", "count": 1 }
]
```

Each entry counts matching entries in the recorded change log — filtered by
`tick` (any tick when absent), `pos`, and `from`/`to` (property-subset matched
like a `blocks` check) — and bounds the count with `count`/`at_least`/
`at_most` (all absent: at least one). `kind` currently understands
`"block-changed"`.

Diagnostics are separate and always on: every failing case appends the
recorded block changes around the first failing tick (`--trace-window` in the
CLI, `RunOptions.trace_window` in code), so "what actually happened" arrives
without a rerun.

## `inert`: foreign blocks, by assertion

The engine refuses to half-simulate a world: a block with no behaviour fails
the run rather than sitting quietly. A foreign structure can still carry
blocks the engine does not model — lithium's `test_block` — so a case may
assert them inert for its own run:

```json
"inert": ["minecraft:test_block"]
```

This is the author's explicit claim, per case, never an engine default.

## The CLI

`nucleation test` runs every carrier from one place and prints a grid — one
row per file, one glyph per case (`✓` pass, `✗` fail, `∅` unported, `!`
unreadable), failures in full above it, exit 1 on any red:

```sh
cargo run -p nucleation-cli -- \
    run tests/scenarios crates/mc-tick/tests/cases
```

Carriers: any schematic nucleation's `FormatManager` detects (`.litematic`,
`.schem`, ... — the suite travels in the root `NucleationTest` tag), `.snbt`
structures with a `<stem>.test.json` sidecar or standalone descriptors, and
`--specs DIR` for spec trees that mirror a fetched corpus — see
`tests/corpus/lithium-specs/README.md` for that loop. `--json` emits machine
output; `--filter S` narrows by substring.
