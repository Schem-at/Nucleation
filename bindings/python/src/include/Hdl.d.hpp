#ifndef NUCLEATION_Hdl_D_HPP
#define NUCLEATION_Hdl_D_HPP

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
    struct Hdl;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespacing opaque for the HDL compiler entry points (static methods,
 * like `Routing`).
 */
class Hdl {
public:

  /**
   * Compile combinational BLIF text into a redstone PLA schematic.
   *
   * `blif` is yosys `write_blif` output (`.latch`/`.subckt` are
   * rejected — combinational only). One floor lever per `.inputs` net
   * drives the build; every signal has a dust probe. `bake=true`
   * settles the build in the tick engine first and saves it at rest
   * (needs the `mc-tick` feature, else errors).
   *
   * Probe/lever coordinates and stats come from `compile_blif_report`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> compile_blif(std::string_view blif, std::string_view name, bool bake);

  /**
   * Compile `blif` and write the JSON report: stats (`prims`,
   * `levels`, `peephole_removed`, `blocks`, `bounds`), `inputs` (=
   * lever drive order), `outputs` (each `{name, probe}` or `{name,
   * const}`), `levers` (`{signal, pos}`), and `probes`
   * (signal -> `[x, y, z]` dust cell, in the schematic's own
   * coordinates).
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> compile_blif_report(std::string_view blif, std::string_view name);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> compile_blif_report_write(std::string_view blif, std::string_view name, W& writeable_output);

  /**
   * Compile `blif` and write its typed-cell contract as JSON — the
   * `CellContract` file format (name, `io` with typed ports/buses,
   * `physical` sidecar). Vector ports (`a[0..3]` or `a0..a3`) group
   * into word buses (LSB = index 0); single bits are boolean. Input
   * port positions are the drive levers, output positions the dust
   * probes, in the same schematic coordinates as `compile_blif`.
   *
   * The `physical.delays_rt` table is ESTIMATED from levelization
   * depth (2 redstone ticks per level), not measured; `paste_safe`
   * is false until proven. Pair with `compile_blif` for the
   * schematic: schematic + this contract = an executable typed cell.
   */
  inline static nucleation::diplomat::result<std::string, nucleation::NucleationError> compile_blif_contract(std::string_view blif, std::string_view name);
  template<typename W>
  inline static nucleation::diplomat::result<std::monostate, nucleation::NucleationError> compile_blif_contract_write(std::string_view blif, std::string_view name, W& writeable_output);

    inline const nucleation::capi::Hdl* AsFFI() const;
    inline nucleation::capi::Hdl* AsFFI();
    inline static const nucleation::Hdl* FromFFI(const nucleation::capi::Hdl* ptr);
    inline static nucleation::Hdl* FromFFI(nucleation::capi::Hdl* ptr);
    inline static void operator delete(void* ptr);
private:
    Hdl() = delete;
    Hdl(const nucleation::Hdl&) = delete;
    Hdl(nucleation::Hdl&&) noexcept = delete;
    Hdl operator=(const nucleation::Hdl&) = delete;
    Hdl operator=(nucleation::Hdl&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Hdl_D_HPP
