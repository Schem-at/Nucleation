package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldProgramLib: Library {
    fun FieldProgram_destroy(handle: Pointer)
    fun FieldProgram_from_json_string(json: Slice): ResultPointerInt
    fun FieldProgram_to_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun FieldProgram_eval_at(handle: Pointer, x: Float, y: Float, z: Float): Float
    fun FieldProgram_gradient(handle: Pointer, x: Float, y: Float, z: Float, epsilon: Float): ResultSdfNormalNativeInt
    fun FieldProgram_bounds(handle: Pointer): SdfBoundsNative
    fun FieldProgram_distance_kind(handle: Pointer): Int
}
/** A validated, sandboxed custom SDF field program: deterministic typed
*bytecode over scalar/vec3/bool values with bounded loops, carrying
*its own explicit finite bounds and distance-kind metadata. Build one
*with [FieldProgramBuilder] or import it from JSON.
*/
class FieldProgram internal constructor (
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

    private class FieldProgramCleaner(val handle: Pointer, val lib: FieldProgramLib) : Runnable {
        override fun run() {
            lib.FieldProgram_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, FieldProgram.FieldProgramCleaner(handle, FieldProgram.lib));
    }

    companion object {
        internal val libClass: Class<FieldProgramLib> = FieldProgramLib::class.java
        internal val lib: FieldProgramLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun fromJsonString(json: String): Result<FieldProgram> {
            val jsonSliceMemory = PrimitiveArrayTools.borrowUtf8(json)

            val returnVal = lib.FieldProgram_from_json_string(jsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = FieldProgram(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                jsonSliceMemory.close()
            }
        }
    }

    fun toJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.FieldProgram_to_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun evalAt(x: Float, y: Float, z: Float): Float {

        val returnVal = lib.FieldProgram_eval_at(handle, x, y, z);
        return (returnVal)
    }

    /** Unit-length gradient of the scalar output at `(x, y, z)`: the
    *program's own forward-mode analytic gradient where it's
    *differentiable there, falling back to a numerical estimate
    *(central differences via `epsilon`) otherwise.
    */
    fun gradient(x: Float, y: Float, z: Float, epsilon: Float): Result<SdfNormal> {

        val returnVal = lib.FieldProgram_gradient(handle, x, y, z, epsilon);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = SdfNormal.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun bounds(): SdfBounds {

        val returnVal = lib.FieldProgram_bounds(handle);
        val returnStruct = SdfBounds.fromNative(returnVal)
        return returnStruct
    }

    fun distanceKind(): FieldProgramDistanceKind {

        val returnVal = lib.FieldProgram_distance_kind(handle);
        return (FieldProgramDistanceKind.fromNative(returnVal))
    }

}
