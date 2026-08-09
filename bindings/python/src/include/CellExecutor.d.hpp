#ifndef NUCLEATION_CellExecutor_D_HPP
#define NUCLEATION_CellExecutor_D_HPP

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
namespace capi { struct CellExecutor; }
class CellExecutor;
namespace capi { struct Schematic; }
class Schematic;
namespace capi { struct Value; }
class Value;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct CellExecutor;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A typed executor bound to a self-describing cell: the schematic's
 * EMBEDDED contract (autodetected, Insign fallback) supplies the port
 * names, types and positions; the vanilla-accurate mc-tick engine
 * supplies the physics. Wraps
 * {@link crate::simulation::typed_executor::BackendCircuitExecutor}.
 */
class CellExecutor {
public:

  /**
   * Bind the schematic's embedded cell contract to the mc-tick
   * engine (needs the `mc-tick` feature, else errors). Cells deploy
   * BAKED: the backend trusts saved block states; an unbaked build
   * sits inert until the first input flip.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::CellExecutor>, nucleation::NucleationError> for_schematic(const nucleation::Schematic& schematic);

  /**
   * Set an input port by name and typed value.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> set_input(std::string_view name, const nucleation::Value& value);

  /**
   * Run until quiescent within `budget` ticks; true when settled.
   */
  inline bool settle(uint32_t budget);

  /**
   * Read an output port by name.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Value>, nucleation::NucleationError> read_output(std::string_view name);

  /**
   * Rebuild the engine from the original schematic (all inputs back
   * to their saved states).
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> reset();

    inline const nucleation::capi::CellExecutor* AsFFI() const;
    inline nucleation::capi::CellExecutor* AsFFI();
    inline static const nucleation::CellExecutor* FromFFI(const nucleation::capi::CellExecutor* ptr);
    inline static nucleation::CellExecutor* FromFFI(nucleation::capi::CellExecutor* ptr);
    inline static void operator delete(void* ptr);
private:
    CellExecutor() = delete;
    CellExecutor(const nucleation::CellExecutor&) = delete;
    CellExecutor(nucleation::CellExecutor&&) noexcept = delete;
    CellExecutor operator=(const nucleation::CellExecutor&) = delete;
    CellExecutor operator=(nucleation::CellExecutor&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_CellExecutor_D_HPP
