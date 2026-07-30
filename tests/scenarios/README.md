# Self-testing builds

Every `.litematic` here carries its own test. Drop a file in, and
`cargo test --test litematic_cases` runs it — nothing recompiles, and there is no
Rust anywhere that names the build.

The descriptor lives inside the schematic, in a root-level `NucleationTest`
compound beside `Metadata`. Format and assertion vocabulary:
[`crates/mc-tick/tests/cases/README.md`](../../crates/mc-tick/tests/cases/README.md).
Runner: [`tests/litematic_cases.rs`](../litematic_cases.rs).

```text
cargo test --test litematic_cases                             # all of them
MC_TICK_CASE=55_3x3 cargo test --test litematic_cases         # one
```

## Adding one

Measure the build, write the numbers down, put them in the file:

```text
cargo run --release --example scenario_inspect -- mybuild.litematic \
    --settle in-world --ticks 400 --cells 3,0,20:5,2,20 --every 1
# ... read off the entity count, the seats, the changes, the fill ...
cargo run --example scenario_inspect -- mybuild.litematic \
    --embed spec.json --write tests/scenarios/mybuild.litematic
```

`--dump-test` gets a descriptor back out for editing, so the only copy of a
scenario is the one inside the file it tests. `scenario_inspect` prints exactly
the quantities a descriptor can assert, on purpose: nothing it reports is
measured *only* there.

## What is here

- **`55_3x3.litematic`** — the record 3x3 piston door, extracted from
  `tests/samples/55_3x3.zip`, DataVersion 4082. Two scenarios in one file:
  untouched it changes no block in 400 ticks and stays quiescent with all 24 of
  its entities and both blaze riders on their exact seats; pressed at tick 5 it
  opens 6 of its 9 doorway cells, settles at 4, records 227 block changes,
  comes to rest, and drops nothing below y=0.

  **6 of 9 is today's truth, not the goal.** The door does not close yet. The
  number is pinned so that a regression reads as a different number instead of
  as nothing at all; when the door does close, this becomes 9 and the settled
  fill becomes 0.

  Its DataVersion is not decoration. 4082 selects the `Entity.load` rules that
  *keep* a NaN velocity, and this door is glued together by nan carts — minecarts
  whose velocity was overflowed to ±Infinity and then collided. Read at the wrong
  version they load as ordinary carts, the door quietly comes apart, and nothing
  reports an error. `litematic_round_trips_the_record_doors_data_version` pins the
  version through a save, and
  `litematic_preserves_an_embedded_test_across_a_resave` pins the test through
  one — a build opened in-game, nudged and re-saved must come back still knowing
  how to test itself.
