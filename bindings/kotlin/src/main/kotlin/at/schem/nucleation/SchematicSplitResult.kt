package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SchematicSplitResultLib: Library {
    fun SchematicSplitResult_destroy(handle: Pointer)
    fun SchematicSplitResult_len(handle: Pointer): FFIUint32
    fun SchematicSplitResult_piece(handle: Pointer, index: FFIUint32): ResultPointerInt
}
/** Deterministic, lossless pieces returned by
*[Schematic::split_connected_attach_nearby]. Pieces are ordered by their
*largest connected component, largest first.
*/
class SchematicSplitResult internal constructor (
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

    private class SchematicSplitResultCleaner(val handle: Pointer, val lib: SchematicSplitResultLib) : Runnable {
        override fun run() {
            lib.SchematicSplitResult_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, SchematicSplitResult.SchematicSplitResultCleaner(handle, SchematicSplitResult.lib));
    }

    companion object {
        internal val libClass: Class<SchematicSplitResultLib> = SchematicSplitResultLib::class.java
        internal val lib: SchematicSplitResultLib = Native.load("nucleation", libClass)
    }

    fun len(): UInt {

        val returnVal = lib.SchematicSplitResult_len(handle);
        return (returnVal.toUInt())
    }

    /** Return an independently owned piece by zero-based index.
    */
    fun piece(index: UInt): Result<Schematic> {

        val returnVal = lib.SchematicSplitResult_piece(handle, FFIUint32(index));
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
