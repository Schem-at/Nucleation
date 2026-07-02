package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;

/**
 * Executes a redstone circuit with named, typed inputs and outputs
 * (built via {@link CircuitBuilder}).
 *
 * <p>Two usage styles:
 * <ul>
 *   <li>One-shot: {@link #execute(Map, int)} — set inputs, run N ticks,
 *       read all outputs.</li>
 *   <li>Manual: {@link #setInput}, {@link #tick}, {@link #flush},
 *       {@link #readOutput} for fine-grained control (pair with
 *       {@link StateMode#MANUAL}).</li>
 * </ul>
 */
public final class TypedCircuitExecutor implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    TypedCircuitExecutor(long handle) {
        this.handle = handle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** Result of one {@link #execute} call. Owns the output {@link Value}s. */
    public static final class ExecutionResult implements AutoCloseable {
        private static final Cleaner CLEANER = Cleaner.create();

        private long handle;
        private final Cleaner.Cleanable cleanable;

        ExecutionResult(long handle) {
            this.handle = handle;
            this.cleanable = CLEANER.register(this, new ResultCleaner(handle));
        }

        public int ticksElapsed() {
            checkOpen();
            return NucleationNative.nExecResultTicks(handle);
        }

        /** Whether the run condition was met (always true for fixed ticks). */
        public boolean conditionMet() {
            checkOpen();
            return NucleationNative.nExecResultConditionMet(handle);
        }

        /** Output names, sorted alphabetically. */
        public List<String> outputNames() {
            checkOpen();
            return List.of(NucleationNative.nExecResultOutputNames(handle));
        }

        /** The value of one output. The caller owns (and should close) the Value. */
        public Value output(String name) {
            checkOpen();
            return new Value(NucleationNative.nExecResultOutput(handle, Objects.requireNonNull(name)));
        }

        /** All outputs as a name-ordered map. The caller owns the Values. */
        public Map<String, Value> outputs() {
            Map<String, Value> map = new LinkedHashMap<>();
            for (String name : outputNames()) {
                map.put(name, output(name));
            }
            return map;
        }

        private void checkOpen() {
            if (handle == 0) throw new IllegalStateException("ExecutionResult is closed");
        }

        @Override
        public void close() {
            if (handle != 0) {
                handle = 0;
                cleanable.clean();
            }
        }

        private static final class ResultCleaner implements Runnable {
            private long h;
            ResultCleaner(long h) { this.h = h; }
            @Override public void run() { if (h != 0) { NucleationNative.nExecResultFree(h); h = 0; } }
        }
    }

    /** Run the circuit for a fixed number of ticks. */
    public ExecutionResult execute(Map<String, Value> inputs, int ticks) {
        return executeInternal(inputs, 0, ticks, 0);
    }

    /** Run until any output changes, checking every {@code checkInterval} ticks. */
    public ExecutionResult executeUntilChange(Map<String, Value> inputs, int maxTicks, int checkInterval) {
        return executeInternal(inputs, 1, maxTicks, checkInterval);
    }

    /** Run until all outputs are stable for {@code stableTicks}. */
    public ExecutionResult executeUntilStable(Map<String, Value> inputs, int stableTicks, int maxTicks) {
        return executeInternal(inputs, 2, stableTicks, maxTicks);
    }

    private ExecutionResult executeInternal(Map<String, Value> inputs, int mode, int p1, int p2) {
        checkOpen();
        String[] names = new String[inputs.size()];
        long[] handles = new long[inputs.size()];
        int i = 0;
        for (Map.Entry<String, Value> e : inputs.entrySet()) {
            names[i] = e.getKey();
            handles[i] = e.getValue().handle();
            i++;
        }
        return new ExecutionResult(
                NucleationNative.nExecutorExecute(handle, names, handles, mode, p1, p2));
    }

    /** Set one input signal without executing. */
    public void setInput(String name, Value value) {
        checkOpen();
        NucleationNative.nExecutorSetInput(handle, Objects.requireNonNull(name), value.handle());
    }

    /** Read one output (flushes first). The caller owns the returned Value. */
    public Value readOutput(String name) {
        checkOpen();
        return new Value(NucleationNative.nExecutorReadOutput(handle, Objects.requireNonNull(name)));
    }

    public void tick(int ticks) {
        checkOpen();
        NucleationNative.nExecutorTick(handle, ticks);
    }

    public void flush() {
        checkOpen();
        NucleationNative.nExecutorFlush(handle);
    }

    /** Reset the circuit to its original schematic state. */
    public void reset() {
        checkOpen();
        NucleationNative.nExecutorReset(handle);
    }

    /** Input names, sorted alphabetically. */
    public List<String> inputNames() {
        checkOpen();
        return List.of(NucleationNative.nExecutorInputNames(handle));
    }

    /** Output names, sorted alphabetically. */
    public List<String> outputNames() {
        checkOpen();
        return List.of(NucleationNative.nExecutorOutputNames(handle));
    }

    public void setStateMode(StateMode mode) {
        checkOpen();
        NucleationNative.nExecutorSetStateMode(handle, mode.code);
    }

    /**
     * Layout description as JSON:
     * {@code {"inputs": {name: {ioType, bitCount, positions: [[x,y,z],...]}}, "outputs": {...}}}.
     */
    public String layoutInfoJson() {
        checkOpen();
        return NucleationNative.nExecutorLayoutInfoJson(handle);
    }

    /** Sync simulation state into the schematic and return a copy of it. */
    public Schematic syncAndGetSchematic() {
        checkOpen();
        return Schematic.adopt(NucleationNative.nExecutorSyncAndGetSchematic(handle));
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("TypedCircuitExecutor is closed");
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
        @Override public void run() { if (h != 0) { NucleationNative.nExecutorFree(h); h = 0; } }
    }
}
