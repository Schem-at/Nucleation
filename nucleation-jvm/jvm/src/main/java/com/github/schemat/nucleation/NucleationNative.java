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
    static native long    nSchematicOpen(String uri);
    static native void    nSchematicSave(long handle, String uri, String version);
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

    // ── SDF generation ─────────────────────────────────────────────────────
    static native long    nSchematicFromSdf(String sdfJson, String rulesJson, int[] bounds);
    static native float   nSdfEval(String sdfJson, float x, float y, float z);

    // ── Simulation (feature-gated; throws UnsatisfiedLinkError if absent) ──
    static native long    nMchprsCreate(long schematicHandle);
    static native long    nMchprsCreateWithOptions(long schematicHandle, boolean optimize, boolean ioOnly, int[] customIo);
    static native void    nMchprsFree(long handle);
    static native void    nMchprsTick(long handle);
    static native void    nMchprsTickMany(long handle, int count);
    static native void    nMchprsFlush(long handle);
    static native long    nMchprsGetSchematic(long handle);
    static native void    nMchprsSyncToSchematic(long handle);
    static native void    nMchprsSetSignalStrength(long handle, int x, int y, int z, int strength);
    static native int     nMchprsGetSignalStrength(long handle, int x, int y, int z);
    static native void    nMchprsSetLeverPower(long handle, int x, int y, int z, boolean powered);
    static native boolean nMchprsGetLeverPower(long handle, int x, int y, int z);
    static native boolean nMchprsIsLit(long handle, int x, int y, int z);
    static native void    nMchprsOnUseBlock(long handle, int x, int y, int z);
    static native int     nMchprsGetRedstonePower(long handle, int x, int y, int z);
    static native void    nMchprsCheckCustomIoChanges(long handle);
    static native int[]   nMchprsPollCustomIoChanges(long handle);
    static native void    nMchprsClearCustomIoChanges(long handle);

    // ── Redstone graph (feature-gated with simulation) ─────────────────────
    static native long    nMchprsExportGraph(long worldHandle);
    static native long    nMchprsExportGraphStructural(long worldHandle);
    static native void    nGraphFree(long handle);
    static native int     nGraphNodeCount(long handle);
    static native int     nGraphEdgeCount(long handle);
    static native String  nGraphToJson(long handle);
    static native long    nGraphFromJson(String json);
    static native String  nGraphNodesJson(long handle);
    static native String  nGraphEdgesJson(long handle);
    static native String  nGraphNodeKindCountsJson(long handle);
    static native boolean nGraphHasCycles(long handle);
    static native boolean nGraphIsCombinational(long handle);
    static native String  nGraphSccsJson(long handle);
    static native int     nGraphWeaklyConnectedComponents(long handle);
    static native int     nGraphCriticalPath(long handle);
    static native int     nGraphDelayWeightedDepth(long handle);
    static native int     nGraphMaxFanIn(long handle);
    static native int     nGraphMaxFanOut(long handle);
    static native String  nGraphFeaturesJson(long handle);
    static native String  nGraphFingerprint(long handle, String preset);
    static native boolean nGraphIsStructurallyEqual(long a, long b);
    static native String  nSchematicGenerateTruthTableJson(long schematicHandle);

    // ── Circuit: Value / IoType / LayoutFunction / DefinitionRegion ────────
    static native long    nValueU32(int value);
    static native long    nValueU64(long value);
    static native long    nValueI32(int value);
    static native long    nValueI64(long value);
    static native long    nValueF32(float value);
    static native long    nValueBool(boolean value);
    static native long    nValueString(String value);
    static native long    nValueBits(boolean[] bits);
    static native long    nValueBytes(byte[] bytes);
    static native String  nValueTypeName(long handle);
    static native long    nValueAsI64(long handle);
    static native float   nValueAsF32(long handle);
    static native boolean nValueAsBool(long handle);
    static native String  nValueAsString(long handle);
    static native boolean[] nValueAsBits(long handle);
    static native byte[]  nValueAsBytes(long handle);
    static native String  nValueDebug(long handle);
    static native void    nValueFree(long handle);

    static native long    nIoTypeUnsignedInt(int bits);
    static native long    nIoTypeSignedInt(int bits);
    static native long    nIoTypeFloat32();
    static native long    nIoTypeBoolean();
    static native long    nIoTypeAscii(int chars);
    static native long    nIoTypeBitArray(int bits);
    static native int     nIoTypeBitCount(long handle);
    static native void    nIoTypeFree(long handle);

    static native long    nLayoutOneToOne();
    static native long    nLayoutPacked4();
    static native long    nLayoutCustom(int[] mapping);
    static native long    nLayoutRowMajor(int rows, int cols, int bitsPerElement);
    static native long    nLayoutColumnMajor(int rows, int cols, int bitsPerElement);
    static native long    nLayoutScanline(int width, int height, int bitsPerPixel);
    static native void    nLayoutFree(long handle);

    static native long    nRegionFromPositions(int[] flatXyz);
    static native long    nRegionFromBounds(int minX, int minY, int minZ, int maxX, int maxY, int maxZ);
    static native void    nRegionAddPoint(long handle, int x, int y, int z);
    static native void    nRegionAddBounds(long handle, int minX, int minY, int minZ, int maxX, int maxY, int maxZ);
    static native long    nRegionVolume(long handle);
    static native void    nRegionFree(long handle);

    // ── CircuitBuilder / TypedCircuitExecutor ───────────────────────────────
    static native long    nCircuitBuilderNew(long schematicHandle);
    static native long    nCircuitBuilderFromInsign(long schematicHandle);
    static native void    nCircuitBuilderWithInput(long handle, String name, long ioType, long layout, long region);
    static native void    nCircuitBuilderWithInputAuto(long handle, String name, long ioType, long region);
    static native void    nCircuitBuilderWithOutput(long handle, String name, long ioType, long layout, long region);
    static native void    nCircuitBuilderWithOutputAuto(long handle, String name, long ioType, long region);
    static native void    nCircuitBuilderWithOptions(long handle, boolean optimize, boolean ioOnly);
    static native void    nCircuitBuilderWithStateMode(long handle, int mode);
    static native String  nCircuitBuilderValidate(long handle);
    static native long    nCircuitBuilderBuild(long handle);
    static native int     nCircuitBuilderInputCount(long handle);
    static native int     nCircuitBuilderOutputCount(long handle);
    static native String[] nCircuitBuilderInputNames(long handle);
    static native String[] nCircuitBuilderOutputNames(long handle);
    static native void    nCircuitBuilderFree(long handle);

    static native void    nExecutorSetInput(long handle, String name, long valueHandle);
    static native long    nExecutorReadOutput(long handle, String name);
    static native long    nExecutorExecute(long handle, String[] names, long[] valueHandles, int mode, int p1, int p2);
    static native void    nExecutorTick(long handle, int ticks);
    static native void    nExecutorFlush(long handle);
    static native void    nExecutorReset(long handle);
    static native String[] nExecutorInputNames(long handle);
    static native String[] nExecutorOutputNames(long handle);
    static native void    nExecutorSetStateMode(long handle, int mode);
    static native String  nExecutorLayoutInfoJson(long handle);
    static native long    nExecutorSyncAndGetSchematic(long handle);
    static native void    nExecutorFree(long handle);

    static native int     nExecResultTicks(long handle);
    static native boolean nExecResultConditionMet(long handle);
    static native String[] nExecResultOutputNames(long handle);
    static native long    nExecResultOutput(long handle, String name);
    static native void    nExecResultFree(long handle);

    // ── Meshing (feature-gated) ────────────────────────────────────────────
    static native long    nResourcePackFromFile(String path);
    static native long    nResourcePackFromBytes(byte[] data);
    static native void    nResourcePackFree(long handle);
    static native int     nResourcePackBlockstateCount(long handle);
    static native int     nResourcePackModelCount(long handle);
    static native int     nResourcePackTextureCount(long handle);

    static native long    nSchematicToItemModel(long schematicHandle, long packHandle,
                                                String modelName, String namespace, boolean center,
                                                int textureResolution, String item, String customModelData,
                                                int scaleMode, float sx, float sy, float sz);
    static native void    nItemModelResultFree(long handle);
    static native String  nItemModelResultModelJson(long handle);
    static native int     nItemModelResultElementCount(long handle);
    static native int     nItemModelResultTextureCount(long handle);
    static native int     nItemModelResultPlaneCount(long handle);
    static native int[]   nItemModelResultDimensions(long handle);
    static native float[] nItemModelResultScale(long handle);
    static native byte[]  nItemModelResultToResourcePackZip(long handle);
    static native byte[]  nItemModelBuildResourcePack(long[] resultHandles);

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

    // ── Fingerprint (stateless; takes schematic handle(s) as leading args) ──
    static native String  nFingerprint(long handle, String preset);
    static native String  nSignature(long handle, String preset);
    static native float   nFootprintDistance(long handleA, long handleB, String preset);
    static native boolean nIsDuplicateOf(long handleA, long handleB, String preset);

    // ── Diff ────────────────────────────────────────────────────────────────
    static native long    nDiff(long handleA, long handleB, String preset);
    // Registered with descriptor (JJLjava/lang/String;IIIILjava/lang/String;)J.
    // Negative cost ints mean "use the preset default"; null symmetry = unset.
    static native long    nDiffWithOverrides(
            long handleA, long handleB, String preset,
            int costAdd, int costDelete, int costChange, int costSwap,
            String symmetry);
    static native void    nDiffFree(long handle);
    static native long    nDiffDistance(long handle);
    static native float   nDiffSupport(long handle);
    static native String  nDiffToJson(long handle);
    static native String  nDiffSummaryJson(long handle);
    static native long    nDiffFromJson(String json);
    static native long    nDiffAdded(long handle);
    static native long    nDiffRemoved(long handle);
    static native long    nDiffChanged(long handle);
    static native long    nDiffSwapped(long handle);
    static native long    nDiffMarkers(long handle);

    // Meshing-gated: present only when the cdylib is built with the meshing
    // feature; calling without it raises UnsatisfiedLinkError.
    static native byte[]  nDiffToOverlayGlb(long handle, byte[] afterGlb);
}
