# Reverse converter authoring cheatsheet

We are building the **reverse** (new → old) direction of a Minecraft DataConverter
port. Forward converters already exist in each
`nucleation/src/dataconverter/versions/vXXXX.rs`. Your job: add the **inverse** of
each data-mutating forward converter, so saving a schematic for an older Minecraft
version restores the older NBT shape — and **report any unavoidable data loss**.

The Java source for `vXXXX.rs` is
`DataConverterJava/src/main/java/ca/spottedleaf/dataconverter/minecraft/versions/V<XXX>.java`
(strip the `v`, uppercase). Read it for exact semantics; the Rust forward file
already cites it.

## Engine API (already built — use exactly these)

Compound types (`DataType`): `reg.block_state`, `reg.tile_entity`, `reg.item_stack`,
`reg.entity`, `reg.data_components`, `reg.structure`, `reg.untagged_spawner`,
`reg.entity_equipment`, `reg.text_component`, `reg.particle`, `reg.villager_trade`.
- `reg.TYPE.add_reverse_converter(version, step, Box::new(|data: &mut NbtMap, _from, _to| { ... }))`
- `reg.TYPE.add_reverse_converter_for_id("<NEW id>", version, step, Box::new(|data, _from, _to| { ... }))`

Value types (`MCValueType`): `reg.block_name`, `reg.item_name`, `reg.entity_name`,
`reg.flat_block_state`, `reg.biome`, `reg.game_event_name`.
- `reg.VALUETYPE.add_reverse_converter(version, step, Box::new(|val: &mut NbtValue, _from, _to| { ... }))`

Loss reporting (call from inside a reverse converter when data can't be perfectly
restored):
```rust
use super::super::loss::{report_loss, LossKind, Severity};
report_loss(VERSION, LossKind::ComponentDropped, Severity::Loss, "minecraft:custom_model_data has no legacy tag");
```
`LossKind`: `FlatteningAmbiguous`, `FlatteningUnknownBlock`, `ItemFlatteningDamage`,
`ComponentDropped`, `EntityMergeAmbiguous`, `RenameAmbiguous`, `FingerprintCollapse`,
`UnsupportedInTarget`, `Other`. `Severity`: `Loss` (unrecoverable) | `Approximated`
(best-effort substitution, usually correct).

## Critical rules

1. **Renames via `register_block_rename` / `register_item_rename` /
   `register_entity_rename` / `register_value_rename` with `map_renamer(TABLE)` are
   ALREADY auto-inverted by the engine. DO NOT add a reverse for those.** Only invert
   `add_structure_converter`, `add_converter_for_id`, and direct value-type
   `.add_converter(...)` forward converters. (A `register_entity_rename` that takes a
   *closure* renamer — not `map_renamer` — has a no-op reverse by default; if you see
   one, note it but it is handled separately.)
2. Register each reverse converter at the **same `(version, step)`** as the forward
   converter it inverts.
3. **Order semantics.** When your reverse converter runs: every newer version has
   already been undone, and the walker has already recursed into and downgraded all
   children. So you operate on data in *this version's forward-output schema*.
4. **`add_reverse_converter_for_id` matches the NEW id** (the forward converter's
   output id), because any inverse id-rename runs later in the descending sweep.
5. **Lossless inverse (buckets A/B)**: write the exact inverse, no `report_loss`.
   **Lossy (bucket C)**: best-effort + `report_loss(... Loss ...)` (or `Approximated`
   when the substitution is usually right). **Additive defaults (bucket D)**: remove
   the field the forward added (ideally only when it equals the known default); no
   report unless removal could drop user data.
6. **Walkers need NO reverse** — they only locate sub-structures and are
   direction-independent.
7. Mirror the Java faithfully; cite line numbers in comments like the forward code.
8. **Only edit your assigned `vXXXX.rs`.** Never touch `mod.rs`, `registry.rs`,
   `engine.rs`, `helpers/`, `loss.rs`, `walker.rs`, or any other version file.
9. Add reverse registrations *inside the existing `pub fn register(reg)`*, right
   after the forward registration they mirror. Add any needed `use` imports.
10. If a forward converter is a pure normalization/repair with no information change
    (e.g. canonicalizing JSON, clamping), its inverse is identity → **no reverse
    converter needed**; say so in your summary rather than writing a no-op.
11. **Lossy means MODERN data can't determine the old value — not "the forward was
    many-to-one for arbitrary inputs".** A split where the *new* id/field uniquely
    encodes the old discriminator is LOSSLESS for real downgrades — do NOT
    `report_loss`. Example: post-split `minecraft:skeleton` is unambiguously a
    normal skeleton, so reverse → `SkeletonType=0` is exact (no loss); only
    `WitherSkeleton`/`Stray`-style ids that were genuinely merged into one modern
    id with no surviving discriminator are lossy. Restoring a default the old
    format always carried (e.g. adding back `SkeletonType=0`) is exact, not loss.
12. **Self-recursive restructure converters (the walker recurses the SAME type your
    converter restructures).** Reverse descends the walker FIRST, so by the time
    your reverse converter runs, the walker has ALREADY reverse-converted your
    nested children of that type. Do NOT rebuild the whole subtree — operate on the
    already-converted children. See `v135.rs` (Riding/Passengers): the reverse
    attaches this node at the *bottom* of the child's already-rebuilt `Riding`
    chain instead of re-nesting from scratch. This only matters when the forward
    converter relocates nested compounds of a type the walker also recurses
    (entity Passengers/Riding, item `tag` nesting). Most converters are local and
    unaffected.

## Worked examples

**Structural split, lossless (v107 — Minecart `Type` → typed id):**
```rust
// forward: id "Minecart" + int Type 0..6  ->  typed id (MinecartChest, ...), Type removed.
// reverse: typed id -> "Minecart" + Type. The new id encodes Type, so this is exact.
for (idx, mid) in ["MinecartRideable","MinecartChest","MinecartFurnace","MinecartTNT",
                   "MinecartSpawner","MinecartHopper","MinecartCommandBlock"].iter().enumerate() {
    let ty = idx as i32;
    reg.entity.add_reverse_converter_for_id(mid, VERSION, 0, Box::new(move |d, _, _| {
        d.set_string("id", "Minecart");
        d.set_i32("Type", ty);
    }));
}
```
(`add_reverse_converter_for_id` takes `&'static str`; loop over a const array, capturing `ty` by `move`.)

**Additive default (v4054 — ominous banner gains `minecraft:rarity`):**
```rust
// reverse: drop the rarity the forward added for ominous banners (bucket D).
reg.tile_entity.add_reverse_converter_for_id("minecraft:banner", VERSION, 0, Box::new(|d,_,_| {
    if let Some(c) = d.get_map_mut("components") {
        // only drop when it is the value the forward set
        if c.get_string("minecraft:rarity") == Some("uncommon") { c.take("minecraft:rarity"); }
    }
}));
```

**Entity merge, lossy (skeleton variants via a removed discriminator):** reconstruct
the discriminator from the split id where the id encodes it; if the merge is genuinely
ambiguous, pick the canonical preimage and `report_loss(VERSION,
LossKind::EntityMergeAmbiguous, Severity::Approximated, "...")`.
