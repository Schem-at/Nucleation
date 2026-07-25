#ifndef WsPartitionHints_D_HPP
#define WsPartitionHints_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"

class NucleationError;




namespace diplomat {
namespace capi {
    struct WsPartitionHints;
} // namespace capi
} // namespace

/**
 * Caller-supplied partition hints (full-column boxes a cluster may never
 * span under {@link PartitionPolicy::HardCut}). Order does not matter:
 * {@link PartitionIndex::new}(crate::world_segment::partition::PartitionIndex)
 * sorts hints by full content at construction time.
 */
class WsPartitionHints {
public:

  inline static std::unique_ptr<WsPartitionHints> create();

  /**
   * Add a full-column hint (`y_range: None`) covering inclusive
   * `x0..=x1, z0..=z1`.
   */
  inline diplomat::result<std::monostate, NucleationError> add(std::string_view id, int32_t x0, int32_t x1, int32_t z0, int32_t z1);

  inline uint32_t len() const;

    inline const diplomat::capi::WsPartitionHints* AsFFI() const;
    inline diplomat::capi::WsPartitionHints* AsFFI();
    inline static const WsPartitionHints* FromFFI(const diplomat::capi::WsPartitionHints* ptr);
    inline static WsPartitionHints* FromFFI(diplomat::capi::WsPartitionHints* ptr);
    inline static void operator delete(void* ptr);
private:
    WsPartitionHints() = delete;
    WsPartitionHints(const WsPartitionHints&) = delete;
    WsPartitionHints(WsPartitionHints&&) noexcept = delete;
    WsPartitionHints operator=(const WsPartitionHints&) = delete;
    WsPartitionHints operator=(WsPartitionHints&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // WsPartitionHints_D_HPP
