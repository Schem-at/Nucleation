#ifndef GeneratedWorldStream_D_HPP
#define GeneratedWorldStream_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct GeneratedChunk; }
class GeneratedChunk;
class NucleationError;




namespace diplomat {
namespace capi {
    struct GeneratedWorldStream;
} // namespace capi
} // namespace

/**
 * A finite, lazy, canonical region-major traversal of a generator.
 */
class GeneratedWorldStream {
public:

  /**
   * Number of chunks not yet requested from the source.
   */
  inline uint64_t remaining() const;

  /**
   * Generate and return the next chunk. Returns `NotFound` at end-of-stream,
   * and `Generation` if the underlying source failed on a valid request.
   */
  inline diplomat::result<std::unique_ptr<GeneratedChunk>, NucleationError> next();

    inline const diplomat::capi::GeneratedWorldStream* AsFFI() const;
    inline diplomat::capi::GeneratedWorldStream* AsFFI();
    inline static const GeneratedWorldStream* FromFFI(const diplomat::capi::GeneratedWorldStream* ptr);
    inline static GeneratedWorldStream* FromFFI(diplomat::capi::GeneratedWorldStream* ptr);
    inline static void operator delete(void* ptr);
private:
    GeneratedWorldStream() = delete;
    GeneratedWorldStream(const GeneratedWorldStream&) = delete;
    GeneratedWorldStream(GeneratedWorldStream&&) noexcept = delete;
    GeneratedWorldStream operator=(const GeneratedWorldStream&) = delete;
    GeneratedWorldStream operator=(GeneratedWorldStream&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // GeneratedWorldStream_D_HPP
