package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ChunkMeshResultLib: Library {
    fun ChunkMeshResult_destroy(handle: Pointer)
    fun ChunkMeshResult_create(schematic: Pointer, pack: Pointer, config: Pointer): ResultPointerInt
    fun ChunkMeshResult_create_with_size(schematic: Pointer, pack: Pointer, config: Pointer, chunkSize: Int): ResultPointerInt
    fun ChunkMeshResult_create_with_atlas(schematic: Pointer, pack: Pointer, config: Pointer, chunkSize: Int, atlas: Pointer): ResultPointerInt
    fun ChunkMeshResult_chunk_count(handle: Pointer): FFIUint32
    fun ChunkMeshResult_chunk_coordinate_at(handle: Pointer, index: FFIUint32): ResultBlockPosNativeInt
    fun ChunkMeshResult_get_mesh(handle: Pointer, cx: Int, cy: Int, cz: Int): ResultPointerInt
    fun ChunkMeshResult_total_vertex_count(handle: Pointer): FFIUint32
    fun ChunkMeshResult_total_triangle_count(handle: Pointer): FFIUint32
    fun ChunkMeshResult_nucm_data_b64(handle: Pointer, write: Pointer): Unit
    fun ChunkMeshResult_nucm_data_with_atlas_b64(handle: Pointer, atlas: Pointer, write: Pointer): Unit
}
/** Per-chunk mesh results. Wraps [crate::meshing::ChunkMeshResult].
*/
class ChunkMeshResult internal constructor (
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

    private class ChunkMeshResultCleaner(val handle: Pointer, val lib: ChunkMeshResultLib) : Runnable {
        override fun run() {
            lib.ChunkMeshResult_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, ChunkMeshResult.ChunkMeshResultCleaner(handle, ChunkMeshResult.lib));
    }

    companion object {
        internal val libClass: Class<ChunkMeshResultLib> = ChunkMeshResultLib::class.java
        internal val lib: ChunkMeshResultLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Mesh with the default chunk size (old ABI: `schematic_mesh_by_chunk`).
        */
        fun create(schematic: Schematic, pack: ResourcePack, config: MeshConfig): Result<ChunkMeshResult> {
            
            val returnVal = lib.ChunkMeshResult_create(schematic.handle, pack.handle, config.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = ChunkMeshResult(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** Mesh with an explicit chunk size (old ABI: `schematic_mesh_by_chunk_size`).
        */
        fun createWithSize(schematic: Schematic, pack: ResourcePack, config: MeshConfig, chunkSize: Int): Result<ChunkMeshResult> {
            
            val returnVal = lib.ChunkMeshResult_create_with_size(schematic.handle, pack.handle, config.handle, chunkSize);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = ChunkMeshResult(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** Mesh chunks against a pre-built shared atlas, synchronously
        *(old ABI: `schematic_mesh_chunks_with_atlas`). For progress
        *reporting use [MeshJob::start] instead.
        */
        fun createWithAtlas(schematic: Schematic, pack: ResourcePack, config: MeshConfig, chunkSize: Int, atlas: TextureAtlas): Result<ChunkMeshResult> {
            
            val returnVal = lib.ChunkMeshResult_create_with_atlas(schematic.handle, pack.handle, config.handle, chunkSize, atlas.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = ChunkMeshResult(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }
    
    fun chunkCount(): UInt {
        
        val returnVal = lib.ChunkMeshResult_chunk_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Coordinate of the `index`-th chunk (old ABI:
    *`chunkmeshresult_chunk_coordinates` returned all of them flat).
    */
    fun chunkCoordinateAt(index: UInt): Result<BlockPos> {
        
        val returnVal = lib.ChunkMeshResult_chunk_coordinate_at(handle, FFIUint32(index));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = BlockPos.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The mesh for one chunk coordinate (cloned).
    */
    fun getMesh(cx: Int, cy: Int, cz: Int): Result<MeshResult> {
        
        val returnVal = lib.ChunkMeshResult_get_mesh(handle, cx, cy, cz);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal 
            val returnOpaque = MeshResult(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun totalVertexCount(): UInt {
        
        val returnVal = lib.ChunkMeshResult_total_vertex_count(handle);
        return (returnVal.toUInt())
    }
    
    fun totalTriangleCount(): UInt {
        
        val returnVal = lib.ChunkMeshResult_total_triangle_count(handle);
        return (returnVal.toUInt())
    }
    
    /** All chunk meshes serialized in the NUCM cache format, base64-encoded.
    */
    fun nucmDataB64(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ChunkMeshResult_nucm_data_b64(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** NUCM v2 with a shared atlas, base64-encoded.
    */
    fun nucmDataWithAtlasB64(atlas: TextureAtlas): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ChunkMeshResult_nucm_data_with_atlas_b64(handle, atlas.handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }

}