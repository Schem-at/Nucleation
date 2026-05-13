# mcstructure accuracy TODO

Plan to make `.mcstructure` import/export round-trip correctly for redstone
and all common block types. Two repos involved:

- **nucleation** (this repo) — `src/formats/mcstructure.rs`
- **blockpedia** at `/Users/harrison/Documents/code/blockpedia/warp/blockpedia`
  (crates.io `blockpedia = "0.1.8"`, used by nucleation as the Java↔Bedrock
  oracle). Owned by the same author; safe to modify + push.

## Symptoms reported (mardi 12 mai 2026)

- **Hoppers always face down** in the exported `.mcstructure`.
- **Random block replacements** — some blocks come out as a different block.
- **Inverted / wrongly-rotated blocks** (orientation lost or mirrored).
- **Sticky pistons and redstone repeaters appear in the hologram preview but
  don't build** — Java palette name carries through but the placed block in
  Bedrock is wrong/missing.

## Root causes (confirmed by code reading)

### Bug A — Properties stripped before Java→Bedrock translation
`src/formats/mcstructure.rs:363` calls
`blockpedia::BlockState::parse(block.name.as_str())`. The first arg is the
bare ID; blockpedia's `parse()` (lib.rs:227) requires the bracketed form
(`minecraft:hopper[facing=north]`) to carry properties through. The
existing call always builds a default state, then asks blockpedia what the
Bedrock equivalent of "default hopper" is. Result: every hopper exports as
`facing_direction=0` (DOWN). Same mechanism breaks pistons, repeaters,
furnaces, droppers, observers, signs, stairs, slabs, fence-gates,
trapdoors — anything with directional state.

**This single bug explains ≥80% of the report.**

### Bug B — Java→Bedrock block-entity NBT translation is not called on export
`src/formats/mcstructure.rs:421` writes
`NbtMap::from_quartz_nbt(&be.to_nbt())` straight into `block_entity_data`.
That payload still has Java keys (`id: "minecraft:hopper"`, Java-format
`Items` slots, Java `TransferCooldown`, Java piston `extending`/`facing`
fields). Bedrock rejects it silently, leaving the placed block functional
but empty (hopper without items, sign without text, etc.) — or, for blocks
where the BE is load-bearing (piston-arm, command-block, jukebox), the
block doesn't materialise at all.

`blockpedia::BlockEntityTranslator` only has the reverse direction
(`translate_bedrock_to_java`), and even that covers just Chest, Barrel,
Shulker, TrappedChest, Comparator. The forward direction must be added.

### Bug C — Bedrock NBT type heuristic
`src/formats/mcstructure.rs:376` decides tag type from the value string
(`"true"`/`"false"`→Byte, parse-i32→Int, else String). Bedrock expects
specific tag types per property — e.g., `facing_direction` is sometimes
Int and sometimes Byte depending on the block, `block_light_filter` is
Int, `multi_face_direction_bits` is Int, etc. Low priority; fix once
Bug A is resolved and we can see what residual misreads remain.

### Index order — not a bug
Exporter and importer both use `X * (H*L) + Y * L + Z`. Round-trip
geometry is consistent.

---

## Nucleation tickets

### N-1. Pass full `name[props]` to blockpedia before translating (Bug A)
**Files:** `src/formats/mcstructure.rs` only.

Build the bracketed form before calling `parse`:

```rust
let full_id = if block.properties.is_empty() {
    block.name.to_string()
} else {
    let mut parts: Vec<String> = block.properties.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect();
    parts.sort();
    format!("{}[{}]", block.name, parts.join(","))
};
let (name, properties) =
    match blockpedia::BlockState::parse(&full_id).and_then(|s| s.to_bedrock()) {
        Ok(bed) => (
            bed.id().to_string(),
            bed.properties().iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        ),
        Err(_) => (block.name.to_string(), block.properties.clone()),
    };
```

Acceptance: round-trip tests in N-3 pass.

### N-2. Translate block-entity NBT on export (Bug B)
**Files:** `src/formats/mcstructure.rs`.

After computing `be_data_map`, run it through blockpedia's new
Java→Bedrock translator:

```rust
let bp_map = be_data_map.to_blockpedia();
let translated = blockpedia::block_entity::BlockEntityTranslator
    ::translate_java_to_bedrock(&bp_map);
let be_data_map = NbtMap::from_blockpedia(&translated);
```

(Uses `BP-2` below.) For unknown block-entity IDs, the translator should
pass through unchanged so we never regress existing working types.

Acceptance: round-trip hopper-with-items keeps items; signs keep text;
pistons retain extension state.

### N-3. Regression tests
**File:** new `tests/mcstructure_redstone_tests.rs`.

Each test builds a UniversalSchematic in Rust, calls `to_mcstructure`,
reloads via `from_mcstructure`, and asserts the property survived.

