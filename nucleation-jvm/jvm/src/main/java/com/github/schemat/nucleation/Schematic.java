package com.github.schemat.nucleation;

import com.github.schemat.nucleation.exceptions.SchematicParseException;
import com.github.schemat.nucleation.exceptions.UnsupportedFeatureException;

import java.lang.ref.Cleaner;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Iterator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Spliterator;
import java.util.Spliterators;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Stream;
import java.util.stream.StreamSupport;

/**
 * The primary entry point: a Minecraft schematic in Nucleation's universal
 * representation, with format-agnostic load/save and block manipulation.
 *
 * <p>Implements {@link AutoCloseable} so try-with-resources is the
 * recommended pattern:
 *
 * <pre>{@code
 * try (Schematic s = Schematic.fromLitematic(bytes)) {
 *     for (Block b : s) System.out.println(b);
 * }
 * }</pre>
 *
 * <p>Without try-with-resources, the native handle is still freed eventually
 * by a registered {@link Cleaner} action. Explicit {@link #close()} is
 * strongly preferred for predictable memory release in long-running JVMs.
 */
public final class Schematic implements AutoCloseable, Iterable<Block> {

    private static final Cleaner CLEANER = Cleaner.create();

    private long handle;
    private final Cleaner.Cleanable cleanable;

