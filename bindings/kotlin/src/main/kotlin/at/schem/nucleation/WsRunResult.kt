package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface WsRunResultLib: Library {
    fun WsRunResult_destroy(handle: Pointer)
    fun WsRunResult_run_dir(job: Pointer, hints: Pointer, profile: Pointer, worldDir: Slice): ResultPointerInt
    fun WsRunResult_builds(handle: Pointer): FFIUint64
    fun WsRunResult_tier_confident(handle: Pointer): FFIUint64
    fun WsRunResult_tier_probable(handle: Pointer): FFIUint64
    fun WsRunResult_tier_debris(handle: Pointer): FFIUint64
    fun WsRunResult_cross_tile(handle: Pointer): FFIUint64
    fun WsRunResult_largest_block_count(handle: Pointer): FFIUint64
    fun WsRunResult_build_count(handle: Pointer): FFIUint32
    fun WsRunResult_stable_id_hex(handle: Pointer, index: FFIUint32, write: Pointer): ResultUnitInt
    fun WsRunResult_fingerprint_hex(handle: Pointer, index: FFIUint32, write: Pointer): ResultUnitInt
    fun WsRunResult_tier_of(handle: Pointer, index: FFIUint32): ResultFFIUint8Int
    fun WsRunResult_block_count_of(handle: Pointer, index: FFIUint32): ResultFFIUint64Int
    fun WsRunResult_bbox_min_of(handle: Pointer, index: FFIUint32): ResultBlockPosNativeInt
    fun WsRunResult_bbox_max_of(handle: Pointer, index: FFIUint32): ResultBlockPosNativeInt
    fun WsRunResult_write_schem_to(handle: Pointer, index: FFIUint32, path: Slice): ResultUnitInt
}
/** The materialized output of one segmentation run: every build (in the
*pipeline's deterministic stable-id order) plus the aggregate
*[RunStats](crate::world_segment::runner::RunStats).
*/
class WsRunResult internal constructor (
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

    private class WsRunResultCleaner(val handle: Pointer, val lib: WsRunResultLib) : Runnable {
        override fun run() {
            lib.WsRunResult_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, WsRunResult.WsRunResultCleaner(handle, WsRunResult.lib));
    }

    companion object {
        internal val libClass: Class<WsRunResultLib> = WsRunResultLib::class.java
        internal val lib: WsRunResultLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Run the full pipeline (source -> segment -> stitch -> score ->
        *identity -> materialize) over a world directory. No prior-snapshot
        *builds are supplied, so every build seeds a fresh stable id (see
        *`StableBuildId::seed`).
        *
        *See the module docs for why this catches a panic instead of
        *propagating it.
        */
        fun runDir(job: WsSegmentJob, hints: WsPartitionHints, profile: WsProfile, worldDir: String): Result<WsRunResult> {
            val worldDirSliceMemory = PrimitiveArrayTools.borrowUtf8(worldDir)
            
            val returnVal = lib.WsRunResult_run_dir(job.handle, hints.handle, profile.handle, worldDirSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = WsRunResult(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                worldDirSliceMemory.close()
            }
        }
    }
    
    /** Total builds materialized (same as `build_count`, from `RunStats`).
    */
    fun builds(): ULong {
        
        val returnVal = lib.WsRunResult_builds(handle);
        return (returnVal.toULong())
    }
    
    fun tierConfident(): ULong {
        
        val returnVal = lib.WsRunResult_tier_confident(handle);
        return (returnVal.toULong())
    }
    
    fun tierProbable(): ULong {
        
        val returnVal = lib.WsRunResult_tier_probable(handle);
        return (returnVal.toULong())
    }
    
    fun tierDebris(): ULong {
        
        val returnVal = lib.WsRunResult_tier_debris(handle);
        return (returnVal.toULong())
    }
    
    fun crossTile(): ULong {
        
        val returnVal = lib.WsRunResult_cross_tile(handle);
        return (returnVal.toULong())
    }
    
    fun largestBlockCount(): ULong {
        
        val returnVal = lib.WsRunResult_largest_block_count(handle);
        return (returnVal.toULong())
    }
    
    /** Number of builds held in this result (indices `0..build_count()`
    *are valid for every per-index accessor below).
    */
    fun buildCount(): UInt {
        
        val returnVal = lib.WsRunResult_build_count(handle);
        return (returnVal.toUInt())
    }
    
    /** The build's stable id (hex), stable across re-runs against the
    *same source under the same config, absent a prior-snapshot match.
    */
    fun stableIdHex(index: UInt): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.WsRunResult_stable_id_hex(handle, FFIUint32(index), write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The build's content fingerprint, as 32 lowercase hex digits (u128,
    *big-endian).
    */
    fun fingerprintHex(index: UInt): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.WsRunResult_fingerprint_hex(handle, FFIUint32(index), write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** `0` = Confident, `1` = Probable, `2` = Debris.
    */
    fun tierOf(index: UInt): Result<UByte> {
        
        val returnVal = lib.WsRunResult_tier_of(handle, FFIUint32(index));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal.toUByte()).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun blockCountOf(index: UInt): Result<ULong> {
        
        val returnVal = lib.WsRunResult_block_count_of(handle, FFIUint32(index));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal.toULong()).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The build's world-space bounding box minimum (inclusive).
    */
    fun bboxMinOf(index: UInt): Result<BlockPos> {
        
        val returnVal = lib.WsRunResult_bbox_min_of(handle, FFIUint32(index));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = BlockPos.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The build's world-space bounding box maximum (inclusive).
    */
    fun bboxMaxOf(index: UInt): Result<BlockPos> {
        
        val returnVal = lib.WsRunResult_bbox_max_of(handle, FFIUint32(index));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val returnStruct = BlockPos.fromNative(nativeOkVal)
            return returnStruct.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Save the build's schematic to a file, picking the format from the
    *file extension — same serializer as
    *[Schematic::save_to_file](super::super::schematic::ffi::Schematic::save_to_file).
    *Not available in JS: the WASM build has no filesystem.
    */
    fun writeSchemTo(index: UInt, path: String): Result<Unit> {
        val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
        
        val returnVal = lib.WsRunResult_write_schem_to(handle, FFIUint32(index), pathSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            pathSliceMemory.close()
        }
    }

}