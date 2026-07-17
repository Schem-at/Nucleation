# nucleation

A high-performance Minecraft schematic engine, powered by a native Rust core. Parse, edit,
diff, fingerprint, and generate schematics from Python.

Wheels are published for CPython 3.12+ (stable ABI) on Linux, macOS, and Windows.

## Install

```bash
pip install nucleation
```

## Quick start

```python
import nucleation

schematic = nucleation.Schematic.create("demo")
schematic.set_block(1, 2, 3, "minecraft:stone")
print(schematic.get_block_name(1, 2, 3))  # "minecraft:stone"

schematic.save_to_file("demo.litematic")
loaded = nucleation.Schematic.load_from_file("demo.litematic")
```

## What is included

The published wheel contains the core feature set: schematic editing, all schematic formats,
world import and export (including streaming), the schematic builder, the procedural building
tool, definition regions, diff and fingerprinting, autostack, NBT helpers, SDF sampling, and
the in-memory/filesystem store.

Redstone simulation, mesh generation, GPU rendering, and embedded scripting require building
the package from source with the extra cargo features enabled (a Rust toolchain is required):

```bash
git clone https://github.com/Schem-at/Nucleation
cd Nucleation
pip install ./bindings/python
```

The source build defaults to the full feature set (`bridge-full`). Set the
`NUCLEATION_FEATURES` environment variable to choose a different cargo feature list,
for example `NUCLEATION_FEATURES=bridge,simulation`.

## Documentation

- [Python API reference](https://github.com/Schem-at/Nucleation/blob/master/docs/python/README.md)
- [Feature guides](https://github.com/Schem-at/Nucleation/tree/master/docs/guides)

## License

MIT
