#ifndef MeshJob_HPP
#define MeshJob_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    diplomat::capi::MeshJob* MeshJob_start(const diplomat::capi::Schematic* schematic, const diplomat::capi::ResourcePack* pack, const diplomat::capi::MeshConfig* config, int32_t chunk_size, const diplomat::capi::TextureAtlas* atlas);

    diplomat::capi::MeshProgress MeshJob_poll_progress(const diplomat::capi::MeshJob* self);

    typedef struct MeshJob_take_result_result {union {diplomat::capi::ChunkMeshResult* ok; diplomat::capi::NucleationError err;}; bool is_ok;} MeshJob_take_result_result;
    MeshJob_take_result_result MeshJob_take_result(diplomat::capi::MeshJob* self);

    void MeshJob_destroy(MeshJob* self);

    } // extern "C"
} // namespace capi
} // namespace

inline std::unique_ptr<MeshJob> MeshJob::start(const Schematic& schematic, const ResourcePack& pack, const MeshConfig& config, int32_t chunk_size, const TextureAtlas& atlas) {
    auto result = diplomat::capi::MeshJob_start(schematic.AsFFI(),
        pack.AsFFI(),
        config.AsFFI(),
        chunk_size,
        atlas.AsFFI());
    return std::unique_ptr<MeshJob>(MeshJob::FromFFI(result));
}

inline MeshProgress MeshJob::poll_progress() const {
    auto result = diplomat::capi::MeshJob_poll_progress(this->AsFFI());
    return MeshProgress::FromFFI(result);
}

inline diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError> MeshJob::take_result() {
    auto result = diplomat::capi::MeshJob_take_result(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError>(diplomat::Ok<std::unique_ptr<ChunkMeshResult>>(std::unique_ptr<ChunkMeshResult>(ChunkMeshResult::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<ChunkMeshResult>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::MeshJob* MeshJob::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::MeshJob*>(this);
}

inline diplomat::capi::MeshJob* MeshJob::AsFFI() {
    return reinterpret_cast<diplomat::capi::MeshJob*>(this);
}

inline const MeshJob* MeshJob::FromFFI(const diplomat::capi::MeshJob* ptr) {
    return reinterpret_cast<const MeshJob*>(ptr);
}

inline MeshJob* MeshJob::FromFFI(diplomat::capi::MeshJob* ptr) {
    return reinterpret_cast<MeshJob*>(ptr);
}

inline void MeshJob::operator delete(void* ptr) {
    diplomat::capi::MeshJob_destroy(reinterpret_cast<diplomat::capi::MeshJob*>(ptr));
}


#endif // MeshJob_HPP
