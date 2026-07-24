package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MeshResultLib: Library {
    fun MeshResult_destroy(handle: Pointer)
    fun MeshResult_create(schematic: Pointer, pack: Pointer, config: Pointer): ResultPointerInt
    fun MeshResult_create_usdz(schematic: Pointer, pack: Pointer, config: Pointer): ResultPointerInt
    fun MeshResult_glb_data_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun MeshResult_usdz_data_b64(handle: Pointer, write: Pointer): ResultUnitInt
    fun MeshResult_nucm_data_b64(handle: Pointer, write: Pointer): Unit
    fun MeshResult_vertex_count(handle: Pointer): FFIUint32
    fun MeshResult_triangle_count(handle: Pointer): FFIUint32
    fun MeshResult_has_transparency(handle: Pointer): Byte
    fun MeshResult_bounds(handle: Pointer): MeshBoundsNative
}
/** A single mesh output. Wraps [crate::meshing::MeshResult].
*/
class MeshResult internal constructor (
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

    private class MeshResultCleaner(val handle: Pointer, val lib: MeshResultLib) : Runnable {
        override fun run() {
            lib.MeshResult_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, MeshResult.MeshResultCleaner(handle, MeshResult.lib));
    }

    companion object {
        internal val libClass: Class<MeshResultLib> = MeshResultLib::class.java
        internal val lib: MeshResultLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Mesh an entire schematic in one pass (old ABI: `schematic_to_mesh`).
        */
        fun create(schematic: Schematic, pack: ResourcePack, config: MeshConfig): Result<MeshResult> {
            
            val returnVal = lib.MeshResult_create(schematic.handle, pack.handle, config.handle);
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
        @JvmStatic
        
        /** Mesh a schematic with USDZ-compatible output (old ABI: `schematic_to_usdz`).
        */
        fun createUsdz(schematic: Schematic, pack: ResourcePack, config: MeshConfig): Result<MeshResult> {
            
            val returnVal = lib.MeshResult_create_usdz(schematic.handle, pack.handle, config.handle);
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
    }
    
    /** The mesh as a binary GLB, base64-encoded.
    */
    fun glbDataB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.MeshResult_glb_data_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The mesh as a USDZ archive, base64-encoded.
    */
    fun usdzDataB64(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.MeshResult_usdz_data_b64(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The mesh serialized in the NUCM cache format, base64-encoded.
    */
    fun nucmDataB64(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.MeshResult_nucm_data_b64(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Total number of vertices in the mesh.
    */
    fun vertexCount(): UInt {
        
        val returnVal = lib.MeshResult_vertex_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Total number of triangles in the mesh.
    */
    fun triangleCount(): UInt {
        
        val returnVal = lib.MeshResult_triangle_count(handle);
        return (returnVal.toUInt())
    }
    
    /** Whether the mesh contains any transparent or translucent geometry.
    */
    fun hasTransparency(): Boolean {
        
        val returnVal = lib.MeshResult_has_transparency(handle);
        return (returnVal > 0)
    }
    
    /** Axis-aligned bounding box of the mesh, in world units.
    */
    fun bounds(): MeshBounds {
        
        val returnVal = lib.MeshResult_bounds(handle);
        val returnStruct = MeshBounds.fromNative(returnVal)
        return returnStruct
    }

}