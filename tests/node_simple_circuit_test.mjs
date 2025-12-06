import init, { SchematicWrapper, ExecutionModeWrapper } from '../pkg/nucleation.js';
import assert from 'assert';

async function testSimpleCircuit() {
    console.log("Testing Simple Circuit Simulation...");
    
    // Initialize Wasm
    await init();

    // 1. Build the Schematic
    const schematic = new SchematicWrapper();
    console.log("Schematic object:", schematic);
    console.log("Schematic prototype:", Object.getPrototypeOf(schematic));
    console.log("Available methods:", Object.getOwnPropertyNames(Object.getPrototypeOf(schematic)));

    // Place blocks: Lever (0,1,0) -> Wire (1,1,0) -> Lamp (2,1,0)
    // Support blocks underneath
    schematic.set_block(0, 1, 0, "minecraft:lever[facing=east,face=floor,powered=false]");
    schematic.set_block(1, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side]");
    schematic.set_block(2, 1, 0, "minecraft:redstone_lamp[lit=false]");
    
    for(let i = 0; i < 3; i++){
        schematic.set_block(i, 0, 0, "minecraft:gray_concrete");
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
    // result structure is { outputs: { lamp: 1 }, ticksElapsed: ..., conditionMet: ... }
    const outputs = result.outputs;
    const lamp = outputs ? outputs.lamp : undefined;

    assert.strictEqual(lamp, 1, `Lamp should be ON (1) when switch is ON. Got: ${lamp}`);

    // 5. Sync & Verify
    const syncedSchematic = executor.syncToSchematic();
    const lampState = syncedSchematic.get_block_string(2, 1, 0);
    console.log("Synced Lamp State:", lampState);
    
    assert.ok(lampState.includes("lit=true"), "Lamp block state should be lit=true after sync");

    console.log("Simple Circuit Test Passed!");
}

testSimpleCircuit().catch(e => {
    console.error("Test Failed:", e);
    process.exit(1);
});
