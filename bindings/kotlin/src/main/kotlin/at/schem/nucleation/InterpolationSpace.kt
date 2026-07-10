package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface InterpolationSpaceLib: Library {
}
/** Color interpolation space for gradient brushes. The old ABI passed this as
*`space: c_int` (`1` = Oklab, anything else = RGB).
*/
enum class InterpolationSpace {
    Rgb,
    Oklab;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<InterpolationSpaceLib> = InterpolationSpaceLib::class.java
        internal val lib: InterpolationSpaceLib = Native.load("nucleation", libClass) 
        fun fromNative(native: Int): InterpolationSpace {
            return InterpolationSpace.entries[native]
        }

        fun default(): InterpolationSpace {
            return Rgb
        }
    }
}
