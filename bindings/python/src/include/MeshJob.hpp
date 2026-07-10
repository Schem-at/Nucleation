#ifndef NUCLEATION_MeshJob_HPP
#define NUCLEATION_MeshJob_HPP

#include "MeshJob.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "ChunkMeshResult.hpp"
#include "MeshConfig.hpp"
#include "MeshProgress.hpp"
#include "NucleationError.hpp"
#include "ResourcePack.hpp"
#include "Schematic.hpp"
#include "TextureAtlas.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    nucleation::capi::MeshJob* MeshJob_start(const nucleation::capi::Schematic* schematic, const nucleation::capi::ResourcePack* pack, const nucleation::capi::MeshConfig* config, int32_t chunk_size, const nucleation::capi::TextureAtlas* atlas);

    nucleation::capi::MeshProgress MeshJob_poll_progress(const nucleation::capi::MeshJob* self);

    typedef struct MeshJob_take_result_result {union {nucleation::capi::ChunkMeshResult* ok; nucleation::capi::NucleationError err;}; bool is_ok;} MeshJob_take_result_result;
    MeshJob_take_result_result MeshJob_take_result(nucleation::capi::MeshJob* self);

    void MeshJob_destroy(MeshJob* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<nucleation::MeshJob> nucleation::MeshJob::start(const nucleation::Schematic& schematic, const nucleation::ResourcePack& pack, const nucleation::MeshConfig& config, int32_t chunk_size, const nucleation::TextureAtlas& atlas) {
    auto result = nucleation::capi::MeshJob_start(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        chunk_size,
        atlas.AsFFI());
    return std::unique_ptr<nucleation::MeshJob>(nucleation::MeshJob::FromFFI(result));
}

inline nucleation::MeshProgress nucleation::MeshJob::poll_progress() const {
    auto result = nucleation::capi::MeshJob_poll_progress(this->AsFFI());
    return nucleation::MeshProgress::FromFFI(result);
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError> nucleation::MeshJob::take_result() {
    auto result = nucleation::capi::MeshJob_take_result(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::ChunkMeshResult>>(std::unique_ptr<nucleation::ChunkMeshResult>(nucleation::ChunkMeshResult::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::ChunkMeshResult>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::MeshJob* nucleation::MeshJob::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::MeshJob*>(this);
}

inline nucleation::capi::MeshJob* nucleation::MeshJob::AsFFI() {
    return reinterpret_cast<nucleation::capi::MeshJob*>(this);
}

inline const nucleation::MeshJob* nucleation::MeshJob::FromFFI(const nucleation::capi::MeshJob* ptr) {
    return reinterpret_cast<const nucleation::MeshJob*>(ptr);
}

inline nucleation::MeshJob* nucleation::MeshJob::FromFFI(nucleation::capi::MeshJob* ptr) {
    return reinterpret_cast<nucleation::MeshJob*>(ptr);
}

inline void nucleation::MeshJob::operator delete(void* ptr) {
    nucleation::capi::MeshJob_destroy(reinterpret_cast<nucleation::capi::MeshJob*>(ptr));
}


#endif // NUCLEATION_MeshJob_HPP
