# Quickstart (Python)

```python
from nucleation import Schematic

cube = Schematic.load_from_file("simple_cube.litematic")
d = cube.dimensions()
print("dimensions:", (d.x, d.y, d.z))
print("palette:   ", cube.palette_json())

cube.set_block(1, 3, 1, "minecraft:glowstone")  # crown the cube
print("new top:   ", cube.get_block_name(1, 3, 1))

cube.save_to_file_with_format("simple_cube.schem", "", "")  # format from extension
print("saved simple_cube.schem")
```

Output:

```text
dimensions: (3, 3, 3)
palette:    ["minecraft:air","minecraft:stone","minecraft:dirt","minecraft:oak_planks"]
new top:    minecraft:glowstone
saved simple_cube.schem
```

_Environment: CPython 3.14.6 + nucleation 0.3.3 wheel (bridge-full, cp312-abi3), macOS arm64, run next to the repo-root `simple_cube.litematic`._

<!-- `save_to_file` always writes Litematic regardless of extension; `save_to_file_with_format(path, "", "")` auto-detects the format from the extension, so that is the idiomatic save. -->
