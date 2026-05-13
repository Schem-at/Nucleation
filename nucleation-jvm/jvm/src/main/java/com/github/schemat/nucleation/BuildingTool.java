package com.github.schemat.nucleation;

/**
 * Stateless helper for shape+brush fills into a {@link Schematic}.
 *
 * <p>Unlike Rust's {@code BuildingTool<'a>} which borrows the schematic for
 * its lifetime, this Java view is per-call — every method takes the
 * schematic, shape, and brush. The Rust side reconstructs a transient
 * {@code BuildingTool} for each call.
 */
public final class BuildingTool {

    private BuildingTool() {}

    /** Fill {@code shape} into {@code schematic} using {@code brush}. */
    public static void fill(Schematic schematic, Shape shape, Brush brush) {
        NucleationNative.nBuildingFill(schematic.handle(), shape.handle(), brush.handle());
    }

    /** Repeat a fill {@code count} times, translating by {@code (dx,dy,dz)} each iteration. */
    public static void rstack(Schematic schematic, Shape shape, Brush brush,
                              int count, int dx, int dy, int dz) {
        NucleationNative.nBuildingRstack(
                schematic.handle(), shape.handle(), brush.handle(), count, dx, dy, dz);
    }
}
