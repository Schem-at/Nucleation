#ifndef NUCLEATION_TickSimulation_D_HPP
#define NUCLEATION_TickSimulation_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"
namespace nucleation {
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct TickSimulation; }
class TickSimulation;
class NucleationError;
class TickSettleMode;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct TickSimulation;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A headless, vanilla-accurate tick simulation of one structure.
 */
class TickSimulation {
public:

  /**
   * Why the last constructor on this thread failed, in words.
   *
   * The enum cannot carry a message, and "Simulation" is useless to
   * someone holding a door that will not load: the engine already knows
   * it is `minecraft:waxed_copper_bulb` at (4,2,1) and says so here.
   * Empty when the last construction succeeded.
   */
  inline static std::string last_error_detail();
  template<typename W>
  inline static void last_error_detail_write(W& writeable_output);

  /**
   * Largest build this will attempt, in cells.
   *
   * A 500x379x442 "door" is a saved world, and loading one exhausts the
   * wasm heap — after which every later call on that instance traps,
   * not just the one that overflowed. Refused up front instead.
   */
  inline static uint32_t max_volume();

  /**
   * Load from Java structure SNBT text.
   *
   * `extra_states`: semicolon-separated block-state descriptors that
   * later `place_block` calls may write (behaviours bind at
   * construction). `minecraft:redstone_block` is always available.
   * `origin_*`: where the build's (0,0,0) sits in world coordinates —
   * wire update order hashes absolute positions.
   *
   * The text's own `DataVersion` selects `Entity.load` Motion semantics,
   * exactly as {@link TickSimulation::from_schematic} uses the schematic's —
   * so `gametest_snbt` → `from_snbt` keeps a nan-cart build's NaN
   * velocities instead of quietly sanitising them. A text with no
   * `DataVersion` gets the engine default (the modern, NaN-dropping
   * rule); read {@link TickSimulation::motion_semantics} to see which
   * applied.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError> from_snbt(std::string_view snbt, nucleation::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, std::string_view extra_states);

  /**
   * Load from a schematic (any format nucleation can read), rendered
   * to gametest-flavor structure SNBT for mc-tick's parser.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError> from_schematic(const nucleation::Schematic& schematic, nucleation::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z, std::string_view extra_states);

  /**
   * GA fast path: construct from a flat genome-cell array — no SNBT
   * text built or parsed. Corridor layout matches the flying-ga app:
   * machine at `x_off`, world size `[bx + travel, by + 2, bz + 2]`,
   * cells flattened `((y * bz) + z) * bx + x`, `air_index` = empty
   * cell. `palette` is the run's alphabet, semicolon-separated; every
   * entry is pre-interned so behaviours bind exactly as the SNBT
   * path's EXTRA_STATES did.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::TickSimulation>, nucleation::NucleationError> from_blocks(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index, nucleation::TickSettleMode settle, int32_t origin_x, int32_t origin_y, int32_t origin_z);

  /**
   * Evaluate a whole batch of kicked flights inside the engine — one
   * wasm call per generation chunk instead of a dozen boundary calls
   * per machine. `cells` holds N genomes concatenated (each
   * `bx*by*bz` entries), `kicks` N structure-space `[x,y,z]` triples.
   * The flight protocol, probe schedule and gait detection mirror the
   * app's evalCore exactly; `early_exit` stops provably-frozen
   * machines at tick 40 without changing any reported value. Writes
   * JSON rows `[n0, startCom, startMinX, startMaxX, comAtMoveCheck |
   * null, comAtMid, period, n1, endCom, endMinX, endMaxX]`.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> eval_flight_batch(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index, nucleation::diplomat::span<const int32_t> kicks, uint32_t eval_ticks, int64_t seed, int32_t must_move_by_tick, bool need_period, bool early_exit);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> eval_flight_batch_write(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index, nucleation::diplomat::span<const int32_t> kicks, uint32_t eval_ticks, int64_t seed, int32_t must_move_by_tick, bool need_period, bool early_exit, W& writeable_output);

  /**
   * Seed the vanilla random source (`java.util.Random`'s LCG,
   * bit-for-bit). Unseeded, jittering behaviours use each
   * distribution's mean — fully deterministic, no noise.
   */
  inline void set_rng_seed(int64_t seed);

  /**
   * Advance one game tick.
   */
  inline void step();

  /**
   * Advance `ticks` game ticks.
   */
  inline void run(uint32_t ticks);

  /**
   * Run until nothing is scheduled or `budget` ticks pass. Returns
   * whether the world went quiet.
   */
  inline bool run_until_quiescent(uint32_t budget);

