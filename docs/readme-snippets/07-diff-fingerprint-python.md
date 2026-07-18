# Diff & fingerprint (Python)

```python
from nucleation import Schematic, Diff, Fingerprint

before = Schematic.create("before")
before.fill_cuboid(0, 0, 0, 4, 4, 4, "minecraft:stone")
after = Schematic.create("after")
after.fill_cuboid(0, 0, 0, 4, 4, 4, "minecraft:stone")
after.set_block(2, 5, 2, "minecraft:torch")  # one extra block

diff = Diff.compute(before, after, "exact")
print("edit distance:", diff.distance())
print("summary:", diff.summary_json())
print("duplicates?", Fingerprint.is_duplicate(before, after, "exact"))
print("fingerprint:", Fingerprint.compute(before, "exact"))
```

Output:

```text
edit distance: 1
summary: {"counts":{"added":1,"changed":0,"removed":0,"swapped":0},"distance":1,"regions":[{"count":1,"kind":"Added","max":[2,5,2],"min":[2,5,2]}],"support":0.9920634627342224,"swaps":[],"translate":[0,0,0]}
duplicates? False
fingerprint: 3fdae2c9855e4794b30f9895b0d31a2c
```

_Environment: CPython 3.14.6 + nucleation 0.3.3 wheel (bridge-full, cp312-abi3), macOS arm64._
