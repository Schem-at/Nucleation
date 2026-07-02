package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;

/**
 * How a value's bits are laid out over circuit positions.
 */
public final class LayoutFunction implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    private LayoutFunction(long handle) {
        this.handle = handle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** One bit per position (signal 0 or 15). */
    public static LayoutFunction oneToOne() {
        return new LayoutFunction(NucleationNative.nLayoutOneToOne());
    }

    /** Four bits per position (signal 0-15). */
    public static LayoutFunction packed4() {
        return new LayoutFunction(NucleationNative.nLayoutPacked4());
    }

    /** Custom bit-index-to-position mapping. */
    public static LayoutFunction custom(int[] mapping) {
        return new LayoutFunction(NucleationNative.nLayoutCustom(mapping));
    }

    public static LayoutFunction rowMajor(int rows, int cols, int bitsPerElement) {
        return new LayoutFunction(NucleationNative.nLayoutRowMajor(rows, cols, bitsPerElement));
    }

    public static LayoutFunction columnMajor(int rows, int cols, int bitsPerElement) {
        return new LayoutFunction(NucleationNative.nLayoutColumnMajor(rows, cols, bitsPerElement));
    }

    /** Scanline layout for screens. */
    public static LayoutFunction scanline(int width, int height, int bitsPerPixel) {
        return new LayoutFunction(NucleationNative.nLayoutScanline(width, height, bitsPerPixel));
    }

    long handle() {
        checkOpen();
        return handle;
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("LayoutFunction is closed");
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
        @Override public void run() { if (h != 0) { NucleationNative.nLayoutFree(h); h = 0; } }
    }
}