  /**
   * Game ticks elapsed since settle.
   */
  inline uint32_t tick_count() const;

  /**
   * Whether nothing is scheduled or queued.
   */
  inline bool is_quiescent() const;

  /**
   * Right-click a block with an empty hand (lever, button, note block).
   */
  inline void use_block(int32_t x, int32_t y, int32_t z);

  /**
   * Write a block state (`minecraft:air` breaks). The state must be in
   * the structure, in `extra_states`, or `minecraft:redstone_block`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> place_block(int32_t x, int32_t y, int32_t z, std::string_view state);

  /**
   * The block state descriptor at a position (`minecraft:air` for empty).
   */
  inline std::string get_block(int32_t x, int32_t y, int32_t z) const;
  template<typename W>
  inline void get_block_write(int32_t x, int32_t y, int32_t z, W& writeable_output) const;

  /**
   * Snapshot the entire simulation; returns a checkpoint id.
   */
  inline uint32_t checkpoint();

  /**
   * Restore a checkpoint taken earlier on this simulation.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> restore(uint32_t id);

  /**
   * Render a schematic as gametest-flavor structure SNBT — the text
   * `from_snbt` and the corpus/render tooling consume. Lets hosts hand
   * a converted `.litematic`/`.schem` to the video renderer.
   */
  inline static std::string gametest_snbt(const nucleation::Schematic& schematic);
  template<typename W>
  inline static void gametest_snbt_write(const nucleation::Schematic& schematic, W& writeable_output);

  /**
   * Report blocks whose behaviour is defined by block-entity data the
   * file does not carry.
   *
   * Some exporters write the blocks and drop the block entities. The
   * build then loads clean and simulates *wrongly but plausibly*: a
   * comparator with no `OutputSignal` reads 0, a barrel holding the
   * item that latched a repeater reads empty, and the door quietly
   * fails to reset. Two files with identical block arrays get
   * different verdicts and nothing says why. `0.45_4x4_funnel.schem`
   * is exactly this — 4 comparators, 2 furnaces, `BlockEntities` of
   * length 0, while its `.litematic` twin carries all 9.
   *
   * This does not refuse the build; it names the doubt so a host can.
   * JSON: `{"present":N,"missing_total":N,"missing":[{"name":..,
   * "count":N}],"summary":"..."}` — `summary` is empty when nothing
   * is missing, and otherwise a sentence fit to show as-is.
   */
  inline static std::string block_entity_audit_json(const nucleation::Schematic& schematic);
  template<typename W>
  inline static void block_entity_audit_json_write(const nucleation::Schematic& schematic, W& writeable_output);

  /**
   * Start (or stop) recording every delivered redstone update.
   *
   * Off by default and much larger than the block-change log — a door's
   * cycle runs several updates per change — so a propagation view asks
   * for it explicitly and pages with
   * {@link TickSimulation::updates_json_between}.
   *
   * Switching it off keeps what was recorded; use
   * {@link TickSimulation::clear_updates} to free it.
   */
  inline void record_updates(bool on);

  /**
   * Drop the recorded updates without changing whether recording is on.
   *
   * A cycle of a 6x6 door is tens of megabytes of log, so a page that
   * certifies several builds on one instance needs to release one
   * before recording the next.
   */
  inline void clear_updates();

  /**
   * Start recording a run timeline from the current tick.
   *
   * A timeline is what makes a span of simulation reviewable after the
   * fact: block deltas, the inputs that caused them and the piston
   * strokes they drove, plus one whole-world frame to replay them from.
   * Off by default — a simulation used for timing should not pay for it.
   *
   * Called again, it restarts from the current tick, and the previously
   * stopped span is released.
   *
   * Starting a recording also wipes the plain block-change log that
   * {@link TickSimulation::changes_json} and {@link TickSimulation::changes_count}
   * read back to empty — a separate reset from
   * {@link TickSimulation::record_updates}/{@link TickSimulation::clear_updates},
   * which govern a different log. A host holding a cursor into the
   * change log (the sim lab keeps a cumulative one) must reset that
   * cursor when it calls this, or it will read past the end of a log
   * that is no longer the one it was walking.
   */
  inline void record_timeline();

  /**
   * End the recording, keeping the span readable.
   *
   * This is a host's Stop button, and it is not a rewind: the span stays
   * readable and exportable until the next
   * {@link TickSimulation::record_timeline}, while the simulation is free to
   * run on without the recording following it. No-op if nothing was
   * recording.
   */
  inline void stop_timeline();

