#ifndef NUCLEATION_BlockPos_D_HPP
#define NUCLEATION_BlockPos_D_HPP

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
    struct BlockPos {
      int32_t x;
      int32_t y;
      int32_t z;
    };

    typedef struct BlockPos_option {union { BlockPos ok; }; bool is_ok; } BlockPos_option;
    typedef struct DiplomatBlockPosView {
      const BlockPos* data;
      size_t len;
    } DiplomatBlockPosView;

    typedef struct DiplomatBlockPosViewMut {
      BlockPos* data;
      size_t len;
    } DiplomatBlockPosViewMut;
} // namespace capi
} // namespace


namespace nucleation {
struct BlockPos {
    int32_t x;
    int32_t y;
    int32_t z;

    inline nucleation::capi::BlockPos AsFFI() const;
    inline static nucleation::BlockPos FromFFI(nucleation::capi::BlockPos c_struct);
};

} // namespace
namespace nucleation::diplomat {
    template<typename T>
    struct diplomat_c_span_convert<T, std::enable_if_t<std::is_same_v<T, span<const nucleation::BlockPos>>>> {
        using type = nucleation::capi::DiplomatBlockPosView;
    };

    template<typename T>
    struct diplomat_c_span_convert<T, std::enable_if_t<std::is_same_v<T, span<nucleation::BlockPos>>>> {
        using type = nucleation::capi::DiplomatBlockPosViewMut;
};
}
#endif // NUCLEATION_BlockPos_D_HPP
