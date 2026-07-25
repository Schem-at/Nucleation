package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface GeneratedWorldStreamLib: Library {
    fun GeneratedWorldStream_destroy(handle: Pointer)
    fun GeneratedWorldStream_remaining(handle: Pointer): FFIUint64
    fun GeneratedWorldStream_next(handle: Pointer): ResultPointerInt
}
/** A finite, lazy, canonical region-major traversal of a generator.
*/
class GeneratedWorldStream internal constructor (
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

    private class GeneratedWorldStreamCleaner(val handle: Pointer, val lib: GeneratedWorldStreamLib) : Runnable {
        override fun run() {
            lib.GeneratedWorldStream_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, GeneratedWorldStream.GeneratedWorldStreamCleaner(handle, GeneratedWorldStream.lib));
    }

    companion object {
        internal val libClass: Class<GeneratedWorldStreamLib> = GeneratedWorldStreamLib::class.java
        internal val lib: GeneratedWorldStreamLib = Native.load("nucleation", libClass)
    }

    /** Number of chunks not yet requested from the source.
    */
    fun remaining(): ULong {

        val returnVal = lib.GeneratedWorldStream_remaining(handle);
        return (returnVal.toULong())
    }

    /** Generate and return the next chunk. Returns `NotFound` at end-of-stream,
    *and `Generation` if the underlying source failed on a valid request.
    */
    fun next(): Result<GeneratedChunk> {

        val returnVal = lib.GeneratedWorldStream_next(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal
            val returnOpaque = GeneratedChunk(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
