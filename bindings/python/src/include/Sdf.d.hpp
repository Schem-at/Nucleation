#ifndef NUCLEATION_Sdf_D_HPP
#define NUCLEATION_Sdf_D_HPP

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
    struct Sdf;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Namespace for the SDF free functions of the old ABI (`schematic_from_sdf`,
 * `sdf_eval`).
 */
class Sdf {
public:

  /**
   * Builds a schematic by sampling an SDF JSON tree with material rules JSON.
   * When `has_bounds` is false the tree's own AABB is used (fails with
   * `InvalidArgument` for unbounded trees) and the `min_*`/`max_*` arguments
   * are ignored.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> schematic_from_sdf(std::string_view sdf_json, std::string_view rules_json, bool has_bounds, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Evaluates an SDF JSON tree at a point, returning the signed distance.
   */
  inline static nucleation::diplomat::result<float, nucleation::NucleationError> eval(std::string_view sdf_json, float x, float y, float z);

    inline const nucleation::capi::Sdf* AsFFI() const;
    inline nucleation::capi::Sdf* AsFFI();
    inline static const nucleation::Sdf* FromFFI(const nucleation::capi::Sdf* ptr);
    inline static nucleation::Sdf* FromFFI(nucleation::capi::Sdf* ptr);
    inline static void operator delete(void* ptr);
private:
    Sdf() = delete;
    Sdf(const nucleation::Sdf&) = delete;
    Sdf(nucleation::Sdf&&) noexcept = delete;
    Sdf operator=(const nucleation::Sdf&) = delete;
    Sdf operator=(nucleation::Sdf&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Sdf_D_HPP
