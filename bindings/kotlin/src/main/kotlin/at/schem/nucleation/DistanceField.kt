package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface DistanceFieldLib: Library {
    fun DistanceField_destroy(handle: Pointer)
    fun DistanceField_from_schematic(schematic: Pointer): Pointer
    fun DistanceField_depth(handle: Pointer, x: Int, y: Int, z: Int): Int
    fun DistanceField_slope(handle: Pointer, x: Int, y: Int, z: Int): Float
    fun DistanceField_normal_json(handle: Pointer, x: Int, y: Int, z: Int, write: Pointer): Unit
}

class DistanceField internal constructor (
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

    private class DistanceFieldCleaner(val handle: Pointer, val lib: DistanceFieldLib) : Runnable {
        override fun run() {
            lib.DistanceField_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, DistanceField.DistanceFieldCleaner(handle, DistanceField.lib));
    }

    companion object {
        internal val libClass: Class<DistanceFieldLib> = DistanceFieldLib::class.java
        internal val lib: DistanceFieldLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Distance transform of a build's occupied voxels: every solid block
        *learns how many blocks it sits below the surface, and the gradient of
        *that depth gives the outward normal. Computed once over the
        *schematic's bounding box.
        */
        fun fromSchematic(schematic: Schematic): DistanceField {
            
            val returnVal = lib.DistanceField_from_schematic(schematic.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = DistanceField(handle, selfEdges, true)
            return returnOpaque
        }
    }
    
    /** Blocks below the surface at a voxel: 0 for empty/outside, 1 at the
    *surface, increasing inward.
    */
    fun depth(x: Int, y: Int, z: Int): Int {
        
        val returnVal = lib.DistanceField_depth(handle, x, y, z);
        return (returnVal)
    }
    
    /** The upward component of the outward surface normal: 1 on flat ground,
    *0 on a vertical face, negative under an overhang. The scalar to key
    *slope-based landscaping on (grass on the flats, stone on the steeps).
    */
    fun slope(x: Int, y: Int, z: Int): Float {
        
        val returnVal = lib.DistanceField_slope(handle, x, y, z);
        return (returnVal)
    }
    
    /** The full outward surface normal as JSON `[nx, ny, nz]`.
    */
    fun normalJson(x: Int, y: Int, z: Int): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.DistanceField_normal_json(handle, x, y, z, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }

}