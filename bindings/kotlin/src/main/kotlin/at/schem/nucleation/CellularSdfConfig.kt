package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface CellularSdfConfigLib: Library {
    fun CellularSdfConfig_destroy(handle: Pointer)
    fun CellularSdfConfig_create(cellSizeX: Int, cellSizeZ: Int, seed: FFIUint64, maxJitterX: Float, maxJitterZ: Float, maxYawDegrees: Float, minScale: Float, maxScale: Float, minYOffset: Int, maxYOffset: Int, presenceNumerator: FFIUint32, presenceDenominator: FFIUint32, featureSalt: FFIUint64): ResultPointerInt
}
/** Immutable hashed-cell variation shared by coordinated SDF source layers.
*/
class CellularSdfConfig internal constructor (
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

    private class CellularSdfConfigCleaner(val handle: Pointer, val lib: CellularSdfConfigLib) : Runnable {
        override fun run() {
            lib.CellularSdfConfig_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, CellularSdfConfig.CellularSdfConfigCleaner(handle, CellularSdfConfig.lib));
    }

    companion object {
        internal val libClass: Class<CellularSdfConfigLib> = CellularSdfConfigLib::class.java
        internal val lib: CellularSdfConfigLib = Native.load("nucleation", libClass)
        @JvmStatic

        /** Validates every field up front, so a config that constructs here is
        *never rejected for its own values by a later `cellular_sdf` call.
        */
        fun create(cellSizeX: Int, cellSizeZ: Int, seed: ULong, maxJitterX: Float, maxJitterZ: Float, maxYawDegrees: Float, minScale: Float, maxScale: Float, minYOffset: Int, maxYOffset: Int, presenceNumerator: UInt, presenceDenominator: UInt, featureSalt: ULong): Result<CellularSdfConfig> {

            val returnVal = lib.CellularSdfConfig_create(cellSizeX, cellSizeZ, FFIUint64(seed), maxJitterX, maxJitterZ, maxYawDegrees, minScale, maxScale, minYOffset, maxYOffset, FFIUint32(presenceNumerator), FFIUint32(presenceDenominator), FFIUint64(featureSalt));
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = CellularSdfConfig(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
    }

}
