#ifndef SchematicRegions_HPP
#define SchematicRegions_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct SchematicRegions_add_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_add_result;
    SchematicRegions_add_result SchematicRegions_add(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name, const diplomat::capi::DefinitionRegion* region);

    typedef struct SchematicRegions_update_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_update_result;
    SchematicRegions_update_result SchematicRegions_update(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name, const diplomat::capi::DefinitionRegion* region);

    typedef struct SchematicRegions_get_result {union {diplomat::capi::DefinitionRegion* ok; diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_get_result;
    SchematicRegions_get_result SchematicRegions_get(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name);

    typedef struct SchematicRegions_remove_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_remove_result;
    SchematicRegions_remove_result SchematicRegions_remove(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name);

    typedef struct SchematicRegions_names_json_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_names_json_result;
    SchematicRegions_names_json_result SchematicRegions_names_json(const diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatWrite* write);

    typedef struct SchematicRegions_create_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_create_result;
    SchematicRegions_create_result SchematicRegions_create(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name);

    typedef struct SchematicRegions_create_from_point_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_create_from_point_result;
    SchematicRegions_create_from_point_result SchematicRegions_create_from_point(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name, int32_t x, int32_t y, int32_t z);

    typedef struct SchematicRegions_create_from_bounds_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_create_from_bounds_result;
    SchematicRegions_create_from_bounds_result SchematicRegions_create_from_bounds(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct SchematicRegions_create_region_result {union {diplomat::capi::DefinitionRegion* ok; diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_create_region_result;
    SchematicRegions_create_region_result SchematicRegions_create_region(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct SchematicRegions_add_bounds_to_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_add_bounds_to_result;
    SchematicRegions_add_bounds_to_result SchematicRegions_add_bounds_to(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct SchematicRegions_add_point_to_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_add_point_to_result;
    SchematicRegions_add_point_to_result SchematicRegions_add_point_to(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name, int32_t x, int32_t y, int32_t z);

    typedef struct SchematicRegions_set_metadata_on_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_set_metadata_on_result;
    SchematicRegions_set_metadata_on_result SchematicRegions_set_metadata_on(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name, diplomat::capi::DiplomatStringView key, diplomat::capi::DiplomatStringView value);

    typedef struct SchematicRegions_shift_region_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} SchematicRegions_shift_region_result;
    SchematicRegions_shift_region_result SchematicRegions_shift_region(diplomat::capi::Schematic* schematic, diplomat::capi::DiplomatStringView name, int32_t dx, int32_t dy, int32_t dz);

    void SchematicRegions_destroy(SchematicRegions* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::add(Schematic& schematic, std::string_view name, const DefinitionRegion& region) {
    auto result = diplomat::capi::SchematicRegions_add(schematic.AsFFI(),
        {name.data(), name.size()},
        region.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::update(Schematic& schematic, std::string_view name, const DefinitionRegion& region) {
    auto result = diplomat::capi::SchematicRegions_update(schematic.AsFFI(),
        {name.data(), name.size()},
        region.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> SchematicRegions::get(const Schematic& schematic, std::string_view name) {
    auto result = diplomat::capi::SchematicRegions_get(schematic.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Ok<std::unique_ptr<DefinitionRegion>>(std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::remove(Schematic& schematic, std::string_view name) {
    auto result = diplomat::capi::SchematicRegions_remove(schematic.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> SchematicRegions::names_json(const Schematic& schematic) {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::SchematicRegions_names_json(schematic.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> SchematicRegions::names_json_write(const Schematic& schematic, W& writeable) {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::SchematicRegions_names_json(schematic.AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::create(Schematic& schematic, std::string_view name) {
    auto result = diplomat::capi::SchematicRegions_create(schematic.AsFFI(),
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::create_from_point(Schematic& schematic, std::string_view name, int32_t x, int32_t y, int32_t z) {
    auto result = diplomat::capi::SchematicRegions_create_from_point(schematic.AsFFI(),
        {name.data(), name.size()},
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::create_from_bounds(Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::SchematicRegions_create_from_bounds(schematic.AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError> SchematicRegions::create_region(Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::SchematicRegions_create_region(schematic.AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Ok<std::unique_ptr<DefinitionRegion>>(std::unique_ptr<DefinitionRegion>(DefinitionRegion::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<DefinitionRegion>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::add_bounds_to(Schematic& schematic, std::string_view name, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::SchematicRegions_add_bounds_to(schematic.AsFFI(),
        {name.data(), name.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::add_point_to(Schematic& schematic, std::string_view name, int32_t x, int32_t y, int32_t z) {
    auto result = diplomat::capi::SchematicRegions_add_point_to(schematic.AsFFI(),
        {name.data(), name.size()},
        x,
        y,
        z);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::set_metadata_on(Schematic& schematic, std::string_view name, std::string_view key, std::string_view value) {
    auto result = diplomat::capi::SchematicRegions_set_metadata_on(schematic.AsFFI(),
        {name.data(), name.size()},
        {key.data(), key.size()},
        {value.data(), value.size()});
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> SchematicRegions::shift_region(Schematic& schematic, std::string_view name, int32_t dx, int32_t dy, int32_t dz) {
    auto result = diplomat::capi::SchematicRegions_shift_region(schematic.AsFFI(),
        {name.data(), name.size()},
        dx,
        dy,
        dz);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::SchematicRegions* SchematicRegions::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::SchematicRegions*>(this);
}

inline diplomat::capi::SchematicRegions* SchematicRegions::AsFFI() {
    return reinterpret_cast<diplomat::capi::SchematicRegions*>(this);
}

inline const SchematicRegions* SchematicRegions::FromFFI(const diplomat::capi::SchematicRegions* ptr) {
    return reinterpret_cast<const SchematicRegions*>(ptr);
}

inline SchematicRegions* SchematicRegions::FromFFI(diplomat::capi::SchematicRegions* ptr) {
    return reinterpret_cast<SchematicRegions*>(ptr);
}

inline void SchematicRegions::operator delete(void* ptr) {
    diplomat::capi::SchematicRegions_destroy(reinterpret_cast<diplomat::capi::SchematicRegions*>(ptr));
}


#endif // SchematicRegions_HPP
