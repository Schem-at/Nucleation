package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ValueLib: Library {
    fun Value_destroy(handle: Pointer)
    fun Value_from_u32(v: FFIUint32): Pointer
    fun Value_from_i32(v: Int): Pointer
    fun Value_from_f32(v: Float): Pointer
    fun Value_from_bool(v: Boolean): Pointer
    fun Value_from_string(s: Slice): ResultPointerInt
    fun Value_as_u32(handle: Pointer): ResultFFIUint32Int
    fun Value_as_i32(handle: Pointer): ResultIntInt
    fun Value_as_f32(handle: Pointer): ResultFloatInt
    fun Value_as_bool(handle: Pointer): ResultByteInt
    fun Value_as_string(handle: Pointer, write: Pointer): ResultUnitInt
    fun Value_type_name(handle: Pointer, write: Pointer): Unit
}
/** A typed circuit value (payload-carrying enum; PORTING rule 10).
*/
class Value internal constructor (
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

    private class ValueCleaner(val handle: Pointer, val lib: ValueLib) : Runnable {
        override fun run() {
            lib.Value_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Value.ValueCleaner(handle, Value.lib));
    }

    companion object {
        internal val libClass: Class<ValueLib> = ValueLib::class.java
        internal val lib: ValueLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        fun fromU32(v: UInt): Value {
            
            val returnVal = lib.Value_from_u32(FFIUint32(v));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Value(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun fromI32(v: Int): Value {
            
            val returnVal = lib.Value_from_i32(v);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Value(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun fromF32(v: Float): Value {
            
            val returnVal = lib.Value_from_f32(v);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Value(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun fromBool(v: Boolean): Value {
            
            val returnVal = lib.Value_from_bool(v);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = Value(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        fun fromString(s: String): Result<Value> {
            val sSliceMemory = PrimitiveArrayTools.borrowUtf8(s)
            
            val returnVal = lib.Value_from_string(sSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = Value(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                sSliceMemory.close()
            }
        }
    }
    
    fun asU32(): Result<UInt> {
        
        val returnVal = lib.Value_as_u32(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal.toUInt()).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun asI32(): Result<Int> {
        
        val returnVal = lib.Value_as_i32(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun asF32(): Result<Float> {
        
        val returnVal = lib.Value_as_f32(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    fun asBool(): Result<Boolean> {
        
        val returnVal = lib.Value_as_bool(handle);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return (nativeOkVal > 0).ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The string payload; fails if this is not a string value.
    */
    fun asString(): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Value_as_string(handle, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            
            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }
    
    /** The type name (e.g. "u32", "bool", "string").
    */
    fun typeName(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.Value_type_name(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }

}