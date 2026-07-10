#ifndef NUCLEATION_IoLayout_D_HPP
#define NUCLEATION_IoLayout_D_HPP

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
namespace capi {
    struct IoLayout;
} // namespace capi
} // namespace

namespace nucleation {
/**
 * An immutable IO layout. Wraps
 * {@link crate::simulation::typed_executor::IoLayout}.
 */
class IoLayout {
public:

  /**
   * Input names as a JSON array string.
   */
  inline std::string input_names_json() const;
  template<typename W>
  inline void input_names_json_write(W& writeable_output) const;

  /**
   * Output names as a JSON array string.
   */
  inline std::string output_names_json() const;
  template<typename W>
  inline void output_names_json_write(W& writeable_output) const;

    inline const nucleation::capi::IoLayout* AsFFI() const;
    inline nucleation::capi::IoLayout* AsFFI();
    inline static const nucleation::IoLayout* FromFFI(const nucleation::capi::IoLayout* ptr);
    inline static nucleation::IoLayout* FromFFI(nucleation::capi::IoLayout* ptr);
    inline static void operator delete(void* ptr);
private:
    IoLayout() = delete;
    IoLayout(const nucleation::IoLayout&) = delete;
    IoLayout(nucleation::IoLayout&&) noexcept = delete;
    IoLayout operator=(const nucleation::IoLayout&) = delete;
    IoLayout operator=(nucleation::IoLayout&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};

} // namespace
#endif // NUCLEATION_IoLayout_D_HPP
