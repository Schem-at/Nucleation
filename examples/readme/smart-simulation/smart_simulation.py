"""Executable Python source for docs/features/smart-placement-and-simulation.md."""

import json
import os
from pathlib import Path

from nucleation import MchprsWorld, Schematic, TickSettleMode, TickSimulation


# --8<-- [start:author]
scene = Schematic.create("smart_circuit")
scene.fill_cuboid(0, 0, 0, 8, 0, 2, "minecraft:smooth_stone")
scene.set_block(0, 1, 0, "minecraft:lever[face=floor,facing=east,powered=false]")

# One engine setup, six placements. Each wire sees the state left by the last.
wire_positions = [coordinate for x in range(1, 7) for coordinate in (x, 1, 0)]
assert scene.set_blocks_simulated(wire_positions, "minecraft:redstone_wire") == 6

scene.set_block(7, 1, 0, "minecraft:redstone_lamp[lit=false]{simulate=true}")
scene.set_block(0, 1, 2, "minecraft:barrel[facing=west]{signal=13,item=iron_ingot}")
# --8<-- [end:author]


# --8<-- [start:mchprs]
world = MchprsWorld.create(scene)
assert not world.is_lit(7, 1, 0)
world.on_use_block(0, 1, 0)
world.tick(2)
world.flush()
assert world.is_lit(7, 1, 0)
assert world.get_redstone_power(6, 1, 0) == 10
# --8<-- [end:mchprs]


# --8<-- [start:tick]
tick = TickSimulation.from_schematic(scene, TickSettleMode.InWorld, 0, 0, 0, "")
tick.use_block(0, 1, 0)
tick.run(2)
assert tick.get_block(7, 1, 0) == "minecraft:redstone_lamp[lit=true]"
assert tick.tick_count() == 2
# --8<-- [end:tick]


wire = scene.get_block_string(3, 1, 0)
barrel = json.loads(scene.get_block_entity_json(0, 1, 2))
assert wire == "minecraft:redstone_wire[east=side,north=none,power=0,south=none,west=side]"
items = barrel["nbt"]["Items"]["List"]
assert items[0]["Compound"]["id"]["String"] == "minecraft:iron_ingot"
assert scene.block_count() == 36
size = scene.tight_dimensions()
assert (size.x, size.y, size.z) == (9, 2, 3)

output = Path(os.environ.get("SMART_SIMULATION_OUT", "smart-circuit.schem"))
output.parent.mkdir(parents=True, exist_ok=True)
scene.save_to_file(str(output))
print(f"Smart simulation Python example: OK ({output})")
