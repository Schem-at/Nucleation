package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface WorldGeneratorLib: Library {
    fun WorldGenerator_destroy(handle: Pointer)
    fun WorldGenerator_sdf(volume: Pointer, material: Pointer, minY: Int, maxY: Int, sourceId: Slice, version: Slice): ResultPointerInt
    fun WorldGenerator_cellular_sdf(volume: Pointer, material: Pointer, minY: Int, maxY: Int, config: Pointer, sourceId: Slice, version: Slice): ResultPointerInt
    fun WorldGenerator_projected_footprints(buildingsJson: Slice, baseBlock: Slice, sourceId: Slice, version: Slice): ResultPointerInt
    fun WorldGenerator_composite(sourceId: Slice, version: Slice): ResultPointerInt
    fun WorldGenerator_add_layer(handle: Pointer, source: Pointer, mode: Int): ResultUnitInt
    fun WorldGenerator_generate(handle: Pointer, cx: Int, cz: Int): ResultPointerInt
    fun WorldGenerator_stream(handle: Pointer, minCx: Int, minCz: Int, maxCx: Int, maxCz: Int): ResultPointerInt
}
/** An immutable native chunk source graph.
*
*Generated bindings expose concrete source constructors rather than host
*callbacks, so SDF evaluation and block placement stay entirely in Rust.
*/
class WorldGenerator internal constructor (
    internal val handle: Pointer,
    // These ensure that anything that is borrowed is kept alive and not cleaned
    // up by the garbage collector.
    internal val selfEdges: List<Any>,
    internal var owned: Boolean,
)  {

    init {
        if (this.owned) {
            this.registerCleaner()
        }
    }

    private class WorldGeneratorCleaner(val handle: Pointer, val lib: WorldGeneratorLib) : Runnable {
        override fun run() {
            lib.WorldGenerator_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, WorldGenerator.WorldGeneratorCleaner(handle, WorldGenerator.lib));
    }

    companion object {
        internal val libClass: Class<WorldGeneratorLib> = WorldGeneratorLib::class.java
        internal val lib: WorldGeneratorLib = Native.load("nucleation", libClass)
        @JvmStatic

        /** Create an SDF-backed source evaluated at voxel centers over the inclusive
        *Y range. `source_id` and `version` become chunk provenance/cache metadata.
        */
        fun sdf(volume: Sdf, material: Brush, minY: Int, maxY: Int, sourceId: String, version: String): Result<WorldGenerator> {
            val sourceIdSliceMemory = PrimitiveArrayTools.borrowUtf8(sourceId)
            val versionSliceMemory = PrimitiveArrayTools.borrowUtf8(version)

            val returnVal = lib.WorldGenerator_sdf(volume.handle, material.handle, minY, maxY, sourceIdSliceMemory.slice, versionSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = WorldGenerator(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sourceIdSliceMemory.close()
                versionSliceMemory.close()
            }
        }
        @JvmStatic

        /** Create a sparse infinite source by placing a bounded SDF motif once per
        *deterministically transformed cell. Reuse `config` across layers to keep
        *terrain, water, vegetation, paths, and structures coordinated.
        */
        fun cellularSdf(volume: Sdf, material: Brush, minY: Int, maxY: Int, config: CellularSdfConfig, sourceId: String, version: String): Result<WorldGenerator> {
            val sourceIdSliceMemory = PrimitiveArrayTools.borrowUtf8(sourceId)
            val versionSliceMemory = PrimitiveArrayTools.borrowUtf8(version)

            val returnVal = lib.WorldGenerator_cellular_sdf(volume.handle, material.handle, minY, maxY, config.handle, sourceIdSliceMemory.slice, versionSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = WorldGenerator(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sourceIdSliceMemory.close()
                versionSliceMemory.close()
            }
        }
        @JvmStatic

        /** Create a sparse source from projected building footprints, including
        *caller-projected OSM-derived data.
        *`buildings_json` uses the same schema as `Geo.extrude_footprints`:
        *`[{"polygon":[[x,z],...],"height":40,"min_y":1,
        *"block":"minecraft:bricks"}]`. `height` is the absolute top Y, matching
        *`Geo.extrude_footprints`. Fetching and lat/lon projection stay
        *caller-controlled; this source rasterizes only requested chunks.
        */
        fun projectedFootprints(buildingsJson: String, baseBlock: String, sourceId: String, version: String): Result<WorldGenerator> {
            val buildingsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(buildingsJson)
            val baseBlockSliceMemory = PrimitiveArrayTools.borrowUtf8(baseBlock)
            val sourceIdSliceMemory = PrimitiveArrayTools.borrowUtf8(sourceId)
            val versionSliceMemory = PrimitiveArrayTools.borrowUtf8(version)

            val returnVal = lib.WorldGenerator_projected_footprints(buildingsJsonSliceMemory.slice, baseBlockSliceMemory.slice, sourceIdSliceMemory.slice, versionSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = WorldGenerator(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                buildingsJsonSliceMemory.close()
                baseBlockSliceMemory.close()
                sourceIdSliceMemory.close()
                versionSliceMemory.close()
            }
        }
        @JvmStatic

        /** Create an initially empty ordered source composition.
        */
        fun composite(sourceId: String, version: String): Result<WorldGenerator> {
            val sourceIdSliceMemory = PrimitiveArrayTools.borrowUtf8(sourceId)
            val versionSliceMemory = PrimitiveArrayTools.borrowUtf8(version)

            val returnVal = lib.WorldGenerator_composite(sourceIdSliceMemory.slice, versionSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = WorldGenerator(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sourceIdSliceMemory.close()
                versionSliceMemory.close()
            }
        }
    }

    /** Append a source to a composite. Later `Replace` layers win at occupied
    *voxels; `KeepExisting` layers only fill air. Errors on non-composites.
    *
    *Streams already created from this generator keep the layer list they
    *were built with; only later `generate`/`stream` calls see the addition.
    */
    fun addLayer(source: WorldGenerator, mode: GeneratedChunkOverlayMode): Result<Unit> {

        val returnVal = lib.WorldGenerator_add_layer(handle, source.handle, mode.toNative());
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Generate one random-access chunk.
    */
    fun generate(cx: Int, cz: Int): Result<GeneratedChunk> {

        val returnVal = lib.WorldGenerator_generate(handle, cx, cz);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = GeneratedChunk(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Traverse an inclusive chunk rectangle lazily in canonical region-major
    *order. The stream snapshots the generator's sources at creation, so
    *later `add_layer` calls do not affect a stream already in flight.
    */
    fun stream(minCx: Int, minCz: Int, maxCx: Int, maxCz: Int): Result<GeneratedWorldStream> {

        val returnVal = lib.WorldGenerator_stream(handle, minCx, minCz, maxCx, maxCz);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = GeneratedWorldStream(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
