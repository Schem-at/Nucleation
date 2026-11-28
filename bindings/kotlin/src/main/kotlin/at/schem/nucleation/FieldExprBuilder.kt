package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldExprBuilderLib: Library {
    fun FieldExprBuilder_destroy(handle: Pointer)
    fun FieldExprBuilder_create(): Pointer
    fun FieldExprBuilder_bind(handle: Pointer, name: Slice, field: Pointer): ResultUnitInt
    fun FieldExprBuilder_set_bounds(handle: Pointer, minX: Float, minY: Float, minZ: Float, maxX: Float, maxY: Float, maxZ: Float): ResultUnitInt
    fun FieldExprBuilder_set_distance_kind(handle: Pointer, kind: Int): Unit
    fun FieldExprBuilder_compile(handle: Pointer, source: Slice): ResultPointerInt
}
/** Builder for compiling infix expressions that reference named external [Field]s.
*/
class FieldExprBuilder internal constructor (
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

    private class FieldExprBuilderCleaner(val handle: Pointer, val lib: FieldExprBuilderLib) : Runnable {
        override fun run() {
            lib.FieldExprBuilder_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, FieldExprBuilder.FieldExprBuilderCleaner(handle, FieldExprBuilder.lib));
    }

    companion object {
        internal val libClass: Class<FieldExprBuilderLib> = FieldExprBuilderLib::class.java
        internal val lib: FieldExprBuilderLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun create(): FieldExprBuilder {

            val returnVal = lib.FieldExprBuilder_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = FieldExprBuilder(handle, selfEdges, true)
            return returnOpaque
        }
    }

    fun bind(name: String, field: Field): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

        val returnVal = lib.FieldExprBuilder_bind(handle, nameSliceMemory.slice, field.handle);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }

    fun setBounds(minX: Float, minY: Float, minZ: Float, maxX: Float, maxY: Float, maxZ: Float): Result<Unit> {

        val returnVal = lib.FieldExprBuilder_set_bounds(handle, minX, minY, minZ, maxX, maxY, maxZ);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun setDistanceKind(kind: FieldDistanceKind): Unit {

        val returnVal = lib.FieldExprBuilder_set_distance_kind(handle, kind.toNative());

    }

    fun compile(source: String): Result<Field> {
        val sourceSliceMemory = PrimitiveArrayTools.borrowUtf8(source)

        val returnVal = lib.FieldExprBuilder_compile(handle, sourceSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Field(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            sourceSliceMemory.close()
        }
    }

}
