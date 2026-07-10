#ifndef WorldStream_HPP
#define WorldStream_HPP

#include "WorldStream.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "NucleationError.hpp"
#include "WorldChunkView.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct WorldStream_open_dir_result {union {diplomat::capi::WorldStream* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldStream_open_dir_result;
    WorldStream_open_dir_result WorldStream_open_dir(diplomat::capi::DiplomatStringView path);

    typedef struct WorldStream_open_dir_bounded_result {union {diplomat::capi::WorldStream* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldStream_open_dir_bounded_result;
    WorldStream_open_dir_bounded_result WorldStream_open_dir_bounded(diplomat::capi::DiplomatStringView path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct WorldStream_from_zip_result {union {diplomat::capi::WorldStream* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldStream_from_zip_result;
    WorldStream_from_zip_result WorldStream_from_zip(diplomat::capi::DiplomatU8View data);

    typedef struct WorldStream_from_zip_bounded_result {union {diplomat::capi::WorldStream* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldStream_from_zip_bounded_result;
    WorldStream_from_zip_bounded_result WorldStream_from_zip_bounded(diplomat::capi::DiplomatU8View data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct WorldStream_next_result {union {diplomat::capi::WorldChunkView* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldStream_next_result;
    WorldStream_next_result WorldStream_next(diplomat::capi::WorldStream* self);

    void WorldStream_destroy(WorldStream* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<WorldStream>, NucleationError> WorldStream::open_dir(std::string_view path) {
    auto result = diplomat::capi::WorldStream_open_dir({path.data(), path.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldStream>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldStream>>(std::unique_ptr<WorldStream>(WorldStream::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldStream>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<WorldStream>, NucleationError> WorldStream::open_dir_bounded(std::string_view path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::WorldStream_open_dir_bounded({path.data(), path.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldStream>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldStream>>(std::unique_ptr<WorldStream>(WorldStream::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldStream>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<WorldStream>, NucleationError> WorldStream::from_zip(diplomat::span<const uint8_t> data) {
    auto result = diplomat::capi::WorldStream_from_zip({data.data(), data.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldStream>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldStream>>(std::unique_ptr<WorldStream>(WorldStream::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldStream>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<WorldStream>, NucleationError> WorldStream::from_zip_bounded(diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = diplomat::capi::WorldStream_from_zip_bounded({data.data(), data.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldStream>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldStream>>(std::unique_ptr<WorldStream>(WorldStream::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldStream>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<WorldChunkView>, NucleationError> WorldStream::next() {
    auto result = diplomat::capi::WorldStream_next(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldChunkView>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldChunkView>>(std::unique_ptr<WorldChunkView>(WorldChunkView::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldChunkView>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::WorldStream* WorldStream::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::WorldStream*>(this);
}

inline diplomat::capi::WorldStream* WorldStream::AsFFI() {
    return reinterpret_cast<diplomat::capi::WorldStream*>(this);
}

inline const WorldStream* WorldStream::FromFFI(const diplomat::capi::WorldStream* ptr) {
    return reinterpret_cast<const WorldStream*>(ptr);
}

inline WorldStream* WorldStream::FromFFI(diplomat::capi::WorldStream* ptr) {
    return reinterpret_cast<WorldStream*>(ptr);
}

inline void WorldStream::operator delete(void* ptr) {
    diplomat::capi::WorldStream_destroy(reinterpret_cast<diplomat::capi::WorldStream*>(ptr));
}


#endif // WorldStream_HPP
