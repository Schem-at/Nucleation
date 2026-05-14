package com.github.schemat.nucleation;

import java.util.Map;

/**
 * Thin, package-private 1:1 mapping to the Rust JNI exports.
 *
 * <p>All methods are {@code static native} and take the opaque handle as the
 * first argument. None of this class is part of the public API — consumers
 * use {@link Schematic}, {@link BlockState}, etc. instead.
 *
 * <p>Bindings are resolved by {@code JNI_OnLoad} via {@code RegisterNatives}
 * the first time {@link NativeLoader#loadOnce()} fires, which the static
 * initialiser triggers.
 */
final class NucleationNative {

    static {
        NativeLoader.loadOnce();
    }

    private NucleationNative() {}

    // ── Meta ────────────────────────────────────────────────────────────────
    static native String  nVersion();
    static native boolean nHasSimulation();
    static native boolean nHasMeshing();

    // ── Schematic ───────────────────────────────────────────────────────────
    static native long    nSchematicCreate(String name);
    static native void    nSchematicFree(long handle);
    static native long    nSchematicCopy(long handle);
    static native String  nSchematicGetName(long handle);
    static native void    nSchematicSetName(long handle, String name);
    static native int[]   nSchematicGetDimensions(long handle);
    static native int     nSchematicGetBlockCount(long handle);
    static native int     nSchematicGetVolume(long handle);
    static native String[] nSchematicGetRegionNames(long handle);
    static native String  nSchematicDebugInfo(long handle);
    static native String  nSchematicPrint(long handle);
    static native String  nSchematicPrintJson(long handle);

    static native boolean nSchematicSetBlockSimple(long handle, int x, int y, int z, String name);
    static native boolean nSchematicSetBlockState(long handle, int x, int y, int z, long stateHandle);
    static native boolean nSchematicSetBlockWithProperties(
            long handle, int x, int y, int z, String name, Map<String, String> properties);
    static native long    nSchematicGetBlock(long handle, int x, int y, int z);
    static native String  nSchematicGetBlockName(long handle, int x, int y, int z);

    static native int     nSchematicFromData(long handle, byte[] data);
    static native int     nSchematicFromLitematic(long handle, byte[] data);
    static native byte[]  nSchematicToLitematic(long handle);
    static native int     nSchematicFromSchematic(long handle, byte[] data);
    static native byte[]  nSchematicToSchematic(long handle);
    static native int     nSchematicFromMcStructure(long handle, byte[] data);
    static native byte[]  nSchematicToMcStructure(long handle);
    static native byte[]  nSchematicToSnapshot(long handle);
    static native int     nSchematicFromSnapshot(long handle, byte[] data);

    static native String  nSchematicGetAllBlocksJson(long handle);
    static native String[] nSchematicGetSupportedImportFormats();
    static native String[] nSchematicGetSupportedExportFormats();
    static native String  nSchematicCountBlockTypesJson(long handle);

    static native void    nSchematicFillCuboid(
            long handle, int x1, int y1, int z1, int x2, int y2, int z2, String name);
    static native void    nSchematicFillSphere(
            long handle, int cx, int cy, int cz, double radius, String name);

    static native String  nDebugSchematic(long handle);
    static native String  nDebugJsonSchematic(long handle);

    // ── BlockState ──────────────────────────────────────────────────────────
    static native long    nBlockStateCreate(String name);
    static native void    nBlockStateFree(long handle);
    static native String  nBlockStateGetName(long handle);
    static native long    nBlockStateWithProperty(long handle, String key, String value);
    static native Map<String, String> nBlockStateGetProperties(long handle);
    static native String  nBlockStateToString(long handle);

    // ── Shape ───────────────────────────────────────────────────────────────
    static native void    nShapeFree(long handle);
    static native long    nShapeSphere(int cx, int cy, int cz, double radius);
    static native long    nShapeCuboid(int x1, int y1, int z1, int x2, int y2, int z2);
    static native long    nShapeEllipsoid(int cx, int cy, int cz, double rx, double ry, double rz);
    static native long    nShapeCylinder(
            double bx, double by, double bz, double ax, double ay, double az,
            double radius, double height);
    static native long    nShapeCone(
            double apx, double apy, double apz, double ax, double ay, double az,
            double radius, double height);
    static native long    nShapeTorus(
            double cx, double cy, double cz, double majorRadius, double minorRadius,
            double ax, double ay, double az);
    static native long    nShapePyramid(
            double bx, double by, double bz, double halfX, double halfZ, double height,
            double ax, double ay, double az);
    static native long    nShapeDisk(
            double cx, double cy, double cz, double radius,
            double nx, double ny, double nz, double thickness);
    static native long    nShapePlane(
            double ox, double oy, double oz,
            double ux, double uy, double uz,
            double vx, double vy, double vz,
            double uExtent, double vExtent, double thickness);
    static native long    nShapeTriangle(
            double ax, double ay, double az,
            double bx, double by, double bz,
            double cx, double cy, double cz,
            double thickness);
    static native long    nShapeLine(
            double x1, double y1, double z1, double x2, double y2, double z2, double thickness);
    static native long    nShapeBezier(double[] controlPoints, double thickness, int resolution);
    static native long    nShapeUnion(long a, long b);
    static native long    nShapeIntersection(long a, long b);
    static native long    nShapeDifference(long a, long b);
    static native long    nShapeHollow(long inner, int thickness);
    static native boolean nShapeContains(long handle, int x, int y, int z);
    static native int[]   nShapeBounds(long handle);

