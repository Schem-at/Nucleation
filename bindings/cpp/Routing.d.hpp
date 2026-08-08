#ifndef Routing_D_HPP
#define Routing_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct Routing;
} // namespace capi
} // namespace

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
  inline static diplomat::result<std::string, NucleationError> route_net(Schematic& schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, std::string_view label);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> route_net_write(Schematic& schematic, int32_t sx, int32_t sy, int32_t sz, int32_t dx, int32_t dy, int32_t dz, std::string_view label, W& writeable_output);

  /**
   * Route every net in `nets_json` with negotiated congestion
   * (pnr-core PathFinder) in one labelled workspace, write the
   * geometry into the schematic, and write the JSON report
   * (`routes` with per-net `path`/`delay_rt`, `notes`,
   * `violations`). Supports per-net-class rule overrides
   * (`classes`: io_contract `NetClassRule`s, with `region`
   * resolving named route zones tagged on the schematic's
   * DefinitionRegions), plus `bounds`, `budget` and `congestion`
   * options — see `crate::routing::route_all_schematic` for the
   * exact request shape.
   */
  inline static diplomat::result<std::string, NucleationError> route_all(Schematic& schematic, std::string_view nets_json);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> route_all_write(Schematic& schematic, std::string_view nets_json, W& writeable_output);

  /**
   * LVS v1: compare an intended netlist (`{"nets": [{"name",
   * "terminals": [[x,y,z], ...]}]}`) against the conduction
   * netlist extracted statically from the schematic (dust
   * adjacency incl. cut diagonals plus repeater/comparator/torch
   * through-component edges). Writes `{"clean", "matched",
   * "opens", "shorts", "cycles"}`.
   */
  inline static diplomat::result<std::string, NucleationError> lvs(const Schematic& schematic, std::string_view intent_json);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> lvs_write(const Schematic& schematic, std::string_view intent_json, W& writeable_output);

  /**
   * Run design-rule checks (support audit, repeater-cycle detection,
   * optional decay) over the schematic. Writes a JSON array; each
   * element has `kind` plus violation-specific fields. Label-aware
   * short checking needs a labelled workspace and stays native.
   */
  inline static diplomat::result<std::string, NucleationError> drc(const Schematic& schematic, bool check_decay);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> drc_write(const Schematic& schematic, bool check_decay, W& writeable_output);

  /**
   * Static timing over the schematic plus a gate netlist given as
   * JSON: `{"inputs": ["a", ...], "gates": [{"out": "y",
   * "ins": ["a", "b"], "delay_rt": 2}, ...]}`. Writes
   * `{"arrival_rt": {sig: rt}, "critical": [sig, ...]}`.
   */
  inline static diplomat::result<std::string, NucleationError> sta(const Schematic& schematic, std::string_view netlist_json);
  template<typename W>
  inline static diplomat::result<std::monostate, NucleationError> sta_write(const Schematic& schematic, std::string_view netlist_json, W& writeable_output);

    inline const diplomat::capi::Routing* AsFFI() const;
    inline diplomat::capi::Routing* AsFFI();
    inline static const Routing* FromFFI(const diplomat::capi::Routing* ptr);
    inline static Routing* FromFFI(diplomat::capi::Routing* ptr);
    inline static void operator delete(void* ptr);
private:
    Routing() = delete;
    Routing(const Routing&) = delete;
    Routing(Routing&&) noexcept = delete;
    Routing operator=(const Routing&) = delete;
    Routing operator=(Routing&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Routing_D_HPP