  /**
   * Where the recorded run was busy, as JSON:
   * `{"start":T,"end":T,"ticks":[{"tick":T,"changes":N,"inputs":N,
   * "pistons":N}]}`.
   *
   * The strip a host draws to let someone pick a span worth exporting.
   * Only ticks that did something appear: an idle tick is **absent**
   * rather than present with zeroes, so a build that sits still does not
   * advance the strip and a long quiet run stays cheap to send.
   *
   * `{"start":0,"end":0,"ticks":[]}` when nothing has been recorded.
   */
  inline std::string timeline_activity_json() const;
  template<typename W>
  inline void timeline_activity_json_write(W& writeable_output) const;

  /**
   * Exact and translated recurrence in the timeline a read query would
   * resolve to (see {@link Self::timeline}), as JSON:
   * `{"exact":{"start":T,"end":T,"period":N,"drift":[x,y,z]}|null,
   * "translated":{...}|null}`.
   *
   * **An absent cycle is `null`, not an error.** Most builds — an
   * adder, a door — never repeat their own state, and that is the
   * ordinary outcome, not a failed search.
   *
   * **O(ticks × blocks): replays the whole recorded span to build one
   * digest per tick boundary**, then rebuilds full frames for the
   * handful of candidates that survive. This is an on-demand "find
   * cycles" action for a host UI button, never something to call per
   * tick or per frame — poll {@link Self::timeline_activity_json} instead.
   *
   * Materialises the whole recorded timeline once to answer this call
   * (an owned `RunTimeline`'s `initial` frame copies every non-air
   * block) — acceptable for one on-demand press, not for a loop.
   *
   * `{"exact":null,"translated":null}` when nothing has been recorded.
   */
  inline std::string timeline_cycles_json() const;
  template<typename W>
  inline void timeline_cycles_json_write(W& writeable_output) const;

  /**
   * Project `[start_tick, end_tick)` of the timeline a read query would
   * resolve to (see {@link Self::timeline}) into the animated-GLB mesher's
   * `Timeline` JSON — `{"origin":[x,y,z],"tick_ms":F,
   * "events":[{"kind":"set_block"|"piston",...}]}` — via
   * `crate::tick_timeline::mesher_timeline_json`.
   *
   * **Materialises the whole recorded timeline to answer this call**
   * (an owned `RunTimeline`'s `initial` frame copies every non-air
   * block in the world) — this is an on-demand "export this
   * selection" action, not something to call per frame or poll.
   *
   * Fails if no timeline has been recorded, or if `start_tick..
   * end_tick` is empty or outside the recorded span.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> animation_timeline_json(uint32_t start_tick, uint32_t end_tick, float tick_ms) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> animation_timeline_json_write(uint32_t start_tick, uint32_t end_tick, float tick_ms, W& writeable_output) const;

  /**
   * The selection's starting scene — `[start_tick, end_tick)` of the
   * timeline a read query would resolve to (see {@link Self::timeline}) —
   * as schematic bytes, base64-encoded.
   *
   * A WASM handle cannot cross a worker boundary, so this exists for a
   * host to hand the bytes to a worker, which rebuilds the schematic
   * with `Schematic.fromData`.
   *
   * **Materialises the whole recorded timeline to answer this call**
   * — see {@link Self::animation_timeline_json}; an on-demand export
   * action, not a per-frame poll.
   *
   * Fails if no timeline has been recorded, or if `start_tick..
   * end_tick` is empty or outside the recorded span.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> selection_schematic_b64(uint32_t start_tick, uint32_t end_tick) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> selection_schematic_b64_write(uint32_t start_tick, uint32_t end_tick, W& writeable_output) const;

  /**
   * How many updates have been recorded — page before pulling them.
   */
  inline uint32_t updates_count() const;

  /**
   * Every recorded update, in delivery order.
   *
   * `seq` counts from 0 within each tick: that is the sub-tick axis, and
   * `(tick, seq)` is the order the engine actually delivered them in.
   * `state` is the block as it stood **at dispatch time**, which is what
   * makes intra-tick order legible — a snapshot cannot show it.
   */
  inline std::string updates_json() const;
  template<typename W>
  inline void updates_json_write(W& writeable_output) const;

  /**
   * The recorded updates for ticks in `[from_tick, to_tick)`.
   *
   * The whole log for a 6x6 door's cycle is megabytes; a scrubber only
   * ever shows one tick, so it should ask for one tick.
   */
  inline std::string updates_json_between(uint32_t from_tick, uint32_t to_tick) const;
  template<typename W>
  inline void updates_json_between_write(uint32_t from_tick, uint32_t to_tick, W& writeable_output) const;

