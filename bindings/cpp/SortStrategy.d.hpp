#ifndef SortStrategy_D_HPP
#define SortStrategy_D_HPP

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
    struct SortStrategy;
} // namespace capi
} // namespace

/**
 * Ordering applied to region positions before bit assignment
 * (PORTING rule 10).
 */
class SortStrategy {
public:

  /**
   * Sort by Y ascending, then X, then Z (default Minecraft layer
   * order). The first position after sorting is bit 0 (LSB).
   */
  inline static std::unique_ptr<SortStrategy> yxz();

  /**
   * Sort by X ascending, then Y, then Z.
   */
  inline static std::unique_ptr<SortStrategy> xyz();

  /**
   * Sort by Z ascending, then Y, then X.
   */
  inline static std::unique_ptr<SortStrategy> zyx();

  /**
   * Sort by Y descending, then X ascending, then Z ascending.
   */
  inline static std::unique_ptr<SortStrategy> y_desc_xz();

  /**
   * Sort by X descending, then Y ascending, then Z ascending.
   */
  inline static std::unique_ptr<SortStrategy> x_desc_yz();

  /**
   * Sort by Z descending, then Y ascending, then X ascending.
   */
  inline static std::unique_ptr<SortStrategy> z_desc_yx();

  /**
   * Old ABI name: `sort_strategy_descending`.
   */
  inline static std::unique_ptr<SortStrategy> descending();

  /**
   * Sort by Euclidean distance from the reference point, closest
   * first (ties broken by Y, X, Z ascending).
   */
  inline static std::unique_ptr<SortStrategy> distance_from(int32_t x, int32_t y, int32_t z);

  /**
   * Sort by Euclidean distance from the reference point, farthest
   * first (ties broken by Y, X, Z descending).
   */
  inline static std::unique_ptr<SortStrategy> distance_from_desc(int32_t x, int32_t y, int32_t z);

  /**
   * Keep positions in the order they were added (no sorting). Useful
   * when positions were ordered manually or box order matters.
   */
  inline static std::unique_ptr<SortStrategy> preserve();

  /**
   * Reverse of the order positions were added.
   */
  inline static std::unique_ptr<SortStrategy> reverse();

  /**
   * Parse from a name (e.g. "yxz", "descending", "preserve").
   */
  inline static diplomat::result<std::unique_ptr<SortStrategy>, NucleationError> from_string(std::string_view s);

  /**
   * The strategy name.
   */
  inline std::string name() const;
  template<typename W>
  inline void name_write(W& writeable_output) const;

    inline const diplomat::capi::SortStrategy* AsFFI() const;
    inline diplomat::capi::SortStrategy* AsFFI();
    inline static const SortStrategy* FromFFI(const diplomat::capi::SortStrategy* ptr);
    inline static SortStrategy* FromFFI(diplomat::capi::SortStrategy* ptr);
    inline static void operator delete(void* ptr);
private:
    SortStrategy() = delete;
    SortStrategy(const SortStrategy&) = delete;
    SortStrategy(SortStrategy&&) noexcept = delete;
    SortStrategy operator=(const SortStrategy&) = delete;
    SortStrategy operator=(SortStrategy&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // SortStrategy_D_HPP
