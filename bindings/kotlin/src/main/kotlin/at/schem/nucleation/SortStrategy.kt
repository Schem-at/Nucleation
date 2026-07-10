package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SortStrategyLib: Library {
    fun SortStrategy_destroy(handle: Pointer)
    fun SortStrategy_yxz(): Pointer
    fun SortStrategy_xyz(): Pointer
    fun SortStrategy_zyx(): Pointer
    fun SortStrategy_y_desc_xz(): Pointer
    fun SortStrategy_x_desc_yz(): Pointer
    fun SortStrategy_z_desc_yx(): Pointer
    fun SortStrategy_descending(): Pointer
    fun SortStrategy_distance_from(x: Int, y: Int, z: Int): Pointer
    fun SortStrategy_distance_from_desc(x: Int, y: Int, z: Int): Pointer
    fun SortStrategy_preserve(): Pointer
    fun SortStrategy_reverse(): Pointer
    fun SortStrategy_from_string(s: Slice): ResultPointerInt
    fun SortStrategy_name(handle: Pointer, write: Pointer): Unit
}
/** Ordering applied to region positions before bit assignment
*(PORTING rule 10).
*/
class SortStrategy internal constructor (
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

    private class SortStrategyCleaner(val handle: Pointer, val lib: SortStrategyLib) : Runnable {
        override fun run() {
            lib.SortStrategy_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, SortStrategy.SortStrategyCleaner(handle, SortStrategy.lib));
    }

    companion object {
        internal val libClass: Class<SortStrategyLib> = SortStrategyLib::class.java
        internal val lib: SortStrategyLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun yxz(): SortStrategy {
            
            val returnVal = lib.SortStrategy_yxz();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun xyz(): SortStrategy {
            
            val returnVal = lib.SortStrategy_xyz();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun zyx(): SortStrategy {
            
            val returnVal = lib.SortStrategy_zyx();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun yDescXz(): SortStrategy {
            
            val returnVal = lib.SortStrategy_y_desc_xz();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun xDescYz(): SortStrategy {
            
            val returnVal = lib.SortStrategy_x_desc_yz();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun zDescYx(): SortStrategy {
            
            val returnVal = lib.SortStrategy_z_desc_yx();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Old ABI name: `sort_strategy_descending`.
        */
        fun descending(): SortStrategy {
            
            val returnVal = lib.SortStrategy_descending();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun distanceFrom(x: Int, y: Int, z: Int): SortStrategy {
            
            val returnVal = lib.SortStrategy_distance_from(x, y, z);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun distanceFromDesc(x: Int, y: Int, z: Int): SortStrategy {
            
            val returnVal = lib.SortStrategy_distance_from_desc(x, y, z);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun preserve(): SortStrategy {
            
            val returnVal = lib.SortStrategy_preserve();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun reverse(): SortStrategy {
            
            val returnVal = lib.SortStrategy_reverse();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SortStrategy(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Parse from a name (e.g. "yxz", "descending", "preserve").
        */
        fun fromString(s: String): Result<SortStrategy> {
            val sSliceMemory = PrimitiveArrayTools.borrowUtf8(s)
            
            val returnVal = lib.SortStrategy_from_string(sSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = SortStrategy(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sSliceMemory.close()
            }
        }
    }
    
    /** The strategy name.
    */
    fun name(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.SortStrategy_name(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }

}