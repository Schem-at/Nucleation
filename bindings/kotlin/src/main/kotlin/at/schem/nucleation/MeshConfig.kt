package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MeshConfigLib: Library {
    fun MeshConfig_destroy(handle: Pointer)
    fun MeshConfig_create(): Pointer
    fun MeshConfig_set_cull_hidden_faces(handle: Pointer, val_: Boolean): Unit
    fun MeshConfig_cull_hidden_faces(handle: Pointer): Byte
    fun MeshConfig_set_ambient_occlusion(handle: Pointer, val_: Boolean): Unit
    fun MeshConfig_ambient_occlusion(handle: Pointer): Byte
    fun MeshConfig_set_ao_intensity(handle: Pointer, val_: Float): Unit
    fun MeshConfig_ao_intensity(handle: Pointer): Float
    fun MeshConfig_set_biome(handle: Pointer, biome: Slice): ResultUnitInt
    fun MeshConfig_clear_biome(handle: Pointer): Unit
    fun MeshConfig_biome(handle: Pointer, write: Pointer): ResultUnitInt
    fun MeshConfig_set_atlas_max_size(handle: Pointer, size: FFIUint32): Unit
    fun MeshConfig_atlas_max_size(handle: Pointer): FFIUint32
    fun MeshConfig_set_cull_occluded_blocks(handle: Pointer, val_: Boolean): Unit
    fun MeshConfig_cull_occluded_blocks(handle: Pointer): Byte
    fun MeshConfig_set_greedy_meshing(handle: Pointer, val_: Boolean): Unit
    fun MeshConfig_greedy_meshing(handle: Pointer): Byte
}
/** Mesh generation configuration. Wraps [crate::meshing::MeshConfig].
*/
class MeshConfig internal constructor (
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

    private class MeshConfigCleaner(val handle: Pointer, val lib: MeshConfigLib) : Runnable {
        override fun run() {
            lib.MeshConfig_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, MeshConfig.MeshConfigCleaner(handle, MeshConfig.lib));
    }

    companion object {
        internal val libClass: Class<MeshConfigLib> = MeshConfigLib::class.java
        internal val lib: MeshConfigLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(): MeshConfig {
            
            val returnVal = lib.MeshConfig_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = MeshConfig(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    fun setCullHiddenFaces(val_: Boolean): Unit {
        
        val returnVal = lib.MeshConfig_set_cull_hidden_faces(handle, val_);
        
    }
    
    fun cullHiddenFaces(): Boolean {
        
        val returnVal = lib.MeshConfig_cull_hidden_faces(handle);
        return (returnVal > 0)
    }
    
    fun setAmbientOcclusion(val_: Boolean): Unit {
        
        val returnVal = lib.MeshConfig_set_ambient_occlusion(handle, val_);
        
    }
    
    fun ambientOcclusion(): Boolean {
        
        val returnVal = lib.MeshConfig_ambient_occlusion(handle);
        return (returnVal > 0)
    }
    
    fun setAoIntensity(val_: Float): Unit {
        
        val returnVal = lib.MeshConfig_set_ao_intensity(handle, val_);
        
    }
    
    fun aoIntensity(): Float {
        
        val returnVal = lib.MeshConfig_ao_intensity(handle);
        return (returnVal)
    }
    
    /** Set the biome used for tinting (e.g. "plains", "swamp").
    */
    fun setBiome(biome: String): Result<Unit> {
        val biomeSliceMemory = PrimitiveArrayTools.borrowUtf8(biome)
        
        val returnVal = lib.MeshConfig_set_biome(handle, biomeSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            biomeSliceMemory.close()
        }
    }
    
    /** Clear the biome (old ABI: `meshconfig_set_biome(NULL)`).
    */
    fun clearBiome(): Unit {
        
        val returnVal = lib.MeshConfig_clear_biome(handle);
        
    }
    
    /** The configured biome; `NotFound` if none is set.
    */
    fun biome(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.MeshConfig_biome(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun setAtlasMaxSize(size: UInt): Unit {
        
        val returnVal = lib.MeshConfig_set_atlas_max_size(handle, FFIUint32(size));
        
    }
    
    fun atlasMaxSize(): UInt {
        
        val returnVal = lib.MeshConfig_atlas_max_size(handle);
        return (returnVal.toUInt())
    }
    
    fun setCullOccludedBlocks(val_: Boolean): Unit {
        
        val returnVal = lib.MeshConfig_set_cull_occluded_blocks(handle, val_);
        
    }
    
    fun cullOccludedBlocks(): Boolean {
        
        val returnVal = lib.MeshConfig_cull_occluded_blocks(handle);
        return (returnVal > 0)
    }
    
    fun setGreedyMeshing(val_: Boolean): Unit {
        
        val returnVal = lib.MeshConfig_set_greedy_meshing(handle, val_);
        
    }
    
    fun greedyMeshing(): Boolean {
        
        val returnVal = lib.MeshConfig_greedy_meshing(handle);
        return (returnVal > 0)
    }

}