  /**
   * Per-tick, per-cell update counts for ticks in `[from_tick, to_tick)`.
   *
   * The resolution playback should run at: `{phases, ticks:[{tick, total,
   * cells:[{p:[x,y,z], n, nb, sh, ph:[…]}]}]}`, where `nb`/`sh` split
   * neighbour from shape and `ph` indexes the `phases` legend. Collapses
   * a tick's tens of thousands of updates into a few hundred cells.
   */
  inline std::string updates_heat_json(uint32_t from_tick, uint32_t to_tick) const;
  template<typename W>
  inline void updates_heat_json_write(uint32_t from_tick, uint32_t to_tick, W& writeable_output) const;

  /**
   * One tick's updates in delivery order, as parallel arrays.
   *
   * For stepping *within* a tick: `seq` is the array index, `pos` is flat
   * x,y,z triples, `kind`/`phase`/`from` are integer codes with legends
   * in the payload, and `state` indexes a deduplicated `states` table.
   */
  inline std::string updates_wave_json(uint32_t tick) const;
  template<typename W>
  inline void updates_wave_json_write(uint32_t tick, W& writeable_output) const;

  /**
   * Every block a piston currently has in flight, as JSON:
   * `[{"to":[x,y,z],"from":[x,y,z],"state":"...","carried":"...",
   * "carried_short":"..."|null,"remains":"..."|null,"dir":"east",
   * "extending":bool,"started":T,"lands":T,"source_piston":bool}]`.
   *
   * Draw `carried` travelling `from` -> `to`, and `remains` (when it is
   * not null) parked at `to` for the whole move. They differ from
   * `state` — what actually lands — only for a retracting piston, whose
   * body stays put while its head comes home; vanilla's
   * `PistonHeadRenderer` splits exactly these two slots.
   *
   * `carried_short` is the same arm with `short=true`. Draw it while the
   * head is **within half a block of its body** — `progress <= 0.5`
   * extending, `progress >= 0.5` retracting — or the shaft passes
   * visibly through the back of the piston as it comes home. Which form
   * to use is yours; naming the state is the engine's.
   *
   * What a renderer needs to animate a stroke, from the simulator that
   * dispatched it. The block-change stream cannot answer this: it says a
   * cell became a `moving_piston` placeholder, not which block set off,
   * which cell it left, or which tick it arrives — so a host that
   * reconstructs strokes from changes is reimplementing piston mechanics
   * downstream of the engine, and animating on a clock the simulation
   * does not share. That desync is what draws a block twice, leaves a
   * gap where one should be, and shears a piston head off its load.
   *
   * `started` and `lands` are tick numbers in the engine's frame, where
   * {@link Self::tick_count} counts *completed* ticks: after stepping to
   * `tick_count == t`, a flight's progress is
   * `(t - started) / (lands - started)`, clamped to 1. Draw it while it
   * is listed and drop it when it stops being listed — the same call
   * that stops reporting it is the tick the real block is written, so
   * there is no frame with both and none with neither.
   */
  inline std::string moving_blocks_json() const;
  template<typename W>
  inline void moving_blocks_json_write(W& writeable_output) const;

  /**
   * Drop the recorded block changes without stopping recording.
   *
   * The log grows for as long as the simulation runs and nothing
   * empties it, so a long-running host — a browser session driving
   * thousands of ticks — accumulates every block change forever. A
   * host that has already consumed {@link TickSimulation::changes_json}
   * can say so here and keep recording on. A host holding a cursor
   * into the change log must reset that cursor when it calls this, or
   * it will read past the end of a log that is no longer the one it
   * was walking — the same hazard {@link TickSimulation::record_timeline}
   * names for its own reset of this log.
   */
  inline void clear_changes();

  inline std::string changes_json() const;
  template<typename W>
  inline void changes_json_write(W& writeable_output) const;

  /**
   * Live item entities and minecarts, as JSON:
   * `{"items":[{"id":N,"item":"...","count":N,"pos":[..],"vel":[..],
   * "on_ground":bool,"contents":[{"id":"...","count":N}]}],
   * "minecarts":[{"id":N,"kind":"...","pos":[..],"vel":[..]}]}`.
   */
  inline std::string item_entities_json() const;
  template<typename W>
  inline void item_entities_json_write(W& writeable_output) const;

  /**
   * Which `Entity.load` Motion semantics this run uses:
   * `"clamp_abs_ten"` (DataVersion <= 4556 — NaN survives a cold load)
   * or `"drop_non_finite"` (>= 4671 — it does not).
   *
   * Exposed because a door built on nan carts is a *different machine*
   * under the two, and a caller that cannot tell them apart cannot
   * report why it came apart.
   */
  inline std::string motion_semantics() const;
  template<typename W>
  inline void motion_semantics_write(W& writeable_output) const;