    /** Create an empty schematic with the given name. */
    public Schematic(String name) {
        Objects.requireNonNull(name, "name");
        this.handle = NucleationNative.nSchematicCreate(name);
        if (this.handle == 0) throw new IllegalStateException("Failed to allocate Schematic");
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** Create an empty schematic named "Untitled". */
    public Schematic() { this("Untitled"); }

    private Schematic(long preMadeHandle) {
        this.handle = preMadeHandle;
        this.cleanable = CLEANER.register(this, new HandleCleaner(handle));
    }

    /** Package-private factory used by {@link SchematicBuilder} to wrap a native handle. */
    static Schematic adopt(long h) {
        if (h == 0) throw new IllegalStateException("zero handle");
        return new Schematic(h);
    }

    // ── Factory methods ─────────────────────────────────────────────────────

    /** Load from any supported format via auto-detection. */
    public static Schematic fromBytes(byte[] data) {
        Schematic s = new Schematic();
        int rc = NucleationNative.nSchematicFromData(s.handle, data);
        if (rc < 0) { s.close(); throw new SchematicParseException("from_bytes failed: rc=" + rc); }
        return s;
    }

    public static Schematic fromLitematic(byte[] data) { return load(data, NucleationNative::nSchematicFromLitematic, "litematic"); }
    public static Schematic fromSchematic(byte[] data) { return load(data, NucleationNative::nSchematicFromSchematic, "schematic"); }
    public static Schematic fromMcStructure(byte[] data) { return load(data, NucleationNative::nSchematicFromMcStructure, "mcstructure"); }
    public static Schematic fromSnapshot(byte[] data) { return load(data, NucleationNative::nSchematicFromSnapshot, "snapshot"); }

    private interface Loader { int load(long handle, byte[] data); }
    private static Schematic load(byte[] data, Loader loader, String label) {
        Schematic s = new Schematic();
        int rc = loader.load(s.handle, data);
        if (rc < 0) { s.close(); throw new SchematicParseException(label + " parse failed: rc=" + rc); }
        return s;
    }

    // ── Save ────────────────────────────────────────────────────────────────

    public byte[] toLitematic()   { checkOpen(); return NucleationNative.nSchematicToLitematic(handle); }
    public byte[] toSchematic()   { checkOpen(); return NucleationNative.nSchematicToSchematic(handle); }
    public byte[] toMcStructure() { checkOpen(); return NucleationNative.nSchematicToMcStructure(handle); }
    public byte[] toSnapshot()    { checkOpen(); return NucleationNative.nSchematicToSnapshot(handle); }

    // ── Identity / metadata ─────────────────────────────────────────────────

    public String name() { checkOpen(); return NucleationNative.nSchematicGetName(handle); }
    public void setName(String name) { checkOpen(); NucleationNative.nSchematicSetName(handle, Objects.requireNonNull(name)); }

    public Dimensions dimensions() {
        checkOpen();
        int[] d = NucleationNative.nSchematicGetDimensions(handle);
        return new Dimensions(d[0], d[1], d[2]);
    }

    public int blockCount() { checkOpen(); return NucleationNative.nSchematicGetBlockCount(handle); }
    public int volume()     { checkOpen(); return NucleationNative.nSchematicGetVolume(handle); }
    public List<String> regionNames() { checkOpen(); return List.of(NucleationNative.nSchematicGetRegionNames(handle)); }
    public String debugInfo() { checkOpen(); return NucleationNative.nSchematicDebugInfo(handle); }
    public String print()     { checkOpen(); return NucleationNative.nSchematicPrint(handle); }
    public String printJson() { checkOpen(); return NucleationNative.nSchematicPrintJson(handle); }

    public static List<String> supportedImportFormats() { return List.of(NucleationNative.nSchematicGetSupportedImportFormats()); }
    public static List<String> supportedExportFormats() { return List.of(NucleationNative.nSchematicGetSupportedExportFormats()); }

    // ── Block manipulation ─────────────────────────────────────────────────

    /** Set a block by name (e.g. {@code "minecraft:stone"}). Returns whether anything changed. */
    public boolean setBlock(int x, int y, int z, String name) {
        checkOpen();
        return NucleationNative.nSchematicSetBlockSimple(handle, x, y, z, Objects.requireNonNull(name));
    }

    /** Set a block to a specific state. */
    public boolean setBlock(int x, int y, int z, BlockState state) {
        checkOpen();
        return NucleationNative.nSchematicSetBlockState(handle, x, y, z, state.handle());
    }

    /** Set a block with inline properties. */
    public boolean setBlock(int x, int y, int z, String name, Map<String, String> properties) {
        checkOpen();
        Objects.requireNonNull(name);
        Map<String, String> p = properties == null ? Map.of() : properties;
        return NucleationNative.nSchematicSetBlockWithProperties(handle, x, y, z, name, p);
    }

    /** Returns the block at (x,y,z), or empty if unset / out of bounds. Caller closes. */
    public Optional<BlockState> getBlock(int x, int y, int z) {
        checkOpen();
        long h = NucleationNative.nSchematicGetBlock(handle, x, y, z);
        return h == 0 ? Optional.empty() : Optional.of(new BlockState(h));
    }

    /** Returns just the block name at (x,y,z) without allocating a state handle. */
    public Optional<String> getBlockName(int x, int y, int z) {
        checkOpen();
        String n = NucleationNative.nSchematicGetBlockName(handle, x, y, z);
        return Optional.ofNullable(n);
    }

    public void fillCuboid(int x1, int y1, int z1, int x2, int y2, int z2, String blockName) {
        checkOpen();
        NucleationNative.nSchematicFillCuboid(handle, x1, y1, z1, x2, y2, z2, Objects.requireNonNull(blockName));
    }

    public void fillSphere(int cx, int cy, int cz, double radius, String blockName) {
        checkOpen();
        NucleationNative.nSchematicFillSphere(handle, cx, cy, cz, radius, Objects.requireNonNull(blockName));
    }

    // ── Meshing ─────────────────────────────────────────────────────────────

    /**
     * Mesh the entire schematic into a single GLB-capable mesh using the
     * default {@link MeshConfig}. The returned {@link MeshResult} aggregates
     * all regions (or the only region, for single-region schematics).
     *
     * <p>Requires the {@code meshing} feature in the loaded cdylib; check
     * with {@link Nucleation#hasMeshing()} or expect an
     * {@link UnsupportedFeatureException}.
     */
    public MeshResult mesh(ResourcePack pack) { return mesh(pack, null); }

    public MeshResult mesh(ResourcePack pack, MeshConfig config) {
        ensureMeshing();
        checkOpen();
        long cfg = config == null ? 0 : config.handle();
        long h = NucleationNative.nSchematicMeshSingle(handle, pack.handle(), cfg);
        if (h == 0) throw new IllegalStateException("Schematic produced no geometry");
        return new MeshResult(h);
    }

    /** One {@link MeshResult} per region, alphabetically ordered. */
    public MultiMeshResult meshByRegion(ResourcePack pack) { return meshByRegion(pack, null); }

    public MultiMeshResult meshByRegion(ResourcePack pack, MeshConfig config) {
        ensureMeshing();
        checkOpen();
        long cfg = config == null ? 0 : config.handle();
        long h = NucleationNative.nSchematicMeshByRegion(handle, pack.handle(), cfg);
        return new MultiMeshResult(h);
    }

    /**
     * Build an <em>animated</em> GLB replaying a captured scenario. This
     * schematic is the initial world state; {@code timelineJson} is the decoded
     * MCAP event timeline — JSON with {@code origin}, {@code tick_ms}, and a
     * list of {@code set_block} / {@code piston} events. Returns the GLB bytes
     * directly (block-state variants toggle via STEP scale tracks, piston-pushed
     * blocks slide via LINEAR translation tracks).
     */
    public byte[] meshAnimated(ResourcePack pack, String timelineJson) {
        ensureMeshing();
        checkOpen();
        return NucleationNative.nSchematicMeshAnimated(
                handle, pack.handle(), Objects.requireNonNull(timelineJson));
    }

    private static void ensureMeshing() {
        if (!Nucleation.hasMeshing()) {
            throw new UnsupportedFeatureException(
                    "Loaded Nucleation cdylib was not built with meshing support");
        }
    }

    /** Deep-clone the schematic. */
    public Schematic copy() {
        checkOpen();
        long h = NucleationNative.nSchematicCopy(handle);
        return new Schematic(h);
    }

    /** Histogram of block-name → count. */
    public Map<String, Integer> countBlockTypes() {
        checkOpen();
        String json = NucleationNative.nSchematicCountBlockTypesJson(handle);
        return parseCountsJson(json);
    }

    // ── Iteration ───────────────────────────────────────────────────────────

    @Override
    public Iterator<Block> iterator() {
        checkOpen();
        // One JNI call dumps every block as JSON; we decode once into a list
        // and iterate it in pure Java. Avoids per-block crossings.
        String json = NucleationNative.nSchematicGetAllBlocksJson(handle);
        List<Block> blocks = parseBlocksJson(json);
        return blocks.iterator();
    }

    public Stream<Block> stream() {
        return StreamSupport.stream(
                Spliterators.spliteratorUnknownSize(iterator(), Spliterator.ORDERED | Spliterator.NONNULL),
                false);
    }

    long handle() { return handle; }

    private void checkOpen() {
        if (handle == 0) throw new IllegalStateException("Schematic is closed");
    }

    @Override
    public void close() {
        if (handle != 0) {
            handle = 0;
            cleanable.clean();
        }
    }

    @Override
    public String toString() {
        if (handle == 0) return "Schematic{closed}";
        return debugInfo();
    }

    // ── JSON decode helpers (tiny hand-rolled — no Jackson dep) ────────────

    private static final Pattern BLOCK_OBJ = Pattern.compile(
            "\\{\"x\":(-?\\d+),\"y\":(-?\\d+),\"z\":(-?\\d+),\"name\":\"([^\"]+)\",\"properties\":(\\{[^}]*\\})\\}");
    private static final Pattern PROP_KV = Pattern.compile("\"([^\"]+)\":\"([^\"]*)\"");

    private static List<Block> parseBlocksJson(String json) {
        List<Block> out = new ArrayList<>();
        if (json == null || json.length() < 2) return out;
        Matcher m = BLOCK_OBJ.matcher(json);
        while (m.find()) {
            int x = Integer.parseInt(m.group(1));
            int y = Integer.parseInt(m.group(2));
            int z = Integer.parseInt(m.group(3));
            String name = unescape(m.group(4));
            String propsJson = m.group(5);
            Map<String, String> props = parseProps(propsJson);
            out.add(new Block(x, y, z, name, props));
        }
        return out;
    }

    private static Map<String, String> parseProps(String json) {
        Map<String, String> out = new LinkedHashMap<>();
        Matcher m = PROP_KV.matcher(json);
        while (m.find()) {
            out.put(unescape(m.group(1)), unescape(m.group(2)));
        }
        return out;
    }

    private static Map<String, Integer> parseCountsJson(String json) {
        Map<String, Integer> out = new LinkedHashMap<>();
        if (json == null || json.isBlank()) return out;
        Pattern p = Pattern.compile("\"([^\"]+)\":(-?\\d+)");
        Matcher m = p.matcher(json);
        while (m.find()) {
            out.put(unescape(m.group(1)), Integer.parseInt(m.group(2)));
        }
        return out;
    }

    private static String unescape(String s) {
        return s.replace("\\\"", "\"").replace("\\\\", "\\");
    }

    /** Suppress an unused-import warning for {@code Arrays}. */
    @SuppressWarnings("unused")
    private static final Object KEEP_ARRAYS = Arrays.class;

    private static final class HandleCleaner implements Runnable {
        private long h;
        HandleCleaner(long h) { this.h = h; }
        @Override public void run() {
            if (h != 0) { NucleationNative.nSchematicFree(h); h = 0; }
        }
    }
}
