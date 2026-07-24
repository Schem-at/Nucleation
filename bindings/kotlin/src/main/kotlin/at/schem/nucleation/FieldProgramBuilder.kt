package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldProgramBuilderLib: Library {
    fun FieldProgramBuilder_destroy(handle: Pointer)
    fun FieldProgramBuilder_create(): Pointer
    fun FieldProgramBuilder_add_slot(handle: Pointer, valueType: Int): ResultFFIUint16Int
    fun FieldProgramBuilder_push_const_scalar(handle: Pointer, value: Float): ResultUnitInt
    fun FieldProgramBuilder_push_const_vec3(handle: Pointer, x: Float, y: Float, z: Float): ResultUnitInt
    fun FieldProgramBuilder_push_const_bool(handle: Pointer, value: Boolean): ResultUnitInt
    fun FieldProgramBuilder_push_pos(handle: Pointer): ResultUnitInt
    fun FieldProgramBuilder_load_local(handle: Pointer, slot: FFIUint16): ResultUnitInt
    fun FieldProgramBuilder_store_local(handle: Pointer, slot: FFIUint16): ResultUnitInt
    fun FieldProgramBuilder_pop(handle: Pointer): ResultUnitInt
    fun FieldProgramBuilder_unary_op(handle: Pointer, op: Int): ResultUnitInt
    fun FieldProgramBuilder_binary_op(handle: Pointer, op: Int): ResultUnitInt
    fun FieldProgramBuilder_clamp(handle: Pointer): ResultUnitInt
    fun FieldProgramBuilder_select(handle: Pointer): ResultUnitInt
    fun FieldProgramBuilder_make_vec3(handle: Pointer): ResultUnitInt
    fun FieldProgramBuilder_break_if(handle: Pointer): ResultUnitInt
    fun FieldProgramBuilder_begin_repeat(handle: Pointer, count: FFIUint32): ResultUnitInt
    fun FieldProgramBuilder_end_repeat(handle: Pointer): ResultUnitInt
    fun FieldProgramBuilder_set_output(handle: Pointer, slot: FFIUint16): ResultUnitInt
    fun FieldProgramBuilder_set_bounds(handle: Pointer, minX: Float, minY: Float, minZ: Float, maxX: Float, maxY: Float, maxZ: Float): ResultUnitInt
    fun FieldProgramBuilder_set_distance_kind(handle: Pointer, kind: Int): ResultUnitInt
    fun FieldProgramBuilder_build(handle: Pointer): ResultPointerInt
}
/** Programmatic builder for a [FieldProgram]: append typed stack
*instructions, then [FieldProgramBuilder::build] to validate and
*obtain a [FieldProgram]. Consuming: every method after `build()`
*(successful or not) returns `AlreadyConsumed`.
*/
class FieldProgramBuilder internal constructor (
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

    private class FieldProgramBuilderCleaner(val handle: Pointer, val lib: FieldProgramBuilderLib) : Runnable {
        override fun run() {
            lib.FieldProgramBuilder_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, FieldProgramBuilder.FieldProgramBuilderCleaner(handle, FieldProgramBuilder.lib));
    }

    companion object {
        internal val libClass: Class<FieldProgramBuilderLib> = FieldProgramBuilderLib::class.java
        internal val lib: FieldProgramBuilderLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun create(): FieldProgramBuilder {

            val returnVal = lib.FieldProgramBuilder_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = FieldProgramBuilder(handle, selfEdges, true)
            return returnOpaque
        }
    }

    /** Declare a new typed local slot and return its index.
    */
    fun addSlot(valueType: FieldProgramValueType): Result<UShort> {

        val returnVal = lib.FieldProgramBuilder_add_slot(handle, valueType.toNative());
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal.toUShort()).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun pushConstScalar(value: Float): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_push_const_scalar(handle, value);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun pushConstVec3(x: Float, y: Float, z: Float): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_push_const_vec3(handle, x, y, z);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun pushConstBool(value: Boolean): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_push_const_bool(handle, value);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Push the `Vec3` position the program is being evaluated at.
    */
    fun pushPos(): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_push_pos(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun loadLocal(slot: UShort): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_load_local(handle, FFIUint16(slot));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun storeLocal(slot: UShort): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_store_local(handle, FFIUint16(slot));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Discard the top of the stack.
    */
    fun pop(): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_pop(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun unaryOp(op: FieldProgramUnaryOp): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_unary_op(handle, op.toNative());
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun binaryOp(op: FieldProgramBinaryOp): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_binary_op(handle, op.toNative());
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Pop `(x, lo, hi)`, push `x` clamped to `[lo, hi]`.
    */
    fun clamp(): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_clamp(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Pop `(a, b, cond)`, push `a` if `cond` else `b`.
    */
    fun select(): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_select(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Pop `(x, y, z)`, push `Vec3([x, y, z])`.
    */
    fun makeVec3(): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_make_vec3(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Pop a `Bool`; if true, stop the nearest enclosing repeat after
    *this iteration. Only valid inside `beginRepeat`/`endRepeat`.
    */
    fun breakIf(): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_break_if(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Open a new statically bounded repeat block; subsequent
    *instructions append to its body until `endRepeat`.
    */
    fun beginRepeat(count: UInt): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_begin_repeat(handle, FFIUint32(count));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Close the innermost open repeat block.
    */
    fun endRepeat(): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_end_repeat(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Declare which scalar slot holds the program's output.
    */
    fun setOutput(slot: UShort): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_set_output(handle, FFIUint16(slot));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Set the program's explicit, author-asserted finite bounds.
    */
    fun setBounds(minX: Float, minY: Float, minZ: Float, maxX: Float, maxY: Float, maxZ: Float): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_set_bounds(handle, minX, minY, minZ, maxX, maxY, maxZ);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun setDistanceKind(kind: FieldProgramDistanceKind): Result<Unit> {

        val returnVal = lib.FieldProgramBuilder_set_distance_kind(handle, kind.toNative());
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Validate and finalize. Consumes the builder even on failure.
    */
    fun build(): Result<FieldProgram> {

        val returnVal = lib.FieldProgramBuilder_build(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = FieldProgram(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
