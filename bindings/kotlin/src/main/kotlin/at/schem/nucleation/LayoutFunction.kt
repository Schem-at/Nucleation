package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface LayoutFunctionLib: Library {
    fun LayoutFunction_destroy(handle: Pointer)
    fun LayoutFunction_one_to_one(): Pointer
    fun LayoutFunction_packed4(): Pointer
    fun LayoutFunction_custom(mapping: Slice): ResultPointerInt
    fun LayoutFunction_row_major(rows: FFIUint32, cols: FFIUint32, bitsPerElement: FFIUint32): Pointer
    fun LayoutFunction_column_major(rows: FFIUint32, cols: FFIUint32, bitsPerElement: FFIUint32): Pointer
    fun LayoutFunction_scanline(width: FFIUint32, height: FFIUint32, bitsPerPixel: FFIUint32): Pointer
}
/** Maps logical bits to physical positions (PORTING rule 10).
*/
class LayoutFunction internal constructor (
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

    private class LayoutFunctionCleaner(val handle: Pointer, val lib: LayoutFunctionLib) : Runnable {
        override fun run() {
            lib.LayoutFunction_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, LayoutFunction.LayoutFunctionCleaner(handle, LayoutFunction.lib));
    }

    companion object {
        internal val libClass: Class<LayoutFunctionLib> = LayoutFunctionLib::class.java
        internal val lib: LayoutFunctionLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** One bit per position: signal strength 0 = false, 15 = true.
        *The most common layout for traditional redstone circuits.
        */
        fun oneToOne(): LayoutFunction {
            
            val returnVal = lib.LayoutFunction_one_to_one();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = LayoutFunction(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Four bits per position, packed as a hex nibble (0-15) with the
        *lowest bit first; uses `ceil(bits / 4)` positions (4x denser than
        *one-to-one).
        */
        fun packed4(): LayoutFunction {
            
            val returnVal = lib.LayoutFunction_packed4();
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = LayoutFunction(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Custom bit-to-position mapping.
        */
        fun custom(mapping: UIntArray): Result<LayoutFunction> {
            val mappingSliceMemory = PrimitiveArrayTools.borrow(mapping)
            
            val returnVal = lib.LayoutFunction_custom(mappingSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = LayoutFunction(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                mappingSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** 2D matrix layout, elements laid out row by row; element bits are
        *packed 4 per position (nibbles).
        */
        fun rowMajor(rows: UInt, cols: UInt, bitsPerElement: UInt): LayoutFunction {
            
            val returnVal = lib.LayoutFunction_row_major(FFIUint32(rows), FFIUint32(cols), FFIUint32(bitsPerElement));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = LayoutFunction(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** 2D matrix layout, elements laid out column by column; element bits
        *are packed 4 per position (nibbles).
        */
        fun columnMajor(rows: UInt, cols: UInt, bitsPerElement: UInt): LayoutFunction {
            
            val returnVal = lib.LayoutFunction_column_major(FFIUint32(rows), FFIUint32(cols), FFIUint32(bitsPerElement));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = LayoutFunction(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Screen layout: pixels laid out row by row, left to right; pixel
        *bits are packed 4 per position (nibbles).
        */
        fun scanline(width: UInt, height: UInt, bitsPerPixel: UInt): LayoutFunction {
            
            val returnVal = lib.LayoutFunction_scanline(FFIUint32(width), FFIUint32(height), FFIUint32(bitsPerPixel));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = LayoutFunction(handle, selfEdges, true)
            return returnOpaque
        }
    }

}