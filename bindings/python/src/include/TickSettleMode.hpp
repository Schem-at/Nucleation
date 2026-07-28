#ifndef NUCLEATION_TickSettleMode_HPP
#define NUCLEATION_TickSettleMode_HPP

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


namespace nucleation {
namespace capi {

} // namespace capi
} // namespace

inline nucleation::capi::TickSettleMode nucleation::TickSettleMode::AsFFI() const {
    return static_cast<nucleation::capi::TickSettleMode>(value);
}

inline nucleation::TickSettleMode nucleation::TickSettleMode::FromFFI(nucleation::capi::TickSettleMode c_enum) {
    switch (c_enum) {
        case nucleation::capi::TickSettleMode_Placement:
        case nucleation::capi::TickSettleMode_Quiet:
        case nucleation::capi::TickSettleMode_InWorld:
            return static_cast<nucleation::TickSettleMode::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // NUCLEATION_TickSettleMode_HPP