| Test | Input | Assertion |
|---|---|---|
| `hopper_facing_preserved` | `minecraft:hopper[facing=north]` at (0,0,0) | reloaded block has `facing=north` |
| `hopper_facing_down` | `minecraft:hopper[facing=down]` | reloaded `facing=down` |
| `repeater_full_state` | `minecraft:repeater[delay=3,facing=east,powered=true,locked=false]` | every property matches |
| `sticky_piston_extended` | `minecraft:sticky_piston[facing=up,extended=true]` | both match |
| `chest_orientation` | `minecraft:chest[facing=south,type=single,waterlogged=false]` | all three match |
| `stairs_half_top` | `minecraft:oak_stairs[facing=west,half=top,shape=straight,waterlogged=false]` | all match |
| `wall_torch_facing` | `minecraft:wall_torch[facing=south]` | matches |
| `observer_facing` | `minecraft:observer[facing=east,powered=true]` | both match |
| `comparator_full` | `minecraft:comparator[facing=north,mode=subtract,powered=true]` | all three match |
| `lever_pos` | `minecraft:lever[facing=east,face=wall,powered=true]` | all three match |

A second batch tests blocks with NBT (depends on N-2):

| Test | Setup | Assertion |
|---|---|---|
| `hopper_keeps_items` | hopper with stone in slot 0 | item present after round-trip |
| `sign_keeps_text` | oak_wall_sign with text on lines | text intact |
| `chest_keeps_items` | chest with items in 3 slots | all 3 slots intact |
| `dispenser_keeps_items` | dispenser with items | intact |
| `furnace_keeps_smelt_state` | furnace with cook progress | survives |

### N-4. Property-type table (Bug C, lower priority)
**File:** `src/formats/mcstructure.rs`.

Replace the value-shape heuristic with a name-keyed switch for the
properties we know Bedrock typing differs on. Source the table from the
Bedrock block-states catalogue (already shipped in
`blockpedia/data/bedrock_block_states.json`).

Defer until N-1 lands and we see what residual misreads remain in fixtures.

### N-5. Capture a real-world failing fixture
**File:** `tests/samples/community_report_2026-05-12.mcstructure` (or similar).

If the user can share one of the bugged exports, drop it in
`tests/samples/`. Add a fixture-driven test that imports it, re-exports
to a different name, and asserts that the relevant problem blocks
(hoppers, pistons, repeaters) come back with correct facing. Without a
real fixture, the synthetic tests in N-3 are still sufficient to catch
the regression.

### N-6. Symmetric `from_mcstructure` audit
**File:** `src/formats/mcstructure.rs:65–323`.

The importer already uses `translate_bedrock_to_java` for block entities
and re-parses block states. Once N-1 lands, re-read the importer for
parallel bugs — specifically, confirm that when blockpedia returns
`Err(_)` on `to_java`, the importer keeps the Bedrock state (rather than
silently dropping a property). Add a fuzz test that imports → exports →
imports a bedrock fixture and asserts equality.

---

## Blockpedia tickets (path: `/Users/harrison/Documents/code/blockpedia/warp/blockpedia`)

### BP-1. Audit existing Java→Bedrock state translations
**File:** `src/tests.rs` already contains
`repeater_java_to_bedrock_facing`, `powered_repeater_java_to_bedrock`,
`wall_torch_java_to_bedrock`, `stairs_java_to_bedrock_half`,
`redstone_wire_java_to_bedrock`. Run `cargo test` and confirm they pass
under the latest geyser mappings; if any are failing, those are
upstream-side issues the nucleation fix in N-1 will surface.

Also add:
- `hopper_java_to_bedrock_facing` covering all 5 valid facings
- `sticky_piston_java_to_bedrock` covering facing × extended
- `observer_java_to_bedrock` facing + powered
- `comparator_java_to_bedrock` mode + facing + powered
- `lever_java_to_bedrock` face × facing × powered
- `furnace_java_to_bedrock` facing + lit
- `dispenser_java_to_bedrock` / `dropper_java_to_bedrock` facing + triggered

If any mapping fails, fix via `data/geyser_mappings.json` (the authoritative
source). Don't hand-edit Rust — patch the JSON and rebuild data.

### BP-2. Add `translate_java_to_bedrock` for block entities
**File:** `src/block_entity.rs` (currently 84 lines).

Mirror the existing `translate_bedrock_to_java` skeleton:

```rust
pub fn translate_java_to_bedrock(
    nbt: &HashMap<String, NbtValue>,
) -> HashMap<String, NbtValue> { ... }
```

Coverage per BE type (minimum):

