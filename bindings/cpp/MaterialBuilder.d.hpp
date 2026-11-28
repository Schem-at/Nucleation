#ifndef MaterialBuilder_D_HPP
#define MaterialBuilder_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Material; }
class Material;
namespace diplomat::capi { struct MaterialCondition; }
class MaterialCondition;
class NucleationError;




namespace diplomat {
namespace capi {
    struct MaterialBuilder;
} // namespace capi
} // namespace

/**
 * Builder for layered conditional materials and CSG tag-dispatched materials.
 */
class MaterialBuilder {
public:

  inline static std::unique_ptr<MaterialBuilder> create_layered();

  inline static std::unique_ptr<MaterialBuilder> create_tag_matcher(bool blend_seams, int32_t seed);

  inline diplomat::result<std::monostate, NucleationError> add_layer(const MaterialCondition& condition, const Material& material);

  inline diplomat::result<std::monostate, NucleationError> add_tag_case(uint16_t material_id, const Material& material);

  inline void set_default(const Material& material);

  inline std::unique_ptr<Material> build() const;

    inline const diplomat::capi::MaterialBuilder* AsFFI() const;
    inline diplomat::capi::MaterialBuilder* AsFFI();
    inline static const MaterialBuilder* FromFFI(const diplomat::capi::MaterialBuilder* ptr);
    inline static MaterialBuilder* FromFFI(diplomat::capi::MaterialBuilder* ptr);
    inline static void operator delete(void* ptr);
private:
    MaterialBuilder() = delete;
    MaterialBuilder(const MaterialBuilder&) = delete;
    MaterialBuilder(MaterialBuilder&&) noexcept = delete;
    MaterialBuilder operator=(const MaterialBuilder&) = delete;
    MaterialBuilder operator=(MaterialBuilder&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // MaterialBuilder_D_HPP
