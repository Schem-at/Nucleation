package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface RawMeshExportLib: Library {
    fun RawMeshExport_destroy(handle: Pointer)
    fun RawMeshExport_create(schematic: Pointer, pack: Pointer, config: Pointer): ResultPointerInt
    fun RawMeshExport_vertex_count(handle: Pointer): FFIUint32
    fun RawMeshExport_triangle_count(handle: Pointer): FFIUint32
    fun RawMeshExport_positions_b64(handle: Pointer, write: Pointer): Unit
    fun RawMeshExport_normals_b64(handle: Pointer, write: Pointer): Unit
    fun RawMeshExport_uvs_b64(handle: Pointer, write: Pointer): Unit
    fun RawMeshExport_colors_b64(handle: Pointer, write: Pointer): Unit
    fun RawMeshExport_indices_b64(handle: Pointer, write: Pointer): Unit
    fun RawMeshExport_texture_rgba_b64(handle: Pointer, write: Pointer): Unit
    fun RawMeshExport_texture_width(handle: Pointer): FFIUint32
    fun RawMeshExport_texture_height(handle: Pointer): FFIUint32
}
/** Raw vertex streams for custom rendering. Wraps [crate::meshing::RawMeshExport].
*/
class RawMeshExport internal constructor (
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

    private class RawMeshExportCleaner(val handle: Pointer, val lib: RawMeshExportLib) : Runnable {
        override fun run() {
            lib.RawMeshExport_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, RawMeshExport.RawMeshExportCleaner(handle, RawMeshExport.lib));
    }

    companion object {
        internal val libClass: Class<RawMeshExportLib> = RawMeshExportLib::class.java
        internal val lib: RawMeshExportLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Export raw mesh data (old ABI: `schematic_to_raw_mesh`).
        */
        fun create(schematic: Schematic, pack: ResourcePack, config: MeshConfig): Result<RawMeshExport> {
            
            val returnVal = lib.RawMeshExport_create(schematic.handle, pack.handle, config.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal 
                val returnOpaque = RawMeshExport(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }
    
    /** Number of vertices in the exported mesh.
    */
    fun vertexCount(): UInt {
        
        val returnVal = lib.RawMeshExport_vertex_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Number of triangles in the exported mesh.
    */
    fun triangleCount(): UInt {
        
        val returnVal = lib.RawMeshExport_triangle_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Flat `[x,y,z,...]` positions as little-endian `f32` bytes, base64-encoded.
    */
    fun positionsB64(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RawMeshExport_positions_b64(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Flat normals as little-endian `f32` bytes, base64-encoded.
    */
    fun normalsB64(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RawMeshExport_normals_b64(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Flat UVs as little-endian `f32` bytes, base64-encoded.
    */
    fun uvsB64(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RawMeshExport_uvs_b64(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Flat vertex colors as little-endian `f32` bytes, base64-encoded.
    */
    fun colorsB64(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RawMeshExport_colors_b64(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Triangle indices as little-endian `u32` bytes, base64-encoded.
    */
    fun indicesB64(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RawMeshExport_indices_b64(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Raw RGBA texture pixels, base64-encoded.
    */
    fun textureRgbaB64(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.RawMeshExport_texture_rgba_b64(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Width of the baked texture in pixels.
    */
    fun textureWidth(): UInt {
        
        val returnVal = lib.RawMeshExport_texture_width(handle);
        return (returnVal.toUInt())
    }
    
    /** Height of the baked texture in pixels.
    */
    fun textureHeight(): UInt {
        
        val returnVal = lib.RawMeshExport_texture_height(handle);
        return (returnVal.toUInt())
    }

}