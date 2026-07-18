# Embedded Lua scripting (Python)

```python
import tempfile, pathlib
from nucleation import Scripting

LUA = """
-- A sine-wave wall, colored bottom-to-top by a concrete gradient.
local wall = Schematic.new("sine_wall")
local ramp = palette_gradient_ids("concrete", 200, 60, 40, 255, 220, 80, 8)

local width, base, amp = 48, 6, 5
for x = 0, width - 1 do
  local h = base + math.floor(amp * math.sin(x * 2 * math.pi / 24) + 0.5)
  for y = 0, h - 1 do
    wall:set_block(x, y, 0, ramp[math.floor(y * (#ramp - 1) / (base + amp - 1)) + 1])
  end
end

result = wall  -- global `result` is what the host receives
"""

path = pathlib.Path(tempfile.mkdtemp()) / "sine_wall.lua"
path.write_text(LUA)
wall = Scripting.run_lua_script(str(path))

d = wall.tight_dimensions()
print("dimensions:", (d.x, d.y, d.z))
print("blocks:    ", wall.block_count())
print("palette:   ", wall.palette_json())
```

Output:

```text
dimensions: (48, 11, 1)
blocks:     287
palette:    ["minecraft:air","minecraft:red_concrete","minecraft:orange_concrete","minecraft:yellow_concrete"]
```

_Environment: CPython 3.14.6 + nucleation 0.3.6 wheel (bridge-full, cp312-abi3), macOS arm64._

<!-- `Scripting.run_lua_script` is file-path-only (no run-from-string entry point in the bridge), hence the tempfile dance. On the returned schematic `dimensions()` reports the allocated (region-padded) box (66, 66, 1) -- `tight_dimensions()` is the real content size. -->
