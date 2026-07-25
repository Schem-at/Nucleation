package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface Field3Lib: Library {
    fun Field3_destroy(handle: Pointer)
    fun Field3_value_noise_fbm(frequency: Float, seed: Int, octaves: FFIUint32): ResultPointerInt
    fun Field3_eval_at(handle: Pointer, x: Float, y: Float, z: Float): Float
    fun Field3_output_range(handle: Pointer): ResultFieldRangeNativeInt
    fun Field3_from_json_string(json: Slice): ResultPointerInt
    fun Field3_to_json(handle: Pointer, write: Pointer): ResultUnitInt
}
/** Immutable scalar field evaluated over world-space `(x, y, z)`.
*
*A `Field3` has scalar semantics only. It may be shared by geometry and
*material consumers without being reinterpreted as a signed-distance field.
*/
class Field3 internal constructor (
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

    private class Field3Cleaner(val handle: Pointer, val lib: Field3Lib) : Runnable {
        override fun run() {
            lib.Field3_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Field3.Field3Cleaner(handle, Field3.lib));
    }

    companion object {
        internal val libClass: Class<Field3Lib> = Field3Lib::class.java
        internal val lib: Field3Lib = Native.load("nucleation", libClass)
        @JvmStatic

        /** Deterministic value-noise FBM normalized to `[-1, 1]`.
        */
        fun valueNoiseFbm(frequency: Float, seed: Int, octaves: UInt): Result<Field3> {

            val returnVal = lib.Field3_value_noise_fbm(frequency, seed, FFIUint32(octaves));
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field3(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun fromJsonString(json: String): Result<Field3> {
            val jsonSliceMemory = PrimitiveArrayTools.borrowUtf8(json)

            val returnVal = lib.Field3_from_json_string(jsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = Field3(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                jsonSliceMemory.close()
            }
        }
    }

    fun evalAt(x: Float, y: Float, z: Float): Float {

        val returnVal = lib.Field3_eval_at(handle, x, y, z);
        return (returnVal)
    }

    /** The field's analytically proven output range.
    *
    *Returns `NotFound` when no range can be proven — callers mapping a
    *field onto a gradient must handle that rather than silently
    *propagating a sentinel into their `lo`/`hi` bounds.
    */
    fun outputRange(): Result<FieldRange> {

        val returnVal = lib.Field3_output_range(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = FieldRange.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun toJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Field3_to_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
