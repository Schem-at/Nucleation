#ifndef NUCLEATION_MaterialBuilder_D_HPP
#define NUCLEATION_MaterialBuilder_D_HPP

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
namespace capi { struct Material; }
class Material;
namespace capi { struct MaterialBuilder; }
class MaterialBuilder;
namespace capi { struct MaterialCondition; }
class MaterialCondition;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct MaterialBuilder;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Builder for layered conditional materials and CSG tag-dispatched materials.
 */
class MaterialBuilder {
public:

  inline static std::unique_ptr<nucleation::MaterialBuilder> create_layered();

  inline static std::unique_ptr<nucleation::MaterialBuilder> create_tag_matcher(bool blend_seams, int32_t seed);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_layer(const nucleation::MaterialCondition& condition, const nucleation::Material& material);

  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add_tag_case(uint16_t material_id, const nucleation::Material& material);

  inline void set_default(const nucleation::Material& material);

  inline std::unique_ptr<nucleation::Material> build() const;

    inline const nucleation::capi::MaterialBuilder* AsFFI() const;
    inline nucleation::capi::MaterialBuilder* AsFFI();
    inline static const nucleation::MaterialBuilder* FromFFI(const nucleation::capi::MaterialBuilder* ptr);
    inline static nucleation::MaterialBuilder* FromFFI(nucleation::capi::MaterialBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    MaterialBuilder() = delete;
    MaterialBuilder(const nucleation::MaterialBuilder&) = delete;
    MaterialBuilder(nucleation::MaterialBuilder&&) noexcept = delete;
    MaterialBuilder operator=(const nucleation::MaterialBuilder&) = delete;
    MaterialBuilder operator=(nucleation::MaterialBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_MaterialBuilder_D_HPP
