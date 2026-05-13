package com.github.schemat.nucleation;


import java.io.IOException;
import java.lang.ref.Cleaner;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Optional;

/**
 * Result of a meshing pass — backed by Nucleation's {@code MeshOutput}.
 *
 * <p>The primary export is {@link #glbData()}, which returns a binary
 * {@code .glb} model ready to be served, dropped into Blender, or rendered
 * with any glTF-compatible viewer.
 *
 * <pre>{@code
 * try (MeshResult m = schematic.mesh(pack)) {
 *     Files.write(Path.of("out.glb"), m.glbData());
 * }
 * }</pre>
 */
public final class MeshResult implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    /** Package-private — produced by Schematic and MultiMeshResult only. */
    MeshResult(long h) {
        if (h == 0) throw new IllegalStateException("MeshResult allocation failed");
        this.handle = h;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** Encoded GLB (glTF binary) representation of the mesh. */
    public byte[] glbData() {
        checkOpen();
        return NucleationNative.nMeshResultGlbData(handle);
    }

    /** Convenience: write {@link #glbData()} to a file. */
    public void saveGlb(Path path) throws IOException {
        Files.write(path, glbData());
    }

    public int vertexCount()       { checkOpen(); return NucleationNative.nMeshResultVertexCount(handle); }
    public int triangleCount()     { checkOpen(); return NucleationNative.nMeshResultTriangleCount(handle); }
    public boolean isEmpty()       { checkOpen(); return NucleationNative.nMeshResultIsEmpty(handle); }
    public boolean hasTransparency(){ checkOpen(); return NucleationNative.nMeshResultHasTransparency(handle); }

    /** Bounding box as {@code [minX, minY, minZ, maxX, maxY, maxZ]} in world coordinates. */
    public float[] bounds() { checkOpen(); return NucleationNative.nMeshResultBounds(handle); }

    public int atlasWidth()  { checkOpen(); return NucleationNative.nMeshResultAtlasWidth(handle); }
    public int atlasHeight() { checkOpen(); return NucleationNative.nMeshResultAtlasHeight(handle); }
    /** Raw RGBA pixel data for the texture atlas, row-major. */
    public byte[] atlasRgba(){ checkOpen(); return NucleationNative.nMeshResultAtlasRgba(handle); }

    public int lodLevel() { checkOpen(); return NucleationNative.nMeshResultLodLevel(handle); }

    /** Chunk coordinate {@code [cx, cy, cz]} when this mesh came from a chunked build. */
    public Optional<int[]> chunkCoord() {
        checkOpen();
        int[] c = NucleationNative.nMeshResultChunkCoord(handle);
        return c == null ? Optional.empty() : Optional.of(c);
    }

    public long handle() { return handle; }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("MeshResult is closed");
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
            if (h != 0) { NucleationNative.nMeshResultFree(h); h = 0; }
        }
    }
}
