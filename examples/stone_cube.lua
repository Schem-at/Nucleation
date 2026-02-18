local Schematic = require("nucleation")

local schem = Schematic.new("Stone Cube")
schem:set_author("Nucleation")

-- Fill a 10x10x10 cube with stone
schem:fill(0, 0, 0, 999, 999, 999, "minecraft:stone")

print("Set 1000 blocks")

schem:save("stone_cube.schematic")
schem:free()
