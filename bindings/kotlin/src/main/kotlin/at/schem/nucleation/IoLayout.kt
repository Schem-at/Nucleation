package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface IoLayoutLib: Library {
    fun IoLayout_destroy(handle: Pointer)
    fun IoLayout_input_names_json(handle: Pointer, write: Pointer): Unit
    fun IoLayout_output_names_json(handle: Pointer, write: Pointer): Unit
}
/** An immutable IO layout. Wraps
*[crate::simulation::typed_executor::IoLayout].
*/
class IoLayout internal constructor (
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

    private class IoLayoutCleaner(val handle: Pointer, val lib: IoLayoutLib) : Runnable {
        override fun run() {
            lib.IoLayout_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, IoLayout.IoLayoutCleaner(handle, IoLayout.lib));
    }

    companion object {
        internal val libClass: Class<IoLayoutLib> = IoLayoutLib::class.java
        internal val lib: IoLayoutLib = Native.load("nucleation", libClass)
    }
    
    /** Input names as a JSON array string.
    */
    fun inputNamesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.IoLayout_input_names_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }
    
    /** Output names as a JSON array string.
    */
    fun outputNamesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.IoLayout_output_names_json(handle, write);
        
        val returnString = DW.writeToString(write)
        return returnString
    }

}