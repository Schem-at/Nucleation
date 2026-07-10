#ifndef NUCLEATION_RegionBounds_D_HPP
#define NUCLEATION_RegionBounds_D_HPP

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
    struct RegionBounds {
      int32_t min_x;
      int32_t min_y;
      int32_t min_z;
      int32_t max_x;
      int32_t max_y;
      int32_t max_z;
    };

    typedef struct RegionBounds_option {union { RegionBounds ok; }; bool is_ok; } RegionBounds_option;
    typedef struct DiplomatRegionBoundsView {
      const RegionBounds* data;
      size_t len;
    } DiplomatRegionBoundsView;

    typedef struct DiplomatRegionBoundsViewMut {
      RegionBounds* data;
      size_t len;
    } DiplomatRegionBoundsViewMut;
} // namespace capi
} // namespace


namespace nucleation {
/**
 * An inclusive block-coordinate box (a definition region is a union of
 * these).
 */
struct RegionBounds {
    int32_t min_x;
    int32_t min_y;
    int32_t min_z;
    int32_t max_x;
    int32_t max_y;
    int32_t max_z;

    inline nucleation::capi::RegionBounds AsFFI() const;
    inline static nucleation::RegionBounds FromFFI(nucleation::capi::RegionBounds c_struct);
};

} // namespace
namespace nucleation::diplomat {
    template<typename T>
    struct diplomat_c_span_convert<T, std::enable_if_t<std::is_same_v<T, span<const nucleation::RegionBounds>>>> {
        using type = nucleation::capi::DiplomatRegionBoundsView;
    };

    template<typename T>
    struct diplomat_c_span_convert<T, std::enable_if_t<std::is_same_v<T, span<nucleation::RegionBounds>>>> {
        using type = nucleation::capi::DiplomatRegionBoundsViewMut;
};
}
#endif // NUCLEATION_RegionBounds_D_HPP
