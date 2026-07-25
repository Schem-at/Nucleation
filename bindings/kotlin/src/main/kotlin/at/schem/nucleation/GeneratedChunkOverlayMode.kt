package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface GeneratedChunkOverlayModeLib: Library {
}
/** How a composite layer treats non-air blocks already emitted by earlier layers.
*/
enum class GeneratedChunkOverlayMode {
    Replace,
    KeepExisting;

    fun toNative(): Int {
        return this.ordinal
    }


    companion object {
        internal val libClass: Class<GeneratedChunkOverlayModeLib> = GeneratedChunkOverlayModeLib::class.java
        internal val lib: GeneratedChunkOverlayModeLib = Native.load("nucleation", libClass)
        fun fromNative(native: Int): GeneratedChunkOverlayMode {
            return GeneratedChunkOverlayMode.entries[native]
        }

        fun default(): GeneratedChunkOverlayMode {
            return Replace
        }
    }
}
