package com.github.schemat.nucleation;

import java.util.Locale;
import java.util.Objects;

/**
 * Fluent builder for SDF JSON trees consumed by
 * {@link Schematic#fromSdf(String, String)}.
 *
 * <p>Immutable: every call returns a new node wrapping the JSON so far.
 * Primitives are centered at the origin; position them with {@link #at},
 * {@link #rotate}, {@link #scale}.
 *
 * <pre>{@code
 * String island = Sdf.superPrism(32, 2.5, 32, 6).at(0, 61, 0)
 *     .smoothUnion(
 *         Sdf.ellipsoid(26, 16, 26).at(0, 48, 0).displace(3.0, 0.07, 42),
 *         4.0)
 *     .toJson();
 * }</pre>
 */
public final class Sdf {

    private final String json;

    private Sdf(String json) { this.json = json; }

    /** The JSON representation of this node tree. */
    public String toJson() { return json; }

    @Override public String toString() { return json; }

    private static String num(double v) {
        if (v == Math.rint(v) && !Double.isInfinite(v)) return String.format(Locale.ROOT, "%.1f", v);
        return String.format(Locale.ROOT, "%s", (float) v);
    }

    private static String vec(double x, double y, double z) {
        return "[" + num(x) + "," + num(y) + "," + num(z) + "]";
    }

    // ── Primitives ──────────────────────────────────────────────────────────

    public static Sdf sphere(double radius) {
        return new Sdf("{\"type\":\"sphere\",\"radius\":" + num(radius) + "}");
    }

    public static Sdf box(double hx, double hy, double hz) {
        return new Sdf("{\"type\":\"box\",\"halfExtents\":" + vec(hx, hy, hz) + "}");
    }

    /** Rounded box; {@code halfExtents} are the full extents including rounding. */
    public static Sdf box(double hx, double hy, double hz, double rounding) {
        return new Sdf("{\"type\":\"box\",\"halfExtents\":" + vec(hx, hy, hz)
                + ",\"rounding\":" + num(rounding) + "}");
    }

    /** Ring in the XZ plane. */
    public static Sdf torus(double majorRadius, double minorRadius) {
        return new Sdf("{\"type\":\"torus\",\"majorRadius\":" + num(majorRadius)
                + ",\"minorRadius\":" + num(minorRadius) + "}");
    }

    public static Sdf capsule(double ax, double ay, double az, double bx, double by, double bz, double radius) {
        return new Sdf("{\"type\":\"capsule\",\"a\":" + vec(ax, ay, az)
                + ",\"b\":" + vec(bx, by, bz) + ",\"radius\":" + num(radius) + "}");
    }

    /** Y-axis aligned. */
    public static Sdf cappedCylinder(double radius, double halfHeight) {
        return new Sdf("{\"type\":\"cappedCylinder\",\"radius\":" + num(radius)
                + ",\"halfHeight\":" + num(halfHeight) + "}");
    }

    /** Y-axis aligned; {@code r1} bottom radius, {@code r2} top radius. */
    public static Sdf cappedCone(double halfHeight, double r1, double r2) {
        return new Sdf("{\"type\":\"cappedCone\",\"halfHeight\":" + num(halfHeight)
                + ",\"r1\":" + num(r1) + ",\"r2\":" + num(r2) + "}");
    }

    /** Unbounded — sampling requires explicit bounds. */
    public static Sdf plane(double nx, double ny, double nz, double offset) {
        return new Sdf("{\"type\":\"plane\",\"normal\":" + vec(nx, ny, nz)
                + ",\"offset\":" + num(offset) + "}");
    }

    /** Approximate distance (iq bound formulation). */
    public static Sdf ellipsoid(double rx, double ry, double rz) {
        return new Sdf("{\"type\":\"ellipsoid\",\"radii\":" + vec(rx, ry, rz) + "}");
    }

    public static Sdf octahedron(double size) {
        return new Sdf("{\"type\":\"octahedron\",\"size\":" + num(size) + "}");
    }

    /** Hexagonal cross-section in XZ, extruded along Y. */
    public static Sdf hexPrism(double radius, double halfHeight) {
        return new Sdf("{\"type\":\"hexPrism\",\"radius\":" + num(radius)
                + ",\"halfHeight\":" + num(halfHeight) + "}");
    }

    /**
     * Superellipse cross-section in XZ extruded along Y with a dead-flat
     * top/bottom — the flat-plateau primitive. Higher {@code exponent} →
     * squarer footprint.
     */
    public static Sdf superPrism(double hx, double hy, double hz, double exponent) {
        return new Sdf("{\"type\":\"superPrism\",\"halfExtents\":" + vec(hx, hy, hz)
                + ",\"exponent\":" + num(exponent) + "}");
    }

    /** Wrap raw SDF JSON (escape hatch for node types without a builder method). */
    public static Sdf raw(String json) {
        return new Sdf(Objects.requireNonNull(json, "json"));
    }

    // ── Operators ───────────────────────────────────────────────────────────

    public Sdf union(Sdf other) {
        return new Sdf("{\"type\":\"union\",\"children\":[" + json + "," + other.json + "]}");
    }

