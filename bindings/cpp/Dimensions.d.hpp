#ifndef Dimensions_D_HPP
#define Dimensions_D_HPP

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
    struct Dimensions {
      int32_t x;
      int32_t y;
      int32_t z;
    };

    typedef struct Dimensions_option {union { Dimensions ok; }; bool is_ok; } Dimensions_option;
    typedef struct DiplomatDimensionsView {
      const Dimensions* data;
      size_t len;
    } DiplomatDimensionsView;

    typedef struct DiplomatDimensionsViewMut {
      Dimensions* data;
      size_t len;
    } DiplomatDimensionsViewMut;
} // namespace capi
} // namespace


struct Dimensions {
    int32_t x;
    int32_t y;
    int32_t z;

    inline diplomat::capi::Dimensions AsFFI() const;
    inline static Dimensions FromFFI(diplomat::capi::Dimensions c_struct);
};


namespace diplomat {
    template<typename T>
    struct diplomat_c_span_convert<T, std::enable_if_t<std::is_same_v<T, span<const Dimensions>>>> {
        using type = capi::DiplomatDimensionsView;
    };

    template<typename T>
    struct diplomat_c_span_convert<T, std::enable_if_t<std::is_same_v<T, span<Dimensions>>>> {
        using type = capi::DiplomatDimensionsViewMut;
};
}
#endif // Dimensions_D_HPP
