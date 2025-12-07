
const fs = require('fs');
const path = require('path');

// Adjust path to where your built WASM is located
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
        console.log('Make sure to build the WASM package first with: ./build-wasm.sh');
        process.exit(1);
    }

    const { SchematicWrapper } = nucleation;

    function createDummySchematic() {
        const schematic = new SchematicWrapper();
        // Create a small platform to define regions on
        for (let x = 0; x < 10; x++) {
            for (let z = 0; z < 10; z++) {
                schematic.set_block(x, 0, z, "minecraft:stone");
            }
        }
        return schematic;
    }

    function testMatrixConfig() {
        console.log('Testing Matrix Configuration...');
        const schem = createDummySchematic();
        
        // Define regions
        schem.createRegion("matrix_in", {x: 0, y: 1, z: 0}, {x: 5, y: 1, z: 6}); // 6x7 area
        schem.createRegion("matrix_out", {x: 0, y: 2, z: 0}, {x: 5, y: 2, z: 6});

        try {
            const executor = schem.buildExecutor({
                inputs: [
                    { 
                        name: "grid_in", 
                        region: "matrix_in",
                        type: "matrix", 
                        rows: 6, 
                        cols: 7, 
                        element: "boolean",
                        sort: "yxz"
                    }
                ],
                outputs: [
                    { 
                        name: "grid_out", 
                        region: "matrix_out",
                        type: "matrix", 
                        rows: 6, 
                        cols: 7, 
                        element: "boolean"
                    }
                ]
            });
            console.log('✅ Matrix config built successfully');
        } catch (e) {
            console.error('❌ Matrix config failed:', e);
            throw e;
        }
    }

    function testArrayConfig() {
        console.log('Testing Array Configuration...');
        const schem = createDummySchematic();
        
        // Define regions
        schem.createRegion("array_in", {x: 0, y: 1, z: 0}, {x: 7, y: 1, z: 0}); // 8 blocks
        schem.createRegion("array_out", {x: 0, y: 2, z: 0}, {x: 7, y: 2, z: 0});

        try {
            const executor = schem.buildExecutor({
                inputs: [
                    { 
                        name: "bytes_in", 
                        region: "array_in",
                        type: "array", 
                        length: 8, 
                        element: "boolean" // 8 booleans
                    }
                ],
                outputs: [
                    { 
                        name: "bytes_out", 
                        region: "array_out",
                        type: "array", 
                        length: 2, 
                        element: { type: "uint", bits: 4 } // 2 nibbles (4 bits each) = 8 bits total
                    }
                ]
            });
            console.log('✅ Array config built successfully');
        } catch (e) {
            console.error('❌ Array config failed:', e);
            throw e;
        }
    }

    function testHexConfig() {
        console.log('Testing Hex Configuration...');
        const schem = createDummySchematic();
        
        schem.createRegion("hex_in", {x: 0, y: 1, z: 0}, {x: 3, y: 1, z: 0}); // 4 blocks

        try {
            const executor = schem.buildExecutor({
                inputs: [
                    { 
                        name: "hex_val", 
                        region: "hex_in",
                        type: "hex" // Should map to uint bits: 4
                    }
                ],
                outputs: []
            });
            console.log('✅ Hex config built successfully');
        } catch (e) {
            console.error('❌ Hex config failed:', e);
            throw e;
        }
    }

    function testCustomSort() {
        console.log('Testing Custom Sort...');
        const schem = createDummySchematic();
        
        schem.createRegion("sorted_region", {x: 0, y: 1, z: 0}, {x: 2, y: 2, z: 2}); // 3x2x3 = 18 blocks

        const sortStrings = [
            "yxz",
            "-y+x-z",
            "zDescYX",
            "+x-z+y"
        ];

        for (const sortStr of sortStrings) {
            try {
                schem.buildExecutor({
                    inputs: [
                        { 
                            name: `sorted_${sortStr}`, 
                            region: "sorted_region",
                            type: "uint",
                            bits: 18,
                            sort: sortStr
                        }
                    ],
                    outputs: []
                });
                console.log(`✅ Sort '${sortStr}' built successfully`);
            } catch (e) {
                console.error(`❌ Sort '${sortStr}' failed:`, e);
                throw e;
            }
        }
    }

    function testComplexMixed() {
        console.log('Testing Complex Mixed Configuration...');
        const schem = createDummySchematic();
        
        schem.createRegion("p1", {x: 0, y: 1, z: 0}, {x: 5, y: 1, z: 6}); // 42 blocks
        schem.createRegion("p2", {x: 10, y: 1, z: 0}, {x: 15, y: 1, z: 6}); // 42 blocks
        schem.createRegion("out", {x: 20, y: 1, z: 0}, {x: 26, y: 1, z: 0}); // 7 blocks

        try {
            const executor = schem.buildExecutor({
                inputs: [
                    { 
                        name: "player_a", 
                        region: "p1",
                        type: "matrix", 
                        rows: 6, 
                        cols: 7, 
                        element: "boolean",
                        sort: "-y+x+z"
                    },
                    { 
                        name: "player_b", 
                        region: "p2",
                        type: "array", 
                        length: 7, 
                        element: { type: "uint", bits: 6 } // 7 * 6 = 42 bits
                    }
                ],
                outputs: [
                    { 
                        name: "result", 
                        region: "out",
                        type: "array", 
                        length: 7, 
                        element: "boolean"
                    }
                ]
            });
            console.log('✅ Complex mixed config built successfully');
        } catch (e) {
            console.error('❌ Complex mixed config failed:', e);
            throw e;
        }
    }

    try {
        testMatrixConfig();
        testArrayConfig();
        testHexConfig();
        testCustomSort();
        testComplexMixed();
        console.log('\n🎉 All tests passed!');
    } catch (e) {
        console.error('\n💥 Some tests failed');
        process.exit(1);
    }
}

runTests();
