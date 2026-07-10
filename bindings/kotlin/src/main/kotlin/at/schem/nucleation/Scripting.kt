package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface ScriptingLib: Library {
    fun Scripting_destroy(handle: Pointer)
    fun Scripting_run_lua_script(path: Slice): ResultPointerInt
    fun Scripting_run_js_script(path: Slice): ResultPointerInt
    fun Scripting_run_script(path: Slice): ResultPointerInt
}
/** Namespace for the script-runner free functions of the old ABI
*(`run_lua_script`, `run_js_script`, `run_script`).
*/
class Scripting internal constructor (
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

    private class ScriptingCleaner(val handle: Pointer, val lib: ScriptingLib) : Runnable {
        override fun run() {
            lib.Scripting_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Scripting.ScriptingCleaner(handle, Scripting.lib));
    }

    companion object {
        internal val libClass: Class<ScriptingLib> = ScriptingLib::class.java
        internal val lib: ScriptingLib = Native.load("nucleation", libClass)
        @JvmStatic
        
        /** Run a Lua script file. Returns the schematic the script assigns to
        *`result`; `NotFound` if it produced none, `Parse` if it failed, and
        *`InvalidArgument` when built without the `scripting-lua` feature.
        */
        fun runLuaScript(path: String): Result<Schematic> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
            
            val returnVal = lib.Scripting_run_lua_script(pathSliceMemory.slice);
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
                pathSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Run a JS script file. Returns the schematic the script assigns to
        *`result`; `NotFound` if it produced none, `Parse` if it failed, and
        *`InvalidArgument` when built without the `scripting-js` feature.
        */
        fun runJsScript(path: String): Result<Schematic> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
            
            val returnVal = lib.Scripting_run_js_script(pathSliceMemory.slice);
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
                pathSliceMemory.close()
            }
        }
        @JvmStatic
        
        /** Run a script file, auto-detecting the engine by extension (`.lua` or
        *`.js`). Returns the schematic the script assigns to `result`; `NotFound`
        *if it produced none, `Parse` if it failed (including unsupported
        *extensions).
        */
        fun runScript(path: String): Result<Schematic> {
            val pathSliceMemory = PrimitiveArrayTools.borrowUtf8(path)
            
            val returnVal = lib.Scripting_run_script(pathSliceMemory.slice);
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
                pathSliceMemory.close()
            }
        }
    }

}