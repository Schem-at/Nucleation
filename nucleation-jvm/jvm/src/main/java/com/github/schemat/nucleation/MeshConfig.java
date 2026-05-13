package com.github.schemat.nucleation;


import java.lang.ref.Cleaner;

/**
 * Configurable parameters for the meshing pipeline.
 *
 * <p>Mirrors {@code PyMeshConfig}. All fields have sensible defaults; the
 * builder-style chain on this class lets you tweak only what you care about:
 *
 * <pre>{@code
 * try (MeshConfig cfg = new MeshConfig()
 *         .greedyMeshing(true)
 *         .atlasMaxSize(2048)
 *         .biome("minecraft:plains")) {
 *     ...
 * }
 * }</pre>
 */
public final class MeshConfig implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    /** Default config: face culling on, AO on (0.4), 4096 atlas, occluded-block culling on. */
    public MeshConfig() {
        this.handle = NucleationNative.nMeshConfigDefault();
        if (this.handle == 0) throw new IllegalStateException("Failed to allocate MeshConfig");
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** Fully-specified constructor mirroring the Python signature. */
    public MeshConfig(boolean cullHiddenFaces, boolean ambientOcclusion, float aoIntensity,
                      String biome, int atlasMaxSize, boolean cullOccludedBlocks, boolean greedyMeshing) {
        this.handle = NucleationNative.nMeshConfigCreate(
                cullHiddenFaces, ambientOcclusion, aoIntensity, biome,
                atlasMaxSize, cullOccludedBlocks, greedyMeshing);
        if (this.handle == 0) throw new IllegalStateException("Failed to allocate MeshConfig");
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public boolean cullHiddenFaces() { return NucleationNative.nMeshConfigGetCullHiddenFaces(handle); }
    public MeshConfig cullHiddenFaces(boolean value) { NucleationNative.nMeshConfigSetCullHiddenFaces(handle, value); return this; }

    public boolean ambientOcclusion() { return NucleationNative.nMeshConfigGetAmbientOcclusion(handle); }
    public MeshConfig ambientOcclusion(boolean value) { NucleationNative.nMeshConfigSetAmbientOcclusion(handle, value); return this; }

    public float aoIntensity() { return NucleationNative.nMeshConfigGetAoIntensity(handle); }
    public MeshConfig aoIntensity(float value) { NucleationNative.nMeshConfigSetAoIntensity(handle, value); return this; }

    public String biome() { return NucleationNative.nMeshConfigGetBiome(handle); }
    public MeshConfig biome(String value) { NucleationNative.nMeshConfigSetBiome(handle, value); return this; }

    public int atlasMaxSize() { return NucleationNative.nMeshConfigGetAtlasMaxSize(handle); }
    public MeshConfig atlasMaxSize(int value) { NucleationNative.nMeshConfigSetAtlasMaxSize(handle, value); return this; }

    public boolean cullOccludedBlocks() { return NucleationNative.nMeshConfigGetCullOccludedBlocks(handle); }
    public MeshConfig cullOccludedBlocks(boolean value) { NucleationNative.nMeshConfigSetCullOccludedBlocks(handle, value); return this; }

    public boolean greedyMeshing() { return NucleationNative.nMeshConfigGetGreedyMeshing(handle); }
    public MeshConfig greedyMeshing(boolean value) { NucleationNative.nMeshConfigSetGreedyMeshing(handle, value); return this; }

    public long handle() { return handle; }

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
            if (h != 0) { NucleationNative.nMeshConfigFree(h); h = 0; }
        }
    }
}
