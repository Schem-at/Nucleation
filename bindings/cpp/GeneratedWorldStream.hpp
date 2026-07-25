#ifndef GeneratedWorldStream_HPP
#define GeneratedWorldStream_HPP

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


namespace diplomat {
namespace capi {
    extern "C" {

    uint64_t GeneratedWorldStream_remaining(const diplomat::capi::GeneratedWorldStream* self);

    typedef struct GeneratedWorldStream_next_result {union {diplomat::capi::GeneratedChunk* ok; diplomat::capi::NucleationError err;}; bool is_ok;} GeneratedWorldStream_next_result;
    GeneratedWorldStream_next_result GeneratedWorldStream_next(diplomat::capi::GeneratedWorldStream* self);

    void GeneratedWorldStream_destroy(GeneratedWorldStream* self);

    } // extern "C"
} // namespace capi
} // namespace

inline uint64_t GeneratedWorldStream::remaining() const {
    auto result = diplomat::capi::GeneratedWorldStream_remaining(this->AsFFI());
    return result;
}

inline diplomat::result<std::unique_ptr<GeneratedChunk>, NucleationError> GeneratedWorldStream::next() {
    auto result = diplomat::capi::GeneratedWorldStream_next(this->AsFFI());
    return result.is_ok ? diplomat::result<std::unique_ptr<GeneratedChunk>, NucleationError>(diplomat::Ok<std::unique_ptr<GeneratedChunk>>(std::unique_ptr<GeneratedChunk>(GeneratedChunk::FromFFI(result.ok)))) : diplomat::result<std::unique_ptr<GeneratedChunk>, NucleationError>(diplomat::Err<NucleationError>(NucleationError::FromFFI(result.err)));
}

inline const diplomat::capi::GeneratedWorldStream* GeneratedWorldStream::AsFFI() const {
    return reinterpret_cast<const diplomat::capi::GeneratedWorldStream*>(this);
}

inline diplomat::capi::GeneratedWorldStream* GeneratedWorldStream::AsFFI() {
    return reinterpret_cast<diplomat::capi::GeneratedWorldStream*>(this);
}

inline const GeneratedWorldStream* GeneratedWorldStream::FromFFI(const diplomat::capi::GeneratedWorldStream* ptr) {
    return reinterpret_cast<const GeneratedWorldStream*>(ptr);
}

inline GeneratedWorldStream* GeneratedWorldStream::FromFFI(diplomat::capi::GeneratedWorldStream* ptr) {
    return reinterpret_cast<GeneratedWorldStream*>(ptr);
}

inline void GeneratedWorldStream::operator delete(void* ptr) {
    diplomat::capi::GeneratedWorldStream_destroy(reinterpret_cast<diplomat::capi::GeneratedWorldStream*>(ptr));
}


#endif // GeneratedWorldStream_HPP
