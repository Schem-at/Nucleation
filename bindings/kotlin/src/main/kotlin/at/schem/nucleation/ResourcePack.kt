package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ResourcePackLib: Library {
    fun ResourcePack_destroy(handle: Pointer)
    fun ResourcePack_from_bytes(data: Slice): ResultPointerInt
    fun ResourcePack_from_list(list: Pointer): ResultPointerInt
    fun ResourcePack_blockstate_count(handle: Pointer): FFIUint32
    fun ResourcePack_model_count(handle: Pointer): FFIUint32
    fun ResourcePack_texture_count(handle: Pointer): FFIUint32
    fun ResourcePack_namespaces_json(handle: Pointer, write: Pointer): Unit
    fun ResourcePack_list_blockstates_json(handle: Pointer, write: Pointer): Unit
    fun ResourcePack_list_models_json(handle: Pointer, write: Pointer): Unit
    fun ResourcePack_list_textures_json(handle: Pointer, write: Pointer): Unit
    fun ResourcePack_get_blockstate_json(handle: Pointer, name: Slice, write: Pointer): ResultUnitInt
    fun ResourcePack_get_model_json(handle: Pointer, name: Slice, write: Pointer): ResultUnitInt
    fun ResourcePack_get_texture_info(handle: Pointer, name: Slice): ResultTextureInfoNativeInt
    fun ResourcePack_get_texture_pixels_b64(handle: Pointer, name: Slice, write: Pointer): ResultUnitInt
    fun ResourcePack_add_blockstate_json(handle: Pointer, name: Slice, json: Slice): ResultUnitInt
    fun ResourcePack_add_model_json(handle: Pointer, name: Slice, json: Slice): ResultUnitInt
    fun ResourcePack_add_texture(handle: Pointer, name: Slice, width: FFIUint32, height: FFIUint32, pixels: Slice): ResultUnitInt
    fun ResourcePack_player_skin_urls_json(schematic: Pointer, write: Pointer): Unit
    fun ResourcePack_add_player_skin(handle: Pointer, urlOrHash: Slice, png: Slice, write: Pointer): ResultUnitInt
    fun ResourcePack_register_mesh_exporter(handle: Pointer): ResultUnitInt
}
/** A loaded (possibly merged) Minecraft resource pack.
*Wraps [crate::meshing::ResourcePackSource].
*/
class ResourcePack internal constructor (
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

    private class ResourcePackCleaner(val handle: Pointer, val lib: ResourcePackLib) : Runnable {
        override fun run() {
            lib.ResourcePack_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, ResourcePack.ResourcePackCleaner(handle, ResourcePack.lib));
    }

    companion object {
        internal val libClass: Class<ResourcePackLib> = ResourcePackLib::class.java
        internal val lib: ResourcePackLib = Native.load("nucleation", libClass)
        @JvmStatic

        /** Load a resource pack from an in-memory ZIP buffer.
        */
        fun fromBytes(data: UByteArray): Result<ResourcePack> {
            val dataSliceMemory = PrimitiveArrayTools.borrow(data)

            val returnVal = lib.ResourcePack_from_bytes(dataSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = ResourcePack(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                dataSliceMemory.close()
            }
        }
        @JvmStatic

        /** Load and merge multiple resource packs, lowest priority first.
        */
        fun fromList(list: ResourcePackList): Result<ResourcePack> {

            val returnVal = lib.ResourcePack_from_list(list.handle);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val selfEdges: List<Any> = listOf()
                val handle = nativeOkVal
                val returnOpaque = ResourcePack(handle, selfEdges, true)
                return returnOpaque.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic

        /** Skin URLs the player heads in `schematic` need, as a sorted JSON
        *array string. Nucleation never downloads skins: fetch each URL and
        *pass the PNG to `add_player_skin`. Heads whose skin is not
        *registered render with vanilla's default skin for their owner.
        */
        fun playerSkinUrlsJson(schematic: Schematic): String {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.ResourcePack_player_skin_urls_json(schematic.handle, write);

            val returnString = DW.writeToString(write)
            return returnString
        }
    }

    /** Number of blockstate definitions in the pack.
    */
    fun blockstateCount(): UInt {

        val returnVal = lib.ResourcePack_blockstate_count(handle);
        return (returnVal.toUInt())
    }

    /** Number of model definitions in the pack.
    */
    fun modelCount(): UInt {

        val returnVal = lib.ResourcePack_model_count(handle);
        return (returnVal.toUInt())
    }

    /** Number of textures in the pack.
    */
    fun textureCount(): UInt {

        val returnVal = lib.ResourcePack_texture_count(handle);
        return (returnVal.toUInt())
    }

    /** Namespaces present in the pack, as a JSON array string.
    */
    fun namespacesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ResourcePack_namespaces_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** All blockstate identifiers, as a JSON array string.
    */
    fun listBlockstatesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ResourcePack_list_blockstates_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** All model identifiers, as a JSON array string.
    */
    fun listModelsJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ResourcePack_list_models_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** All texture identifiers, as a JSON array string.
    */
    fun listTexturesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ResourcePack_list_textures_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** The blockstate definition JSON for `name`; `NotFound` if absent.
    */
    fun getBlockstateJson(name: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ResourcePack_get_blockstate_json(handle, nameSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }

    /** The model definition JSON for `name`; `NotFound` if absent.
    */
    fun getModelJson(name: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ResourcePack_get_model_json(handle, nameSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }

    /** Texture metadata (size, animation flag, frame count).
    */
    fun getTextureInfo(name: String): Result<TextureInfo> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

        val returnVal = lib.ResourcePack_get_texture_info(handle, nameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                val returnStruct = TextureInfo.fromNative(nativeOkVal)
                return returnStruct.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }

    /** Raw RGBA pixels of a texture, base64-encoded.
    */
    fun getTexturePixelsB64(name: String): Result<String> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ResourcePack_get_texture_pixels_b64(handle, nameSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }

    /** Add (or override) a blockstate definition from a JSON string.
    */
    fun addBlockstateJson(name: String, json: String): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val jsonSliceMemory = PrimitiveArrayTools.borrowUtf8(json)

        val returnVal = lib.ResourcePack_add_blockstate_json(handle, nameSliceMemory.slice, jsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            jsonSliceMemory.close()
        }
    }

    /** Add (or override) a model definition from a JSON string.
    */
    fun addModelJson(name: String, json: String): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val jsonSliceMemory = PrimitiveArrayTools.borrowUtf8(json)

        val returnVal = lib.ResourcePack_add_model_json(handle, nameSliceMemory.slice, jsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            jsonSliceMemory.close()
        }
    }

    /** Add a raw RGBA texture (`pixels` length must be `width * height * 4`).
    */
    fun addTexture(name: String, width: UInt, height: UInt, pixels: UByteArray): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        val pixelsSliceMemory = PrimitiveArrayTools.borrow(pixels)

        val returnVal = lib.ResourcePack_add_texture(handle, nameSliceMemory.slice, FFIUint32(width), FFIUint32(height), pixelsSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
            pixelsSliceMemory.close()
        }
    }

    /** Register a downloaded player skin: `url_or_hash` is a URL from
    *`player_skin_urls_json` (or its texture hash), `png` the 64x64 (or
    *legacy 64x32) skin image. Writes the texture's resource location,
    *`nucleation:skins/<hash>`. `InvalidArgument` if the URL has no hash
    *or the PNG is not a skin.
    */
    fun addPlayerSkin(urlOrHash: String, png: UByteArray): Result<String> {
        val urlOrHashSliceMemory = PrimitiveArrayTools.borrowUtf8(urlOrHash)
        val pngSliceMemory = PrimitiveArrayTools.borrow(png)
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.ResourcePack_add_player_skin(handle, urlOrHashSliceMemory.slice, pngSliceMemory.slice, write);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {

                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            urlOrHashSliceMemory.close()
            pngSliceMemory.close()
        }
    }

    /** Register a MeshExporter with the FormatManager so `save_as("mesh", ...)`
    *works. (Old ABI: `schematic_register_mesh_exporter`.)
    */
    fun registerMeshExporter(): Result<Unit> {

        val returnVal = lib.ResourcePack_register_mesh_exporter(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}
