#ifndef NUCLEATION_SortStrategy_D_HPP
#define NUCLEATION_SortStrategy_D_HPP

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
namespace capi { struct SortStrategy; }
class SortStrategy;
class NucleationError;
} // namespace nucleation



namespace nucleation {
namespace capi {
    struct SortStrategy;
} // namespace capi
} // namespace

namespace nucleation {
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
  inline static std::unique_ptr<nucleation::SortStrategy> yxz();

  /**
   * Sort by X ascending, then Y, then Z.
   */
  inline static std::unique_ptr<nucleation::SortStrategy> xyz();

  /**
   * Sort by Z ascending, then Y, then X.
   */
  inline static std::unique_ptr<nucleation::SortStrategy> zyx();

  /**
   * Sort by Y descending, then X ascending, then Z ascending.
   */
  inline static std::unique_ptr<nucleation::SortStrategy> y_desc_xz();

  /**
   * Sort by X descending, then Y ascending, then Z ascending.
   */
  inline static std::unique_ptr<nucleation::SortStrategy> x_desc_yz();

  /**
   * Sort by Z descending, then Y ascending, then X ascending.
   */
  inline static std::unique_ptr<nucleation::SortStrategy> z_desc_yx();

  /**
   * Old ABI name: `sort_strategy_descending`.
   */
  inline static std::unique_ptr<nucleation::SortStrategy> descending();

  /**
   * Sort by Euclidean distance from the reference point, closest
   * first (ties broken by Y, X, Z ascending).
   */
  inline static std::unique_ptr<nucleation::SortStrategy> distance_from(int32_t x, int32_t y, int32_t z);

  /**
   * Sort by Euclidean distance from the reference point, farthest
   * first (ties broken by Y, X, Z descending).
   */
  inline static std::unique_ptr<nucleation::SortStrategy> distance_from_desc(int32_t x, int32_t y, int32_t z);

  /**
   * Keep positions in the order they were added (no sorting). Useful
   * when positions were ordered manually or box order matters.
   */
  inline static std::unique_ptr<nucleation::SortStrategy> preserve();

  /**
   * Reverse of the order positions were added.
   */
  inline static std::unique_ptr<nucleation::SortStrategy> reverse();

  /**
   * Parse from a name (e.g. "yxz", "descending", "preserve").
   */
  inline static nucleation::diplomat::result<std::unique_ptr<nucleation::SortStrategy>, nucleation::NucleationError> from_string(std::string_view s);

  /**
   * The strategy name.
   */
  inline std::string name() const;
  template<typename W>
  inline void name_write(W& writeable_output) const;

    inline const nucleation::capi::SortStrategy* AsFFI() const;
    inline nucleation::capi::SortStrategy* AsFFI();
    inline static const nucleation::SortStrategy* FromFFI(const nucleation::capi::SortStrategy* ptr);
    inline static nucleation::SortStrategy* FromFFI(nucleation::capi::SortStrategy* ptr);
    inline static void operator delete(void* ptr);
private:
    SortStrategy() = delete;
    SortStrategy(const nucleation::SortStrategy&) = delete;
    SortStrategy(nucleation::SortStrategy&&) noexcept = delete;
    SortStrategy operator=(const nucleation::SortStrategy&) = delete;
    SortStrategy operator=(nucleation::SortStrategy&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_SortStrategy_D_HPP
