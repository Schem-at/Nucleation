# Simulation: a typed circuit executor (Python)

Beyond poking blocks by hand, a `TypedCircuitExecutor` drives a circuit through
named, typed ports. Here an 8-bit `a` input is bound to a bus of levers and an
8-bit `y` output to the lamps; `execute` takes a **number** and the redpiler
sets the bus, with the typed output read straight back.

```python
import json
from nucleation import Schematic, CircuitBuilder, IoType, ExecutionMode

# An 8-bit bus: eight lever -> wire -> lamp lines.
bus = Schematic.create("bus")
levers, lamps = [], []
for i in range(8):
    z = i * 2
    for x in range(3):
        bus.set_block(x, 0, z, "minecraft:gray_concrete")
    bus.set_block_from_string(0, 1, z, "minecraft:lever[facing=east,face=floor,powered=false]")
    bus.set_block_from_string(1, 1, z, "minecraft:redstone_wire[power=0,east=side,west=side]")
    bus.set_block_from_string(2, 1, z, "minecraft:redstone_lamp[lit=false]")
    levers += [0, 1, z]     # flat [x, y, z, ...] positions, one triple per bit
    lamps  += [2, 1, z]

# Bind typed ports to the hardware and build the executor.
cb = CircuitBuilder.create(bus)
cb.with_input_auto("a", IoType.unsigned_int(8), levers)
cb.with_output_auto("y", IoType.unsigned_int(8), lamps)
ex = cb.build()

# Drive it by value: no wires toggled by hand.
for a in (42, 178):
    res = json.loads(ex.execute(json.dumps({"a": a}), ExecutionMode.until_stable(2, 100)))
    y = res["outputs"]["y"]["value"]
    print(f"execute a={a:>3} -> y={y:>3}  lamps={y:08b}  ({res['ticks_elapsed']} ticks)")

# The lit bus is a real schematic you can render or export.
lit = ex.sync_to_schematic()
print("bus dimensions:", tuple(getattr(lit.tight_dimensions(), d) for d in "xyz"),
      "blocks:", lit.block_count())
```

Output:

```text
execute a= 42 -> y= 42  lamps=00101010  (2 ticks)
execute a=178 -> y=178  lamps=10110010  (2 ticks)
bus dimensions: (3, 2, 15) blocks: 48
```

`with_input_auto` / `with_output_auto` infer the bit layout from the port
positions; `with_input` / `with_output` take an explicit `LayoutFunction`
(`row_major`, `scanline`, `packed4`, ...) for buses that are not one lever per
bit. Inputs can be booleans, signed or unsigned ints, `float32`, or ASCII, and
`ExecutionMode` also offers `fixed_ticks`, `until_change`, and `until_condition`.

_Environment: CPython 3.14.6 + nucleation 0.3.17 wheel (bridge-full, cp312-abi3), macOS arm64._
