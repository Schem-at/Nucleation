#ifndef NUCLEATION_Design_D_HPP
#define NUCLEATION_Design_D_HPP

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
namespace capi { struct Design; }
class Design;
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Design;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A composition document: loose blocks + cell instances + bus layers
 * over a shared coordinate space.
 */
class Design {
public:

  /**
   * An empty design.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> create(std::string_view name);

  /**
   * A design whose loose block layer is a copy of `base` (endpoint
   * hardware placed with raw `set_block`).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Design>, nucleation::NucleationError> for_schematic(std::string_view name, const nucleation::Schematic& base);

  /**
   * Register a library cell; its contract is resolved from the
   * schematic (embedded metadata first, Insign signs as fallback)
   * and registration fails loudly when no source defines one.
   * Writes resolution warnings as a JSON array of strings.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> add_cell(std::string_view name, const nucleation::Schematic& cell);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_cell_write(std::string_view name, const nucleation::Schematic& cell, W& writeable_output);

  /**
   * Place an instance layer referencing a library cell. `rot_y` is
   * in degrees, a multiple of 90.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> place(std::string_view name, std::string_view cell, int32_t x, int32_t y, int32_t z, int32_t rot_y);

  /**
   * Declare a drivable input port: anchor = bit-0 connection cell,
   * step to the next bit, `width` bits of `ty` (`"uint"` or
   * `"bool"`). The hardware is scanned (adjacent lever per bit) and
   * validated loudly.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> declare_input(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty);

  /**
   * Declare a readable output port (adjacent lamp per bit); same
   * shape as `declare_input`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> declare_output(std::string_view name, int32_t ax, int32_t ay, int32_t az, int32_t sx, int32_t sy, int32_t sz, uint8_t width, std::string_view ty);

  /**
   * Declare AND realize a bus. `sinks_json` is a JSON array of port
   * names; `gates_json` an array of `{"name", "anchor": [x,y,z],
   * "step": [x,y,z]}` (pass `[]` for none); `style_json` an object
   * with optional `bus_block` / `transparent_block`. Declaration
   * errors are error returns; geometric unroutability is the
   * written STATE: `"routed"` or `"failed: reason"` — realization
   * is atomic, never half-routed.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> route_bus(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json);
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> route_bus_write(std::string_view name, std::string_view driver, std::string_view sinks_json, std::string_view gates_json, std::string_view style_json, W& writeable_output);

  /**
   * The lifecycle state of a bus: `"intended"`, `"routed"` or
   * `"failed: reason"`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> bus_state(std::string_view name) const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> bus_state_write(std::string_view name, W& writeable_output) const;

  /**
   * Rip a bus: clear its fragment, back to `intended`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> rip(std::string_view name);

  /**
   * Collapse the layer stack into ONE self-describing schematic:
   * named regions per layer (`inst:x`, `bus:y`) and the merged
   * contract embedded in the metadata — itself placeable as a cell.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> flatten() const;

  /**
   * DRC + LVS over the flattened artifact. Writes `{"clean",
   * "drc": [...], "lvs": {...}, "buses": {...}}`.
   */
  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> check() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> check_write(W& writeable_output) const;

  /**
   * Settle the flattened artifact in the vanilla-accurate tick
   * engine and return it with every settled state written back and
   * `InitialState::Baked` stamped into the embedded contract (needs
   * the `mc-tick` feature, else errors).
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> bake(uint32_t budget) const;

    inline const nucleation::capi::Design* AsFFI() const;
    inline nucleation::capi::Design* AsFFI();
    inline static const nucleation::Design* FromFFI(const nucleation::capi::Design* ptr);
    inline static nucleation::Design* FromFFI(nucleation::capi::Design* ptr);
    inline static void operator delete(void* ptr);
private:
    Design() = delete;
    Design(const nucleation::Design&) = delete;
    Design(nucleation::Design&&) noexcept = delete;
    Design operator=(const nucleation::Design&) = delete;
    Design operator=(nucleation::Design&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Design_D_HPP
