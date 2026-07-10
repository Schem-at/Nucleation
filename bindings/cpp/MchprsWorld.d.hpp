#ifndef MchprsWorld_D_HPP
#define MchprsWorld_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct RedstoneGraph; }
class RedstoneGraph;
namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct MchprsWorld;
} // namespace capi
} // namespace

/**
 * A running MCHPRS redstone simulation. Wraps {@link crate::simulation::MchprsWorld}.
 */
class MchprsWorld {
public:

  /**
   * Create a simulation world from a schematic with default options.
   */
  inline static diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError> create(const Schematic& schematic);

  /**
   * Create a simulation world with explicit options.
   */
  inline static diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError> create_with_options(const Schematic& schematic, bool optimize, bool io_only);

  /**
   * Create a simulation world with custom IO positions
   * (`custom_io_positions` is flat `[x,y,z, x,y,z, ...]`).
   */
  inline static diplomat::result<std::unique_ptr<MchprsWorld>, NucleationError> create_with_custom_io(const Schematic& schematic, bool optimize, bool io_only, diplomat::span<const int32_t> custom_io_positions);

  /**
   * One-shot convenience (old ABI: `schematic_simulate_use_block`):
   * build a world from `schematic`, fire an `on_use_block` event per
   * triple in `events_xyz`, run `ticks` ticks, and return the simulated
   * schematic. Unlike the old ABI (which mutated in place), this returns
   * a new `Schematic`.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> simulate_use_block(const Schematic& schematic, uint32_t ticks, diplomat::span<const int32_t> events_xyz);

  /**
   * Advance the simulation by `ticks` ticks.
   */
  inline void tick(uint32_t ticks);

  /**
   * Flush pending changes from the compiler to the world.
   */
  inline void flush();

  /**
   * Set the power state of a lever.
   */
  inline void set_lever_power(int32_t x, int32_t y, int32_t z, bool powered);

  /**
   * Get the power state of a lever.
   */
  inline bool get_lever_power(int32_t x, int32_t y, int32_t z) const;

  /**
   * Whether a redstone lamp is lit at the given position.
   */
  inline bool is_lit(int32_t x, int32_t y, int32_t z) const;

  /**
   * Set the signal strength (0-15) at a custom IO position.
   */
  inline void set_signal_strength(int32_t x, int32_t y, int32_t z, uint8_t strength);

  /**
   * Get the signal strength (0-15) at a position.
   */
  inline uint8_t get_signal_strength(int32_t x, int32_t y, int32_t z) const;

  /**
   * Simulate a right-click on a block (typically a lever).
   */
  inline void on_use_block(int32_t x, int32_t y, int32_t z);

  /**
   * Sync the simulation state back to the internal schematic.
   */
  inline void sync_to_schematic();

  /**
   * A clone of the world's schematic.
   */
  inline std::unique_ptr<Schematic> get_schematic() const;

  /**
   * The redstone power level (0-15) at a position.
   */
  inline uint8_t get_redstone_power(int32_t x, int32_t y, int32_t z) const;

  /**
   * Check for custom IO changes since the last check. Call before
   * `poll_custom_io_changes_json`.
   */
  inline void check_custom_io_changes();

  /**
   * Queued custom IO changes as a JSON array
   * (`[{"x":..,"y":..,"z":..,"old_power":..,"new_power":..}, ...]`),
   * clearing the queue.
   */
  inline std::string poll_custom_io_changes_json();
  template<typename W>
  inline void poll_custom_io_changes_json_write(W& writeable_output);

  /**
   * Queued custom IO changes as JSON without clearing the queue.
   */
  inline std::string peek_custom_io_changes_json() const;
  template<typename W>
  inline void peek_custom_io_changes_json_write(W& writeable_output) const;

  /**
   * Clear all queued custom IO changes.
   */
  inline void clear_custom_io_changes();

  /**
   * Extract the compiled (post-optimization) redstone logic graph.
   */
  inline diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError> export_graph() const;

  /**
   * Extract the structural (pre-fold, as-built) redstone logic graph.
   */
  inline diplomat::result<std::unique_ptr<RedstoneGraph>, NucleationError> export_graph_structural() const;

    inline const diplomat::capi::MchprsWorld* AsFFI() const;
    inline diplomat::capi::MchprsWorld* AsFFI();
    inline static const MchprsWorld* FromFFI(const diplomat::capi::MchprsWorld* ptr);
    inline static MchprsWorld* FromFFI(diplomat::capi::MchprsWorld* ptr);
    inline static void operator delete(void* ptr);
private:
    MchprsWorld() = delete;
    MchprsWorld(const MchprsWorld&) = delete;
    MchprsWorld(MchprsWorld&&) noexcept = delete;
    MchprsWorld operator=(const MchprsWorld&) = delete;
    MchprsWorld operator=(MchprsWorld&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // MchprsWorld_D_HPP
