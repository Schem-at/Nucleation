# Block entities, entities & NBT (Python)

```python
from nucleation import Schematic

s = Schematic.create("loot")

# A chest with contents, set straight from SNBT:
s.set_block_entity(0, 0, 0, "minecraft:chest",
    '{Items:[{Slot:0b,id:"minecraft:diamond",Count:3b},'
            '{Slot:1b,id:"minecraft:emerald",Count:5b}]}')
print("chest:", s.get_block_entity_snbt(0, 0, 0))

# Entities parse from SNBT too:
s.add_entity_from_snbt('{id:"minecraft:armor_stand",Pos:[0.5d,1.0d,0.5d],Rotation:[0f,0f]}')
print("entity_count:", s.entity_count())
```

Output:

```text
chest: {Items:[{Count:3B,id:"minecraft:diamond",Slot:0B},{Slot:1B,id:"minecraft:emerald",Count:5B}]}
entity_count: 1
```

_Environment: CPython 3.14.6 + nucleation 0.3.10 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- SNBT compound key order is not stable (it round-trips through a map), so compare structurally, not textually. `set_block_with_nbt` exists too but takes a flat string→string map — use `set_block_entity` for compound NBT like Items. -->
