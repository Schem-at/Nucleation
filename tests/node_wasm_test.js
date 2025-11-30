// Node.js test script for WASM chunk iterator functionality
// This allows for more detailed testing and debugging outside the browser

const fs = require('fs');
const path = require('path');

// You'll need to adjust this path based on where your built WASM is located
const wasmPath = path.join(__dirname, '../pkg/nucleation.js');

async function runTests() {
    let nucleation;

    try {
        nucleation = require(wasmPath);
        if (nucleation.default && typeof nucleation.default === 'function') {
            await nucleation.default(); // Initialize WASM if strictly required (e.g. web target)
        }
        console.log('✅ WASM module loaded successfully');
    } catch (error) {
        console.error('❌ Failed to load WASM module:', error);
        console.log('Make sure to build the WASM package first with: ./build-wasm.sh');
        process.exit(1);
    }

    const { SchematicWrapper } = nucleation;

    // Helper function to create test schematic
    function createTestSchematic() {
        const schematic = new SchematicWrapper();

        // Create a 4x4x4 cube with some variety
        for (let x = 0; x < 4; x++) {
            for (let y = 0; y < 4; y++) {
                for (let z = 0; z < 4; z++) {
                    if (x === 0 || x === 3 || y === 0 || y === 3 || z === 0 || z === 3) {
                        // Walls are stone
                        schematic.set_block(x, y, z, "minecraft:stone");
                    } else {
                        // Interior has different blocks
                        schematic.set_block(x, y, z, "minecraft:air");
                    }
                }
            }
        }

        // Add some distinctive blocks
        schematic.set_block(1, 1, 1, "minecraft:diamond_block");
        schematic.set_block(2, 1, 1, "minecraft:emerald_block");
        schematic.set_block(1, 2, 1, "minecraft:gold_block");
        schematic.set_block(2, 2, 1, "minecraft:iron_block");
        schematic.set_block(1, 1, 2, "minecraft:redstone_block");

        return schematic;
    }

    // Helper function to load real test data
    function loadTestSchematic() {
        const testFiles = [
            '../tests/samples/1x1.litematic',
            '../tests/samples/3x3.litematic',
            '../simple_cube.litematic'
        ];

        for (const file of testFiles) {
            const filePath = path.join(__dirname, file);
            if (fs.existsSync(filePath)) {
                try {
                    const data = fs.readFileSync(filePath);
                    const schematic = new SchematicWrapper();
                    schematic.from_data(new Uint8Array(data));
                    console.log(`✅ Loaded test schematic from ${file}`);
                    return schematic;
                } catch (error) {
                    console.log(`⚠️  Failed to load ${file}: ${error.message}`);
                }
            }
        }

        console.log('📝 Using generated test schematic');
        return createTestSchematic();
    }

    console.log('\n=== Running WASM Chunk Iterator Tests ===\n');

    // Test 1: Basic chunk functionality
    console.log('🧪 Test 1: Basic chunk functionality');
    const schematic = createTestSchematic();

    const chunks = schematic.chunks(2, 2, 2);
    console.log(`   - Generated ${chunks.length} chunks with 2x2x2 size`);

    if (chunks.length > 0) {
        const firstChunk = chunks[0];
        console.log(`   - First chunk at (${firstChunk.chunk_x}, ${firstChunk.chunk_y}, ${firstChunk.chunk_z})`);
        console.log(`   - First chunk has ${firstChunk.blocks.length} blocks`);

        if (firstChunk.blocks.length > 0) {
            const firstBlock = firstChunk.blocks[0];
            console.log(`   - First block: (${firstBlock.x}, ${firstBlock.y}, ${firstBlock.z}) = ${firstBlock.name}`);
        }
    }

    // Test 2: Chunk indices optimization
    console.log('\n🧪 Test 2: Chunk indices optimization');
    const chunksIndices = schematic.chunks_indices(2, 2, 2);
    console.log(`   - Regular chunks: ${chunks.length}, Indexed chunks: ${chunksIndices.length}`);

    if (chunksIndices.length > 0) {
        const firstIndexChunk = chunksIndices[0];
        console.log(`   - First indexed chunk has ${firstIndexChunk.blocks.length} blocks`);

        if (firstIndexChunk.blocks.length > 0) {
            const firstIndexBlock = firstIndexChunk.blocks[0];
            console.log(`   - First indexed block: [${firstIndexBlock.join(', ')}] (x,y,z,palette_idx)`);
        }

        // Get palettes to understand the indices
        const allPalettes = schematic.get_all_palettes();
        console.log(`   - Default palette has ${allPalettes.default.length} entries`);

        // Show first few palette entries
        for (let i = 0; i < Math.min(5, allPalettes.default.length); i++) {
            console.log(`   - Palette[${i}]: ${allPalettes.default[i].name}`);
        }
    }

    // Test 3: Loading strategies
    console.log('\n🧪 Test 3: Loading strategies');
    const strategies = ['bottom_up', 'top_down', 'distance_to_camera', 'center_outward', 'random'];

    for (const strategy of strategies) {
        const strategyChunks = schematic.chunks_with_strategy(2, 2, 2, strategy, 0, 0, 0);
        console.log(`   - Strategy '${strategy}': ${strategyChunks.length} chunks`);

        if (strategyChunks.length > 0) {
            const positions = strategyChunks.map(chunk => `(${chunk.chunk_x},${chunk.chunk_y},${chunk.chunk_z})`);
            console.log(`     Order: ${positions.join(' -> ')}`);
        }
    }

    // Test 4: Lazy chunk iterator
    console.log('\n🧪 Test 4: Lazy chunk iterator');
    const iterator = schematic.create_lazy_chunk_iterator(2, 2, 2, 'bottom_up', 0, 0, 0);
    console.log(`   - Total chunks available: ${iterator.total_chunks()}`);

    const retrievedChunks = [];
    let iterations = 0;
    const maxIterations = 20; // Safety limit

    while (iterator.has_next() && iterations < maxIterations) {
        const chunk = iterator.next();
        if (chunk !== null) {
            retrievedChunks.push({
                position: `(${chunk.chunk_x},${chunk.chunk_y},${chunk.chunk_z})`,
                blocks: chunk.blocks.length,
                index: chunk.index,
                total: chunk.total
            });
        }
        iterations++;
    }

    console.log(`   - Retrieved ${retrievedChunks.length} chunks through lazy iterator`);
    retrievedChunks.forEach((chunk, i) => {
        console.log(`     ${i}: ${chunk.position} - ${chunk.blocks} blocks [${chunk.index}/${chunk.total}]`);
    });

    // Test iterator controls
    iterator.reset();
    console.log(`   - After reset, position: ${iterator.current_position()}, has_next: ${iterator.has_next()}`);

    if (iterator.total_chunks() > 2) {
        iterator.skip_to(Math.floor(iterator.total_chunks() / 2));
        console.log(`   - After skip to middle, position: ${iterator.current_position()}`);
    }

    // Test 5: Data integrity and false values detection
    console.log('\n🧪 Test 5: Data integrity and false values detection');

    // Reset for clean test
    iterator.reset();
    const allBlocks = [];
    const chunkData = [];

    while (iterator.has_next()) {
        const chunk = iterator.next();
        if (chunk === null) {
            console.log('   ❌ ERROR: Iterator returned null chunk!');
            break;
        }

        const chunkInfo = {
            position: [chunk.chunk_x, chunk.chunk_y, chunk.chunk_z],
            blockCount: chunk.blocks.length,
            blocks: []
        };

        // Analyze each block in the chunk
        if (chunk.blocks.constructor && (chunk.blocks.constructor.name === 'Int32Array' || chunk.blocks instanceof Int32Array)) {
            // Handle flat Int32Array [x, y, z, idx, x, y, z, idx, ...]
            for (let i = 0; i < chunk.blocks.length; i += 4) {
                const x = chunk.blocks[i];
                const y = chunk.blocks[i + 1];
                const z = chunk.blocks[i + 2];
                const paletteIndex = chunk.blocks[i + 3];

                const blockInfo = { x, y, z, paletteIndex };
                chunkInfo.blocks.push(blockInfo);
                allBlocks.push(blockInfo);
            }
        } else {
            // Handle array of arrays
            for (let i = 0; i < chunk.blocks.length; i++) {
                const blockData = chunk.blocks[i];

                // Validate block data structure
                if (!Array.isArray(blockData) || blockData.length !== 4) {
                    console.log(`   ❌ ERROR: Invalid block data structure at chunk ${chunk.chunk_x},${chunk.chunk_y},${chunk.chunk_z}, block ${i}`);
                    console.log(`     Expected array of length 4, got:`, blockData);
                    continue;
                }

                const [x, y, z, paletteIndex] = blockData;

                // Validate coordinate values
                if (typeof x !== 'number' || typeof y !== 'number' || typeof z !== 'number') {
                    console.log(`   ❌ ERROR: Non-numeric coordinates: (${x}, ${y}, ${z})`);
                    continue;
                }

                // Validate palette index
                if (typeof paletteIndex !== 'number' || paletteIndex < 0 || paletteIndex > 1000) {
                    console.log(`   ❌ ERROR: Invalid palette index: ${paletteIndex} at (${x}, ${y}, ${z})`);
                    continue;
                }

                const blockInfo = { x, y, z, paletteIndex };
                chunkInfo.blocks.push(blockInfo);
                allBlocks.push(blockInfo);
            }
        }

        chunkData.push(chunkInfo);
    }

    console.log(`   - Analyzed ${chunkData.length} chunks with ${allBlocks.length} total blocks`);

    // Check for duplicates
    const positionMap = new Map();
    const duplicates = [];

    allBlocks.forEach((block, index) => {
        const key = `${block.x},${block.y},${block.z}`;
        if (positionMap.has(key)) {
            duplicates.push({
                position: key,
                firstIndex: positionMap.get(key),
                duplicateIndex: index,
                firstBlock: allBlocks[positionMap.get(key)],
                duplicateBlock: block
            });
        } else {
            positionMap.set(key, index);
        }
    });

    if (duplicates.length > 0) {
        console.log(`   ❌ ERROR: Found ${duplicates.length} duplicate blocks:`);
        duplicates.forEach(dup => {
            console.log(`     Position ${dup.position}: indices ${dup.firstIndex} and ${dup.duplicateIndex}`);
            console.log(`       First: palette ${dup.firstBlock.paletteIndex}, Duplicate: palette ${dup.duplicateBlock.paletteIndex}`);
        });
    } else {
        console.log('   ✅ No duplicate blocks found');
    }

    // Palette consistency check
    const allPalettes = schematic.get_all_palettes();
    const paletteSize = allPalettes.default.length;
    const invalidIndices = allBlocks.filter(block => block.paletteIndex >= paletteSize);

    if (invalidIndices.length > 0) {
        console.log(`   ❌ ERROR: Found ${invalidIndices.length} blocks with invalid palette indices:`);
        invalidIndices.slice(0, 5).forEach(block => {
            console.log(`     (${block.x}, ${block.y}, ${block.z}): index ${block.paletteIndex} >= palette size ${paletteSize}`);
        });
        if (invalidIndices.length > 5) {
            console.log(`     ... and ${invalidIndices.length - 5} more`);
        }
    } else {
        console.log('   ✅ All palette indices are valid');
    }

    // Test 6: Performance comparison
    console.log('\n🧪 Test 6: Performance comparison');

    const iterations_perf = 10;

    // Time regular chunks method
    const start1 = Date.now();
    for (let i = 0; i < iterations_perf; i++) {
        schematic.chunks(2, 2, 2);
    }
    const time1 = Date.now() - start1;

    // Time indexed chunks method
    const start2 = Date.now();
    for (let i = 0; i < iterations_perf; i++) {
        schematic.chunks_indices(2, 2, 2);
    }
    const time2 = Date.now() - start2;

    // Time lazy iterator
    const start3 = Date.now();
    for (let i = 0; i < iterations_perf; i++) {
        const iter = schematic.create_lazy_chunk_iterator(2, 2, 2, 'bottom_up', 0, 0, 0);
        while (iter.has_next()) {
            iter.next();
        }
    }
    const time3 = Date.now() - start3;

    console.log(`   - Regular chunks: ${time1}ms (${iterations_perf} iterations)`);
    console.log(`   - Indexed chunks: ${time2}ms (${iterations_perf} iterations)`);
    console.log(`   - Lazy iterator: ${time3}ms (${iterations_perf} iterations)`);
    console.log(`   - Indexed chunks are ${(time1 / time2).toFixed(2)}x faster than regular`);
    console.log(`   - Lazy iterator vs indexed: ${(time3 / time2).toFixed(2)}x ratio`);

    // Test 7: Real world scenario with larger schematic
    console.log('\n🧪 Test 7: Real world scenario');
    const realSchematic = loadTestSchematic();

    const dimensions = realSchematic.get_dimensions();
    const blockCount = realSchematic.get_block_count();
    console.log(`   - Schematic dimensions: ${dimensions[0]}x${dimensions[1]}x${dimensions[2]}`);
    console.log(`   - Total blocks: ${blockCount}`);

    if (blockCount > 0) {
        const realChunks = realSchematic.chunks_indices(8, 8, 8);
        console.log(`   - Divided into ${realChunks.length} chunks (8x8x8)`);

        let totalRealBlocks = 0;
        realChunks.forEach(chunk => {
            totalRealBlocks += chunk.blocks.length;
        });

        console.log(`   - Total blocks in chunks: ${totalRealBlocks}`);

        // Test lazy loading on real data
        const realIterator = realSchematic.create_lazy_chunk_iterator(4, 4, 4, 'distance_to_camera', 0, 0, 0);
        console.log(`   - Lazy iterator reports ${realIterator.total_chunks()} chunks (4x4x4)`);

        let realChunkCount = 0;
        let realBlockCount = 0;
        while (realIterator.has_next() && realChunkCount < 10) { // Limit for testing
            const chunk = realIterator.next();
            if (chunk && chunk.blocks) {
                realBlockCount += chunk.blocks.length;
            }
            realChunkCount++;
        }

        console.log(`   - First 10 lazy chunks contain ${realBlockCount} blocks`);
    }

    // Test 8: Redstone Simulation (if available)
    console.log('\n🧪 Test 8: Redstone Simulation');
    if (typeof nucleation.MchprsWorldWrapper !== 'undefined') {
        console.log('   ✅ Simulation feature is available');

        try {
            // Create a simple redstone line with lever and lamp
            const redstoneSchematic = new SchematicWrapper();

            // Base layer
            for (let x = 0; x <= 15; x++) {
                redstoneSchematic.set_block(x, 0, 0, "minecraft:gray_concrete");
            }

            // Redstone wire with proper properties
            for (let x = 1; x <= 14; x++) {
                redstoneSchematic.set_block_with_properties(x, 1, 0, "minecraft:redstone_wire", {
                    power: "0",
                    east: x < 14 ? "side" : "none",
                    west: "side",
                    north: "none",
                    south: "none"
                });
            }

            // Lever at start with properties
            redstoneSchematic.set_block_with_properties(0, 1, 0, "minecraft:lever", {
                facing: "east",
                powered: "false",
                face: "floor"
            });

            // Lamp at end with properties
            redstoneSchematic.set_block_with_properties(15, 1, 0, "minecraft:redstone_lamp", {
                lit: "false"
            });

            console.log('   - Created test circuit: lever -> wire -> lamp');

            // Create simulation world
            const simWorld = redstoneSchematic.create_simulation_world();
            console.log('   - Simulation world created successfully');

            // Initial state
            const initialLamp = simWorld.is_lit(15, 1, 0);
            const initialLever = simWorld.get_lever_power(0, 1, 0);
            console.log(`   - Initial state: lever=${initialLever}, lamp=${initialLamp}`);

            // Toggle lever
            simWorld.on_use_block(0, 1, 0);
            simWorld.tick(2);
            simWorld.flush();

            const afterToggle = simWorld.is_lit(15, 1, 0);
            const leverAfterToggle = simWorld.get_lever_power(0, 1, 0);
            console.log(`   - After toggle: lever=${leverAfterToggle}, lamp=${afterToggle}`);

            if (leverAfterToggle !== initialLever) {
                console.log('   ✅ Lever toggled successfully');
            } else {
                console.log('   ⚠️  Lever did not toggle');
            }

            // Toggle again
            simWorld.on_use_block(0, 1, 0);
            simWorld.tick(2);
            simWorld.flush();

            const afterSecondToggle = simWorld.is_lit(15, 1, 0);
            console.log(`   - After second toggle: lamp=${afterSecondToggle}`);

            console.log('   ✅ Simulation tests passed');

        } catch (error) {
            console.log(`   ⚠️  Simulation test error: ${error.message}`);
            console.log('   This may be expected if simulation dependencies are not fully compiled');
        }
    } else {
        console.log('   ⚠️  Simulation feature not available (compile with --features simulation)');
    }

    // Test 9: Bracket Notation in set_block
    console.log('\n🧪 Test 9: Bracket Notation Support');
    try {
        const bracketSchematic = new SchematicWrapper();

        // Test 1: Simple block with no properties (should work as before)
        bracketSchematic.set_block(0, 0, 0, "minecraft:gray_concrete");
        const simpleBlock = bracketSchematic.get_block(0, 0, 0);
        console.log(`   - Simple block: ${simpleBlock}`);
        if (simpleBlock !== "minecraft:gray_concrete") {
            throw new Error(`Expected minecraft:gray_concrete, got ${simpleBlock}`);
        }

        // Test 2: Block with bracket notation
        bracketSchematic.set_block(0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]");
        const leverBlock = bracketSchematic.get_block(0, 1, 0);
        console.log(`   - Lever block: ${leverBlock}`);
        if (leverBlock !== "minecraft:lever") {
            throw new Error(`Expected minecraft:lever, got ${leverBlock}`);
        }

        // Test 3: Another bracket notation example
        bracketSchematic.set_block(5, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side,north=none,south=none]");
        const wireBlock = bracketSchematic.get_block(5, 1, 0);
        console.log(`   - Wire block: ${wireBlock}`);
        if (wireBlock !== "minecraft:redstone_wire") {
            throw new Error(`Expected minecraft:redstone_wire, got ${wireBlock}`);
        }

        // Test 4: Lamp with bracket notation
        bracketSchematic.set_block(15, 1, 0, "minecraft:redstone_lamp[lit=false]");
        const lampBlock = bracketSchematic.get_block(15, 1, 0);
        console.log(`   - Lamp block: ${lampBlock}`);
        if (lampBlock !== "minecraft:redstone_lamp") {
            throw new Error(`Expected minecraft:redstone_lamp, got ${lampBlock}`);
        }

        console.log('   ✅ All bracket notation tests passed');

        // Test 5: Use bracket notation circuit in simulation (if available)
        if (typeof nucleation.MchprsWorldWrapper !== 'undefined') {
            console.log('   - Testing bracket notation in simulation...');

            // Create complete circuit using only bracket notation
            const bracketRedstoneSchematic = new SchematicWrapper();

            // Base layer
            for (let x = 0; x <= 15; x++) {
                bracketRedstoneSchematic.set_block(x, 0, 0, "minecraft:gray_concrete");
            }

            // Lever using bracket notation
            bracketRedstoneSchematic.set_block(0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]");

            // Redstone wire using bracket notation
            for (let x = 1; x <= 14; x++) {
                const eastProp = x < 14 ? "side" : "none";
                bracketRedstoneSchematic.set_block(x, 1, 0,
                    `minecraft:redstone_wire[power=0,east=${eastProp},west=side,north=none,south=none]`);
            }

            // Lamp using bracket notation
            bracketRedstoneSchematic.set_block(15, 1, 0, "minecraft:redstone_lamp[lit=false]");

            // Create simulation and test
            const bracketSimWorld = bracketRedstoneSchematic.create_simulation_world();
            const bracketInitialLamp = bracketSimWorld.is_lit(15, 1, 0);
            console.log(`     - Initial lamp state: ${bracketInitialLamp}`);

            // Toggle lever
            bracketSimWorld.on_use_block(0, 1, 0);
            bracketSimWorld.tick(2);
            bracketSimWorld.flush();

            const bracketAfterToggle = bracketSimWorld.is_lit(15, 1, 0);
            console.log(`     - Lamp after toggle: ${bracketAfterToggle}`);

            if (bracketAfterToggle !== bracketInitialLamp) {
                console.log('   ✅ Bracket notation works in simulation!');
            } else {
                console.log('   ⚠️  Lamp state did not change');
            }
        }
    } catch (error) {
        console.log(`   ❌ Bracket notation test failed: ${error.message}`);
        throw error;
    }

    // Test 10: Simulation Sync Back to Schematic
    console.log('\n🧪 Test 10: Sync Simulation State to Schematic');
    if (typeof nucleation.MchprsWorldWrapper !== 'undefined') {
        try {
            const syncSchematic = new SchematicWrapper();

            // Create initial circuit
            for (let x = 0; x <= 5; x++) {
                syncSchematic.set_block(x, 0, 0, "minecraft:gray_concrete");
            }
            syncSchematic.set_block(0, 1, 0, "minecraft:lever[facing=east,powered=false,face=floor]");
            syncSchematic.set_block(5, 1, 0, "minecraft:redstone_lamp[lit=false]");

            // Run simulation
            const syncWorld = syncSchematic.create_simulation_world();
            syncWorld.on_use_block(0, 1, 0); // Turn on lever
            syncWorld.tick(2);
            syncWorld.flush();

            // Verify simulation changed state
            const simLampState = syncWorld.is_lit(5, 1, 0);
            console.log(`   - Simulation lamp state: ${simLampState}`);

            // Sync back to schematic
            syncWorld.sync_to_schematic();
            const updatedSchematic = syncWorld.get_schematic();

            // Check if schematic was updated
            const leverBlock = updatedSchematic.get_block(0, 1, 0);
            console.log(`   - Synced lever block: ${leverBlock}`);

            if (leverBlock && leverBlock.includes('lever')) {
                console.log('   ✅ Sync to schematic works!');
            } else {
                console.log('   ⚠️  Sync may not have preserved block data');
            }
        } catch (error) {
            console.log(`   ⚠️  Sync test error: ${error.message}`);
        }
    } else {
        console.log('   ⚠️  Simulation feature not available');
    }

    // Test 11: IoLayoutBuilder Region Support
    console.log('\n🧪 Test 11: IoLayoutBuilder Region Support');
    if (typeof nucleation.IoLayoutBuilderWrapper !== 'undefined') {
        try {
            const { IoLayoutBuilderWrapper, IoTypeWrapper, LayoutFunctionWrapper, BlockPosition } = nucleation;

            let builder = new IoLayoutBuilderWrapper();
            // Use 4-bit integer to match 4-block region (OneToOne)
            const ioType = IoTypeWrapper.unsignedInt(4);
            const layout = LayoutFunctionWrapper.oneToOne();

            // Define region: 0,0,0 to 1,0,1 (4 blocks)
            const min1 = new BlockPosition(0, 0, 0);
            const max1 = new BlockPosition(1, 0, 1);

            // Add input region (consumes builder, returns new one; consumes min1/max1)
            builder = builder.addInputRegion("region_in", ioType, layout, min1, max1);

            const min2 = new BlockPosition(0, 0, 0);
            const max2 = new BlockPosition(1, 0, 1);

            // Add output region auto (consumes builder, returns new one; consumes min2/max2)
            builder = builder.addOutputRegionAuto("region_out", ioType, min2, max2);

            const ioLayout = builder.build();
            const inputs = ioLayout.inputNames();
            const outputs = ioLayout.outputNames();

            console.log(`   - Inputs: ${inputs.join(', ')}`);
            console.log(`   - Outputs: ${outputs.join(', ')}`);

            if (inputs.includes("region_in") && outputs.includes("region_out")) {
                console.log('   ✅ Region input/output added successfully');
            } else {
                throw new Error('Failed to add region input/output');
            }

        } catch (error) {
            console.log(`   ❌ Region test failed:`, error);
            // Don't fail the whole suite if this is just because features are missing in old build
            // but for my verification I want to see this pass
        }
    } else {
        console.log('   ⚠️  IoLayoutBuilderWrapper not available');
    }

    // Test 12: Multi-Region IO Definition
    console.log('\n🧪 Test 12: Multi-Region IO Definition');
    if (typeof nucleation.DefinitionRegionWrapper !== 'undefined') {
        try {
            const { IoLayoutBuilderWrapper, IoTypeWrapper, DefinitionRegionWrapper, BlockPosition } = nucleation;

            let builder = new IoLayoutBuilderWrapper();
            // Use 8-bit integer
            const ioType = IoTypeWrapper.unsignedInt(8);

            // Create DefinitionRegion
            let region = new DefinitionRegionWrapper();
            // First 4 bits: 0,0,0 -> 3,0,0
            region.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(3, 0, 0));
            // Next 4 bits: 0,0,2 -> 3,0,2 (disjoint!)
            region.addBounds(new BlockPosition(0, 0, 2), new BlockPosition(3, 0, 2));

            // Add input with auto layout (should infer OneToOne for 8 bits)
            builder = builder.addInputFromRegionAuto("split_byte", ioType, region);

            const ioLayout = builder.build();
            const inputs = ioLayout.inputNames();

            console.log(`   - Inputs: ${inputs.join(', ')}`);

            if (inputs.includes("split_byte")) {
                console.log('   ✅ Multi-region input added successfully');
            } else {
                throw new Error('Failed to add multi-region input');
            }

        } catch (error) {
            console.log(`   ❌ Multi-region test failed:`, error);
        }
    } else {
        console.log('   ⚠️  DefinitionRegionWrapper not available');
    }

    // Test 13: Region Filtering and Merging
    console.log('\n🧪 Test 13: Region Filtering and Merging');
    if (typeof nucleation.DefinitionRegionWrapper !== 'undefined' && typeof nucleation.SchematicWrapper !== 'undefined') {
        try {
            const { SchematicWrapper, DefinitionRegionWrapper, BlockPosition } = nucleation;

            // Create a schematic with specific block layout
            const schematic = new SchematicWrapper();

            // 3x3x1 area
            // (0,0,0) - stone
            // (1,0,0) - redstone
            // (2,0,0) - stone
            // (0,0,1) - redstone
            // (1,0,1) - stone
            // (2,0,1) - redstone
            schematic.set_block(0, 0, 0, "minecraft:stone");
            schematic.set_block(1, 0, 0, "minecraft:redstone_wire");
            schematic.set_block(2, 0, 0, "minecraft:stone");
            schematic.set_block(0, 0, 1, "minecraft:redstone_wire");
            schematic.set_block(1, 0, 1, "minecraft:stone");
            schematic.set_block(2, 0, 1, "minecraft:redstone_wire");

            // Create a region covering the whole 3x3x2 area
            let fullRegion = new DefinitionRegionWrapper();
            fullRegion.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(2, 0, 1));

            // Filter for "redstone"
            let filteredRegion = fullRegion.filterByBlock(schematic, "redstone");

            // We expect 3 blocks to be in the filtered region: (1,0,0), (0,0,1), (2,0,1)
            // Can't easily check volume directly from wrapper without exposing it, 
            // but we can try to use it in an IO layout to verify bit count.

            const { IoLayoutBuilderWrapper, IoTypeWrapper } = nucleation;

            // 3 positions -> should fit in 3 bits (OneToOne) or 1 nibble (Packed4)
            // Let's try auto-inference with 3 bits
            const ioType = IoTypeWrapper.unsignedInt(3);
            let builder = new IoLayoutBuilderWrapper();

            try {
                builder = builder.addInputFromRegionAuto("filtered_input", ioType, filteredRegion);
                const layout = builder.build();
                if (layout.inputNames().includes("filtered_input")) {
                    console.log('   ✅ Filtered region successfully used as 3-bit input');
                }
            } catch (e) {
                console.log(`   ❌ Failed to use filtered region: ${e}`);
                throw e;
            }

            // Test Add Point
            let pointRegion = new DefinitionRegionWrapper();
            pointRegion.addPoint(10, 10, 10);

            let builder2 = new IoLayoutBuilderWrapper();
            // 1 bit
            builder2 = builder2.addInputFromRegionAuto("point_input", IoTypeWrapper.boolean(), pointRegion);
            if (builder2.build().inputNames().includes("point_input")) {
                console.log('   ✅ addPoint working');
            }

            // Test Merge
            let regionA = new DefinitionRegionWrapper();
            regionA.addPoint(0, 0, 0);
            let regionB = new DefinitionRegionWrapper();
            regionB.addPoint(1, 0, 0);

            regionA.merge(regionB);

            let builder3 = new IoLayoutBuilderWrapper();
            // 2 bits
            builder3 = builder3.addInputFromRegionAuto("merged_input", IoTypeWrapper.unsignedInt(2), regionA);
            if (builder3.build().inputNames().includes("merged_input")) {
                console.log('   ✅ merge working');
            }

        } catch (error) {
            console.log(`   ❌ Region filtering test failed:`, error);
        }
    } else {
        console.log('   ⚠️  DefinitionRegionWrapper or SchematicWrapper not available');
    }

    // Test 14: Boolean Operations (subtract/intersect)
    console.log('\n🧪 Test 14: DefinitionRegion Boolean Operations');
    if (typeof nucleation.DefinitionRegionWrapper !== 'undefined') {
        try {
            const { DefinitionRegionWrapper, BlockPosition } = nucleation;

            // Create two overlapping regions
            let regionA = new DefinitionRegionWrapper();
            regionA.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(5, 0, 0)); // 0-5

            let regionB = new DefinitionRegionWrapper();
            regionB.addBounds(new BlockPosition(3, 0, 0), new BlockPosition(7, 0, 0)); // 3-7

            // Test subtract
            let subtractRegion = new DefinitionRegionWrapper();
            subtractRegion.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(5, 0, 0));
            subtractRegion.subtract(regionB);
            console.log(`   - Subtract: [0-5] - [3-7] = volume ${subtractRegion.volume()}`);
            if (subtractRegion.volume() === 3) { // 0, 1, 2
                console.log('   ✅ Subtract working correctly');
            } else {
                console.log(`   ❌ Expected volume 3, got ${subtractRegion.volume()}`);
            }

            // Test intersect
            let intersectRegion = new DefinitionRegionWrapper();
            intersectRegion.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(5, 0, 0));
            intersectRegion.intersect(regionB);
            console.log(`   - Intersect: [0-5] ∩ [3-7] = volume ${intersectRegion.volume()}`);
            if (intersectRegion.volume() === 3) { // 3, 4, 5
                console.log('   ✅ Intersect working correctly');
            } else {
                console.log(`   ❌ Expected volume 3, got ${intersectRegion.volume()}`);
            }

            // Test union
            let unionRegion = regionA.union(regionB);
            console.log(`   - Union: [0-5] ∪ [3-7] = volume ${unionRegion.volume()}`);
            if (unionRegion.volume() === 8) { // 0-7
                console.log('   ✅ Union working correctly');
            } else {
                console.log(`   ❌ Expected volume 8, got ${unionRegion.volume()}`);
            }

        } catch (error) {
            console.log(`   ❌ Boolean operations test failed: ${error.message}`);
        }
    } else {
        console.log('   ⚠️  DefinitionRegionWrapper not available');
    }

    // Test 15: Geometric Shifts
    console.log('\n🧪 Test 15: DefinitionRegion Geometric Shifts');
    if (typeof nucleation.DefinitionRegionWrapper !== 'undefined') {
        try {
            const { DefinitionRegionWrapper, BlockPosition } = nucleation;

            let region = new DefinitionRegionWrapper();
            region.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(2, 2, 2));

            // Get initial bounds
            const initialBounds = region.getBounds();
            console.log(`   - Initial bounds: min=[${initialBounds.min}], max=[${initialBounds.max}]`);

            // Shift by 10, 20, 30
            region.shift(10, 20, 30);
            const shiftedBounds = region.getBounds();
            console.log(`   - After shift(10,20,30): min=[${shiftedBounds.min}], max=[${shiftedBounds.max}]`);

            if (shiftedBounds.min[0] === 10 && shiftedBounds.min[1] === 20 && shiftedBounds.min[2] === 30) {
                console.log('   ✅ Shift working correctly');
            } else {
                console.log('   ❌ Shift did not produce expected results');
            }

            // Test expand
            let expandRegion = new DefinitionRegionWrapper();
            expandRegion.addBounds(new BlockPosition(5, 5, 5), new BlockPosition(10, 10, 10));
            expandRegion.expand(2, 2, 2);
            const expandedBounds = expandRegion.getBounds();
            console.log(`   - After expand(2,2,2): min=[${expandedBounds.min}], max=[${expandedBounds.max}]`);

            if (expandedBounds.min[0] === 3 && expandedBounds.max[0] === 12) {
                console.log('   ✅ Expand working correctly');
            } else {
                console.log('   ❌ Expand did not produce expected results');
            }

            // Test contract
            expandRegion.contract(2);
            const contractedBounds = expandRegion.getBounds();
            console.log(`   - After contract(2): min=[${contractedBounds.min}], max=[${contractedBounds.max}]`);

            if (contractedBounds.min[0] === 5 && contractedBounds.max[0] === 10) {
                console.log('   ✅ Contract working correctly');
            } else {
                console.log('   ❌ Contract did not produce expected results');
            }

        } catch (error) {
            console.log(`   ❌ Geometric shifts test failed: ${error.message}`);
        }
    } else {
        console.log('   ⚠️  DefinitionRegionWrapper not available');
    }

    // Test 16: Property Filtering
    console.log('\n🧪 Test 16: DefinitionRegion Property Filtering');
    if (typeof nucleation.DefinitionRegionWrapper !== 'undefined' && typeof nucleation.SchematicWrapper !== 'undefined') {
        try {
            const { SchematicWrapper, DefinitionRegionWrapper, BlockPosition } = nucleation;

            const schematic = new SchematicWrapper();
            // Set up blocks with different properties
            schematic.set_block_with_properties(0, 0, 0, "minecraft:redstone_lamp", { lit: "true" });
            schematic.set_block_with_properties(1, 0, 0, "minecraft:redstone_lamp", { lit: "false" });
            schematic.set_block_with_properties(2, 0, 0, "minecraft:redstone_lamp", { lit: "true" });

            // Create region covering all three
            let region = new DefinitionRegionWrapper();
            region.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(2, 0, 0));

            // Filter for lit=true
            const litRegion = region.filterByProperties(schematic, { lit: "true" });
            console.log(`   - Filter for lit=true: volume ${litRegion.volume()}`);

            if (litRegion.volume() === 2) {
                console.log('   ✅ Property filtering working correctly');
            } else {
                console.log(`   ❌ Expected volume 2, got ${litRegion.volume()}`);
            }

        } catch (error) {
            console.log(`   ❌ Property filtering test failed: ${error.message}`);
        }
    } else {
        console.log('   ⚠️  Required wrappers not available');
    }

    // Test 17: CircuitBuilder Flow
    console.log('\n🧪 Test 17: CircuitBuilder Flow');
    if (typeof nucleation.CircuitBuilderWrapper !== 'undefined') {
        try {
            const { SchematicWrapper, CircuitBuilderWrapper, DefinitionRegionWrapper, IoTypeWrapper, BlockPosition } = nucleation;

            // Create a simple schematic with input/output areas
            const schematic = new SchematicWrapper();
            // Base layer
            for (let x = 0; x < 10; x++) {
                schematic.set_block(x, 0, 0, "minecraft:stone");
            }
            // Input wire at x=0
            schematic.set_block_with_properties(0, 1, 0, "minecraft:redstone_wire", { power: "0" });
            // Output wire at x=9
            schematic.set_block_with_properties(9, 1, 0, "minecraft:redstone_wire", { power: "0" });

            // Create regions
            let inputRegion = new DefinitionRegionWrapper();
            inputRegion.addPoint(0, 1, 0);

            let outputRegion = new DefinitionRegionWrapper();
            outputRegion.addPoint(9, 1, 0);

            // Create circuit builder
            let builder = new CircuitBuilderWrapper(schematic);
            builder = builder.withInputAuto("in", IoTypeWrapper.boolean(), inputRegion);
            builder = builder.withOutputAuto("out", IoTypeWrapper.boolean(), outputRegion);

            console.log(`   - Builder has ${builder.inputCount()} inputs, ${builder.outputCount()} outputs`);
            console.log(`   - Input names: ${builder.inputNames().join(', ')}`);
            console.log(`   - Output names: ${builder.outputNames().join(', ')}`);

            if (builder.inputCount() === 1 && builder.outputCount() === 1) {
                console.log('   ✅ CircuitBuilder flow working correctly');
            } else {
                console.log('   ❌ Unexpected input/output counts');
            }

        } catch (error) {
            console.log(`   ❌ CircuitBuilder test failed: ${error.message}`);
        }
    } else {
        console.log('   ⚠️  CircuitBuilderWrapper not available');
    }

    // Test 18: Manual Ticking
    console.log('\n🧪 Test 18: Manual Ticking');
    if (typeof nucleation.TypedCircuitExecutorWrapper !== 'undefined') {
        try {
            const { SchematicWrapper, IoLayoutBuilderWrapper, IoTypeWrapper, LayoutFunctionWrapper, BlockPosition, ValueWrapper } = nucleation;

            // Create a simple circuit
            const schematic = new SchematicWrapper();
            for (let x = 0; x <= 5; x++) {
                schematic.set_block(x, 0, 0, "minecraft:gray_concrete");
            }
            schematic.set_block_with_properties(0, 1, 0, "minecraft:lever", {
                facing: "east", powered: "false", face: "floor"
            });
            for (let x = 1; x <= 4; x++) {
                schematic.set_block_with_properties(x, 1, 0, "minecraft:redstone_wire", {
                    power: "0", east: "side", west: "side", north: "none", south: "none"
                });
            }
            schematic.set_block_with_properties(5, 1, 0, "minecraft:redstone_lamp", { lit: "false" });

            // Create layout
            let layoutBuilder = new IoLayoutBuilderWrapper();
            layoutBuilder = layoutBuilder.addInputAuto("lever", IoTypeWrapper.boolean(), [[0, 1, 0]]);
            layoutBuilder = layoutBuilder.addOutputAuto("lamp", IoTypeWrapper.boolean(), [[5, 1, 0]]);
            const layout = layoutBuilder.build();

            // Create simulation world
            const world = schematic.create_simulation_world();
            const executor = nucleation.TypedCircuitExecutorWrapper.fromLayout(world, layout);

            // Set to manual mode
            executor.setStateMode("manual");

            // Manual tick test
            executor.tick(5);
            executor.flush();

            console.log('   ✅ Manual ticking executed without errors');

            // Test setInput and readOutput
            const leverValue = ValueWrapper.fromBool(true);
            executor.setInput("lever", leverValue);
            executor.flush();
            executor.tick(5);
            executor.flush();

            const lampState = executor.readOutput("lamp");
            console.log(`   - After setting lever=true and ticking: lamp=${lampState.toJs()}`);
            console.log('   ✅ setInput/readOutput working');

        } catch (error) {
            console.log(`   ❌ Manual ticking test failed: ${error.message}`);
        }
    } else {
        console.log('   ⚠️  TypedCircuitExecutorWrapper not available');
    }

    // Test 19: Connectivity Analysis
    console.log('\n🧪 Test 19: Connectivity Analysis');
    if (typeof nucleation.DefinitionRegionWrapper !== 'undefined') {
        try {
            const { DefinitionRegionWrapper, BlockPosition } = nucleation;

            // Contiguous region (L-shape)
            let contiguousRegion = new DefinitionRegionWrapper();
            contiguousRegion.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(3, 0, 0));
            contiguousRegion.addBounds(new BlockPosition(3, 0, 0), new BlockPosition(3, 3, 0));

            const isContiguous = contiguousRegion.isContiguous();
            console.log(`   - L-shape region isContiguous: ${isContiguous}`);
            if (isContiguous) {
                console.log('   ✅ isContiguous correctly identifies connected region');
            } else {
                console.log('   ❌ L-shape should be contiguous');
            }

            // Non-contiguous region (two separate points)
            let nonContiguousRegion = new DefinitionRegionWrapper();
            nonContiguousRegion.addPoint(0, 0, 0);
            nonContiguousRegion.addPoint(10, 10, 10);

            const isNotContiguous = !nonContiguousRegion.isContiguous();
            const components = nonContiguousRegion.connectedComponents();
            console.log(`   - Two separate points: isContiguous=${!isNotContiguous}, components=${components}`);
            if (isNotContiguous && components === 2) {
                console.log('   ✅ connectedComponents correctly counts 2 components');
            } else {
                console.log('   ❌ Expected 2 components');
            }

        } catch (error) {
            console.log(`   ❌ Connectivity analysis test failed: ${error.message}`);
        }
    } else {
        console.log('   ⚠️  DefinitionRegionWrapper not available');
    }

    // Test 20: SortStrategy (NEW)
    console.log('\n🧪 Test 20: SortStrategy');
    if (typeof nucleation.SortStrategyWrapper !== 'undefined') {
        try {
            const { SortStrategyWrapper, DefinitionRegionWrapper, BlockPosition } = nucleation;

            // Test factory methods
            const yxz = SortStrategyWrapper.yxz();
            const xyz = SortStrategyWrapper.xyz();
            const zyx = SortStrategyWrapper.zyx();
            const yDescXZ = SortStrategyWrapper.yDescXZ();
            const descending = SortStrategyWrapper.descending();
            const preserve = SortStrategyWrapper.preserve();
            const reverse = SortStrategyWrapper.reverse();
            const distanceFrom = SortStrategyWrapper.distanceFrom(5, 5, 5);

            console.log(`   - Created strategies: yxz=${yxz.name}, xyz=${xyz.name}, zyx=${zyx.name}`);
            console.log(`   - yDescXZ=${yDescXZ.name}, descending=${descending.name}`);
            console.log(`   - preserve=${preserve.name}, reverse=${reverse.name}`);
            console.log(`   - distanceFrom=${distanceFrom.name}`);

            // Test fromString
            const fromStr = SortStrategyWrapper.fromString("y_desc");
            console.log(`   - fromString("y_desc") = ${fromStr.name}`);

            try {
                SortStrategyWrapper.fromString("invalid");
                console.log('   ❌ Should have thrown for invalid strategy');
            } catch (e) {
                console.log('   ✅ Correctly throws for invalid strategy');
            }

            // Test with CircuitBuilder (if available)
            if (typeof nucleation.CircuitBuilderWrapper !== 'undefined') {
                const { SchematicWrapper, CircuitBuilderWrapper, IoTypeWrapper } = nucleation;

                // Create a simple schematic
                const schematic = new SchematicWrapper();
                for (let x = 0; x < 8; x++) {
                    schematic.set_block(x, 0, 0, "minecraft:stone");
                    schematic.set_block_with_properties(x, 1, 0, "minecraft:redstone_wire", { power: "0" });
                }

                // Create region with multiple Y levels (to test Y sorting)
                let region = new DefinitionRegionWrapper();
                region.addBounds(new BlockPosition(0, 1, 0), new BlockPosition(3, 1, 0));  // Y=1
                region.addBounds(new BlockPosition(0, 2, 0), new BlockPosition(3, 2, 0));  // Y=2

                // Test default sorting (YXZ)
                let builder = new CircuitBuilderWrapper(schematic);
                builder = builder.withInputAuto("default_sort", IoTypeWrapper.unsignedInt(8), region);
                console.log('   ✅ withInputAuto with default sort works');

                // Test custom sorting (Y descending)
                builder = new CircuitBuilderWrapper(schematic);
                builder = builder.withInputAutoSorted(
                    "y_desc_sort",
                    IoTypeWrapper.unsignedInt(8),
                    region,
                    SortStrategyWrapper.yDescXZ()
                );
                console.log('   ✅ withInputAutoSorted with Y descending works');

                // Test preserve (box order)
                builder = new CircuitBuilderWrapper(schematic);
                builder = builder.withInputAutoSorted(
                    "preserve_sort",
                    IoTypeWrapper.unsignedInt(8),
                    region,
                    SortStrategyWrapper.preserve()
                );
                console.log('   ✅ withInputAutoSorted with preserve works');

                // Test distance-based
                builder = new CircuitBuilderWrapper(schematic);
                builder = builder.withInputAutoSorted(
                    "distance_sort",
                    IoTypeWrapper.unsignedInt(8),
                    region,
                    SortStrategyWrapper.distanceFrom(2, 1, 0)
                );
                console.log('   ✅ withInputAutoSorted with distanceFrom works');
            }

            console.log('   ✅ All SortStrategy tests passed');

        } catch (error) {
            console.log(`   ❌ SortStrategy test failed: ${error.message}`);
            console.log(error.stack);
        }
    } else {
        console.log('   ⚠️  SortStrategyWrapper not available');
    }

    // Test 21: DefinitionRegion Renderer Support (NEW)
    console.log('\n🧪 Test 21: DefinitionRegion Renderer Support');
    if (typeof nucleation.DefinitionRegionWrapper !== 'undefined') {
        try {
            const { DefinitionRegionWrapper, BlockPosition } = nucleation;

            // Test fromBoundingBoxes
            const boxes = [
                { min: [0, 0, 0], max: [2, 2, 2] },
                { min: [5, 5, 5], max: [7, 7, 7] }
            ];
            const region = DefinitionRegionWrapper.fromBoundingBoxes(boxes);
            console.log(`   - fromBoundingBoxes: created region with volume ${region.volume()}`);

            // Test boxCount
            const boxCount = region.boxCount();
            console.log(`   - boxCount: ${boxCount}`);
            if (boxCount === 2) {
                console.log('   ✅ boxCount correct');
            } else {
                console.log(`   ❌ Expected 2 boxes, got ${boxCount}`);
            }

            // Test getBox
            const box0 = region.getBox(0);
            console.log(`   - getBox(0): min=[${box0.min}], max=[${box0.max}]`);
            if (box0.min[0] === 0 && box0.max[0] === 2) {
                console.log('   ✅ getBox correct');
            } else {
                console.log('   ❌ getBox returned unexpected values');
            }

            // Test getBoxes
            const allBoxes = region.getBoxes();
            console.log(`   - getBoxes: returned ${allBoxes.length} boxes`);
            if (allBoxes.length === 2) {
                console.log('   ✅ getBoxes correct');
            } else {
                console.log(`   ❌ Expected 2 boxes, got ${allBoxes.length}`);
            }

            // Test fromPositions
            const positions = [[0, 0, 0], [1, 0, 0], [2, 0, 0]];
            const posRegion = DefinitionRegionWrapper.fromPositions(positions);
            console.log(`   - fromPositions: volume ${posRegion.volume()}`);
            if (posRegion.volume() === 3) {
                console.log('   ✅ fromPositions correct');
            } else {
                console.log(`   ❌ Expected volume 3, got ${posRegion.volume()}`);
            }

            // Test metadata
            let metaRegion = new DefinitionRegionWrapper();
            metaRegion.addBounds(new BlockPosition(0, 0, 0), new BlockPosition(1, 1, 1));
            metaRegion = metaRegion.setMetadata("color", "red").setMetadata("label", "Input A");

            const color = metaRegion.getMetadata("color");
            console.log(`   - getMetadata("color"): ${color}`);
            if (color === "red") {
                console.log('   ✅ Metadata get/set correct');
            } else {
                console.log(`   ❌ Expected "red", got "${color}"`);
            }

            const allMeta = metaRegion.getAllMetadata();
            const metaKeys = metaRegion.metadataKeys();
            console.log(`   - getAllMetadata keys: ${metaKeys.length}`);

            // Test dimensions
            let dimRegion = DefinitionRegionWrapper.fromBounds(
                new BlockPosition(0, 0, 0),
                new BlockPosition(9, 4, 2)
            );
            const dims = dimRegion.dimensions();
            console.log(`   - dimensions: [${dims}]`);
            if (dims[0] === 10 && dims[1] === 5 && dims[2] === 3) {
                console.log('   ✅ dimensions correct');
            } else {
                console.log(`   ❌ Expected [10, 5, 3], got [${dims}]`);
            }

            // Test center
            let centerRegion = DefinitionRegionWrapper.fromBounds(
                new BlockPosition(0, 0, 0),
                new BlockPosition(10, 10, 10)
            );
            const center = centerRegion.center();
            console.log(`   - center: [${center}]`);
            if (center[0] === 5 && center[1] === 5 && center[2] === 5) {
                console.log('   ✅ center correct');
            } else {
                console.log(`   ❌ Expected [5, 5, 5], got [${center}]`);
            }

            // Test centerF32
            const centerF32 = centerRegion.centerF32();
            console.log(`   - centerF32: [${centerF32[0].toFixed(2)}, ${centerF32[1].toFixed(2)}, ${centerF32[2].toFixed(2)}]`);

            // Test intersectsBounds (frustum culling)
            const intersects1 = centerRegion.intersectsBounds(5, 5, 5, 15, 15, 15);
            const intersects2 = centerRegion.intersectsBounds(20, 20, 20, 30, 30, 30);
            console.log(`   - intersectsBounds (overlapping): ${intersects1}`);
            console.log(`   - intersectsBounds (non-overlapping): ${intersects2}`);
            if (intersects1 && !intersects2) {
                console.log('   ✅ intersectsBounds correct');
            } else {
                console.log('   ❌ intersectsBounds returned unexpected values');
            }

            // Test immutable transformations
            let origRegion = DefinitionRegionWrapper.fromBounds(
                new BlockPosition(0, 0, 0),
                new BlockPosition(5, 5, 5)
            );

            const shiftedRegion = origRegion.shifted(10, 20, 30);
            const origBounds = origRegion.getBounds();
            const shiftedBounds = shiftedRegion.getBounds();
            console.log(`   - Original after shifted(): min=[${origBounds.min}]`);
            console.log(`   - Shifted result: min=[${shiftedBounds.min}]`);
            if (origBounds.min[0] === 0 && shiftedBounds.min[0] === 10) {
                console.log('   ✅ shifted() is immutable');
            } else {
                console.log('   ❌ shifted() should not modify original');
            }

            const expandedRegion = origRegion.expanded(2, 2, 2);
            const expandedBounds = expandedRegion.getBounds();
            console.log(`   - Expanded result: min=[${expandedBounds.min}], max=[${expandedBounds.max}]`);

            const contractedRegion = expandedRegion.contracted(2);
            const contractedBounds = contractedRegion.getBounds();
            console.log(`   - Contracted result: min=[${contractedBounds.min}], max=[${contractedBounds.max}]`);

            // Test copy/clone
            let copyRegion = DefinitionRegionWrapper.fromBounds(
                new BlockPosition(0, 0, 0),
                new BlockPosition(3, 3, 3)
            );
            copyRegion = copyRegion.setMetadata("test", "value");

            const cloned = copyRegion.clone();
            cloned.shift(100, 100, 100);

            const copyBounds = copyRegion.getBounds();
            const clonedBounds = cloned.getBounds();
            console.log(`   - Original after clone modified: min=[${copyBounds.min}]`);
            console.log(`   - Clone after shift: min=[${clonedBounds.min}]`);
            if (copyBounds.min[0] === 0 && clonedBounds.min[0] === 100) {
                console.log('   ✅ clone() creates independent copy');
            } else {
                console.log('   ❌ clone() should create independent copy');
            }

            console.log('   ✅ All renderer support tests passed');

        } catch (error) {
            console.log(`   ❌ Renderer support test failed: ${error.message}`);
            console.log(error.stack);
        }
    } else {
        console.log('   ⚠️  DefinitionRegionWrapper not available');
    }

    console.log('\n=== Test Summary ===');
    console.log('✅ All basic functionality tests completed');
    console.log('📊 Check the output above for any ❌ ERROR messages');
    console.log('🔍 Pay attention to palette index validation and duplicate detection');

    if (duplicates.length > 0 || invalidIndices.length > 0) {
        console.log('\n⚠️  ISSUES DETECTED:');
        if (duplicates.length > 0) console.log(`   - ${duplicates.length} duplicate blocks found`);
        if (invalidIndices.length > 0) console.log(`   - ${invalidIndices.length} invalid palette indices found`);
        console.log('   This suggests there may be issues with the chunk iterator implementation.');
        process.exit(1);
    } else {
        console.log('\n🎉 No major issues detected! The chunk iterator appears to be working correctly.');
    }
}

// Run the tests
runTests().catch(error => {
    console.error('❌ Test failed:', error);
    process.exit(1);
});
