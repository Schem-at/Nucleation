#ifndef NUCLEATION_SchematicSplitResult_D_HPP
#define NUCLEATION_SchematicSplitResult_D_HPP

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
namespace capi { struct Schematic; }
class Schematic;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct SchematicSplitResult;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline nucleation::diplomat::result<std::unique_ptr<nucleation::Schematic>, nucleation::NucleationError> piece(uint32_t index) const;

    inline const nucleation::capi::SchematicSplitResult* AsFFI() const;
    inline nucleation::capi::SchematicSplitResult* AsFFI();
    inline static const nucleation::SchematicSplitResult* FromFFI(const nucleation::capi::SchematicSplitResult* ptr);
    inline static nucleation::SchematicSplitResult* FromFFI(nucleation::capi::SchematicSplitResult* ptr);
    inline static void operator delete(void* ptr);
private:
    SchematicSplitResult() = delete;
    SchematicSplitResult(const nucleation::SchematicSplitResult&) = delete;
    SchematicSplitResult(nucleation::SchematicSplitResult&&) noexcept = delete;
    SchematicSplitResult operator=(const nucleation::SchematicSplitResult&) = delete;
    SchematicSplitResult operator=(nucleation::SchematicSplitResult&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_SchematicSplitResult_D_HPP
