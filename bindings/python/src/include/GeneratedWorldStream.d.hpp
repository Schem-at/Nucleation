#ifndef NUCLEATION_GeneratedWorldStream_D_HPP
#define NUCLEATION_GeneratedWorldStream_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"
namespace nucleation {
namespace capi { struct GeneratedChunk; }
class GeneratedChunk;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct GeneratedWorldStream;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::GeneratedChunk>, nucleation::NucleationError> next();

    inline const nucleation::capi::GeneratedWorldStream* AsFFI() const;
    inline nucleation::capi::GeneratedWorldStream* AsFFI();
    inline static const nucleation::GeneratedWorldStream* FromFFI(const nucleation::capi::GeneratedWorldStream* ptr);
    inline static nucleation::GeneratedWorldStream* FromFFI(nucleation::capi::GeneratedWorldStream* ptr);
    inline static void operator delete(void* ptr);
private:
    GeneratedWorldStream() = delete;
    GeneratedWorldStream(const nucleation::GeneratedWorldStream&) = delete;
    GeneratedWorldStream(nucleation::GeneratedWorldStream&&) noexcept = delete;
    GeneratedWorldStream operator=(const nucleation::GeneratedWorldStream&) = delete;
    GeneratedWorldStream operator=(nucleation::GeneratedWorldStream&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_GeneratedWorldStream_D_HPP
