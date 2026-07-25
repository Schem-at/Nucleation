package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface GeneratedChunkCoverageLib: Library {
}
/** Coverage of a generated chunk by its source graph.
*/
enum class GeneratedChunkCoverage {
    Complete,
    Partial,
    Outside;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<GeneratedChunkCoverageLib> = GeneratedChunkCoverageLib::class.java
        internal val lib: GeneratedChunkCoverageLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): GeneratedChunkCoverage {
            return GeneratedChunkCoverage.entries[native]
        }

        fun default(): GeneratedChunkCoverage {
            return Complete
        }
    }
}