    public Sdf intersect(Sdf other) {
        return new Sdf("{\"type\":\"intersect\",\"children\":[" + json + "," + other.json + "]}");
    }

    /** This shape minus {@code other}. */
    public Sdf subtract(Sdf other) {
        return new Sdf("{\"type\":\"subtract\",\"a\":" + json + ",\"b\":" + other.json + "}");
    }

    public Sdf smoothUnion(Sdf other, double k) {
        return new Sdf("{\"type\":\"smoothUnion\",\"a\":" + json + ",\"b\":" + other.json
                + ",\"k\":" + num(k) + "}");
    }

    public Sdf smoothSubtract(Sdf other, double k) {
        return new Sdf("{\"type\":\"smoothSubtract\",\"a\":" + json + ",\"b\":" + other.json
                + ",\"k\":" + num(k) + "}");
    }

    public Sdf smoothIntersect(Sdf other, double k) {
        return new Sdf("{\"type\":\"smoothIntersect\",\"a\":" + json + ",\"b\":" + other.json
                + ",\"k\":" + num(k) + "}");
    }

    /** Inflate the surface outward by {@code radius}. */
    public Sdf round(double radius) {
        return new Sdf("{\"type\":\"round\",\"child\":" + json + ",\"radius\":" + num(radius) + "}");
    }

    /** Hollow shell of the surface with the given thickness. */
    public Sdf shell(double thickness) {
        return new Sdf("{\"type\":\"shell\",\"child\":" + json + ",\"thickness\":" + num(thickness) + "}");
    }

    // ── Transforms ──────────────────────────────────────────────────────────

    /** Translate (alias: {@link #translate}). */
    public Sdf at(double x, double y, double z) { return translate(x, y, z); }

    public Sdf translate(double x, double y, double z) {
        return new Sdf("{\"type\":\"translate\",\"child\":" + json + ",\"offset\":" + vec(x, y, z) + "}");
    }

    /** Euler rotation in degrees, applied X then Y then Z. */
    public Sdf rotate(double degX, double degY, double degZ) {
        return new Sdf("{\"type\":\"rotate\",\"child\":" + json + ",\"angles\":" + vec(degX, degY, degZ) + "}");
    }

    public Sdf rotateY(double degrees) { return rotate(0, degrees, 0); }

    /** Uniform scale. */
    public Sdf scale(double factor) {
        return new Sdf("{\"type\":\"scale\",\"child\":" + json + ",\"factor\":" + num(factor) + "}");
    }

    /** Mirror across the plane orthogonal to the axis ("x", "y" or "z"). */
    public Sdf mirror(String axis) {
        String a = axis.toLowerCase(Locale.ROOT);
        if (!a.equals("x") && !a.equals("y") && !a.equals("z")) {
            throw new IllegalArgumentException("axis must be x, y or z");
        }
        return new Sdf("{\"type\":\"mirror\",\"child\":" + json + ",\"axis\":\"" + a + "\"}");
    }

    /** Counted repetition: {@code count} instances per side of the origin on each axis. */
    public Sdf repeat(double sx, double sy, double sz, int cx, int cy, int cz) {
        return new Sdf("{\"type\":\"repeat\",\"child\":" + json + ",\"spacing\":" + vec(sx, sy, sz)
                + ",\"count\":[" + cx + "," + cy + "," + cz + "]}");
    }

    /** Infinite repetition (spacing 0 disables an axis) — requires explicit sampling bounds. */
    public Sdf repeat(double sx, double sy, double sz) {
        return new Sdf("{\"type\":\"repeat\",\"child\":" + json + ",\"spacing\":" + vec(sx, sy, sz) + "}");
    }

    // ── Noise ───────────────────────────────────────────────────────────────

    /** FBM surface displacement (3 octaves). */
    public Sdf displace(double amplitude, double frequency, int seed) {
        return displace(amplitude, frequency, seed, 3);
    }

    public Sdf displace(double amplitude, double frequency, int seed, int octaves) {
        return new Sdf("{\"type\":\"displace\",\"child\":" + json
                + ",\"amplitude\":" + num(amplitude) + ",\"frequency\":" + num(frequency)
                + ",\"seed\":" + seed + ",\"octaves\":" + octaves + "}");
    }

    /** Domain warp with seeded value noise. */
    public Sdf warp(double amplitude, double frequency, int seed) {
        return new Sdf("{\"type\":\"warp\",\"child\":" + json
                + ",\"amplitude\":" + num(amplitude) + ",\"frequency\":" + num(frequency)
                + ",\"seed\":" + seed + "}");
    }

    // ── Evaluation convenience ─────────────────────────────────────────────

    /** Signed distance at a point (negative = inside). */
    public float eval(float x, float y, float z) {
        return Schematic.sdfEval(json, x, y, z);
    }

    /** Sample into a schematic with the given material rules. */
    public Schematic toSchematic(SdfRules rules) {
        return Schematic.fromSdf(json, rules.toJson());
    }

    public Schematic toSchematic(SdfRules rules, int[] bounds) {
        return Schematic.fromSdf(json, rules.toJson(), bounds);
    }
}
