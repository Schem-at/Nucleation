package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ScatterRuleLib: Library {
    fun ScatterRule_destroy(handle: Pointer)
    fun ScatterRule_create(face: Int, density: Float, blocksJson: Slice, weights: Slice, seed: Int): ResultPointerInt
    fun ScatterRule_set_condition(handle: Pointer, condition: Pointer): Unit
    fun ScatterRule_set_on_block(handle: Pointer, prefix: Slice): ResultUnitInt
    fun ScatterRule_set_height_range(handle: Pointer, min: FFIUint32, max: FFIUint32): ResultUnitInt
}
/** Surface scatter rule attached to a [VoxelPipeline].
*/
class ScatterRule internal constructor (
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

    private class ScatterRuleCleaner(val handle: Pointer, val lib: ScatterRuleLib) : Runnable {
        override fun run() {
            lib.ScatterRule_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, ScatterRule.ScatterRuleCleaner(handle, ScatterRule.lib));
    }

    companion object {
        internal val libClass: Class<ScatterRuleLib> = ScatterRuleLib::class.java
        internal val lib: ScatterRuleLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun create(face: ScatterFace, density: Float, blocksJson: String, weights: FloatArray, seed: Int): Result<ScatterRule> {
            val blocksJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(blocksJson)
            val weightsSliceMemory = PrimitiveArrayTools.borrow(weights)

            val returnVal = lib.ScatterRule_create(face.toNative(), density, blocksJsonSliceMemory.slice, weightsSliceMemory.slice, seed);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = ScatterRule(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                blocksJsonSliceMemory.close()
                weightsSliceMemory.close()
            }
        }
    }

    fun setCondition(condition: MaterialCondition): Unit {

        val returnVal = lib.ScatterRule_set_condition(handle, condition.handle);

    }

    fun setOnBlock(prefix: String): Result<Unit> {
        val prefixSliceMemory = PrimitiveArrayTools.borrowUtf8(prefix)

        val returnVal = lib.ScatterRule_set_on_block(handle, prefixSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            prefixSliceMemory.close()
        }
    }

    fun setHeightRange(min: UInt, max: UInt): Result<Unit> {

        val returnVal = lib.ScatterRule_set_height_range(handle, FFIUint32(min), FFIUint32(max));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
