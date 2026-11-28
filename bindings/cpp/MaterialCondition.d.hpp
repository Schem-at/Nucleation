#ifndef MaterialCondition_D_HPP
#define MaterialCondition_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Field; }
class Field;
class FieldChannel;
class NucleationError;




namespace diplomat {
namespace capi {
    struct MaterialCondition;
} // namespace capi
} // namespace

/**
 * Composable voxel predicate for {@link Material} layers and {@link ScatterRule}s.
 */
class MaterialCondition {
public:

  inline static std::unique_ptr<MaterialCondition> always();

  inline std::unique_ptr<MaterialCondition> and_with(const MaterialCondition& other) const;

  inline std::unique_ptr<MaterialCondition> or_with(const MaterialCondition& other) const;

  inline std::unique_ptr<MaterialCondition> negated() const;

  inline static std::unique_ptr<MaterialCondition> tag(uint16_t material_id);

  inline static diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> channel_range(FieldChannel channel, bool has_min, float min, bool has_max, float max);

  inline static diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> field_range(const Field& field, bool has_min, float min, bool has_max, float max);

  inline static diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> y_range(bool has_min, float min, bool has_max, float max);

  inline static std::unique_ptr<MaterialCondition> depth_below_surface(bool has_min, int32_t min, bool has_max, int32_t max);

  inline static diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> slope_range(bool has_min, float min, bool has_max, float max);

  inline static diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> curvature_range(bool has_min, float min, bool has_max, float max);

  inline static diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> normal_dot(float dx, float dy, float dz, bool has_min, float min, bool has_max, float max);

  inline static std::unique_ptr<MaterialCondition> exposed_face(uint8_t mask, bool require_all);

  inline static diplomat::result<std::unique_ptr<MaterialCondition>, NucleationError> chance(float probability, int32_t seed);

    inline const diplomat::capi::MaterialCondition* AsFFI() const;
    inline diplomat::capi::MaterialCondition* AsFFI();
    inline static const MaterialCondition* FromFFI(const diplomat::capi::MaterialCondition* ptr);
    inline static MaterialCondition* FromFFI(diplomat::capi::MaterialCondition* ptr);
    inline static void operator delete(void* ptr);
private:
    MaterialCondition() = delete;
    MaterialCondition(const MaterialCondition&) = delete;
    MaterialCondition(MaterialCondition&&) noexcept = delete;
    MaterialCondition operator=(const MaterialCondition&) = delete;
    MaterialCondition operator=(MaterialCondition&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // MaterialCondition_D_HPP
