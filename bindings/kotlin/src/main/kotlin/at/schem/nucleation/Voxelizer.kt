package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface VoxelizerLib: Library {
    fun Voxelizer_destroy(handle: Pointer)
    fun Voxelizer_shape_from_glb(data: Slice, targetSize: Float, shell: Float): ResultPointerInt
    fun Voxelizer_shape_from_obj(text: Slice, targetSize: Float, shell: Float): ResultPointerInt
    fun Voxelizer_schematic_from_glb_textured(data: Slice, targetSize: Float, shell: Float, palette: Pointer, name: Slice): ResultPointerInt
}
/** Namespace for the mesh-voxelization entry points (GLB/OBJ → shapes
*and textured schematics).
*/
class Voxelizer internal constructor (
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

    private class VoxelizerCleaner(val handle: Pointer, val lib: VoxelizerLib) : Runnable {
        override fun run() {
            lib.Voxelizer_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Voxelizer.VoxelizerCleaner(handle, Voxelizer.lib));
    }

    companion object {
        internal val libClass: Class<VoxelizerLib> = VoxelizerLib::class.java
        internal val lib: VoxelizerLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Load a binary glTF (`.glb`, embedded buffers/images) and voxelize
        *it into a fillable Shape: the model is uniformly scaled so its
        *largest dimension equals `target_size` voxels, centered on x/z
        *with its base resting at y = 0. Solidity is a parity test at each
        *voxel center (robust on closed meshes), plus — when `shell` > 0 —
        *every voxel whose center is within `shell` blocks of the surface,
        *which rescues thin walls and hollow vessels (0.7–1.0 closes
        *single-voxel shells; 0 = pure parity; a *negative* `shell` is
        *surface-only — a skin |shell| blocks thick with no interior fill,
        *for open sheets/ribbons that dip or self-overlap). Errors with `Parse` on
        *malformed/triangle-less GLB and `InvalidArgument` on a
        *non-positive `target_size`.
        */
        fun shapeFromGlb(data: UByteArray, targetSize: Float, shell: Float): Result<Shape> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            
            val returnVal = lib.Voxelizer_shape_from_glb(dataSliceMemory.slice, targetSize, shell);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Shape(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Load a Wavefront OBJ (`v`/`vt`/`f` lines; polygon faces are
        *fan-triangulated, negative indices supported, materials ignored)
        *and voxelize it into a fillable Shape, fitted and shelled exactly
        *like `shape_from_glb`. Errors with `Parse` on malformed/triangle-less
        *OBJ and `InvalidArgument` on invalid UTF-8 or a non-positive
        *`target_size`.
        */
        fun shapeFromObj(text: String, targetSize: Float, shell: Float): Result<Shape> {
            val textSliceMemory = PrimitiveArrayTools.borrowUtf8(text)
            
            val returnVal = lib.Voxelizer_shape_from_obj(textSliceMemory.slice, targetSize, shell);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Shape(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                textSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Load a binary glTF and voxelize it directly into a textured
        *schematic named `name`: every solid voxel becomes the `palette`
        *block closest to its nearest-surface texture color (interior
        *voxels inherit the nearest surface color; voxels without texture
        *info snap to mid-gray). `shell` behaves as in `shape_from_glb` —
        *use ~0.7 for thin-walled models. Errors with `Parse` on malformed GLB and
        *`InvalidArgument` on invalid UTF-8 or a non-positive
        *`target_size`.
        */
        fun schematicFromGlbTextured(data: UByteArray, targetSize: Float, shell: Float, palette: Palette, name: String): Result<Schematic> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            
            val returnVal = lib.Voxelizer_schematic_from_glb_textured(dataSliceMemory.slice, targetSize, shell, palette.handle, nameSliceMemory.slice);
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
                dataSliceMemory.close()
                nameSliceMemory.close()
            }
        }
    }

}