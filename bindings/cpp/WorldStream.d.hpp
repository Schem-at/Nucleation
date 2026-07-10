#ifndef WorldStream_D_HPP
#define WorldStream_D_HPP

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
class NucleationError;




namespace diplomat {
namespace capi {
    struct WorldStream;
} // namespace capi
} // namespace

/**
 * A streaming iterator over the chunks of a world.
 */
class WorldStream {
public:

  /**
   * Open a streaming iterator over a world directory.
   */
  inline static diplomat::result<std::unique_ptr<WorldStream>, NucleationError> open_dir(std::string_view path);

  /**
   * Open a streaming iterator over a world directory, bounded to the given
   * block-coordinate box `[min_x..max_x, min_y..max_y, min_z..max_z]`.
   */
  inline static diplomat::result<std::unique_ptr<WorldStream>, NucleationError> open_dir_bounded(std::string_view path, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Open a streaming iterator from a zip archive in memory.
   */
  inline static diplomat::result<std::unique_ptr<WorldStream>, NucleationError> from_zip(diplomat::span<const uint8_t> data);

  /**
   * Open a bounded streaming iterator from a zip archive in memory.
   */
  inline static diplomat::result<std::unique_ptr<WorldStream>, NucleationError> from_zip_bounded(diplomat::span<const uint8_t> data, int32_t min_x, int32_t min_y, int32_t min_z, int32_t max_x, int32_t max_y, int32_t max_z);

  /**
   * Advance the iterator and return the next chunk view. Errors with
   * `NotFound` at end-of-stream (the old ABI returned NULL). Corrupt
   * chunks are silently skipped, matching the old ABI.
   */
  inline diplomat::result<std::unique_ptr<WorldChunkView>, NucleationError> next();

    inline const diplomat::capi::WorldStream* AsFFI() const;
    inline diplomat::capi::WorldStream* AsFFI();
    inline static const WorldStream* FromFFI(const diplomat::capi::WorldStream* ptr);
    inline static WorldStream* FromFFI(diplomat::capi::WorldStream* ptr);
    inline static void operator delete(void* ptr);
private:
    WorldStream() = delete;
    WorldStream(const WorldStream&) = delete;
    WorldStream(WorldStream&&) noexcept = delete;
    WorldStream operator=(const WorldStream&) = delete;
    WorldStream operator=(WorldStream&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // WorldStream_D_HPP
