package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface BlockPosLib: Library {
}

internal class BlockPosNative: Structure(), Structure.ByValue {
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




internal class OptionBlockPosNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: BlockPosNative = BlockPosNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): BlockPosNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: BlockPosNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: BlockPosNative): OptionBlockPosNative {
            return OptionBlockPosNative(value, 1)
        }

        internal fun none(): OptionBlockPosNative {
            return OptionBlockPosNative(BlockPosNative(), 0)
        }
    }

}

class BlockPos (var x: Int, var y: Int, var z: Int) {
    companion object {

        internal val libClass: Class<BlockPosLib> = BlockPosLib::class.java
        internal val lib: BlockPosLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(BlockPosNative::class.java).toLong()

        internal fun fromNative(nativeStruct: BlockPosNative): BlockPos {
            val x: Int = nativeStruct.x
            val y: Int = nativeStruct.y
            val z: Int = nativeStruct.z

            return BlockPos(x, y, z)
        }

    }
    internal fun toNative(): BlockPosNative {
        var native = BlockPosNative()
        native.x = this.x
        native.y = this.y
        native.z = this.z
        return native
    }

}