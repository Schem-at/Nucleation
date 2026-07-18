#ifndef Sdf_D_HPP
#define Sdf_D_HPP

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
    struct Sdf;
} // namespace capi
} // namespace

/**
 * Namespace for the SDF free functions of the old ABI (`schematic_from_sdf`,
 * `sdf_eval`).
 */
class Sdf {
public:

  /**
   * Builds a schematic by sampling an SDF JSON tree with material rules JSON.
   * Sample an SDF tree into a schematic using the tree's own AABB —
   * the ergonomic path for bounded trees (all primitives except
   * `plane`). Fails with `InvalidArgument` for unbounded trees; use
   * `schematic_from_sdf` with explicit bounds for those.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> schematic_from_sdf_auto(std::string_view sdf_json, std::string_view rules_json);

  /**
   * When `has_bounds` is false the tree's own AABB is used (fails with
   * `InvalidArgument` for unbounded trees) and the `min_*`/`max_*` arguments
   * are ignored.
   */
  inline static diplomat::result<std::unique_ptr<Schematic>, NucleationError> schematic_from_sdf(std::string_view sdf_json, std::string_view rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Evaluates an SDF JSON tree at a point, returning the signed distance.
   */
  inline static diplomat::result<float, NucleationError> eval(std::string_view sdf_json, float x, float y, float z);

    inline const diplomat::capi::Sdf* AsFFI() const;
    inline diplomat::capi::Sdf* AsFFI();
    inline static const Sdf* FromFFI(const diplomat::capi::Sdf* ptr);
    inline static Sdf* FromFFI(diplomat::capi::Sdf* ptr);
    inline static void operator delete(void* ptr);
private:
    Sdf() = delete;
    Sdf(const Sdf&) = delete;
    Sdf(Sdf&&) noexcept = delete;
    Sdf operator=(const Sdf&) = delete;
    Sdf operator=(Sdf&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Sdf_D_HPP
