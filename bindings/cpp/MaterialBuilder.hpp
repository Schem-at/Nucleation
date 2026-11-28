#ifndef MaterialBuilder_HPP
#define MaterialBuilder_HPP

#include "MaterialBuilder.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Material.hpp"
#include "MaterialCondition.hpp"
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::MaterialBuilder* MaterialBuilder_create_layered(void);

    diplomat::capi::MaterialBuilder* MaterialBuilder_create_tag_matcher(bool blend_seams, int32_t seed);

    typedef struct MaterialBuilder_add_layer_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} MaterialBuilder_add_layer_result;
    MaterialBuilder_add_layer_result MaterialBuilder_add_layer(diplomat::capi::MaterialBuilder* self, const diplomat::capi::MaterialCondition* condition, const diplomat::capi::Material* material);

    typedef struct MaterialBuilder_add_tag_case_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} MaterialBuilder_add_tag_case_result;
    MaterialBuilder_add_tag_case_result MaterialBuilder_add_tag_case(diplomat::capi::MaterialBuilder* self, uint16_t material_id, const diplomat::capi::Material* material);

    void MaterialBuilder_set_default(diplomat::capi::MaterialBuilder* self, const diplomat::capi::Material* material);

    diplomat::capi::Material* MaterialBuilder_build(const diplomat::capi::MaterialBuilder* self);

    void MaterialBuilder_destroy(MaterialBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<MaterialBuilder> MaterialBuilder::create_layered() {
    auto result = diplomat::capi::MaterialBuilder_create_layered();
    return std::unique_ptr<MaterialBuilder>(MaterialBuilder::FromFFI(result));
}

inline std::unique_ptr<MaterialBuilder> MaterialBuilder::create_tag_matcher(bool blend_seams, int32_t seed) {
    auto result = diplomat::capi::MaterialBuilder_create_tag_matcher(blend_seams,
        seed);
    return std::unique_ptr<MaterialBuilder>(MaterialBuilder::FromFFI(result));
}

inline diplomat::result<std::monostate, NucleationError> MaterialBuilder::add_layer(const MaterialCondition& condition, const Material& material) {
    auto result = diplomat::capi::MaterialBuilder_add_layer(this->AsFFI(),
        condition.AsFFI(),
        material.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> MaterialBuilder::add_tag_case(uint16_t material_id, const Material& material) {
    auto result = diplomat::capi::MaterialBuilder_add_tag_case(this->AsFFI(),
        material_id,
        material.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void MaterialBuilder::set_default(const Material& material) {
    diplomat::capi::MaterialBuilder_set_default(this->AsFFI(),
        material.AsFFI());
}

inline std::unique_ptr<Material> MaterialBuilder::build() const {
    auto result = diplomat::capi::MaterialBuilder_build(this->AsFFI());
    return std::unique_ptr<Material>(Material::FromFFI(result));
}

inline const diplomat::capi::MaterialBuilder* MaterialBuilder::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::MaterialBuilder*>(this);
}

inline diplomat::capi::MaterialBuilder* MaterialBuilder::AsFFI() {
    return reinterpret_cast<diplomat::capi::MaterialBuilder*>(this);
}

inline const MaterialBuilder* MaterialBuilder::FromFFI(const diplomat::capi::MaterialBuilder* ptr) {
    return reinterpret_cast<const MaterialBuilder*>(ptr);
}

inline MaterialBuilder* MaterialBuilder::FromFFI(diplomat::capi::MaterialBuilder* ptr) {
    return reinterpret_cast<MaterialBuilder*>(ptr);
}

inline void MaterialBuilder::operator delete(void* ptr) {
    diplomat::capi::MaterialBuilder_destroy(reinterpret_cast<diplomat::capi::MaterialBuilder*>(ptr));
}


#endif // MaterialBuilder_HPP
