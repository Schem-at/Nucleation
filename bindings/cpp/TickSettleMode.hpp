#ifndef TickSettleMode_HPP
#define TickSettleMode_HPP

#include "TickSettleMode.d.hpp"

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

} // namespace capi
} // namespace

inline diplomat::capi::TickSettleMode TickSettleMode::AsFFI() const {
    return static_cast<diplomat::capi::TickSettleMode>(value);
}

inline TickSettleMode TickSettleMode::FromFFI(diplomat::capi::TickSettleMode c_enum) {
    switch (c_enum) {
        case diplomat::capi::TickSettleMode_Placement:
        case diplomat::capi::TickSettleMode_Quiet:
        case diplomat::capi::TickSettleMode_InWorld:
            return static_cast<TickSettleMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // TickSettleMode_HPP
