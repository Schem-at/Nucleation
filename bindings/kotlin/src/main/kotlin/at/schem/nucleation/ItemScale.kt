package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ItemScaleLib: Library {
}

internal class ItemScaleNative: Structure(), Structure.ByValue {
    @JvmField
    internal var x: Float = 0.0F;
    @JvmField
    internal var y: Float = 0.0F;
    @JvmField
    internal var z: Float = 0.0F;

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("x", "y", "z")
    }
}




internal class OptionItemScaleNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: ItemScaleNative = ItemScaleNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): ItemScaleNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: ItemScaleNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: ItemScaleNative): OptionItemScaleNative {
            return OptionItemScaleNative(value, 1)
        }

        internal fun none(): OptionItemScaleNative {
            return OptionItemScaleNative(ItemScaleNative(), 0)
        }
    }

}

/** Non-uniform model scale factors.
*/
class ItemScale (var x: Float, var y: Float, var z: Float) {
    companion object {

        internal val libClass: Class<ItemScaleLib> = ItemScaleLib::class.java
        internal val lib: ItemScaleLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(ItemScaleNative::class.java).toLong()

        internal fun fromNative(nativeStruct: ItemScaleNative): ItemScale {
            val x: Float = nativeStruct.x
            val y: Float = nativeStruct.y
            val z: Float = nativeStruct.z

            return ItemScale(x, y, z)
        }

    }
    internal fun toNative(): ItemScaleNative {
        var native = ItemScaleNative()
        native.x = this.x
        native.y = this.y
        native.z = this.z
        return native
    }

}