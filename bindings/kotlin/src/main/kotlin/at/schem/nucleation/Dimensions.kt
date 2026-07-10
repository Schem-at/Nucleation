package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface DimensionsLib: Library {
}

internal class DimensionsNative: Structure(), Structure.ByValue {
    @JvmField
    internal var x: Int = 0;
    @JvmField
    internal var y: Int = 0;
    @JvmField
    internal var z: Int = 0;

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("x", "y", "z")
    }
}




internal class OptionDimensionsNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: DimensionsNative = DimensionsNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): DimensionsNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: DimensionsNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: DimensionsNative): OptionDimensionsNative {
            return OptionDimensionsNative(value, 1)
        }

        internal fun none(): OptionDimensionsNative {
            return OptionDimensionsNative(DimensionsNative(), 0)
        }
    }

}

class Dimensions (var x: Int, var y: Int, var z: Int) {
    companion object {

        internal val libClass: Class<DimensionsLib> = DimensionsLib::class.java
        internal val lib: DimensionsLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(DimensionsNative::class.java).toLong()

        internal fun fromNative(nativeStruct: DimensionsNative): Dimensions {
            val x: Int = nativeStruct.x
            val y: Int = nativeStruct.y
            val z: Int = nativeStruct.z

            return Dimensions(x, y, z)
        }

    }
    internal fun toNative(): DimensionsNative {
        var native = DimensionsNative()
        native.x = this.x
        native.y = this.y
        native.z = this.z
        return native
    }

}