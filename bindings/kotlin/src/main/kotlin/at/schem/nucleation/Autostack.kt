package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface AutostackLib: Library {
    fun Autostack_destroy(handle: Pointer)
    fun Autostack_detect_structures(schematic: Pointer, write: Pointer): Unit
    fun Autostack_detect_structures_graph(schematic: Pointer, write: Pointer): Unit
    fun Autostack_resize_1d(schematic: Pointer, vx: Int, vy: Int, vz: Int, units: FFIUint32): ResultPointerInt
    fun Autostack_resize_2d(schematic: Pointer, v1x: Int, v1y: Int, v1z: Int, v2x: Int, v2y: Int, v2z: Int, n1: FFIUint32, n2: FFIUint32): ResultPointerInt
}
/** Free functions in the old ABI hung directly off `schematic_*`; here they get a
*namespacing opaque-less home via static methods on a zero-size type is not
*supported, so they live as static methods taking `&Schematic` explicitly.
*/
class Autostack internal constructor (
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

    private class AutostackCleaner(val handle: Pointer, val lib: AutostackLib) : Runnable {
        override fun run() {
            lib.Autostack_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Autostack.AutostackCleaner(handle, Autostack.lib));
    }

    companion object {
        internal val libClass: Class<AutostackLib> = AutostackLib::class.java
        internal val lib: AutostackLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Detect repeating structures (region coverage). Writes a JSON array string;
        *each element has `mode`, `vectors`, `coverage`, `region_min`/`region_max`,
        *`cell_min`/`cell_max`, `label`.
        */
        fun detectStructures(schematic: Schematic): String {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Autostack_detect_structures(schematic.handle, write);
            
            val returnString = DW.writeToString(write)
            return returnString
        }
        @JvmStatic
        
        /** Graph-based detection: recovers diagonal lattice periods via the redstone
        *logic graph. Writes `"[]"` for non-redstone builds. Requires the
        *`simulation` feature; writes `"[]"` when compiled without it.
        */
        fun detectStructuresGraph(schematic: Schematic): String {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Autostack_detect_structures_graph(schematic.handle, write);
            
            val returnString = DW.writeToString(write)
            return returnString
        }
        @JvmStatic
        
        /** Resize a 1D / diagonal structure along its period vector.
        */
        fun resize1d(schematic: Schematic, vx: Int, vy: Int, vz: Int, units: UInt): Result<Schematic> {
            
            val returnVal = lib.Autostack_resize_1d(schematic.handle, vx, vy, vz, FFIUint32(units));
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = Schematic(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** Resize a 2D structure to `n1`×`n2` cells along the two period vectors.
        */
        fun resize2d(schematic: Schematic, v1x: Int, v1y: Int, v1z: Int, v2x: Int, v2y: Int, v2z: Int, n1: UInt, n2: UInt): Result<Schematic> {
            
            val returnVal = lib.Autostack_resize_2d(schematic.handle, v1x, v1y, v1z, v2x, v2y, v2z, FFIUint32(n1), FFIUint32(n2));
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = Schematic(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }

}