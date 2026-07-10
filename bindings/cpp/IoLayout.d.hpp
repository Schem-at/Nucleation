#ifndef IoLayout_D_HPP
#define IoLayout_D_HPP

#include <stdio.h>
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <memory>
#include <functional>
#include <optional>
#include <cstdlib>
#include "diplomat_runtime.hpp"


namespace diplomat {
namespace capi {
    struct IoLayout;
} // namespace capi
} // namespace

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

    inline const diplomat::capi::IoLayout* AsFFI() const;
    inline diplomat::capi::IoLayout* AsFFI();
    inline static const IoLayout* FromFFI(const diplomat::capi::IoLayout* ptr);
    inline static IoLayout* FromFFI(diplomat::capi::IoLayout* ptr);
    inline static void operator delete(void* ptr);
private:
    IoLayout() = delete;
    IoLayout(const IoLayout&) = delete;
    IoLayout(IoLayout&&) noexcept = delete;
    IoLayout operator=(const IoLayout&) = delete;
    IoLayout operator=(IoLayout&&) noexcept = delete;
    static void operator delete[](void*, size_t) = delete;
};


#endif // IoLayout_D_HPP
