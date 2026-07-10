#ifndef NUCLEATION_WorldStream_HPP
#define NUCLEATION_WorldStream_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct WorldStream_open_dir_result {union {nucleation::capi::WorldStream* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldStream_open_dir_result;
    WorldStream_open_dir_result WorldStream_open_dir(nucleation::diplomat::capi::DiplomatStringView path);

    typedef struct WorldStream_open_dir_bounded_result {union {nucleation::capi::WorldStream* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldStream_open_dir_bounded_result;
    WorldStream_open_dir_bounded_result WorldStream_open_dir_bounded(nucleation::diplomat::capi::DiplomatStringView path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct WorldStream_from_zip_result {union {nucleation::capi::WorldStream* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldStream_from_zip_result;
    WorldStream_from_zip_result WorldStream_from_zip(nucleation::diplomat::capi::DiplomatU8View data);

    typedef struct WorldStream_from_zip_bounded_result {union {nucleation::capi::WorldStream* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldStream_from_zip_bounded_result;
    WorldStream_from_zip_bounded_result WorldStream_from_zip_bounded(nucleation::diplomat::capi::DiplomatU8View data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

    typedef struct WorldStream_next_result {union {nucleation::capi::WorldChunkView* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldStream_next_result;
    WorldStream_next_result WorldStream_next(nucleation::capi::WorldStream* self);

    void WorldStream_destroy(WorldStream* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError> nucleation::WorldStream::open_dir(std::string_view path) {
    auto result = nucleation::capi::WorldStream_open_dir({path.data(), path.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldStream>>(std::unique_ptr<nucleation::WorldStream>(nucleation::WorldStream::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError> nucleation::WorldStream::open_dir_bounded(std::string_view path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::WorldStream_open_dir_bounded({path.data(), path.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldStream>>(std::unique_ptr<nucleation::WorldStream>(nucleation::WorldStream::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError> nucleation::WorldStream::from_zip(nucleation::diplomat::span<const uint8_t> data) {
    auto result = nucleation::capi::WorldStream_from_zip({data.data(), data.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldStream>>(std::unique_ptr<nucleation::WorldStream>(nucleation::WorldStream::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError> nucleation::WorldStream::from_zip_bounded(nucleation::diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z) {
    auto result = nucleation::capi::WorldStream_from_zip_bounded({data.data(), data.size()},
        min_x,
        min_y,
        min_z,
        max_x,
        max_y,
        max_z);
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldStream>>(std::unique_ptr<nucleation::WorldStream>(nucleation::WorldStream::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldChunkView>, nucleation::NucleationError> nucleation::WorldStream::next() {
    auto result = nucleation::capi::WorldStream_next(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldChunkView>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldChunkView>>(std::unique_ptr<nucleation::WorldChunkView>(nucleation::WorldChunkView::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldChunkView>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::WorldStream* nucleation::WorldStream::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::WorldStream*>(this);
}

inline nucleation::capi::WorldStream* nucleation::WorldStream::AsFFI() {
    return reinterpret_cast<nucleation::capi::WorldStream*>(this);
}

inline const nucleation::WorldStream* nucleation::WorldStream::FromFFI(const nucleation::capi::WorldStream* ptr) {
    return reinterpret_cast<const nucleation::WorldStream*>(ptr);
}

inline nucleation::WorldStream* nucleation::WorldStream::FromFFI(nucleation::capi::WorldStream* ptr) {
    return reinterpret_cast<nucleation::WorldStream*>(ptr);
}

inline void nucleation::WorldStream::operator delete(void* ptr) {
    nucleation::capi::WorldStream_destroy(reinterpret_cast<nucleation::capi::WorldStream*>(ptr));
}


#endif // NUCLEATION_WorldStream_HPP
