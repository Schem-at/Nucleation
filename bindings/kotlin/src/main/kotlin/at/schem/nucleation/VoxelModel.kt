package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface VoxelModelLib: Library {
    fun VoxelModel_destroy(handle: Pointer)
    fun VoxelModel_plan_json(handle: Pointer, optionsJson: Slice, write: Pointer): ResultUnitInt
    fun VoxelModel_to_schematic(handle: Pointer, optionsJson: Slice, palette: Pointer, name: Slice): ResultPointerInt
}
/** A parsed GLB/OBJ, reusable for size estimates and configured imports.
*/
class VoxelModel internal constructor (
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

    private class VoxelModelCleaner(val handle: Pointer, val lib: VoxelModelLib) : Runnable {
        override fun run() {
            lib.VoxelModel_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, VoxelModel.VoxelModelCleaner(handle, VoxelModel.lib));
    }

    companion object {
        internal val libClass: Class<VoxelModelLib> = VoxelModelLib::class.java
        internal val lib: VoxelModelLib = Native.load("nucleation", libClass)
    }

    /** Return {dimensions:[width,height,depth],volume} or {error:message}.
    *Options: target_size, axis (longest/x/y/z), hollow, optional lighting
    *{direction:[x,y,z],strength:0..1}, optional untextured_block.
    *Estimates preserve proportions and run before voxel-grid allocation.
    */
    fun planJson(optionsJson: String): Result<String> {
        val optionsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(optionsJson)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.VoxelModel_plan_json(handle, optionsJsonSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            optionsJsonSliceMemory.close()
        }
    }

    /** Import using plan_json's options. Anchored at (0,0,0), with exact
    *axis-based uniform scaling. Hollow uses a sparse surface raster;
    *lighting darkens sampled texture colours before palette matching.
    *Rejects oversized/over-complex output with InvalidArgument.
    */
    fun toSchematic(optionsJson: String, palette: Palette, name: String): Result<Schematic> {
        val optionsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(optionsJson)
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

        val returnVal = lib.VoxelModel_to_schematic(handle, optionsJsonSliceMemory.slice, palette.handle, nameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = Schematic(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            optionsJsonSliceMemory.close()
            nameSliceMemory.close()
        }
    }

}
