# Embedded scripting (Lua / JS)

## Scripting


Embedded Lua and JS engines run build scripts against the full API. This sine
wall is a 12-line Lua script run through `Scripting.run_lua_script`
([scripting guide](scripting.md)):

<img src="https://raw.githubusercontent.com/Schem-at/Nucleation/master/docs/media/scripting-wall.png" width="700" alt="A sine-wave wall built by an embedded Lua script with a concrete gradient">

---

## Reference

Nucleation embeds Lua and JavaScript engines (behind the `scripting-lua` /
`scripting-js` features, both in `bridge-full`) that generate schematics
from sandboxed scripts.

```python
wall = Scripting.run_lua_script(path)   # the script's global `result` comes back
js   = Scripting.run_js_script(path)
```

Inside the sandbox: `Schematic.new(name)` with the block-editing surface
(`set_block`, `fill_cuboid`, ...), plus the palette toolbox —
`palette_gradient_ids(name, r1,g1,b1, r2,g2,b2, steps)`,
`palette_block_ids(name)`, `palette_closest_block(name, r,g,b)`.

```lua
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
```

Output: 48×11×1, 287 blocks, red/orange/yellow concrete — the verified
run lives in
[`docs/readme-snippets/10-scripting-lua-python.md`](../readme-snippets/10-scripting-lua-python.md).

Current quirks: the entry points are file-path-only (no run-from-string in
the bridge yet), and `dimensions()` on a script's schematic reports the
allocated region — use `tight_dimensions()` for the content size.
