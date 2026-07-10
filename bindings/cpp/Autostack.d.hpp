#ifndef Autostack_D_HPP
#define Autostack_D_HPP

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
    struct Autostack;
} // namespace capi
} // namespace

/**
 * Free functions in the old ABI hung directly off `schematic_*`; here they get a
 * namespacing opaque-less home via static methods on a zero-size type is not
 * supported, so they live as static methods taking `&Schematic` explicitly.
 */
class Autostack {
public:

  /**
   * Detect repeating structures (region coverage). Writes a JSON array string;
   * each element has `mode`, `vectors`, `coverage`, `region_min`/`region_max`,
   * `cell_min`/`cell_max`, `label`.
   */
  inline static std::string detect_structures(const Schematic& schematic);
  template<typename W>
  inline static void detect_structures_write(const Schematic& schematic, W& writeable_output);

  /**
   * Graph-based detection: recovers diagonal lattice periods via the redstone
   * logic graph. Writes `"[]"` for non-redstone builds. Requires the
   * `simulation` feature; writes `"[]"` when compiled without it.
   */
  inline static std::string detect_structures_graph(const Schematic& schematic);
  template<typename W>
  inline static void detect_structures_graph_write(const Schematic& schematic, W& writeable_output);

  /**
   * Resize a 1D / diagonal structure along its period vector.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> resize_1d(const Schematic& schematic, int32_t vx, int32_t vy, int32_t vz, uint32_t units);

  /**
   * Resize a 2D structure to `n1`×`n2` cells along the two period vectors.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> resize_2d(const Schematic& schematic, int32_t v1x, int32_t v1y, int32_t v1z, int32_t v2x, int32_t v2y, int32_t v2z, uint32_t n1, uint32_t n2);

    inline const diplomat::capi::Autostack* AsFFI() const;
    inline diplomat::capi::Autostack* AsFFI();
    inline static const Autostack* FromFFI(const diplomat::capi::Autostack* ptr);
    inline static Autostack* FromFFI(diplomat::capi::Autostack* ptr);
    inline static void operator delete(void* ptr);
private:
    Autostack() = delete;
    Autostack(const Autostack&) = delete;
    Autostack(Autostack&&) noexcept = delete;
    Autostack operator=(const Autostack&) = delete;
    Autostack operator=(Autostack&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Autostack_D_HPP
