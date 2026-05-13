package com.github.schemat.nucleation;

import com.github.schemat.nucleation.exceptions.SchematicParseException;

import java.io.IOException;
import java.lang.ref.Cleaner;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Objects;

/**
 * A Minecraft resource pack (textures + models + blockstates) loaded into
 * memory for use with the meshing pipeline.
 *
 * <p>Construct from a zipped pack on disk via {@link #fromFile(Path)} or
 * from raw zip bytes via {@link #fromBytes(byte[])}. Pass the returned
 * instance to {@link com.github.schemat.nucleation.Schematic#mesh(ResourcePack)}
 * or its multi-region sibling.
 *
 * <p>Implements {@link AutoCloseable}; use try-with-resources for
 * deterministic disposal of the native pack.
 */
public final class ResourcePack implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    private ResourcePack(long h) {
        if (h == 0) throw new SchematicParseException("ResourcePack allocation failed");
        this.handle = h;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** Load from a {@code .zip} resource pack on disk. */
    public static ResourcePack fromFile(Path path) {
        Objects.requireNonNull(path, "path");
        return new ResourcePack(NucleationNative.nResourcePackFromFile(path.toAbsolutePath().toString()));
    }

    /** Load from in-memory zip bytes (e.g. fetched from a URL or read from disk). */
    public static ResourcePack fromBytes(byte[] zipBytes) {
        Objects.requireNonNull(zipBytes, "zipBytes");
        return new ResourcePack(NucleationNative.nResourcePackFromBytes(zipBytes));
    }

    /** Convenience: read a file and forward to {@link #fromBytes(byte[])}. */
    public static ResourcePack fromZipPath(Path zipPath) throws IOException {
        return fromBytes(Files.readAllBytes(zipPath));
    }

    public int blockstateCount() { checkOpen(); return NucleationNative.nResourcePackBlockstateCount(handle); }
    public int modelCount()      { checkOpen(); return NucleationNative.nResourcePackModelCount(handle); }
    public int textureCount()    { checkOpen(); return NucleationNative.nResourcePackTextureCount(handle); }

    public long handle() { return handle; }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("ResourcePack is closed");
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
            if (h != 0) { NucleationNative.nResourcePackFree(h); h = 0; }
        }
    }
}
