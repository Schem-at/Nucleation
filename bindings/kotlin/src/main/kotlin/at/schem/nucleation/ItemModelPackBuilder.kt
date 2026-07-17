package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ItemModelPackBuilderLib: Library {
    fun ItemModelPackBuilder_destroy(handle: Pointer)
    fun ItemModelPackBuilder_create(): Pointer
    fun ItemModelPackBuilder_len(handle: Pointer): FFIUint32
    fun ItemModelPackBuilder_build_zip_b64(handle: Pointer, write: Pointer): ResultUnitInt
}
/** Accumulates [ItemModelResult]s to build a combined resource pack ZIP
*(old ABI: `itemmodel_build_resource_pack` took an array of pointers).
*/
class ItemModelPackBuilder internal constructor (
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

    private class ItemModelPackBuilderCleaner(val handle: Pointer, val lib: ItemModelPackBuilderLib) : Runnable {
        override fun run() {
            lib.ItemModelPackBuilder_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, ItemModelPackBuilder.ItemModelPackBuilderCleaner(handle, ItemModelPackBuilder.lib));
    }

    companion object {
        internal val libClass: Class<ItemModelPackBuilderLib> = ItemModelPackBuilderLib::class.java
        internal val lib: ItemModelPackBuilderLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Create an empty pack builder.
        */
        fun create(): ItemModelPackBuilder {
            
            val returnVal = lib.ItemModelPackBuilder_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = ItemModelPackBuilder(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** Number of results added so far.
    */
    fun len(): UInt {
        
        val returnVal = lib.ItemModelPackBuilder_len(handle);
        return (returnVal.toUInt())
    }
    
    /** Build a resource pack ZIP from every added result, base64-encoded.
    */
    fun buildZipB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ItemModelPackBuilder_build_zip_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}