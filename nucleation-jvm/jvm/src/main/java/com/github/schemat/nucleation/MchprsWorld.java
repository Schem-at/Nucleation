package com.github.schemat.nucleation;

import com.github.schemat.nucleation.exceptions.UnsupportedFeatureException;

import java.lang.ref.Cleaner;
import java.util.Objects;

/**
 * MCHPRS-backed redstone simulation world.
 *
 * <p>Only available when the loaded cdylib was compiled with the
 * {@code simulation} feature. Check via {@link Nucleation#hasSimulation()}
 * before constructing — the constructor throws {@link UnsupportedFeatureException}
 * if simulation is not available.
 */
public final class MchprsWorld implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    public MchprsWorld(Schematic schematic) {
        Objects.requireNonNull(schematic, "schematic");
        if (!Nucleation.hasSimulation()) {
            throw new UnsupportedFeatureException(
                    "Loaded Nucleation cdylib was not built with simulation support");
        }
        this.handle = NucleationNative.nMchprsCreate(schematic.handle());
        if (this.handle == 0) {
            throw new UnsupportedFeatureException("MchprsWorld init returned zero handle");
        }
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public void tick() {
        checkOpen();
        NucleationNative.nMchprsTick(handle);
    }

    public void tick(int count) {
        checkOpen();
        NucleationNative.nMchprsTickMany(handle, count);
    }

    /** Returns a new Schematic representing the current simulation state. */
    public Schematic getSchematic() {
        checkOpen();
        long h = NucleationNative.nMchprsGetSchematic(handle);
        return Schematic.adopt(h);
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("MchprsWorld is closed");
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
        @Override public void run() { if (h != 0) { NucleationNative.nMchprsFree(h); h = 0; } }
    }
}
