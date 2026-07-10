#ifndef NUCLEATION_WorldSink_HPP
#define NUCLEATION_WorldSink_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct WorldSink_create_result {union {nucleation::capi::WorldSink* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldSink_create_result;
    WorldSink_create_result WorldSink_create(nucleation::diplomat::capi::DiplomatStringView dir, nucleation::diplomat::capi::DiplomatStringView options_json);

    typedef struct WorldSink_open_existing_result {union {nucleation::capi::WorldSink* ok; nucleation::capi::NucleationError err;}; bool is_ok;} WorldSink_open_existing_result;
    WorldSink_open_existing_result WorldSink_open_existing(nucleation::diplomat::capi::DiplomatStringView dir);

    typedef struct WorldSink_write_chunk_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WorldSink_write_chunk_result;
    WorldSink_write_chunk_result WorldSink_write_chunk(nucleation::capi::WorldSink* self, const nucleation::capi::WorldChunkView* view);

    typedef struct WorldSink_put_chunk_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WorldSink_put_chunk_result;
    WorldSink_put_chunk_result WorldSink_put_chunk(nucleation::capi::WorldSink* self, const nucleation::capi::WorldChunkView* view);

    typedef struct WorldSink_finish_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} WorldSink_finish_result;
    WorldSink_finish_result WorldSink_finish(nucleation::capi::WorldSink* self);

    void WorldSink_destroy(WorldSink* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldSink>, nucleation::NucleationError> nucleation::WorldSink::create(std::string_view dir, std::string_view options_json) {
    auto result = nucleation::capi::WorldSink_create({dir.data(), dir.size()},
        {options_json.data(), options_json.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldSink>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldSink>>(std::unique_ptr<nucleation::WorldSink>(nucleation::WorldSink::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldSink>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldSink>, nucleation::NucleationError> nucleation::WorldSink::open_existing(std::string_view dir) {
    auto result = nucleation::capi::WorldSink_open_existing({dir.data(), dir.size()});
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldSink>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldSink>>(std::unique_ptr<nucleation::WorldSink>(nucleation::WorldSink::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldSink>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WorldSink::write_chunk(const nucleation::WorldChunkView& view) {
    auto result = nucleation::capi::WorldSink_write_chunk(this->AsFFI(),
        view.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WorldSink::put_chunk(const nucleation::WorldChunkView& view) {
    auto result = nucleation::capi::WorldSink_put_chunk(this->AsFFI(),
        view.AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::WorldSink::finish() {
    auto result = nucleation::capi::WorldSink_finish(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::WorldSink* nucleation::WorldSink::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::WorldSink*>(this);
}

inline nucleation::capi::WorldSink* nucleation::WorldSink::AsFFI() {
    return reinterpret_cast<nucleation::capi::WorldSink*>(this);
}

inline const nucleation::WorldSink* nucleation::WorldSink::FromFFI(const nucleation::capi::WorldSink* ptr) {
    return reinterpret_cast<const nucleation::WorldSink*>(ptr);
}

inline nucleation::WorldSink* nucleation::WorldSink::FromFFI(nucleation::capi::WorldSink* ptr) {
    return reinterpret_cast<nucleation::WorldSink*>(ptr);
}

inline void nucleation::WorldSink::operator delete(void* ptr) {
    nucleation::capi::WorldSink_destroy(reinterpret_cast<nucleation::capi::WorldSink*>(ptr));
}


#endif // NUCLEATION_WorldSink_HPP
