#ifndef NUCLEATION_GeneratedChunk_D_HPP
#define NUCLEATION_GeneratedChunk_D_HPP

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
namespace capi { struct WorldChunkView; }
class WorldChunkView;
class GeneratedChunkCoverage;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct GeneratedChunk;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * One generated chunk plus coverage and source-version metadata.
 * Call `take_view` once to move its chunk into the existing world-stream API.
 */
class GeneratedChunk {
public:

  inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> cx() const;

  inline nucleation::diplomat::result<int32_t, nucleation::NucleationError> cz() const;

  inline nucleation::diplomat::result<nucleation::GeneratedChunkCoverage, nucleation::NucleationError> coverage() const;

  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> source_id() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> source_id_write(W& writeable_output) const;

  inline nucleation::diplomat::result<std::string, nucleation::NucleationError> version() const;
  template<typename W>
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> version_write(W& writeable_output) const;

  /**
   * Consume the generated chunk payload. Metadata access and a second call
   * return `AlreadyConsumed` afterwards.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldChunkView>, nucleation::NucleationError> take_view();

    inline const nucleation::capi::GeneratedChunk* AsFFI() const;
    inline nucleation::capi::GeneratedChunk* AsFFI();
    inline static const nucleation::GeneratedChunk* FromFFI(const nucleation::capi::GeneratedChunk* ptr);
    inline static nucleation::GeneratedChunk* FromFFI(nucleation::capi::GeneratedChunk* ptr);
    inline static void operator delete(void* ptr);
private:
    GeneratedChunk() = delete;
    GeneratedChunk(const nucleation::GeneratedChunk&) = delete;
    GeneratedChunk(nucleation::GeneratedChunk&&) noexcept = delete;
    GeneratedChunk operator=(const nucleation::GeneratedChunk&) = delete;
    GeneratedChunk operator=(nucleation::GeneratedChunk&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_GeneratedChunk_D_HPP
