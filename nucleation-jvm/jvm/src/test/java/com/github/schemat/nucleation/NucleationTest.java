package com.github.schemat.nucleation;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class NucleationTest {

    @Test
    void versionIsNonEmpty() {
        String v = Nucleation.version();
        assertNotNull(v);
        assertFalse(v.isBlank());
        // Version should look like X.Y.Z
        assertTrue(v.matches("\\d+\\.\\d+\\.\\d+.*"), "version=" + v);
    }

    @Test
    void hasSimulationReturnsBoolean() {
        // Just verify the call works; result depends on build features.
        boolean ignored = Nucleation.hasSimulation();
        assertNotNull(Boolean.valueOf(ignored));
    }
}
