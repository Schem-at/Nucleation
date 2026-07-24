# The block database

## The block database


Under it all sits a block database extracted from Mojang's own data generator
and the vanilla jars: kinds, variant families, resolved tags, geometry, and
measured colors for all 1,196 Minecraft 26.2 blocks. It
[updates itself](block-database.md) when Mojang ships a new
version, and it's what lets palettes reason about color and brushes about block
facts:

```python
json.loads(Blocks.get_json("minecraft:oak_stairs"))
# {"kind": "minecraft:stair", "base_block": "minecraft:oak_planks",
#  "tags": ["minecraft:mineable/axe", ...], "full_cube": false, ...}

json.loads(Blocks.variants_of_json("minecraft:oak_planks"))
# [oak_planks, oak_button, oak_fence, oak_fence_gate, oak_pressure_plate, oak_slab, ...]
```

---

## Reference

The block database (formerly the standalone `blockpedia` crate) lives in-tree at
`src/blockpedia/`. Its data ships as gzipped snapshots in `data/blockpedia/` — currently pinned
to Minecraft **26.2** (Java block states from Mojang's own data generator, official block
semantics — kind/base-block/tags/full-cube geometry, Bedrock block states, Geyser blockstate
mappings, and a color cache derived from the vanilla texture pack) — and `build.rs` bakes them
into static PHF tables at compile time. Normal builds never touch the network.

## Refreshing for a new Minecraft version

To refresh the Java data for a new Minecraft version (needs a JRE new enough for the server
jar on `PATH`; MC 26.x wants Java 25+):

```bash
# 1. Vanilla report converter: downloads the server jar, runs Mojang's data
#    generator (--reports), and rebuilds prismarinejs_blocks.json.gz — the
#    report is authoritative for the block list, properties and state ids;
#    enrichment fields (transparency, hardness, light, ...) carry forward
#    from the previous snapshot, and blocks new in the version are enriched
#    from an analogue block or a model-shape heuristic over the client jar
#    (the run prints the added/removed diff and every derived fact).
#    Also rebuilds block_semantics.json.gz from official data only:
#    - kind + base block: the report's definition.type / definition.base_state
#      (stairs), plus a model-texture linkage for the other shape variants
#      (oak_slab renders with block/oak_planks, owned by oak_planks)
#    - tags: every data/minecraft/tags/block/** tag from the server jar's
#      inner (bundler) jar, nested #tag refs resolved
#    - full_cube: blockstate models root in a cube-family template or carry
#      a full 16x16x16 element
#    These drive BlockFacts::{kind, base_block, has_tag, is_full_cube},
#    blocks_by_tag/variants_of, and the BlockFilter/only_solid classifiers
#    (which no longer guess from name substrings).
cargo run --release --bin refresh-block-data --features mc-data-refresh

# 2. Texture colors: downloads the client jar, extracts block textures,
#    regenerates color_cache.json.gz (alpha-weighted averages + biome tints).
cargo run --release --bin fetch-texture-colors --features mc-data-refresh
```

Both tools take the version as an optional trailing arg (`-- 26.2`) and default to the
manifest's latest release, so a routine bump needs no code edits. A normal `cargo build`
afterwards bakes the new tables in.

The PrismarineJS `blocks.json` schema is kept as the on-disk format (PrismarineJS itself has
no 26.x data). `tests/blockpedia_data_refresh.rs` guards data currency.

## Automated refreshes

The refresh is also automated: `.github/workflows/data-refresh.yml` runs weekly (and on
manual dispatch, optionally with an explicit version), compares the version manifest's
`latest.release` against `data/blockpedia/DATA_VERSION` (a plain-text marker
`refresh-block-data` rewrites on every run), and — when Mojang has shipped a new release —
regenerates the snapshots and opens/updates a PR on `data-refresh/<version>`. The PR body
carries the added/removed-block diff, file size deltas, and color coverage. Two failure
modes are tolerated by design: `refresh-bedrock-mappings` may fail while GeyserMC's
mappings lag the Java release (noted in the PR, previous mappings kept), and `cargo test`
may fail because new blocks need human test updates (the PR is still opened, marked
failing, with the failure tail).

## Java ↔ Bedrock mappings

- `geyser_mappings.json.gz` — regenerated from **GeyserMC/mappings** (`blocks.nbt` @
  [`efe0f2c`](https://github.com/GeyserMC/mappings/commit/efe0f2cabeaf4c8147c45e63f2d744e90d3b4156),
  "Mappings for Minecraft Java **26.2**"). GeyserMC retired the old `mappings-generator`
  JSON dumps; the canonical data is now gzipped NBT: a `bedrock_mappings` list with one
  compound per Java blockstate **in runtime state-id order** (java side implicit by index;
  `bedrock_identifier` absent ⇒ same name as Java, `state` absent ⇒ bedrock default state).
  `refresh-bedrock-mappings` converts that back into the JSON schema `build.rs` consumes,
  reconstructing the java side from `prismarinejs_blocks.json.gz` (state-id enumeration
  validated 32,366/32,366 against the vanilla 26.2 report). All 32,366 Java 26.2 states are
  mapped (was 29,671 at the 1.21.x pin; all carried-over entries identical, +2,695 gained,
  none lost), so the identity fallback for unmapped blocks is currently unused:

  ```bash
  cargo run --release --bin refresh-bedrock-mappings --features mc-data-refresh -- \
      --data-version 4903   # java world data version, for provenance only
  ```

- `bedrock_block_states.json.gz` — PrismarineJS `data/bedrock/1.26.30/blockStates.json`
  gzipped verbatim (content-identical to the previous snapshot; the per-state `version`
  field `1.21.60.33` is Bedrock's *state-format* version, which hasn't bumped since —
  the palette content is current and includes the 26.x cinnabar/sulfur blocks). Every
  `bedrock_identifier` emitted by the mappings exists in this palette.
