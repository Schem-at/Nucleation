#ifndef WorldSink_D_HPP
#define WorldSink_D_HPP

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
    struct WorldSink;
} // namespace capi
} // namespace

/**
 * A world writer. `finish` is consuming (PORTING rule 11): the inner sink is
 * held in an `Option` and taken on `finish`; every method afterwards returns
 * `AlreadyConsumed`. Dropping the handle without `finish` abandons the sink.
 */
class WorldSink {
public:

  /**
   * Create a new world sink that writes fresh chunk data to `dir`.
   * `options_json` is a serialized `WorldExportOptions` (empty string =
   * defaults).
   */
  inline static diplomat::result<std::unique_ptr<WorldSink>, NucleationError> create(std::string_view dir, std::string_view options_json);

  /**
   * Open an existing world directory for patching via `put_chunk`.
   */
  inline static diplomat::result<std::unique_ptr<WorldSink>, NucleationError> open_existing(std::string_view dir);

  /**
   * Write (append) a chunk view into the sink. The view is not consumed.
   */
  inline diplomat::result<std::monostate, NucleationError> write_chunk(const WorldChunkView& view);

  /**
   * Overwrite the chunk at (`view.cx`, `view.cz`) of the sink's world with
   * the supplied view's block data. Only valid on sinks opened with
   * `open_existing`; errors with `Io` on a create-mode sink. The view is
   * not consumed.
   */
  inline diplomat::result<std::monostate, NucleationError> put_chunk(const WorldChunkView& view);

  /**
   * Finalise and flush all pending writes. Consuming (PORTING rule 11):
   * afterwards every method on this sink returns `AlreadyConsumed`.
   */
  inline diplomat::result<std::monostate, NucleationError> finish();

    inline const diplomat::capi::WorldSink* AsFFI() const;
    inline diplomat::capi::WorldSink* AsFFI();
    inline static const WorldSink* FromFFI(const diplomat::capi::WorldSink* ptr);
    inline static WorldSink* FromFFI(diplomat::capi::WorldSink* ptr);
    inline static void operator delete(void* ptr);
private:
    WorldSink() = delete;
    WorldSink(const WorldSink&) = delete;
    WorldSink(WorldSink&&) noexcept = delete;
    WorldSink operator=(const WorldSink&) = delete;
    WorldSink operator=(WorldSink&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // WorldSink_D_HPP
