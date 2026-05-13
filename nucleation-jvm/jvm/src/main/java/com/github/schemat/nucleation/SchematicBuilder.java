package com.github.schemat.nucleation;

import com.github.schemat.nucleation.exceptions.SchematicParseException;

import java.lang.ref.Cleaner;
import java.util.List;
import java.util.Objects;

/**
 * Fluent ASCII-art schematic builder, mirroring {@code PySchematicBuilder}.
 *
 * <pre>{@code
 * try (SchematicBuilder b = new SchematicBuilder()) {
 *     try (Schematic s = b.name("Demo")
 *                          .useStandardPalette()
 *                          .layer("###", "# #", "###")
 *                          .build()) {
 *         byte[] bytes = s.toLitematic();
 *     }
 * }
 * }</pre>
 *
 * <p>Like the Python and Rust counterparts, the builder accumulates state
 * mutably on the same handle — each fluent method returns {@code this}.
 */
public final class SchematicBuilder implements AutoCloseable {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    public SchematicBuilder() {
        this.handle = NucleationNative.nBuilderCreate();
        if (this.handle == 0) throw new IllegalStateException("Failed to allocate SchematicBuilder");
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    private SchematicBuilder(long preMade) {
        this.handle = preMade;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    public SchematicBuilder name(String name) {
        checkOpen();
        NucleationNative.nBuilderName(handle, Objects.requireNonNull(name));
        return this;
    }

    public SchematicBuilder map(char ch, String block) {
        checkOpen();
        NucleationNative.nBuilderMap(handle, ch, Objects.requireNonNull(block));
        return this;
    }

    public SchematicBuilder layer(String... rows) {
        checkOpen();
        NucleationNative.nBuilderLayer(handle, rows);
        return this;
    }

    public SchematicBuilder layer(List<String> rows) {
        return layer(rows.toArray(new String[0]));
    }

    public SchematicBuilder offset(int x, int y, int z) {
        checkOpen();
        NucleationNative.nBuilderOffset(handle, x, y, z);
        return this;
    }

    public SchematicBuilder useStandardPalette() { checkOpen(); NucleationNative.nBuilderUseStandardPalette(handle); return this; }
    public SchematicBuilder useMinimalPalette()  { checkOpen(); NucleationNative.nBuilderUseMinimalPalette(handle); return this; }
    public SchematicBuilder useCompactPalette()  { checkOpen(); NucleationNative.nBuilderUseCompactPalette(handle); return this; }

    /** Returns an empty string on success, or the validation error message. */
    public String validate() {
        checkOpen();
        return NucleationNative.nBuilderValidate(handle);
    }

    /** Build the schematic. Consumes builder state (subsequent fluent calls would operate on a reset builder). */
    public Schematic build() {
        checkOpen();
        long h = NucleationNative.nBuilderBuild(handle);
        if (h == 0) throw new SchematicParseException("SchematicBuilder.build failed");
        return Schematic.adopt(h);
    }

    public String toTemplate() {
        checkOpen();
        return NucleationNative.nBuilderToTemplate(handle);
    }

    public static SchematicBuilder fromTemplate(String template) {
        long h = NucleationNative.nBuilderFromTemplate(Objects.requireNonNull(template));
        if (h == 0) throw new SchematicParseException("SchematicBuilder.fromTemplate failed");
        return new SchematicBuilder(h);
    }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("SchematicBuilder is closed");
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
        @Override public void run() { if (h != 0) { NucleationNative.nBuilderFree(h); h = 0; } }
    }
}
