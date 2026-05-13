package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;

/**
 * Geometric shape — wraps Nucleation's {@code ShapeEnum}.
 *
 * <p>Create via static factories:
 * <pre>{@code
 * try (Shape s = Shape.sphere(0, 64, 0, 8.0)) { ... }
 * }</pre>
 *
 * <p>Composite operations ({@link #union}, {@link #intersection}, {@link #difference},
 * {@link #hollow}) return new {@code Shape} instances; the inputs remain valid.
 */
public final class Shape implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    private Shape(long h) {
        if (h == 0) throw new IllegalStateException("Shape allocation failed");
        this.handle = h;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public static Shape sphere(int cx, int cy, int cz, double radius) {
        return new Shape(NucleationNative.nShapeSphere(cx, cy, cz, radius));
    }

    public static Shape cuboid(int x1, int y1, int z1, int x2, int y2, int z2) {
        return new Shape(NucleationNative.nShapeCuboid(x1, y1, z1, x2, y2, z2));
    }

    public static Shape ellipsoid(int cx, int cy, int cz, double rx, double ry, double rz) {
        return new Shape(NucleationNative.nShapeEllipsoid(cx, cy, cz, rx, ry, rz));
    }

    /** Cylinder along an arbitrary axis vector. For Y-axis use {@link #cylinderY}. */
    public static Shape cylinder(double bx, double by, double bz, double ax, double ay, double az, double radius, double height) {
        return new Shape(NucleationNative.nShapeCylinder(bx, by, bz, ax, ay, az, radius, height));
    }

    public static Shape cylinderY(double bx, double by, double bz, double radius, double height) {
        return cylinder(bx, by, bz, 0, 1, 0, radius, height);
    }

    public static Shape cone(double apx, double apy, double apz, double ax, double ay, double az, double radius, double height) {
        return new Shape(NucleationNative.nShapeCone(apx, apy, apz, ax, ay, az, radius, height));
    }

    public static Shape torus(double cx, double cy, double cz, double majorR, double minorR, double ax, double ay, double az) {
        return new Shape(NucleationNative.nShapeTorus(cx, cy, cz, majorR, minorR, ax, ay, az));
    }

    public static Shape pyramid(double bx, double by, double bz, double halfX, double halfZ, double height, double ax, double ay, double az) {
        return new Shape(NucleationNative.nShapePyramid(bx, by, bz, halfX, halfZ, height, ax, ay, az));
    }

    public static Shape disk(double cx, double cy, double cz, double radius, double nx, double ny, double nz, double thickness) {
        return new Shape(NucleationNative.nShapeDisk(cx, cy, cz, radius, nx, ny, nz, thickness));
    }

    public static Shape plane(double ox, double oy, double oz, double ux, double uy, double uz,
                              double vx, double vy, double vz, double uExtent, double vExtent, double thickness) {
        return new Shape(NucleationNative.nShapePlane(ox, oy, oz, ux, uy, uz, vx, vy, vz, uExtent, vExtent, thickness));
    }

    public static Shape triangle(double ax, double ay, double az, double bx, double by, double bz,
                                 double cx, double cy, double cz, double thickness) {
        return new Shape(NucleationNative.nShapeTriangle(ax, ay, az, bx, by, bz, cx, cy, cz, thickness));
    }

    public static Shape line(double x1, double y1, double z1, double x2, double y2, double z2, double thickness) {
        return new Shape(NucleationNative.nShapeLine(x1, y1, z1, x2, y2, z2, thickness));
    }

    /** Bezier curve from a flat array of triples [x0,y0,z0,x1,y1,z1,...]. */
    public static Shape bezier(double[] controlPoints, double thickness, int resolution) {
        return new Shape(NucleationNative.nShapeBezier(controlPoints, thickness, resolution));
    }

    public static Shape union(Shape a, Shape b) {
        return new Shape(NucleationNative.nShapeUnion(a.handle, b.handle));
    }

    public static Shape intersection(Shape a, Shape b) {
        return new Shape(NucleationNative.nShapeIntersection(a.handle, b.handle));
    }

    public static Shape difference(Shape a, Shape b) {
        return new Shape(NucleationNative.nShapeDifference(a.handle, b.handle));
    }

    public static Shape hollow(Shape inner, int thickness) {
        return new Shape(NucleationNative.nShapeHollow(inner.handle, thickness));
    }

    public boolean contains(int x, int y, int z) {
        checkOpen();
        return NucleationNative.nShapeContains(handle, x, y, z);
    }

    /** Returns axis-aligned bounds as {@code [minX,minY,minZ,maxX,maxY,maxZ]}. */
    public int[] bounds() {
        checkOpen();
        return NucleationNative.nShapeBounds(handle);
    }

    long handle() { return handle; }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("Shape is closed");
    }

    @Override
    public void close() {
        if (handle != 0) {
            handle = 0;
            cleanable.clean();
        }
    }

    private static final class HandleCleaner implements Runnable {
        private long h;
        HandleCleaner(long h) { this.h = h; }
        @Override public void run() { if (h != 0) { NucleationNative.nShapeFree(h); h = 0; } }
    }
}
