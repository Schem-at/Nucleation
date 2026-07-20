#ifndef DistanceField_D_HPP
#define DistanceField_D_HPP

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




namespace diplomat {
namespace capi {
    struct DistanceField;
} // namespace capi
} // namespace

class DistanceField {
public:

  /**
   * Distance transform of a build's occupied voxels: every solid block
   * learns how many blocks it sits below the surface, and the gradient of
   * that depth gives the outward normal. Computed once over the
   * schematic's bounding box.
   */
  inline static std::unique_ptr<DistanceField> from_schematic(const Schematic& schematic);

  /**
   * Blocks below the surface at a voxel: 0 for empty/outside, 1 at the
   * surface, increasing inward.
   */
  inline int32_t depth(int32_t x, int32_t y, int32_t z) const;

  /**
   * The upward component of the outward surface normal: 1 on flat ground,
   * 0 on a vertical face, negative under an overhang. The scalar to key
   * slope-based landscaping on (grass on the flats, stone on the steeps).
   */
  inline float slope(int32_t x, int32_t y, int32_t z) const;

  /**
   * The full outward surface normal as JSON `[nx, ny, nz]`.
   */
  inline std::string normal_json(int32_t x, int32_t y, int32_t z) const;
  template<typename W>
  inline void normal_json_write(int32_t x, int32_t y, int32_t z, W& writeable_output) const;

    inline const diplomat::capi::DistanceField* AsFFI() const;
    inline diplomat::capi::DistanceField* AsFFI();
    inline static const DistanceField* FromFFI(const diplomat::capi::DistanceField* ptr);
    inline static DistanceField* FromFFI(diplomat::capi::DistanceField* ptr);
    inline static void operator delete(void* ptr);
private:
    DistanceField() = delete;
    DistanceField(const DistanceField&) = delete;
    DistanceField(DistanceField&&) noexcept = delete;
    DistanceField operator=(const DistanceField&) = delete;
    DistanceField operator=(DistanceField&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // DistanceField_D_HPP
