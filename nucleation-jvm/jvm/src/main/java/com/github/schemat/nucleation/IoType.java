package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;

/**
 * Semantic type of a circuit input or output — how a {@link Value} maps
 * to bits on redstone positions.
 */
public final class IoType implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    private IoType(long handle) {
        this.handle = handle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** Unsigned integer with the given bit width. */
    public static IoType unsignedInt(int bits) {
        return new IoType(NucleationNative.nIoTypeUnsignedInt(bits));
    }

    /** Signed (two's complement) integer with the given bit width. */
    public static IoType signedInt(int bits) {
        return new IoType(NucleationNative.nIoTypeSignedInt(bits));
    }

    /** 32-bit IEEE 754 float. */
    public static IoType float32() {
        return new IoType(NucleationNative.nIoTypeFloat32());
    }

    /** Single boolean. */
    public static IoType bool() {
        return new IoType(NucleationNative.nIoTypeBoolean());
    }

    /** Fixed-length ASCII string. */
    public static IoType ascii(int chars) {
        return new IoType(NucleationNative.nIoTypeAscii(chars));
    }

    /** Raw bit array (no interpretation). */
    public static IoType bitArray(int bits) {
        return new IoType(NucleationNative.nIoTypeBitArray(bits));
    }

    /** Total number of bits this type occupies. */
    public int bitCount() {
        checkOpen();
        return NucleationNative.nIoTypeBitCount(handle);
    }

    long handle() {
        checkOpen();
        return handle;
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("IoType is closed");
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
        @Override public void run() { if (h != 0) { NucleationNative.nIoTypeFree(h); h = 0; } }
    }
}
