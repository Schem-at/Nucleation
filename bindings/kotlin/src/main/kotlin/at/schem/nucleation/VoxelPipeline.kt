package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface VoxelPipelineLib: Library {
    fun VoxelPipeline_destroy(handle: Pointer)
    fun VoxelPipeline_create(geometry: Pointer, material: Pointer): Pointer
    fun VoxelPipeline_add_fluid(handle: Pointer, field: Pointer, block: Slice): ResultUnitInt
    fun VoxelPipeline_add_fluid_level(handle: Pointer, yLevel: Float, block: Slice): ResultUnitInt
    fun VoxelPipeline_add_scatter(handle: Pointer, rule: Pointer): Unit
    fun VoxelPipeline_set_fill_replace(handle: Pointer): Unit
    fun VoxelPipeline_set_fill_keep_existing(handle: Pointer): Unit
    fun VoxelPipeline_set_fill_replacing(handle: Pointer, targetsJson: Slice): ResultUnitInt
    fun VoxelPipeline_to_schematic(handle: Pointer, name: Slice): ResultPointerInt
    fun VoxelPipeline_to_schematic_bounded(handle: Pointer, name: Slice, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultPointerInt
    fun VoxelPipeline_apply_to_schematic(handle: Pointer, schematic: Pointer): ResultUnitInt
    fun VoxelPipeline_apply_to_schematic_bounded(handle: Pointer, schematic: Pointer, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): ResultUnitInt
}
/** Unified brick-pruned, parallel voxelization pipeline.
*/
class VoxelPipeline internal constructor (
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

    private class VoxelPipelineCleaner(val handle: Pointer, val lib: VoxelPipelineLib) : Runnable {
        override fun run() {
            lib.VoxelPipeline_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, VoxelPipeline.VoxelPipelineCleaner(handle, VoxelPipeline.lib));
    }

    companion object {
        internal val libClass: Class<VoxelPipelineLib> = VoxelPipelineLib::class.java
        internal val lib: VoxelPipelineLib = Native.load("nucleation", libClass)
        @JvmStatic

        fun create(geometry: Field, material: Material): VoxelPipeline {

            val returnVal = lib.VoxelPipeline_create(geometry.handle, material.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal
            val returnOpaque = VoxelPipeline(handle, selfEdges, true)
            return returnOpaque
        }
    }

    fun addFluid(field: Field, block: String): Result<Unit> {
        val blockSliceMemory = PrimitiveArrayTools.borrowUtf8(block)

        val returnVal = lib.VoxelPipeline_add_fluid(handle, field.handle, blockSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockSliceMemory.close()
        }
    }

    fun addFluidLevel(yLevel: Float, block: String): Result<Unit> {
        val blockSliceMemory = PrimitiveArrayTools.borrowUtf8(block)

        val returnVal = lib.VoxelPipeline_add_fluid_level(handle, yLevel, blockSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            blockSliceMemory.close()
        }
    }

    fun addScatter(rule: ScatterRule): Unit {

        val returnVal = lib.VoxelPipeline_add_scatter(handle, rule.handle);

    }

    fun setFillReplace(): Unit {

        val returnVal = lib.VoxelPipeline_set_fill_replace(handle);

    }

    fun setFillKeepExisting(): Unit {

        val returnVal = lib.VoxelPipeline_set_fill_keep_existing(handle);

    }

    fun setFillReplacing(targetsJson: String): Result<Unit> {
        val targetsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(targetsJson)

        val returnVal = lib.VoxelPipeline_set_fill_replacing(handle, targetsJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            targetsJsonSliceMemory.close()
        }
    }

    fun toSchematic(name: String): Result<Schematic> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

        val returnVal = lib.VoxelPipeline_to_schematic(handle, nameSliceMemory.slice);
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
            nameSliceMemory.close()
        }
    }

    fun toSchematicBounded(name: String, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Schematic> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

        val returnVal = lib.VoxelPipeline_to_schematic_bounded(handle, nameSliceMemory.slice, minX, minY, minZ, maxX, maxY, maxZ);
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
            nameSliceMemory.close()
        }
    }

    fun applyToSchematic(schematic: Schematic): Result<Unit> {

        val returnVal = lib.VoxelPipeline_apply_to_schematic(handle, schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    fun applyToSchematicBounded(schematic: Schematic, minX: Int, minY: Int, minZ: Int, maxX: Int, maxY: Int, maxZ: Int): Result<Unit> {

        val returnVal = lib.VoxelPipeline_apply_to_schematic_bounded(handle, schematic.handle /* note this is a mutable reference. Think carefully about using, especially concurrently */, minX, minY, minZ, maxX, maxY, maxZ);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
