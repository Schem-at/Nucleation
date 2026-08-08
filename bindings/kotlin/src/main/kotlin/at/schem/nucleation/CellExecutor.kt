package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface CellExecutorLib: Library {
    fun CellExecutor_destroy(handle: Pointer)
    fun CellExecutor_for_schematic(schematic: Pointer): ResultPointerInt
    fun CellExecutor_set_input(handle: Pointer, name: Slice, value: Pointer): ResultUnitInt
    fun CellExecutor_settle(handle: Pointer, budget: FFIUint32): Byte
    fun CellExecutor_read_output(handle: Pointer, name: Slice): ResultPointerInt
    fun CellExecutor_reset(handle: Pointer): ResultUnitInt
}
/** A typed executor bound to a self-describing cell: the schematic's
*EMBEDDED contract (autodetected, Insign fallback) supplies the port
*names, types and positions; the vanilla-accurate mc-tick engine
*supplies the physics. Wraps
*[crate::simulation::typed_executor::BackendCircuitExecutor].
*/
class CellExecutor internal constructor (
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

    private class CellExecutorCleaner(val handle: Pointer, val lib: CellExecutorLib) : Runnable {
        override fun run() {
            lib.CellExecutor_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, CellExecutor.CellExecutorCleaner(handle, CellExecutor.lib));
    }

    companion object {
        internal val libClass: Class<CellExecutorLib> = CellExecutorLib::class.java
        internal val lib: CellExecutorLib = Native.load("nucleation", libClass)
        @JvmStatic

        /** Bind the schematic's embedded cell contract to the mc-tick
        *engine (needs the `mc-tick` feature, else errors). Cells deploy
        *BAKED: the backend trusts saved block states; an unbaked build
        *sits inert until the first input flip.
        */
        fun forSchematic(schematic: Schematic): Result<CellExecutor> {

            val returnVal = lib.CellExecutor_for_schematic(schematic.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = CellExecutor(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }

    /** Set an input port by name and typed value.
    */
    fun setInput(name: String, value: Value): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

        val returnVal = lib.CellExecutor_set_input(handle, nameSliceMemory.slice, value.handle);
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

    /** Run until quiescent within `budget` ticks; true when settled.
    */
    fun settle(budget: UInt): Boolean {

        val returnVal = lib.CellExecutor_settle(handle, FFIUint32(budget));
        return (returnVal > 0)
    }

    /** Read an output port by name.
    */
    fun readOutput(name: String): Result<Value> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

        val returnVal = lib.CellExecutor_read_output(handle, nameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Value(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }

    /** Rebuild the engine from the original schematic (all inputs back
    *to their saved states).
    */
    fun reset(): Result<Unit> {

        val returnVal = lib.CellExecutor_reset(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
