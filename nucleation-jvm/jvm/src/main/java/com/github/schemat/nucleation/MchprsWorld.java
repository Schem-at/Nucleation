package com.github.schemat.nucleation;

import com.github.schemat.nucleation.exceptions.UnsupportedFeatureException;

import java.lang.ref.Cleaner;
import java.util.ArrayList;
import java.util.List;
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

    /**
     * Creation options mirroring the native {@code SimulationOptions}.
     *
     * @param optimize enable redpiler optimization passes
     * @param ioOnly   only track inputs/outputs (faster, no intermediate wire states)
     * @param customIo positions flattened as {@code [x, y, z, x, y, z, ...]} to
     *                 designate as custom IO nodes for signal injection/monitoring
     */
    public record Options(boolean optimize, boolean ioOnly, int[] customIo) {
        public Options {
            if (customIo != null && customIo.length % 3 != 0) {
                throw new IllegalArgumentException(
                        "customIo length must be a multiple of 3 (x,y,z triples), got " + customIo.length);
            }
        }

        public static Options defaults() { return new Options(true, true, null); }

        public static Options withCustomIo(List<int[]> positions) {
            int[] flat = new int[positions.size() * 3];
            int i = 0;
            for (int[] p : positions) {
                if (p.length != 3) throw new IllegalArgumentException("each position must be [x,y,z]");
                flat[i++] = p[0]; flat[i++] = p[1]; flat[i++] = p[2];
            }
            return new Options(true, true, flat);
        }
    }

    /** A power-level change observed at a custom IO position. */
    public record CustomIoChange(int x, int y, int z, int oldPower, int newPower) {}

    public MchprsWorld(Schematic schematic) {
        Objects.requireNonNull(schematic, "schematic");
        checkSimulationAvailable();
        this.handle = NucleationNative.nMchprsCreate(schematic.handle());
        if (this.handle == 0) {
            throw new UnsupportedFeatureException("MchprsWorld init returned zero handle");
        }
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public MchprsWorld(Schematic schematic, Options options) {
        Objects.requireNonNull(schematic, "schematic");
        Objects.requireNonNull(options, "options");
        checkSimulationAvailable();
        this.handle = NucleationNative.nMchprsCreateWithOptions(
                schematic.handle(), options.optimize(), options.ioOnly(), options.customIo());
        if (this.handle == 0) {
            throw new UnsupportedFeatureException("MchprsWorld init returned zero handle");
        }
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    private static void checkSimulationAvailable() {
        if (!Nucleation.hasSimulation()) {
            throw new UnsupportedFeatureException(
                    "Loaded Nucleation cdylib was not built with simulation support");
        }
    }

    public void tick() {
        checkOpen();
        NucleationNative.nMchprsTick(handle);
    }

    public void tick(int count) {
        checkOpen();
        NucleationNative.nMchprsTickMany(handle, count);
    }

    /** Flushes pending compiler state back into the world (call after ticking). */
    public void flush() {
        checkOpen();
        NucleationNative.nMchprsFlush(handle);
    }

    /** Returns a new Schematic representing the current simulation state. */
    public Schematic getSchematic() {
        checkOpen();
        long h = NucleationNative.nMchprsGetSchematic(handle);
        return Schematic.adopt(h);
    }

    /** Writes the current simulation state back into the world's own schematic. */
    public void syncToSchematic() {
        checkOpen();
        NucleationNative.nMchprsSyncToSchematic(handle);
    }

    /** Injects a signal strength (0-15) at a custom IO position. */
    public void setSignalStrength(int x, int y, int z, int strength) {
        checkOpen();
        NucleationNative.nMchprsSetSignalStrength(handle, x, y, z, strength);
    }

    /** Reads the signal strength (0-15) at a position. */
    public int getSignalStrength(int x, int y, int z) {
        checkOpen();
        return NucleationNative.nMchprsGetSignalStrength(handle, x, y, z);
    }

    public void setLeverPower(int x, int y, int z, boolean powered) {
        checkOpen();
        NucleationNative.nMchprsSetLeverPower(handle, x, y, z, powered);
    }

    public boolean getLeverPower(int x, int y, int z) {
        checkOpen();
        return NucleationNative.nMchprsGetLeverPower(handle, x, y, z);
    }

    /** Whether the block at the position (e.g. a redstone lamp) is lit. */
    public boolean isLit(int x, int y, int z) {
        checkOpen();
        return NucleationNative.nMchprsIsLit(handle, x, y, z);
    }

    /** Simulates a right-click on a block (typically a lever or button). */
    public void onUseBlock(int x, int y, int z) {
        checkOpen();
        NucleationNative.nMchprsOnUseBlock(handle, x, y, z);
    }

    /** Redstone power level (0-15) the block at the position receives. */
    public int getRedstonePower(int x, int y, int z) {
        checkOpen();
        return NucleationNative.nMchprsGetRedstonePower(handle, x, y, z);
    }

    /** Scans custom IO positions and records power changes since the last check. */
    public void checkCustomIoChanges() {
        checkOpen();
        NucleationNative.nMchprsCheckCustomIoChanges(handle);
    }

    /** Returns and clears pending custom-IO changes. */
    public List<CustomIoChange> pollCustomIoChanges() {
        checkOpen();
        int[] flat = NucleationNative.nMchprsPollCustomIoChanges(handle);
        List<CustomIoChange> changes = new ArrayList<>();
        if (flat != null) {
            for (int i = 0; i + 4 < flat.length; i += 5) {
                changes.add(new CustomIoChange(flat[i], flat[i + 1], flat[i + 2], flat[i + 3], flat[i + 4]));
            }
        }
        return changes;
    }

    /** Discards pending custom-IO changes without reading them. */
    public void clearCustomIoChanges() {
        checkOpen();
        NucleationNative.nMchprsClearCustomIoChanges(handle);
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
