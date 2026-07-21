package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface Curve3DLib: Library {
    fun Curve3D_destroy(handle: Pointer)
    fun Curve3D_from_points(coordinates: Slice, closed: Boolean): ResultPointerInt
    fun Curve3D_point_count(handle: Pointer): FFIUint32
    fun Curve3D_is_closed(handle: Pointer): Byte
}
/** A sampled 3D polyline. Closed curves include the final segment back to
*the first point and retain arc-length parameterisation for animation.
*/
class Curve3D internal constructor (
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

    private class Curve3DCleaner(val handle: Pointer, val lib: Curve3DLib) : Runnable {
        override fun run() {
            lib.Curve3D_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Curve3D.Curve3DCleaner(handle, Curve3D.lib));
    }

    companion object {
        internal val libClass: Class<Curve3DLib> = Curve3DLib::class.java
        internal val lib: Curve3DLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Create a curve from flat `[x0, y0, z0, x1, y1, z1, …]` coordinates.
        */
        fun fromPoints(coordinates: DoubleArray, closed: Boolean): Result<Curve3D> {
            val coordinatesSliceMemory = PrimitiveArrayTools.borrow(coordinates)
            
            val returnVal = lib.Curve3D_from_points(coordinatesSliceMemory.slice, closed);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Curve3D(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                coordinatesSliceMemory.close()
            }
        }
    }
    
    fun pointCount(): UInt {
        
        val returnVal = lib.Curve3D_point_count(handle);
        return (returnVal.toUInt())
    }
    
    fun isClosed(): Boolean {
        
        val returnVal = lib.Curve3D_is_closed(handle);
        return (returnVal > 0)
    }

}