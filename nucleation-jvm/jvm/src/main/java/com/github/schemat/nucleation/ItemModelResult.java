package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;
import java.util.List;

/**
 * Result of item-model generation ({@link Schematic#toItemModel}).
 *
 * <p>Only available when the loaded cdylib was compiled with the
 * {@code meshing} feature.
 */
public final class ItemModelResult implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    ItemModelResult(long handle) {
        this.handle = handle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** The generated Minecraft item model JSON. */
    public String modelJson() {
        checkOpen();
        return NucleationNative.nItemModelResultModelJson(handle);
    }

    public int elementCount() {
        checkOpen();
        return NucleationNative.nItemModelResultElementCount(handle);
    }

    public int textureCount() {
        checkOpen();
        return NucleationNative.nItemModelResultTextureCount(handle);
    }

    public int planeCount() {
        checkOpen();
        return NucleationNative.nItemModelResultPlaneCount(handle);
    }

    /** Schematic dimensions {@code [width, height, depth]}. */
    public int[] dimensions() {
        checkOpen();
        return NucleationNative.nItemModelResultDimensions(handle);
    }

    /** Resolved scale factors {@code [sx, sy, sz]}. */
    public float[] scale() {
        checkOpen();
        return NucleationNative.nItemModelResultScale(handle);
    }

    /** Packages this single result as a complete resource pack ZIP. */
    public byte[] toResourcePackZip() {
        checkOpen();
        return NucleationNative.nItemModelResultToResourcePackZip(handle);
    }

    /**
     * Builds one merged resource pack ZIP from several results. Models bound
     * to the same item share one item definition with multiple
     * {@code custom_model_data} cases.
     */
    public static byte[] buildResourcePack(List<ItemModelResult> results) {
        long[] handles = new long[results.size()];
        for (int i = 0; i < results.size(); i++) {
            ItemModelResult r = results.get(i);
            r.checkOpen();
            handles[i] = r.handle;
        }
        return NucleationNative.nItemModelBuildResourcePack(handles);
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("ItemModelResult is closed");
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
        @Override public void run() { if (h != 0) { NucleationNative.nItemModelResultFree(h); h = 0; } }
    }
}
