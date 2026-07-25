#ifndef NUCLEATION_WorldGenerator_HPP
#define NUCLEATION_WorldGenerator_HPP

#include "WorldGenerator.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "Brush.hpp"
#include "CellularSdfConfig.hpp"
#include "GeneratedChunk.hpp"
#include "GeneratedChunkOverlayMode.hpp"
#include "GeneratedWorldStream.hpp"
#include "NucleationError.hpp"
#include "Sdf.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct WorldGenerator_sdf_result {union {nucleation::capi::WorldGenerator* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldGenerator_sdf_result;
    WorldGenerator_sdf_result WorldGenerator_sdf(const nucleation::capi::Sdf* volume, const nucleation::capi::Brush* material, int32_t min_y, int32_t max_y, nucleation::diplomat::capi::DiplomatStringView source_id, nucleation::diplomat::capi::DiplomatStringView version);

    typedef struct WorldGenerator_cellular_sdf_result {union {nucleation::capi::WorldGenerator* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldGenerator_cellular_sdf_result;
    WorldGenerator_cellular_sdf_result WorldGenerator_cellular_sdf(const nucleation::capi::Sdf* volume, const nucleation::capi::Brush* material, int32_t min_y, int32_t max_y, const nucleation::capi::CellularSdfConfig* config, nucleation::diplomat::capi::DiplomatStringView source_id, nucleation::diplomat::capi::DiplomatStringView version);

    typedef struct WorldGenerator_projected_footprints_result {union {nucleation::capi::WorldGenerator* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldGenerator_projected_footprints_result;
    WorldGenerator_projected_footprints_result WorldGenerator_projected_footprints(nucleation::diplomat::capi::DiplomatStringView buildings_json, nucleation::diplomat::capi::DiplomatStringView base_block, nucleation::diplomat::capi::DiplomatStringView source_id, nucleation::diplomat::capi::DiplomatStringView version);

    typedef struct WorldGenerator_composite_result {union {nucleation::capi::WorldGenerator* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldGenerator_composite_result;
    WorldGenerator_composite_result WorldGenerator_composite(nucleation::diplomat::capi::DiplomatStringView source_id, nucleation::diplomat::capi::DiplomatStringView version);

    typedef struct WorldGenerator_add_layer_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WorldGenerator_add_layer_result;
    WorldGenerator_add_layer_result WorldGenerator_add_layer(nucleation::capi::WorldGenerator* self, const nucleation::capi::WorldGenerator* source, nucleation::capi::GeneratedChunkOverlayMode mode);

    typedef struct WorldGenerator_generate_result {union {nucleation::capi::GeneratedChunk* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldGenerator_generate_result;
    WorldGenerator_generate_result WorldGenerator_generate(const nucleation::capi::WorldGenerator* self, int32_t cx, int32_t cz);

    typedef struct WorldGenerator_stream_result {union {nucleation::capi::GeneratedWorldStream* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldGenerator_stream_result;
    WorldGenerator_stream_result WorldGenerator_stream(const nucleation::capi::WorldGenerator* self, int32_t min_cx, int32_t min_cz, int32_t max_cx, int32_t max_cz);

    void WorldGenerator_destroy(WorldGenerator* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError> nucleation::WorldGenerator::sdf(const nucleation::Sdf& volume, const nucleation::Brush& material, int32_t min_y, int32_t max_y, std::string_view source_id, std::string_view version) {
    auto result = nucleation::capi::WorldGenerator_sdf(volume.AsFFI(),
        material.AsFFI(),
        min_y,
        max_y,
        {source_id.data(), source_id.size()},
        {version.data(), version.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldGenerator>>(std::unique_ptr<nucleation::WorldGenerator>(nucleation::WorldGenerator::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError> nucleation::WorldGenerator::cellular_sdf(const nucleation::Sdf& volume, const nucleation::Brush& material, int32_t min_y, int32_t max_y, const nucleation::CellularSdfConfig& config, std::string_view source_id, std::string_view version) {
    auto result = nucleation::capi::WorldGenerator_cellular_sdf(volume.AsFFI(),
        material.AsFFI(),
        min_y,
        max_y,
        config.AsFFI(),
        {source_id.data(), source_id.size()},
        {version.data(), version.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldGenerator>>(std::unique_ptr<nucleation::WorldGenerator>(nucleation::WorldGenerator::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError> nucleation::WorldGenerator::projected_footprints(std::string_view buildings_json, std::string_view base_block, std::string_view source_id, std::string_view version) {
    auto result = nucleation::capi::WorldGenerator_projected_footprints({buildings_json.data(), buildings_json.size()},
        {base_block.data(), base_block.size()},
        {source_id.data(), source_id.size()},
        {version.data(), version.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldGenerator>>(std::unique_ptr<nucleation::WorldGenerator>(nucleation::WorldGenerator::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError> nucleation::WorldGenerator::composite(std::string_view source_id, std::string_view version) {
    auto result = nucleation::capi::WorldGenerator_composite({source_id.data(), source_id.size()},
        {version.data(), version.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldGenerator>>(std::unique_ptr<nucleation::WorldGenerator>(nucleation::WorldGenerator::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldGenerator>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WorldGenerator::add_layer(const nucleation::WorldGenerator& source, nucleation::GeneratedChunkOverlayMode mode) {
    auto result = nucleation::capi::WorldGenerator_add_layer(this->AsFFI(),
        source.AsFFI(),
        mode.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedChunk>, nucleation::NucleationError> nucleation::WorldGenerator::generate(int32_t cx, int32_t cz) const {
    auto result = nucleation::capi::WorldGenerator_generate(this->AsFFI(),
        cx,
        cz);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedChunk>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::GeneratedChunk>>(std::unique_ptr<nucleation::GeneratedChunk>(nucleation::GeneratedChunk::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedChunk>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedWorldStream>, nucleation::NucleationError> nucleation::WorldGenerator::stream(int32_t min_cx, int32_t min_cz, int32_t max_cx, int32_t max_cz) const {
    auto result = nucleation::capi::WorldGenerator_stream(this->AsFFI(),
        min_cx,
        min_cz,
        max_cx,
        max_cz);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedWorldStream>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::GeneratedWorldStream>>(std::unique_ptr<nucleation::GeneratedWorldStream>(nucleation::GeneratedWorldStream::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedWorldStream>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::WorldGenerator* nucleation::WorldGenerator::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::WorldGenerator*>(this);
}

inline nucleation::capi::WorldGenerator* nucleation::WorldGenerator::AsFFI() {
    return reinterpret_cast<nucleation::capi::WorldGenerator*>(this);
}

inline const nucleation::WorldGenerator* nucleation::WorldGenerator::FromFFI(const nucleation::capi::WorldGenerator* ptr) {
    return reinterpret_cast<const nucleation::WorldGenerator*>(ptr);
}

inline nucleation::WorldGenerator* nucleation::WorldGenerator::FromFFI(nucleation::capi::WorldGenerator* ptr) {
    return reinterpret_cast<nucleation::WorldGenerator*>(ptr);
}

inline void nucleation::WorldGenerator::operator delete(void* ptr) {
    nucleation::capi::WorldGenerator_destroy(reinterpret_cast<nucleation::capi::WorldGenerator*>(ptr));
}


#endif // NUCLEATION_WorldGenerator_HPP
