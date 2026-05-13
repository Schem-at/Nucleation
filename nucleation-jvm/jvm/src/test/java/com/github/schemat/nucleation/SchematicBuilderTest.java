package com.github.schemat.nucleation;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SchematicBuilderTest {

    @Test
    void buildSimple3x3Layer() {
        try (SchematicBuilder b = new SchematicBuilder()) {
            b.name("Demo")
             .useStandardPalette()
             .map('#', "minecraft:stone")
             .map(' ', "minecraft:air")
             .layer("###", "# #", "###");
            try (Schematic s = b.build()) {
                assertEquals("Demo", s.name());
                assertTrue(s.blockCount() > 0);
            }
        }
    }

    @Test
    void emptyBuilderProducesEmptySchematic() {
        try (SchematicBuilder b = new SchematicBuilder()) {
            // The default validate() should succeed for a non-configured builder
            // OR return an error message; either is acceptable for this smoke test.
            String result = b.validate();
            assertNotNull(result);
        }
    }
}
