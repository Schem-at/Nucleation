#ifndef WorldGenerator_HPP
#define WorldGenerator_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct WorldGenerator_sdf_result {union {diplomat::capi::WorldGenerator* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldGenerator_sdf_result;
    WorldGenerator_sdf_result WorldGenerator_sdf(const diplomat::capi::Sdf* volume, const diplomat::capi::Brush* material, int32_t min_y, int32_t max_y, diplomat::capi::DiplomatStringView source_id, diplomat::capi::DiplomatStringView version);

    typedef struct WorldGenerator_cellular_sdf_result {union {diplomat::capi::WorldGenerator* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldGenerator_cellular_sdf_result;
    WorldGenerator_cellular_sdf_result WorldGenerator_cellular_sdf(const diplomat::capi::Sdf* volume, const diplomat::capi::Brush* material, int32_t min_y, int32_t max_y, const diplomat::capi::CellularSdfConfig* config, diplomat::capi::DiplomatStringView source_id, diplomat::capi::DiplomatStringView version);

    typedef struct WorldGenerator_projected_footprints_result {union {diplomat::capi::WorldGenerator* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldGenerator_projected_footprints_result;
    WorldGenerator_projected_footprints_result WorldGenerator_projected_footprints(diplomat::capi::DiplomatStringView buildings_json, diplomat::capi::DiplomatStringView base_block, diplomat::capi::DiplomatStringView source_id, diplomat::capi::DiplomatStringView version);

    typedef struct WorldGenerator_composite_result {union {diplomat::capi::WorldGenerator* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldGenerator_composite_result;
    WorldGenerator_composite_result WorldGenerator_composite(diplomat::capi::DiplomatStringView source_id, diplomat::capi::DiplomatStringView version);

    typedef struct WorldGenerator_add_layer_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WorldGenerator_add_layer_result;
    WorldGenerator_add_layer_result WorldGenerator_add_layer(diplomat::capi::WorldGenerator* self, const diplomat::capi::WorldGenerator* source, diplomat::capi::GeneratedChunkOverlayMode mode);

    typedef struct WorldGenerator_generate_result {union {diplomat::capi::GeneratedChunk* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldGenerator_generate_result;
    WorldGenerator_generate_result WorldGenerator_generate(const diplomat::capi::WorldGenerator* self, int32_t cx, int32_t cz);

    typedef struct WorldGenerator_stream_result {union {diplomat::capi::GeneratedWorldStream* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldGenerator_stream_result;
    WorldGenerator_stream_result WorldGenerator_stream(const diplomat::capi::WorldGenerator* self, int32_t min_cx, int32_t min_cz, int32_t max_cx, int32_t max_cz);

    void WorldGenerator_destroy(WorldGenerator* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError> WorldGenerator::sdf(const Sdf& volume, const Brush& material, int32_t min_y, int32_t max_y, std::string_view source_id, std::string_view version) {
    auto result = diplomat::capi::WorldGenerator_sdf(volume.AsFFI(),
        material.AsFFI(),
        min_y,
        max_y,
        {source_id.data(), source_id.size()},
        {version.data(), version.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldGenerator>>(std::unique_ptr<WorldGenerator>(WorldGenerator::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError> WorldGenerator::cellular_sdf(const Sdf& volume, const Brush& material, int32_t min_y, int32_t max_y, const CellularSdfConfig& config, std::string_view source_id, std::string_view version) {
    auto result = diplomat::capi::WorldGenerator_cellular_sdf(volume.AsFFI(),
        material.AsFFI(),
        min_y,
        max_y,
        config.AsFFI(),
        {source_id.data(), source_id.size()},
        {version.data(), version.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldGenerator>>(std::unique_ptr<WorldGenerator>(WorldGenerator::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError> WorldGenerator::projected_footprints(std::string_view buildings_json, std::string_view base_block, std::string_view source_id, std::string_view version) {
    auto result = diplomat::capi::WorldGenerator_projected_footprints({buildings_json.data(), buildings_json.size()},
        {base_block.data(), base_block.size()},
        {source_id.data(), source_id.size()},
        {version.data(), version.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldGenerator>>(std::unique_ptr<WorldGenerator>(WorldGenerator::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError> WorldGenerator::composite(std::string_view source_id, std::string_view version) {
    auto result = diplomat::capi::WorldGenerator_composite({source_id.data(), source_id.size()},
        {version.data(), version.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldGenerator>>(std::unique_ptr<WorldGenerator>(WorldGenerator::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldGenerator>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> WorldGenerator::add_layer(const WorldGenerator& source, GeneratedChunkOverlayMode mode) {
    auto result = diplomat::capi::WorldGenerator_add_layer(this->AsFFI(),
        source.AsFFI(),
        mode.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<GeneratedChunk>, NucleationError> WorldGenerator::generate(int32_t cx, int32_t cz) const {
    auto result = diplomat::capi::WorldGenerator_generate(this->AsFFI(),
        cx,
        cz);
    return result.is_ok ? diplomat::result<std::unique_ptr<GeneratedChunk>, NucleationError>(diplomat::Ok<std::unique_ptr<GeneratedChunk>>(std::unique_ptr<GeneratedChunk>(GeneratedChunk::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<GeneratedChunk>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<GeneratedWorldStream>, NucleationError> WorldGenerator::stream(int32_t min_cx, int32_t min_cz, int32_t max_cx, int32_t max_cz) const {
    auto result = diplomat::capi::WorldGenerator_stream(this->AsFFI(),
        min_cx,
        min_cz,
        max_cx,
        max_cz);
    return result.is_ok ? diplomat::result<std::unique_ptr<GeneratedWorldStream>, NucleationError>(diplomat::Ok<std::unique_ptr<GeneratedWorldStream>>(std::unique_ptr<GeneratedWorldStream>(GeneratedWorldStream::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<GeneratedWorldStream>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::WorldGenerator* WorldGenerator::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::WorldGenerator*>(this);
}

inline diplomat::capi::WorldGenerator* WorldGenerator::AsFFI() {
    return reinterpret_cast<diplomat::capi::WorldGenerator*>(this);
}

inline const WorldGenerator* WorldGenerator::FromFFI(const diplomat::capi::WorldGenerator* ptr) {
    return reinterpret_cast<const WorldGenerator*>(ptr);
}

inline WorldGenerator* WorldGenerator::FromFFI(diplomat::capi::WorldGenerator* ptr) {
    return reinterpret_cast<WorldGenerator*>(ptr);
}

inline void WorldGenerator::operator delete(void* ptr) {
    diplomat::capi::WorldGenerator_destroy(reinterpret_cast<diplomat::capi::WorldGenerator*>(ptr));
}


#endif // WorldGenerator_HPP
