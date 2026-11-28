#ifndef NUCLEATION_MaterialBuilder_HPP
#define NUCLEATION_MaterialBuilder_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::MaterialBuilder* MaterialBuilder_create_layered(void);

    nucleation::capi::MaterialBuilder* MaterialBuilder_create_tag_matcher(bool blend_seams, int32_t seed);

    typedef struct MaterialBuilder_add_layer_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} MaterialBuilder_add_layer_result;
    MaterialBuilder_add_layer_result MaterialBuilder_add_layer(nucleation::capi::MaterialBuilder* self, const nucleation::capi::MaterialCondition* condition, const nucleation::capi::Material* material);

    typedef struct MaterialBuilder_add_tag_case_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} MaterialBuilder_add_tag_case_result;
    MaterialBuilder_add_tag_case_result MaterialBuilder_add_tag_case(nucleation::capi::MaterialBuilder* self, uint16_t material_id, const nucleation::capi::Material* material);

    void MaterialBuilder_set_default(nucleation::capi::MaterialBuilder* self, const nucleation::capi::Material* material);

    nucleation::capi::Material* MaterialBuilder_build(const nucleation::capi::MaterialBuilder* self);

    void MaterialBuilder_destroy(MaterialBuilder* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::MaterialBuilder> nucleation::MaterialBuilder::create_layered() {
    auto result = nucleation::capi::MaterialBuilder_create_layered();
    return std::unique_ptr<nucleation::MaterialBuilder>(nucleation::MaterialBuilder::FromFFI(result));
}

inline std::unique_ptr<nucleation::MaterialBuilder> nucleation::MaterialBuilder::create_tag_matcher(bool blend_seams, int32_t seed) {
    auto result = nucleation::capi::MaterialBuilder_create_tag_matcher(blend_seams,
        seed);
    return std::unique_ptr<nucleation::MaterialBuilder>(nucleation::MaterialBuilder::FromFFI(result));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::MaterialBuilder::add_layer(const nucleation::MaterialCondition& condition, const nucleation::Material& material) {
    auto result = nucleation::capi::MaterialBuilder_add_layer(this->AsFFI(),
        condition.AsFFI(),
        material.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::MaterialBuilder::add_tag_case(uint16_t material_id, const nucleation::Material& material) {
    auto result = nucleation::capi::MaterialBuilder_add_tag_case(this->AsFFI(),
        material_id,
        material.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::MaterialBuilder::set_default(const nucleation::Material& material) {
    nucleation::capi::MaterialBuilder_set_default(this->AsFFI(),
        material.AsFFI());
}

inline std::unique_ptr<nucleation::Material> nucleation::MaterialBuilder::build() const {
    auto result = nucleation::capi::MaterialBuilder_build(this->AsFFI());
    return std::unique_ptr<nucleation::Material>(nucleation::Material::FromFFI(result));
}

inline const nucleation::capi::MaterialBuilder* nucleation::MaterialBuilder::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::MaterialBuilder*>(this);
}

inline nucleation::capi::MaterialBuilder* nucleation::MaterialBuilder::AsFFI() {
    return reinterpret_cast<nucleation::capi::MaterialBuilder*>(this);
}

inline const nucleation::MaterialBuilder* nucleation::MaterialBuilder::FromFFI(const nucleation::capi::MaterialBuilder* ptr) {
    return reinterpret_cast<const nucleation::MaterialBuilder*>(ptr);
}

inline nucleation::MaterialBuilder* nucleation::MaterialBuilder::FromFFI(nucleation::capi::MaterialBuilder* ptr) {
    return reinterpret_cast<nucleation::MaterialBuilder*>(ptr);
}

inline void nucleation::MaterialBuilder::operator delete(void* ptr) {
    nucleation::capi::MaterialBuilder_destroy(reinterpret_cast<nucleation::capi::MaterialBuilder*>(ptr));
}


#endif // NUCLEATION_MaterialBuilder_HPP
