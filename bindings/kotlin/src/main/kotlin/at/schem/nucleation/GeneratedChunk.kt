package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface GeneratedChunkLib: Library {
    fun GeneratedChunk_destroy(handle: Pointer)
    fun GeneratedChunk_cx(handle: Pointer): ResultIntInt
    fun GeneratedChunk_cz(handle: Pointer): ResultIntInt
    fun GeneratedChunk_coverage(handle: Pointer): ResultIntInt
    fun GeneratedChunk_source_id(handle: Pointer, write: Pointer): ResultUnitInt
    fun GeneratedChunk_version(handle: Pointer, write: Pointer): ResultUnitInt
    fun GeneratedChunk_take_view(handle: Pointer): ResultPointerInt
}
/** One generated chunk plus coverage and source-version metadata.
*Call `take_view` once to move its chunk into the existing world-stream API.
*/
class GeneratedChunk internal constructor (
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

    private class GeneratedChunkCleaner(val handle: Pointer, val lib: GeneratedChunkLib) : Runnable {
        override fun run() {
            lib.GeneratedChunk_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, GeneratedChunk.GeneratedChunkCleaner(handle, GeneratedChunk.lib));
    }

    companion object {
        internal val libClass: Class<GeneratedChunkLib> = GeneratedChunkLib::class.java
        internal val lib: GeneratedChunkLib = Native.load("nucleation", libClass)
    }

    fun cx(): Result<Int> {

        val returnVal = lib.GeneratedChunk_cx(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun cz(): Result<Int> {

        val returnVal = lib.GeneratedChunk_cz(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun coverage(): Result<GeneratedChunkCoverage> {

        val returnVal = lib.GeneratedChunk_coverage(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (GeneratedChunkCoverage.fromNative(nativeOkVal)).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun sourceId(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.GeneratedChunk_source_id(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun version(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.GeneratedChunk_version(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Consume the generated chunk payload. Metadata access and a second call
    *return `AlreadyConsumed` afterwards.
    */
    fun takeView(): Result<WorldChunkView> {

        val returnVal = lib.GeneratedChunk_take_view(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = WorldChunkView(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
