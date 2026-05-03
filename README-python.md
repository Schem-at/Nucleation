# Nucleation for Python

[![PyPI](https://img.shields.io/pypi/v/nucleation.svg)](https://pypi.org/project/nucleation)

Python bindings for **Nucleation**, a high-performance Minecraft schematic
engine. Read, edit, build, simulate, and mesh `.schematic`, `.litematic`,
`.nbt`, and `.mcstructure` files — at native speed.

The core is written in Rust and compiled to a CPython extension via PyO3
(`abi3-py38`), so a single wheel works on every CPython 3.8+ on your
platform — no Rust toolchain required to install.

---

## Install

```bash
pip install nucleation
```

Pre-built wheels are published for:

- Linux x86_64 and aarch64
- macOS x86_64 (Intel) and arm64 (Apple Silicon)
- Windows x86_64

If your platform isn't covered, `pip` will fall back to the source
distribution and build locally (requires a Rust toolchain).

---

## Quick start

```python
from nucleation import Schematic, sign, text

# Three explicit constructors (legacy `Schematic("file.schem")` still works):
schem = Schematic.open("example.litematic")        # load from disk
fresh = Schematic.new("my_schematic")              # blank
templ = Schematic.from_template("ab\ncd")          # ASCII template

# Polished set_block: tuple coords, structured state and NBT, chainable.
schem.set_block((0, 0, 0), "minecraft:repeater",
                state={"delay": 4, "facing": "east"})
schem.set_block((0, 1, 0), "minecraft:oak_sign",
                state={"rotation": 8},
                nbt=sign([text("Hello", color="gold"), "world"]))

# Format inferred from extension.
schem.save("out.litematic")
```

### Hot loops

For placing many blocks, prefer the batch and fast-path APIs:

```python
# 30+ M placements/sec — one native call.
schem.set_blocks([(x, 0, 0) for x in range(1_000_000)], "minecraft:stone")

# 10+ M placements/sec — pre-resolve once, place by index.
stone = schem.prepare_block("minecraft:stone")
place = schem.place
for x, y, z in positions:
    place(x, y, z, stone)
```

### Tile-entity helpers

```python
from nucleation import Schematic, chest, sign, text, Item

schem = Schematic.new("loot_room")

schem.set_block((0, 0, 0), "minecraft:chest",
                state={"facing": "north"},
                nbt=chest([
                    Item("minecraft:diamond", count=64),
                    Item("minecraft:elytra"),
                ]))

schem.set_block((0, 1, 0), "minecraft:oak_sign",
                nbt=sign([text("Loot", color="gold"), "this way →"]))
```

---

## Documentation

Full reference, including simulation, meshing, and resource-pack rendering,
lives in the main repository:

- [Nucleation on GitHub](https://github.com/Schem-at/Nucleation)
- [Python API reference](https://github.com/Schem-at/Nucleation/blob/master/docs/api-reference-python.md)
- [Schematic Builder guide](https://github.com/Schem-at/Nucleation/blob/master/docs/guide/schematic-builder.md)

---

## Why Rust?

Nucleation's core is shared across Rust, WebAssembly/JS, Python, and C/PHP —
one engine, one set of behaviour, one set of tests. The Python package you
install is the same engine that powers the JS and Rust libraries; you get
native throughput for free.

## License

AGPL-3.0-only. See the [main repository](https://github.com/Schem-at/Nucleation)
for details.
