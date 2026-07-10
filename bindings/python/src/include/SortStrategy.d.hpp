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

  inline static std::unique_ptr<nucleation::SortStrategy> yxz();

  inline static std::unique_ptr<nucleation::SortStrategy> xyz();

  inline static std::unique_ptr<nucleation::SortStrategy> zyx();

  inline static std::unique_ptr<nucleation::SortStrategy> y_desc_xz();

  inline static std::unique_ptr<nucleation::SortStrategy> x_desc_yz();

  inline static std::unique_ptr<nucleation::SortStrategy> z_desc_yx();

  /**
   * Old ABI name: `sort_strategy_descending`.
   */
  inline static std::unique_ptr<nucleation::SortStrategy> descending();

  inline static std::unique_ptr<nucleation::SortStrategy> distance_from(int32_t x, int32_t y, int32_t z);

  inline static std::unique_ptr<nucleation::SortStrategy> distance_from_desc(int32_t x, int32_t y, int32_t z);

  inline static std::unique_ptr<nucleation::SortStrategy> preserve();

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
