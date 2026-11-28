package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldChannelLib: Library {
}
/** Built-in voxel channels readable by material gradients, ramps, and conditions.
*/
enum class FieldChannel {
    WorldX,
    WorldY,
    WorldZ,
    Distance,
    DepthBelowSurface,
    Slope,
    NormalY,
    Curvature;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<FieldChannelLib> = FieldChannelLib::class.java
        internal val lib: FieldChannelLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): FieldChannel {
            return FieldChannel.entries[native]
        }

        fun default(): FieldChannel {
            return WorldX
        }
    }
}
