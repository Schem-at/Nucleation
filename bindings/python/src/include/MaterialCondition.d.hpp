#ifndef NUCLEATION_MaterialCondition_D_HPP
#define NUCLEATION_MaterialCondition_D_HPP

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
namespace capi { struct Field; }
class Field;
namespace capi { struct MaterialCondition; }
class MaterialCondition;
class FieldChannel;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct MaterialCondition;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Composable voxel predicate for {@link Material} layers and {@link ScatterRule}s.
 */
class MaterialCondition {
public:

  inline static std::unique_ptr<nucleation::MaterialCondition> always();

  inline std::unique_ptr<nucleation::MaterialCondition> and_with(const nucleation::MaterialCondition& other) const;

  inline std::unique_ptr<nucleation::MaterialCondition> or_with(const nucleation::MaterialCondition& other) const;

  inline std::unique_ptr<nucleation::MaterialCondition> negated() const;

  inline static std::unique_ptr<nucleation::MaterialCondition> tag(uint16_t material_id);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> channel_range(nucleation::FieldChannel channel, bool has_min, float min, bool has_max, float max);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> field_range(const nucleation::Field& field, bool has_min, float min, bool has_max, float max);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> y_range(bool has_min, float min, bool has_max, float max);

  inline static std::unique_ptr<nucleation::MaterialCondition> depth_below_surface(bool has_min, int32_t min, bool has_max, int32_t max);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> slope_range(bool has_min, float min, bool has_max, float max);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> curvature_range(bool has_min, float min, bool has_max, float max);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> normal_dot(float dx, float dy, float dz, bool has_min, float min, bool has_max, float max);

  inline static std::unique_ptr<nucleation::MaterialCondition> exposed_face(uint8_t mask, bool require_all);

  /**
   * True when the voxel's top (`+Y`) face is exposed to air (`face_mask::TOP`).
   */
  inline static std::unique_ptr<nucleation::MaterialCondition> exposed_top();

  /**
   * True when the voxel's bottom (`-Y`) face is exposed to air (`face_mask::BOTTOM`).
   */
  inline static std::unique_ptr<nucleation::MaterialCondition> exposed_bottom();

  /**
   * True when any horizontal side face (`±X` or `±Z`) is exposed to air.
   */
  inline static std::unique_ptr<nucleation::MaterialCondition> exposed_side();

  /**
   * True when any face of the voxel is exposed to air.
   */
  inline static std::unique_ptr<nucleation::MaterialCondition> exposed_any();

  /**
   * True when all six neighbor voxels are solid (completely interior).
   */
  inline static std::unique_ptr<nucleation::MaterialCondition> interior();

  /**
   * True when world `y >= min_y`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> y_above(float min_y);

  /**
   * True when world `y <= max_y`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> y_below(float max_y);

  /**
   * True when world `min_y <= y <= max_y`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> y_between(float min_y, float max_y);

  /**
   * True when surface slope (`1 - normal.y`) is `<= max_slope` (flatter than threshold).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> slope_below(float max_slope);

  /**
   * True when surface slope (`1 - normal.y`) is `>= min_slope` (steeper than threshold).
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> slope_above(float min_slope);

  /**
   * True when column depth below the exposed surface is in `[min_depth, max_depth]`.
   */
  inline static std::unique_ptr<nucleation::MaterialCondition> depth_between(int32_t min_depth, int32_t max_depth);

  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::MaterialCondition>, nucleation::NucleationError> chance(float probability, int32_t seed);

    inline const nucleation::capi::MaterialCondition* AsFFI() const;
    inline nucleation::capi::MaterialCondition* AsFFI();
    inline static const nucleation::MaterialCondition* FromFFI(const nucleation::capi::MaterialCondition* ptr);
    inline static nucleation::MaterialCondition* FromFFI(nucleation::capi::MaterialCondition* ptr);
    inline static void operator delete(void* ptr);
private:
    MaterialCondition() = delete;
    MaterialCondition(const nucleation::MaterialCondition&) = delete;
    MaterialCondition(nucleation::MaterialCondition&&) noexcept = delete;
    MaterialCondition operator=(const nucleation::MaterialCondition&) = delete;
    MaterialCondition operator=(nucleation::MaterialCondition&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_MaterialCondition_D_HPP