  /**
   * How many times an entity stood in a **retracting** piston's sweep
   * that the engine could not reproduce.
   *
   * A tripwire from when retraction was unmodelled — extension
   * displacement was measured and implemented while
   * `tools/gametest/captures/piston_pull.entities.log`'s sub-0.03
   * movements, not uniformly backwards, had no model here. All three
   * retraction geometries are implemented now, so this reports **0**,
   * including on the record 3x3 door, which used to name six. It is kept
   * because the next geometry that turns out not to be covered should be
   * reported rather than guessed at: non-zero means this run leaned on
   * behaviour we do not reproduce and its result is not trustworthy.
   */
  inline uint32_t piston_retract_contacts() const;

  /**
   * Per-tick aggregates over the recorded changes, as JSON:
   * `[{"tick":N,"changes":N,"piston":N,"redstone":N}]` — `piston`
   * counts changes touching piston blocks (base, head, moving), and
   * `redstone` changes touching wire/torch/repeater/comparator/
   * observer/lamp/lever/button/pressure-plate states.
   */
  inline std::string events_summary_json() const;
  template<typename W>
  inline void events_summary_json_write(W& writeable_output) const;

  /**
   * Every non-air block, as JSON:
   * `[{"pos":[x,y,z],"state":"..."}]`.
   * How many non-air blocks stand in the world right now.
   */
  inline uint32_t non_air_count() const;

  /**
   * Center of mass (x) of every non-air block — the GA's displacement
   * metric without a JSON round-trip. NaN when the world is empty.
   */
  inline double non_air_center_x() const;

  /**
   * Smallest x holding a non-air block; `i32::MAX` when empty.
   */
  inline int32_t non_air_min_x() const;

  /**
   * Largest x holding a non-air block; `i32::MIN` when empty.
   */
  inline int32_t non_air_max_x() const;

  /**
   * How many block changes recording has captured so far.
   */
  inline uint32_t changes_count() const;

  inline std::string world_snapshot_json() const;
  template<typename W>
  inline void world_snapshot_json_write(W& writeable_output) const;

  /**
   * Static structural analysis of the build standing in this world.
   *
   * One call, one JSON document: adhesion groups, piston/observer/source
   * nodes, the four edge kinds, every minimal self-translating subgraph
   * (the engine), payload, kickers, dead weight, and any proof that the
   * machine cannot move.
   *
   * The analysis lives in the engine rather than in the caller on
   * purpose. Every "what would this piston move?" answer comes from
   * `resolve_push`/`resolve_pull` — the same oracle-verified resolver the
   * tick loop runs — and a second copy of Minecraft's push rules written
   * on the far side of this boundary would drift from it silently.
   */
  inline std::string machine_graph_json() const;
  template<typename W>
  inline void machine_graph_json_write(W& writeable_output) const;

  /**
   * GA pre-filter: static verdicts for a whole batch of genomes.
   *
   * Same flat-cell layout as {@link Self::eval_flight_batch}, and meant to run
   * immediately before it: whatever this rejects never needs simulating.
   * Writes one row per genome, `[rejected, rejected_for_sustained,
   * engine_cell_count, payload_cell_count, dead_cell_count, "codes"]`.
   *
   * The registry, behaviour table and movability rules are built once for
   * the batch — building them per genome costs more than the analysis.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> machine_graph_batch_json(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> machine_graph_batch_json_write(int32_t bx, int32_t by, int32_t bz, int32_t travel, int32_t x_off, std::string_view palette, nucleation::diplomat::span<const uint16_t> cells, uint16_t air_index, W& writeable_output);

    inline const nucleation::capi::TickSimulation* AsFFI() const;
    inline nucleation::capi::TickSimulation* AsFFI();
    inline static const nucleation::TickSimulation* FromFFI(const nucleation::capi::TickSimulation* ptr);
    inline static nucleation::TickSimulation* FromFFI(nucleation::capi::TickSimulation* ptr);
    inline static void operator delete(void* ptr);
private:
    TickSimulation() = delete;
    TickSimulation(const nucleation::TickSimulation&) = delete;
    TickSimulation(nucleation::TickSimulation&&) noexcept = delete;
    TickSimulation operator=(const nucleation::TickSimulation&) = delete;
    TickSimulation operator=(nucleation::TickSimulation&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_TickSimulation_D_HPP
