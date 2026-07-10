package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface RegionBoundsLib: Library {
}

internal class RegionBoundsNative: Structure(), Structure.ByValue {
    @JvmField
    internal var minX: Int = 0;
    @JvmField
    internal var minY: Int = 0;
    @JvmField
    internal var minZ: Int = 0;
    @JvmField
    internal var maxX: Int = 0;
    @JvmField
    internal var maxY: Int = 0;
    @JvmField
    internal var maxZ: Int = 0;

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("minX", "minY", "minZ", "maxX", "maxY", "maxZ")
    }
}




internal class OptionRegionBoundsNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: RegionBoundsNative = RegionBoundsNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): RegionBoundsNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: RegionBoundsNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: RegionBoundsNative): OptionRegionBoundsNative {
            return OptionRegionBoundsNative(value, 1)
        }

        internal fun none(): OptionRegionBoundsNative {
            return OptionRegionBoundsNative(RegionBoundsNative(), 0)
        }
    }

}

/** An inclusive block-coordinate box (a definition region is a union of
*these).
*/
class RegionBounds (var minX: Int, var minY: Int, var minZ: Int, var maxX: Int, var maxY: Int, var maxZ: Int) {
    companion object {

        internal val libClass: Class<RegionBoundsLib> = RegionBoundsLib::class.java
        internal val lib: RegionBoundsLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(RegionBoundsNative::class.java).toLong()

        internal fun fromNative(nativeStruct: RegionBoundsNative): RegionBounds {
            val minX: Int = nativeStruct.minX
            val minY: Int = nativeStruct.minY
            val minZ: Int = nativeStruct.minZ
            val maxX: Int = nativeStruct.maxX
            val maxY: Int = nativeStruct.maxY
            val maxZ: Int = nativeStruct.maxZ

            return RegionBounds(minX, minY, minZ, maxX, maxY, maxZ)
        }

    }
    internal fun toNative(): RegionBoundsNative {
        var native = RegionBoundsNative()
        native.minX = this.minX
        native.minY = this.minY
        native.minZ = this.minZ
        native.maxX = this.maxX
        native.maxY = this.maxY
        native.maxZ = this.maxZ
        return native
    }

}