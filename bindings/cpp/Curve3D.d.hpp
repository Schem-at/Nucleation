#ifndef Curve3D_D_HPP
#define Curve3D_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct Curve3D;
} // namespace capi
} // namespace

/**
 * A sampled 3D polyline. Closed curves include the final segment back to
 * the first point and retain arc-length parameterisation for animation.
 */
class Curve3D {
public:

  /**
   * Create a curve from flat `[x0, y0, z0, x1, y1, z1, …]` coordinates.
   */
  inline static diplomat::result<std::unique_ptr<Curve3D>, NucleationError> from_points(diplomat::span<const double> coordinates, bool closed);

  inline uint32_t point_count() const;

  inline bool is_closed() const;

    inline const diplomat::capi::Curve3D* AsFFI() const;
    inline diplomat::capi::Curve3D* AsFFI();
    inline static const Curve3D* FromFFI(const diplomat::capi::Curve3D* ptr);
    inline static Curve3D* FromFFI(diplomat::capi::Curve3D* ptr);
    inline static void operator delete(void* ptr);
private:
    Curve3D() = delete;
    Curve3D(const Curve3D&) = delete;
    Curve3D(Curve3D&&) noexcept = delete;
    Curve3D operator=(const Curve3D&) = delete;
    Curve3D operator=(Curve3D&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // Curve3D_D_HPP
