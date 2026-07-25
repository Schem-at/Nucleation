#ifndef GeneratedChunk_HPP
#define GeneratedChunk_HPP

#include "GeneratedChunk.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "GeneratedChunkCoverage.hpp"
#include "NucleationError.hpp"
#include "WorldChunkView.hpp"
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    extern "C" {

    typedef struct GeneratedChunk_cx_result {union {int32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_cx_result;
    GeneratedChunk_cx_result GeneratedChunk_cx(const diplomat::capi::GeneratedChunk* self);

    typedef struct GeneratedChunk_cz_result {union {int32_t ok; diplomat::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_cz_result;
    GeneratedChunk_cz_result GeneratedChunk_cz(const diplomat::capi::GeneratedChunk* self);

    typedef struct GeneratedChunk_coverage_result {union {diplomat::capi::GeneratedChunkCoverage ok; diplomat::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_coverage_result;
    GeneratedChunk_coverage_result GeneratedChunk_coverage(const diplomat::capi::GeneratedChunk* self);

    typedef struct GeneratedChunk_source_id_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_source_id_result;
    GeneratedChunk_source_id_result GeneratedChunk_source_id(const diplomat::capi::GeneratedChunk* self, diplomat::capi::DiplomatWrite* write);

    typedef struct GeneratedChunk_version_result {union { diplomat::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_version_result;
    GeneratedChunk_version_result GeneratedChunk_version(const diplomat::capi::GeneratedChunk* self, diplomat::capi::DiplomatWrite* write);

    typedef struct GeneratedChunk_take_view_result {union {diplomat::capi::WorldChunkView* ok; diplomat::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_take_view_result;
    GeneratedChunk_take_view_result GeneratedChunk_take_view(diplomat::capi::GeneratedChunk* self);

    void GeneratedChunk_destroy(GeneratedChunk* self);

    } // extern "C"
} // namespace capi
} // namespace

inline diplomat::result<int32_t, NucleationError> GeneratedChunk::cx() const {
    auto result = diplomat::capi::GeneratedChunk_cx(this->AsFFI());
    return result.is_ok ? diplomat::result<int32_t, NucleationError>(diplomat::Ok<int32_t>(result.ok)) : diplomat::result<int32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<int32_t, NucleationError> GeneratedChunk::cz() const {
    auto result = diplomat::capi::GeneratedChunk_cz(this->AsFFI());
    return result.is_ok ? diplomat::result<int32_t, NucleationError>(diplomat::Ok<int32_t>(result.ok)) : diplomat::result<int32_t, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<GeneratedChunkCoverage, NucleationError> GeneratedChunk::coverage() const {
    auto result = diplomat::capi::GeneratedChunk_coverage(this->AsFFI());
    return result.is_ok ? diplomat::result<GeneratedChunkCoverage, NucleationError>(diplomat::Ok<GeneratedChunkCoverage>(GeneratedChunkCoverage::FromFFI(result.ok))) : diplomat::result<GeneratedChunkCoverage, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> GeneratedChunk::source_id() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::GeneratedChunk_source_id(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> GeneratedChunk::source_id_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::GeneratedChunk_source_id(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::string, NucleationError> GeneratedChunk::version() const {
    std::string output;
    diplomat::capi::DiplomatWrite write = diplomat::WriteFromString(output);
    auto result = diplomat::capi::GeneratedChunk_version(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::string, NucleationError>(diplomat::Ok<std::string>(std::move(output))) : diplomat::result<std::string, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}
template<typename W>
inline diplomat::result<std::monostate, NucleationError> GeneratedChunk::version_write(W& writeable) const {
    diplomat::capi::DiplomatWrite write = diplomat::WriteTrait<W>::Construct(writeable);
    auto result = diplomat::capi::GeneratedChunk_version(this->AsFFI(),
        &write);
    return result.is_ok ? diplomat::result<std::monostate, NucleationError>(diplomat::Ok<std::monostate>()) : diplomat::result<std::monostate, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline diplomat::result<std::unique_ptr<WorldChunkView>, NucleationError> GeneratedChunk::take_view() {
    auto result = diplomat::capi::GeneratedChunk_take_view(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<WorldChunkView>, NucleationError>(diplomat::Ok<std::unique_ptr<WorldChunkView>>(std::unique_ptr<WorldChunkView>(WorldChunkView::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<WorldChunkView>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::GeneratedChunk* GeneratedChunk::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::GeneratedChunk*>(this);
}

inline diplomat::capi::GeneratedChunk* GeneratedChunk::AsFFI() {
    return reinterpret_cast<diplomat::capi::GeneratedChunk*>(this);
}

inline const GeneratedChunk* GeneratedChunk::FromFFI(const diplomat::capi::GeneratedChunk* ptr) {
    return reinterpret_cast<const GeneratedChunk*>(ptr);
}

inline GeneratedChunk* GeneratedChunk::FromFFI(diplomat::capi::GeneratedChunk* ptr) {
    return reinterpret_cast<GeneratedChunk*>(ptr);
}

inline void GeneratedChunk::operator delete(void* ptr) {
    diplomat::capi::GeneratedChunk_destroy(reinterpret_cast<diplomat::capi::GeneratedChunk*>(ptr));
}


#endif // GeneratedChunk_HPP
