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
   * Every recorded block change since settle, as JSON:
   * `[{"tick":N,"pos":[x,y,z],"from":"...","to":"..."}]`.
   * Render a schematic as gametest-flavor structure SNBT — the text
   * `from_snbt` and the corpus/render tooling consume. Lets hosts hand
   * a converted `.litematic`/`.schem` to the video renderer.
   */
  inline static std::string gametest_snbt(const nucleation::Schematic& schematic);
  template<typename W>
  inline static void gametest_snbt_write(const nucleation::Schematic& schematic, W& writeable_output);

  /**
   * Start (or stop) recording every delivered redstone update.
   *
   * Off by default and much larger than the block-change log — a door's
   * cycle runs several updates per change — so a propagation view asks
   * for it explicitly and pages with
   * {@link TickSimulation::updates_json_between}.
   */
  inline void record_updates(bool on);

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
