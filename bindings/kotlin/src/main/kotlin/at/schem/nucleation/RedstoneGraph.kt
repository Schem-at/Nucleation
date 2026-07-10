package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface RedstoneGraphLib: Library {
    fun RedstoneGraph_destroy(handle: Pointer)
    fun RedstoneGraph_node_count(handle: Pointer): FFIUint32
    fun RedstoneGraph_edge_count(handle: Pointer): FFIUint32
    fun RedstoneGraph_nodes_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun RedstoneGraph_edges_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun RedstoneGraph_features_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun RedstoneGraph_fingerprint(handle: Pointer, preset: Slice, write: Pointer): ResultUnitInt
    fun RedstoneGraph_to_json(handle: Pointer, write: Pointer): ResultUnitInt
    fun RedstoneGraph_from_json(json: Slice): ResultPointerInt
}
/** An extracted redstone logic graph. Wraps
*[crate::simulation::graph::RedstoneGraph].
*/
class RedstoneGraph internal constructor (
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

    private class RedstoneGraphCleaner(val handle: Pointer, val lib: RedstoneGraphLib) : Runnable {
        override fun run() {
            lib.RedstoneGraph_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, RedstoneGraph.RedstoneGraphCleaner(handle, RedstoneGraph.lib));
    }

    companion object {
        internal val libClass: Class<RedstoneGraphLib> = RedstoneGraphLib::class.java
        internal val lib: RedstoneGraphLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Deserialize a graph from a JSON string.
        */
        fun fromJson(json: String): Result<RedstoneGraph> {
            val jsonSliceMemory = PrimitiveArrayTools.borrowUtf8(json)
            
            val returnVal = lib.RedstoneGraph_from_json(jsonSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = RedstoneGraph(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                jsonSliceMemory.close()
            }
        }
    }
    
    /** Number of nodes in the graph.
    */
    fun nodeCount(): UInt {
        
        val returnVal = lib.RedstoneGraph_node_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Total number of directed edges in the graph.
    */
    fun edgeCount(): UInt {
        
        val returnVal = lib.RedstoneGraph_edge_count(handle);
        return (returnVal.toUInt())
    }
    
    /** The nodes as a JSON array string.
    */
    fun nodesJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RedstoneGraph_nodes_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The directed edges as a JSON array string.
    */
    fun edgesJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RedstoneGraph_edges_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Computed graph features as a JSON object string.
    */
    fun featuresJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RedstoneGraph_features_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Fingerprint (hex string) for `preset` ("structural" | "functional" |
    *"exact"; empty defaults to "structural").
    */
    fun fingerprint(preset: String): Result<String> {
        val presetSliceMemory = PrimitiveArrayTools.borrowUtf8(preset)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RedstoneGraph_fingerprint(handle, presetSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            presetSliceMemory.close()
        }
    }
    
    /** Serialize the whole graph to JSON.
    */
    fun toJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RedstoneGraph_to_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}