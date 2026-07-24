package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface WsSegmentJobLib: Library {
    fun WsSegmentJob_destroy(handle: Pointer)
    fun WsSegmentJob_create(cellSize: FFIUint32, closingRadius: FFIUint32, minClusterBlocks: FFIUint64, sourceId: Slice, snapshotId: Slice, minY: Int, maxY: Int, extractedAt: Long, matchIou: Float, hardCut: Boolean): ResultPointerInt
}
/** One segmentation run's parameters (the primitive knobs of
*[SegmentJob](crate::world_segment::runner::SegmentJob), plus a
*`hard_cut` flag selecting [PartitionPolicy]). Built once, passed by
*reference into [WsRunResult::run_dir].
*/
class WsSegmentJob internal constructor (
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

    private class WsSegmentJobCleaner(val handle: Pointer, val lib: WsSegmentJobLib) : Runnable {
        override fun run() {
            lib.WsSegmentJob_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, WsSegmentJob.WsSegmentJobCleaner(handle, WsSegmentJob.lib));
    }

    companion object {
        internal val libClass: Class<WsSegmentJobLib> = WsSegmentJobLib::class.java
        internal val lib: WsSegmentJobLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** `algorithm_version` is pinned to
        *[SegConfig::default](crate::world_segment::segment::SegConfig)'s
        *value; `score_config` uses `ScoreConfig::default()`. Neither is
        *exposed as a knob here — construct a `SegmentJob` directly in Rust
        *if you need to override them.
        */
        fun create(cellSize: UInt, closingRadius: UInt, minClusterBlocks: ULong, sourceId: String, snapshotId: String, minY: Int, maxY: Int, extractedAt: Long, matchIou: Float, hardCut: Boolean): Result<WsSegmentJob> {
            val sourceIdSliceMemory = PrimitiveArrayTools.borrowUtf8(sourceId)
            val snapshotIdSliceMemory = PrimitiveArrayTools.borrowUtf8(snapshotId)
            
            val returnVal = lib.WsSegmentJob_create(FFIUint32(cellSize), FFIUint32(closingRadius), FFIUint64(minClusterBlocks), sourceIdSliceMemory.slice, snapshotIdSliceMemory.slice, minY, maxY, extractedAt, matchIou, hardCut);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = WsSegmentJob(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sourceIdSliceMemory.close()
                snapshotIdSliceMemory.close()
            }
        }
    }

}