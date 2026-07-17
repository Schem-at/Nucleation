package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MultiMeshResultLib: Library {
    fun MultiMeshResult_destroy(handle: Pointer)
    fun MultiMeshResult_create(schematic: Pointer, pack: Pointer, config: Pointer): ResultPointerInt
    fun MultiMeshResult_region_names_json(handle: Pointer, write: Pointer): Unit
    fun MultiMeshResult_get_mesh(handle: Pointer, regionName: Slice): ResultPointerInt
    fun MultiMeshResult_total_vertex_count(handle: Pointer): FFIUint32
    fun MultiMeshResult_total_triangle_count(handle: Pointer): FFIUint32
    fun MultiMeshResult_mesh_count(handle: Pointer): FFIUint32
}
/** Per-region mesh results. Wraps [crate::meshing::MultiMeshResult].
*/
class MultiMeshResult internal constructor (
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

    private class MultiMeshResultCleaner(val handle: Pointer, val lib: MultiMeshResultLib) : Runnable {
        override fun run() {
            lib.MultiMeshResult_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, MultiMeshResult.MultiMeshResultCleaner(handle, MultiMeshResult.lib));
    }

    companion object {
        internal val libClass: Class<MultiMeshResultLib> = MultiMeshResultLib::class.java
        internal val lib: MultiMeshResultLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Mesh each region separately (old ABI: `schematic_mesh_by_region`).
        */
        fun create(schematic: Schematic, pack: ResourcePack, config: MeshConfig): Result<MultiMeshResult> {
            
            val returnVal = lib.MultiMeshResult_create(schematic.handle, pack.handle, config.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = MultiMeshResult(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }
    
    /** Region names, as a JSON array string.
    */
    fun regionNamesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.MultiMeshResult_region_names_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** The mesh for one region (cloned).
    */
    fun getMesh(regionName: String): Result<MeshResult> {
        val regionNameSliceMemory = PrimitiveArrayTools.borrowUtf8(regionName)
        
        val returnVal = lib.MultiMeshResult_get_mesh(handle, regionNameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = MeshResult(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            regionNameSliceMemory.close()
        }
    }
    
    /** Total vertex count across all region meshes.
    */
    fun totalVertexCount(): UInt {
        
        val returnVal = lib.MultiMeshResult_total_vertex_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Total triangle count across all region meshes.
    */
    fun totalTriangleCount(): UInt {
        
        val returnVal = lib.MultiMeshResult_total_triangle_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Number of region meshes.
    */
    fun meshCount(): UInt {
        
        val returnVal = lib.MultiMeshResult_mesh_count(handle);
        return (returnVal.toUInt())
    }

}