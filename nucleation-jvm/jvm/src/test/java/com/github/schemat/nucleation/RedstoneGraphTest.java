package com.github.schemat.nucleation;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

class RedstoneGraphTest {

    @BeforeAll
    static void requireSimulation() {
        assumeTrue(Nucleation.hasSimulation(),
                "cdylib built without the simulation feature — skipping");
    }

    private static Schematic leverLampCircuit() {
        Schematic s = new Schematic("circuit");
        s.setBlock(0, 0, 0, "minecraft:stone");
        s.setBlock(1, 0, 0, "minecraft:stone");
        s.setBlock(2, 0, 0, "minecraft:stone");
        s.setBlock(0, 1, 0, "minecraft:lever[face=floor,facing=east,powered=false]");
        s.setBlock(1, 1, 0, "minecraft:redstone_wire");
        s.setBlock(2, 1, 0, "minecraft:redstone_lamp[lit=false]");
        return s;
    }

    @Test
    void exportAndAnalyze() {
        try (Schematic s = leverLampCircuit();
             MchprsWorld w = new MchprsWorld(s);
             RedstoneGraph g = w.exportGraph()) {
            assertTrue(g.nodeCount() > 0, "graph should have nodes");
            assertFalse(g.hasCycles(), "lever->lamp is combinational");
            assertTrue(g.isCombinational());
            assertTrue(g.weaklyConnectedComponents() >= 1);
            assertTrue(g.criticalPath() >= 1);
            assertTrue(g.maxFanIn() >= 0);
            assertTrue(g.maxFanOut() >= 0);

            String features = g.featuresJson();
            assertTrue(features.startsWith("{"), features);

            String kinds = g.nodeKindCountsJson();
            assertTrue(kinds.contains("Lamp"), kinds);

            assertTrue(g.nodesJson().startsWith("["));
            assertTrue(g.edgesJson().startsWith("["));
            assertTrue(g.stronglyConnectedComponentsJson().startsWith("["));
        }
    }

    @Test
    void jsonRoundTripPreservesStructure() {
        try (Schematic s = leverLampCircuit();
             MchprsWorld w = new MchprsWorld(s);
             RedstoneGraph g = w.exportGraph()) {
            String json = g.toJson();
            try (RedstoneGraph restored = RedstoneGraph.fromJson(json)) {
                assertEquals(g.nodeCount(), restored.nodeCount());
                assertEquals(g.edgeCount(), restored.edgeCount());
                assertTrue(g.isStructurallyEqual(restored));
                assertEquals(g.fingerprint(), restored.fingerprint());
            }
        }
    }

    @Test
    void fingerprintPresets() {
        try (Schematic s = leverLampCircuit();
             MchprsWorld w = new MchprsWorld(s);
             RedstoneGraph g = w.exportGraph()) {
            assertFalse(g.fingerprint("structural").isEmpty());
            assertFalse(g.fingerprint("functional").isEmpty());
            assertFalse(g.fingerprint("exact").isEmpty());
            assertThrows(RuntimeException.class, () -> g.fingerprint("bogus"));
        }
    }

    @Test
    void structuralExportWorks() {
        try (Schematic s = leverLampCircuit();
             MchprsWorld w = new MchprsWorld(s);
             RedstoneGraph g = w.exportGraphStructural()) {
            assertTrue(g.nodeCount() > 0);
        }
    }

    @Test
    void truthTableJson() {
        try (Schematic s = leverLampCircuit()) {
            String json = s.generateTruthTableJson();
            assertTrue(json.startsWith("["), json);
            assertTrue(json.length() > 2, "truth table should not be empty: " + json);
        }
    }
}
