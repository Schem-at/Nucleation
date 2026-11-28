#ifndef VoxelPipeline_HPP
#define VoxelPipeline_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::VoxelPipeline* VoxelPipeline_create(const diplomat::capi::Field* geometry, const diplomat::capi::Material* material);

    typedef struct VoxelPipeline_add_fluid_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_add_fluid_result;
    VoxelPipeline_add_fluid_result VoxelPipeline_add_fluid(diplomat::capi::VoxelPipeline* self, const diplomat::capi::Field* field, diplomat::capi::DiplomatStringView block);

    typedef struct VoxelPipeline_add_fluid_level_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_add_fluid_level_result;
    VoxelPipeline_add_fluid_level_result VoxelPipeline_add_fluid_level(diplomat::capi::VoxelPipeline* self, float y_level, diplomat::capi::DiplomatStringView block);

    void VoxelPipeline_add_scatter(diplomat::capi::VoxelPipeline* self, const diplomat::capi::ScatterRule* rule);

    void VoxelPipeline_set_fill_replace(diplomat::capi::VoxelPipeline* self);

    void VoxelPipeline_set_fill_keep_existing(diplomat::capi::VoxelPipeline* self);

    typedef struct VoxelPipeline_set_fill_replacing_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_set_fill_replacing_result;
    VoxelPipeline_set_fill_replacing_result VoxelPipeline_set_fill_replacing(diplomat::capi::VoxelPipeline* self, diplomat::capi::DiplomatStringView targets_json);

    typedef struct VoxelPipeline_to_schematic_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_to_schematic_result;
    VoxelPipeline_to_schematic_result VoxelPipeline_to_schematic(const diplomat::capi::VoxelPipeline* self, diplomat::capi::DiplomatStringView name);

    typedef struct VoxelPipeline_to_schematic_bounded_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_to_schematic_bounded_result;
    VoxelPipeline_to_schematic_bounded_result VoxelPipeline_to_schematic_bounded(const diplomat::capi::VoxelPipeline* self, diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct VoxelPipeline_apply_to_schematic_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_apply_to_schematic_result;
    VoxelPipeline_apply_to_schematic_result VoxelPipeline_apply_to_schematic(const diplomat::capi::VoxelPipeline* self, diplomat::capi::Schematic* schematic);

    typedef struct VoxelPipeline_apply_to_schematic_bounded_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} VoxelPipeline_apply_to_schematic_bounded_result;
    VoxelPipeline_apply_to_schematic_bounded_result VoxelPipeline_apply_to_schematic_bounded(const diplomat::capi::VoxelPipeline* self, diplomat::capi::Schematic* schematic, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    void VoxelPipeline_destroy(VoxelPipeline* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<VoxelPipeline> VoxelPipeline::create(const Field& geometry, const Material& material) {
    auto result = diplomat::capi::VoxelPipeline_create(geometry.AsFFI(),
        material.AsFFI());
    return std::unique_ptr<VoxelPipeline>(VoxelPipeline::FromFFI(result));
}

inline diplomat::result<std::monostate, NucleationError> VoxelPipeline::add_fluid(const Field& field, std::string_view block) {
    auto result = diplomat::capi::VoxelPipeline_add_fluid(this->AsFFI(),
        field.AsFFI(),
        {block.data(), block.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> VoxelPipeline::add_fluid_level(float y_level, std::string_view block) {
    auto result = diplomat::capi::VoxelPipeline_add_fluid_level(this->AsFFI(),
        y_level,
        {block.data(), block.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline void VoxelPipeline::add_scatter(const ScatterRule& rule) {
    diplomat::capi::VoxelPipeline_add_scatter(this->AsFFI(),
        rule.AsFFI());
}

inline void VoxelPipeline::set_fill_replace() {
    diplomat::capi::VoxelPipeline_set_fill_replace(this->AsFFI());
}

inline void VoxelPipeline::set_fill_keep_existing() {
    diplomat::capi::VoxelPipeline_set_fill_keep_existing(this->AsFFI());
}

inline diplomat::result<std::monostate, NucleationError> VoxelPipeline::set_fill_replacing(std::string_view targets_json) {
    auto result = diplomat::capi::VoxelPipeline_set_fill_replacing(this->AsFFI(),
        {targets_json.data(), targets_json.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> VoxelPipeline::to_schematic(std::string_view name) const {
    auto result = diplomat::capi::VoxelPipeline_to_schematic(this->AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> VoxelPipeline::to_schematic_bounded(std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = diplomat::capi::VoxelPipeline_to_schematic_bounded(this->AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> VoxelPipeline::apply_to_schematic(Schematic& schematic) const {
    auto result = diplomat::capi::VoxelPipeline_apply_to_schematic(this->AsFFI(),
        schematic.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> VoxelPipeline::apply_to_schematic_bounded(Schematic& schematic, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) const {
    auto result = diplomat::capi::VoxelPipeline_apply_to_schematic_bounded(this->AsFFI(),
        schematic.AsFFI(),
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::VoxelPipeline* VoxelPipeline::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::VoxelPipeline*>(this);
}

inline diplomat::capi::VoxelPipeline* VoxelPipeline::AsFFI() {
    return reinterpret_cast<diplomat::capi::VoxelPipeline*>(this);
}

inline const VoxelPipeline* VoxelPipeline::FromFFI(const diplomat::capi::VoxelPipeline* ptr) {
    return reinterpret_cast<const VoxelPipeline*>(ptr);
}

inline VoxelPipeline* VoxelPipeline::FromFFI(diplomat::capi::VoxelPipeline* ptr) {
    return reinterpret_cast<VoxelPipeline*>(ptr);
}

inline void VoxelPipeline::operator delete(void* ptr) {
    diplomat::capi::VoxelPipeline_destroy(reinterpret_cast<diplomat::capi::VoxelPipeline*>(ptr));
}


#endif // VoxelPipeline_HPP
