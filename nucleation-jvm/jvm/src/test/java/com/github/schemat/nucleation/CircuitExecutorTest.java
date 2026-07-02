package com.github.schemat.nucleation;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

class CircuitExecutorTest {

    @BeforeAll
    static void requireSimulation() {
        assumeTrue(Nucleation.hasSimulation(),
                "cdylib built without the simulation feature — skipping");
    }

    /**
     * Torch-based AND gate (ported from the core integration tests).
     * Input A at (0,1,0), input B at (0,1,2), output at (4,1,1).
     */
    private static Schematic andGate() {
        Schematic s = new Schematic("and-gate");
        String[][] blocks = {
                {"0,0,0", "minecraft:gray_concrete"},
                {"1,0,0", "minecraft:gray_concrete"},
                {"0,0,2", "minecraft:gray_concrete"},
                {"1,0,2", "minecraft:gray_concrete"},
                {"2,0,0", "minecraft:gray_concrete"},
                {"2,0,1", "minecraft:gray_concrete"},
                {"2,0,2", "minecraft:gray_concrete"},
                {"2,1,0", "minecraft:gray_concrete"},
                {"2,1,1", "minecraft:gray_concrete"},
                {"2,1,2", "minecraft:gray_concrete"},
                {"3,0,1", "minecraft:gray_concrete"},
                {"4,0,1", "minecraft:gray_concrete"},
                {"0,1,0", "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]"},
                {"1,1,0", "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]"},
                {"0,1,2", "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]"},
                {"1,1,2", "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]"},
                {"2,2,0", "minecraft:redstone_torch[lit=true]"},
                {"2,2,2", "minecraft:redstone_torch[lit=true]"},
                {"2,2,1", "minecraft:redstone_wire[power=15,north=side,south=side,east=none,west=none]"},
                {"3,1,1", "minecraft:redstone_wall_torch[facing=east,lit=false]"},
                {"4,1,1", "minecraft:redstone_wire[power=0,north=none,south=none,east=side,west=side]"},
        };
        for (String[] b : blocks) {
            String[] p = b[0].split(",");
            assertTrue(s.setBlock(Integer.parseInt(p[0]), Integer.parseInt(p[1]), Integer.parseInt(p[2]), b[1]),
                    "failed to set " + b[1]);
        }
        return s;
    }

    private static TypedCircuitExecutor buildAndGateExecutor(Schematic s) {
        try (IoType boolTy = IoType.bool();
             LayoutFunction oneToOne = LayoutFunction.oneToOne();
             DefinitionRegion a = DefinitionRegion.fromPositions(new int[]{0, 1, 0});
             DefinitionRegion b = DefinitionRegion.fromPositions(new int[]{0, 1, 2});
             DefinitionRegion out = DefinitionRegion.fromPositions(new int[]{4, 1, 1});
             CircuitBuilder builder = new CircuitBuilder(s)) {
            return builder
                    .withInput("a", boolTy, oneToOne, a)
                    .withInput("b", boolTy, oneToOne, b)
                    .withOutput("output", boolTy, oneToOne, out)
                    .validate()
                    .build();
        }
    }

    private static boolean runAndGate(TypedCircuitExecutor exec, boolean a, boolean b) {
        try (Value va = Value.ofBool(a); Value vb = Value.ofBool(b);
             TypedCircuitExecutor.ExecutionResult result = exec.execute(Map.of("a", va, "b", vb), 20)) {
            try (Value out = result.output("output")) {
                return out.asBool();
            }
        }
    }

    @Test
    void andGateTruthTable() {
        try (Schematic s = andGate(); TypedCircuitExecutor exec = buildAndGateExecutor(s)) {
            assertEquals(java.util.List.of("a", "b"), exec.inputNames());
            assertEquals(java.util.List.of("output"), exec.outputNames());
            assertFalse(runAndGate(exec, false, false), "false AND false");
            assertFalse(runAndGate(exec, true, false), "true AND false");
            assertFalse(runAndGate(exec, false, true), "false AND true");
            assertTrue(runAndGate(exec, true, true), "true AND true");
        }
    }

    @Test
    void manualModeSetInputReadOutput() {
        try (Schematic s = andGate(); TypedCircuitExecutor exec = buildAndGateExecutor(s)) {
            exec.setStateMode(StateMode.MANUAL);
            try (Value t = Value.ofBool(true)) {
                exec.setInput("a", t);
                exec.setInput("b", t);
            }
            exec.tick(20);
            exec.flush();
            try (Value out = exec.readOutput("output")) {
                assertTrue(out.asBool(), "manual-mode AND(true,true) should be true");
            }
            exec.reset();
        }
    }

    @Test
    void builderReportsCountsAndValidationErrors() {
        try (Schematic s = andGate(); CircuitBuilder builder = new CircuitBuilder(s)) {
            assertEquals(0, builder.inputCount());
            // No inputs/outputs declared: validate must throw
            assertThrows(IllegalStateException.class, builder::validate);
        }
    }

    @Test
    void layoutInfoJsonDescribesIo() {
        try (Schematic s = andGate(); TypedCircuitExecutor exec = buildAndGateExecutor(s)) {
            String json = exec.layoutInfoJson();
            assertTrue(json.contains("\"inputs\""), json);
            assertTrue(json.contains("\"a\""), json);
            assertTrue(json.contains("\"positions\""), json);
        }
    }

    @Test
    void valueRoundTrips() {
        try (Value v = Value.ofU32(42)) {
            assertEquals("U32", v.typeName());
            assertEquals(42, v.asLong());
        }
        try (Value v = Value.ofBool(true)) {
            assertTrue(v.asBool());
        }
        try (Value v = Value.ofString("hi")) {
            assertEquals("hi", v.asString());
        }
        try (Value v = Value.ofBits(new boolean[]{true, false, true})) {
            assertArrayEquals(new boolean[]{true, false, true}, v.asBits());
        }
        try (Value v = Value.ofI64(-7)) {
            assertEquals(-7, v.asLong());
            assertThrows(RuntimeException.class, v::asBool, "wrong-variant access should throw");
        }
    }

    @Test
    void ioTypeBitCounts() {
        try (IoType t = IoType.unsignedInt(8)) {
            assertEquals(8, t.bitCount());
        }
        try (IoType t = IoType.bool()) {
            assertEquals(1, t.bitCount());
        }
        try (IoType t = IoType.ascii(4)) {
            assertEquals(32, t.bitCount());
        }
    }

    @Test
    void definitionRegionVolume() {
        try (DefinitionRegion r = DefinitionRegion.fromBounds(0, 0, 0, 1, 1, 1)) {
            assertEquals(8, r.volume());
            r.addPoint(5, 5, 5);
            assertEquals(9, r.volume());
        }
    }
}
