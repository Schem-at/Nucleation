package com.github.schemat.nucleation;

import org.junit.jupiter.api.Test;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.*;

class SdfTest {

    private static String islandSdf() {
        return Sdf.superPrism(24, 2.5, 24, 6).at(0, 61, 0)
                .smoothUnion(
                        Sdf.ellipsoid(20, 14, 20).at(0, 48, 0).displace(3.0, 0.07, 42),
                        4.0)
                .toJson();
    }

    private static String islandRules() {
        return new SdfRules()
                .fillDepth(0, 0, "minecraft:grass_block")
                .fillDepth(1, 3, "minecraft:dirt")
                .fillBelowY(40, "minecraft:deepslate")
                .fill("minecraft:stone")
                .surface(0.15, 31, "minecraft:grass_block", "minecraft:short_grass", "minecraft:fern")
                .toJson();
    }

    @Test
    void sdfEvalMatchesSphereDistance() {
        String sphere = Sdf.sphere(5).toJson();
        assertEquals(-5.0f, Schematic.sdfEval(sphere, 0, 0, 0), 1e-5);
        assertEquals(3.0f, Schematic.sdfEval(sphere, 8, 0, 0), 1e-5);
        assertEquals(3.0f, Sdf.sphere(5).eval(8, 0, 0), 1e-5);
    }

    @Test
    void invalidJsonThrows() {
        assertThrows(RuntimeException.class, () -> Schematic.fromSdf("{nope", "{}"));
    }

    @Test
    void floatingIslandHasFlatPlateauAndShells() {
        try (Schematic s = Schematic.fromSdf(islandSdf(), islandRules())) {
            assertTrue(s.blockCount() > 1000, "expected real volume, got " + s.blockCount());

            for (int[] col : new int[][]{{0, 0}, {10, -10}, {-15, 15}}) {
                int top = Integer.MIN_VALUE;
                for (int y = 90; y >= 0; y--) {
                    Optional<String> n = s.getBlockName(col[0], y, col[1]);
                    if (n.isPresent() && !n.get().equals("minecraft:air")) { top = y; break; }
                }
                assertEquals(63, top, "plateau top at (" + col[0] + "," + col[1] + ")");
                assertEquals("minecraft:grass_block", s.getBlockName(col[0], 63, col[1]).orElseThrow());
                assertEquals("minecraft:dirt", s.getBlockName(col[0], 62, col[1]).orElseThrow());
            }
        }
    }

    @Test
    void samplingIsDeterministic() {
        try (Schematic a = Schematic.fromSdf(islandSdf(), islandRules());
             Schematic b = Schematic.fromSdf(islandSdf(), islandRules())) {
            assertEquals(a.blockCount(), b.blockCount());
            assertEquals(a.fingerprint("exact"), b.fingerprint("exact"));
        }
    }

    @Test
    void explicitBoundsClipSampling() {
        String slab = Sdf.box(10, 2, 10).toJson();
        String rules = new SdfRules().fill("minecraft:stone").toJson();
        try (Schematic full = Schematic.fromSdf(slab, rules);
             Schematic clipped = Schematic.fromSdf(slab, rules, new int[]{-2, -2, -2, 2, 2, 2})) {
            assertTrue(clipped.blockCount() < full.blockCount());
            assertTrue(clipped.blockCount() > 0);
        }
    }
}
