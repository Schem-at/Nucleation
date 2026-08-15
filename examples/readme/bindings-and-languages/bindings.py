"""Executable Python source for docs/features/bindings-and-languages.md."""

import os
from pathlib import Path

from nucleation import Schematic


# --8<-- [start:build]
stack = Schematic.create("binding_stack")
stack.fill_cuboid(-3, 0, -3, 3, 0, 3, "minecraft:polished_deepslate")
stack.fill_cuboid(-2, 1, -2, 2, 1, 2, "minecraft:light_blue_concrete")
stack.fill_cuboid(-1, 2, -1, 1, 2, 1, "minecraft:yellow_concrete")
stack.set_block(0, 3, 0, "minecraft:emerald_block")

size = stack.tight_dimensions()
assert stack.block_count() == 84
assert (size.x, size.y, size.z) == (7, 4, 7)
# --8<-- [end:build]


output = Path(os.environ.get("BINDINGS_OUT", "binding-stack.schem"))
output.parent.mkdir(parents=True, exist_ok=True)
stack.save_to_file(str(output))
print(f"Bindings Python example: OK ({output})")
