#ifndef NUCLEATION_GeneratedChunk_HPP
#define NUCLEATION_GeneratedChunk_HPP

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


namespace nucleation {
namespace capi {
    extern "C" {

    typedef struct GeneratedChunk_cx_result {union {int32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_cx_result;
    GeneratedChunk_cx_result GeneratedChunk_cx(const nucleation::capi::GeneratedChunk* self);

    typedef struct GeneratedChunk_cz_result {union {int32_t ok; nucleation::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_cz_result;
    GeneratedChunk_cz_result GeneratedChunk_cz(const nucleation::capi::GeneratedChunk* self);

    typedef struct GeneratedChunk_coverage_result {union {nucleation::capi::GeneratedChunkCoverage ok; nucleation::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_coverage_result;
    GeneratedChunk_coverage_result GeneratedChunk_coverage(const nucleation::capi::GeneratedChunk* self);

    typedef struct GeneratedChunk_source_id_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_source_id_result;
    GeneratedChunk_source_id_result GeneratedChunk_source_id(const nucleation::capi::GeneratedChunk* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct GeneratedChunk_version_result {union { nucleation::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_version_result;
    GeneratedChunk_version_result GeneratedChunk_version(const nucleation::capi::GeneratedChunk* self, nucleation::diplomat::capi::DiplomatWrite* write);

    typedef struct GeneratedChunk_take_view_result {union {nucleation::capi::WorldChunkView* ok; nucleation::capi::NucleationError err;}; bool is_ok;} GeneratedChunk_take_view_result;
    GeneratedChunk_take_view_result GeneratedChunk_take_view(nucleation::capi::GeneratedChunk* self);

    void GeneratedChunk_destroy(GeneratedChunk* self);

    } // extern "C"
} // namespace capi
} // namespace

inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> nucleation::GeneratedChunk::cx() const {
    auto result = nucleation::capi::GeneratedChunk_cx(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<int32_t>(result.ok)) : nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> nucleation::GeneratedChunk::cz() const {
    auto result = nucleation::capi::GeneratedChunk_cz(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Ok<int32_t>(result.ok)) : nucleation::diplomat::result<int32_t, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<nucleation::GeneratedChunkCoverage, nucleation::NucleationError> nucleation::GeneratedChunk::coverage() const {
    auto result = nucleation::capi::GeneratedChunk_coverage(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<nucleation::GeneratedChunkCoverage, nucleation::NucleationError>(nucleation::diplomat::Ok<nucleation::GeneratedChunkCoverage>(nucleation::GeneratedChunkCoverage::FromFFI(result.ok))) : nucleation::diplomat::result<nucleation::GeneratedChunkCoverage, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::GeneratedChunk::source_id() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::GeneratedChunk_source_id(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::GeneratedChunk::source_id_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::GeneratedChunk_source_id(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::string, nucleation::NucleationError> nucleation::GeneratedChunk::version() const {
    std::string output;
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteFromString(output);
    auto result = nucleation::capi::GeneratedChunk_version(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Ok<std::string>(std::move(output))) : nucleation::diplomat::result<std::string, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}
template<typename W>
inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> nucleation::GeneratedChunk::version_write(W& writeable) const {
    nucleation::diplomat::capi::DiplomatWrite write = nucleation::diplomat::WriteTrait<W>::Construct(writeable);
    auto result = nucleation::capi::GeneratedChunk_version(this->AsFFI(),
        &write);
    return result.is_ok ? nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Ok<std::monostate>()) : nucleation::diplomat::result<std::monostate, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldChunkView>, nucleation::NucleationError> nucleation::GeneratedChunk::take_view() {
    auto result = nucleation::capi::GeneratedChunk_take_view(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::WorldChunkView>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::WorldChunkView>>(std::unique_ptr<nucleation::WorldChunkView>(nucleation::WorldChunkView::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::WorldChunkView>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::GeneratedChunk* nucleation::GeneratedChunk::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::GeneratedChunk*>(this);
}

inline nucleation::capi::GeneratedChunk* nucleation::GeneratedChunk::AsFFI() {
    return reinterpret_cast<nucleation::capi::GeneratedChunk*>(this);
}

inline const nucleation::GeneratedChunk* nucleation::GeneratedChunk::FromFFI(const nucleation::capi::GeneratedChunk* ptr) {
    return reinterpret_cast<const nucleation::GeneratedChunk*>(ptr);
}

inline nucleation::GeneratedChunk* nucleation::GeneratedChunk::FromFFI(nucleation::capi::GeneratedChunk* ptr) {
    return reinterpret_cast<nucleation::GeneratedChunk*>(ptr);
}

inline void nucleation::GeneratedChunk::operator delete(void* ptr) {
    nucleation::capi::GeneratedChunk_destroy(reinterpret_cast<nucleation::capi::GeneratedChunk*>(ptr));
}


#endif // NUCLEATION_GeneratedChunk_HPP
