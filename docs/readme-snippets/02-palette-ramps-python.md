# Palette ramps (Python)

```python
import json
from nucleation import Palette, Schematic

sunset = Palette.wool()  # all 16 wool colors, ready for gradient snapping
ramp = json.loads(sunset.gradient_ids_json(255, 80, 40, 60, 40, 180, 8))

strip = Schematic.create("sunset_strip")
for x, wool in enumerate(ramp):
    strip.set_block(x, 0, 0, wool)

print("\n".join(ramp))
```

Output:

```text
minecraft:orange_wool
minecraft:red_wool
minecraft:red_wool
minecraft:red_wool
minecraft:magenta_wool
minecraft:purple_wool
minecraft:purple_wool
minecraft:blue_wool
```

_Environment: CPython 3.14.6 + nucleation 0.3.3 wheel (bridge-full, cp312-abi3), macOS arm64._
