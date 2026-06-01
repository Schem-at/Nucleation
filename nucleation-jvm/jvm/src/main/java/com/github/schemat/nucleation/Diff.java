package com.github.schemat.nucleation;

import com.github.schemat.nucleation.exceptions.UnsupportedFeatureException;

import java.lang.ref.Cleaner;
import java.util.Objects;

/**
 * Result of comparing two {@link Schematic}s — backed by Nucleation's
 * {@code Diff}. Produced by {@link Schematic#diff(Schematic, String)} (and the
 * overrides overload) or reconstructed from JSON via {@link #fromJson(String)}.
 *
 * <p>Owns an opaque native handle, so it implements {@link AutoCloseable} and
 * should be used with try-with-resources:
 *
 * <pre>{@code
 * try (Diff d = before.diff(after, "default")) {
 *     System.out.println(d.distance() + " ops, support=" + d.support());
 *     try (Schematic added = d.added()) {
 *         // inspect the blocks that were added
 *     }
 * }
 * }</pre>
 *
 * <p>The {@link #added()}, {@link #removed()}, {@link #changed()},
 * {@link #swapped()}, and {@link #markers()} accessors each materialize a fresh
 * {@link Schematic} with its own native handle; the caller is responsible for
 * closing them.
 *
 * <p>Without try-with-resources the handle is still freed eventually by a
 * registered {@link Cleaner} action, but explicit {@link #close()} is strongly
 * preferred for predictable native memory release.
 */
public final class Diff implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    /** Package-private — produced by {@link Schematic} and {@link #fromJson}. */
    Diff(long h) {
        if (h == 0) throw new IllegalStateException("Diff allocation failed");
        this.handle = h;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /**
     * Reconstruct a {@link Diff} from its JSON serialization (as produced by
     * {@link #toJson()}).
     *
     * @throws com.github.schemat.nucleation.exceptions.NucleationException
     *         if {@code json} cannot be parsed into a Diff.
     */
    public static Diff fromJson(String json) {
        Objects.requireNonNull(json, "json");
        long h = NucleationNative.nDiffFromJson(json);
        return new Diff(h);
    }

    /** Total edit distance (number of weighted operations) between the two schematics. */
    public int distance() {
        checkOpen();
        return NucleationNative.nDiffDistance(handle);
    }

    /** Structural support score in {@code [0, 1]} — higher means more similar. */
    public float support() {
        checkOpen();
        return NucleationNative.nDiffSupport(handle);
    }

    /** Full JSON serialization of this diff (round-trips via {@link #fromJson}). */
    public String toJson() {
        checkOpen();
        return NucleationNative.nDiffToJson(handle);
    }

    /** Compact JSON summary (counts and headline metrics, not the full op list). */
    public String summaryJson() {
        checkOpen();
        return NucleationNative.nDiffSummaryJson(handle);
    }

    /** Blocks present only in the "after" schematic, as a fresh {@link Schematic}. Caller closes. */
    public Schematic added() {
        checkOpen();
        return Schematic.adopt(NucleationNative.nDiffAdded(handle));
    }

    /** Blocks present only in the "before" schematic, as a fresh {@link Schematic}. Caller closes. */
    public Schematic removed() {
        checkOpen();
        return Schematic.adopt(NucleationNative.nDiffRemoved(handle));
    }

    /** Blocks whose state changed between the two schematics. Caller closes. */
    public Schematic changed() {
        checkOpen();
        return Schematic.adopt(NucleationNative.nDiffChanged(handle));
    }

    /** Blocks that were swapped (relocated) between the two schematics. Caller closes. */
    public Schematic swapped() {
        checkOpen();
        return Schematic.adopt(NucleationNative.nDiffSwapped(handle));
    }

    /** Marker blocks summarizing the diff for visualization. Caller closes. */
    public Schematic markers() {
        checkOpen();
        return Schematic.adopt(NucleationNative.nDiffMarkers(handle));
    }

    /**
     * Render an overlay GLB that highlights this diff on top of the "after"
     * model. {@code afterGlb} is the GLB bytes of the after-state mesh.
     *
     * <p>Requires the {@code meshing} feature in the loaded cdylib; check with
     * {@link Nucleation#hasMeshing()} or expect an
     * {@link UnsupportedFeatureException}.
     */
    public byte[] toOverlayGlb(byte[] afterGlb) {
        ensureMeshing();
        checkOpen();
        Objects.requireNonNull(afterGlb, "afterGlb");
        return NucleationNative.nDiffToOverlayGlb(handle, afterGlb);
    }

    private static void ensureMeshing() {
        if (!Nucleation.hasMeshing()) {
            throw new UnsupportedFeatureException(
                    "Loaded Nucleation cdylib was not built with meshing support");
        }
    }

    public long handle() { return handle; }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("Diff is closed");
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
        @Override public void run() {
            if (h != 0) { NucleationNative.nDiffFree(h); h = 0; }
        }
    }
}
