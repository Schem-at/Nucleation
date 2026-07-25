#ifndef GeneratedChunkCoverage_HPP
#define GeneratedChunkCoverage_HPP

#include "GeneratedChunkCoverage.d.hpp"

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

inline diplomat::capi::GeneratedChunkCoverage GeneratedChunkCoverage::AsFFI() const {
    return static_cast<diplomat::capi::GeneratedChunkCoverage>(value);
}

inline GeneratedChunkCoverage GeneratedChunkCoverage::FromFFI(diplomat::capi::GeneratedChunkCoverage c_enum) {
    switch (c_enum) {
        case diplomat::capi::GeneratedChunkCoverage_Complete:
        case diplomat::capi::GeneratedChunkCoverage_Partial:
        case diplomat::capi::GeneratedChunkCoverage_Outside:
            return static_cast<GeneratedChunkCoverage::Value>(c_enum);
        default:
            std::abort();
    }
}
#endif // GeneratedChunkCoverage_HPP
