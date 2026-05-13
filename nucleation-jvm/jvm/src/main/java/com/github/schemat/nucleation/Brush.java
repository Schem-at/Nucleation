package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;

/**
 * Brush — how a {@link Shape} colours the schematic during a fill operation.
 *
 * <p>The simplest case is {@link #solid(String)} which paints every position
 * with the same block. {@link #color(int, int, int)} dithers across the
 * closest matching colored blocks (concrete, wool, terracotta) in
 * Nucleation's palette.
 */
public final class Brush implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    private Brush(long h) {
        if (h == 0) throw new IllegalStateException("Brush allocation failed");
        this.handle = h;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public static Brush solid(String blockName) {
        return new Brush(NucleationNative.nBrushSolid(blockName));
    }

    public static Brush color(int r, int g, int b) {
        return new Brush(NucleationNative.nBrushColor(r, g, b));
    }

    long handle() { return handle; }

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
        @Override public void run() { if (h != 0) { NucleationNative.nBrushFree(h); h = 0; } }
    }
}
