package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface FingerprintLib: Library {
    fun Fingerprint_destroy(handle: Pointer)
    fun Fingerprint_compute(schematic: Pointer, preset: Slice, write: Pointer): ResultUnitInt
    fun Fingerprint_signature_json(schematic: Pointer, preset: Slice, write: Pointer): ResultUnitInt
    fun Fingerprint_footprint_distance(a: Pointer, b: Pointer, preset: Slice): ResultFloatInt
    fun Fingerprint_footprint_json(schematic: Pointer, preset: Slice, write: Pointer): ResultUnitInt
    fun Fingerprint_is_duplicate(a: Pointer, b: Pointer, preset: Slice): ResultByteInt
}
/** Namespace type for the fingerprint free functions (PORTING rule 12).
*/
class Fingerprint internal constructor (
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

    private class FingerprintCleaner(val handle: Pointer, val lib: FingerprintLib) : Runnable {
        override fun run() {
            lib.Fingerprint_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Fingerprint.FingerprintCleaner(handle, Fingerprint.lib));
    }

    companion object {
        internal val libClass: Class<FingerprintLib> = FingerprintLib::class.java
        internal val lib: FingerprintLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** The fingerprint of a schematic for the given preset, as a hex string.
        *Errors with `InvalidArgument` on an unknown preset.
        */
        fun compute(schematic: Schematic, preset: String): Result<String> {
            val presetSliceMemory = PrimitiveArrayTools.borrowUtf8(preset)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Fingerprint_compute(schematic.handle, presetSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                presetSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** The structural signature (JSON) of a schematic for the given preset.
        */
        fun signatureJson(schematic: Schematic, preset: String): Result<String> {
            val presetSliceMemory = PrimitiveArrayTools.borrowUtf8(preset)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Fingerprint_signature_json(schematic.handle, presetSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                presetSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Translation-invariant fuzzy distance between two builds' footprints.
        */
        fun footprintDistance(a: Schematic, b: Schematic, preset: String): Result<Float> {
            val presetSliceMemory = PrimitiveArrayTools.borrowUtf8(preset)
            
            val returnVal = lib.Fingerprint_footprint_distance(a.handle, b.handle, presetSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return (nativeOkVal).ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                presetSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** The schematic's translation/scale-invariant FFT shape footprint as a
        *JSON array of floats.
        */
        fun footprintJson(schematic: Schematic, preset: String): Result<String> {
            val presetSliceMemory = PrimitiveArrayTools.borrowUtf8(preset)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Fingerprint_footprint_json(schematic.handle, presetSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    
                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                presetSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Whether two schematics share the same fingerprint for the given preset.
        */
        fun isDuplicate(a: Schematic, b: Schematic, preset: String): Result<Boolean> {
            val presetSliceMemory = PrimitiveArrayTools.borrowUtf8(preset)
            
            val returnVal = lib.Fingerprint_is_duplicate(a.handle, b.handle, presetSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    return (nativeOkVal > 0).ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                presetSliceMemory.close()
            }
        }
    }

}