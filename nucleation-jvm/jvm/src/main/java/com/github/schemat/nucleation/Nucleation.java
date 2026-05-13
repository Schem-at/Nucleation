package com.github.schemat.nucleation;

/**
 * Top-level entry point for the Nucleation JVM bindings.
 *
 * <p>Provides metadata helpers and acts as a single touchpoint that ensures
 * the native library is loaded before any wrapper class is instantiated.
 * Most consumers will instead start with {@link Schematic#fromBytes(byte[])}
 * or one of the format-specific helpers.
 */
public final class Nucleation {

    private Nucleation() {}

    static {
        // Force-load the cdylib on first reference.
        NucleationNative.nVersion();
    }

    /** Returns the version string baked into the native cdylib. */
    public static String version() {
        return NucleationNative.nVersion();
    }

    /** Reports whether the loaded cdylib was compiled with the {@code simulation} feature. */
    public static boolean hasSimulation() {
        return NucleationNative.nHasSimulation();
    }

    /** Reports whether the loaded cdylib was compiled with the {@code meshing} feature. */
    public static boolean hasMeshing() {
        return NucleationNative.nHasMeshing();
    }
}
