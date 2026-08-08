#ifndef CellExecutor_D_HPP
#define CellExecutor_D_HPP

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
namespace diplomat::capi { struct Value; }
class Value;
class NucleationError;




namespace diplomat {
namespace capi {
    struct CellExecutor;
} // namespace capi
} // namespace

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
  inline static diplomat::result<std::unique_ptr<CellExecutor>, NucleationError> for_schematic(const Schematic& schematic);

  /**
   * Set an input port by name and typed value.
   */
  inline diplomat::result<std::monostate, NucleationError> set_input(std::string_view name, const Value& value);

  /**
   * Run until quiescent within `budget` ticks; true when settled.
   */
  inline bool settle(uint32_t budget);

  /**
   * Read an output port by name.
   */
  inline diplomat::result<std::unique_ptr<Value>, NucleationError> read_output(std::string_view name);

  /**
   * Rebuild the engine from the original schematic (all inputs back
   * to their saved states).
   */
  inline diplomat::result<std::monostate, NucleationError> reset();

    inline const diplomat::capi::CellExecutor* AsFFI() const;
    inline diplomat::capi::CellExecutor* AsFFI();
    inline static const CellExecutor* FromFFI(const diplomat::capi::CellExecutor* ptr);
    inline static CellExecutor* FromFFI(diplomat::capi::CellExecutor* ptr);
    inline static void operator delete(void* ptr);
private:
    CellExecutor() = delete;
    CellExecutor(const CellExecutor&) = delete;
    CellExecutor(CellExecutor&&) noexcept = delete;
    CellExecutor operator=(const CellExecutor&) = delete;
    CellExecutor operator=(CellExecutor&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // CellExecutor_D_HPP
