package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SdfBoundsLib: Library {
}

internal class SdfBoundsNative: Structure(), Structure.ByValue {
    @JvmField
    internal var minX: Float = 0.0F;
    @JvmField
    internal var minY: Float = 0.0F;
    @JvmField
    internal var minZ: Float = 0.0F;
    @JvmField
    internal var maxX: Float = 0.0F;
    @JvmField
    internal var maxY: Float = 0.0F;
    @JvmField
    internal var maxZ: Float = 0.0F;

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("minX", "minY", "minZ", "maxX", "maxY", "maxZ")
    }
}




internal class OptionSdfBoundsNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: SdfBoundsNative = SdfBoundsNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): SdfBoundsNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: SdfBoundsNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: SdfBoundsNative): OptionSdfBoundsNative {
            return OptionSdfBoundsNative(value, 1)
        }

        internal fun none(): OptionSdfBoundsNative {
            return OptionSdfBoundsNative(SdfBoundsNative(), 0)
        }
    }

}

/** Continuous bounds of a bounded SDF graph.
*/
class SdfBounds (var minX: Float, var minY: Float, var minZ: Float, var maxX: Float, var maxY: Float, var maxZ: Float) {
    companion object {

        internal val libClass: Class<SdfBoundsLib> = SdfBoundsLib::class.java
        internal val lib: SdfBoundsLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(SdfBoundsNative::class.java).toLong()

        internal fun fromNative(nativeStruct: SdfBoundsNative): SdfBounds {
            val minX: Float = nativeStruct.minX
            val minY: Float = nativeStruct.minY
            val minZ: Float = nativeStruct.minZ
            val maxX: Float = nativeStruct.maxX
            val maxY: Float = nativeStruct.maxY
            val maxZ: Float = nativeStruct.maxZ

            return SdfBounds(minX, minY, minZ, maxX, maxY, maxZ)
        }

    }
}
