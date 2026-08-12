from pathlib import Path

from nucleation import Schematic, Renderer, RenderConfig, ResourcePack

schem = Schematic.create("crossing")
pack = ResourcePack.from_bytes(Path("pack.zip").read_bytes())


def place_input(pos: tuple[int, int, int], word_size: int = 8, bus_block: str = "minecraft:gray_concrete"):
    x, y, z = pos
    # place a word of inputs stacked on the y-axis, 2-block pitch
    for y_offset in range(0, word_size * 2, 2):
        schem.set_block(x + 1, y + y_offset, z, bus_block)
        schem.set_block(x , y + y_offset -1, z, bus_block)
        schem.set_block(x , y + y_offset -1, z, bus_block)
        schem.set_block(x , y + y_offset, z, "minecraft:redstone_wire[power=0]")
        schem.set_block(x + 2,y + y_offset,z,"minecraft:lever[face=wall,facing=east,powered=false]")
        
def place_output(pos: tuple[int, int, int], word_size: int = 8, bus_block: str = "minecraft:gray_concrete"):
    x, y, z = pos
    # place a word of outputs stacked on the y-axis, 2-block pitch
    for y_offset in range(0, word_size * 2, 2):
        schem.set_block(x + 1 , y + y_offset -1, z, bus_block)
        schem.set_block(x + 1 , y + y_offset, z, "minecraft:redstone_wire[power=0]")
        schem.set_block(x ,y + y_offset,z,"minecraft:redstone_lamp[lit=false]")
        


input_a_location = (10, 0, 0)
input_b_location = (10, 0, 10)

output_a_location = (0, 0, 0)
output_b_location = (0, 0, 10)

place_input(input_a_location, word_size=8, bus_block="minecraft:lime_concrete")
place_input(input_b_location, word_size=8, bus_block="minecraft:cyan_concrete")

place_output(output_a_location, word_size=8, bus_block="minecraft:cyan_concrete")
place_output(output_b_location, word_size=8, bus_block="minecraft:lime_concrete")



design = schem.create_design()


input_a = design.create_input("input_a", input_a_location, bit_width=8, bus_direction=(0, 2, 0))
input_b = design.create_input("input_b", input_b_location, bit_width=8, bus_direction=(0, 2, 0))

output_a = design.create_output("output_a", output_a_location, bit_width=8, bus_direction=(0, 2, 0))
output_b = design.create_output("output_b", output_b_location, bit_width=8, bus_direction=(0, 2, 0))

bus_a = design.create_bus("bus_a", input_a, output_a, bus_block="minecraft:lime_concrete") # no transparent defaults to glass or something 
bus_b = design.create_bus("bus_b", input_b, output_b, bus_block="minecraft:cyan_concrete", transparent_block="minecraft:cyan_stained_glass")

# we could also have controle points to force the bus to go through certain points, or areas idk 

design.compile()


config = RenderConfig.create(1024, 1024)
config.set_isometric()
config.set_background(0.0, 0.0, 0.0, 0.0)

Renderer.render_to_file_with_pack(design, pack, config, "build.png") # maybe a design acts as a schematic ??? 
