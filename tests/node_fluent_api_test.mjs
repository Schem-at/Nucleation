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
    
    // We can't easily check emptiness from JS without exposing more methods, 
    // but we can check if it affects circuit building if we used it.
    // For now, just ensure it doesn't crash.

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
