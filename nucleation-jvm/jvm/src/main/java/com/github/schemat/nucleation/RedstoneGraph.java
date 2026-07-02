package com.github.schemat.nucleation;

import java.lang.ref.Cleaner;
import java.util.Objects;

/**
 * Compiled redstone circuit graph, exported from a running simulation via
 * {@link MchprsWorld#exportGraph()} or {@link MchprsWorld#exportGraphStructural()}.
 *
 * <p>Scalar analyses (cycles, critical path, fan-in/out) are native methods;
 * aggregate views (nodes, edges, features, SCCs, kind counts) are returned as
 * JSON strings so consumers can use whatever JSON library they already ship.
 */
public final class RedstoneGraph implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    RedstoneGraph(long handle) {
        this.handle = handle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** Deserialize a graph previously serialized with {@link #toJson()}. */
    public static RedstoneGraph fromJson(String json) {
        return new RedstoneGraph(NucleationNative.nGraphFromJson(Objects.requireNonNull(json)));
    }

    public int nodeCount() {
        checkOpen();
        return NucleationNative.nGraphNodeCount(handle);
    }

    public int edgeCount() {
        checkOpen();
        return NucleationNative.nGraphEdgeCount(handle);
    }

    /** Full graph serialization (round-trips through {@link #fromJson}). */
    public String toJson() {
        checkOpen();
        return NucleationNative.nGraphToJson(handle);
    }

    /** Nodes as a JSON array (id, kind, position, kind-specific fields). */
    public String nodesJson() {
        checkOpen();
        return NucleationNative.nGraphNodesJson(handle);
    }

    /** Edges as a JSON array (from, to, kind, distance). */
    public String edgesJson() {
        checkOpen();
        return NucleationNative.nGraphEdgesJson(handle);
    }

    /** Node counts per kind as a JSON object, e.g. {@code {"Repeater": 4, "Wire": 12}}. */
    public String nodeKindCountsJson() {
        checkOpen();
        return NucleationNative.nGraphNodeKindCountsJson(handle);
    }

    /** Whether the graph contains feedback loops (memory). */
    public boolean hasCycles() {
        checkOpen();
        return NucleationNative.nGraphHasCycles(handle);
    }

    /** Pure combinational logic: no cycles, output depends only on input. */
    public boolean isCombinational() {
        checkOpen();
        return NucleationNative.nGraphIsCombinational(handle);
    }

    /** Strongly connected components as a JSON array of node-id arrays. */
    public String stronglyConnectedComponentsJson() {
        checkOpen();
        return NucleationNative.nGraphSccsJson(handle);
    }

    /** Number of weakly connected components (independent sub-circuits). */
    public int weaklyConnectedComponents() {
        checkOpen();
        return NucleationNative.nGraphWeaklyConnectedComponents(handle);
    }

    /** Longest input-to-output path in nodes. */
    public int criticalPath() {
        checkOpen();
        return NucleationNative.nGraphCriticalPath(handle);
    }

    /** Longest path weighted by component delays (repeater ticks etc.). */
    public int delayWeightedDepth() {
        checkOpen();
        return NucleationNative.nGraphDelayWeightedDepth(handle);
    }

    public int maxFanIn() {
        checkOpen();
        return NucleationNative.nGraphMaxFanIn(handle);
    }

    public int maxFanOut() {
        checkOpen();
        return NucleationNative.nGraphMaxFanOut(handle);
    }

    /** All computed graph features as a JSON object. */
    public String featuresJson() {
        checkOpen();
        return NucleationNative.nGraphFeaturesJson(handle);
    }

    /** Structural fingerprint (default preset) as a hex string. */
    public String fingerprint() {
        return fingerprint("structural");
    }

    /**
     * Fingerprint as a hex string.
     *
     * @param preset {@code "structural"}, {@code "functional"}, or {@code "exact"}
     */
    public String fingerprint(String preset) {
        checkOpen();
        return NucleationNative.nGraphFingerprint(handle, Objects.requireNonNull(preset));
    }

    /** Whether two graphs have identical structure (ignoring positions). */
    public boolean isStructurallyEqual(RedstoneGraph other) {
        checkOpen();
        other.checkOpen();
        return NucleationNative.nGraphIsStructurallyEqual(handle, other.handle);
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("RedstoneGraph is closed");
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
        @Override public void run() { if (h != 0) { NucleationNative.nGraphFree(h); h = 0; } }
    }
}
