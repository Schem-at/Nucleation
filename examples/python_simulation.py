#!/usr/bin/env python3
"""
Example demonstrating simulation and bracket notation support in Python bindings.

Install: pip install nucleation
"""

from nucleation import Schematic

# Create a new schematic
schematic = Schematic("Python Redstone Example")

# Set blocks using bracket notation (new in 0.1.73)
schematic.set_block(0, 0, 0, "minecraft:gray_concrete")
schematic.set_block(0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]")

# Add redstone wire
for x in range(1, 15):
    schematic.set_block(x, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]")

# Add lamp at the end
schematic.set_block(15, 0, 0, "minecraft:gray_concrete")
schematic.set_block(15, 1, 0, "minecraft:redstone_lamp[lit=false]")

print(f"Created schematic with {schematic.block_count} blocks")

# Create simulation world (new in 0.1.73)
sim_world = schematic.create_simulation_world()
print("Simulation world created")

# Check initial state
initial_lamp = sim_world.is_lit(15, 1, 0)
initial_lever = sim_world.get_lever_power(0, 1, 0)
print(f"Initial state: lever={initial_lever}, lamp={initial_lamp}")

# Toggle lever
sim_world.on_use_block(0, 1, 0)
sim_world.tick(2)
sim_world.flush()

# Check state after toggle
after_lever = sim_world.get_lever_power(0, 1, 0)
after_lamp = sim_world.is_lit(15, 1, 0)
print(f"After toggle: lever={after_lever}, lamp={after_lamp}")

# Toggle again
sim_world.on_use_block(0, 1, 0)
sim_world.tick(2)
sim_world.flush()

final_lamp = sim_world.is_lit(15, 1, 0)
print(f"After second toggle: lamp={final_lamp}")

# Save schematic
# save_schematic(schematic, "output.litematic", format="litematic")
