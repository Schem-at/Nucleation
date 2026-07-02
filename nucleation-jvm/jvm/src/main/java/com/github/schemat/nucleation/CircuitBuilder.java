package com.github.schemat.nucleation;

import com.github.schemat.nucleation.exceptions.UnsupportedFeatureException;

import java.lang.ref.Cleaner;
import java.util.List;
import java.util.Objects;

/**
 * Fluent builder for a {@link TypedCircuitExecutor}: declare named, typed
 * inputs and outputs anchored to positions in a {@link Schematic}, then
 * {@link #build()} an executor that runs the circuit under MCHPRS.
 *
 * <p>IO can be declared explicitly ({@link #withInput}), with automatic
 * layout inference ({@link #withInputAuto}), or parsed from in-world signs
 * via {@link #fromInsign(Schematic)}.
 *
 * <p>Only available when the loaded cdylib was compiled with the
 * {@code simulation} feature — check {@link Nucleation#hasSimulation()}.
 */
public final class CircuitBuilder implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    private CircuitBuilder(long handle) {
        this.handle = handle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public CircuitBuilder(Schematic schematic) {
        this(create(schematic));
    }

    private static long create(Schematic schematic) {
        Objects.requireNonNull(schematic, "schematic");
        checkSimulationAvailable();
        return NucleationNative.nCircuitBuilderNew(schematic.handle());
    }

    /** Parse IO declarations from insign-formatted signs in the schematic. */
    public static CircuitBuilder fromInsign(Schematic schematic) {
        Objects.requireNonNull(schematic, "schematic");
        checkSimulationAvailable();
        return new CircuitBuilder(NucleationNative.nCircuitBuilderFromInsign(schematic.handle()));
    }

    private static void checkSimulationAvailable() {
        if (!Nucleation.hasSimulation()) {
            throw new UnsupportedFeatureException(
                    "Loaded Nucleation cdylib was not built with simulation support");
        }
    }

    /** Add an input with an explicit layout (default YXZ position ordering). */
    public CircuitBuilder withInput(String name, IoType type, LayoutFunction layout,
                                    DefinitionRegion region) {
        checkOpen();
        NucleationNative.nCircuitBuilderWithInput(
                handle, Objects.requireNonNull(name), type.handle(), layout.handle(), region.handle());
        return this;
    }

    /** Add an input, inferring the layout from the type and region size. */
    public CircuitBuilder withInputAuto(String name, IoType type, DefinitionRegion region) {
        checkOpen();
        NucleationNative.nCircuitBuilderWithInputAuto(
                handle, Objects.requireNonNull(name), type.handle(), region.handle());
        return this;
    }

    public CircuitBuilder withOutput(String name, IoType type, LayoutFunction layout,
                                     DefinitionRegion region) {
        checkOpen();
        NucleationNative.nCircuitBuilderWithOutput(
                handle, Objects.requireNonNull(name), type.handle(), layout.handle(), region.handle());
        return this;
    }

    public CircuitBuilder withOutputAuto(String name, IoType type, DefinitionRegion region) {
        checkOpen();
        NucleationNative.nCircuitBuilderWithOutputAuto(
                handle, Objects.requireNonNull(name), type.handle(), region.handle());
        return this;
    }

    /** Simulation options (default: optimize=true, ioOnly=false). */
    public CircuitBuilder withOptions(boolean optimize, boolean ioOnly) {
        checkOpen();
        NucleationNative.nCircuitBuilderWithOptions(handle, optimize, ioOnly);
        return this;
    }

    public CircuitBuilder withStateMode(StateMode mode) {
        checkOpen();
        NucleationNative.nCircuitBuilderWithStateMode(handle, mode.code);
        return this;
    }

    /**
     * Validate the declared IO against the schematic.
     *
     * @throws IllegalStateException with the validation message when invalid
     */
    public CircuitBuilder validate() {
        checkOpen();
        String message = NucleationNative.nCircuitBuilderValidate(handle);
        if (message != null) throw new IllegalStateException(message);
        return this;
    }

    /**
     * Build the executor. Consumes the builder — further calls on this
     * builder throw.
     */
    public TypedCircuitExecutor build() {
        checkOpen();
        return new TypedCircuitExecutor(NucleationNative.nCircuitBuilderBuild(handle));
    }

    public int inputCount() {
        checkOpen();
        return NucleationNative.nCircuitBuilderInputCount(handle);
    }

    public int outputCount() {
        checkOpen();
        return NucleationNative.nCircuitBuilderOutputCount(handle);
    }

    /** Declared input names, sorted alphabetically. */
    public List<String> inputNames() {
        checkOpen();
        return List.of(NucleationNative.nCircuitBuilderInputNames(handle));
    }

    /** Declared output names, sorted alphabetically. */
    public List<String> outputNames() {
        checkOpen();
        return List.of(NucleationNative.nCircuitBuilderOutputNames(handle));
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("CircuitBuilder is closed");
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
        @Override public void run() { if (h != 0) { NucleationNative.nCircuitBuilderFree(h); h = 0; } }
    }
}
