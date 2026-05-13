package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;
import java.util.Map;
import java.util.Objects;

/**
 * Immutable Minecraft block state — a block name plus key/value properties.
 *
 * <p>Like {@code PyBlockState}, this is logically immutable: {@link #withProperty}
 * returns a new instance rather than mutating in place. The native handle is
 * cleaned up automatically by a {@link Cleaner}, but consumers should call
 * {@link #close()} (or use try-with-resources) for deterministic disposal.
 */
public final class BlockState implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    /** Create a block state with the given name and no properties. */
    public BlockState(String name) {
        Objects.requireNonNull(name, "name");
        this.handle = NucleationNative.nBlockStateCreate(name);
        if (this.handle == 0) {
            throw new IllegalStateException("Failed to allocate BlockState");
        }
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    BlockState(long preMadeHandle) {
        if (preMadeHandle == 0) throw new IllegalArgumentException("zero handle");
        this.handle = preMadeHandle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public String name() {
        checkOpen();
        return NucleationNative.nBlockStateGetName(handle);
    }

    public Map<String, String> properties() {
        checkOpen();
        Map<String, String> m = NucleationNative.nBlockStateGetProperties(handle);
        return m == null ? Map.of() : m;
    }

    /** Returns a NEW block state with the given property set; this instance is unchanged. */
    public BlockState withProperty(String key, String value) {
        Objects.requireNonNull(key, "key");
        Objects.requireNonNull(value, "value");
        checkOpen();
        long newHandle = NucleationNative.nBlockStateWithProperty(handle, key, value);
        return new BlockState(newHandle);
    }

    long handle() { return handle; }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("BlockState is closed");
    }

    @Override
    public void close() {
        if (handle != 0) {
            handle = 0;
            // cleanable.clean() runs HandleCleaner exactly once and de-registers it,
            // so this both frees and prevents the Cleaner from later double-freeing.
            cleanable.clean();
        }
    }

    @Override
    public String toString() {
        if (handle == 0) return "BlockState{closed}";
        return NucleationNative.nBlockStateToString(handle);
    }

    private static final class HandleCleaner implements Runnable {
        private long h;
        HandleCleaner(long h) { this.h = h; }
        @Override public void run() {
            if (h != 0) { NucleationNative.nBlockStateFree(h); h = 0; }
        }
    }
}
