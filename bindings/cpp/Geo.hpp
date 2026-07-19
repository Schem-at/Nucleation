#ifndef Geo_HPP
#define Geo_HPP

#include "Geo.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "Schematic.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct Geo_extrude_footprints_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Geo_extrude_footprints_result;
    Geo_extrude_footprints_result Geo_extrude_footprints(diplomat::capi::DiplomatStringView buildings_json, diplomat::capi::DiplomatStringView base_block, diplomat::capi::DiplomatStringView name);

    typedef struct Geo_heightmap_terrain_result {union {diplomat::capi::Schematic* ok; diplomat::capi::NucleationError err;}; bool is_ok;} Geo_heightmap_terrain_result;
    Geo_heightmap_terrain_result Geo_heightmap_terrain(diplomat::capi::DiplomatStringView heights_json, int32_t width, diplomat::capi::DiplomatStringView surface_blocks_json, diplomat::capi::DiplomatStringView subsurface_block, int32_t surface_depth, diplomat::capi::DiplomatStringView name);

    void Geo_destroy(Geo* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Geo::extrude_footprints(std::string_view buildings_json, std::string_view base_block, std::string_view name) {
    auto result = diplomat::capi::Geo_extrude_footprints({buildings_json.data(), buildings_json.size()},
        {base_block.data(), base_block.size()},
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> Geo::heightmap_terrain(std::string_view heights_json, int32_t width, std::string_view surface_blocks_json, std::string_view subsurface_block, int32_t surface_depth, std::string_view name) {
    auto result = diplomat::capi::Geo_heightmap_terrain({heights_json.data(), heights_json.size()},
        width,
        {surface_blocks_json.data(), surface_blocks_json.size()},
        {subsurface_block.data(), subsurface_block.size()},
        surface_depth,
        {name.data(), name.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Ok<std::unique_ptr<Schematic>>(std::unique_ptr<Schematic>(Schematic::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<Schematic>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::Geo* Geo::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::Geo*>(this);
}

inline diplomat::capi::Geo* Geo::AsFFI() {
    return reinterpret_cast<diplomat::capi::Geo*>(this);
}

inline const Geo* Geo::FromFFI(const diplomat::capi::Geo* ptr) {
    return reinterpret_cast<const Geo*>(ptr);
}

inline Geo* Geo::FromFFI(diplomat::capi::Geo* ptr) {
    return reinterpret_cast<Geo*>(ptr);
}

inline void Geo::operator delete(void* ptr) {
    diplomat::capi::Geo_destroy(reinterpret_cast<diplomat::capi::Geo*>(ptr));
}


#endif // Geo_HPP
