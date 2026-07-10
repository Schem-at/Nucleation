#ifndef NUCLEATION_WorldSink_D_HPP
#define NUCLEATION_WorldSink_D_HPP

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
namespace capi { struct WorldSink; }
class WorldSink;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct WorldSink;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldSink>, nucleation::NucleationError> create(std::string_view dir, std::string_view options_json);

  /**
   * Open an existing world directory for patching via `put_chunk`.
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::WorldSink>, nucleation::NucleationError> open_existing(std::string_view dir);

  /**
   * Write (append) a chunk view into the sink. The view is not consumed.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> write_chunk(const nucleation::WorldChunkView& view);

  /**
   * Overwrite the chunk at (`view.cx`, `view.cz`) of the sink's world with
   * the supplied view's block data. Only valid on sinks opened with
   * `open_existing`; errors with `Io` on a create-mode sink. The view is
   * not consumed.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> put_chunk(const nucleation::WorldChunkView& view);

  /**
   * Finalise and flush all pending writes. Consuming (PORTING rule 11):
   * afterwards every method on this sink returns `AlreadyConsumed`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> finish();

    inline const nucleation::capi::WorldSink* AsFFI() const;
    inline nucleation::capi::WorldSink* AsFFI();
    inline static const nucleation::WorldSink* FromFFI(const nucleation::capi::WorldSink* ptr);
    inline static nucleation::WorldSink* FromFFI(nucleation::capi::WorldSink* ptr);
    inline static void operator delete(void* ptr);
private:
    WorldSink() = delete;
    WorldSink(const nucleation::WorldSink&) = delete;
    WorldSink(nucleation::WorldSink&&) noexcept = delete;
    WorldSink operator=(const nucleation::WorldSink&) = delete;
    WorldSink operator=(nucleation::WorldSink&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_WorldSink_D_HPP
