package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface WsProfileLib: Library {
    fun WsProfile_destroy(handle: Pointer)
    fun WsProfile_derive_from_dir(worldDir: Slice, minY: Int, maxY: Int, sample: FFIUint32, coverage: Float): ResultPointerInt
    fun WsProfile_band_min(handle: Pointer): Int
    fun WsProfile_band_max(handle: Pointer): Int
    fun WsProfile_palette_len(handle: Pointer): FFIUint32
    fun WsProfile_write_palette_json(handle: Pointer, write: Pointer): ResultUnitInt
}
/** A pinned [WorldProfile](crate::world_segment::profile::WorldProfile):
*the substrate palette + Y band derived (or supplied) once per world and
*reused across every segmentation run against it.
*/
class WsProfile internal constructor (
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

    private class WsProfileCleaner(val handle: Pointer, val lib: WsProfileLib) : Runnable {
        override fun run() {
            lib.WsProfile_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, WsProfile.WsProfileCleaner(handle, WsProfile.lib));
    }

    companion object {
        internal val libClass: Class<WsProfileLib> = WsProfileLib::class.java
        internal val lib: WsProfileLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Derive a profile from up to `sample` tiles (regions) of a world
        *directory, in ascending `(x, z)` region order. `coverage` is
        *`ProfileParams::min_slab_coverage`; every other `ProfileParams`
        *field uses its default (`sample_stride: 1`, `y_scan: (-64, 320)`).
        */
        fun deriveFromDir(worldDir: String, minY: Int, maxY: Int, sample: UInt, coverage: Float): Result<WsProfile> {
            val worldDirSliceMemory = PrimitiveArrayTools.borrowUtf8(worldDir)
            
            val returnVal = lib.WsProfile_derive_from_dir(worldDirSliceMemory.slice, minY, maxY, FFIUint32(sample), coverage);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = WsProfile(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                worldDirSliceMemory.close()
            }
        }
    }
    
    /** The derived substrate Y band's lower bound (inclusive).
    */
    fun bandMin(): Int {
        
        val returnVal = lib.WsProfile_band_min(handle);
        return (returnVal)
    }
    
    /** The derived substrate Y band's upper bound (inclusive).
    */
    fun bandMax(): Int {
        
        val returnVal = lib.WsProfile_band_max(handle);
        return (returnVal)
    }
    
    /** Number of distinct block names in the substrate palette.
    */
    fun paletteLen(): UInt {
        
        val returnVal = lib.WsProfile_palette_len(handle);
        return (returnVal.toUInt())
    }
    
    /** The substrate palette as a JSON array of block-name strings.
    */
    fun writePaletteJson(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.WsProfile_write_palette_json(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}