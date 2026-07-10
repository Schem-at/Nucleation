package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SdfLib: Library {
    fun Sdf_destroy(handle: Pointer)
    fun Sdf_schematic_from_sdf(sdfJson: Slice, rulesJson: Slice, hasBounds: Boolean, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun Sdf_eval(sdfJson: Slice, x: Float, y: Float, z: Float): ResultFloatInt
}
/** Namespace for the SDF free functions of the old ABI (`schematic_from_sdf`,
*`sdf_eval`).
*/
class Sdf internal constructor (
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

    private class SdfCleaner(val handle: Pointer, val lib: SdfLib) : Runnable {
        override fun run() {
            lib.Sdf_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Sdf.SdfCleaner(handle, Sdf.lib));
    }

    companion object {
        internal val libClass: Class<SdfLib> = SdfLib::class.java
        internal val lib: SdfLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Builds a schematic by sampling an SDF JSON tree with material rules JSON.
        *When `has_bounds` is false the tree's own AABB is used (fails with
        *`InvalidArgument` for unbounded trees) and the `min_*`/`max_*` arguments
        *are ignored.
        */
        fun schematicFromSdf(sdfJson: String, rulesJson: String, hasBounds: Boolean, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Schematic> {
            val sdfJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(sdfJson)
            val rulesJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(rulesJson)
            
            val returnVal = lib.Sdf_schematic_from_sdf(sdfJsonSliceMemory.slice, rulesJsonSliceMemory.slice, hasBounds, minX, minY, minZ, maxX, maxY, maxZ);
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
                sdfJsonSliceMemory.close()
                rulesJsonSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Evaluates an SDF JSON tree at a point, returning the signed distance.
        */
        fun eval(sdfJson: String, x: Float, y: Float, z: Float): Result<Float> {
            val sdfJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(sdfJson)
            
            val returnVal = lib.Sdf_eval(sdfJsonSliceMemory.slice, x, y, z);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return (nativeOkVal).ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sdfJsonSliceMemory.close()
            }
        }
    }

}