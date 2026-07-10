package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface WorldStreamLib: Library {
    fun WorldStream_destroy(handle: Pointer)
    fun WorldStream_open_dir(path: Slice): ResultPointerInt
    fun WorldStream_open_dir_bounded(path: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun WorldStream_from_zip(data: Slice): ResultPointerInt
    fun WorldStream_from_zip_bounded(data: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun WorldStream_next(handle: Pointer): ResultPointerInt
}
/** A streaming iterator over the chunks of a world.
*/
class WorldStream internal constructor (
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

    private class WorldStreamCleaner(val handle: Pointer, val lib: WorldStreamLib) : Runnable {
        override fun run() {
            lib.WorldStream_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, WorldStream.WorldStreamCleaner(handle, WorldStream.lib));
    }

    companion object {
        internal val libClass: Class<WorldStreamLib> = WorldStreamLib::class.java
        internal val lib: WorldStreamLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Open a streaming iterator over a world directory.
        */
        fun openDir(path: String): Result<WorldStream> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
            
            val returnVal = lib.WorldStream_open_dir(pathSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = WorldStream(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                pathSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Open a streaming iterator over a world directory, bounded to the given
        *block-coordinate box `[min_x..max_x, min_y..max_y, min_z..max_z]`.
        */
        fun openDirBounded(path: String, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<WorldStream> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
            
            val returnVal = lib.WorldStream_open_dir_bounded(pathSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = WorldStream(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                pathSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Open a streaming iterator from a zip archive in memory.
        */
        fun fromZip(data: UByteArray): Result<WorldStream> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.WorldStream_from_zip(dataSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = WorldStream(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Open a bounded streaming iterator from a zip archive in memory.
        */
        fun fromZipBounded(data: UByteArray, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<WorldStream> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.WorldStream_from_zip_bounded(dataSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = WorldStream(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
    }
    
    /** Advance the iterator and return the next chunk view. Errors with
    *`NotFound` at end-of-stream (the old ABI returned NULL). Corrupt
    *chunks are silently skipped, matching the old ABI.
    */
    fun next(): Result<WorldChunkView> {
        
        val returnVal = lib.WorldStream_next(handle);
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