# `dataconverter` — Minecraft datafixers

A Rust port of [PaperMC/spottedleaf's DataConverter](https://github.com/PaperMC/DataConverter)
(`DataConverterJava/`), restricted to the NBT shapes that appear in a schematic
file: **block states, block entities, entities, and item stacks**. It converts
that data between Minecraft *data versions* so a schematic loaded from one game
version can be edited and re-saved for another.

## Model

- **Canonical version** — all loaded data is forward-converted to a single
  in-memory data version, `CANONICAL_DATA_VERSION` (currently `4790`). Editing
  always happens at canonical; "save as current" stamps this version.
- **Forward conversion** (old → new) mirrors the Java engine exactly and is
  lossless.
- **Reverse conversion** (new → old) is needed to save for an older version. It
  is built on inverse rename tables plus hand-written best-effort inverses, and
  is inherently lossy — every approximation or dropped field is recorded in a
  `LossReport` so callers can warn the user before writing. Conversion is never
  silently destructive.

## Public API

The entry points most callers use live on `UniversalSchematic`
(`convert_to_data_version`, `convert_to_canonical`) and in the litematic exporter
(`to_litematic_for_data_version`); every binding re-exposes them
(`convertToVersion` / `convert_to_version`, `toLitematicForVersion`,
`canonicalDataVersion`, …). Directly from this module:

- `CANONICAL_DATA_VERSION` / `DATACONVERTER_FORWARD_MAX` — version constants.
- `apply::convert_schematic` / `apply::convert_schematic_reverse` — convert a
  `UniversalSchematic` in place; the reverse form returns a `LossReport`.
- `loss::{LossReport, LossEntry, LossKind, Severity}` — the loss model. Each
  entry is `{version, kind, severity, path, detail}`.
- `registry::registry` — the lazily-built, immutable converter registry.
- `version::{get_version, get_step, encode_versions}` — version-tree navigation.

## Layout

| Path | Role |
|------|------|
| `engine.rs` | Tree walker + conversion hooks (`walk_with_breakpoints[_reverse]`). |
| `registry.rs` | Builds the immutable registry of all data types + per-version converters. |
| `apply.rs` | High-level `convert_schematic[_reverse]` entry points. |
| `version.rs` | Version encoding / breakpoint navigation. |
| `loss.rs` | `LossReport` and the loss taxonomy. |
| `types.rs`, `components.rs`, `helpers.rs`, `walker.rs` | Shared NBT type defs + helpers. |
| `flattening/` | The 1.13 "flattening" block/item tables. |
| `versions/vXXXX.rs` | One file per data-version step; each registers its forward (and, where written, reverse) converters into the registry. |

## Authoring reverse converters

See [`REVERSE_CHEATSHEET.md`](./REVERSE_CHEATSHEET.md). Each `versions/vXXXX.rs`
cites the upstream Java source it was ported from
(`ca/spottedleaf/dataconverter/minecraft/versions/V<XXX>.java`); that Java tree
is **not vendored here** — clone PaperMC/DataConverter separately to cross-check
semantics.
