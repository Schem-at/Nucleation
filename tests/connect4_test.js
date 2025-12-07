
const fs = require('fs');
const path = require('path');

const wasmPath = path.join(__dirname, '../pkg/nucleation.js');

async function runTests() {
    let nucleation;
    try {
        nucleation = require(wasmPath);
        if (nucleation.default && typeof nucleation.default === 'function') {
            await nucleation.default();
        }
        console.log('✅ WASM module loaded successfully');
    } catch (error) {
        console.error('❌ Failed to load WASM module:', error);
        process.exit(1);
    }

    const { SchematicWrapper, ExecutionModeWrapper } = nucleation;

    async function testSchematic(filename, expectedBehavior) {
        console.log(`\nTesting ${filename}...`);
        const schemPath = path.join(__dirname, 'samples', filename);
        const buffer = fs.readFileSync(schemPath);
        const schem = new SchematicWrapper();
        schem.from_data(buffer);

        const input_min = {x:0,y:5,z:6};
        const input_max = {x:0,y:25,z:32};
        const output_min = {x:51,y:26,z:7};
        const output_max = {x:51,y:26,z:31};
        
        // Map "oponent" to "player_a" and "self" to "player_b" for consistency with executor config
        // User code:
        // schem.createRegion("oponent", input_min, input_max).addFilter("yellow")...
        // schem.createRegion("self", input_min, input_max).addFilter("red")...
        
        // We will use "player_a" for opponent (yellow) and "player_b" for self (red)
        schem.createRegion("player_a", input_min, input_max)
            .addFilter("yellow")
            .setColor(0xffff00)
            .shift(0,1,0);

        schem.createRegion("player_b", input_min, input_max)
            .addFilter("red")
            .setColor(0xff0000)
            .shift(0,1,0);
            
        schem.createRegion("output", output_min, output_max)
            .addFilter("repeater")
            .setColor(0x00ff00);

        const executor = schem.buildExecutor({
            inputs: [
                { 
                    name: "player_a", 
                    region: "player_a",
                    type: "matrix", 
                    rows: 6, 
                    cols: 7, 
                    element: "boolean",
                    sort: "-yxz" 
                },
                { 
                    name: "player_b", 
                    region: "player_b",
                    type: "matrix", 
                    rows: 6, 
                    cols: 7, 
                    element: "boolean",
                    sort: "-yxz" 
                }
            ],
            outputs: [
                { 
                    name: "output", 
                    region: "output",
                    type: "array", 
                    length: 7, 
                    element: "boolean" 
                }
            ]
        });

        // Empty board
        const emptyInputs = { 
            "player_a": Array(6).fill().map(() => Array(7).fill(false)), 
            "player_b": Array(6).fill().map(() => Array(7).fill(false))
        };
        
        const mode = ExecutionModeWrapper.fixedTicks(20);
        // Warmup / Reset?
        // executor.execute(emptyInputs, mode);

        // Test Case: Opponent plays in column 3 (index 3)
        // Note: User said "add a coin in the oponent column it should reply with a output with the same index column"
        // Let's try placing a piece in the bottom row (index 5? or 0? depending on sort)
        // Sort is "-yxz". y goes 5 to 25. -y means 25 down to 5? Or inverted axis?
        // If rows=6, cols=7.
        
        // Let's just set one true in the matrix and see what happens.
        const inputs = JSON.parse(JSON.stringify(emptyInputs));
        // Set a piece in column 3. Which row? Let's try bottom row.
        // If sort is -y, maybe index 0 is top or bottom?
        // Let's just set the whole column to be sure, or just one piece.
        inputs["player_a"][5][3] = true; // Bottom row?

        console.log("Executing with input in col 3...");
        const result = executor.execute(inputs, mode).outputs;
        console.log("Result:", result);
        
        if (expectedBehavior === 'copy') {
             // Expect output index 3 to be true
             if (result.output[3]) {
                 console.log("✅ Success: Output matches input column.");
             } else {
                 console.log("❌ Failure: Output did not match input column.");
             }
        } else if (expectedBehavior === 'leftmost') {
            // Expect output index 0 (leftmost) to be true, assuming it's available
            if (result.output[0]) {
                console.log("✅ Success: Output is leftmost column.");
            } else {
                console.log("❌ Failure: Output is not leftmost column.");
            }
        }
    }

    await testSchematic('c4_ai_last_played.schem', 'copy');
    await testSchematic('c4_ai_left_most.schem', 'leftmost');
}

runTests().catch(console.error);
