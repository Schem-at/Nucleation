package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface SchematicBuilderLib: Library {
    fun SchematicBuilder_destroy(handle: Pointer)
    fun SchematicBuilder_create(): Pointer
    fun SchematicBuilder_from_template(template: Slice): ResultPointerInt
    fun SchematicBuilder_name(handle: Pointer, name: Slice): ResultUnitInt
    fun SchematicBuilder_map(handle: Pointer, ch: Slice, block: Slice): ResultUnitInt
    fun SchematicBuilder_layers(handle: Pointer, layersJson: Slice): ResultUnitInt
    fun SchematicBuilder_layer(handle: Pointer, rowsJson: Slice): ResultUnitInt
    fun SchematicBuilder_palette(handle: Pointer, pairsJson: Slice): ResultUnitInt
    fun SchematicBuilder_offset(handle: Pointer, x: Int, y: Int, z: Int): ResultUnitInt
    fun SchematicBuilder_use_standard_palette(handle: Pointer): ResultUnitInt
    fun SchematicBuilder_use_minimal_palette(handle: Pointer): ResultUnitInt
    fun SchematicBuilder_use_compact_palette(handle: Pointer): ResultUnitInt
    fun SchematicBuilder_validate(handle: Pointer): ResultUnitInt
    fun SchematicBuilder_to_template(handle: Pointer, write: Pointer): ResultUnitInt
    fun SchematicBuilder_build(handle: Pointer): ResultPointerInt
}
/** Fluent builder for schematics from character-mapped text layers.
*
*`build` is consuming (PORTING rule 11): the inner builder is held in an
*`Option` and taken on `build`; every method afterwards returns
*`AlreadyConsumed`.
*/
class SchematicBuilder internal constructor (
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

    private class SchematicBuilderCleaner(val handle: Pointer, val lib: SchematicBuilderLib) : Runnable {
        override fun run() {
            lib.SchematicBuilder_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, SchematicBuilder.SchematicBuilderCleaner(handle, SchematicBuilder.lib));
    }

    companion object {
        internal val libClass: Class<SchematicBuilderLib> = SchematicBuilderLib::class.java
        internal val lib: SchematicBuilderLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun create(): SchematicBuilder {
            
            val returnVal = lib.SchematicBuilder_create();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = SchematicBuilder(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Parse a builder from the canonical template text format.
        */
        fun fromTemplate(template: String): Result<SchematicBuilder> {
            val templateSliceMemory = PrimitiveArrayTools.borrowUtf8(template)
            
            val returnVal = lib.SchematicBuilder_from_template(templateSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = SchematicBuilder(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                templateSliceMemory.close()
            }
        }
    }
    
    /** Set the schematic name.
    */
    fun name(name: String): Result<Unit> {
        val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
        
        val returnVal = lib.SchematicBuilder_name(handle, nameSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            nameSliceMemory.close()
        }
    }
    
    /** Map a palette character to a block string. `ch` must contain exactly
    *one character (its first char is used).
    */
    fun map(ch: String, block: String): Result<Unit> {
        val chSliceMemory = PrimitiveArrayTools.borrowUtf8(ch)
        val blockSliceMemory = PrimitiveArrayTools.borrowUtf8(block)
        
        val returnVal = lib.SchematicBuilder_map(handle, chSliceMemory.slice, blockSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            chSliceMemory.close()
            blockSliceMemory.close()
        }
    }
    
    /** Append layers. `layers_json` is a JSON array of arrays of row strings,
    *e.g. `[["ab","cd"],["ef","gh"]]`.
    */
    fun layers(layersJson: String): Result<Unit> {
        val layersJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(layersJson)
        
        val returnVal = lib.SchematicBuilder_layers(handle, layersJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            layersJsonSliceMemory.close()
        }
    }
    
    /** Append a single layer of rows. `rows_json` is a JSON array of strings,
    *e.g. `["abc", "def"]`. Equivalent to a one-element layers array.
    */
    fun layer(rowsJson: String): Result<Unit> {
        val rowsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(rowsJson)
        
        val returnVal = lib.SchematicBuilder_layer(handle, rowsJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            rowsJsonSliceMemory.close()
        }
    }
    
    /** Bulk-register palette characters. `pairs_json` is a JSON array of
    *`[char, block]` two-element arrays, e.g.
    *`[["c", "minecraft:gray_concrete"], [" ", "minecraft:air"]]`.
    */
    fun palette(pairsJson: String): Result<Unit> {
        val pairsJsonSliceMemory = PrimitiveArrayTools.borrowUtf8(pairsJson)
        
        val returnVal = lib.SchematicBuilder_palette(handle, pairsJsonSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            pairsJsonSliceMemory.close()
        }
    }
    
    /** Set the build offset applied to every placed block.
    */
    fun offset(x: Int, y: Int, z: Int): Result<Unit> {
        
        val returnVal = lib.SchematicBuilder_offset(handle, x, y, z);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun useStandardPalette(): Result<Unit> {
        
        val returnVal = lib.SchematicBuilder_use_standard_palette(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun useMinimalPalette(): Result<Unit> {
        
        val returnVal = lib.SchematicBuilder_use_minimal_palette(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun useCompactPalette(): Result<Unit> {
        
        val returnVal = lib.SchematicBuilder_use_compact_palette(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Run pre-build validation without consuming the builder.
    */
    fun validate(): Result<Unit> {
        
        val returnVal = lib.SchematicBuilder_validate(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Serialize the builder back into the canonical template format.
    */
    fun toTemplate(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.SchematicBuilder_to_template(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** Build the schematic. Consuming: the builder cannot be reused afterwards
    *(subsequent calls return `AlreadyConsumed`), including after a failed
    *build.
    */
    fun build(): Result<Schematic> {
        
        val returnVal = lib.SchematicBuilder_build(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            val selfEdges: List<Any> = listOf()
            val handle = nativeOkVal 
            val returnOpaque = Schematic(handle, selfEdges, true)
            return returnOpaque.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

}