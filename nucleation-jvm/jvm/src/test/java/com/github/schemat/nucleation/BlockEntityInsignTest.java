package com.github.schemat.nucleation;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

/**
 * setBlockEntity SNBT write path + insign round-trip: author sign text
 * programmatically, then build a typed executor from the signs alone.
 */
class BlockEntityInsignTest {

    @BeforeAll
    static void requireSimulation() {
        assumeTrue(Nucleation.hasSimulation(), "cdylib built without simulation — skipping");
    }

    private static String jsonEscape(String s) {
        return s.replace("\\", "\\\\").replace("\"", "\\\"");
    }

    /**
     * Sign lines as SNBT: each message is a DOUBLE-quoted SNBT string holding
     * a JSON text component. SNBT single-quoted strings do not support the
     * {@code \"} escape, so double-quoted with escaped quotes is required.
     */
    private static String signSnbt(String... lines) {
        StringBuilder msgs = new StringBuilder();
        for (int i = 0; i < 4; i++) {
            if (i > 0) msgs.append(',');
            String text = i < lines.length ? lines[i] : "";
            String json = "{\"text\":\"" + jsonEscape(text) + "\"}";
            msgs.append('\"').append(jsonEscape(json)).append('\"');
        }
        return "{front_text:{messages:[" + msgs + "]}}";
    }

    @Test
    void setBlockEntityAndFromInsignRoundTrip() {
        try (Schematic s = new Schematic("insign-rt")) {
            // lever-less wire pass-through: in wire -> out wire
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(1, 0, 0, "minecraft:stone");
            s.setBlock(2, 0, 0, "minecraft:stone");
            s.setBlock(0, 1, 0, "minecraft:redstone_wire");
            s.setBlock(1, 1, 0, "minecraft:redstone_wire");
            s.setBlock(2, 1, 0, "minecraft:redstone_wire");

            // signs above, declaring the wires below them as IO
            s.setBlock(0, 3, 0, "minecraft:oak_sign");
            assertTrue(s.setBlockEntity(0, 3, 0, "minecraft:sign", signSnbt(
                    "@io.a=rc([0,-2,0],[0,-2,0])",
                    "#io.a:type=\"input\"",
                    "#io.a:data_type=\"signal_strength\"")));
            s.setBlock(2, 3, 0, "minecraft:oak_sign");
            assertTrue(s.setBlockEntity(2, 3, 0, "minecraft:sign", signSnbt(
                    "@io.out=rc([0,-2,0],[0,-2,0])",
                    "#io.out:type=\"output\"",
                    "#io.out:data_type=\"signal_strength\"")));

            try (CircuitBuilder cb = CircuitBuilder.fromInsign(s);
                 TypedCircuitExecutor exec = cb.build()) {
                assertTrue(exec.inputNames().contains("a"), "insign input parsed");
                assertTrue(exec.outputNames().contains("out"), "insign output parsed");
                try (TypedCircuitExecutor.ExecutionResult r =
                             exec.execute(Map.of("a", Value.ofU32(15)), 4)) {
                    assertTrue(r.output("out").asInt() > 0, "signal should propagate");
                }
            }
        }
    }

    @Test
    void invalidSnbtReturnsFalseViaException() {
        try (Schematic s = new Schematic("bad")) {
            s.setBlock(0, 0, 0, "minecraft:oak_sign");
            assertThrows(Exception.class,
                    () -> s.setBlockEntity(0, 0, 0, "minecraft:sign", "{unclosed"));
        }
    }
}
