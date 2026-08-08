package at.schem.nucleation;
import com.sun.jna.Callback
import com.sun.jna.Library
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.Structure

internal interface TickSimulationLib: Library {
    fun TickSimulation_destroy(handle: Pointer)
    fun TickSimulation_last_error_detail(write: Pointer): Unit
    fun TickSimulation_max_volume(): FFIUint32
    fun TickSimulation_from_snbt(snbt: Slice, settle: Int, originX: Int, originY: Int, originZ: Int, extraStates: Slice): ResultPointerInt
    fun TickSimulation_from_schematic(schematic: Pointer, settle: Int, originX: Int, originY: Int, originZ: Int, extraStates: Slice): ResultPointerInt
    fun TickSimulation_from_blocks(bx: Int, by: Int, bz: Int, travel: Int, xOff: Int, palette: Slice, cells: Slice, airIndex: FFIUint16, settle: Int, originX: Int, originY: Int, originZ: Int): ResultPointerInt
    fun TickSimulation_eval_flight_batch(bx: Int, by: Int, bz: Int, travel: Int, xOff: Int, palette: Slice, cells: Slice, airIndex: FFIUint16, kicks: Slice, evalTicks: FFIUint32, seed: Long, mustMoveByTick: Int, needPeriod: Boolean, earlyExit: Boolean, write: Pointer): ResultUnitInt
    fun TickSimulation_set_rng_seed(handle: Pointer, seed: Long): Unit
    fun TickSimulation_step(handle: Pointer): Unit
    fun TickSimulation_run(handle: Pointer, ticks: FFIUint32): Unit
    fun TickSimulation_run_until_quiescent(handle: Pointer, budget: FFIUint32): Byte
    fun TickSimulation_tick_count(handle: Pointer): FFIUint32
    fun TickSimulation_is_quiescent(handle: Pointer): Byte
    fun TickSimulation_use_block(handle: Pointer, x: Int, y: Int, z: Int): Unit
    fun TickSimulation_place_block(handle: Pointer, x: Int, y: Int, z: Int, state: Slice): ResultUnitInt
    fun TickSimulation_get_block(handle: Pointer, x: Int, y: Int, z: Int, write: Pointer): Unit
    fun TickSimulation_checkpoint(handle: Pointer): FFIUint32
    fun TickSimulation_restore(handle: Pointer, id: FFIUint32): ResultUnitInt
    fun TickSimulation_gametest_snbt(schematic: Pointer, write: Pointer): Unit
    fun TickSimulation_block_entity_audit_json(schematic: Pointer, write: Pointer): Unit
    fun TickSimulation_record_updates(handle: Pointer, on: Boolean): Unit
    fun TickSimulation_clear_updates(handle: Pointer): Unit
    fun TickSimulation_record_timeline(handle: Pointer): Unit
    fun TickSimulation_stop_timeline(handle: Pointer): Unit
    fun TickSimulation_timeline_activity_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_timeline_cycles_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_animation_timeline_json(handle: Pointer, startTick: FFIUint32, endTick: FFIUint32, tickMs: Float, write: Pointer): ResultUnitInt
    fun TickSimulation_selection_schematic_b64(handle: Pointer, startTick: FFIUint32, endTick: FFIUint32, write: Pointer): ResultUnitInt
    fun TickSimulation_updates_count(handle: Pointer): FFIUint32
    fun TickSimulation_updates_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_updates_json_between(handle: Pointer, fromTick: FFIUint32, toTick: FFIUint32, write: Pointer): Unit
    fun TickSimulation_updates_heat_json(handle: Pointer, fromTick: FFIUint32, toTick: FFIUint32, write: Pointer): Unit
    fun TickSimulation_updates_wave_json(handle: Pointer, tick: FFIUint32, write: Pointer): Unit
    fun TickSimulation_moving_blocks_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_clear_changes(handle: Pointer): Byte
    fun TickSimulation_changes_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_changes_json_from(handle: Pointer, start: FFIUint32, write: Pointer): Unit
    fun TickSimulation_item_entities_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_motion_semantics(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_piston_retract_contacts(handle: Pointer): FFIUint32
    fun TickSimulation_events_summary_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_non_air_count(handle: Pointer): FFIUint32
    fun TickSimulation_non_air_center_x(handle: Pointer): Double
    fun TickSimulation_non_air_min_x(handle: Pointer): Int
    fun TickSimulation_non_air_max_x(handle: Pointer): Int
    fun TickSimulation_changes_count(handle: Pointer): FFIUint32
    fun TickSimulation_world_snapshot_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_machine_graph_json(handle: Pointer, write: Pointer): Unit
    fun TickSimulation_machine_graph_batch_json(bx: Int, by: Int, bz: Int, travel: Int, xOff: Int, palette: Slice, cells: Slice, airIndex: FFIUint16, write: Pointer): ResultUnitInt
}
/** A headless, vanilla-accurate tick simulation of one structure.
*/
class TickSimulation internal constructor (
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

    private class TickSimulationCleaner(val handle: Pointer, val lib: TickSimulationLib) : Runnable {
        override fun run() {
            lib.TickSimulation_destroy(handle)
        }
    }
    private fun registerCleaner() {
        CLEANER.register(this, TickSimulation.TickSimulationCleaner(handle, TickSimulation.lib));
    }

    companion object {
        internal val libClass: Class<TickSimulationLib> = TickSimulationLib::class.java
        internal val lib: TickSimulationLib = Native.load("nucleation", libClass)
        @JvmStatic

        /** Why the last constructor on this thread failed, in words.
        *
        *The enum cannot carry a message, and "Simulation" is useless to
        *someone holding a door that will not load: the engine already knows
        *it is `minecraft:waxed_copper_bulb` at (4,2,1) and says so here.
        *Empty when the last construction succeeded.
        */
        fun lastErrorDetail(): String {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.TickSimulation_last_error_detail(write);

            val returnString = DW.writeToString(write)
            return returnString
        }
        @JvmStatic

        /** Largest build this will attempt, in cells.
        *
        *A 500x379x442 "door" is a saved world, and loading one exhausts the
        *wasm heap — after which every later call on that instance traps,
        *not just the one that overflowed. Refused up front instead.
        */
        fun maxVolume(): UInt {

            val returnVal = lib.TickSimulation_max_volume();
            return (returnVal.toUInt())
        }
        @JvmStatic

        /** Load from Java structure SNBT text.
        *
        *`extra_states`: semicolon-separated block-state descriptors that
        *later `place_block` calls may write (behaviours bind at
        *construction). `minecraft:redstone_block` is always available.
        *`origin_*`: where the build's (0,0,0) sits in world coordinates —
        *wire update order hashes absolute positions.
        *
        *The text's own `DataVersion` selects `Entity.load` Motion semantics,
        *exactly as [TickSimulation::from_schematic] uses the schematic's —
        *so `gametest_snbt` → `from_snbt` keeps a nan-cart build's NaN
        *velocities instead of quietly sanitising them. A text with no
        *`DataVersion` gets the engine default (the modern, NaN-dropping
        *rule); read [TickSimulation::motion_semantics] to see which
        *applied.
        */
        fun fromSnbt(snbt: String, settle: TickSettleMode, originX: Int, originY: Int, originZ: Int, extraStates: String): Result<TickSimulation> {
            val snbtSliceMemory = PrimitiveArrayTools.borrowUtf8(snbt)
            val extraStatesSliceMemory = PrimitiveArrayTools.borrowUtf8(extraStates)

            val returnVal = lib.TickSimulation_from_snbt(snbtSliceMemory.slice, settle.toNative(), originX, originY, originZ, extraStatesSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = TickSimulation(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                snbtSliceMemory.close()
                extraStatesSliceMemory.close()
            }
        }
        @JvmStatic

        /** Load from a schematic (any format nucleation can read), rendered
        *to gametest-flavor structure SNBT for mc-tick's parser.
        */
        fun fromSchematic(schematic: Schematic, settle: TickSettleMode, originX: Int, originY: Int, originZ: Int, extraStates: String): Result<TickSimulation> {
            val extraStatesSliceMemory = PrimitiveArrayTools.borrowUtf8(extraStates)

            val returnVal = lib.TickSimulation_from_schematic(schematic.handle, settle.toNative(), originX, originY, originZ, extraStatesSliceMemory.slice);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = TickSimulation(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                extraStatesSliceMemory.close()
            }
        }
        @JvmStatic

        /** GA fast path: construct from a flat genome-cell array — no SNBT
        *text built or parsed. Corridor layout matches the flying-ga app:
        *machine at `x_off`, world size `[bx + travel, by + 2, bz + 2]`,
        *cells flattened `((y * bz) + z) * bx + x`, `air_index` = empty
        *cell. `palette` is the run's alphabet, semicolon-separated; every
        *entry is pre-interned so behaviours bind exactly as the SNBT
        *path's EXTRA_STATES did.
        */
        fun fromBlocks(bx: Int, by: Int, bz: Int, travel: Int, xOff: Int, palette: String, cells: UShortArray, airIndex: UShort, settle: TickSettleMode, originX: Int, originY: Int, originZ: Int): Result<TickSimulation> {
            val paletteSliceMemory = PrimitiveArrayTools.borrowUtf8(palette)
            val cellsSliceMemory = PrimitiveArrayTools.borrow(cells)

            val returnVal = lib.TickSimulation_from_blocks(bx, by, bz, travel, xOff, paletteSliceMemory.slice, cellsSliceMemory.slice, FFIUint16(airIndex), settle.toNative(), originX, originY, originZ);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {
                    val selfEdges: List<Any> = listOf()
                    val handle = nativeOkVal
                    val returnOpaque = TickSimulation(handle, selfEdges, true)
                    return returnOpaque.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                paletteSliceMemory.close()
                cellsSliceMemory.close()
            }
        }
        @JvmStatic

        /** Evaluate a whole batch of kicked flights inside the engine — one
        *wasm call per generation chunk instead of a dozen boundary calls
        *per machine. `cells` holds N genomes concatenated (each
        *`bx*by*bz` entries), `kicks` N structure-space `[x,y,z]` triples.
        *The flight protocol, probe schedule and gait detection mirror the
        *app's evalCore exactly; `early_exit` stops provably-frozen
        *machines at tick 40 without changing any reported value. Writes
        *JSON rows `[n0, startCom, startMinX, startMaxX, comAtMoveCheck |
        *null, comAtMid, period, n1, endCom, endMinX, endMaxX]`.
        */
        fun evalFlightBatch(bx: Int, by: Int, bz: Int, travel: Int, xOff: Int, palette: String, cells: UShortArray, airIndex: UShort, kicks: IntArray, evalTicks: UInt, seed: Long, mustMoveByTick: Int, needPeriod: Boolean, earlyExit: Boolean): Result<String> {
            val paletteSliceMemory = PrimitiveArrayTools.borrowUtf8(palette)
            val cellsSliceMemory = PrimitiveArrayTools.borrow(cells)
            val kicksSliceMemory = PrimitiveArrayTools.borrow(kicks)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.TickSimulation_eval_flight_batch(bx, by, bz, travel, xOff, paletteSliceMemory.slice, cellsSliceMemory.slice, FFIUint16(airIndex), kicksSliceMemory.slice, FFIUint32(evalTicks), seed, mustMoveByTick, needPeriod, earlyExit, write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {

                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                paletteSliceMemory.close()
                cellsSliceMemory.close()
                kicksSliceMemory.close()
            }
        }
        @JvmStatic

        /** Render a schematic as gametest-flavor structure SNBT — the text
        *`from_snbt` and the corpus/render tooling consume. Lets hosts hand
        *a converted `.litematic`/`.schem` to the video renderer.
        */
        fun gametestSnbt(schematic: Schematic): String {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.TickSimulation_gametest_snbt(schematic.handle, write);

            val returnString = DW.writeToString(write)
            return returnString
        }
        @JvmStatic

        /** Report blocks whose behaviour is defined by block-entity data the
        *file does not carry.
        *
        *Some exporters write the blocks and drop the block entities. The
        *build then loads clean and simulates *wrongly but plausibly*: a
        *comparator with no `OutputSignal` reads 0, a barrel holding the
        *item that latched a repeater reads empty, and the door quietly
        *fails to reset. Two files with identical block arrays get
        *different verdicts and nothing says why. `0.45_4x4_funnel.schem`
        *is exactly this — 4 comparators, 2 furnaces, `BlockEntities` of
        *length 0, while its `.litematic` twin carries all 9.
        *
        *This does not refuse the build; it names the doubt so a host can.
        *JSON: `{"present":N,"missing_total":N,"missing":[{"name":..,
        *"count":N}],"summary":"..."}` — `summary` is empty when nothing
        *is missing, and otherwise a sentence fit to show as-is.
        */
        fun blockEntityAuditJson(schematic: Schematic): String {
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.TickSimulation_block_entity_audit_json(schematic.handle, write);

            val returnString = DW.writeToString(write)
            return returnString
        }
        @JvmStatic

        /** GA pre-filter: static verdicts for a whole batch of genomes.
        *
        *Same flat-cell layout as [Self::eval_flight_batch], and meant to run
        *immediately before it: whatever this rejects never needs simulating.
        *Writes one row per genome, `[rejected, rejected_for_sustained,
        *engine_cell_count, payload_cell_count, dead_cell_count, "codes"]`.
        *
        *The registry, behaviour table and movability rules are built once for
        *the batch — building them per genome costs more than the analysis.
        */
        fun machineGraphBatchJson(bx: Int, by: Int, bz: Int, travel: Int, xOff: Int, palette: String, cells: UShortArray, airIndex: UShort): Result<String> {
            val paletteSliceMemory = PrimitiveArrayTools.borrowUtf8(palette)
            val cellsSliceMemory = PrimitiveArrayTools.borrow(cells)
            val write = DW.lib.diplomat_buffer_write_create(0)
            val returnVal = lib.TickSimulation_machine_graph_batch_json(bx, by, bz, travel, xOff, paletteSliceMemory.slice, cellsSliceMemory.slice, FFIUint16(airIndex), write);
            try {
                val nativeOkVal = returnVal.getNativeOk();
                if (nativeOkVal != null) {

                    val returnString = DW.writeToString(write)
                    return returnString.ok()
                } else {
                    return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
                }
            } finally {
                paletteSliceMemory.close()
                cellsSliceMemory.close()
            }
        }
    }

    /** Seed the vanilla random source (`java.util.Random`'s LCG,
    *bit-for-bit). Unseeded, jittering behaviours use each
    *distribution's mean — fully deterministic, no noise.
    */
    fun setRngSeed(seed: Long): Unit {

        val returnVal = lib.TickSimulation_set_rng_seed(handle, seed);

    }

    /** Advance one game tick.
    */
    fun step(): Unit {

        val returnVal = lib.TickSimulation_step(handle);

    }

    /** Advance `ticks` game ticks.
    */
    fun run(ticks: UInt): Unit {

        val returnVal = lib.TickSimulation_run(handle, FFIUint32(ticks));

    }

    /** Run until nothing is scheduled or `budget` ticks pass. Returns
    *whether the world went quiet.
    */
    fun runUntilQuiescent(budget: UInt): Boolean {

        val returnVal = lib.TickSimulation_run_until_quiescent(handle, FFIUint32(budget));
        return (returnVal > 0)
    }

    /** Game ticks elapsed since settle.
    */
    fun tickCount(): UInt {

        val returnVal = lib.TickSimulation_tick_count(handle);
        return (returnVal.toUInt())
    }

    /** Whether nothing is scheduled or queued.
    */
    fun isQuiescent(): Boolean {

        val returnVal = lib.TickSimulation_is_quiescent(handle);
        return (returnVal > 0)
    }

    /** Right-click a block with an empty hand (lever, button, note block).
    */
    fun useBlock(x: Int, y: Int, z: Int): Unit {

        val returnVal = lib.TickSimulation_use_block(handle, x, y, z);

    }

    /** Write a block state (`minecraft:air` breaks). The state must be in
    *the structure, in `extra_states`, or `minecraft:redstone_block`.
    */
    fun placeBlock(x: Int, y: Int, z: Int, state: String): Result<Unit> {
        val stateSliceMemory = PrimitiveArrayTools.borrowUtf8(state)

        val returnVal = lib.TickSimulation_place_block(handle, x, y, z, stateSliceMemory.slice);
        try {
            val nativeOkVal = returnVal.getNativeOk();
            if (nativeOkVal != null) {
                return Unit.ok()
            } else {
                return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
            }
        } finally {
            stateSliceMemory.close()
        }
    }

    /** The block state descriptor at a position (`minecraft:air` for empty).
    */
    fun getBlock(x: Int, y: Int, z: Int): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_get_block(handle, x, y, z, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Snapshot the entire simulation; returns a checkpoint id.
    */
    fun checkpoint(): UInt {

        val returnVal = lib.TickSimulation_checkpoint(handle);
        return (returnVal.toUInt())
    }

    /** Restore a checkpoint taken earlier on this simulation.
    */
    fun restore(id: UInt): Result<Unit> {

        val returnVal = lib.TickSimulation_restore(handle, FFIUint32(id));
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {
            return Unit.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** Start (or stop) recording every delivered redstone update.
    *
    *Off by default and much larger than the block-change log — a door's
    *cycle runs several updates per change — so a propagation view asks
    *for it explicitly and pages with
    *[TickSimulation::updates_json_between].
    *
    *Switching it off keeps what was recorded; use
    *[TickSimulation::clear_updates] to free it.
    */
    fun recordUpdates(on: Boolean): Unit {

        val returnVal = lib.TickSimulation_record_updates(handle, on);

    }

    /** Drop the recorded updates without changing whether recording is on.
    *
    *A cycle of a 6x6 door is tens of megabytes of log, so a page that
    *certifies several builds on one instance needs to release one
    *before recording the next.
    */
    fun clearUpdates(): Unit {

        val returnVal = lib.TickSimulation_clear_updates(handle);

    }

    /** Start recording a run timeline from the current tick.
    *
    *A timeline is what makes a span of simulation reviewable after the
    *fact: block deltas, the inputs that caused them and the piston
    *strokes they drove, plus one whole-world frame to replay them from.
    *Off by default — a simulation used for timing should not pay for it.
    *
    *Called again, it restarts from the current tick, and the previously
    *stopped span is released.
    *
    *Starting a recording also wipes the plain block-change log that
    *[TickSimulation::changes_json] and [TickSimulation::changes_count]
    *read back to empty — a separate reset from
    *[TickSimulation::record_updates]/[TickSimulation::clear_updates],
    *which govern a different log. A host holding a cursor into the
    *change log (the sim lab keeps a cumulative one) must reset that
    *cursor when it calls this, or it will read past the end of a log
    *that is no longer the one it was walking.
    */
    fun recordTimeline(): Unit {

        val returnVal = lib.TickSimulation_record_timeline(handle);

    }

    /** End the recording, keeping the span readable.
    *
    *This is a host's Stop button, and it is not a rewind: the span stays
    *readable and exportable until the next
    *[TickSimulation::record_timeline], while the simulation is free to
    *run on without the recording following it. No-op if nothing was
    *recording.
    */
    fun stopTimeline(): Unit {

        val returnVal = lib.TickSimulation_stop_timeline(handle);

    }

    /** Where the recorded run was busy, as JSON:
    *`{"start":T,"end":T,"ticks":[{"tick":T,"changes":N,"inputs":N,
    *"pistons":N}]}`.
    *
    *The strip a host draws to let someone pick a span worth exporting.
    *Only ticks that did something appear: an idle tick is **absent**
    *rather than present with zeroes, so a build that sits still does not
    *advance the strip and a long quiet run stays cheap to send.
    *
    *`{"start":0,"end":0,"ticks":[]}` when nothing has been recorded.
    */
    fun timelineActivityJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_timeline_activity_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Exact and translated recurrence in the timeline a read query would
    *resolve to (see [Self::timeline]), as JSON:
    *`{"exact":{"start":T,"end":T,"period":N,"drift":[x,y,z]}|null,
    *"translated":{...}|null}`.
    *
    ***An absent cycle is `null`, not an error.** Most builds — an
    *adder, a door — never repeat their own state, and that is the
    *ordinary outcome, not a failed search.
    *
    ***O(ticks × blocks): replays the whole recorded span to build one
    *digest per tick boundary**, then rebuilds full frames for the
    *handful of candidates that survive. This is an on-demand "find
    *cycles" action for a host UI button, never something to call per
    *tick or per frame — poll [Self::timeline_activity_json] instead.
    *
    *Materialises the whole recorded timeline once to answer this call
    *(an owned `RunTimeline`'s `initial` frame copies every non-air
    *block) — acceptable for one on-demand press, not for a loop.
    *
    *`{"exact":null,"translated":null}` when nothing has been recorded.
    */
    fun timelineCyclesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_timeline_cycles_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Project `[start_tick, end_tick)` of the timeline a read query would
    *resolve to (see [Self::timeline]) into the animated-GLB mesher's
    *`Timeline` JSON — `{"origin":[x,y,z],"tick_ms":F,
    *"events":[{"kind":"set_block"|"piston",...}]}` — via
    *`crate::tick_timeline::mesher_timeline_json`.
    *
    ***Materialises the whole recorded timeline to answer this call**
    *(an owned `RunTimeline`'s `initial` frame copies every non-air
    *block in the world) — this is an on-demand "export this
    *selection" action, not something to call per frame or poll.
    *
    *Fails if no timeline has been recorded, or if `start_tick..
    *end_tick` is empty or outside the recorded span.
    */
    fun animationTimelineJson(startTick: UInt, endTick: UInt, tickMs: Float): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_animation_timeline_json(handle, FFIUint32(startTick), FFIUint32(endTick), tickMs, write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** The selection's starting scene — `[start_tick, end_tick)` of the
    *timeline a read query would resolve to (see [Self::timeline]) —
    *as schematic bytes, base64-encoded.
    *
    *A WASM handle cannot cross a worker boundary, so this exists for a
    *host to hand the bytes to a worker, which rebuilds the schematic
    *with `Schematic.fromData`.
    *
    ***Materialises the whole recorded timeline to answer this call**
    *— see [Self::animation_timeline_json]; an on-demand export
    *action, not a per-frame poll.
    *
    *Fails if no timeline has been recorded, or if `start_tick..
    *end_tick` is empty or outside the recorded span.
    */
    fun selectionSchematicB64(startTick: UInt, endTick: UInt): Result<String> {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_selection_schematic_b64(handle, FFIUint32(startTick), FFIUint32(endTick), write);
        val nativeOkVal = returnVal.getNativeOk();
        if (nativeOkVal != null) {

            val returnString = DW.writeToString(write)
            return returnString.ok()
        } else {
            return NucleationErrorError(NucleationError.fromNative(returnVal.getNativeErr()!!)).err()
        }
    }

    /** How many updates have been recorded — page before pulling them.
    */
    fun updatesCount(): UInt {

        val returnVal = lib.TickSimulation_updates_count(handle);
        return (returnVal.toUInt())
    }

    /** Every recorded update, in delivery order.
    *
    *`seq` counts from 0 within each tick: that is the sub-tick axis, and
    *`(tick, seq)` is the order the engine actually delivered them in.
    *`state` is the block as it stood **at dispatch time**, which is what
    *makes intra-tick order legible — a snapshot cannot show it.
    */
    fun updatesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_updates_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** The recorded updates for ticks in `[from_tick, to_tick)`.
    *
    *The whole log for a 6x6 door's cycle is megabytes; a scrubber only
    *ever shows one tick, so it should ask for one tick.
    */
    fun updatesJsonBetween(fromTick: UInt, toTick: UInt): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_updates_json_between(handle, FFIUint32(fromTick), FFIUint32(toTick), write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Per-tick, per-cell update counts for ticks in `[from_tick, to_tick)`.
    *
    *The resolution playback should run at: `{phases, ticks:[{tick, total,
    *cells:[{p:[x,y,z], n, nb, sh, ph:[…]}]}]}`, where `nb`/`sh` split
    *neighbour from shape and `ph` indexes the `phases` legend. Collapses
    *a tick's tens of thousands of updates into a few hundred cells.
    */
    fun updatesHeatJson(fromTick: UInt, toTick: UInt): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_updates_heat_json(handle, FFIUint32(fromTick), FFIUint32(toTick), write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** One tick's updates in delivery order, as parallel arrays.
    *
    *For stepping *within* a tick: `seq` is the array index, `pos` is flat
    *x,y,z triples, `kind`/`phase`/`from` are integer codes with legends
    *in the payload, and `state` indexes a deduplicated `states` table.
    */
    fun updatesWaveJson(tick: UInt): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_updates_wave_json(handle, FFIUint32(tick), write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Every block a piston currently has in flight, as JSON:
    *`[{"to":[x,y,z],"from":[x,y,z],"state":"...","carried":"...",
    *"carried_short":"..."|null,"remains":"..."|null,"dir":"east",
    *"extending":bool,"started":T,"lands":T,"source_piston":bool}]`.
    *
    *Draw `carried` travelling `from` -> `to`, and `remains` (when it is
    *not null) parked at `to` for the whole move. They differ from
    *`state` — what actually lands — only for a retracting piston, whose
    *body stays put while its head comes home; vanilla's
    *`PistonHeadRenderer` splits exactly these two slots.
    *
    *`carried_short` is the same arm with `short=true`. Draw it while the
    *head is **within half a block of its body** — `progress <= 0.5`
    *extending, `progress >= 0.5` retracting — or the shaft passes
    *visibly through the back of the piston as it comes home. Which form
    *to use is yours; naming the state is the engine's.
    *
    *What a renderer needs to animate a stroke, from the simulator that
    *dispatched it. The block-change stream cannot answer this: it says a
    *cell became a `moving_piston` placeholder, not which block set off,
    *which cell it left, or which tick it arrives — so a host that
    *reconstructs strokes from changes is reimplementing piston mechanics
    *downstream of the engine, and animating on a clock the simulation
    *does not share. That desync is what draws a block twice, leaves a
    *gap where one should be, and shears a piston head off its load.
    *
    *`started` and `lands` are tick numbers in the engine's frame, where
    *[Self::tick_count] counts *completed* ticks: after stepping to
    *`tick_count == t`, a flight's progress is
    *`(t - started) / (lands - started)`, clamped to 1. Draw it while it
    *is listed and drop it when it stops being listed — the same call
    *that stops reporting it is the tick the real block is written, so
    *there is no frame with both and none with neither.
    */
    fun movingBlocksJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_moving_blocks_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Drop the recorded block changes without stopping recording.
    *
    *The log grows for as long as the simulation runs and nothing
    *empties it, so a long-running host — a browser session driving
    *thousands of ticks — accumulates every block change forever. A
    *host that has already consumed [TickSimulation::changes_json]
    *can say so here and keep recording on. A host holding a cursor
    *into the change log must reset that cursor when it calls this, or
    *it will read past the end of a log that is no longer the one it
    *was walking — the same hazard [TickSimulation::record_timeline]
    *names for its own reset of this log.
    *
    *Refuses — and leaves the log untouched — while
    *[TickSimulation::record_timeline] is recording: a run timeline
    *is a seed frame plus this same log, and every timeline reader
    *trusts that the log describes every mutation since recording
    *began. Clearing it out from under a live recording would make
    *replay silently wrong rather than fail loudly. Returns `true` if
    *the log was cleared, `false` if the call was refused — following
    *[TickSimulation::run_until_quiescent]'s convention of reporting
    *whether the call achieved what it was asked, rather than swallowing
    *a no-op.
    */
    fun clearChanges(): Boolean {

        val returnVal = lib.TickSimulation_clear_changes(handle);
        return (returnVal > 0)
    }

    fun changesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_changes_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** The same JSON array [TickSimulation::changes_json] produces,
    *but only the entries from index `start` onward.
    *
    *Exists for a host draining the log every frame while a run
    *timeline recording refuses [TickSimulation::clear_changes]: the
    *log only ever grows in that state, so without this,
    *`changes_json` re-serialises the whole backlog on every single
    *drain — a cost that climbs for as long as the recording runs,
    *which is exactly when a session runs longest. Reading from a
    *cursor keeps a drain's cost to what is actually new.
    *
    *`start` at or past the end of the log yields `[]`, not an error —
    *a host racing a draining cursor against a growing log should not
    *have to special-case "nothing new yet".
    */
    fun changesJsonFrom(start: UInt): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_changes_json_from(handle, FFIUint32(start), write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Live item entities and minecarts, as JSON:
    *`{"items":[{"id":N,"item":"...","count":N,"pos":[..],"vel":[..],
    *"on_ground":bool,"contents":[{"id":"...","count":N}]}],
    *"minecarts":[{"id":N,"kind":"...","pos":[..],"vel":[..]}]}`.
    */
    fun itemEntitiesJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_item_entities_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Which `Entity.load` Motion semantics this run uses:
    *`"clamp_abs_ten"` (DataVersion <= 4556 — NaN survives a cold load)
    *or `"drop_non_finite"` (>= 4671 — it does not).
    *
    *Exposed because a door built on nan carts is a *different machine*
    *under the two, and a caller that cannot tell them apart cannot
    *report why it came apart.
    */
    fun motionSemantics(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_motion_semantics(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** How many times an entity stood in a **retracting** piston's sweep
    *that the engine could not reproduce.
    *
    *A tripwire from when retraction was unmodelled — extension
    *displacement was measured and implemented while
    *`tools/gametest/captures/piston_pull.entities.log`'s sub-0.03
    *movements, not uniformly backwards, had no model here. All three
    *retraction geometries are implemented now, so this reports **0**,
    *including on the record 3x3 door, which used to name six. It is kept
    *because the next geometry that turns out not to be covered should be
    *reported rather than guessed at: non-zero means this run leaned on
    *behaviour we do not reproduce and its result is not trustworthy.
    */
    fun pistonRetractContacts(): UInt {

        val returnVal = lib.TickSimulation_piston_retract_contacts(handle);
        return (returnVal.toUInt())
    }

    /** Per-tick aggregates over the recorded changes, as JSON:
    *`[{"tick":N,"changes":N,"piston":N,"redstone":N}]` — `piston`
    *counts changes touching piston blocks (base, head, moving), and
    *`redstone` changes touching wire/torch/repeater/comparator/
    *observer/lamp/lever/button/pressure-plate states.
    */
    fun eventsSummaryJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_events_summary_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Every non-air block, as JSON:
    *`[{"pos":[x,y,z],"state":"..."}]`.
    *How many non-air blocks stand in the world right now.
    */
    fun nonAirCount(): UInt {

        val returnVal = lib.TickSimulation_non_air_count(handle);
        return (returnVal.toUInt())
    }

    /** Center of mass (x) of every non-air block — the GA's displacement
    *metric without a JSON round-trip. NaN when the world is empty.
    */
    fun nonAirCenterX(): Double {

        val returnVal = lib.TickSimulation_non_air_center_x(handle);
        return (returnVal)
    }

    /** Smallest x holding a non-air block; `i32::MAX` when empty.
    */
    fun nonAirMinX(): Int {

        val returnVal = lib.TickSimulation_non_air_min_x(handle);
        return (returnVal)
    }

    /** Largest x holding a non-air block; `i32::MIN` when empty.
    */
    fun nonAirMaxX(): Int {

        val returnVal = lib.TickSimulation_non_air_max_x(handle);
        return (returnVal)
    }

    /** How many block changes recording has captured so far.
    */
    fun changesCount(): UInt {

        val returnVal = lib.TickSimulation_changes_count(handle);
        return (returnVal.toUInt())
    }

    fun worldSnapshotJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_world_snapshot_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

    /** Static structural analysis of the build standing in this world.
    *
    *One call, one JSON document: adhesion groups, piston/observer/source
    *nodes, the four edge kinds, every minimal self-translating subgraph
    *(the engine), payload, kickers, dead weight, and any proof that the
    *machine cannot move.
    *
    *The analysis lives in the engine rather than in the caller on
    *purpose. Every "what would this piston move?" answer comes from
    *`resolve_push`/`resolve_pull` — the same oracle-verified resolver the
    *tick loop runs — and a second copy of Minecraft's push rules written
    *on the far side of this boundary would drift from it silently.
    */
    fun machineGraphJson(): String {
        val write = DW.lib.diplomat_buffer_write_create(0)
        val returnVal = lib.TickSimulation_machine_graph_json(handle, write);

        val returnString = DW.writeToString(write)
        return returnString
    }

}
