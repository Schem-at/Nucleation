package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface OutputConditionLib: Library {
    fun OutputCondition_destroy(handle: Pointer)
    fun OutputCondition_equals(value: Pointer): Pointer
    fun OutputCondition_not_equals(value: Pointer): Pointer
    fun OutputCondition_greater_than(value: Pointer): Pointer
    fun OutputCondition_less_than(value: Pointer): Pointer
    fun OutputCondition_bitwise_and(mask: FFIUint32): Pointer
}
/** A condition on an output value, for `ExecutionMode::until_condition`
*(PORTING rule 10).
*/
class OutputCondition internal constructor (
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

    private class OutputConditionCleaner(val handle: Pointer, val lib: OutputConditionLib) : Runnable {
        override fun run() {
            lib.OutputCondition_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, OutputCondition.OutputConditionCleaner(handle, OutputCondition.lib));
    }

    companion object {
        internal val libClass: Class<OutputConditionLib> = OutputConditionLib::class.java
        internal val lib: OutputConditionLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Met when the output equals `value`.
        */
        fun equals(value: Value): OutputCondition {
            
            val returnVal = lib.OutputCondition_equals(value.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = OutputCondition(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Met when the output does not equal `value`.
        */
        fun notEquals(value: Value): OutputCondition {
            
            val returnVal = lib.OutputCondition_not_equals(value.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = OutputCondition(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Met when the output is greater than `value`. Numeric only: both
        *sides must be the same numeric type (u32/i32/f32), else never met.
        */
        fun greaterThan(value: Value): OutputCondition {
            
            val returnVal = lib.OutputCondition_greater_than(value.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = OutputCondition(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Met when the output is less than `value`. Numeric only: both sides
        *must be the same numeric type (u32/i32/f32), else never met.
        */
        fun lessThan(value: Value): OutputCondition {
            
            val returnVal = lib.OutputCondition_less_than(value.handle);
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = OutputCondition(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Met when `output & mask` is non-zero (flag checking). Integer
        *outputs (u32/i32) only; never met for other types.
        */
        fun bitwiseAnd(mask: UInt): OutputCondition {
            
            val returnVal = lib.OutputCondition_bitwise_and(FFIUint32(mask));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = OutputCondition(handle, selfEdges, true)
            return returnOpaque
        }
    }

}