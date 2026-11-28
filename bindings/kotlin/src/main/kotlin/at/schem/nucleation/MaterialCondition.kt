package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MaterialConditionLib: Library {
    fun MaterialCondition_destroy(handle: Pointer)
    fun MaterialCondition_always(): Pointer
    fun MaterialCondition_and_with(handle: Pointer, other: Pointer): Pointer
    fun MaterialCondition_or_with(handle: Pointer, other: Pointer): Pointer
    fun MaterialCondition_negated(handle: Pointer): Pointer
    fun MaterialCondition_tag(materialId: FFIUint16): Pointer
    fun MaterialCondition_channel_range(channel: Int, hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): ResultPointerInt
    fun MaterialCondition_field_range(field: Pointer, hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): ResultPointerInt
    fun MaterialCondition_y_range(hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): ResultPointerInt
    fun MaterialCondition_depth_below_surface(hasMin: Boolean, min: Int, hasMax: Boolean, max: Int): Pointer
    fun MaterialCondition_slope_range(hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): ResultPointerInt
    fun MaterialCondition_curvature_range(hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): ResultPointerInt
    fun MaterialCondition_normal_dot(dx: Float, dy: Float, dz: Float, hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): ResultPointerInt
    fun MaterialCondition_exposed_face(mask: FFIUint8, requireAll: Boolean): Pointer
    fun MaterialCondition_chance(probability: Float, seed: Int): ResultPointerInt
}
/** Composable voxel predicate for [Material] layers and [ScatterRule]s.
*/
class MaterialCondition internal constructor (
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

    private class MaterialConditionCleaner(val handle: Pointer, val lib: MaterialConditionLib) : Runnable {
        override fun run() {
            lib.MaterialCondition_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, MaterialCondition.MaterialConditionCleaner(handle, MaterialCondition.lib));
    }

    companion object {
        internal val libClass: Class<MaterialConditionLib> = MaterialConditionLib::class.java
        internal val lib: MaterialConditionLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun always(): MaterialCondition {

            val returnVal = lib.MaterialCondition_always();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = MaterialCondition(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun tag(materialId: UShort): MaterialCondition {

            val returnVal = lib.MaterialCondition_tag(FFIUint16(materialId));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = MaterialCondition(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun channelRange(channel: FieldChannel, hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): Result<MaterialCondition> {

            val returnVal = lib.MaterialCondition_channel_range(channel.toNative(), hasMin, min, hasMax, max);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = MaterialCondition(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun fieldRange(field: Field, hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): Result<MaterialCondition> {

            val returnVal = lib.MaterialCondition_field_range(field.handle, hasMin, min, hasMax, max);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = MaterialCondition(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun yRange(hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): Result<MaterialCondition> {

            val returnVal = lib.MaterialCondition_y_range(hasMin, min, hasMax, max);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = MaterialCondition(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun depthBelowSurface(hasMin: Boolean, min: Int, hasMax: Boolean, max: Int): MaterialCondition {

            val returnVal = lib.MaterialCondition_depth_below_surface(hasMin, min, hasMax, max);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = MaterialCondition(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun slopeRange(hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): Result<MaterialCondition> {

            val returnVal = lib.MaterialCondition_slope_range(hasMin, min, hasMax, max);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = MaterialCondition(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun curvatureRange(hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): Result<MaterialCondition> {

            val returnVal = lib.MaterialCondition_curvature_range(hasMin, min, hasMax, max);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = MaterialCondition(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun normalDot(dx: Float, dy: Float, dz: Float, hasMin: Boolean, min: Float, hasMax: Boolean, max: Float): Result<MaterialCondition> {

            val returnVal = lib.MaterialCondition_normal_dot(dx, dy, dz, hasMin, min, hasMax, max);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = MaterialCondition(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        fun exposedFace(mask: UByte, requireAll: Boolean): MaterialCondition {

            val returnVal = lib.MaterialCondition_exposed_face(FFIUint8(mask), requireAll);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = MaterialCondition(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic

        fun chance(probability: Float, seed: Int): Result<MaterialCondition> {

            val returnVal = lib.MaterialCondition_chance(probability, seed);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = MaterialCondition(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }

    fun andWith(other: MaterialCondition): MaterialCondition {

        val returnVal = lib.MaterialCondition_and_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = MaterialCondition(handle, selfEdges, true)
        return returnOpaque
    }

    fun orWith(other: MaterialCondition): MaterialCondition {

        val returnVal = lib.MaterialCondition_or_with(handle, other.handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = MaterialCondition(handle, selfEdges, true)
        return returnOpaque
    }

    fun negated(): MaterialCondition {

        val returnVal = lib.MaterialCondition_negated(handle);
        val selfEdges: List<Any> = listOf()
        val handle = returnVal
        val returnOpaque = MaterialCondition(handle, selfEdges, true)
        return returnOpaque
    }

}
