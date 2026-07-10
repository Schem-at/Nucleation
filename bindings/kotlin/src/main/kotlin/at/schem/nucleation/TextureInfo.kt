package at.schem.nucleation

import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface TextureInfoLib: Library {
}

internal class TextureInfoNative: Structure(), Structure.ByValue {
    @JvmField
    internal var width: FFIUint32 = FFIUint32();
    @JvmField
    internal var height: FFIUint32 = FFIUint32();
    @JvmField
    internal var animated: Byte = 0;
    @JvmField
    internal var frameCount: FFIUint32 = FFIUint32();

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("width", "height", "animated", "frameCount")
    }
}




internal class OptionTextureInfoNative constructor(): Structure(), Structure.ByValue {
    @JvmField
    internal var value: TextureInfoNative = TextureInfoNative()

    @JvmField
    internal var isOk: Byte = 0

    // Define the fields of the struct
    override fun getFieldOrder(): List<String> {
        return listOf("value", "isOk")
    }

    internal fun option(): TextureInfoNative? {
        if (isOk == 1.toByte()) {
            return value
        } else {
            return null
        }
    }


    constructor(value: TextureInfoNative, isOk: Byte): this() {
        this.value = value
        this.isOk = isOk
    }

    companion object {
        internal fun some(value: TextureInfoNative): OptionTextureInfoNative {
            return OptionTextureInfoNative(value, 1)
        }

        internal fun none(): OptionTextureInfoNative {
            return OptionTextureInfoNative(TextureInfoNative(), 0)
        }
    }

}

/** Texture size/animation metadata for one texture in a resource pack.
*/
class TextureInfo (var width: UInt, var height: UInt, var animated: Boolean, var frameCount: UInt) {
    companion object {

        internal val libClass: Class<TextureInfoLib> = TextureInfoLib::class.java
        internal val lib: TextureInfoLib = Native.load("nucleation", libClass)
        val NATIVESIZE: Long = Native.getNativeSize(TextureInfoNative::class.java).toLong()

        internal fun fromNative(nativeStruct: TextureInfoNative): TextureInfo {
            val width: UInt = nativeStruct.width.toUInt()
            val height: UInt = nativeStruct.height.toUInt()
            val animated: Boolean = nativeStruct.animated > 0
            val frameCount: UInt = nativeStruct.frameCount.toUInt()

            return TextureInfo(width, height, animated, frameCount)
        }

    }
    internal fun toNative(): TextureInfoNative {
        var native = TextureInfoNative()
        native.width = FFIUint32(this.width)
        native.height = FFIUint32(this.height)
        native.animated = if (this.animated) 1 else 0
        native.frameCount = FFIUint32(this.frameCount)
        return native
    }

}