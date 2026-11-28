package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ScatterFaceLib: Library {
}
/** Face direction for surface scatter rules.
*/
enum class ScatterFace {
    Top,
    Bottom,
    North,
    South,
    East,
    West;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<ScatterFaceLib> = ScatterFaceLib::class.java
        internal val lib: ScatterFaceLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): ScatterFace {
            return ScatterFace.entries[native]
        }

        fun default(): ScatterFace {
            return Top
        }
    }
}
