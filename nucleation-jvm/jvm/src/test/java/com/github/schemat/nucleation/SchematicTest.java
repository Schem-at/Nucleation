package com.github.schemat.nucleation;

import org.junit.jupiter.api.Test;

import java.util.List;
import java.util.Map;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.*;

class SchematicTest {

    @Test
    void createEmptyAndClose() {
        try (Schematic s = new Schematic("hello")) {
            assertEquals("hello", s.name());
            assertEquals(0, s.blockCount());
        }
    }

    @Test
    void setAndGetBlockSimple() {
        try (Schematic s = new Schematic("t")) {
            assertTrue(s.setBlock(0, 0, 0, "minecraft:stone"));
            assertEquals(1, s.blockCount());
            Optional<String> name = s.getBlockName(0, 0, 0);
            assertTrue(name.isPresent());
            assertEquals("minecraft:stone", name.get());
        }
    }

    @Test
    void setBlockWithProperties() {
        try (Schematic s = new Schematic()) {
            Map<String, String> props = Map.of("axis", "y");
            s.setBlock(1, 2, 3, "minecraft:oak_log", props);
            try (BlockState b = s.getBlock(1, 2, 3).orElseThrow()) {
                assertEquals("minecraft:oak_log", b.name());
                assertEquals("y", b.properties().get("axis"));
            }
        }
    }

    @Test
    void dimensionsReflectAddedBlocks() {
        try (Schematic s = new Schematic()) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(4, 5, 6, "minecraft:stone");
            Dimensions d = s.dimensions();
            assertTrue(d.width() >= 1);
            assertTrue(d.height() >= 1);
            assertTrue(d.length() >= 1);
        }
    }

    @Test
    void litematicRoundTrip() {
        byte[] bytes;
        try (Schematic s = new Schematic("rt")) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(1, 0, 0, "minecraft:dirt");
            s.setBlock(2, 0, 0, "minecraft:oak_log");
            bytes = s.toLitematic();
            assertNotNull(bytes);
            assertTrue(bytes.length > 0);
        }
        try (Schematic s2 = Schematic.fromLitematic(bytes)) {
            assertEquals(3, s2.blockCount());
        }
    }

    @Test
    void schemRoundTrip() {
        byte[] bytes;
        try (Schematic s = new Schematic()) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            bytes = s.toSchematic();
        }
        try (Schematic s2 = Schematic.fromSchematic(bytes)) {
            assertEquals(1, s2.blockCount());
        }
    }

    @Test
    void snapshotRoundTrip() {
        byte[] bytes;
        try (Schematic s = new Schematic()) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(1, 0, 0, "minecraft:diamond_block");
            bytes = s.toSnapshot();
        }
        try (Schematic s2 = Schematic.fromSnapshot(bytes)) {
            assertEquals(2, s2.blockCount());
        }
    }

    @Test
    void iterateBlocks() {
        try (Schematic s = new Schematic()) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(1, 0, 0, "minecraft:dirt");
            s.setBlock(2, 0, 0, "minecraft:oak_planks");

            int seen = 0;
            for (Block b : s) {
                assertNotNull(b.name());
                assertNotNull(b.properties());
                seen++;
            }
            assertEquals(3, seen);
        }
    }

    @Test
    void streamBlocks() {
        try (Schematic s = new Schematic()) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(1, 0, 0, "minecraft:stone");
            long count = s.stream()
                    .filter(b -> b.name().equals("minecraft:stone"))
                    .count();
            assertEquals(2, count);
        }
    }

    @Test
    void copyIsIndependent() {
        try (Schematic a = new Schematic()) {
            a.setBlock(0, 0, 0, "minecraft:stone");
            try (Schematic b = a.copy()) {
                assertEquals(1, b.blockCount());
                b.setBlock(1, 0, 0, "minecraft:dirt");
                assertEquals(2, b.blockCount());
                // Original unchanged
                assertEquals(1, a.blockCount());
            }
        }
    }

    @Test
    void countBlockTypes() {
        try (Schematic s = new Schematic()) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(1, 0, 0, "minecraft:stone");
            s.setBlock(2, 0, 0, "minecraft:dirt");
            Map<String, Integer> counts = s.countBlockTypes();
            assertEquals(2, counts.get("minecraft:stone"));
            assertEquals(1, counts.get("minecraft:dirt"));
        }
    }

    @Test
    void fillCuboid() {
        try (Schematic s = new Schematic()) {
            s.fillCuboid(0, 0, 0, 2, 2, 2, "minecraft:stone");
            assertEquals(27, s.blockCount());
        }
    }

    @Test
    void fillSphere() {
        try (Schematic s = new Schematic()) {
            s.fillSphere(0, 0, 0, 3.0, "minecraft:stone");
            assertTrue(s.blockCount() > 0);
        }
    }

    @Test
    void supportedFormatsListed() {
        List<String> imp = Schematic.supportedImportFormats();
        List<String> exp = Schematic.supportedExportFormats();
        assertTrue(imp.contains("litematic"));
        assertTrue(exp.contains("litematic"));
    }

    @Test
    void closedSchematicRejectsCalls() {
        Schematic s = new Schematic();
        s.close();
        assertThrows(IllegalStateException.class, s::name);
    }

    @Test
    void invalidBytesThrows() {
        byte[] junk = new byte[] {0x00, 0x01, 0x02, 0x03};
        assertThrows(RuntimeException.class, () -> Schematic.fromLitematic(junk));
    }
}
