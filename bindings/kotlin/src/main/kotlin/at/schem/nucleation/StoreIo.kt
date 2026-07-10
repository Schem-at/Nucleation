package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface StoreIoLib: Library {
    fun StoreIo_destroy(handle: Pointer)
    fun StoreIo_open(uri: Slice): ResultPointerInt
    fun StoreIo_save(schematic: Pointer, uri: Slice, version: Slice): ResultUnitInt
    fun StoreIo_export_settings_schema(format: Slice, write: Pointer): ResultUnitInt
    fun StoreIo_import_settings_schema(format: Slice, write: Pointer): ResultUnitInt
    fun StoreIo_supported_import_formats(write: Pointer): ResultUnitInt
    fun StoreIo_supported_export_formats(write: Pointer): ResultUnitInt
    fun StoreIo_format_versions(format: Slice, write: Pointer): ResultUnitInt
    fun StoreIo_default_format_version(format: Slice, write: Pointer): ResultUnitInt
}
/** Namespace type for the URI-based transparent I/O and format-manager
*queries (PORTING rule 12).
*/
class StoreIo internal constructor (
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

    private class StoreIoCleaner(val handle: Pointer, val lib: StoreIoLib) : Runnable {
        override fun run() {
            lib.StoreIo_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, StoreIo.StoreIoCleaner(handle, StoreIo.lib));
    }

    companion object {
        internal val libClass: Class<StoreIoLib> = StoreIoLib::class.java
        internal val lib: StoreIoLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Open a schematic from a URI: a local path, `file://...`, or
        *`s3://bucket/key.schem`. The format is auto-detected from the URI's
        *extension. Single-string URIs for `redis://`, `postgres://`, and
        *`mem://` are rejected by the core resolver; use `Store::open_schematic`
        *with an explicit store for those backends.
        */
        fun open(uri: String): Result<Schematic> {
            val uriSliceMemory = PrimitiveArrayTools.borrowUtf8(uri)
            
            val returnVal = lib.StoreIo_open(uriSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Schematic(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                uriSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Save a schematic to a URI: a local path, `file://...`, or
        *`s3://bucket/key.schem`. The format is auto-detected from the URI's
        *extension; `version` selects the format version (empty string =
        *format default). Single-string URIs for `redis://`, `postgres://`, and
        *`mem://` are rejected by the core resolver; use `Store::save_schematic`
        *with an explicit store for those backends.
        */
        fun save(schematic: Schematic, uri: String, version: String): Result<Unit> {
            val uriSliceMemory = PrimitiveArrayTools.borrowUtf8(uri)
            val versionSliceMemory = PrimitiveArrayTools.borrowUtf8(version)
            
            val returnVal = lib.StoreIo_save(schematic.handle, uriSliceMemory.slice, versionSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return Unit.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                uriSliceMemory.close()
                versionSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** The JSON schema describing the export settings of `format`. Errors
        *with `NotFound` for an unknown format.
        */
        fun exportSettingsSchema(format: String): Result<String> {
            val formatSliceMemory = PrimitiveArrayTools.borrowUtf8(format)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.StoreIo_export_settings_schema(formatSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                formatSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** The JSON schema describing the import settings of `format`. Errors
        *with `NotFound` for an unknown format.
        */
        fun importSettingsSchema(format: String): Result<String> {
            val formatSliceMemory = PrimitiveArrayTools.borrowUtf8(format)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.StoreIo_import_settings_schema(formatSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                formatSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** The supported import formats, written as a JSON array string.
        */
        fun supportedImportFormats(): Result<String> {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.StoreIo_supported_import_formats(write);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** The supported export formats, written as a JSON array string.
        */
        fun supportedExportFormats(): Result<String> {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.StoreIo_supported_export_formats(write);
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                
                val returnString = DW.writeToString(write)
                return returnString.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        }
        @JvmStatic
        
        /** The known versions of an export format, written as a JSON array string
        *(empty array for an unknown format, matching the old ABI).
        */
        fun formatVersions(format: String): Result<String> {
            val formatSliceMemory = PrimitiveArrayTools.borrowUtf8(format)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.StoreIo_format_versions(formatSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                formatSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** The default version of an export format. Errors with `NotFound` for an
        *unknown format.
        */
        fun defaultFormatVersion(format: String): Result<String> {
            val formatSliceMemory = PrimitiveArrayTools.borrowUtf8(format)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.StoreIo_default_format_version(formatSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                formatSliceMemory.close()
            }
        }
    }

}