    // ── Brush ───────────────────────────────────────────────────────────────
    static native void    nBrushFree(long handle);
    static native long    nBrushSolid(String blockName);
    static native long    nBrushColor(int r, int g, int b);

    // ── BuildingTool (stateless; takes schematic+shape+brush handles) ──────
    static native void    nBuildingFill(long schematicHandle, long shapeHandle, long brushHandle);
    static native void    nBuildingRstack(
            long schematicHandle, long shapeHandle, long brushHandle,
            int count, int dx, int dy, int dz);

    // ── Builder ─────────────────────────────────────────────────────────────
    static native long    nBuilderCreate();
    static native void    nBuilderFree(long handle);
    static native void    nBuilderName(long handle, String name);
    static native void    nBuilderMap(long handle, char ch, String block);
    static native void    nBuilderLayer(long handle, String[] rows);
    static native void    nBuilderOffset(long handle, int x, int y, int z);
    static native void    nBuilderUseStandardPalette(long handle);
    static native void    nBuilderUseMinimalPalette(long handle);
    static native void    nBuilderUseCompactPalette(long handle);
    static native String  nBuilderValidate(long handle);
    static native long    nBuilderBuild(long handle);
    static native String  nBuilderToTemplate(long handle);
    static native long    nBuilderFromTemplate(String template);

    // ── Simulation (feature-gated; throws UnsatisfiedLinkError if absent) ──
    static native long    nMchprsCreate(long schematicHandle);
    static native void    nMchprsFree(long handle);
    static native void    nMchprsTick(long handle);
    static native void    nMchprsTickMany(long handle, int count);
    static native long    nMchprsGetSchematic(long handle);

    // ── Meshing (feature-gated) ────────────────────────────────────────────
    static native long    nResourcePackFromFile(String path);
    static native long    nResourcePackFromBytes(byte[] data);
    static native void    nResourcePackFree(long handle);
    static native int     nResourcePackBlockstateCount(long handle);
    static native int     nResourcePackModelCount(long handle);
    static native int     nResourcePackTextureCount(long handle);

    static native long    nMeshConfigCreate(
            boolean cullHiddenFaces, boolean ambientOcclusion, float aoIntensity,
            String biome, int atlasMaxSize, boolean cullOccludedBlocks, boolean greedyMeshing);
    static native long    nMeshConfigDefault();
    static native void    nMeshConfigFree(long handle);
    static native boolean nMeshConfigGetCullHiddenFaces(long handle);
    static native void    nMeshConfigSetCullHiddenFaces(long handle, boolean value);
    static native boolean nMeshConfigGetAmbientOcclusion(long handle);
    static native void    nMeshConfigSetAmbientOcclusion(long handle, boolean value);
    static native float   nMeshConfigGetAoIntensity(long handle);
    static native void    nMeshConfigSetAoIntensity(long handle, float value);
    static native String  nMeshConfigGetBiome(long handle);
    static native void    nMeshConfigSetBiome(long handle, String value);
    static native int     nMeshConfigGetAtlasMaxSize(long handle);
    static native void    nMeshConfigSetAtlasMaxSize(long handle, int value);
    static native boolean nMeshConfigGetCullOccludedBlocks(long handle);
    static native void    nMeshConfigSetCullOccludedBlocks(long handle, boolean value);
    static native boolean nMeshConfigGetGreedyMeshing(long handle);
    static native void    nMeshConfigSetGreedyMeshing(long handle, boolean value);

    static native void    nMeshResultFree(long handle);
    static native byte[]  nMeshResultGlbData(long handle);
    static native int     nMeshResultVertexCount(long handle);
    static native int     nMeshResultTriangleCount(long handle);
    static native boolean nMeshResultIsEmpty(long handle);
    static native boolean nMeshResultHasTransparency(long handle);
    static native float[] nMeshResultBounds(long handle);
    static native int     nMeshResultAtlasWidth(long handle);
    static native int     nMeshResultAtlasHeight(long handle);
    static native byte[]  nMeshResultAtlasRgba(long handle);
    static native int     nMeshResultLodLevel(long handle);
    static native int[]   nMeshResultChunkCoord(long handle);

    static native void    nMultiMeshFree(long handle);
    static native int     nMultiMeshSize(long handle);
    static native String[] nMultiMeshRegionNames(long handle);
    static native long    nMultiMeshGet(long handle, String regionName);
    static native long[]  nMultiMeshAllHandles(long handle);

    static native long    nSchematicMeshByRegion(long schematicHandle, long packHandle, long configHandle);
    static native long    nSchematicMeshSingle(long schematicHandle, long packHandle, long configHandle);
    static native byte[]  nSchematicMeshAnimated(long schematicHandle, long packHandle, String timelineJson);
}
