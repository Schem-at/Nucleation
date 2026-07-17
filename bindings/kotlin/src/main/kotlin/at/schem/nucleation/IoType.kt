package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface IoTypeLib: Library {
    fun IoType_destroy(handle: Pointer)
    fun IoType_unsigned_int(bits: FFIUint32): Pointer
    fun IoType_signed_int(bits: FFIUint32): Pointer
    fun IoType_float32(): Pointer
    fun IoType_boolean(): Pointer
    fun IoType_ascii(chars: FFIUint32): Pointer
}
/** The wire type of a circuit input/output (PORTING rule 10).
*/
class IoType internal constructor (
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

    private class IoTypeCleaner(val handle: Pointer, val lib: IoTypeLib) : Runnable {
        override fun run() {
            lib.IoType_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, IoType.IoTypeCleaner(handle, IoType.lib));
    }

    companion object {
        internal val libClass: Class<IoTypeLib> = IoTypeLib::class.java
        internal val lib: IoTypeLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Unsigned integer of `bits` bits (LSB-first bit order).
        */
        fun unsignedInt(bits: UInt): IoType {
            
            val returnVal = lib.IoType_unsigned_int(FFIUint32(bits));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = IoType(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Signed integer of `bits` bits (two's complement, LSB-first).
        */
        fun signedInt(bits: UInt): IoType {
            
            val returnVal = lib.IoType_signed_int(FFIUint32(bits));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = IoType(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** 32-bit IEEE 754 float (crosses the wire as its 32 raw bits).
        */
        fun float32(): IoType {
            
            val returnVal = lib.IoType_float32();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = IoType(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Single boolean (1 bit).
        */
        fun boolean(): IoType {
            
            val returnVal = lib.IoType_boolean();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = IoType(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Fixed-length ASCII string of `chars` characters (8 bits per char;
        *shorter strings are zero-padded, longer ones truncated).
        */
        fun ascii(chars: UInt): IoType {
            
            val returnVal = lib.IoType_ascii(FFIUint32(chars));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = IoType(handle, selfEdges, true)
            return returnOpaque
        }
    }

}