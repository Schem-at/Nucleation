package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface BlocksLib: Library {
    fun Blocks_destroy(handle: Pointer)
    fun Blocks_get_json(id: Slice, write: Pointer): ResultUnitInt
    fun Blocks_ids_json(write: Pointer): Unit
    fun Blocks_by_color_json(r: FFIUint8, g: FFIUint8, b: FFIUint8, maxDistance: Float, write: Pointer): ResultUnitInt
    fun Blocks_by_tag_json(tag: Slice, write: Pointer): ResultUnitInt
    fun Blocks_by_kind_json(kind: Slice, write: Pointer): ResultUnitInt
    fun Blocks_variants_of_json(baseId: Slice, write: Pointer): ResultUnitInt
    fun Blocks_tags_json(write: Pointer): Unit
    fun Blocks_states_json(id: Slice, write: Pointer): ResultUnitInt
    fun Blocks_count(): FFISizet
}
/** Namespace for read-only queries over the built-in block table
*(Java 26.2 blocks + official semantics extracted from the vanilla
*jars: definition kinds, base-block links, block tags, model
*geometry). All list results are JSON array strings sorted by id;
*block/tag/kind arguments accept both `minecraft:`-prefixed and
*short forms (`minecraft:oak_stairs` / `oak_stairs`).
*/
class Blocks internal constructor (
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

    private class BlocksCleaner(val handle: Pointer, val lib: BlocksLib) : Runnable {
        override fun run() {
            lib.Blocks_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Blocks.BlocksCleaner(handle, Blocks.lib));
    }

    companion object {
        internal val libClass: Class<BlocksLib> = BlocksLib::class.java
        internal val lib: BlocksLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Full facts for one block as a JSON object:
        *`{id, kind, base_block, tags: [...], full_cube, transparent,
        *color: [r, g, b] | null, properties: {name: [values...]},
        *default_state: {name: value}}`. `kind` is the official
        *definition kind (`minecraft:stair`, plain full blocks are
        *`minecraft:block`); `base_block` is the block this one is a
        *shape variant of (or `null`); `color` is the texture-derived
        *average RGB. Errors with `NotFound` for unknown ids.
        */
        fun getJson(id: String): Result<String> {
            val idSliceMemory = PrimitiveArrayTools.borrowUtf8(id)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Blocks_get_json(idSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                idSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** All known block ids as a sorted JSON array string.
        */
        fun idsJson(): String {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Blocks_ids_json(write);
            
            val returnString = DW.writeToString(write)
            return returnString
        }
        @JvmStatic
        
        /** Ids of every block carrying the vanilla block tag, as a sorted
        *Blocks whose measured texture color is within `max_distance`
        *(Oklab; ~0.05 = same color family, ~0.15 = generous) of the given
        *RGB, as a JSON array of `{"id", "color": [r,g,b], "distance"}`
        *sorted nearest-first. Blocks without color data never match.
        */
        fun byColorJson(r: UByte, g: UByte, b: UByte, maxDistance: Float): Result<String> {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Blocks_by_color_json(FFIUint8(r), FFIUint8(g), FFIUint8(b), maxDistance, write);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** JSON array string (`[]` for unknown tags). Accepts
        *`minecraft:wool` and short `wool` forms, including nested paths
        *like `mineable/pickaxe`.
        */
        fun byTagJson(tag: String): Result<String> {
            val tagSliceMemory = PrimitiveArrayTools.borrowUtf8(tag)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Blocks_by_tag_json(tagSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                tagSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Ids of every block of the given official definition kind
        *(`minecraft:stair`, `minecraft:slab`, `minecraft:door`, ...), as
        *a sorted JSON array string (`[]` for unknown kinds).
        */
        fun byKindJson(kind: String): Result<String> {
            val kindSliceMemory = PrimitiveArrayTools.borrowUtf8(kind)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Blocks_by_kind_json(kindSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                kindSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** The base block followed by all its shape variants — blocks whose
        *`base_block` is `base_id` (stairs, slabs, walls, fences of the
        *base) — as a JSON array string. The base itself is always first;
        *variants follow sorted by id. Errors with `NotFound` for unknown
        *base ids.
        */
        fun variantsOfJson(baseId: String): Result<String> {
            val baseIdSliceMemory = PrimitiveArrayTools.borrowUtf8(baseId)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Blocks_variants_of_json(baseIdSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                baseIdSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** All known vanilla block tag names as a sorted JSON array string
        *(`minecraft:`-prefixed, e.g. `minecraft:wool`).
        */
        fun tagsJson(): String {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Blocks_tags_json(write);
            
            val returnString = DW.writeToString(write)
            return returnString
        }
        @JvmStatic
        
        /** Every property-value combination of the block as a JSON array of
        *`{prop: value}` objects (a single `{}` entry for property-less
        *blocks). Errors with `NotFound` for unknown ids and with
        *`InvalidArgument` if the combination count exceeds 4096 (guard
        *against pathological output; the current data tops out at 1350
        *for `minecraft:note_block`).
        */
        fun statesJson(id: String): Result<String> {
            val idSliceMemory = PrimitiveArrayTools.borrowUtf8(id)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Blocks_states_json(idSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                idSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Total number of blocks in the table.
        */
        fun count(): ULong {
            
            val returnVal = lib.Blocks_count();
            return (returnVal.toULong())
        }
    }

}