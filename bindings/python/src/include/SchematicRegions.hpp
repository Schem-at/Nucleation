#ifndef NUCLEATION_SchematicRegions_HPP
#define NUCLEATION_SchematicRegions_HPP

#include "SchematicRegions.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "DefinitionRegion.hpp"
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct SchematicRegions_add_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_add_result;
    SchematicRegions_add_result SchematicRegions_add(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::DefinitionRegion* region);

    typedef struct SchematicRegions_update_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_update_result;
    SchematicRegions_update_result SchematicRegions_update(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name, const nucleation::capi::DefinitionRegion* region);

    typedef struct SchematicRegions_get_result {union {nucleation::capi::DefinitionRegion* ok; nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_get_result;
    SchematicRegions_get_result SchematicRegions_get(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct SchematicRegions_remove_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_remove_result;
    SchematicRegions_remove_result SchematicRegions_remove(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct SchematicRegions_names_json_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_names_json_result;
    SchematicRegions_names_json_result SchematicRegions_names_json(const nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct SchematicRegions_create_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_create_result;
    SchematicRegions_create_result SchematicRegions_create(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct SchematicRegions_create_from_point_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_create_from_point_result;
    SchematicRegions_create_from_point_result SchematicRegions_create_from_point(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name, int32_t x, int32_t y, int32_t z);

    typedef struct SchematicRegions_create_from_bounds_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_create_from_bounds_result;
    SchematicRegions_create_from_bounds_result SchematicRegions_create_from_bounds(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct SchematicRegions_create_region_result {union {nucleation::capi::DefinitionRegion* ok; nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_create_region_result;
    SchematicRegions_create_region_result SchematicRegions_create_region(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct SchematicRegions_add_bounds_to_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_add_bounds_to_result;
    SchematicRegions_add_bounds_to_result SchematicRegions_add_bounds_to(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct SchematicRegions_add_point_to_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_add_point_to_result;
    SchematicRegions_add_point_to_result SchematicRegions_add_point_to(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name, int32_t x, int32_t y, int32_t z);

    typedef struct SchematicRegions_set_metadata_on_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_set_metadata_on_result;
    SchematicRegions_set_metadata_on_result SchematicRegions_set_metadata_on(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name, nucleation::diplomat::capi::DiplomatStringView key, nucleation::diplomat::capi::DiplomatStringView value);

    typedef struct SchematicRegions_shift_region_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} SchematicRegions_shift_region_result;
    SchematicRegions_shift_region_result SchematicRegions_shift_region(nucleation::capi::Schematic* schematic, nucleation::diplomat::capi::DiplomatStringView name, int32_t dx, int32_t dy, int32_t dz);

    void SchematicRegions_destroy(SchematicRegions* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::add(nucleation::Schematic& schematic, std::string_view name, const nucleation::DefinitionRegion& region) {
    auto result = nucleation::capi::SchematicRegions_add(schematic.AsFFI(),
        {name.data(), name.size()},
        region.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::update(nucleation::Schematic& schematic, std::string_view name, const nucleation::DefinitionRegion& region) {
    auto result = nucleation::capi::SchematicRegions_update(schematic.AsFFI(),
        {name.data(), name.size()},
        region.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> nucleation::SchematicRegions::get(const nucleation::Schematic& schematic, std::string_view name) {
    auto result = nucleation::capi::SchematicRegions_get(schematic.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::DefinitionRegion>>(std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::remove(nucleation::Schematic& schematic, std::string_view name) {
    auto result = nucleation::capi::SchematicRegions_remove(schematic.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::SchematicRegions::names_json(const nucleation::Schematic& schematic) {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::SchematicRegions_names_json(schematic.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::names_json_write(const nucleation::Schematic& schematic, W& writeable) {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::SchematicRegions_names_json(schematic.AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::create(nucleation::Schematic& schematic, std::string_view name) {
    auto result = nucleation::capi::SchematicRegions_create(schematic.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::create_from_point(nucleation::Schematic& schematic, std::string_view name, int32_t x, int32_t y, int32_t z) {
    auto result = nucleation::capi::SchematicRegions_create_from_point(schematic.AsFFI(),
        {name.data(), name.size()},
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::create_from_bounds(nucleation::Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::SchematicRegions_create_from_bounds(schematic.AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError> nucleation::SchematicRegions::create_region(nucleation::Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::SchematicRegions_create_region(schematic.AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::DefinitionRegion>>(std::unique_ptr<nucleation::DefinitionRegion>(nucleation::DefinitionRegion::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::DefinitionRegion>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::add_bounds_to(nucleation::Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::SchematicRegions_add_bounds_to(schematic.AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::add_point_to(nucleation::Schematic& schematic, std::string_view name, int32_t x, int32_t y, int32_t z) {
    auto result = nucleation::capi::SchematicRegions_add_point_to(schematic.AsFFI(),
        {name.data(), name.size()},
        x,
        y,
        z);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::set_metadata_on(nucleation::Schematic& schematic, std::string_view name, std::string_view key, std::string_view value) {
    auto result = nucleation::capi::SchematicRegions_set_metadata_on(schematic.AsFFI(),
        {name.data(), name.size()},
        {key.data(), key.size()},
        {value.data(), value.size()});
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::SchematicRegions::shift_region(nucleation::Schematic& schematic, std::string_view name, int32_t dx, int32_t dy, int32_t dz) {
    auto result = nucleation::capi::SchematicRegions_shift_region(schematic.AsFFI(),
        {name.data(), name.size()},
        dx,
        dy,
        dz);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::SchematicRegions* nucleation::SchematicRegions::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::SchematicRegions*>(this);
}

inline nucleation::capi::SchematicRegions* nucleation::SchematicRegions::AsFFI() {
    return reinterpret_cast<nucleation::capi::SchematicRegions*>(this);
}

inline const nucleation::SchematicRegions* nucleation::SchematicRegions::FromFFI(const nucleation::capi::SchematicRegions* ptr) {
    return reinterpret_cast<const nucleation::SchematicRegions*>(ptr);
}

inline nucleation::SchematicRegions* nucleation::SchematicRegions::FromFFI(nucleation::capi::SchematicRegions* ptr) {
    return reinterpret_cast<nucleation::SchematicRegions*>(ptr);
}

inline void nucleation::SchematicRegions::operator delete(void* ptr) {
    nucleation::capi::SchematicRegions_destroy(reinterpret_cast<nucleation::capi::SchematicRegions*>(ptr));
}


#endif // NUCLEATION_SchematicRegions_HPP
