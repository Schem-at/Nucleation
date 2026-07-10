#ifndef NUCLEATION_ChunkMeshResult_HPP
#define NUCLEATION_ChunkMeshResult_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct ChunkMeshResult_create_result {union {nucleation::capi::ChunkMeshResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_create_result;
    ChunkMeshResult_create_result ChunkMeshResult_create(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::MeshConfig* config);

    typedef struct ChunkMeshResult_create_with_size_result {union {nucleation::capi::ChunkMeshResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_create_with_size_result;
    ChunkMeshResult_create_with_size_result ChunkMeshResult_create_with_size(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::MeshConfig* config, int32_t chunk_size);

    typedef struct ChunkMeshResult_create_with_atlas_result {union {nucleation::capi::ChunkMeshResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_create_with_atlas_result;
    ChunkMeshResult_create_with_atlas_result ChunkMeshResult_create_with_atlas(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::MeshConfig* config, int32_t chunk_size, const nucleation::capi::TextureAtlas* atlas);

    uint32_t ChunkMeshResult_chunk_count(const nucleation::capi::ChunkMeshResult* self);

    typedef struct ChunkMeshResult_chunk_coordinate_at_result {union {nucleation::capi::BlockPos ok; nucleation::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_chunk_coordinate_at_result;
    ChunkMeshResult_chunk_coordinate_at_result ChunkMeshResult_chunk_coordinate_at(const nucleation::capi::ChunkMeshResult* self, uint32_t index);

    typedef struct ChunkMeshResult_get_mesh_result {union {nucleation::capi::MeshResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} ChunkMeshResult_get_mesh_result;
    ChunkMeshResult_get_mesh_result ChunkMeshResult_get_mesh(const nucleation::capi::ChunkMeshResult* self, int32_t cx, int32_t cy, int32_t cz);

    uint32_t ChunkMeshResult_total_vertex_count(const nucleation::capi::ChunkMeshResult* self);

    uint32_t ChunkMeshResult_total_triangle_count(const nucleation::capi::ChunkMeshResult* self);

    void ChunkMeshResult_nucm_data_b64(const nucleation::capi::ChunkMeshResult* self, nucleation::diplomat::capi::DiplomatWrite* write);

    void ChunkMeshResult_nucm_data_with_atlas_b64(const nucleation::capi::ChunkMeshResult* self, const nucleation::capi::TextureAtlas* atlas, nucleation::diplomat::capi::DiplomatWrite* write);

    void ChunkMeshResult_destroy(ChunkMeshResult* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError> nucleation::ChunkMeshResult::create(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config) {
    auto result = nucleation::capi::ChunkMeshResult_create(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ChunkMeshResult>>(std::unique_ptr<nucleation::ChunkMeshResult>(nucleation::ChunkMeshResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError> nucleation::ChunkMeshResult::create_with_size(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config, int32_t chunk_size) {
    auto result = nucleation::capi::ChunkMeshResult_create_with_size(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        chunk_size);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ChunkMeshResult>>(std::unique_ptr<nucleation::ChunkMeshResult>(nucleation::ChunkMeshResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError> nucleation::ChunkMeshResult::create_with_atlas(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config, int32_t chunk_size, const nucleation::TextureAtlas& atlas) {
    auto result = nucleation::capi::ChunkMeshResult_create_with_atlas(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        chunk_size,
        atlas.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ChunkMeshResult>>(std::unique_ptr<nucleation::ChunkMeshResult>(nucleation::ChunkMeshResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::ChunkMeshResult::chunk_count() const {
    auto result = nucleation::capi::ChunkMeshResult_chunk_count(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError> nucleation::ChunkMeshResult::chunk_coordinate_at(uint32_t index) const {
    auto result = nucleation::capi::ChunkMeshResult_chunk_coordinate_at(this->AsFFI(),
        index);
    return result.is_ok ? nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::BlockPos>(nucleation::BlockPos::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::BlockPos, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError> nucleation::ChunkMeshResult::get_mesh(int32_t cx, int32_t cy, int32_t cz) const {
    auto result = nucleation::capi::ChunkMeshResult_get_mesh(this->AsFFI(),
        cx,
        cy,
        cz);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::MeshResult>>(std::unique_ptr<nucleation::MeshResult>(nucleation::MeshResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::MeshResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline uint32_t nucleation::ChunkMeshResult::total_vertex_count() const {
    auto result = nucleation::capi::ChunkMeshResult_total_vertex_count(this->AsFFI());
    return result;
}

inline uint32_t nucleation::ChunkMeshResult::total_triangle_count() const {
    auto result = nucleation::capi::ChunkMeshResult_total_triangle_count(this->AsFFI());
    return result;
}

inline std::string nucleation::ChunkMeshResult::nucm_data_b64() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::ChunkMeshResult_nucm_data_b64(this->AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::ChunkMeshResult::nucm_data_b64_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::ChunkMeshResult_nucm_data_b64(this->AsFFI(),
        &write);
}

inline std::string nucleation::ChunkMeshResult::nucm_data_with_atlas_b64(const nucleation::TextureAtlas& atlas) const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    nucleation::capi::ChunkMeshResult_nucm_data_with_atlas_b64(this->AsFFI(),
        atlas.AsFFI(),
        &write);
    return output;
}
template<typename W>
inline void nucleation::ChunkMeshResult::nucm_data_with_atlas_b64_write(const nucleation::TextureAtlas& atlas, W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    nucleation::capi::ChunkMeshResult_nucm_data_with_atlas_b64(this->AsFFI(),
        atlas.AsFFI(),
        &write);
}

inline const nucleation::capi::ChunkMeshResult* nucleation::ChunkMeshResult::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::ChunkMeshResult*>(this);
}

inline nucleation::capi::ChunkMeshResult* nucleation::ChunkMeshResult::AsFFI() {
    return reinterpret_cast<nucleation::capi::ChunkMeshResult*>(this);
}

inline const nucleation::ChunkMeshResult* nucleation::ChunkMeshResult::FromFFI(const nucleation::capi::ChunkMeshResult* ptr) {
    return reinterpret_cast<const nucleation::ChunkMeshResult*>(ptr);
}

inline nucleation::ChunkMeshResult* nucleation::ChunkMeshResult::FromFFI(nucleation::capi::ChunkMeshResult* ptr) {
    return reinterpret_cast<nucleation::ChunkMeshResult*>(ptr);
}

inline void nucleation::ChunkMeshResult::operator delete(void* ptr) {
    nucleation::capi::ChunkMeshResult_destroy(reinterpret_cast<nucleation::capi::ChunkMeshResult*>(ptr));
}


#endif // NUCLEATION_ChunkMeshResult_HPP
