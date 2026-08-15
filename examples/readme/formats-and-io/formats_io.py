"""Executable Python source for docs/features/formats-and-io.md."""

from base64 import b64decode
import os
from pathlib import Path

from nucleation import Schematic


# --8<-- [start:build]
build = Schematic.create("round_trip")
build.fill_cuboid(0, 0, 0, 3, 0, 3, "minecraft:stone_bricks")
build.set_block(1, 1, 1, "minecraft:oak_stairs[facing=east,half=bottom]")
build.set_block(2, 1, 1, "minecraft:lever[face=floor,facing=east,powered=false]")
build.set_block_with_nbt(
    0, 1, 0,
    "minecraft:chest[facing=south]",
    '{"CustomName":"Treasure"}',
)
# --8<-- [end:build]


# --8<-- [start:bytes]
payload = b64decode(build.save_as_b64("litematic", "", ""))
loaded = Schematic.from_data(payload)  # content detection; no filename required
assert loaded.block_count() == 19

v3 = b64decode(loaded.save_as_b64("schematic", "v3", ""))
Path("round-trip.schem").write_bytes(v3)
# --8<-- [end:bytes]


formats = {
    "litematic": ("", ".litematic"),
    "schematic": ("v3", ".schem"),
    "structure_snbt": ("", ".snbt"),
    "snapshot": ("", ".nusn"),
    "mcstructure": ("", ".mcstructure"),
}
output = Path(os.environ.get("FORMATS_IO_OUT_DIR", "formats-output"))
output.mkdir(parents=True, exist_ok=True)
for format_name, (version, extension) in formats.items():
    data = b64decode(build.save_as_b64(format_name, version, ""))
    back = Schematic.from_data(data)
    assert back.block_count() == 19
    (output / f"round-trip{extension}").write_bytes(data)

print(f"Formats and I/O Python example: OK ({output})")
