#ifndef BlockPos_D_HPP
#define BlockPos_D_HPP

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


struct BlockPos {
    int32_t x;
    int32_t y;
    int32_t z;

    inline diplomat::capi::BlockPos AsFFI() const;
    inline static BlockPos FromFFI(diplomat::capi::BlockPos c_struct);
};


namespace diplomat {
    template<typename T>
    struct diplomat_c_span_convert<T, std::enable_if_t<std::is_same_v<T, span<const BlockPos>>>> {
        using type = capi::DiplomatBlockPosView;
    };

    template<typename T>
    struct diplomat_c_span_convert<T, std::enable_if_t<std::is_same_v<T, span<BlockPos>>>> {
        using type = capi::DiplomatBlockPosViewMut;
};
}
#endif // BlockPos_D_HPP
