package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface GeoLib: Library {
    fun Geo_destroy(handle: Pointer)
    fun Geo_extrude_footprints(buildingsJson: Slice, baseBlock: Slice, name: Slice): ResultPointerInt
    fun Geo_heightmap_terrain(heightsJson: Slice, width: Int, surfaceBlocksJson: Slice, subsurfaceBlock: Slice, surfaceDepth: Int, name: Slice): ResultPointerInt
}
/** Namespace for the geodata entry points (no network — data goes in,
*blocks come out).
*/
class Geo internal constructor (
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

    private class GeoCleaner(val handle: Pointer, val lib: GeoLib) : Runnable {
        override fun run() {
            lib.Geo_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Geo.GeoCleaner(handle, Geo.lib));
    }

    companion object {
        internal val libClass: Class<GeoLib> = GeoLib::class.java
        internal val lib: GeoLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Extrude building footprints into a massed schematic. `buildings_json`
        *is a JSON array of objects:
        *`{"polygon": [[x, z], ...], "height": <blocks>, "block": "minecraft:...",
        *"min_y": <optional base, default 1>}`. Footprints are stamped
        *tallest-last, so overlaps keep the tallest occupant per column.
        *`base_block` (empty string = none) lays a ground slab at y=0 under the
        *whole extent. Errors `Parse` on bad JSON, `InvalidArgument` on non-UTF-8.
        */
        fun extrudeFootprints(buildingsJson: String, baseBlock: String, name: String): Result<Schematic> {
            val buildingsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(buildingsJson)
            val baseBlockSliceMemory = PrimitiveArrayTools.borrowUtf8(baseBlock)
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.Geo_extrude_footprints(buildingsJsonSliceMemory.slice, baseBlockSliceMemory.slice, nameSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                buildingsJsonSliceMemory.close()
                baseBlockSliceMemory.close()
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Raise terrain from a heightmap. `heights_json` is a flat row-major
        *JSON array of per-column heights (blocks); `width` is columns per row.
        *`surface_blocks_json` is a JSON array of block names — one entry (the
        *same surface everywhere) or one per column, row-major and the same
        *length as `heights`, for elevation/slope banding. Each column's top
        *`surface_depth` blocks are its surface block, the rest are
        *`subsurface_block`. Errors `Parse` on bad JSON, `InvalidArgument` on a
        *non-positive width, empty surface list, or non-UTF-8.
        */
        fun heightmapTerrain(heightsJson: String, width: Int, surfaceBlocksJson: String, subsurfaceBlock: String, surfaceDepth: Int, name: String): Result<Schematic> {
            val heightsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(heightsJson)
            val surfaceBlocksJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(surfaceBlocksJson)
            val subsurfaceBlockSliceMemory = PrimitiveArrayTools.borrowUtf8(subsurfaceBlock)
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.Geo_heightmap_terrain(heightsJsonSliceMemory.slice, width, surfaceBlocksJsonSliceMemory.slice, subsurfaceBlockSliceMemory.slice, surfaceDepth, nameSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                heightsJsonSliceMemory.close()
                surfaceBlocksJsonSliceMemory.close()
                subsurfaceBlockSliceMemory.close()
                nameSliceMemory.close()
            }
        }
    }

}