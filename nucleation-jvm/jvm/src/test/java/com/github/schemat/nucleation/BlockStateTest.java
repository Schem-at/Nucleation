package com.github.schemat.nucleation;

import org.junit.jupiter.api.Test;

import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;

class BlockStateTest {

    @Test
    void createAndRead() {
        try (BlockState s = new BlockState("minecraft:stone")) {
            assertEquals("minecraft:stone", s.name());
            assertTrue(s.properties().isEmpty());
        }
    }

    @Test
    void withPropertyReturnsNewInstance() {
        try (BlockState a = new BlockState("minecraft:oak_log");
             BlockState b = a.withProperty("axis", "y")) {
            assertTrue(a.properties().isEmpty(), "original should be unchanged");
            Map<String, String> p = b.properties();
            assertEquals("y", p.get("axis"));
        }
    }

    @Test
    void multiplePropertiesAccumulate() {
        try (BlockState root = new BlockState("minecraft:repeater");
             BlockState s1 = root.withProperty("delay", "4");
             BlockState s2 = s1.withProperty("facing", "north")) {
            Map<String, String> p = s2.properties();
            assertEquals("4", p.get("delay"));
            assertEquals("north", p.get("facing"));
        }
    }

    @Test
    void toStringFormatsBracketed() {
        try (BlockState a = new BlockState("minecraft:oak_log");
             BlockState b = a.withProperty("axis", "y")) {
            String s = b.toString();
            assertTrue(s.startsWith("minecraft:oak_log["));
            assertTrue(s.contains("axis=y"));
        }
    }

    @Test
    void closedRejectsAccess() {
        BlockState a = new BlockState("minecraft:stone");
        a.close();
        assertThrows(IllegalStateException.class, a::name);
    }
}
