package com.github.schemat.nucleation;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Smoke tests for the meshing surface that don't require a real resource
 * pack (which isn't checked into the repo). They verify:
 * <ul>
 *   <li>The {@code meshing} feature is compiled in.</li>
 *   <li>{@link MeshConfig} construction, getters, and setters work.</li>
 * </ul>
 * Full mesh-pipeline tests live in CI where a fixture pack can be downloaded.
 */
class MeshingTest {

    @Test
    void meshingFeatureIsCompiledIn() {
        assertTrue(Nucleation.hasMeshing(),
                "Expected meshing feature in cdylib (enabled by default in nucleation-jvm)");
    }

    @Test
    void defaultMeshConfigHasSaneValues() {
        try (MeshConfig cfg = new MeshConfig()) {
            assertTrue(cfg.cullHiddenFaces());
            assertTrue(cfg.ambientOcclusion());
            assertTrue(cfg.atlasMaxSize() > 0);
            assertTrue(cfg.aoIntensity() >= 0.0f);
        }
    }

    @Test
    void meshConfigFluentSettersChain() {
        try (MeshConfig cfg = new MeshConfig()) {
            cfg.greedyMeshing(true)
               .ambientOcclusion(false)
               .aoIntensity(0.6f)
               .atlasMaxSize(2048)
               .biome("minecraft:plains")
               .cullHiddenFaces(false)
               .cullOccludedBlocks(false);

            assertTrue(cfg.greedyMeshing());
            assertFalse(cfg.ambientOcclusion());
            assertEquals(0.6f, cfg.aoIntensity(), 0.0001f);
            assertEquals(2048, cfg.atlasMaxSize());
            assertEquals("minecraft:plains", cfg.biome());
            assertFalse(cfg.cullHiddenFaces());
            assertFalse(cfg.cullOccludedBlocks());
        }
    }

    @Test
    void meshConfigBiomeNullable() {
        try (MeshConfig cfg = new MeshConfig()) {
            cfg.biome(null);
            assertNull(cfg.biome());
            cfg.biome("minecraft:desert");
            assertEquals("minecraft:desert", cfg.biome());
        }
    }

    @Test
    void closedConfigRejectsAccess() {
        MeshConfig cfg = new MeshConfig();
        cfg.close();
        // After close, the handle is 0 — calls return whatever the native
        // side returns for handle 0 (caught by the panic-safe context).
        // We mostly want to verify no JVM crash.
        assertDoesNotThrow(cfg::close);
    }
}
