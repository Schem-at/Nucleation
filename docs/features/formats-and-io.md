# Formats and I/O

Nucleation uses one editable [`Schematic`](basics.md) model for every supported
container. Load by content, edit through the normal construction API, then choose
the output format explicitly or by file extension.

```python
from nucleation import Schematic

build = Schematic.load_from_file("build.litematic")
build.set_block(1, 3, 1, "minecraft:glowstone")
build.save_to_file("build.schem")
```

`load_from_file` inspects the bytes rather than trusting the filename. The
extension matters when writing: `.litematic`, `.schem`, `.mcstructure`, `.nusn`,
and `.zip` select their corresponding exporters. An unknown output extension is
an error; Nucleation does not silently choose a fallback format.

For coordinates, block placement, automatic region growth, and block-state
strings, start with [Basics](basics.md).

## Schematic containers

| Format | Extension | Read | Write | Exporter key | Notes |
| --- | --- | :---: | :---: | --- | --- |
| **Litematica** | `.litematic` | yes | yes | `litematic` | Multi-region native; the reference container |
| **Sponge Schematic** | `.schem` | yes | yes | `schematic` | WorldEdit/community format; exporter versions `v2` and `v3` |
| **Bedrock structure** | `.mcstructure` | yes | yes | `mcstructure` | Java block IDs and states are translated through GeyserMC mappings |
| **Nucleation snapshot** | `.nusn` | yes | yes | `snapshot` | Fast, uncompressed internal interchange format |
| **Legacy MCEdit** | `.schematic` | yes | no | — | Pre-Flattening numeric IDs; import only |

Use `.schem` for Sponge output. The modern Sponge exporter accepts
`.schematic` as an extension alias, but it does not produce the deprecated
MCEdit format.

## Worlds are a separate I/O surface

Anvil data can be much larger than a normal schematic container, so world APIs
support bounded imports and directory-oriented output:

- `.mca` region file: `Schematic.from_mca(...)` or
  `Schematic.from_mca_bounded(...)`;
- zipped world: `Schematic.from_world_zip(...)`, bounded variants, or generic
  content detection through `from_data`;
- world directory: `Schematic.from_world_directory(...)` or
  `from_world_directory_bounded(...)` on native targets;
- export: `save_world(...)`, `to_world_zip_b64(...)`, or `.zip` through the
  normal extension-based writer.

See [Chunk iteration, streaming, and worlds](streaming-and-worlds.md) for
constant-memory pipelines and world-generation workflows.

## Input detection is content-based

Both `load_from_file` and `from_data` run the registered format detectors over
the payload. Renaming a litematic to `mystery.bin` does not change what it is:

```python
from pathlib import Path
from nucleation import Schematic

payload = Path("mystery.bin").read_bytes()
build = Schematic.from_data(payload)
```

The detector recognizes Litematica, Sponge, Bedrock structures, Nucleation
snapshots, legacy MCEdit schematics, MCA region files, and zipped worlds. If no
format matches, loading fails instead of constructing a partial schematic.

## Byte pipelines

Generated bindings accept raw bytes on input. Serialized output crosses the
shared binding boundary as base64, so decode it before writing or sending binary
data:

```python
from base64 import b64decode
from pathlib import Path
from nucleation import Schematic

build = Schematic.from_data(Path("in.litematic").read_bytes())
build.set_block(1, 3, 1, "minecraft:glowstone")

encoded = build.save_as_b64("schematic", "v3", "")
Path("out.schem").write_bytes(b64decode(encoded))
```

The three arguments to `save_as_b64` are the exporter key, optional version,
and optional JSON settings. Pass an empty string for a format's default version
or settings. JavaScript/WASM has no filesystem methods, so this byte/base64 path
is its normal I/O workflow; generated bindings apply their usual language casing.

Convenience exporters such as `to_litematic_b64`, `to_schematic_b64`,
`to_mcstructure_b64`, `to_snapshot_b64`, and `to_world_zip_b64` are also
available when the target format is fixed.

## Explicit output format and version

`save_to_file` selects an exporter from the filename. Use
`save_to_file_with_format` when the filename is intentionally opaque or when a
specific Sponge version is required:

```python
build.save_to_file_with_format(
    "artifact.bin",
    "schematic",
    "v2",
)
```

The supported named exporter keys are `litematic`, `schematic`, `mcstructure`,
`snapshot`, and `world`. An unsupported key, version, or extension returns a
`NucleationError`; it never writes Litematic as an implicit fallback.

## Round-trip fidelity, measured

[`examples/readme/formats-and-io/round-trip.rs`](../../examples/readme/formats-and-io/round-trip.rs)
builds one 19-block fixture containing block-state properties and chest NBT. It
writes every schematic container with an exporter, auto-detects each result,
loads it, and compares a content-exact fingerprint with the original.

The current checkout produced:

| Format | Bytes | Content fingerprint | Result |
| --- | ---: | --- | --- |
| Litematica | 483 | `5654c8b94f558113be01dc6a31c0dc8d` | identical |
| Sponge `.schem` | 328 | `ab2071dca4aa393fe44697bf160ad00b` | equivalent content; reader adds an empty `components` compound |
| Snapshot `.nusn` | 70,202 | `5654c8b94f558113be01dc6a31c0dc8d` | identical |
| Bedrock `.mcstructure` | 945 | `b881f6eb62f4bd96b8185bcd7b9fad0f` | translated to Bedrock IDs and states |

Download the exact generated outputs:

- [`round-trip.litematic`](../downloads/readme/formats-and-io/round-trip.litematic)
- [`round-trip.schem`](../downloads/readme/formats-and-io/round-trip.schem)
- [`round-trip.nusn`](../downloads/readme/formats-and-io/round-trip.nusn)
- [`round-trip.mcstructure`](../downloads/readme/formats-and-io/round-trip.mcstructure)

“Identical” here means the canonical schematic content fingerprint matches, not
that compressed container bytes are byte-for-byte identical. Sponge preserves
the fixture's blocks, states, and chest data; its reader adds an empty Minecraft
1.20.5+ `components` placeholder, which is harmless but visible to an exact NBT
fingerprint. Bedrock is a translation boundary rather than a same-edition
round trip. See [Versions and translation](versions-and-translation.md).

Legacy MCEdit is intentionally import-only: its numeric-ID model cannot express
modern block states without loss.

## Next

- [Basics](basics.md) for editing the loaded schematic
- [Versions and translation](versions-and-translation.md) for Java/Bedrock and data-version changes
- [Chunk iteration, streaming, and worlds](streaming-and-worlds.md) for large-world I/O
- [Pluggable storage](storage.md) for memory, filesystem, S3, Redis, and Postgres URIs
