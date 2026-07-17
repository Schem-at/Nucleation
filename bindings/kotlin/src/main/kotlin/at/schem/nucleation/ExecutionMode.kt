package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ExecutionModeLib: Library {
    fun ExecutionMode_destroy(handle: Pointer)
    fun ExecutionMode_fixed_ticks(ticks: FFIUint32): Pointer
    fun ExecutionMode_until_condition(outputName: Slice, condition: Pointer, maxTicks: FFIUint32, checkInterval: FFIUint32): ResultPointerInt
    fun ExecutionMode_until_change(maxTicks: FFIUint32, checkInterval: FFIUint32): Pointer
    fun ExecutionMode_until_stable(stableTicks: FFIUint32, maxTicks: FFIUint32): Pointer
}
/** How long to run the circuit for one `execute` call (PORTING rule 10).
*/
class ExecutionMode internal constructor (
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

    private class ExecutionModeCleaner(val handle: Pointer, val lib: ExecutionModeLib) : Runnable {
        override fun run() {
            lib.ExecutionMode_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, ExecutionMode.ExecutionModeCleaner(handle, ExecutionMode.lib));
    }

    companion object {
        internal val libClass: Class<ExecutionModeLib> = ExecutionModeLib::class.java
        internal val lib: ExecutionModeLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Run for exactly `ticks` ticks.
        */
        fun fixedTicks(ticks: UInt): ExecutionMode {
            
            val returnVal = lib.ExecutionMode_fixed_ticks(FFIUint32(ticks));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = ExecutionMode(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Run until the output named `output_name` meets `condition`,
        *checking every `check_interval` ticks, giving up after `max_ticks`
        *ticks (the result's `condition_met` reports which happened).
        */
        fun untilCondition(outputName: String, condition: OutputCondition, maxTicks: UInt, checkInterval: UInt): Result<ExecutionMode> {
            val outputNameSliceMemory = PrimitiveArrayTools.borrowUtf8(outputName)
            
            val returnVal = lib.ExecutionMode_until_condition(outputNameSliceMemory.slice, condition.handle, FFIUint32(maxTicks), FFIUint32(checkInterval));
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal 
                    val returnOpaque = ExecutionMode(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                outputNameSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Run until any output changes from its initial reading, checking
        *every `check_interval` ticks, giving up after `max_ticks` ticks.
        */
        fun untilChange(maxTicks: UInt, checkInterval: UInt): ExecutionMode {
            
            val returnVal = lib.ExecutionMode_until_change(FFIUint32(maxTicks), FFIUint32(checkInterval));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = ExecutionMode(handle, selfEdges, true)
            return returnOpaque
        }
        @JvmStatic
        
        /** Run (one tick at a time) until all outputs have been unchanged for
        *`stable_ticks` consecutive ticks, giving up after `max_ticks`
        *ticks (the result's `condition_met` reports stability).
        */
        fun untilStable(stableTicks: UInt, maxTicks: UInt): ExecutionMode {
            
            val returnVal = lib.ExecutionMode_until_stable(FFIUint32(stableTicks), FFIUint32(maxTicks));
            val selfEdges: List<Any> = listOf()
            val handle = returnVal 
            val returnOpaque = ExecutionMode(handle, selfEdges, true)
            return returnOpaque
        }
    }

}