"""Executable Python source for docs/features/data-driven-generation.md."""

# --8<-- [start:example]
from PIL import Image
from nucleation import Schematic


def barrel_position(x, y, channel):
    alternate = x & 1
    z = 5 + alternate if (channel & 1) == (y & 1) else 5 * alternate
    return (
        -((channel + y) & 1),
        -2 - channel - 3 * y,
        6 * (x // 2) + z - 2,
    )
rom = Schematic.create("image_rom")

with Image.open("rom-input.png") as source:
    image = source.convert("RGB")

    for index, (red, green, blue) in enumerate(image.getdata()):
        x, y = index % image.width, index // image.width

        for channel, signal in enumerate((blue >> 4, green >> 4, red >> 4)):
            rom.set_block(
                *barrel_position(x, y, channel),
                f"minecraft:barrel{{signal={signal}}}",
            )

rom.save_to_file("image-rom.schem")
# --8<-- [end:example]


assert rom.block_count() == 16 * 10 * 3
assert rom.tight_dimensions().x == 2
print("Data-driven Python example: OK")
