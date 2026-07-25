#ifndef GeneratedChunk_D_HPP
#define GeneratedChunk_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct WorldChunkView; }
class WorldChunkView;
class GeneratedChunkCoverage;
class NucleationError;




namespace diplomat {
namespace capi {
    struct GeneratedChunk;
} // namespace capi
} // namespace

/**
 * One generated chunk plus coverage and source-version metadata.
 * Call `take_view` once to move its chunk into the existing world-stream API.
 */
class GeneratedChunk {
public:

  inline diplomat::result<int32_t, NucleationError> cx() const;

  inline diplomat::result<int32_t, NucleationError> cz() const;

  inline diplomat::result<GeneratedChunkCoverage, NucleationError> coverage() const;

  inline diplomat::result<std::string, NucleationError> source_id() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> source_id_write(W& writeable_output) const;

  inline diplomat::result<std::string, NucleationError> version() const;
  template<typename W>
  inline diplomat::result<std::monostate, NucleationError> version_write(W& writeable_output) const;

  /**
   * Consume the generated chunk payload. Metadata access and a second call
   * return `AlreadyConsumed` afterwards.
   */
  inline diplomat::result<std::unique_ptr<WorldChunkView>, NucleationError> take_view();

    inline const diplomat::capi::GeneratedChunk* AsFFI() const;
    inline diplomat::capi::GeneratedChunk* AsFFI();
    inline static const GeneratedChunk* FromFFI(const diplomat::capi::GeneratedChunk* ptr);
    inline static GeneratedChunk* FromFFI(diplomat::capi::GeneratedChunk* ptr);
    inline static void operator delete(void* ptr);
private:
    GeneratedChunk() = delete;
    GeneratedChunk(const GeneratedChunk&) = delete;
    GeneratedChunk(GeneratedChunk&&) noexcept = delete;
    GeneratedChunk operator=(const GeneratedChunk&) = delete;
    GeneratedChunk operator=(GeneratedChunk&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // GeneratedChunk_D_HPP
