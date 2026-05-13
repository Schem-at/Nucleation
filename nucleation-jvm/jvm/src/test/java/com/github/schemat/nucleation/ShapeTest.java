package com.github.schemat.nucleation;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ShapeTest {

    @Test
    void sphereBounds() {
        try (Shape s = Shape.sphere(0, 0, 0, 3.0)) {
            int[] b = s.bounds();
            assertEquals(6, b.length);
            assertEquals(-3, b[0]);
            assertEquals(3, b[3]);
        }
    }

    @Test
    void cuboidContains() {
        try (Shape s = Shape.cuboid(0, 0, 0, 4, 4, 4)) {
            assertTrue(s.contains(2, 2, 2));
            assertFalse(s.contains(5, 5, 5));
        }
    }

    @Test
    void fillSphereIntoSchematic() {
        try (Schematic schem = new Schematic("sphere");
             Shape shape = Shape.sphere(0, 0, 0, 4.0);
             Brush brush = Brush.solid("minecraft:stone")) {
            BuildingTool.fill(schem, shape, brush);
            assertTrue(schem.blockCount() > 0, "sphere fill should place blocks");
        }
    }

    @Test
    void unionOfTwoCuboids() {
        try (Shape a = Shape.cuboid(0, 0, 0, 2, 2, 2);
             Shape b = Shape.cuboid(5, 0, 0, 7, 2, 2);
             Shape u = Shape.union(a, b)) {
            assertTrue(u.contains(1, 1, 1));
            assertTrue(u.contains(6, 1, 1));
        }
    }
}
