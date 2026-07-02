package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;
import java.util.Objects;

/**
 * A typed value passed to or returned from a circuit
 * ({@link TypedCircuitExecutor}).
 *
 * <p>Create with the static factories ({@link #ofU32}, {@link #ofBool}, ...);
 * read back with the {@code as*} accessors, which throw when the underlying
 * variant does not match. {@link #typeName()} reports the variant.
 */
public final class Value implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    Value(long handle) {
        this.handle = handle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public static Value ofU32(int value) { return new Value(NucleationNative.nValueU32(value)); }
    public static Value ofU64(long value) { return new Value(NucleationNative.nValueU64(value)); }
    public static Value ofI32(int value) { return new Value(NucleationNative.nValueI32(value)); }
    public static Value ofI64(long value) { return new Value(NucleationNative.nValueI64(value)); }
    public static Value ofF32(float value) { return new Value(NucleationNative.nValueF32(value)); }
    public static Value ofBool(boolean value) { return new Value(NucleationNative.nValueBool(value)); }

    public static Value ofString(String value) {
        return new Value(NucleationNative.nValueString(Objects.requireNonNull(value)));
    }

    /** Raw bit array; index 0 is the least significant bit. */
    public static Value ofBits(boolean[] bits) {
        return new Value(NucleationNative.nValueBits(Objects.requireNonNull(bits)));
    }

    public static Value ofBytes(byte[] bytes) {
        return new Value(NucleationNative.nValueBytes(Objects.requireNonNull(bytes)));
    }

    /** Variant name: U32, U64, I32, I64, F32, Bool, String, BitArray, Bytes, Array, Struct. */
    public String typeName() {
        checkOpen();
        return NucleationNative.nValueTypeName(handle);
    }

    /** Integer value (accepts any integer variant and Bool). */
    public long asLong() {
        checkOpen();
        return NucleationNative.nValueAsI64(handle);
    }

    public int asInt() { return Math.toIntExact(asLong()); }

    public float asF32() {
        checkOpen();
        return NucleationNative.nValueAsF32(handle);
    }

    public boolean asBool() {
        checkOpen();
        return NucleationNative.nValueAsBool(handle);
    }

    public String asString() {
        checkOpen();
        return NucleationNative.nValueAsString(handle);
    }

    public boolean[] asBits() {
        checkOpen();
        return NucleationNative.nValueAsBits(handle);
    }

    public byte[] asBytes() {
        checkOpen();
        return NucleationNative.nValueAsBytes(handle);
    }

    long handle() {
        checkOpen();
        return handle;
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("Value is closed");
    }

    @Override
    public String toString() {
        if (handle == 0) return "Value{closed}";
        return NucleationNative.nValueDebug(handle);
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
        @Override public void run() { if (h != 0) { NucleationNative.nValueFree(h); h = 0; } }
    }
}
