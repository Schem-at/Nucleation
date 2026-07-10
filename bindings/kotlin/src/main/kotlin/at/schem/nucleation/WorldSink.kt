package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface WorldSinkLib: Library {
    fun WorldSink_destroy(handle: Pointer)
    fun WorldSink_create(dir: Slice, optionsJson: Slice): ResultPointerInt
    fun WorldSink_open_existing(dir: Slice): ResultPointerInt
    fun WorldSink_write_chunk(handle: Pointer, view: Pointer): ResultUnitInt
    fun WorldSink_put_chunk(handle: Pointer, view: Pointer): ResultUnitInt
    fun WorldSink_finish(handle: Pointer): ResultUnitInt
}
/** A world writer. `finish` is consuming (PORTING rule 11): the inner sink is
*held in an `Option` and taken on `finish`; every method afterwards returns
*`AlreadyConsumed`. Dropping the handle without `finish` abandons the sink.
*/
class WorldSink internal constructor (
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

    private class WorldSinkCleaner(val handle: Pointer, val lib: WorldSinkLib) : Runnable {
        override fun run() {
            lib.WorldSink_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, WorldSink.WorldSinkCleaner(handle, WorldSink.lib));
    }

    companion object {
        internal val libClass: Class<WorldSinkLib> = WorldSinkLib::class.java
        internal val lib: WorldSinkLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Create a new world sink that writes fresh chunk data to `dir`.
        *`options_json` is a serialized `WorldExportOptions` (empty string =
        *defaults).
        */
        fun create(dir: String, optionsJson: String): Result<WorldSink> {
            val dirSliceMemory = PrimitiveArrayTools.borrowUtf8(dir)
            val optionsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(optionsJson)
            
            val returnVal = lib.WorldSink_create(dirSliceMemory.slice, optionsJsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = WorldSink(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dirSliceMemory.close()
                optionsJsonSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Open an existing world directory for patching via `put_chunk`.
        */
        fun openExisting(dir: String): Result<WorldSink> {
            val dirSliceMemory = PrimitiveArrayTools.borrowUtf8(dir)
            
            val returnVal = lib.WorldSink_open_existing(dirSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = WorldSink(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dirSliceMemory.close()
            }
        }
    }
    
    /** Write (append) a chunk view into the sink. The view is not consumed.
    */
    fun writeChunk(view: WorldChunkView): Result<Unit> {
        
        val returnVal = lib.WorldSink_write_chunk(handle, view.handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Overwrite the chunk at (`view.cx`, `view.cz`) of the sink's world with
    *the supplied view's block data. Only valid on sinks opened with
    *`open_existing`; errors with `Io` on a create-mode sink. The view is
    *not consumed.
    */
    fun putChunk(view: WorldChunkView): Result<Unit> {
        
        val returnVal = lib.WorldSink_put_chunk(handle, view.handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Finalise and flush all pending writes. Consuming (PORTING rule 11):
    *afterwards every method on this sink returns `AlreadyConsumed`.
    */
    fun finish(): Result<Unit> {
        
        val returnVal = lib.WorldSink_finish(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}