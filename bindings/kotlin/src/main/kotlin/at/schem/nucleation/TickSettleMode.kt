package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface TickSettleModeLib: Library {
}
/** How the loaded structure is settled before tick 0.
*/
enum class TickSettleMode {
    Placement,
    Quiet,
    InWorld;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<TickSettleModeLib> = TickSettleModeLib::class.java
        internal val lib: TickSettleModeLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): TickSettleMode {
            return TickSettleMode.entries[native]
        }

        fun default(): TickSettleMode {
            return Placement
        }
    }
}
