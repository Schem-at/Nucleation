#ifndef NUCLEATION_VoxelPipeline_HPP
#define NUCLEATION_VoxelPipeline_HPP

#include "VoxelPipeline.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Field.hpp"
#include "Material.hpp"
#include "NucleationError.hpp"
#include "ScatterRule.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::VoxelPipeline* VoxelPipeline_create(const nucleation::capi::Field* geometry, const nucleation::capi::Material* material);

    typedef struct VoxelPipeline_add_fluid_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_add_fluid_result;
    VoxelPipeline_add_fluid_result VoxelPipeline_add_fluid(nucleation::capi::VoxelPipeline* self, const nucleation::capi::Field* field, nucleation::diplomat::capi::DiplomatStringView block);

    typedef struct VoxelPipeline_add_fluid_level_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_add_fluid_level_result;
    VoxelPipeline_add_fluid_level_result VoxelPipeline_add_fluid_level(nucleation::capi::VoxelPipeline* self, float y_level, nucleation::diplomat::capi::DiplomatStringView block);

    void VoxelPipeline_add_scatter(nucleation::capi::VoxelPipeline* self, const nucleation::capi::ScatterRule* rule);

    void VoxelPipeline_set_fill_replace(nucleation::capi::VoxelPipeline* self);

    void VoxelPipeline_set_fill_keep_existing(nucleation::capi::VoxelPipeline* self);

    typedef struct VoxelPipeline_set_fill_replacing_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_set_fill_replacing_result;
    VoxelPipeline_set_fill_replacing_result VoxelPipeline_set_fill_replacing(nucleation::capi::VoxelPipeline* self, nucleation::diplomat::capi::DiplomatStringView targets_json);

    typedef struct VoxelPipeline_to_schematic_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_to_schematic_result;
    VoxelPipeline_to_schematic_result VoxelPipeline_to_schematic(const nucleation::capi::VoxelPipeline* self, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct VoxelPipeline_to_schematic_bounded_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_to_schematic_bounded_result;
    VoxelPipeline_to_schematic_bounded_result VoxelPipeline_to_schematic_bounded(const nucleation::capi::VoxelPipeline* self, nucleation::diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct VoxelPipeline_apply_to_schematic_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_apply_to_schematic_result;
    VoxelPipeline_apply_to_schematic_result VoxelPipeline_apply_to_schematic(const nucleation::capi::VoxelPipeline* self, nucleation::capi::Schematic* schematic);

    typedef struct VoxelPipeline_apply_to_schematic_bounded_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_apply_to_schematic_bounded_result;
    VoxelPipeline_apply_to_schematic_bounded_result VoxelPipeline_apply_to_schematic_bounded(const nucleation::capi::VoxelPipeline* self, nucleation::capi::Schematic* schematic, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    void VoxelPipeline_destroy(VoxelPipeline* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::VoxelPipeline> nucleation::VoxelPipeline::create(const nucleation::Field& geometry, const nucleation::Material& material) {
    auto result = nucleation::capi::VoxelPipeline_create(geometry.AsFFI(),
        material.AsFFI());
    return std::unique_ptr<nucleation::VoxelPipeline>(nucleation::VoxelPipeline::FromFFI(result));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::VoxelPipeline::add_fluid(const nucleation::Field& field, std::string_view block) {
    auto result = nucleation::capi::VoxelPipeline_add_fluid(this->AsFFI(),
        field.AsFFI(),
        {block.data(), block.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::VoxelPipeline::add_fluid_level(float y_level, std::string_view block) {
    auto result = nucleation::capi::VoxelPipeline_add_fluid_level(this->AsFFI(),
        y_level,
        {block.data(), block.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline void nucleation::VoxelPipeline::add_scatter(const nucleation::ScatterRule& rule) {
    nucleation::capi::VoxelPipeline_add_scatter(this->AsFFI(),
        rule.AsFFI());
}

inline void nucleation::VoxelPipeline::set_fill_replace() {
    nucleation::capi::VoxelPipeline_set_fill_replace(this->AsFFI());
}

inline void nucleation::VoxelPipeline::set_fill_keep_existing() {
    nucleation::capi::VoxelPipeline_set_fill_keep_existing(this->AsFFI());
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::VoxelPipeline::set_fill_replacing(std::string_view targets_json) {
    auto result = nucleation::capi::VoxelPipeline_set_fill_replacing(this->AsFFI(),
        {targets_json.data(), targets_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::VoxelPipeline::to_schematic(std::string_view name) const {
    auto result = nucleation::capi::VoxelPipeline_to_schematic(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::VoxelPipeline::to_schematic_bounded(std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = nucleation::capi::VoxelPipeline_to_schematic_bounded(this->AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::VoxelPipeline::apply_to_schematic(nucleation::Schematic& schematic) const {
    auto result = nucleation::capi::VoxelPipeline_apply_to_schematic(this->AsFFI(),
        schematic.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::VoxelPipeline::apply_to_schematic_bounded(nucleation::Schematic& schematic, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = nucleation::capi::VoxelPipeline_apply_to_schematic_bounded(this->AsFFI(),
        schematic.AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::VoxelPipeline* nucleation::VoxelPipeline::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::VoxelPipeline*>(this);
}

inline nucleation::capi::VoxelPipeline* nucleation::VoxelPipeline::AsFFI() {
    return reinterpret_cast<nucleation::capi::VoxelPipeline*>(this);
}

inline const nucleation::VoxelPipeline* nucleation::VoxelPipeline::FromFFI(const nucleation::capi::VoxelPipeline* ptr) {
    return reinterpret_cast<const nucleation::VoxelPipeline*>(ptr);
}

inline nucleation::VoxelPipeline* nucleation::VoxelPipeline::FromFFI(nucleation::capi::VoxelPipeline* ptr) {
    return reinterpret_cast<nucleation::VoxelPipeline*>(ptr);
}

inline void nucleation::VoxelPipeline::operator delete(void* ptr) {
    nucleation::capi::VoxelPipeline_destroy(reinterpret_cast<nucleation::capi::VoxelPipeline*>(ptr));
}


#endif // NUCLEATION_VoxelPipeline_HPP
