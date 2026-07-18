# Quickstart (Python)

```python
from nucleation import Schematic

cube = Schematic.load_from_file("simple_cube.litematic")   # any format, auto-detected
d = cube.dimensions()
print("dimensions:", (d.x, d.y, d.z))

cube.set_block(1, 3, 1, "minecraft:glowstone")             # the region grows to fit
print("new top:   ", cube.get_block_name(1, 3, 1))

cube.save_to_file("cube.schem")                            # format from the extension
back = Schematic.load_from_file("cube.schem")
print("round-trip:", back.get_block_name(1, 3, 1))
```

Output:

```text
dimensions: (3, 3, 3)
new top:    minecraft:glowstone
round-trip: minecraft:glowstone
```

_Environment: CPython 3.14.6 + nucleation 0.3.8 wheel (bridge-full, cp312-abi3), macOS arm64, run next to the repo-root `simple_cube.litematic`._

<!-- Since 0.3.8, load_from_file auto-detects the format from the file
contents and save_to_file picks the format from the extension (litematic
fallback for unknown extensions); save_to_file_with_format remains for
explicit format/version control. -->
