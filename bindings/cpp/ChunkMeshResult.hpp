#ifndef ChunkMeshResult_HPP
#define ChunkMeshResult_HPP

#include "ChunkMeshResult.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "BlockPos.hpp"
#include "MeshConfig.hpp"
#include "MeshResult.hpp"
#include "NucleationError.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "TextureAtlas.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct ChunkMeshResult_create_result {union {diplomat::capi::ChunkMeshResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_create_result;
    ChunkMeshResult_create_result ChunkMeshResult_create(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::MeshConfig* config);

    typedef struct ChunkMeshResult_create_with_size_result {union {diplomat::capi::ChunkMeshResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_create_with_size_result;
    ChunkMeshResult_create_with_size_result ChunkMeshResult_create_with_size(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::MeshConfig* config, int32_t chunk_size);

    typedef struct ChunkMeshResult_create_with_atlas_result {union {diplomat::capi::ChunkMeshResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_create_with_atlas_result;
    ChunkMeshResult_create_with_atlas_result ChunkMeshResult_create_with_atlas(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::MeshConfig* config, int32_t chunk_size, const diplomat::capi::TextureAtlas* atlas);

    uint32_t ChunkMeshResult_chunk_count(const diplomat::capi::ChunkMeshResult* self);

    typedef struct ChunkMeshResult_chunk_coordinate_at_result {union {diplomat::capi::BlockPos ok; diplomat::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_chunk_coordinate_at_result;
    ChunkMeshResult_chunk_coordinate_at_result ChunkMeshResult_chunk_coordinate_at(const diplomat::capi::ChunkMeshResult* self, uint32_t index);

    typedef struct ChunkMeshResult_get_mesh_result {union {diplomat::capi::MeshResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_get_mesh_result;
    ChunkMeshResult_get_mesh_result ChunkMeshResult_get_mesh(const diplomat::capi::ChunkMeshResult* self, int32_t cx, int32_t cy, int32_t cz);

    uint32_t ChunkMeshResult_total_vertex_count(const diplomat::capi::ChunkMeshResult* self);

    uint32_t ChunkMeshResult_total_triangle_count(const diplomat::capi::ChunkMeshResult* self);

    void ChunkMeshResult_nucm_data_b64(const diplomat::capi::ChunkMeshResult* self, diplomat::capi::DiplomatWrite* write);

    void ChunkMeshResult_nucm_data_with_atlas_b64(const diplomat::capi::ChunkMeshResult* self, const diplomat::capi::TextureAtlas* atlas, diplomat::capi::DiplomatWrite* write);

    void ChunkMeshResult_destroy(ChunkMeshResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError> ChunkMeshResult::create(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config) {
    auto result = diplomat::capi::ChunkMeshResult_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError>(diplomat::Ok<std::unique_ptr<ChunkMeshResult>>(std::unique_ptr<ChunkMeshResult>(ChunkMeshResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError> ChunkMeshResult::create_with_size(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config, int32_t chunk_size) {
    auto result = diplomat::capi::ChunkMeshResult_create_with_size(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        chunk_size);
    return result.is_ok ? diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError>(diplomat::Ok<std::unique_ptr<ChunkMeshResult>>(std::unique_ptr<ChunkMeshResult>(ChunkMeshResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError> ChunkMeshResult::create_with_atlas(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config, int32_t chunk_size, const TextureAtlas& atlas) {
    auto result = diplomat::capi::ChunkMeshResult_create_with_atlas(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        chunk_size,
        atlas.AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError>(diplomat::Ok<std::unique_ptr<ChunkMeshResult>>(std::unique_ptr<ChunkMeshResult>(ChunkMeshResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t ChunkMeshResult::chunk_count() const {
    auto result = diplomat::capi::ChunkMeshResult_chunk_count(this->AsFFI());
    return result;
}

inline diplomat::result<BlockPos, NucleationError> ChunkMeshResult::chunk_coordinate_at(uint32_t index) const {
    auto result = diplomat::capi::ChunkMeshResult_chunk_coordinate_at(this->AsFFI(),
        index);
    return result.is_ok ? diplomat::result<BlockPos, NucleationError>(diplomat::Ok<BlockPos>(BlockPos::FromFFI(result.ok))) : diplomat::result<BlockPos, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<MeshResult>, NucleationError> ChunkMeshResult::get_mesh(int32_t cx, int32_t cy, int32_t cz) const {
    auto result = diplomat::capi::ChunkMeshResult_get_mesh(this->AsFFI(),
        cx,
        cy,
        cz);
    return result.is_ok ? diplomat::result<std::unique_ptr<MeshResult>, NucleationError>(diplomat::Ok<std::unique_ptr<MeshResult>>(std::unique_ptr<MeshResult>(MeshResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<MeshResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline uint32_t ChunkMeshResult::total_vertex_count() const {
    auto result = diplomat::capi::ChunkMeshResult_total_vertex_count(this->AsFFI());
    return result;
}

inline uint32_t ChunkMeshResult::total_triangle_count() const {
    auto result = diplomat::capi::ChunkMeshResult_total_triangle_count(this->AsFFI());
    return result;
}

inline std::string ChunkMeshResult::nucm_data_b64() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::ChunkMeshResult_nucm_data_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void ChunkMeshResult::nucm_data_b64_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::ChunkMeshResult_nucm_data_b64(this->AsFFI(),
        &write);
}

inline std::string ChunkMeshResult::nucm_data_with_atlas_b64(const TextureAtlas& atlas) const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    diplomat::capi::ChunkMeshResult_nucm_data_with_atlas_b64(this->AsFFI(),
        atlas.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void ChunkMeshResult::nucm_data_with_atlas_b64_write(const TextureAtlas& atlas, W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    diplomat::capi::ChunkMeshResult_nucm_data_with_atlas_b64(this->AsFFI(),
        atlas.AsFFI(),
        &write);
}

inline const diplomat::capi::ChunkMeshResult* ChunkMeshResult::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::ChunkMeshResult*>(this);
}

inline diplomat::capi::ChunkMeshResult* ChunkMeshResult::AsFFI() {
    return reinterpret_cast<diplomat::capi::ChunkMeshResult*>(this);
}

inline const ChunkMeshResult* ChunkMeshResult::FromFFI(const diplomat::capi::ChunkMeshResult* ptr) {
    return reinterpret_cast<const ChunkMeshResult*>(ptr);
}

inline ChunkMeshResult* ChunkMeshResult::FromFFI(diplomat::capi::ChunkMeshResult* ptr) {
    return reinterpret_cast<ChunkMeshResult*>(ptr);
}

inline void ChunkMeshResult::operator delete(void* ptr) {
    diplomat::capi::ChunkMeshResult_destroy(reinterpret_cast<diplomat::capi::ChunkMeshResult*>(ptr));
}


#endif // ChunkMeshResult_HPP
