#ifndef NUCLEATION_WorldStream_D_HPP
#define NUCLEATION_WorldStream_D_HPP

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
namespace capi { struct WorldStream; }
class WorldStream;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct WorldStream;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * A streaming iterator over the chunks of a world.
 */
class WorldStream {
public:

  /**
   * Open a streaming iterator over a world directory.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError> open_dir(std::string_view path);

  /**
   * Open a streaming iterator over a world directory, bounded to the given
   * block-coordinate box `[min_x..max_x, min_y..max_y, min_z..max_z]`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError> open_dir_bounded(std::string_view path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Open a streaming iterator from a zip archive in memory.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError> from_zip(nucleation::diplomat::span<const uint8_t> data);

  /**
   * Open a bounded streaming iterator from a zip archive in memory.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldStream>, nucleation::NucleationError> from_zip_bounded(nucleation::diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Advance the iterator and return the next chunk view. Errors with
   * `NotFound` at end-of-stream (the old ABI returned NULL). Corrupt
   * chunks are silently skipped, matching the old ABI.
   */
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::WorldChunkView>, nucleation::NucleationError> next();

    inline const nucleation::capi::WorldStream* AsFFI() const;
    inline nucleation::capi::WorldStream* AsFFI();
    inline static const nucleation::WorldStream* FromFFI(const nucleation::capi::WorldStream* ptr);
    inline static nucleation::WorldStream* FromFFI(nucleation::capi::WorldStream* ptr);
    inline static void operator delete(void* ptr);
private:
    WorldStream() = delete;
    WorldStream(const nucleation::WorldStream&) = delete;
    WorldStream(nucleation::WorldStream&&) noexcept = delete;
    WorldStream operator=(const nucleation::WorldStream&) = delete;
    WorldStream operator=(nucleation::WorldStream&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_WorldStream_D_HPP
