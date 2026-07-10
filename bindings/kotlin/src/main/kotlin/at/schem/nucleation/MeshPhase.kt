package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MeshPhaseLib: Library {
}
/** Phase of a running [MeshJob].
*/
enum class MeshPhase {
    BuildingAtlas,
    MeshingChunks,
    Complete,
    Failed;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<MeshPhaseLib> = MeshPhaseLib::class.java
        internal val lib: MeshPhaseLib = Native.load("nucleation", libClass) 
        fun fromNative(native: Int): MeshPhase {
            return MeshPhase.entries[native]
        }

        fun default(): MeshPhase {
            return BuildingAtlas
        }
    }
}
