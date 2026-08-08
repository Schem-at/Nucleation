package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface HdlLib: Library {
    fun Hdl_destroy(handle: Pointer)
    fun Hdl_compile_blif(blif: Slice, name: Slice, bake: Boolean): ResultPointerInt
    fun Hdl_compile_blif_report(blif: Slice, name: Slice, write: Pointer): ResultUnitInt
    fun Hdl_compile_blif_contract(blif: Slice, name: Slice, write: Pointer): ResultUnitInt
}
/** Namespacing opaque for the HDL compiler entry points (static methods,
*like `Routing`).
*/
class Hdl internal constructor (
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

    private class HdlCleaner(val handle: Pointer, val lib: HdlLib) : Runnable {
        override fun run() {
            lib.Hdl_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, Hdl.HdlCleaner(handle, Hdl.lib));
    }

    companion object {
        internal val libClass: Class<HdlLib> = HdlLib::class.java
        internal val lib: HdlLib = Native.load("nucleation", libClass)
        @JvmStatic

        /** Compile combinational BLIF text into a redstone PLA schematic.
        *
        *`blif` is yosys `write_blif` output (`.latch`/`.subckt` are
        *rejected — combinational only). One floor lever per `.inputs` net
        *drives the build; every signal has a dust probe. `bake=true`
        *settles the build in the tick engine first and saves it at rest
        *(needs the `mc-tick` feature, else errors).
        *
        *Probe/lever coordinates and stats come from `compile_blif_report`.
        */
        fun compileBlif(blif: String, name: String, bake: Boolean): Result<Schematic> {
            val blifSliceMemory = PrimitiveArrayTools.borrowUtf8(blif)
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)

            val returnVal = lib.Hdl_compile_blif(blifSliceMemory.slice, nameSliceMemory.slice, bake);
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
                blifSliceMemory.close()
                nameSliceMemory.close()
            }
        }
        @JvmStatic

        /** Compile `blif` and write the JSON report: stats (`prims`,
        *`levels`, `peephole_removed`, `blocks`, `bounds`), `inputs` (=
        *lever drive order), `outputs` (each `{name, probe}` or `{name,
        *const}`), `levers` (`{signal, pos}`), and `probes`
        *(signal -> `[x, y, z]` dust cell, in the schematic's own
        *coordinates).
        */
        fun compileBlifReport(blif: String, name: String): Result<String> {
            val blifSliceMemory = PrimitiveArrayTools.borrowUtf8(blif)
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Hdl_compile_blif_report(blifSliceMemory.slice, nameSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {

                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                blifSliceMemory.close()
                nameSliceMemory.close()
            }
        }
        @JvmStatic

        /** Compile `blif` and write its typed-cell contract as JSON — the
        *`CellContract` file format (name, `io` with typed ports/buses,
        *`physical` sidecar). Vector ports (`a[0..3]` or `a0..a3`) group
        *into word buses (LSB = index 0); single bits are boolean. Input
        *port positions are the drive levers, output positions the dust
        *probes, in the same schematic coordinates as `compile_blif`.
        *
        *The `physical.delays_rt` table is ESTIMATED from levelization
        *depth (2 redstone ticks per level), not measured; `paste_safe`
        *is false until proven. Pair with `compile_blif` for the
        *schematic: schematic + this contract = an executable typed cell.
        */
        fun compileBlifContract(blif: String, name: String): Result<String> {
            val blifSliceMemory = PrimitiveArrayTools.borrowUtf8(blif)
            val nameSliceMemory = PrimitiveArrayTools.borrowUtf8(name)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.Hdl_compile_blif_contract(blifSliceMemory.slice, nameSliceMemory.slice, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {

                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                blifSliceMemory.close()
                nameSliceMemory.close()
            }
        }
    }

}
