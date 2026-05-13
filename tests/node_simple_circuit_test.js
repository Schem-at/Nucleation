const path = require('path');
const assert = require('assert');

const wasmPath = path.join(__dirname, '../pkg/nucleation.js');

async function testSimpleCircuit() {
    console.log("Testing Simple Circuit Simulation...");

    // Load the WASM module — `pkg/nucleation.js` is an async initialiser
    // that reads the .wasm file and returns the wrapped exports. Tests that
    // construct types synchronously crash with
    // `Cannot read properties of undefined (reading 'schematicwrapper_new')`.
    let nucleation;
    try {
        nucleation = require(wasmPath);
        if (nucleation.default && typeof nucleation.default === 'function') {
            await nucleation.default();
        }
    } catch (error) {
        console.error('❌ Failed to load WASM module:', error);
        console.log('Make sure to build the WASM package first with: ./build-wasm.sh');
        process.exit(1);
    }

    const { SchematicWrapper, ExecutionModeWrapper } = nucleation;

    // 1. Build the Schematic
    const schematic = new SchematicWrapper("SimpleCircuit");

    // Place blocks: Lever (0,1,0) -> Wire (1,1,0) -> Lamp (2,1,0)
    // Support blocks underneath
    schematic.set_block_from_string(0, 1, 0, "minecraft:lever[facing=east,face=floor,powered=false]");
    schematic.set_block_from_string(1, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side]");
    schematic.set_block_from_string(2, 1, 0, "minecraft:redstone_lamp[lit=false]");

    for (let i = 0; i < 3; i++) {
        schematic.set_block_from_string(i, 0, 0, "minecraft:gray_concrete");
    }

    // 2. Define Regions
    schematic.createDefinitionRegionFromPoint("input_src", 0, 1, 0);
    schematic.createDefinitionRegionFromPoint("output_sink", 2, 1, 0);

    // 3. Create & Configure Executor
    const executor = schematic.buildExecutor({
        inputs: [
            { name: "switch", bits: 1, region: "input_src" },
        ],
        outputs: [
            { name: "lamp", bits: 1, region: "output_sink" },
        ],
    });

    // 4. Run Simulation — turn switch ON
    const inputs = { switch: 1 };
    const mode = ExecutionModeWrapper.untilStable(2, 1000);

    const result = executor.execute(inputs, mode);
    console.log("Simulation Result:", result);

    // Lamp should be ON (1). The executor wraps output values in `outputs`
    // alongside `ticksElapsed` / `conditionMet`.
    assert.strictEqual(
        result.outputs.lamp,
        1,
        "Lamp should be ON when switch is ON",
    );

    // 5. Sync & Verify
    const syncedSchematic = executor.syncToSchematic();
    const lampState = syncedSchematic.get_block_string(2, 1, 0);
    console.log("Synced Lamp State:", lampState);

    assert.ok(
        lampState.includes("lit=true"),
        "Lamp block state should be lit=true after sync",
    );

    console.log("Simple Circuit Test Passed!");
}

testSimpleCircuit().catch((e) => {
    console.error("Test Failed:", e);
    process.exit(1);
});
