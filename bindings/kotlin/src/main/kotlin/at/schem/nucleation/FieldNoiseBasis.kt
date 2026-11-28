package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FieldNoiseBasisLib: Library {
}
/** Noise basis for procedural 2D/3D noise and FBM.
*/
enum class FieldNoiseBasis {
    ValueNoise,
    Perlin;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<FieldNoiseBasisLib> = FieldNoiseBasisLib::class.java
        internal val lib: FieldNoiseBasisLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): FieldNoiseBasis {
            return FieldNoiseBasis.entries[native]
        }

        fun default(): FieldNoiseBasis {
            return ValueNoise
        }
    }
}
