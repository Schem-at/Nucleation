package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface BlockStateLib: Library {
    fun BlockState_destroy(handle: Pointer)
    fun BlockState_create(name: Slice): Pointer
    fun BlockState_with_property(handle: Pointer, key: Slice, value: Slice): ResultPointerInt
    fun BlockState_name(handle: Pointer, write: Pointer): Unit
    fun BlockState_properties_json(handle: Pointer, write: Pointer): Unit
}
/** A block state: a block name plus its properties. Port of the old
*`BlockStateWrapper` / `blockstate_*` fns.
*/
class BlockState internal constructor (
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

    private class BlockStateCleaner(val handle: Pointer, val lib: BlockStateLib) : Runnable {
        override fun run() {
            lib.BlockState_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, BlockState.BlockStateCleaner(handle, BlockState.lib));
    }

    companion object {
        internal val libClass: Class<BlockStateLib> = BlockStateLib::class.java
        internal val lib: BlockStateLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(name: String): BlockState {
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.BlockState_create(nameSliceMemory.slice);
            try {
                val selfEdges: List<Any> = listOf()
                val handle = returnVal 
                val returnOpaque = BlockState(handle, selfEdges, true)
                return returnOpaque
            } finally {
                nameSliceMemory.close()
            }
        }
    }
    
    /** A copy of this block state with `key=value` set; the original is
    *unchanged.
    */
    fun withProperty(key: String, value: String): Result<BlockState> {
        val keySliceMemory = PrimitiveArrayTools.borrowUtf8(key)
        val valueSliceMemory = PrimitiveArrayTools.borrowUtf8(value)
        
        val returnVal = lib.BlockState_with_property(handle, keySliceMemory.slice, valueSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = BlockState(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            keySliceMemory.close()
            valueSliceMemory.close()
        }
    }
    
    fun name(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.BlockState_name(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** The properties as a JSON object of string→string (the old
    *`CPropertyArray`).
    */
    fun propertiesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.BlockState_properties_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }

}