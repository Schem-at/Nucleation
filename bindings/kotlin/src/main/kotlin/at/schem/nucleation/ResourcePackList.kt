package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ResourcePackListLib: Library {
    fun ResourcePackList_destroy(handle: Pointer)
    fun ResourcePackList_create(): Pointer
    fun ResourcePackList_add(handle: Pointer, data: Slice): Unit
    fun ResourcePackList_len(handle: Pointer): FFIUint32
}
/** Ordered list of raw resource-pack ZIP buffers, lowest priority first.
*Feed it to [ResourcePack::from_list].
*/
class ResourcePackList internal constructor (
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

    private class ResourcePackListCleaner(val handle: Pointer, val lib: ResourcePackListLib) : Runnable {
        override fun run() {
            lib.ResourcePackList_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, ResourcePackList.ResourcePackListCleaner(handle, ResourcePackList.lib));
    }

    companion object {
        internal val libClass: Class<ResourcePackListLib> = ResourcePackListLib::class.java
        internal val lib: ResourcePackListLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(): ResourcePackList {
            
            val returnVal = lib.ResourcePackList_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = ResourcePackList(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** Append one resource-pack ZIP buffer. Later buffers overlay earlier
    *ones on per-key collision (Minecraft pack-ordering semantics).
    */
    fun add(data: UByteArray): Unit {
        val dataSliceMemory = PrimitiveArrayTools.borrow(data)
        
        val returnVal = lib.ResourcePackList_add(handle, dataSliceMemory.slice);
        try {
            
        } finally {
            dataSliceMemory.close()
        }
    }
    
    fun len(): UInt {
        
        val returnVal = lib.ResourcePackList_len(handle);
        return (returnVal.toUInt())
    }

}