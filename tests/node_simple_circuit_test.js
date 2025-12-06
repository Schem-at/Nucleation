
const { SchematicWrapper, ExecutionModeWrapper, BlockPosition } = require('../pkg/nucleation');
const assert = require('assert');

function testSimpleCircuit() {
    console.log("Testing Simple Circuit Simulation...");

    // 1. Build the Schematic
    const schematic = new SchematicWrapper("SimpleCircuit");

    // Place blocks: Lever (0,1,0) -> Wire (1,1,0) -> Lamp (2,1,0)
    // Support blocks underneath
    schematic.setBlock(0, 1, 0, "minecraft:lever[facing=east,face=floor,powered=false]");
    schematic.setBlock(1, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side]");
    schematic.setBlock(2, 1, 0, "minecraft:redstone_lamp[lit=false]");
    
    for(let i = 0; i < 3; i++){
        schematic.setBlock(i, 0, 0, "minecraft:gray_concrete");
    }

    // 2. Define Regions
    schematic.createDefinitionRegionFromPoint("input_src", 0, 1, 0);
    schematic.createDefinitionRegionFromPoint("output_sink", 2, 1, 0);

    // 3. Create & Configure Executor
    const executor = schematic.buildExecutor({
        inputs: [
            { name: "switch", bits: 1, region: "input_src" }
        ],
        outputs: [
            { name: "lamp", bits: 1, region: "output_sink" }
        ]
    });

    // 4. Run Simulation
    // Turn switch ON
    const inputs = { "switch": 1 };
    const mode = ExecutionModeWrapper.untilStable(2, 1000);

    const result = executor.execute(inputs, mode);
    console.log("Simulation Result:", result);

    // Assertions
    // Lamp should be ON (1)
    assert.strictEqual(result.lamp, 1, "Lamp should be ON when switch is ON");

    // 5. Sync & Verify
    const syncedSchematic = executor.syncToSchematic();
    const lampState = syncedSchematic.getBlockString(2, 1, 0);
    console.log("Synced Lamp State:", lampState);
    
    assert.ok(lampState.includes("lit=true"), "Lamp block state should be lit=true after sync");

    console.log("Simple Circuit Test Passed!");
}

// Run the test
try {
    testSimpleCircuit();
} catch (e) {
    console.error("Test Failed:", e);
    process.exit(1);
}
