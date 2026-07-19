#ifndef NUCLEATION_Geo_HPP
#define NUCLEATION_Geo_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct Geo_extrude_footprints_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Geo_extrude_footprints_result;
    Geo_extrude_footprints_result Geo_extrude_footprints(nucleation::diplomat::capi::DiplomatStringView buildings_json, nucleation::diplomat::capi::DiplomatStringView base_block, nucleation::diplomat::capi::DiplomatStringView name);

    typedef struct Geo_heightmap_terrain_result {union {nucleation::capi::Schematic* ok; nucleation::capi::NucleationError err;}; bool is_ok;} Geo_heightmap_terrain_result;
    Geo_heightmap_terrain_result Geo_heightmap_terrain(nucleation::diplomat::capi::DiplomatStringView heights_json, int32_t width, nucleation::diplomat::capi::DiplomatStringView surface_blocks_json, nucleation::diplomat::capi::DiplomatStringView subsurface_block, int32_t surface_depth, nucleation::diplomat::capi::DiplomatStringView name);

    void Geo_destroy(Geo* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Geo::extrude_footprints(std::string_view buildings_json, std::string_view base_block, std::string_view name) {
    auto result = nucleation::capi::Geo_extrude_footprints({buildings_json.data(), buildings_json.size()},
        {base_block.data(), base_block.size()},
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> nucleation::Geo::heightmap_terrain(std::string_view heights_json, int32_t width, std::string_view surface_blocks_json, std::string_view subsurface_block, int32_t surface_depth, std::string_view name) {
    auto result = nucleation::capi::Geo_heightmap_terrain({heights_json.data(), heights_json.size()},
        width,
        {surface_blocks_json.data(), surface_blocks_json.size()},
        {subsurface_block.data(), subsurface_block.size()},
        surface_depth,
        {name.data(), name.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::Schematic>>(std::unique_ptr<nucleation::Schematic>(nucleation::Schematic::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::Geo* nucleation::Geo::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::Geo*>(this);
}

inline nucleation::capi::Geo* nucleation::Geo::AsFFI() {
    return reinterpret_cast<nucleation::capi::Geo*>(this);
}

inline const nucleation::Geo* nucleation::Geo::FromFFI(const nucleation::capi::Geo* ptr) {
    return reinterpret_cast<const nucleation::Geo*>(ptr);
}

inline nucleation::Geo* nucleation::Geo::FromFFI(nucleation::capi::Geo* ptr) {
    return reinterpret_cast<nucleation::Geo*>(ptr);
}

inline void nucleation::Geo::operator delete(void* ptr) {
    nucleation::capi::Geo_destroy(reinterpret_cast<nucleation::capi::Geo*>(ptr));
}


#endif // NUCLEATION_Geo_HPP
