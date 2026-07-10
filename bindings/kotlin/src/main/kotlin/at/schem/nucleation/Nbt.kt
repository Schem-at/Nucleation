package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface NbtLib: Library {
    fun Nbt_destroy(handle: Pointer)
    fun Nbt_text_build(s: Slice, color: Slice, bold: Int, italic: Int, write: Pointer): ResultUnitInt
    fun Nbt_chest_build(itemsJson: Slice, name: Slice, write: Pointer): ResultUnitInt
    fun Nbt_sign_build(frontJson: Slice, backJson: Slice, color: Slice, glowing: Boolean, waxed: Boolean, write: Pointer): ResultUnitInt
}
/** Namespace type for the free-standing NBT builder helpers (the old
*`nbt_text_build` / `nbt_chest_build` / `nbt_sign_build`), following the
*static-methods-on-a-dummy-opaque pattern.
*/
class Nbt internal constructor (
    internal val handle: Pointer,
    // These ensure that anything that is borrowed is kept alive and not cleaned
    // up by the garbage collector.
    internal val selfEdges: List<Any>,
    internal var owned: Boolean,
)  {

    init {
        if (this.owned) {
            this.registerCleaner()
        }
    }

    private class NbtCleaner(val handle: Pointer, val lib: NbtLib) : Runnable {
        override fun run() {
            lib.Nbt_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Nbt.NbtCleaner(handle, Nbt.lib));
    }

    companion object {
        internal val libClass: Class<NbtLib> = NbtLib::class.java
        internal val lib: NbtLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Build a Minecraft JSON text-component string.
        *
        *`color` may be empty (no color). `bold` and `italic` use `-1` for
        *unset, `0` for false, nonzero for true.
        */
        fun textBuild(s: String, color: String, bold: Int, italic: Int): Result<String> {
            val sSliceMemory = PrimitiveArrayTools.borrowUtf8(s)
            val colorSliceMemory = PrimitiveArrayTools.borrowUtf8(color)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Nbt_text_build(sSliceMemory.slice, colorSliceMemory.slice, bold, italic, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sSliceMemory.close()
                colorSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Build a chest-NBT SNBT string for use as the `{...}` portion of a block
        *string.
        *
        *`items_json` is a JSON array of `{"id": string, "count"?: int,
        *"slot"?: int}` entries (may be empty or `[]`); a missing/non-positive
        *`count` defaults to 1, a missing/negative `slot` auto-assigns
        *positionally. `name` is an optional plain-text custom name (empty = no
        *`CustomName`); it is wrapped in a JSON text component automatically
        *unless it already starts with `{`.
        */
        fun chestBuild(itemsJson: String, name: String): Result<String> {
            val itemsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(itemsJson)
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Nbt_chest_build(itemsJsonSliceMemory.slice, nameSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                itemsJsonSliceMemory.close()
                nameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Build a modern (1.20+) sign-NBT SNBT string.
        *
        *`front_json` and `back_json` are JSON arrays of up to 4 line strings
        *(either may be empty or `[]`). Each line may be a plain string
        *(auto-wrapped via `text_build`) or an already-built JSON component
        *(starts with `{`). `color` is the dye color string (empty defaults to
        *`"black"`).
        */
        fun signBuild(frontJson: String, backJson: String, color: String, glowing: Boolean, waxed: Boolean): Result<String> {
            val frontJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(frontJson)
            val backJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(backJson)
            val colorSliceMemory = PrimitiveArrayTools.borrowUtf8(color)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Nbt_sign_build(frontJsonSliceMemory.slice, backJsonSliceMemory.slice, colorSliceMemory.slice, glowing, waxed, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                frontJsonSliceMemory.close()
                backJsonSliceMemory.close()
                colorSliceMemory.close()
            }
        }
    }

}