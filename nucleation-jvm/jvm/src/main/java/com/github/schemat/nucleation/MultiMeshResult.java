package com.github.schemat.nucleation;


import java.lang.ref.Cleaner;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Optional;

/**
 * One {@link MeshResult} per region of a multi-region schematic. The
 * collection is ordered alphabetically by region name.
 *
 * <p>Implements {@link Iterable} for ergonomic iteration:
 * <pre>{@code
 * try (MultiMeshResult multi = schematic.meshByRegion(pack)) {
 *     for (MeshResult mesh : multi) {
 *         Files.write(Path.of(mesh.bounds()[0] + ".glb"), mesh.glbData());
 *     }
 * }
 * }</pre>
 *
 * <p>Note: iteration produces fresh {@link MeshResult} wrappers — close
 * each one, or close the outer {@code MultiMeshResult} to drop the entire
 * underlying mesh cache.
 */
public final class MultiMeshResult implements AutoCloseable, Iterable<MeshResult> {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    MultiMeshResult(long h) {
        if (h == 0) throw new IllegalStateException("MultiMeshResult allocation failed");
        this.handle = h;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public int size() { checkOpen(); return NucleationNative.nMultiMeshSize(handle); }
    public boolean isEmpty() { return size() == 0; }

    /** Region names in alphabetical order. */
    public List<String> regionNames() {
        checkOpen();
        String[] names = NucleationNative.nMultiMeshRegionNames(handle);
        return names == null ? List.of() : List.of(names);
    }

    /** Get the mesh for a specific region by name. */
    public Optional<MeshResult> get(String regionName) {
        checkOpen();
        long h = NucleationNative.nMultiMeshGet(handle, regionName);
        return h == 0 ? Optional.empty() : Optional.of(new MeshResult(h));
    }

    /** Snapshot of all region meshes; caller must close each entry. */
    public List<MeshResult> asList() {
        checkOpen();
        long[] handles = NucleationNative.nMultiMeshAllHandles(handle);
        List<MeshResult> out = new ArrayList<>(handles.length);
        for (long h : handles) out.add(new MeshResult(h));
        return out;
    }

    @Override
    public Iterator<MeshResult> iterator() {
        return asList().iterator();
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("MultiMeshResult is closed");
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
            if (h != 0) { NucleationNative.nMultiMeshFree(h); h = 0; }
        }
    }
}
