#ifndef NUCLEATION_Routing_D_HPP
#define NUCLEATION_Routing_D_HPP

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
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Routing;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespacing opaque for routing entry points (static methods taking
 * `&Schematic` explicitly, like `Autostack`).
 */
class Routing {
public:

  /**
   * Route one net from `(sx, sy, sz)` to `(dx, dy, dz)` with default
   * rules (torch-ladder vias, stair cap 4, refresh 5) and write the
   * emitted geometry into the schematic. Writes the routed path as a
   * JSON array of `[x, y, z]` cells.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> route_net(nucleation::Schematic& schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, std::string_view label);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> route_net_write(nucleation::Schematic& schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, std::string_view label, W& writeable_output);

  /**
   * Run design-rule checks (support audit, repeater-cycle detection,
   * optional decay) over the schematic. Writes a JSON array; each
   * element has `kind` plus violation-specific fields. Label-aware
   * short checking needs a labelled workspace and stays native.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> drc(const nucleation::Schematic& schematic, bool check_decay);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> drc_write(const nucleation::Schematic& schematic, bool check_decay, W& writeable_output);

  /**
   * Static timing over the schematic plus a gate netlist given as
   * JSON: `{"inputs": ["a", ...], "gates": [{"out": "y",
   * "ins": ["a", "b"], "delay_rt": 2}, ...]}`. Writes
   * `{"arrival_rt": {sig: rt}, "critical": [sig, ...]}`.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> sta(const nucleation::Schematic& schematic, std::string_view netlist_json);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> sta_write(const nucleation::Schematic& schematic, std::string_view netlist_json, W& writeable_output);

    inline const nucleation::capi::Routing* AsFFI() const;
    inline nucleation::capi::Routing* AsFFI();
    inline static const nucleation::Routing* FromFFI(const nucleation::capi::Routing* ptr);
    inline static nucleation::Routing* FromFFI(nucleation::capi::Routing* ptr);
    inline static void operator delete(void* ptr);
private:
    Routing() = delete;
    Routing(const nucleation::Routing&) = delete;
    Routing(nucleation::Routing&&) noexcept = delete;
    Routing operator=(const nucleation::Routing&) = delete;
    Routing operator=(nucleation::Routing&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Routing_D_HPP
