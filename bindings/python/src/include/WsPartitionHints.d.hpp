#ifndef NUCLEATION_WsPartitionHints_D_HPP
#define NUCLEATION_WsPartitionHints_D_HPP

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
namespace capi { struct WsPartitionHints; }
class WsPartitionHints;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct WsPartitionHints;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * Caller-supplied partition hints (full-column boxes a cluster may never
 * span under {@link PartitionPolicy::HardCut}). Order does not matter:
 * {@link PartitionIndex::new}(crate::world_segment::partition::PartitionIndex)
 * sorts hints by full content at construction time.
 */
class WsPartitionHints {
public:

  inline static std::unique_ptr<nucleation::WsPartitionHints> create();

  /**
   * Add a full-column hint (`y_range: None`) covering inclusive
   * `x0..=x1, z0..=z1`.
   */
  inline nucleation::diplomat::result<std::monostate, nucleation::NucleationError> add(std::string_view id, int32_t x0, int32_t x1, int32_t z0, int32_t z1);

  inline uint32_t len() const;

    inline const nucleation::capi::WsPartitionHints* AsFFI() const;
    inline nucleation::capi::WsPartitionHints* AsFFI();
    inline static const nucleation::WsPartitionHints* FromFFI(const nucleation::capi::WsPartitionHints* ptr);
    inline static nucleation::WsPartitionHints* FromFFI(nucleation::capi::WsPartitionHints* ptr);
    inline static void operator delete(void* ptr);
private:
    WsPartitionHints() = delete;
    WsPartitionHints(const nucleation::WsPartitionHints&) = delete;
    WsPartitionHints(nucleation::WsPartitionHints&&) noexcept = delete;
    WsPartitionHints operator=(const nucleation::WsPartitionHints&) = delete;
    WsPartitionHints operator=(nucleation::WsPartitionHints&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_WsPartitionHints_D_HPP
