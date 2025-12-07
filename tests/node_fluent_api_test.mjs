import init, { SchematicWrapper } from '../pkg/nucleation.js';
import assert from 'assert';

async function testFluentApi() {
    console.log("Testing Fluent API...");
    await init();
    const s = new SchematicWrapper("FluentCircuit");

    // Setup blocks
    s.set_block(0, 1, 0, "minecraft:lever[facing=east,face=floor,powered=false]");
    s.set_block(1, 1, 0, "minecraft:redstone_wire[power=0,east=side,west=side]");
    s.set_block(2, 1, 0, "minecraft:redstone_lamp[lit=false]");
    for(let i=0; i<3; i++) s.set_block(i, 0, 0, "minecraft:gray_concrete");

    // Create regions with chaining
    // Now supports automatic syncing via internal pointer
    s.createRegion("a", {x: 0, y: 1, z: 0}, {x: 0, y: 1, z: 0})
        .addFilter("lever")
        .setColor(0x00ff00);

    s.createRegion("c", {x: 2, y: 1, z: 0}, {x: 2, y: 1, z: 0})
        .addFilter("minecraft:redstone_lamp");

    // Test excludeBlock (Negative Filter)
    // Create a region covering (0,0,0) to (2,0,0) which are gray_concrete
    // And exclude gray_concrete -> should be empty
    const r3 = s.createRegion("b", {x: 0, y: 0, z: 0}, {x: 2, y: 0, z: 0})
        .excludeBlock("gray_concrete");
    
    // Verify excludeBlock worked: Region 'b' should be empty.
    // Attempting to use it for a 1-bit input should fail.
    console.log("Verifying excludeBlock...");
    let filterWorked = false;
    try {
        s.createCircuit(
            [ { name: "b", bits: 1, region: "b" } ],
            []
        );
    } catch (e) {
        filterWorked = true;
        console.log("Caught expected error (Region empty):", e);
    }
    assert.ok(filterWorked, "excludeBlock failed: Region 'b' should be empty, but createCircuit succeeded.");

    // Verify addFilter (Positive Filter)
    // Create region covering (0,0,0) [concrete] and (0,1,0) [lever]
    // Filter for "lever" -> should have 1 block
    console.log("Verifying addFilter...");
    console.log("Verifying addFilter...");
    console.log("Block at 0,0,0:", s.get_block_string(0, 0, 0));
    console.log("Block at 0,1,0:", s.get_block_string(0, 1, 0));
    const d = s.createRegion("d", {x: 0, y: 0, z: 0}, {x: 0, y: 1, z: 0})
        .addFilter("lever");
    
    const blocks = d.getBlocks();
    console.log("Region 'd' blocks:", blocks);
    
    if (blocks.length !== 1) {
        throw new Error(`Expected 1 block in region 'd', got ${blocks.length}`);
    }
    if (!blocks[0].block.includes("lever")) {
        throw new Error(`Expected block to be a lever, got ${blocks[0].block}`);
    }

    // Should have 1 block.
    // If we try to use it for 5 bits:
    // - Packed4 needs ceil(5/4) = 2 blocks.
    // - OneToOne needs 5 blocks.
    // So with 1 block, it should fail.
    // If addFilter failed (2 blocks), Packed4 would succeed.
    let addFilterWorked = false;
    try {
        s.createCircuit(
            [ { name: "d", bits: 5, region: "d" } ],
            []
        );
    } catch (e) {
        // Expected error: Region 'd' has 1 blocks, but IO type requires 2 (Packed4)
        console.log("Caught expected error (Not enough blocks):", e);
        addFilterWorked = true;
    }
    
    if (!addFilterWorked) {
         console.error("FAIL: addFilter did not remove non-matching blocks from region 'd'");
    }
    assert.ok(addFilterWorked, "addFilter failed: Region 'd' should have 1 block, but seems to have 2");

    // Create circuit
    const circuit = s.createCircuit(
        [ { name: "a", bits: 1, region: "a" } ],
        [ { name: "out", bits: 1, region: "c" } ]
    );

    // Run
    const res1 = circuit.run({ a: 1 }, 5, 'fixed');
    console.log("Run 1 (Input 1):", res1);
    assert.strictEqual(res1.outputs.out, 1);

    const res2 = circuit.run({ a: 0 }, 5, 'fixed');
    console.log("Run 2 (Input 0):", res2);
    assert.strictEqual(res2.outputs.out, 0);

    console.log("Fluent API Test Passed!");
}

testFluentApi().catch(e => {
    console.error(e);
    process.exit(1);
});