| Java BE id | Bedrock id | Notes |
|---|---|---|
| `minecraft:chest` | `Chest` | item-slot translation |
| `minecraft:trapped_chest` | `TrappedChest` | same |
| `minecraft:barrel` | `Barrel` | same |
| `minecraft:hopper` | `Hopper` | items + TransferCooldown |
| `minecraft:dispenser` | `Dispenser` | items |
| `minecraft:dropper` | `Dropper` | items |
| `minecraft:furnace` | `Furnace` | items, BurnTime, CookTime, CookTimeTotal |
| `minecraft:smoker` | `Smoker` | same as furnace |
| `minecraft:blast_furnace` | `BlastFurnace` | same |
| `minecraft:brewing_stand` | `BrewingStand` | items, FuelTotal, FuelAmount |
| `minecraft:shulker_box` (+ coloured variants) | `ShulkerBox` | items |
| `minecraft:sign` + variants + wall_sign + hanging_sign | `Sign` | front_text / back_text → Text1..4 |
| `minecraft:comparator` | `Comparator` | OutputSignal pass-through |
| `minecraft:beacon` | `Beacon` | Levels, Primary, Secondary |
| `minecraft:banner` | `Banner` | Patterns list |
| `minecraft:bed` | `Bed` | color |
| `minecraft:jukebox` | `Jukebox` | RecordItem |
| `minecraft:lectern` | `Lectern` | Book |
| `minecraft:piston` / `sticky_piston` | `PistonArm` | facing, progress, extending, state |
| `minecraft:command_block` (+ chain/repeating) | `CommandBlock` | Command, etc. |
| `minecraft:structure_block` | `StructureBlock` | metadata fields |
| `minecraft:end_portal` / `end_gateway` | … | pass-through |
| `minecraft:skull` (+ all heads) | `Skull` | SkullType, Owner |

For unknown ids, **return the input unchanged**. Never lossy-discard.

Tests in `tests/block_entity_test.rs` (new) — one per type, round-trip
NBT through Java→Bedrock→Java and assert equality on a canonical key set.

### BP-3. Item NBT translation (both directions)
**File:** `src/block_entity.rs` → expand `translate_item`.

Bedrock item shape differs from Java in several ways:
- Java `id`, `Count`, `tag` ↔ Bedrock `Name`, `Count`, `Damage`, `Block` (for placeable blocks), `tag`
- Damage: Java stores in `tag.Damage` (1.13+) or as component (1.20.5+); Bedrock stores `Damage` on the item itself
- Block items: Bedrock has a `Block` compound on the item that mirrors the placed block's state
- Slot indexing: matches across editions

Add:
```rust
fn translate_item_java_to_bedrock(item: &mut HashMap<String, NbtValue>) { ... }
fn translate_item_bedrock_to_java(item: &mut HashMap<String, NbtValue>) { ... }
```

Tests in `tests/item_translation_test.rs`: round-trip a stack of bricks,
a damaged pickaxe, a block (oak_log with axis=y), and a written book.

### BP-4. Per-property NBT type table
**File:** new `src/bedrock_property_types.rs` (or embedded in
`bedrock_mapping.rs`).

For each Bedrock state-property name, declare the expected NBT tag type
(Byte/Int/String). Used by nucleation in N-4. Source: existing
`data/bedrock_block_states.json` already contains type information from
the Bedrock catalogue; expose it as `pub fn bedrock_prop_type(name: &str)
-> Option<BedrockTagType>`.

### BP-5. Release & version bump
Once BP-1..BP-4 land:
- Bump to `0.1.9` in `blockpedia/Cargo.toml`
- `cargo publish` (after `cargo test`)
- Update nucleation's `Cargo.toml` to `blockpedia = "0.1.9"`

Until the release lands, nucleation can develop against blockpedia via a
local path override:

```toml
[patch.crates-io]
blockpedia = { path = "/Users/harrison/Documents/code/blockpedia/warp/blockpedia" }
```

(Add to nucleation's root `Cargo.toml`; remove before release.)

---

## Execution order

```
N-3 (regression tests, mostly failing) ──┐
                                          ├─► BP-1 (audit existing state translations)
N-1 (pass props to blockpedia) ───────────┘   │
                                              ▼
                            N-1 + N-3 hopper/repeater/piston tests pass
                                              │
                            BP-2 (Java→Bedrock BE translator)
                                              │
                            BP-3 (item translation)
                                              │
                            N-2 (wire BE translator into exporter)
                                              │
                          N-3 BE-keeps-items tests pass
                                              │
                  BP-4 + N-4 (typing polish, lower priority)
                                              │
                  N-5 (real-world fixture) + N-6 (importer audit)
                                              │
                  BP-5 release blockpedia 0.1.9 + bump nucleation
```

## Notes for the implementer

- Blockpedia repo is **clean** (no uncommitted changes, on `main` tracking
  `origin/main`). Safe to branch, commit, push.
- Nucleation repo has **7 untracked top-level items** (the JVM bindings
  work). They should be committed separately under the user's direction
  — do not commit them as part of mcstructure fixes.
- Both repos use the same author; license is AGPL-3.0 for nucleation and
  (check) for blockpedia.
- When testing locally, point nucleation at the local blockpedia via
  the `[patch.crates-io]` trick above so you don't need to publish for
  every iteration.
- The pre-push hook (`pre-push.sh` → `tools/prepush.py`) will pick up the
  new mcstructure tests automatically; no plumbing needed.
- The Bedrock mcstructure format reference Microsoft publishes is at
  <https://learn.microsoft.com/en-us/minecraft/creator/documents/mcstructureitems>
  — useful when typing fields in BP-4.
