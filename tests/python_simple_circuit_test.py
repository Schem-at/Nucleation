
import pytest
from nucleation import Schematic

def test_simple_circuit_simulation():
    print("Testing Simple Circuit Simulation (Python)...")

    # 1. Build the Schematic
    schematic = Schematic("SimpleCircuit")

    # Place blocks: Lever (0,1,0) -> Wire (1,1,0) -> Lamp (2,1,0)
    # Support blocks underneath
    schematic.set_block(0, 1, 0, "minecraft:lever[facing=east,face=floor,powered=false]")
    schematic.set_block(1, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side]")
    schematic.set_block(2, 1, 0, "minecraft:redstone_lamp[lit=false]")
    
    for i in range(3):
        schematic.set_block(i, 0, 0, "minecraft:gray_concrete")

    # 2. Define Regions
    schematic.create_definition_region_from_point("input_src", 0, 1, 0)
    schematic.create_definition_region_from_point("output_sink", 2, 1, 0)

    # 3. Create & Configure Executor
    executor = schematic.build_executor(
        inputs=[
            {"name": "switch", "bits": "1", "region": "input_src"}
        ],
        outputs=[
            {"name": "lamp", "bits": "1", "region": "output_sink"}
        ]
    )

    # 4. Run Simulation
    # Turn switch ON
    inputs = {"switch": 1}
    # Note: Python API might handle execution mode differently or use defaults
    # Assuming execute takes inputs and optional ticks/mode
    result = executor.execute(inputs, max_ticks=1000)
    print("Simulation Result:", result)

    # Assertions
    # Lamp should be ON (1)
    assert result["lamp"] == 1, "Lamp should be ON when switch is ON"

    # 5. Sync & Verify
    synced_schematic = executor.sync_to_schematic()
    lamp_state = synced_schematic.get_block_string(2, 1, 0)
    print("Synced Lamp State:", lamp_state)
    
    assert "lit=true" in lamp_state, "Lamp block state should be lit=true after sync"

    print("Simple Circuit Test Passed!")

if __name__ == "__main__":
    test_simple_circuit_simulation()
