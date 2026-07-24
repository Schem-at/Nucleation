package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface WsPartitionHintsLib: Library {
    fun WsPartitionHints_destroy(handle: Pointer)
    fun WsPartitionHints_create(): Pointer
    fun WsPartitionHints_add(handle: Pointer, id: Slice, x0: Int, x1: Int, z0: Int, z1: Int): ResultUnitInt
    fun WsPartitionHints_len(handle: Pointer): FFIUint32
}
/** Caller-supplied partition hints (full-column boxes a cluster may never
*span under [PartitionPolicy::HardCut]). Order does not matter:
*[PartitionIndex::new](crate::world_segment::partition::PartitionIndex)
*sorts hints by full content at construction time.
*/
class WsPartitionHints internal constructor (
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

    private class WsPartitionHintsCleaner(val handle: Pointer, val lib: WsPartitionHintsLib) : Runnable {
        override fun run() {
            lib.WsPartitionHints_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, WsPartitionHints.WsPartitionHintsCleaner(handle, WsPartitionHints.lib));
    }

    companion object {
        internal val libClass: Class<WsPartitionHintsLib> = WsPartitionHintsLib::class.java
        internal val lib: WsPartitionHintsLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(): WsPartitionHints {
            
            val returnVal = lib.WsPartitionHints_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = WsPartitionHints(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** Add a full-column hint (`y_range: None`) covering inclusive
    *`x0..=x1, z0..=z1`.
    */
    fun add(id: String, x0: Int, x1: Int, z0: Int, z1: Int): Result<Unit> {
        val idSliceMemory = PrimitiveArrayTools.borrowUtf8(id)
        
        val returnVal = lib.WsPartitionHints_add(handle, idSliceMemory.slice, x0, x1, z0, z1);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            idSliceMemory.close()
        }
    }
    
    fun len(): UInt {
        
        val returnVal = lib.WsPartitionHints_len(handle);
        return (returnVal.toUInt())
    }

}