package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;
import java.util.List;

/**
 * A set of block positions used to anchor circuit IO
 * ({@link CircuitBuilder#withInput}).
 *
 * <p>This is a minimal surface of the native DefinitionRegion (positions and
 * bounds); the full region algebra (merge/subtract/filter) is not yet exposed.
 */
public final class DefinitionRegion implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    private DefinitionRegion(long handle) {
        this.handle = handle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** From positions flattened as {@code [x, y, z, x, y, z, ...]}. */
    public static DefinitionRegion fromPositions(int[] flatXyz) {
        return new DefinitionRegion(NucleationNative.nRegionFromPositions(flatXyz));
    }

    /** From a list of {@code [x, y, z]} triples. */
    public static DefinitionRegion fromPositions(List<int[]> positions) {
        int[] flat = new int[positions.size() * 3];
        int i = 0;
        for (int[] p : positions) {
            if (p.length != 3) throw new IllegalArgumentException("each position must be [x,y,z]");
            flat[i++] = p[0]; flat[i++] = p[1]; flat[i++] = p[2];
        }
        return fromPositions(flat);
    }

    /** Axis-aligned box, inclusive on both corners. */
    public static DefinitionRegion fromBounds(int minX, int minY, int minZ,
                                              int maxX, int maxY, int maxZ) {
        return new DefinitionRegion(
                NucleationNative.nRegionFromBounds(minX, minY, minZ, maxX, maxY, maxZ));
    }

    public DefinitionRegion addPoint(int x, int y, int z) {
        checkOpen();
        NucleationNative.nRegionAddPoint(handle, x, y, z);
        return this;
    }

    public DefinitionRegion addBounds(int minX, int minY, int minZ,
                                      int maxX, int maxY, int maxZ) {
        checkOpen();
        NucleationNative.nRegionAddBounds(handle, minX, minY, minZ, maxX, maxY, maxZ);
        return this;
    }

    /** Number of positions in the region. */
    public long volume() {
        checkOpen();
        return NucleationNative.nRegionVolume(handle);
    }

    long handle() {
        checkOpen();
        return handle;
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("DefinitionRegion is closed");
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
        @Override public void run() { if (h != 0) { NucleationNative.nRegionFree(h); h = 0; } }
    }
}
