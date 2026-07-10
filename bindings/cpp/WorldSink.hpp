#ifndef WorldSink_HPP
#define WorldSink_HPP

#include "WorldSink.d.hpp"

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

    typedef struct WorldSink_create_result {union {diplomat::capi::WorldSink* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldSink_create_result;
    WorldSink_create_result WorldSink_create(diplomat::capi::DiplomatStringView dir, diplomat::capi::DiplomatStringView options_json);

    typedef struct WorldSink_open_existing_result {union {diplomat::capi::WorldSink* ok; diplomat::capi::NucleationError err;}; bool is_ok;} WorldSink_open_existing_result;
    WorldSink_open_existing_result WorldSink_open_existing(diplomat::capi::DiplomatStringView dir);

    typedef struct WorldSink_write_chunk_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WorldSink_write_chunk_result;
    WorldSink_write_chunk_result WorldSink_write_chunk(diplomat::capi::WorldSink* self, const diplomat::capi::WorldChunkView* view);

    typedef struct WorldSink_put_chunk_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WorldSink_put_chunk_result;
    WorldSink_put_chunk_result WorldSink_put_chunk(diplomat::capi::WorldSink* self, const diplomat::capi::WorldChunkView* view);

    typedef struct WorldSink_finish_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} WorldSink_finish_result;
    WorldSink_finish_result WorldSink_finish(diplomat::capi::WorldSink* self);

    void WorldSink_destroy(WorldSink* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<std::unique_ptr<WorldSink>, NucleationError> WorldSink::create(std::string_view dir, std::string_view options_json) {
    auto result = diplomat::capi::WorldSink_create({dir.data(), dir.size()},
        {options_json.data(), options_json.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldSink>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldSink>>(std::unique_ptr<WorldSink>(WorldSink::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldSink>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<WorldSink>, NucleationError> WorldSink::open_existing(std::string_view dir) {
    auto result = diplomat::capi::WorldSink_open_existing({dir.data(), dir.size()});
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldSink>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldSink>>(std::unique_ptr<WorldSink>(WorldSink::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldSink>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> WorldSink::write_chunk(const WorldChunkView& view) {
    auto result = diplomat::capi::WorldSink_write_chunk(this->AsFFI(),
        view.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> WorldSink::put_chunk(const WorldChunkView& view) {
    auto result = diplomat::capi::WorldSink_put_chunk(this->AsFFI(),
        view.AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::monostate, NucleationError> WorldSink::finish() {
    auto result = diplomat::capi::WorldSink_finish(this->AsFFI());
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::WorldSink* WorldSink::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::WorldSink*>(this);
}

inline diplomat::capi::WorldSink* WorldSink::AsFFI() {
    return reinterpret_cast<diplomat::capi::WorldSink*>(this);
}

inline const WorldSink* WorldSink::FromFFI(const diplomat::capi::WorldSink* ptr) {
    return reinterpret_cast<const WorldSink*>(ptr);
}

inline WorldSink* WorldSink::FromFFI(diplomat::capi::WorldSink* ptr) {
    return reinterpret_cast<WorldSink*>(ptr);
}

inline void WorldSink::operator delete(void* ptr) {
    diplomat::capi::WorldSink_destroy(reinterpret_cast<diplomat::capi::WorldSink*>(ptr));
}


#endif // WorldSink_HPP
