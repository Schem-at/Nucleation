#ifndef NUCLEATION_GeneratedWorldStream_HPP
#define NUCLEATION_GeneratedWorldStream_HPP

#include "GeneratedWorldStream.d.hpp"

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "GeneratedChunk.hpp"
#include "NucleationError.hpp"
#include "diplomat_runtime.hpp"


namespace nucleation {
namespace capi {
    extern "C" {

    uint64_t GeneratedWorldStream_remaining(const nucleation::capi::GeneratedWorldStream* self);

    typedef struct GeneratedWorldStream_next_result {union {nucleation::capi::GeneratedChunk* ok; nucleation::capi::NucleationError err;}; bool is_ok;} GeneratedWorldStream_next_result;
    GeneratedWorldStream_next_result GeneratedWorldStream_next(nucleation::capi::GeneratedWorldStream* self);

    void GeneratedWorldStream_destroy(GeneratedWorldStream* self);

    } // extern "C"
} // namespace capi
} // namespace

inline uint64_t nucleation::GeneratedWorldStream::remaining() const {
    auto result = nucleation::capi::GeneratedWorldStream_remaining(this->AsFFI());
    return result;
}

inline nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedChunk>, nucleation::NucleationError> nucleation::GeneratedWorldStream::next() {
    auto result = nucleation::capi::GeneratedWorldStream_next(this->AsFFI());
    return result.is_ok ? nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedChunk>, nucleation::NucleationError>(nucleation::diplomat::Ok<std::unique_ptr<nucleation::GeneratedChunk>>(std::unique_ptr<nucleation::GeneratedChunk>(nucleation::GeneratedChunk::FromFFI(result.ok)))) : nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedChunk>, nucleation::NucleationError>(nucleation::diplomat::Err<nucleation::NucleationError>(nucleation::NucleationError::FromFFI(result.err)));
}

inline const nucleation::capi::GeneratedWorldStream* nucleation::GeneratedWorldStream::AsFFI() const {
    return reinterpret_cast<const nucleation::capi::GeneratedWorldStream*>(this);
}

inline nucleation::capi::GeneratedWorldStream* nucleation::GeneratedWorldStream::AsFFI() {
    return reinterpret_cast<nucleation::capi::GeneratedWorldStream*>(this);
}

inline const nucleation::GeneratedWorldStream* nucleation::GeneratedWorldStream::FromFFI(const nucleation::capi::GeneratedWorldStream* ptr) {
    return reinterpret_cast<const nucleation::GeneratedWorldStream*>(ptr);
}

inline nucleation::GeneratedWorldStream* nucleation::GeneratedWorldStream::FromFFI(nucleation::capi::GeneratedWorldStream* ptr) {
    return reinterpret_cast<nucleation::GeneratedWorldStream*>(ptr);
}

inline void nucleation::GeneratedWorldStream::operator delete(void* ptr) {
    nucleation::capi::GeneratedWorldStream_destroy(reinterpret_cast<nucleation::capi::GeneratedWorldStream*>(ptr));
}


#endif // NUCLEATION_GeneratedWorldStream_HPP
