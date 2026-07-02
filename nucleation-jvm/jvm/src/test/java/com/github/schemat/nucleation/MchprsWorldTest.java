package com.github.schemat.nucleation;

import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.*;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

class MchprsWorldTest {

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
    void leverTogglesLamp() {
        try (Schematic s = leverLampCircuit(); MchprsWorld w = new MchprsWorld(s)) {
            assertFalse(w.isLit(2, 1, 0), "lamp should start unlit");
            assertFalse(w.getLeverPower(0, 1, 0), "lever should start unpowered");

            w.onUseBlock(0, 1, 0);
            w.tick(4);
            w.flush();
            assertTrue(w.getLeverPower(0, 1, 0), "lever should be powered after toggle");
            assertTrue(w.isLit(2, 1, 0), "lamp should be lit after lever toggle");

            w.onUseBlock(0, 1, 0);
            w.tick(4);
            w.flush();
            assertFalse(w.getLeverPower(0, 1, 0));
            assertFalse(w.isLit(2, 1, 0), "lamp should be unlit after lever off");
        }
    }

    @Test
    void signalInjectionWithCustomIo() {
        try (Schematic s = new Schematic("sig")) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(1, 0, 0, "minecraft:stone");
            s.setBlock(2, 0, 0, "minecraft:stone");
            s.setBlock(0, 1, 0, "minecraft:redstone_wire");
            s.setBlock(1, 1, 0, "minecraft:redstone_wire");
            s.setBlock(2, 1, 0, "minecraft:redstone_wire");

            MchprsWorld.Options opts = new MchprsWorld.Options(
                    true, true, new int[]{0, 1, 0, 2, 1, 0});
            try (MchprsWorld w = new MchprsWorld(s, opts)) {
                w.setSignalStrength(0, 1, 0, 15);
                w.tick(2);
                w.flush();
                assertTrue(w.getSignalStrength(2, 1, 0) > 0,
                        "signal should propagate to the far custom IO node");
            }
        }
    }

    @Test
    void customIoChangePolling() {
        try (Schematic s = new Schematic("poll")) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(1, 0, 0, "minecraft:stone");
            s.setBlock(0, 1, 0, "minecraft:redstone_wire");
            s.setBlock(1, 1, 0, "minecraft:redstone_wire");

            MchprsWorld.Options opts = new MchprsWorld.Options(
                    true, true, new int[]{0, 1, 0, 1, 1, 0});
            try (MchprsWorld w = new MchprsWorld(s, opts)) {
                w.checkCustomIoChanges();
                w.setSignalStrength(0, 1, 0, 15);
                w.tick(2);
                w.flush();
                w.checkCustomIoChanges();
                List<MchprsWorld.CustomIoChange> changes = w.pollCustomIoChanges();
                assertFalse(changes.isEmpty(), "expected at least one custom IO change");
                assertTrue(w.pollCustomIoChanges().isEmpty(), "poll should clear pending changes");
            }
        }
    }

    @Test
    void syncToSchematicPersistsLeverState() {
        try (Schematic s = new Schematic("sync")) {
            s.setBlock(0, 0, 0, "minecraft:stone");
            s.setBlock(0, 1, 0, "minecraft:lever[face=floor,facing=east,powered=false]");

            try (MchprsWorld w = new MchprsWorld(s)) {
                w.setLeverPower(0, 1, 0, true);
                w.tick(1);
                w.flush();
                w.syncToSchematic();

                try (Schematic updated = w.getSchematic();
                     BlockState b = updated.getBlock(0, 1, 0).orElseThrow()) {
                    assertEquals("true", b.properties().get("powered"),
                            "lever should be powered in synced schematic");
                }
            }
        }
    }

    @Test
    void optionsRejectsMalformedCustomIo() {
        assertThrows(IllegalArgumentException.class,
                () -> new MchprsWorld.Options(true, true, new int[]{1, 2}));
    }
}
