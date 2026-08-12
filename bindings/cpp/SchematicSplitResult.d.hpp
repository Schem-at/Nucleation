#ifndef SchematicSplitResult_D_HPP
#define SchematicSplitResult_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

namespace diplomat::capi { struct Schematic; }
class Schematic;
class NucleationError;




namespace diplomat {
namespace capi {
    struct SchematicSplitResult;
} // namespace capi
} // namespace

/**
 * Deterministic, lossless pieces returned by
 * {@link Schematic::split_connected_attach_nearby}. Pieces are ordered by their
 * largest connected component, largest first.
 */
class SchematicSplitResult {
public:

  inline uint32_t len() const;

  /**
   * Return an independently owned piece by zero-based index.
   */
  inline diplomat::result<std::unique_ptr<Schematic>, NucleationError> piece(uint32_t index) const;

    inline const diplomat::capi::SchematicSplitResult* AsFFI() const;
    inline diplomat::capi::SchematicSplitResult* AsFFI();
    inline static const SchematicSplitResult* FromFFI(const diplomat::capi::SchematicSplitResult* ptr);
    inline static SchematicSplitResult* FromFFI(diplomat::capi::SchematicSplitResult* ptr);
    inline static void operator delete(void* ptr);
private:
    SchematicSplitResult() = delete;
    SchematicSplitResult(const SchematicSplitResult&) = delete;
    SchematicSplitResult(SchematicSplitResult&&) noexcept = delete;
    SchematicSplitResult operator=(const SchematicSplitResult&) = delete;
    SchematicSplitResult operator=(SchematicSplitResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // SchematicSplitResult_D_HPP
