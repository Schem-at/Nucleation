# Nucleation documentation

Start with the [project README](../README.md) — installation, the basics, and
an illustrated tour. Everything here goes deeper.

Since v0.3.0 every binding is generated from one source of truth
(`src/bridge/`), so **the API is the same everywhere**: same types, same
methods, per-language casing (`set_block` in Rust/Python, `setBlock` in
JS/Kotlin/PHP), unified `NucleationError` errors.

## Feature guides

- [Shapes, brushes, and masked fills](guides/shapes-and-brushes.md)
- [Palettes: turning colors into blocks](guides/palettes.md)
- [SDF shapes and terrain](guides/sdf-terrain.md)
- [Embedded scripting (Lua / JS)](guides/scripting.md)
- [Auto-stack: detect and resize repetition](autostack.md)
  ([design notes](autostack-design.pdf))
- [The Minecraft block database](guides/minecraft-block-data.md) — data
  provenance, the 26.2 refresh pipeline, Bedrock mappings
- [Insign IO integration](insign-io-integration.md) — executors from sign
  annotations
- [Meshing, .nucm, and rendering](meshing-nucm-rendering.md)

## Per-language references

- [Rust](rust/) · [JavaScript](javascript/) · [Python](python/) ·
  [Kotlin](kotlin/) · [PHP](php/) · [C](c/) · [C++](cpp/)

## Verified examples

Every snippet in the README ran for real with captured output:
[`docs/readme-snippets/`](readme-snippets/). The README's images regenerate
from [`tools/readme-media/generate.py`](../tools/readme-media/generate.py).

## Formats

`.litematic` · Sponge `.schem` · WorldEdit `.schematic` · Bedrock
`.mcstructure` · structure `.nbt` · `.nusn` (fast binary snapshot) — with
auto-detection — plus world folders (Anvil region files) in both directions.

## License

MIT — see [LICENSE](../LICENSE).
