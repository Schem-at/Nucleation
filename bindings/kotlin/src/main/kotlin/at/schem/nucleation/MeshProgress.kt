package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface MeshProgressLib: Library {
}

internal class MeshProgressNative: Structure(), Structure.ByValue {
    @JvmField
    internal var phase: Int = MeshPhase.default().toNative();
    /** Chunks completed so far.
    */
    @JvmField
    internal var current: FFIUint32 = FFIUint32();
    /** Total chunks to mesh (0 until known).
    */
    @JvmField
    internal var total: FFIUint32 = FFIUint32();

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("phase", "current", "total")
    }
}




internal class OptionMeshProgressNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: MeshProgressNative = MeshProgressNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): MeshProgressNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: MeshProgressNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: MeshProgressNative): OptionMeshProgressNative {
            return OptionMeshProgressNative(value, 1)
        }

        internal fun none(): OptionMeshProgressNative {
            return OptionMeshProgressNative(MeshProgressNative(), 0)
        }
    }

}

/** Snapshot of a [MeshJob]'s progress.
*/
class MeshProgress (var phase: MeshPhase, var current: UInt, var total: UInt) {
    companion object {

        internal val libClass: Class<MeshProgressLib> = MeshProgressLib::class.java
        internal val lib: MeshProgressLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(MeshProgressNative::class.java).toLong()

        internal fun fromNative(nativeStruct: MeshProgressNative): MeshProgress {
            val phase: MeshPhase = MeshPhase.fromNative(nativeStruct.phase)
            val current: UInt = nativeStruct.current.toUInt()
            val total: UInt = nativeStruct.total.toUInt()

            return MeshProgress(phase, current, total)
        }

    }
    internal fun toNative(): MeshProgressNative {
        var native = MeshProgressNative()
        native.phase = this.phase.toNative()
        native.current = FFIUint32(this.current)
        native.total = FFIUint32(this.total)
        return native
    }

}