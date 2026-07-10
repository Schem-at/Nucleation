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

  inline static std::unique_ptr<SortStrategy> yxz();

  inline static std::unique_ptr<SortStrategy> xyz();

  inline static std::unique_ptr<SortStrategy> zyx();

  inline static std::unique_ptr<SortStrategy> y_desc_xz();

  inline static std::unique_ptr<SortStrategy> x_desc_yz();

  inline static std::unique_ptr<SortStrategy> z_desc_yx();

  /**
   * Old ABI name: `sort_strategy_descending`.
   */
  inline static std::unique_ptr<SortStrategy> descending();

  inline static std::unique_ptr<SortStrategy> distance_from(int32_t x, int32_t y, int32_t z);

  inline static std::unique_ptr<SortStrategy> distance_from_desc(int32_t x, int32_t y, int32_t z);

  inline static std::unique_ptr<SortStrategy> preserve();

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
