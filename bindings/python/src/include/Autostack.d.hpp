#ifndef NUCLEATION_Autostack_D_HPP
#define NUCLEATION_Autostack_D_HPP

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
    struct Autostack;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline static std::string detect_structures(const nucleation::Schematic& schematic);
  template<typename W>
  inline static void detect_structures_write(const nucleation::Schematic& schematic, W& writeable_output);

  /**
   * Graph-based detection: recovers diagonal lattice periods via the redstone
   * logic graph. Writes `"[]"` for non-redstone builds. Requires the
   * `simulation` feature; writes `"[]"` when compiled without it.
   */
  inline static std::string detect_structures_graph(const nucleation::Schematic& schematic);
  template<typename W>
  inline static void detect_structures_graph_write(const nucleation::Schematic& schematic, W& writeable_output);

  /**
   * Resize a 1D / diagonal structure along its period vector.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> resize_1d(const nucleation::Schematic& schematic, int32_t vx, int32_t vy, int32_t vz, uint32_t units);

  /**
   * Resize a 2D structure to `n1`×`n2` cells along the two period vectors.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> resize_2d(const nucleation::Schematic& schematic, int32_t v1x, int32_t v1y, int32_t v1z, int32_t v2x, int32_t v2y, int32_t v2z, uint32_t n1, uint32_t n2);

    inline const nucleation::capi::Autostack* AsFFI() const;
    inline nucleation::capi::Autostack* AsFFI();
    inline static const nucleation::Autostack* FromFFI(const nucleation::capi::Autostack* ptr);
    inline static nucleation::Autostack* FromFFI(nucleation::capi::Autostack* ptr);
    inline static void operator delete(void* ptr);
private:
    Autostack() = delete;
    Autostack(const nucleation::Autostack&) = delete;
    Autostack(nucleation::Autostack&&) noexcept = delete;
    Autostack operator=(const nucleation::Autostack&) = delete;
    Autostack operator=(nucleation::Autostack&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Autostack_D_HPP
