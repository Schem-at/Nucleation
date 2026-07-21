#ifndef NUCLEATION_Curve3D_D_HPP
#define NUCLEATION_Curve3D_D_HPP

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
namespace capi { struct Curve3D; }
class Curve3D;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct Curve3D;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A sampled 3D polyline. Closed curves include the final segment back to
 * the first point and retain arc-length parameterisation for animation.
 */
class Curve3D {
public:

  /**
   * Create a curve from flat `[x0, y0, z0, x1, y1, z1, …]` coordinates.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::Curve3D>, nucleation::NucleationError> from_points(nucleation::diplomat::span<const double> coordinates, bool closed);

  inline uint32_t point_count() const;

  inline bool is_closed() const;

    inline const nucleation::capi::Curve3D* AsFFI() const;
    inline nucleation::capi::Curve3D* AsFFI();
    inline static const nucleation::Curve3D* FromFFI(const nucleation::capi::Curve3D* ptr);
    inline static nucleation::Curve3D* FromFFI(nucleation::capi::Curve3D* ptr);
    inline static void operator delete(void* ptr);
private:
    Curve3D() = delete;
    Curve3D(const nucleation::Curve3D&) = delete;
    Curve3D(nucleation::Curve3D&&) noexcept = delete;
    Curve3D operator=(const nucleation::Curve3D&) = delete;
    Curve3D operator=(nucleation::Curve3D&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_Curve3D_D_HPP
